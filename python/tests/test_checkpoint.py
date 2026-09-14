from datetime import datetime, timedelta
from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "checkpoint_core.inp"
)
RTK_FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "run"
    / "tests"
    / "data"
    / "regression-suite"
    / "complex"
    / "rtk.inp"
)
LID_FIXTURE = Path(__file__).resolve().parent / "data" / "lids.inp"


def _normalized_report(path: Path) -> str:
    return "\n".join(
        line
        for line in path.read_text(encoding="utf-8").splitlines()
        if not any(
            marker in line
            for marker in (
                "Analysis begun on:",
                "Analysis ended on:",
                "Total elapsed time:",
            )
        )
    )


def _finish_and_read_report(simulation: swmmrs.Simulation, report: Path) -> str:
    for _ in simulation:
        pass
    simulation.end()
    simulation.report()
    simulation.close()
    return _normalized_report(report)


def test_checkpoint_resume_state_load_and_fork_are_public_and_independent(
    tmp_path: Path,
) -> None:
    source = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "source.rpt",
        tmp_path / "source.out",
    )
    receiver = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "receiver.rpt",
        tmp_path / "receiver.out",
    )
    source.start()
    assert source.step() is not None

    checkpoint = tmp_path / "state.json"
    source.save_checkpoint(checkpoint)
    checkpoint_bytes = checkpoint.read_bytes()

    resumed = swmmrs.Simulation.resume(
        checkpoint,
        tmp_path / "resumed.rpt",
        tmp_path / "resumed.out",
    )
    child = source.fork(tmp_path / "child.rpt", tmp_path / "child.out")
    receiver.load_checkpoint_state(checkpoint)

    assert source.state is swmmrs.SimulationState.RUNNING
    assert resumed.state is swmmrs.SimulationState.RUNNING
    assert child.state is swmmrs.SimulationState.RUNNING
    assert receiver.state is swmmrs.SimulationState.OPEN
    assert source.step() == resumed.step() == child.step()
    assert checkpoint.read_bytes() == checkpoint_bytes

    blocked_report = tmp_path / "blocked.rpt"
    blocked_report.write_bytes(b"caller-owned")
    with pytest.raises(swmmrs.SolverError) as caught:
        swmmrs.Simulation.resume(
            checkpoint,
            blocked_report,
            tmp_path / "blocked.out",
        )
    assert caught.value.code is swmmrs.SolverErrorCode.CHECKPOINT_DESTINATION_VALIDATION
    assert blocked_report.read_bytes() == b"caller-owned"
    assert source.state is swmmrs.SimulationState.RUNNING
    for simulation in (source, resumed, child):
        simulation.end()
        simulation.close()
    receiver.close()


def test_checkpoint_resume_retains_editable_declarations_for_rerun(
    tmp_path: Path,
) -> None:
    source = swmmrs.Simulation(
        LID_FIXTURE,
        tmp_path / "declarations-source.rpt",
        tmp_path / "declarations-source.out",
    )
    source.start(save_results=False)
    assert source.step() is not None
    checkpoint = tmp_path / "declarations.json"
    source.save_checkpoint(checkpoint)

    resumed = swmmrs.Simulation.resume(
        checkpoint,
        tmp_path / "declarations-resumed.rpt",
        tmp_path / "declarations-resumed.out",
    )
    while resumed.step() is not None:
        pass
    resumed.end()

    unit = resumed.subcatchments["With-Lids"].lid_units[0]
    surface = resumed.lid_controls["BC"].surface
    assert surface is not None
    unit.area = 55.0
    surface.roughness = 0.2
    resumed.options.routing_step = timedelta(seconds=20)
    assert resumed.state is swmmrs.SimulationState.OPEN
    assert resumed.configuration_dirty
    assert unit.area == 55.0
    assert surface.roughness == 0.2

    resumed.start(save_results=False)
    assert not resumed.configuration_dirty
    while resumed.step() is not None:
        pass
    resumed.end()
    resumed.close()
    source.close()


def test_resume_rejects_retained_input_destination_before_publication(
    tmp_path: Path,
) -> None:
    source_input = tmp_path / "deleted-source.inp"
    source_input.write_bytes(FIXTURE.read_bytes())
    source = swmmrs.Simulation(
        source_input,
        tmp_path / "deleted-source.rpt",
        tmp_path / "deleted-source.out",
    )
    source.start()
    assert source.step() is not None
    checkpoint = tmp_path / "deleted-source.json"
    source.save_checkpoint(checkpoint)
    source.end()
    source.close()
    source_input.unlink()

    output = tmp_path / "collision.out"
    with pytest.raises(swmmrs.ValidationError):
        swmmrs.Simulation.resume(checkpoint, source_input, output)

    assert not source_input.exists()
    assert not output.exists()


def test_checkpoint_and_fork_collisions_use_validation_error(tmp_path: Path) -> None:
    hotstart = tmp_path / "configured.hsf"
    producer = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "producer.rpt",
        tmp_path / "producer.out",
    )
    producer.start(save_results=False)
    assert producer.step() is not None
    producer.save_hotstart(hotstart)
    producer.end()
    producer.close()

    source = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "collision-source.rpt",
        tmp_path / "collision-source.out",
    )
    source.use_hotstart(hotstart)
    source.start()

    with pytest.raises(swmmrs.ValidationError):
        source.save_checkpoint(hotstart)
    source_output = source.output_path
    assert source_output is not None
    for destination in (source.input_path, source.report_path, source_output, hotstart):
        child_output = tmp_path / f"child-{destination.name}.out"
        with pytest.raises(swmmrs.ValidationError):
            source.fork(destination, child_output)
        assert not child_output.exists()
    assert source.state is swmmrs.SimulationState.RUNNING
    source.end()
    source.close()


def test_load_checkpoint_state_retains_receiver_iterator_cadence(tmp_path: Path) -> None:
    source = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "cadence-source.rpt",
        tmp_path / "cadence-source.out",
    )
    source.start()
    assert source.step() is not None
    checkpoint = tmp_path / "cadence.json"
    source.save_checkpoint(checkpoint)

    receiver = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "cadence-receiver.rpt",
        tmp_path / "cadence-receiver.out",
    )
    receiver.step_advance(timedelta(minutes=1))
    expected = receiver.start_time + timedelta(minutes=1)
    receiver.load_checkpoint_state(checkpoint)

    assert next(receiver) == expected
    for simulation in (source, receiver):
        simulation.end()
        simulation.close()


def test_save_and_fork_are_allowed_between_iterator_advances(tmp_path: Path) -> None:
    source = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "iterator-source.rpt",
        tmp_path / "iterator-source.out",
    )
    source.step_advance(60)
    first = next(source)

    checkpoint = tmp_path / "iterator-state.json"
    source.save_checkpoint(checkpoint)
    resumed = swmmrs.Simulation.resume(
        checkpoint,
        tmp_path / "iterator-resumed.rpt",
        tmp_path / "iterator-resumed.out",
    )
    child = source.fork(
        tmp_path / "iterator-child.rpt",
        tmp_path / "iterator-child.out",
    )

    assert resumed.input_path == source.input_path == child.input_path
    assert resumed.report_path == tmp_path / "iterator-resumed.rpt"
    assert child.report_path == tmp_path / "iterator-child.rpt"

    source_next = next(source)
    resumed_next = next(resumed)
    child_next = next(child)
    assert source_next - first == timedelta(minutes=1)
    assert resumed_next == child_next
    assert resumed_next < source_next

    for simulation in (source, resumed, child):
        simulation.end()
        simulation.close()


@pytest.mark.skip
def test_rtk_checkpoint_fork_and_resume_reports_match(tmp_path: Path) -> None:
    source_report = tmp_path / "rtk-source.rpt"
    source = swmmrs.Simulation(
        RTK_FIXTURE,
        source_report,
        tmp_path / "rtk-source.out",
    )
    source.step_advance(3600)
    source.report_start = datetime(2015, 5, 31, 15)

    source.start_time = datetime(2015, 5, 31, 15)
    source.end_time = datetime(2015, 6, 2, 15)
    checkpoint_time = datetime(2015, 6, 1)
    model_time = next(time for time in source if time >= checkpoint_time)
    assert model_time == checkpoint_time

    checkpoint = tmp_path / "rtk-state.json"
    source.save_checkpoint(checkpoint)
    child_report = tmp_path / "rtk-child.rpt"
    child = source.fork(child_report, tmp_path / "rtk-child.out")
    child.step_advance(3600)

    source_data = _finish_and_read_report(source, source_report)
    child_data = _finish_and_read_report(child, child_report)

    resumed_report = tmp_path / "rtk-resumed.rpt"
    resumed = swmmrs.Simulation.resume(
        checkpoint,
        resumed_report,
        tmp_path / "rtk-resumed.out",
    )
    resumed.step_advance(3600)
    resumed_data = _finish_and_read_report(resumed, resumed_report)

    assert source_data == child_data == resumed_data
