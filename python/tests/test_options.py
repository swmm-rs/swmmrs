from __future__ import annotations

import copy
import math
import pickle
import threading
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs
from swmmrs.enums import (
    CustomEllipseModel,
    FlowUnits,
    InertiaDamping,
    NormalFlowLimit,
    SurchargeMethod,
    UnitSystem,
)
from swmmrs.objects import SimulationOptions

RUN_FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "test_ex1_metric.inp"
)
DYNAMIC_WAVE_FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "link_constantinflow.inp"
)
DYNAMIC_WAVE_METRIC_FIXTURE = RUN_FIXTURE.with_name("test_ex1_metric_dynwave.inp")

FIXTURE = Path(__file__).parent / "data" / "collections.inp"


def _simulation(directory: Path) -> swmmrs.Simulation:
    return swmmrs.Simulation(
        FIXTURE,
        directory / "model.rpt",
        directory / "model.out",
    )


def assert_validation_unchanged(
    simulation: swmmrs.Simulation,
    owner: object,
    name: str,
    value: object,
) -> None:
    before = getattr(owner, name)
    state = simulation.state
    with pytest.raises(swmmrs.ValidationError):
        setattr(owner, name, value)
    assert getattr(owner, name) == before
    assert simulation.state is state


def test_stable_option_and_unit_enum_inventory() -> None:
    assert [(member.name, member.value) for member in FlowUnits] == [
        ("CFS", "cfs"),
        ("GPM", "gpm"),
        ("MGD", "mgd"),
        ("CMS", "cms"),
        ("LPS", "lps"),
        ("MLD", "mld"),
    ]
    assert [(member.name, member.value) for member in UnitSystem] == [
        ("US", "us"),
        ("SI", "si"),
    ]
    assert [(member.name, member.value) for member in SurchargeMethod] == [
        ("EXTRAN", "extran"),
        ("SLOT", "slot"),
    ]
    assert [(member.name, member.value) for member in InertiaDamping] == [
        ("NONE", "none"),
        ("PARTIAL", "partial"),
        ("FULL", "full"),
    ]
    assert [(member.name, member.value) for member in NormalFlowLimit] == [
        ("SLOPE", "slope"),
        ("FROUDE", "froude"),
        ("BOTH", "both"),
        ("NEITHER", "neither"),
    ]
    assert [(member.name, member.value) for member in CustomEllipseModel] == [
        ("EPA_LEGACY", "epa_legacy"),
        ("TRUE_ELLIPSE", "true_ellipse"),
    ]


def test_time_units_and_every_option_are_typed(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    assert simulation.start_time == datetime(2020, 1, 1)
    assert simulation.report_start == datetime(2020, 1, 1)
    assert simulation.end_time == datetime(2020, 1, 1, 2)
    assert simulation.flow_units is FlowUnits.CFS
    assert simulation.unit_system is UnitSystem.US
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.current_time
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.elapsed_time

    options = simulation.options
    assert isinstance(options, SimulationOptions)
    representation = repr(options)
    for name, value in simulation._owner.options_read(simulation._generation()).items():
        assert f"{name}={value!r}" in representation
    assert options.routing_step == timedelta(seconds=30)
    assert options.maximum_routing_step == timedelta(seconds=30)
    assert options.report_step == timedelta(minutes=5)
    # Zero is a valid requested RULE_STEP and must remain visible before preparation.
    assert options.rule_step >= timedelta(0)
    assert options.minimum_routing_step > timedelta(0)
    assert options.lengthening_step >= timedelta(0)
    assert isinstance(options.antecedent_dry_duration, timedelta)
    assert options.detailed_reporting_enabled
    assert options.custom_ellipse_model is CustomEllipseModel.EPA_LEGACY
    assert options.surcharge_method is SurchargeMethod.EXTRAN
    assert not options.allow_ponding
    assert options.inertia_damping is InertiaDamping.PARTIAL
    assert options.normal_flow_limit is NormalFlowLimit.BOTH
    assert not options.skip_steady_state
    assert options.rainfall_enabled
    assert options.rdii_enabled
    assert options.snowmelt_enabled
    assert options.groundwater_enabled
    assert options.routing_enabled
    assert options.quality_enabled
    assert len(options.sweep_start) == 2
    assert len(options.sweep_end) == 2
    # Zero is the retained request for the canonical Dynamic Wave default.
    assert options.maximum_trials >= 0
    assert options.requested_threads >= 0
    assert simulation.effective_threads >= 1
    assert isinstance(options.courant_factor, float)
    assert isinstance(options.minimum_surface_area, float)
    assert isinstance(options.minimum_conduit_slope, float)
    assert isinstance(options.head_tolerance, float)
    assert isinstance(options.system_flow_tolerance, float)
    assert isinstance(options.lateral_flow_tolerance, float)
    simulation.close()


def test_date_relationships_are_validated_atomically(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    original = (
        simulation.start_time,
        simulation.report_start,
        simulation.end_time,
    )
    invalid_values = (
        date(2020, 1, 1),
        datetime(2020, 1, 1, tzinfo=timezone.utc),
        datetime(2020, 1, 1, 0, 0, 0, 1),
        True,
        0,
    )
    for name in ("start_time", "report_start", "end_time"):
        for value in invalid_values:
            with pytest.raises(swmmrs.ValidationError):
                setattr(simulation, name, value)
            assert (
                simulation.start_time,
                simulation.report_start,
                simulation.end_time,
            ) == original

    # Relational schedule edits are retained in arbitrary order and diagnosed only at start.
    simulation.start_time = original[2]
    assert simulation.configuration_dirty
    with pytest.raises(swmmrs.ConfigurationError) as caught:
        simulation.start()
    assert any(
        diagnostic.rule_code == "schedule.start_before_end"
        for diagnostic in caught.value.diagnostics
    )
    simulation.update_schedule(
        start_time=datetime(2020, 1, 1, 0, 30),
        report_start=datetime(2020, 1, 1, 1),
        end_time=datetime(2020, 1, 1, 1, 30),
    )
    assert simulation.start_time == datetime(2020, 1, 1, 0, 30)
    assert simulation.report_start == datetime(2020, 1, 1, 1)
    assert simulation.end_time == datetime(2020, 1, 1, 1, 30)
    simulation.close()


def test_non_midnight_whole_hour_run_preserves_full_duration(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        RUN_FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    midnight = simulation.start_time
    simulation.report_start = midnight + timedelta(hours=1)
    simulation.start_time = midnight + timedelta(minutes=30)
    simulation.end_time = midnight + timedelta(hours=1, minutes=30)
    simulation.start()
    while simulation.step() is not None:
        pass
    assert simulation.current_time == simulation.end_time
    assert simulation.elapsed_time == timedelta(hours=1)
    simulation.close()


def test_maximum_routing_step_read_does_not_change_dynamic_wave_results(tmp_path: Path) -> None:
    def first_step(read_maximum: bool) -> tuple[datetime | None, timedelta]:
        root = tmp_path / ("read" if read_maximum else "unread")
        root.mkdir()
        simulation = swmmrs.Simulation(
            DYNAMIC_WAVE_FIXTURE,
            root / "model.rpt",
            root / "model.out",
        )
        options = simulation.options
        options.routing_step = timedelta(seconds=30)
        options.minimum_routing_step = timedelta(seconds=2.5)
        options.courant_factor = 1.0
        # REPORT_STEP must satisfy the requested/base route step before start.
        options.report_step = timedelta(seconds=60)
        simulation.start()
        if read_maximum:
            assert options.maximum_routing_step > timedelta(0)
        result = simulation.step()
        elapsed = simulation.elapsed_time
        simulation.close()
        return result, elapsed

    assert first_step(True) == first_step(False)


def test_dynamic_wave_options_preserve_project_units_and_step_clamps(
    tmp_path: Path,
) -> None:
    simulation = swmmrs.Simulation(
        DYNAMIC_WAVE_METRIC_FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    options = simulation.options
    assert options.minimum_surface_area == pytest.approx(1.2, abs=1e-7, rel=0)
    assert options.head_tolerance == pytest.approx(0.015, abs=1e-7, rel=0)

    options.minimum_surface_area = 2.4
    assert options.minimum_surface_area == pytest.approx(2.4, abs=1e-7, rel=0)
    assert simulation.configuration_dirty
    options.minimum_routing_step = timedelta(seconds=120)
    assert options.minimum_routing_step == timedelta(seconds=120)
    assert simulation.configuration_dirty
    options.minimum_routing_step = timedelta(microseconds=1)
    assert options.minimum_routing_step == timedelta(microseconds=1)
    assert simulation.configuration_dirty

    simulation.start()
    assert options.minimum_surface_area == pytest.approx(2.4, abs=1e-7, rel=0)
    assert options.head_tolerance == pytest.approx(0.015, abs=1e-7, rel=0)
    simulation.close()


def test_duration_boundaries_preserve_intrinsic_validation_and_defer_coupling(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    options = simulation.options

    for name in (
        "routing_step",
        "minimum_routing_step",
        "lengthening_step",
        "antecedent_dry_duration",
        "report_step",
        "rule_step",
    ):
        for value in (True, 1, 1.0, "1", object()):
            assert_validation_unchanged(simulation, options, name, value)

    for name in ("routing_step", "report_step"):
        for value in (timedelta(0), timedelta(microseconds=-1)):
            assert_validation_unchanged(simulation, options, name, value)
    for name in ("minimum_routing_step", "lengthening_step", "rule_step"):
        assert_validation_unchanged(simulation, options, name, timedelta(microseconds=-1))

    for name in ("report_step", "rule_step"):
        assert_validation_unchanged(simulation, options, name, timedelta(seconds=1, microseconds=1))
        assert_validation_unchanged(simulation, options, name, timedelta(days=30_000))

    assert_validation_unchanged(
        simulation,
        options,
        "antecedent_dry_duration",
        timedelta(microseconds=-1),
    )
    options.antecedent_dry_duration = timedelta(microseconds=1)
    assert options.antecedent_dry_duration == timedelta(microseconds=1)
    options.minimum_routing_step = timedelta(seconds=2, microseconds=500_000)
    options.courant_factor = 1.5
    options.routing_step = timedelta(microseconds=1)
    assert options.routing_step == timedelta(microseconds=1)
    # Routing-step assignment keeps the existing requested Courant side effect;
    # minimum-step normalization occurs only during preparation.
    assert options.courant_factor == 0.0
    options.lengthening_step = timedelta(microseconds=1)
    assert options.lengthening_step == timedelta(microseconds=1)
    options.report_step = timedelta(seconds=1)
    options.rule_step = timedelta(seconds=2)
    assert options.report_step == timedelta(seconds=1)
    assert options.rule_step == timedelta(seconds=2)
    simulation.close()


def test_scalar_boolean_enum_sweep_and_thread_boundaries_are_atomic(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    options = simulation.options

    bool_names = (
        "detailed_reporting_enabled",
        "allow_ponding",
        "skip_steady_state",
        "rainfall_enabled",
        "rdii_enabled",
        "snowmelt_enabled",
        "groundwater_enabled",
        "routing_enabled",
        "quality_enabled",
    )
    for name in bool_names:
        assert_validation_unchanged(simulation, options, name, 1)
        setattr(options, name, not getattr(options, name))

    options.custom_ellipse_model = "TrUe_ElLiPsE"
    assert options.custom_ellipse_model is CustomEllipseModel.TRUE_ELLIPSE
    options.custom_ellipse_model = CustomEllipseModel.EPA_LEGACY
    for value in ("unknown", 1, SurchargeMethod.SLOT):
        assert_validation_unchanged(simulation, options, "custom_ellipse_model", value)

    options.surcharge_method = "SlOt"
    assert options.surcharge_method is SurchargeMethod.SLOT
    options.surcharge_method = SurchargeMethod.EXTRAN
    for value in ("unknown", 1, InertiaDamping.NONE):
        assert_validation_unchanged(simulation, options, "surcharge_method", value)
    options.inertia_damping = "FuLl"
    assert options.inertia_damping is InertiaDamping.FULL
    options.normal_flow_limit = "nEiThEr"
    assert options.normal_flow_limit is NormalFlowLimit.NEITHER
    options.inertia_damping = InertiaDamping.PARTIAL
    options.normal_flow_limit = NormalFlowLimit.BOTH
    for name, value in (
        ("inertia_damping", "unknown"),
        ("inertia_damping", " full"),
        ("inertia_damping", "ful"),
        ("normal_flow_limit", "unknown"),
        ("normal_flow_limit", " neither"),
        ("normal_flow_limit", "neith"),
        ("inertia_damping", SurchargeMethod.SLOT),
        ("normal_flow_limit", SurchargeMethod.SLOT),
    ):
        assert_validation_unchanged(simulation, options, name, value)

    for name in ("sweep_start", "sweep_end"):
        for value in (
            (2, 29),
            (0, 1),
            (13, 1),
            (1, 0),
            (True, 1),
            [1, 1],
            (1,),
            "1,1",
        ):
            assert_validation_unchanged(simulation, options, name, value)
    options.sweep_start = (2, 28)
    options.sweep_end = (12, 31)
    assert options.sweep_start == (2, 28)
    assert options.sweep_end == (12, 31)

    integer_cases = {
        "maximum_trials": (2, (-1, 1, True, 2.0, 2**31)),
        "requested_threads": (0, (-1, True, 1.0, 2**31)),
    }
    for name, (valid, invalids) in integer_cases.items():
        for invalid in invalids:
            assert_validation_unchanged(simulation, options, name, invalid)
        setattr(options, name, valid)
        assert getattr(options, name) == valid

    options.requested_threads = 2
    assert options.requested_threads == 2
    assert simulation.effective_threads in (1, 2)
    with pytest.raises(AttributeError):
        simulation.effective_threads = 2  # type: ignore[misc]

    numeric_cases = {
        "courant_factor": ((0.000001, 2.0), (0.0, -1.0, 2.000001)),
        "minimum_surface_area": ((0.0, 1.25), (-0.000001,)),
        "minimum_conduit_slope": ((0.0, 0.999999), (-0.000001, 1.0)),
    }
    for name, (valids, invalids) in numeric_cases.items():
        for invalid in (True, "1", math.nan, math.inf, -math.inf, *invalids):
            assert_validation_unchanged(simulation, options, name, invalid)
        for valid in valids:
            setattr(options, name, valid)
            assert getattr(options, name) == float(valid)
    simulation.close()


def test_options_are_generation_bound_and_not_copyable(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    options = simulation.options
    for operation in (copy.copy, copy.deepcopy, pickle.dumps):
        with pytest.raises(TypeError):
            operation(options)
    simulation.close()
    simulation.open(
        FIXTURE,
        tmp_path / "second.rpt",
        tmp_path / "second.out",
    )
    with pytest.raises(swmmrs.StaleViewError):
        _ = options.routing_step
    with pytest.raises(swmmrs.StaleViewError):
        options.routing_step = timedelta(seconds=1)
    simulation.close()


def test_atomic_option_updates_are_order_independent_and_deferred(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    options = simulation.options
    before_dirty = simulation.configuration_dirty
    options.update()
    assert simulation.configuration_dirty is before_dirty

    options.minimum_routing_step = timedelta(seconds=2.5)
    options.update(
        routing_step=timedelta(microseconds=1),
        courant_factor=1.25,
        rainfall_enabled=False,
    )
    assert options.routing_step == timedelta(microseconds=1)
    assert options.courant_factor == 1.25
    assert not options.rainfall_enabled

    options.update(courant_factor=0.75, routing_step=timedelta(seconds=12))
    assert options.routing_step == timedelta(seconds=12)
    assert options.courant_factor == 0.75

    before = (
        options.routing_step,
        options.minimum_routing_step,
        options.courant_factor,
        options.rainfall_enabled,
    )
    options.update(
        routing_step=timedelta(seconds=1),
        minimum_routing_step=timedelta(seconds=2),
    )
    assert options.routing_step == timedelta(seconds=1)
    assert options.minimum_routing_step == timedelta(seconds=2)
    assert simulation.configuration_dirty
    with pytest.raises(swmmrs.ValidationError):
        options.update(rainfall_enabled=True, maximum_trials=1)
    assert options.rainfall_enabled is before[3]
    with pytest.raises(swmmrs.ValidationError):
        options.update(maximum_routing_step=timedelta(seconds=1))
    simulation.close()


def test_atomic_schedule_update_stages_candidate_for_deferred_validation(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    start = datetime(2020, 1, 2, 3)
    report = datetime(2020, 1, 2, 3, 30)
    end = datetime(2020, 1, 2, 4)

    simulation.update_schedule(end_time=end, start_time=start, report_start=report)
    assert (simulation.start_time, simulation.report_start, simulation.end_time) == (
        start,
        report,
        end,
    )

    before = (simulation.start_time, simulation.report_start, simulation.end_time)
    simulation.update_schedule(start_time=end)
    assert simulation.start_time == end
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start()
    simulation.update_schedule(start_time=start, report_start=report, end_time=end)
    assert (simulation.start_time, simulation.report_start, simulation.end_time) == before
    with pytest.raises(swmmrs.ValidationError):
        simulation.update_schedule(start_time=datetime(2020, 1, 1, tzinfo=timezone.utc))
    with pytest.raises(swmmrs.ValidationError):
        simulation.update_schedule(end_time=datetime(2020, 1, 2, 5, microsecond=1))
    with pytest.raises(swmmrs.ValidationError):
        simulation.update_schedule(unknown=start)

    dirty = simulation.configuration_dirty
    simulation.update_schedule()
    assert simulation.configuration_dirty is dirty
    stale_options = simulation.options
    simulation.close()
    simulation.open(FIXTURE, tmp_path / "next.rpt", tmp_path / "next.out")
    with pytest.raises(swmmrs.StaleViewError):
        stale_options.update()
    simulation.close()


def test_concurrent_atomic_option_updates_preserve_disjoint_changes(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    options = simulation.options
    launch = threading.Barrier(3)
    errors: list[BaseException] = []

    def update(**changes: object) -> None:
        try:
            launch.wait(timeout=5.0)
            options.update(**changes)
        except BaseException as error:
            errors.append(error)

    threads = [
        threading.Thread(target=update, kwargs={"rainfall_enabled": False}),
        threading.Thread(target=update, kwargs={"quality_enabled": False}),
    ]
    for thread in threads:
        thread.start()
    launch.wait(timeout=5.0)
    for thread in threads:
        thread.join(timeout=5.0)
        assert not thread.is_alive()
    assert not errors
    assert not options.rainfall_enabled
    assert not options.quality_enabled
    simulation.close()


def test_options_repr_is_one_coherent_atomic_snapshot(tmp_path: Path) -> None:
    """Keep paired option values coherent while repr races atomic updates."""
    simulation = _simulation(tmp_path)
    options = simulation.options
    launch = threading.Barrier(2)
    finished = threading.Event()
    errors: list[object] = []

    def update_pairs() -> None:
        try:
            launch.wait(timeout=5.0)
            for index in range(500):
                value = index % 2 == 0
                options.update(rainfall_enabled=value, quality_enabled=value)
        except BaseException as error:
            errors.append(error)
        finally:
            finished.set()

    writer = threading.Thread(target=update_pairs)
    writer.start()
    launch.wait(timeout=5.0)
    observations = 0
    while not finished.is_set() or observations < 500:
        representation = repr(options)
        rainfall = "rainfall_enabled=True" in representation
        quality = "quality_enabled=True" in representation
        if rainfall != quality:
            errors.append(representation)
            break
        observations += 1
    writer.join(timeout=5.0)
    assert not writer.is_alive()
    assert not errors
    simulation.close()


def test_atomic_policy_lifecycle_empty_and_ended_rerun(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        RUN_FIXTURE,
        tmp_path / "atomic.rpt",
        tmp_path / "atomic.out",
    )
    options = simulation.options
    simulation.start()
    options.update()
    simulation.update_schedule()
    with pytest.raises(swmmrs.LifecycleError):
        options.update(requested_threads=1)
    with pytest.raises(swmmrs.LifecycleError):
        simulation.update_schedule(end_time=simulation.end_time + timedelta(hours=1))
    while simulation.step() is not None:
        pass
    simulation.end()

    options.update()
    simulation.update_schedule()
    assert simulation.state is swmmrs.SimulationState.ENDED
    simulation.update_schedule(end_time=simulation.end_time + timedelta(hours=1))
    assert simulation.state is swmmrs.SimulationState.OPEN
    simulation.start()
    while simulation.step() is not None:
        pass
    simulation.end()
    assert simulation.state is swmmrs.SimulationState.ENDED
    simulation.close()


def test_running_time_and_ended_configuration_transition(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        RUN_FIXTURE,
        tmp_path / "model.rpt",
        tmp_path / "model.out",
    )
    options = simulation.options
    simulation.start()
    assert simulation.current_time == simulation.start_time
    assert simulation.elapsed_time == timedelta(0)
    with pytest.raises(swmmrs.LifecycleError):
        options.requested_threads = 1
    while simulation.step() is not None:
        pass
    assert simulation.current_time == simulation.end_time
    assert simulation.elapsed_time == simulation.end_time - simulation.start_time
    simulation.end()
    assert simulation.state is swmmrs.SimulationState.ENDED
    assert simulation.report_period_count > 0

    options.requested_threads = 2
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert options.requested_threads == 2
    assert simulation.effective_threads in (1, 2)
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.current_time
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.report_period_count
    with pytest.raises(swmmrs.LifecycleError):
        simulation.report()
    simulation.close()
