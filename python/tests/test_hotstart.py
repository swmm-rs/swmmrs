from __future__ import annotations

from itertools import chain
from pathlib import Path

import pytest

import swmmrs

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "test_ex1_metric.inp"
)
COLD_OUTPUT_DIGEST = 0xD64922B3D87F1B36
CONTINUED_OUTPUT_DIGEST = 0x0B21E44B93B94BE6


class StringPath:
    def __init__(self, value: Path) -> None:
        self._value = value

    def __fspath__(self) -> str:
        return str(self._value)


class BytePath:
    def __fspath__(self) -> bytes:
        return b"checkpoint.hsf"


def output_digest(path: Path) -> int:
    """Hash every output byte except the package-derived solver version word."""
    data = path.read_bytes()
    assert len(data) >= 8, "output is too short for a binary version word"
    digest = 0xCBF29CE484222325
    for byte in chain(data[:4], data[8:]):
        digest ^= byte
        digest = digest * 0x100000001B3 & 0xFFFFFFFFFFFFFFFF
    return digest


def finish(simulation: swmmrs.Simulation) -> None:
    while simulation.stride(86400) is not None:
        pass


def _simulation(root: Path, name: str) -> swmmrs.Simulation:
    return swmmrs.Simulation(
        FIXTURE,
        root / f"{name}.rpt",
        root / f"{name}.out",
    )


def _checkpoint(root: Path) -> Path:
    checkpoint = root / "checkpoint.hsf"
    producer = _simulation(root, "producer")
    producer.start(save_results=False)
    assert producer.stride(86400) is not None
    producer.save_hotstart(checkpoint)
    producer.end()
    producer.close()
    assert checkpoint.stat().st_size > 15
    return checkpoint


def test_checkpoint_continues_real_fixture_and_persists_until_clear(tmp_path: Path) -> None:
    root = tmp_path
    checkpoint = _checkpoint(root)
    simulation = _simulation(root, "consumer")
    simulation.use_hotstart(StringPath(checkpoint))

    simulation.start()
    finish(simulation)
    simulation.end()
    simulation.report()
    first_digest = output_digest(root / "consumer.out")

    simulation.start()
    finish(simulation)
    simulation.end()
    simulation.report()
    assert output_digest(root / "consumer.out") == first_digest
    assert first_digest == CONTINUED_OUTPUT_DIGEST

    simulation.use_hotstart(None)
    simulation.start()
    finish(simulation)
    simulation.end()
    simulation.report()
    assert output_digest(root / "consumer.out") == COLD_OUTPUT_DIGEST
    simulation.close()


def test_successful_reopen_clears_generation_hotstart_configuration(tmp_path: Path) -> None:
    root = tmp_path
    checkpoint = _checkpoint(root)
    simulation = _simulation(root, "reopen")
    simulation.use_hotstart(checkpoint)
    simulation.close()
    simulation.open(
        FIXTURE,
        root / "reopened.rpt",
        root / "reopened.out",
    )
    simulation.start()
    finish(simulation)
    simulation.end()
    simulation.report()
    simulation.close()
    assert output_digest(root / "reopened.out") == COLD_OUTPUT_DIGEST


def test_paths_reject_bytes_and_same_owner_writable_collisions(tmp_path: Path) -> None:
    root = tmp_path
    simulation = _simulation(root, "paths")
    for value in (b"checkpoint.hsf", BytePath()):
        with pytest.raises(swmmrs.ValidationError):
            simulation.use_hotstart(value)  # type: ignore[arg-type]

    for value in (
        simulation.input_path,
        simulation.report_path,
        simulation.output_path,
    ):
        with pytest.raises(swmmrs.ValidationError):
            simulation.use_hotstart(value)

    checkpoint = _checkpoint(root)
    simulation.use_hotstart(checkpoint)
    simulation.start()
    for value in (b"checkpoint.hsf", BytePath()):
        with pytest.raises(swmmrs.ValidationError):
            simulation.save_hotstart(value)  # type: ignore[arg-type]
    for value in (
        simulation.input_path,
        simulation.report_path,
        simulation.output_path,
        checkpoint,
    ):
        with pytest.raises(swmmrs.ValidationError):
            simulation.save_hotstart(value)
    simulation.close()


def test_relative_path_is_resolved_at_configuration_time(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = tmp_path
    checkpoint = _checkpoint(root)
    consumer = _simulation(root, "relative")
    monkeypatch.chdir(root)
    consumer.use_hotstart(Path(".") / checkpoint.name)
    monkeypatch.chdir(root.parent)
    consumer.start(save_results=False)
    consumer.end()
    consumer.close()


def test_hotstart_failures_have_semantic_codes_and_native_stickiness(tmp_path: Path) -> None:
    root = tmp_path
    cases = (
        (root / "missing.hsf", swmmrs.SolverErrorCode.HOTSTART_FILE_OPEN),
        (root / "malformed.hsf", swmmrs.SolverErrorCode.HOTSTART_FILE_FORMAT),
        (root / "truncated.hsf", swmmrs.SolverErrorCode.HOTSTART_FILE_READ),
    )
    (root / "malformed.hsf").write_bytes(b"not a hotstart")
    (root / "truncated.hsf").write_bytes(b"SWMM5-HOTSTART4")
    for index, (path, expected) in enumerate(cases):
        simulation = _simulation(root, f"failure-{index}")
        simulation.use_hotstart(path)
        with pytest.raises(swmmrs.SolverError) as caught:
            simulation.start()
        assert caught.value.code == expected
        assert caught.value.operation == "start"
        assert simulation.state == swmmrs.SimulationState.FAILED
        simulation.close()


def test_immediate_save_open_and_write_failures_preserve_running_state(tmp_path: Path) -> None:
    root = tmp_path
    simulation = _simulation(root, "save-failure")
    simulation.start(save_results=False)
    with pytest.raises(swmmrs.SolverError) as caught:
        simulation.save_hotstart(root / "missing" / "checkpoint.hsf")
    assert caught.value.code == swmmrs.SolverErrorCode.HOTSTART_FILE_OPEN
    assert simulation.state == swmmrs.SimulationState.RUNNING

    if Path("/dev/full").exists():
        with pytest.raises(swmmrs.SolverError) as caught:
            simulation.save_hotstart("/dev/full")
        assert caught.value.code == swmmrs.SolverErrorCode.OUT_WRITE
        assert caught.value.semantic_code == "hotstart_file_write"
        assert simulation.state == swmmrs.SimulationState.RUNNING
    simulation.close()


@pytest.mark.skipif(not Path("/dev/full").exists(), reason="requires /dev/full")
def test_scheduled_write_failure_is_sticky_and_owner_local(tmp_path: Path) -> None:
    root = tmp_path
    scheduled = root / "scheduled.inp"
    scheduled.write_text(
        FIXTURE.read_text() + '\n[FILES]\nSAVE HOTSTART "/dev/full" 01/01/1998 00:00:01\n'
    )
    failed = swmmrs.Simulation(
        scheduled,
        root / "failed.rpt",
        root / "failed.out",
    )
    with pytest.raises(swmmrs.ValidationError):
        failed.use_hotstart("/dev/full")
    failed.start(save_results=False)
    with pytest.raises(swmmrs.ValidationError):
        failed.save_hotstart("/dev/full")

    with pytest.raises(swmmrs.SolverError) as first:
        failed.step()
    with pytest.raises(swmmrs.SolverError) as replay:
        failed.step()
    assert first.value.code == swmmrs.SolverErrorCode.OUT_WRITE
    assert first.value.semantic_code == "hotstart_file_write"
    assert replay.value.code == first.value.code
    assert replay.value.operation == first.value.operation
    assert replay.value.detail == first.value.detail
    assert failed.state == swmmrs.SimulationState.FAILED

    healthy = _simulation(root, "healthy")
    healthy.start(save_results=False)
    assert healthy.step() is not None
    healthy.close()
    failed.close()


def test_exact_lifecycle_windows_and_cross_owner_paths(tmp_path: Path) -> None:
    root = tmp_path
    shared = root / "shared.hsf"
    first = _simulation(root, "first")
    with pytest.raises(swmmrs.LifecycleError):
        first.save_hotstart(shared)
    first.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        first.use_hotstart(None)
    first.save_hotstart(shared)
    finish(first)
    first.save_hotstart(shared)
    first.end()
    with pytest.raises(swmmrs.LifecycleError):
        first.save_hotstart(shared)
    first.use_hotstart(None)
    assert first.state == swmmrs.SimulationState.OPEN
    with pytest.raises(swmmrs.LifecycleError):
        first.report()
    first.close()

    second = _simulation(root, "second")
    second.start(save_results=False)
    second.save_hotstart(shared)
    second.close()
    assert shared.is_file()

    scheduled_input = root / "scheduled-collision.inp"
    scheduled_input.write_text(FIXTURE.read_text() + '\n[FILES]\nSAVE HOTSTART "shared.hsf"\n')
    scheduled = swmmrs.Simulation(
        scheduled_input,
        root / "scheduled-collision.rpt",
        root / "scheduled-collision.out",
    )
    scheduled.start(save_results=False)
    finish(scheduled)
    scheduled.end()
    with pytest.raises(swmmrs.ValidationError):
        scheduled.use_hotstart(shared)
    assert scheduled.state == swmmrs.SimulationState.ENDED
    scheduled.close()
