from __future__ import annotations

# pyright: reportArgumentType=false, reportMissingImports=false, reportOptionalCall=false
import copy
import gc
import os
import pickle
import shutil
import threading
import warnings
from datetime import datetime, timedelta
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
EXPECTED_OUTPUT_DIGEST = 0xD64922B3D87F1B36


class StringPath:
    def __init__(self, value: Path) -> None:
        self._value = value

    def __fspath__(self) -> str:
        return str(self._value)


class ExplodingPath:
    def __fspath__(self) -> str:
        raise RuntimeError("path conversion failed")


def output_digest(path: Path) -> int:
    """Hash every output byte except the package-derived solver version word."""
    data = path.read_bytes()
    assert len(data) >= 8, "output is too short for a binary version word"
    digest = 0xCBF29CE484222325
    for byte in chain(data[:4], data[8:]):
        digest ^= byte
        digest = digest * 0x100000001B3 & 0xFFFFFFFFFFFFFFFF
    return digest


def resource_warnings(
    caught: list[warnings.WarningMessage],
) -> list[warnings.WarningMessage]:
    return [warning for warning in caught if issubclass(warning.category, ResourceWarning)]


def assert_artifacts_released(*paths: Path) -> None:
    for descriptor_directory in (Path("/proc/self/fd"), Path("/dev/fd")):
        if not descriptor_directory.is_dir():
            continue

        try:
            descriptors = list(descriptor_directory.iterdir())
        except OSError:
            continue

        open_targets = set()
        for descriptor in descriptors:
            try:
                target = os.readlink(descriptor).removesuffix(" (deleted)")
                open_targets.add(Path(target).resolve())
            except OSError:
                pass
        if open_targets:
            for path in paths:
                assert path.resolve() not in open_targets
            return

    if os.name == "nt":
        import ctypes
        from ctypes import wintypes

        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        create_file = kernel32.CreateFileW
        create_file.argtypes = [
            wintypes.LPCWSTR,
            wintypes.DWORD,
            wintypes.DWORD,
            wintypes.LPVOID,
            wintypes.DWORD,
            wintypes.DWORD,
            wintypes.HANDLE,
        ]
        create_file.restype = wintypes.HANDLE
        close_handle = kernel32.CloseHandle
        close_handle.argtypes = [wintypes.HANDLE]
        close_handle.restype = wintypes.BOOL
        invalid_handle = wintypes.HANDLE(-1).value
        open_existing = 3

        for path in paths:
            handle = create_file(
                str(path),
                0,
                0,
                None,
                open_existing,
                0,
                None,
            )
            if handle == invalid_handle:
                pytest.fail(
                    f"{path} is not exclusively openable: "
                    f"{ctypes.WinError(ctypes.get_last_error())}"
                )
            assert close_handle(handle), ctypes.WinError(ctypes.get_last_error())
        return

    pytest.skip("native file-descriptor inspection is unavailable")


def test_native_owner_class_is_not_in_package_namespace() -> None:
    assert not hasattr(swmmrs, "NativeSimulation")
    assert not hasattr(swmmrs, "_NativeSimulation")


def test_scalar_native_failures_map_to_public_internal_error(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    poison = getattr(simulation._owner, "_poison_for_test", None)
    if poison is None:
        pytest.skip("requires the test-support extension feature")
    poison()

    for name in (
        "state",
        "is_open",
        "is_started",
        "warning_count",
        "report_period_count",
    ):
        with pytest.raises(swmmrs.InternalSimulationError) as caught:
            getattr(simulation, name)
        assert type(caught.value) is swmmrs.InternalSimulationError
        assert caught.value.__cause__ is None
        assert len(caught.value.args) == 1
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", ResourceWarning)
        del poison, simulation
        gc.collect()


def test_context_cleanup_maps_poison_failures(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )

    with pytest.raises(swmmrs.InternalSimulationError) as caught:
        with simulation:
            poison = getattr(simulation._owner, "_poison_for_test", None)
            if poison is None:
                pytest.skip("requires the test-support extension feature")
            poison()

    assert caught.value.__cause__ is None
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", ResourceWarning)
        del poison, caught, simulation
        gc.collect()


def test_solver_failure_is_structured_and_sticky_until_close(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    simulation.start()
    force_failure = getattr(simulation._owner, "_force_timestep_error_for_test", None)
    if force_failure is None:
        pytest.skip("requires the test-support extension feature")
    force_failure()

    with pytest.raises(swmmrs.SolverError) as caught:
        simulation.step()

    assert type(caught.value) is swmmrs.SolverError
    assert caught.value.__cause__ is None
    assert caught.value.code == swmmrs.SolverErrorCode.TIMESTEP
    assert caught.value.operation == "step"
    assert caught.value.detail is None
    assert isinstance(caught.value.native_code, int)
    assert caught.value.semantic_code == "timestep"
    assert simulation.state == swmmrs.SimulationState.FAILED
    assert simulation.is_open
    assert not simulation.is_started
    assert isinstance(simulation.warning_count, int)
    assert simulation.input_path == FIXTURE.absolute()

    for operation in (
        simulation.start,
        simulation.step,
        simulation.end,
        simulation.report,
    ):
        with pytest.raises(swmmrs.SolverError) as replay:
            operation()
        assert replay.value.code == caught.value.code
        assert replay.value.operation == caught.value.operation
        assert replay.value.detail == caught.value.detail
        assert simulation.state == swmmrs.SimulationState.FAILED
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.report_period_count

    simulation.close()
    assert simulation.state == swmmrs.SimulationState.CLOSED


@pytest.mark.parametrize("method", ["step", "stride"])
def test_direct_advancement_failure_replays_once_through_iterator(
    tmp_path: Path, method: str
) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / f"{method}.rpt",
        tmp_path / f"{method}.out",
    )
    simulation.start()
    force_failure = getattr(simulation._owner, "_force_timestep_error_for_test", None)
    if force_failure is None:
        pytest.skip("requires the test-support extension feature")
    force_failure()

    with pytest.raises(swmmrs.SolverError) as direct:
        if method == "step":
            simulation.step()
        else:
            simulation.stride(60)
    with pytest.raises(swmmrs.SolverError) as replay:
        next(simulation)

    assert replay.value.code is direct.value.code
    assert replay.value.operation == direct.value.operation
    with pytest.raises(StopIteration):
        next(simulation)
    simulation.close()


def test_validation_and_lifecycle_failures_do_not_mutate_open_owner(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    with pytest.raises(swmmrs.ValidationError):
        simulation.start(save_results=1)
    assert simulation.state == swmmrs.SimulationState.OPEN
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.report_period_count

    with pytest.raises(swmmrs.LifecycleError):
        simulation.step()
    assert simulation.state == swmmrs.SimulationState.OPEN
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.report_period_count
    simulation.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        simulation.start(save_results=True)
    while simulation.step() is not None:
        pass
    assert simulation.report_period_count == 0
    simulation.end()
    simulation.close()


def test_status_is_one_immutable_lifecycle_snapshot(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "status.rpt")

    opened = simulation.status
    assert opened.state is swmmrs.SimulationState.OPEN
    assert opened.is_open
    assert not opened.is_started
    assert opened.current_time is None
    assert opened.elapsed_time == timedelta()
    assert opened.percent_complete == 0.0

    simulation.start(save_results=False)
    running = simulation.status
    assert running.state is swmmrs.SimulationState.RUNNING
    assert running.is_started
    assert not running.save_results
    assert running.current_time == simulation.start_time
    assert running.duration == simulation.end_time - simulation.start_time
    assert running.step_count == 0

    simulation.close()
    assert opened.state is swmmrs.SimulationState.OPEN


def test_native_failure_categories_have_direct_args_and_no_cause(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "categories.rpt")

    with pytest.raises(swmmrs.ValidationError) as validation:
        simulation.start(save_results=1)
    with pytest.raises(swmmrs.LifecycleError) as lifecycle:
        _ = simulation.report_period_count
    with pytest.raises(KeyError) as key:
        simulation.nodes["missing"]
    with pytest.raises(IndexError) as index:
        simulation.nodes.by_index(999)

    view = simulation.nodes["9"]
    simulation.close()
    with pytest.raises(swmmrs.StaleViewError) as stale:
        _ = view.id

    for caught in (validation, lifecycle, key, index, stale):
        assert caught.value.__cause__ is None
        assert len(caught.value.args) == 1
    assert key.value.args == ("missing",)
    assert index.value.args == (999,)
    assert validation.value.args == ("save_results must be bool",)
    assert lifecycle.value.args[0].startswith("report_period_count:")
    assert stale.value.args[0].startswith("validate_identity:")
    assert lifecycle.value.native_code is not None
    assert lifecycle.value.operation == "report_period_count"
    assert lifecycle.value.detail is not None


def test_reset_solver_reopens_ended_owner_and_rejects_running_owner(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "reset.rpt",
        tmp_path / "reset.out",
    )
    simulation.start(save_results=False)
    assert simulation.step() is not None
    simulation.end()

    simulation.reset_solver()
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert not simulation.configuration_dirty
    simulation.reset_solver()
    assert simulation.state is swmmrs.SimulationState.OPEN

    simulation.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        simulation.reset_solver()
    simulation.end()

    node = simulation.nodes["9"]
    initial_depth = node.settings.initial_depth
    invalid_depth = node.settings.full_depth + node.settings.surcharge_depth + 1.0
    node.settings.initial_depth = invalid_depth
    dirty_initial_depth = node.settings.initial_depth
    assert dirty_initial_depth == pytest.approx(invalid_depth)
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert simulation.configuration_dirty
    with pytest.raises(swmmrs.ConfigurationError) as caught:
        simulation.start(save_results=False)
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert simulation.configuration_dirty
    assert type(caught.value) is swmmrs.ConfigurationError
    assert caught.value.__cause__ is None
    assert len(caught.value.diagnostics) == 1
    diagnostic = caught.value.diagnostics[0]
    assert type(diagnostic) is swmmrs.ConfigurationDiagnostic
    assert type(diagnostic.object) is swmmrs.ConfigurationObjectIdentity
    assert diagnostic.object.object_type == "node"
    assert diagnostic.object.id == "9"
    assert diagnostic.object.index == 0
    assert diagnostic.property_path == "initial_depth"
    assert diagnostic.rule_code == "node.initial_depth.maximum_depth"
    assert diagnostic.message == "initial depth must not exceed full depth plus surcharge depth"
    assert diagnostic.conflicting_object is None
    assert caught.value.args == (
        "start: post-open preparation rejected 1 configuration violation(s)",
    )

    with pytest.raises(swmmrs.ConfigurationError) as repeated:
        simulation.start(save_results=False)
    assert repeated.value.args == caught.value.args
    assert repeated.value.diagnostics == caught.value.diagnostics
    assert repeated.value.__cause__ is None
    assert node.settings.initial_depth == dirty_initial_depth
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert simulation.configuration_dirty

    node.settings.initial_depth = initial_depth
    simulation.start(save_results=False)
    assert simulation.state is swmmrs.SimulationState.RUNNING
    assert not simulation.configuration_dirty
    simulation.end()
    simulation.close()


def test_native_panic_poison_is_owner_local_and_stable(tmp_path: Path) -> None:
    broken = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "broken.rpt",
        tmp_path / "broken.out",
    )
    healthy = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "healthy.rpt",
        tmp_path / "healthy.out",
    )
    panic = getattr(broken._owner, "_panic_for_test", None)
    if panic is None:
        pytest.skip("requires the test-support extension feature")

    with pytest.raises(swmmrs.InternalSimulationError):
        panic()
    with pytest.raises(swmmrs.InternalSimulationError):
        broken.start()
    with pytest.raises(swmmrs.InternalSimulationError):
        broken.close()

    healthy.start()
    assert healthy.step() is not None
    healthy.end()
    healthy.close()
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", ResourceWarning)
        del panic, broken
        gc.collect()


def test_construction_opens_string_pathlike_with_safe_defaults(tmp_path: Path) -> None:
    input_path = tmp_path / "model.inp"
    shutil.copyfile(FIXTURE, input_path)

    simulation = swmmrs.Simulation(StringPath(input_path))

    assert simulation.state == swmmrs.SimulationState.OPEN
    assert simulation.is_open
    assert not simulation.is_started
    assert simulation.input_path == input_path.absolute()
    assert simulation.report_path == input_path.with_suffix(".rpt").absolute()
    assert simulation.output_path is None
    assert simulation.report_path.is_file()
    with pytest.raises(TypeError):
        copy.copy(simulation)
    with pytest.raises(TypeError):
        copy.deepcopy(simulation)
    with pytest.raises(TypeError):
        pickle.dumps(simulation)
    with simulation as entered:
        assert entered is simulation
        assert simulation.state == swmmrs.SimulationState.OPEN
        with pytest.raises(swmmrs.LifecycleError):
            simulation.__enter__()
    assert simulation.state == swmmrs.SimulationState.CLOSED


def test_open_failure_exposes_semantic_solver_fields(tmp_path: Path) -> None:
    missing = tmp_path / "missing.inp"
    with pytest.raises(swmmrs.SolverError) as caught:
        swmmrs.Simulation(missing)

    assert caught.value.code == swmmrs.SolverErrorCode.INP_FILE
    assert caught.value.operation == "open"
    assert isinstance(caught.value.detail, str)


def test_byte_paths_are_rejected() -> None:
    with pytest.raises(swmmrs.ValidationError):
        swmmrs.Simulation(bytes(FIXTURE))
    with pytest.raises(RuntimeError, match="path conversion failed"):
        swmmrs.Simulation(ExplodingPath())  # type: ignore[arg-type]


def test_real_model_runs_complete_public_lifecycle(tmp_path: Path) -> None:
    report_path = tmp_path / "model.rpt"
    output_path = tmp_path / "model.out"
    simulation = swmmrs.Simulation(str(FIXTURE), report_path, output_path)

    with pytest.raises(swmmrs.LifecycleError):
        simulation.step()
    assert simulation.state == swmmrs.SimulationState.OPEN
    with pytest.raises(swmmrs.LifecycleError):
        simulation.end()
    assert simulation.state == swmmrs.SimulationState.OPEN

    simulation.start(save_results=True)
    assert simulation.state == swmmrs.SimulationState.RUNNING
    assert simulation.is_open
    assert simulation.is_started
    assert isinstance(simulation.warning_count, int)
    assert simulation.report_period_count == 0
    with pytest.raises(swmmrs.LifecycleError):
        simulation.start()
    assert simulation.state == swmmrs.SimulationState.RUNNING

    previous: datetime | None = None
    step_count = 0
    while True:
        current = simulation.step()
        step_count += 1
        if current is None:
            break
        assert current.tzinfo is None
        if previous is not None:
            assert current > previous
        previous = current

    assert step_count > 100
    assert simulation.state == swmmrs.SimulationState.COMPLETE
    assert simulation.is_started
    assert simulation.report_period_count > 0
    assert simulation.step() is None

    with pytest.raises(swmmrs.LifecycleError):
        simulation.report()
    assert simulation.state == swmmrs.SimulationState.COMPLETE

    simulation.end()
    simulation.end()
    assert simulation.state == swmmrs.SimulationState.ENDED
    assert not simulation.is_started
    with pytest.raises(swmmrs.LifecycleError):
        simulation.step()
    assert simulation.state == swmmrs.SimulationState.ENDED
    simulation.report()

    report = report_path.read_text()
    assert "Runoff Quantity Continuity" in report
    assert "Flow Routing Continuity" in report
    assert "Node Depth Summary" in report
    assert "Link Flow Summary" in report
    simulation.report()
    assert report_path.read_text() == report

    simulation.close()
    simulation.close()
    with pytest.raises(swmmrs.LifecycleError):
        simulation.end()
    assert simulation.state == swmmrs.SimulationState.CLOSED
    assert not simulation.is_open
    assert report_path.is_file()
    assert output_path.is_file()
    assert output_digest(output_path) == EXPECTED_OUTPUT_DIGEST


def test_save_results_false_keeps_report_period_count_zero(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    simulation.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        simulation.terminate()
    with pytest.raises(swmmrs.LifecycleError):
        simulation.execute()
    assert simulation.state == swmmrs.SimulationState.RUNNING
    assert simulation.is_open
    assert isinstance(simulation.stride(timedelta(hours=1)), datetime)
    while simulation.step() is not None:
        pass
    assert simulation.report_period_count == 0
    simulation.end()
    with pytest.raises(swmmrs.LifecycleError):
        simulation.report()
    assert simulation.state == swmmrs.SimulationState.ENDED
    with pytest.raises(StopIteration):
        next(simulation)
    simulation.start(save_results=False)
    assert isinstance(next(simulation), datetime)
    assert simulation.state == swmmrs.SimulationState.RUNNING
    simulation.close()


def test_stride_strict_modes(tmp_path: Path) -> None:
    stepped = swmmrs.Simulation(FIXTURE, tmp_path / "stepped.rpt")
    strided = swmmrs.Simulation(FIXTURE, tmp_path / "strided.rpt")
    strict_stride = swmmrs.Simulation(FIXTURE, tmp_path / "strict.rpt")
    stepped.start(save_results=False)
    strided.start(save_results=False)
    strict_stride.start(save_results=False)
    target = stepped.current_time + timedelta(hours=1)

    stepped_time = stepped.step()
    while stepped_time is not None and stepped_time < target:
        stepped_time = stepped.step()
    strided_time = strided.stride(
        timedelta(hours=1),
        strict=False,
    )

    assert strict_stride.stride(timedelta(hours=1), strict=True) == target
    assert strided_time == stepped_time
    assert strided.links["1"].flow == stepped.links["1"].flow
    assert strided.links["1"].depth == stepped.links["1"].depth
    with pytest.raises(swmmrs.ValidationError):
        strided.stride(60, strict=1)
    stepped.close()
    strided.close()
    strict_stride.close()


@pytest.mark.parametrize("strict", [True, False, None])
def test_step_advance_strict_modes(tmp_path: Path, strict: bool | None) -> None:
    with (
        swmmrs.Simulation(FIXTURE, tmp_path / "iterator.rpt") as simulation,
        swmmrs.Simulation(FIXTURE, tmp_path / "reference.rpt") as reference,
    ):
        simulation.start(save_results=False)
        reference.start(save_results=False)
        for duration in (61, timedelta(seconds=127)):
            if strict is None:
                simulation.step_advance(duration)
            else:
                simulation.step_advance(duration, strict=strict)
            for _ in range(3):
                expected = reference.stride(duration, strict=strict is not False)
                assert next(simulation) == expected
                assert simulation.links["1"].flow == reference.links["1"].flow
                assert simulation.links["1"].depth == reference.links["1"].depth
        # Clearing the cadence restores ordinary routing steps.
        simulation.step_advance(None, strict=False)
        assert next(simulation) == reference.step()
        # Omitting strict after non-strict configuration restores the default.
        simulation.step_advance(61, strict=False)
        simulation.step_advance(61)
        assert next(simulation) == reference.stride(61)


@pytest.mark.parametrize("strict", [None, 0, 1, "false", 1.0])
@pytest.mark.parametrize("duration", [60, None])
def test_step_advance_rejects_non_bool_strict(
    tmp_path: Path, strict: object, duration: int | None
) -> None:
    with swmmrs.Simulation(FIXTURE, tmp_path / "model.rpt") as simulation:
        with pytest.raises(swmmrs.ValidationError):
            simulation.step_advance(duration, strict=strict)


def test_iteration_uses_mutable_stride_and_terminates_at_checkpoint(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    for value in (
        True,
        0,
        -1,
        1.5,
        float("nan"),
        float("inf"),
        timedelta(microseconds=1),
        timedelta(days=30000),
    ):
        with pytest.raises(swmmrs.ValidationError):
            simulation.step_advance(value)
        with pytest.raises(swmmrs.ValidationError):
            simulation.stride(value)
    simulation.step_advance(3600)
    iterator = iter(simulation)
    first = next(iterator)
    simulation.step_advance(timedelta(hours=2))
    second = next(iterator)
    assert second - first == timedelta(hours=2)
    with pytest.raises(swmmrs.LifecycleError):
        simulation.execute()
    third = next(iterator)
    assert third - second == timedelta(hours=2)

    termination_errors: list[BaseException] = []

    def request_termination() -> None:
        try:
            simulation.terminate()
        except BaseException as error:
            termination_errors.append(error)

    requester = threading.Thread(target=request_termination)
    requester.start()
    requester.join()
    assert termination_errors == []
    other = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "other.rpt",
        tmp_path / "other.out",
    )
    other.step_advance(60)
    other_iterator = iter(other)
    next(other_iterator)
    assert isinstance(next(other_iterator), datetime)
    other.close()

    with pytest.raises(swmmrs.LifecycleError):
        simulation.step()
    with pytest.raises(swmmrs.LifecycleError):
        simulation.stride(60)
    simulation.terminate()
    simulation.terminate()
    with pytest.raises(StopIteration):
        next(iterator)
    assert simulation.state == swmmrs.SimulationState.ENDED
    with pytest.raises(StopIteration):
        next(iterator)
    simulation.report()
    simulation.close()


def test_execute_runs_full_lifecycle_and_closes(tmp_path: Path) -> None:
    report_path = tmp_path / "execute.rpt"
    output_path = tmp_path / "execute.out"
    simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)

    simulation.execute(save_results=True)

    assert simulation.state == swmmrs.SimulationState.CLOSED
    assert "Flow Routing Continuity" in report_path.read_text()
    assert output_digest(output_path) == EXPECTED_OUTPUT_DIGEST


def test_execute_without_saved_results_closes_scratch_output(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "execute-no-results.rpt")

    simulation.execute(save_results=False)

    assert simulation.state is swmmrs.SimulationState.CLOSED


def test_execute_restarts_an_ended_project_generation(tmp_path: Path) -> None:
    report_path = tmp_path / "restart.rpt"
    output_path = tmp_path / "restart.out"
    simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
    simulation.start(save_results=False)
    while simulation.stride(86400) is not None:
        pass
    simulation.end()

    simulation.execute(save_results=True)

    assert simulation.state == swmmrs.SimulationState.CLOSED
    assert "Flow Routing Continuity" in report_path.read_text()
    assert output_digest(output_path) == EXPECTED_OUTPUT_DIGEST


def test_execute_preserves_primary_failure_after_cleanup(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "failed.rpt",
        tmp_path / "failed.out",
    )
    force_failure = getattr(simulation._owner, "_force_timestep_error_for_test", None)
    if force_failure is None:
        pytest.skip("requires the test-support extension feature")
    force_failure()

    with pytest.raises(swmmrs.SolverError) as caught:
        simulation.execute()

    assert caught.value.code == swmmrs.SolverErrorCode.TIMESTEP
    assert caught.value.operation == "execute"
    assert caught.value.detail is None
    assert simulation.state == swmmrs.SimulationState.CLOSED


def test_project_generation_repeats_runs_with_per_run_result_policy(tmp_path: Path) -> None:
    report_path = tmp_path / "repeat.rpt"
    output_path = tmp_path / "repeat.out"
    simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
    simulation.step_advance(3600)

    simulation.start(save_results=False)
    first_run_time = next(simulation)
    for _ in simulation:
        pass
    assert simulation.state is swmmrs.SimulationState.COMPLETE
    with pytest.raises(StopIteration):
        next(simulation)
    with pytest.raises(StopIteration):
        next(simulation)
    assert simulation.report_period_count == 0
    simulation.end()
    with pytest.raises(swmmrs.LifecycleError):
        simulation.report()

    simulation.start(save_results=True)
    assert next(simulation) == first_run_time
    for _ in simulation:
        pass
    assert simulation.report_period_count > 0
    simulation.end()
    simulation.report()
    report = report_path.read_text()
    simulation.report()
    assert report_path.read_text() == report
    simulation.close()
    assert output_digest(output_path) == EXPECTED_OUTPUT_DIGEST


def test_reopen_replaces_only_successful_generation_paths_and_cadence(tmp_path: Path) -> None:
    root = tmp_path
    first_report = root / "first.rpt"
    first_output = root / "first.out"
    simulation = swmmrs.Simulation(FIXTURE, first_report, first_output)
    collision = root / "collision"
    with pytest.raises(swmmrs.LifecycleError):
        simulation.open(FIXTURE, collision, collision)
    simulation.step_advance(3600)
    first_time = next(simulation)
    simulation.close()
    assert first_output.is_file()

    retained_paths = (
        simulation.input_path,
        simulation.report_path,
        simulation.output_path,
    )
    missing_input = root / "missing.inp"
    with pytest.raises(swmmrs.SolverError) as caught:
        simulation.open(missing_input)
    assert str(missing_input) in str(caught.value)
    assert simulation.state == swmmrs.SimulationState.CLOSED
    assert (
        simulation.input_path,
        simulation.report_path,
        simulation.output_path,
    ) == retained_paths
    with pytest.raises(swmmrs.ValidationError) as caught:
        simulation.open(FIXTURE, collision, collision)
    assert str(collision) in str(caught.value)
    assert simulation.state == swmmrs.SimulationState.CLOSED
    assert (
        simulation.input_path,
        simulation.report_path,
        simulation.output_path,
    ) == retained_paths

    second_input = root / "second.inp"
    shutil.copyfile(FIXTURE, second_input)
    second_report = root / "second.rpt"
    simulation.open(second_input, second_report)
    assert simulation.input_path == second_input.absolute()
    assert simulation.report_path == second_report.absolute()
    assert simulation.output_path is None
    assert simulation.state == swmmrs.SimulationState.OPEN
    assert next(simulation) < first_time
    simulation.close()


def test_context_body_failure_remains_primary_when_cleanup_fails(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        FIXTURE,
        tmp_path / "body-failure.rpt",
        tmp_path / "body-failure.out",
    )

    with pytest.raises(ValueError, match="body failure") as caught:
        with simulation:
            poison = getattr(simulation._owner, "_poison_for_test", None)
            if poison is None:
                pytest.skip("requires the test-support extension feature")
            poison()
            raise ValueError("body failure")

    assert any(
        "Simulation cleanup failed with InternalSimulationError" in note
        for note in caught.value.__notes__
    )
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", ResourceWarning)
        del poison, caught, simulation
        gc.collect()


def test_report_flush_and_close_failures_use_public_exceptions(tmp_path: Path) -> None:
    full_device = Path("/dev/full")
    if not full_device.exists():
        pytest.skip("requires a deterministic write-failure device")
    simulation = swmmrs.Simulation(
        FIXTURE,
        full_device,
        tmp_path / "cleanup-failure.out",
    )
    view = simulation.nodes["9"]
    simulation.start()
    while simulation.stride(86400) is not None:
        pass
    simulation.end()

    with pytest.raises(swmmrs.SolverError) as report_failure:
        simulation.report()
    assert report_failure.value.operation == "report"
    assert report_failure.value.code == swmmrs.SolverErrorCode.RPT_FILE
    assert simulation.state == swmmrs.SimulationState.FAILED

    with pytest.raises(swmmrs.SolverError) as close_failure:
        simulation.close()
    assert close_failure.value.operation == "close"
    assert close_failure.value.code == swmmrs.SolverErrorCode.RPT_FILE
    assert simulation.state == swmmrs.SimulationState.CLOSED
    with pytest.raises(swmmrs.StaleViewError):
        _ = view.id


def test_context_ends_started_or_complete_run_and_owns_one_generation(tmp_path: Path) -> None:
    root = tmp_path
    simulation = swmmrs.Simulation(FIXTURE, root / "context.rpt")
    with simulation:
        next(simulation)
        simulation.terminate()
    assert simulation.state == swmmrs.SimulationState.CLOSED

    simulation.open(FIXTURE, root / "owned.rpt")
    with simulation:
        simulation.close()
        with pytest.raises(swmmrs.LifecycleError):
            simulation.open(FIXTURE, root / "other.rpt")

    simulation.open(FIXTURE, root / "complete.rpt")
    with simulation:
        for _ in simulation:
            pass
        assert simulation.state == swmmrs.SimulationState.COMPLETE
    assert simulation.state == swmmrs.SimulationState.CLOSED

    simulation.open(FIXTURE, root / "ended.rpt")
    simulation.start(save_results=False)
    simulation.end()
    with simulation as entered:
        assert entered is simulation
        assert simulation.state == swmmrs.SimulationState.ENDED
    assert simulation.state == swmmrs.SimulationState.CLOSED


def test_forgotten_open_owner_warns_and_releases_artifacts(tmp_path: Path) -> None:
    report_path = tmp_path / "forgotten-open.rpt"
    output_path = tmp_path / "forgotten-open.out"
    gc.collect()
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always", ResourceWarning)
        simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
        del simulation
        gc.collect()

    caught_resources = resource_warnings(caught)
    assert len(caught_resources) == 1
    assert "close" in str(caught_resources[0].message)
    assert_artifacts_released(report_path, output_path)


def test_forgotten_running_complete_failed_and_closed_owners_release_artifacts(
    tmp_path: Path,
) -> None:
    for target_state in (
        swmmrs.SimulationState.RUNNING,
        swmmrs.SimulationState.COMPLETE,
        swmmrs.SimulationState.FAILED,
        swmmrs.SimulationState.CLOSED,
    ):
        report_path = tmp_path / f"forgotten-{target_state}.rpt"
        output_path = tmp_path / f"forgotten-{target_state}.out"
        simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
        if target_state is swmmrs.SimulationState.CLOSED:
            simulation.close()
        else:
            simulation.start()
            if target_state is swmmrs.SimulationState.COMPLETE:
                while simulation.stride(86400) is not None:
                    pass
            elif target_state is swmmrs.SimulationState.FAILED:
                force_failure = getattr(simulation._owner, "_force_timestep_error_for_test", None)
                if force_failure is None:
                    pytest.skip("requires the test-support extension feature")
                force_failure()
                del force_failure
                with pytest.raises(swmmrs.SolverError):
                    simulation.step()
        assert simulation.state == target_state
        gc.collect()

        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always", ResourceWarning)
            del simulation
            gc.collect()

        assert len(resource_warnings(caught)) == (
            0 if target_state is swmmrs.SimulationState.CLOSED else 1
        )
        assert_artifacts_released(report_path, output_path)


def test_forgotten_poisoned_owner_cleanup_never_raises_or_leaks(tmp_path: Path) -> None:
    report_path = tmp_path / "forgotten-poisoned.rpt"
    output_path = tmp_path / "forgotten-poisoned.out"
    gc.collect()
    simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
    poison = getattr(simulation._owner, "_poison_for_test", None)
    if poison is None:
        pytest.skip("requires the test-support extension feature")
    poison()
    del poison

    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always", ResourceWarning)
        del simulation
        gc.collect()

    assert len(resource_warnings(caught)) == 1
    assert_artifacts_released(report_path, output_path)


def test_owner_retaining_view_delays_cleanup_without_creating_a_cycle(tmp_path: Path) -> None:
    class OwnerRetainingView:
        __slots__ = ("simulation",)

        def __init__(self, simulation: swmmrs.Simulation) -> None:
            self.simulation = simulation

    report_path = tmp_path / "retained-view.rpt"
    output_path = tmp_path / "retained-view.out"
    gc.collect()
    simulation = swmmrs.Simulation(FIXTURE, report_path, output_path)
    view = OwnerRetainingView(simulation)

    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always", ResourceWarning)
        del simulation
        gc.collect()
        assert caught == []
        del view
        gc.collect()

    assert len(resource_warnings(caught)) == 1
    assert_artifacts_released(report_path, output_path)
