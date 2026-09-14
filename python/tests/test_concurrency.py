from __future__ import annotations

import asyncio
import os
import re
import threading
from pathlib import Path
from typing import Callable, TypeVar, cast

import pytest  # type: ignore[import-not-found]

import swmmrs

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "test_ex1_metric.inp"
)
PARITY_FIXTURE = (
    Path(__file__).resolve().parents[2] / "crates" / "solver" / "benches" / "data" / "rtk.inp"
)
_T = TypeVar("_T")
_SYNC_TIMEOUT_SECONDS = 30.0
_RUN_TIMEOUT_SECONDS = 180.0


def _thread_call(
    operation: Callable[[], _T], result: list[_T], errors: list[BaseException]
) -> None:
    try:
        result.append(operation())
    except BaseException as error:
        errors.append(error)


def _test_hook(simulation: swmmrs.Simulation, name: str) -> Callable[..., object]:
    hook = getattr(simulation._owner, name, None)
    if hook is None:
        pytest.skip("requires the test-support extension feature")
    return cast(Callable[..., object], hook)


def _run_lifecycle(simulation: swmmrs.Simulation) -> tuple[str, bytes]:
    simulation.start()
    while simulation.step() is not None:
        pass
    simulation.end()
    simulation.report()
    report = simulation.report_path.read_text()
    simulation.close()
    output_path = simulation.output_path
    assert output_path is not None
    return report, output_path.read_bytes()


def _run_overlapping_lifecycles(
    simulations: list[swmmrs.Simulation],
) -> list[tuple[str, bytes]]:
    for simulation in simulations:
        _test_hook(simulation, "_arm_owner_pause_for_test")()

    launch = threading.Barrier(len(simulations) + 1)
    results: list[tuple[str, bytes]] = []
    errors: list[BaseException] = []

    def run(simulation: swmmrs.Simulation) -> tuple[str, bytes]:
        launch.wait(timeout=_SYNC_TIMEOUT_SECONDS)
        return _run_lifecycle(simulation)

    threads = [
        threading.Thread(
            target=_thread_call,
            args=(lambda simulation=simulation: run(simulation), results, errors),
            daemon=True,
        )
        for simulation in simulations
    ]
    for thread in threads:
        thread.start()

    synchronization_error: BaseException | None = None
    try:
        launch.wait(timeout=_SYNC_TIMEOUT_SECONDS)
        for simulation in simulations:
            reached = _test_hook(simulation, "_wait_owner_paused_for_test")(_SYNC_TIMEOUT_SECONDS)
            if not reached:
                raise AssertionError("owner lifecycle did not reach its native pause")

        unrelated_progress = threading.Event()
        progress_thread = threading.Thread(target=unrelated_progress.set, daemon=True)
        progress_thread.start()
        if not unrelated_progress.wait(timeout=_SYNC_TIMEOUT_SECONDS):
            raise AssertionError("unrelated Python thread made no progress")
        progress_thread.join(timeout=_SYNC_TIMEOUT_SECONDS)
        if progress_thread.is_alive():
            raise AssertionError("unrelated Python thread did not finish")
    except BaseException as error:
        synchronization_error = error
    finally:
        for simulation in simulations:
            _test_hook(simulation, "_release_owner_pause_for_test")()
        for thread in threads:
            thread.join(timeout=_RUN_TIMEOUT_SECONDS)

    if synchronization_error is not None:
        raise synchronization_error
    if any(thread.is_alive() for thread in threads):
        raise AssertionError("owner lifecycle thread did not finish")
    if errors:
        raise errors[0]
    return results


def _fnv1a64(data: bytes) -> int:
    digest = 0xCBF29CE484222325
    for byte in data:
        digest ^= byte
        digest = digest * 0x100000001B3 & 0xFFFFFFFFFFFFFFFF
    return digest


def _normalized_report(report: str, threads: int) -> str:
    line = f"  Number of Threads ........ {threads}"
    if report.count(line) != 1:
        raise AssertionError(f"report did not contain exactly one {line!r}")
    return report.replace(line, "  Number of Threads ........ <normalized>", 1)


def _write_parity_variant(source: Path, destination: Path, threads: int) -> None:
    variant = source.read_text()
    # Retain RTK's 12-way topology but run only five simulated minutes.
    for option, value in (
        ("END_DATE", "05/31/2015"),
        ("END_TIME", "10:05:00"),
        ("THREADS", str(threads)),
    ):
        variant, replacements = re.subn(
            rf"(?mi)^(\s*{option}\s+)\S+(\s*(?:;.*)?)$",
            rf"\g<1>{value}\g<2>",
            variant,
        )
        if replacements != 1:
            raise AssertionError(f"expected one {option} option, found {replacements}")
    destination.write_text(variant)


def test_same_owner_scalar_read_waits_without_blocking_python(tmp_path: Path) -> None:
    root = tmp_path
    simulation = swmmrs.Simulation(FIXTURE, root / "same.rpt", root / "same.out")
    _test_hook(simulation, "_arm_owner_pause_for_test")()
    start_results: list[None] = []
    start_errors: list[BaseException] = []
    start_thread = threading.Thread(
        target=_thread_call,
        args=(simulation.start, start_results, start_errors),
        daemon=True,
    )
    start_thread.start()
    read_thread: threading.Thread | None = None
    try:
        reached = _test_hook(simulation, "_wait_owner_paused_for_test")(_SYNC_TIMEOUT_SECONDS)
        assert reached, "start did not reach its native pause"

        _test_hook(simulation, "_arm_owner_probe_for_test")()
        read_results: list[swmmrs.SimulationState] = []
        read_errors: list[BaseException] = []
        read_thread = threading.Thread(
            target=_thread_call,
            args=(lambda: simulation.state, read_results, read_errors),
            daemon=True,
        )
        read_thread.start()
        probe_waiting = _test_hook(simulation, "_wait_owner_probe_waiting_for_test")(
            _SYNC_TIMEOUT_SECONDS
        )
        assert probe_waiting, "scalar read did not reach owner acquisition"
        assert not _test_hook(simulation, "_owner_probe_acquired_for_test")()

        unrelated_progress = threading.Event()
        progress_thread = threading.Thread(target=unrelated_progress.set, daemon=True)
        progress_thread.start()
        assert unrelated_progress.wait(timeout=_SYNC_TIMEOUT_SECONDS), (
            "unrelated Python thread made no progress"
        )
        progress_thread.join(timeout=_SYNC_TIMEOUT_SECONDS)
        assert not progress_thread.is_alive()
    finally:
        _test_hook(simulation, "_release_owner_pause_for_test")()
        start_thread.join(timeout=_SYNC_TIMEOUT_SECONDS)
        if read_thread is not None:
            read_thread.join(timeout=_SYNC_TIMEOUT_SECONDS)

    assert not start_thread.is_alive()
    assert read_thread is not None
    assert read_thread is not None
    assert not read_thread.is_alive()

    assert start_results == [None]
    assert start_errors == []
    assert read_results == [swmmrs.SimulationState.RUNNING]
    assert read_errors == []
    assert _test_hook(simulation, "_owner_probe_acquired_for_test")()
    simulation.end()
    simulation.close()


def test_distinct_owner_lifecycles_overlap_and_match_artifacts(tmp_path: Path) -> None:
    root = tmp_path
    simulations = [
        swmmrs.Simulation(FIXTURE, root / f"owner-{index}.rpt", root / f"owner-{index}.out")
        for index in range(2)
    ]
    results = _run_overlapping_lifecycles(simulations)

    assert len(results) == 2
    assert results[0][1] == results[1][1]
    assert results[0][0] == results[1][0]
    assert simulations[0].report_path != simulations[1].report_path
    assert simulations[0].output_path != simulations[1].output_path


def test_asyncio_to_thread_composes_the_synchronous_lifecycle(tmp_path: Path) -> None:
    root = tmp_path

    async def run(index: int) -> bytes:
        simulation = await asyncio.to_thread(
            swmmrs.Simulation,
            FIXTURE,
            root / f"async-{index}.rpt",
            root / f"async-{index}.out",
        )
        await asyncio.to_thread(simulation.start)
        while await asyncio.to_thread(simulation.step) is not None:
            pass
        await asyncio.to_thread(simulation.end)
        await asyncio.to_thread(simulation.report)
        await asyncio.to_thread(simulation.close)
        output_path = simulation.output_path
        assert output_path is not None
        return output_path.read_bytes()

    async def run_both() -> tuple[bytes, bytes]:
        first, second = await asyncio.gather(run(0), run(1))
        return first, second

    outputs = asyncio.run(run_both())
    assert outputs[0] == outputs[1]


def test_explicit_worker_sleep_wakes_next_threaded_step(tmp_path: Path) -> None:
    root = tmp_path
    input_path = root / "rtk.inp"
    _write_parity_variant(PARITY_FIXTURE, input_path, 4)
    simulation = swmmrs.Simulation(input_path, root / "model.rpt", root / "model.out")
    with pytest.raises(swmmrs.LifecycleError):
        simulation.sleep_workers()
    simulation.start()
    expected_threads = min(4, os.cpu_count() or 1)
    assert simulation.effective_threads == expected_threads
    iterator = iter(simulation)
    assert next(iterator) is not None

    simulation.sleep_workers()

    assert next(iterator) is not None
    simulation.terminate()
    with pytest.raises(StopIteration):
        next(iterator)
    simulation.close()


@pytest.mark.skip
def test_supported_thread_counts_and_process_cpu_budget_parity_gate(
    tmp_path: Path,
) -> None:
    process_cpu_budget = 8

    root = tmp_path
    support_probe = swmmrs.Simulation(
        FIXTURE, root / "support-probe.rpt", root / "support-probe.out"
    )
    try:
        _test_hook(support_probe, "_arm_owner_pause_for_test")
    finally:
        support_probe.close()
    variants: dict[int, Path] = {}
    reference_report: str | None = None
    reference_output: bytes | None = None

    for requested_threads in range(1, 13):
        variant = root / f"rtk-threads-{requested_threads}.inp"
        variants[requested_threads] = variant
        _write_parity_variant(PARITY_FIXTURE, variant, requested_threads)
        simulation = swmmrs.Simulation(
            variant,
            root / f"serial-{requested_threads}.rpt",
            root / f"serial-{requested_threads}.out",
        )
        report, output = _run_lifecycle(simulation)
        report = _normalized_report(report, requested_threads)
        if reference_report is None:
            reference_report = report
            reference_output = output
            assert len(output) == 6425
            assert _fnv1a64(output) == 14828365437482443081
            report_bytes = report.encode()
            assert len(report_bytes) == 774268
            assert _fnv1a64(report_bytes) == 16456147465769070337
        else:
            assert output == reference_output
            assert report == reference_report

    assert reference_report is not None
    assert reference_output is not None
    for simulation_count, requested_threads in ((2, 4), (4, 2)):
        assert simulation_count * requested_threads <= process_cpu_budget
        simulations = [
            swmmrs.Simulation(
                variants[requested_threads],
                root / f"budget-{simulation_count}x{requested_threads}-{index}.rpt",
                root / f"budget-{simulation_count}x{requested_threads}-{index}.out",
            )
            for index in range(simulation_count)
        ]
        assert len({simulation.report_path for simulation in simulations}) == simulation_count
        assert len({simulation.output_path for simulation in simulations}) == simulation_count
        for report, output in _run_overlapping_lifecycles(simulations):
            assert output == reference_output
            assert _normalized_report(report, requested_threads) == reference_report
