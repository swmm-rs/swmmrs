from __future__ import annotations

# pyright: reportMissingImports=false
import math
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

import swmmrs
import swmmrs.objects as objects
from swmmrs.enums import InfilKind, OutKind
from swmmrs.exceptions import ConfigurationError
from swmmrs.objects import (
    CurveNumberInfiltrationSettings,
    GreenAmptInfiltrationSettings,
    HortonInfiltrationSettings,
    Junction,
    ModifiedGreenAmptInfiltrationSettings,
    ModifiedHortonInfiltrationSettings,
    RainGage,
    Subcatchment,
    SubcatchmentSettings,
)

FIXTURE = Path(__file__).resolve().parent / "data" / "rain_subcatch.inp"
METRIC_FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "test_ex1_metric.inp"
)


def _simulation(directory: Path, name: str = "model") -> swmmrs.Simulation:
    return swmmrs.Simulation(
        FIXTURE,
        directory / f"{name}.rpt",
        directory / f"{name}.out",
    )


def assert_invalid_unchanged(
    simulation: swmmrs.Simulation, owner: object, name: str, value: object
) -> None:
    before = getattr(owner, name)
    state = simulation.state
    with pytest.raises(swmmrs.ValidationError):
        setattr(owner, name, value)
    assert getattr(owner, name) == before
    assert simulation.state is state


def test_rain_gage_forcing_precedence_clear_unlink_and_repeat_run(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    gage = simulation.rain_gages["gage-b"]
    assert isinstance(gage, RainGage)

    with pytest.raises(swmmrs.LifecycleError):
        _ = gage.total_precip
    assert gage.external_precipitation_rate is None

    gage.use_external_precipitation(1.25)
    gage.rainfall_override = 2.5
    settings = simulation.subcatchments["Sub-To-Sub"].settings
    settings.rain_gage = gage
    assert settings.rain_gage == gage
    assert simulation.configuration_dirty
    assert gage.rainfall_override == 2.5
    assert gage.external_precipitation_rate == 1.25
    assert_invalid_unchanged(simulation, gage, "rainfall_override", -0.1)
    assert_invalid_unchanged(simulation, gage, "rainfall_override", math.nan)
    assert_invalid_unchanged(simulation, gage, "rainfall_override", True)
    for invalid in (-1, math.inf, True):
        before = gage.external_precipitation_rate
        state = simulation.state
        with pytest.raises(swmmrs.ValidationError):
            gage.use_external_precipitation(invalid)
        assert gage.external_precipitation_rate == before
        assert simulation.state is state
    simulation.start(save_results=False)
    assert gage.total_precip == 2.5
    assert gage.rainfall == 2.5
    assert gage.snowfall == 0.0
    simulation.step()
    assert gage.total_precip == 2.5
    assert gage.rainfall == 2.5
    assert gage.snowfall == 0.0

    gage.rainfall_override = None
    assert gage.rainfall_override is None
    simulation.stride(60)
    assert gage.total_precip == 1.25
    assert gage.rainfall == 1.25
    simulation.end()
    assert gage.total_precip == 1.25

    simulation.start(save_results=False)
    assert gage.total_precip == 1.25
    assert gage.rainfall == 1.25
    assert gage.snowfall == 0.0
    simulation.step()
    assert gage.total_precip == 1.25
    simulation.end()
    gage.rainfall_override = 0.75
    assert simulation.state is swmmrs.SimulationState.OPEN
    simulation.start(save_results=False)
    assert gage.total_precip == 0.75
    assert gage.rainfall == 0.75
    assert gage.snowfall == 0.0
    simulation.step()
    assert gage.total_precip == 0.75
    simulation.end()
    simulation.start(save_results=False)
    simulation.step()
    assert gage.total_precip == 0.75
    simulation.end()
    simulation.close()


def test_relationship_edit_matches_parsed_short_interval_gage_rainfall(
    tmp_path: Path,
) -> None:
    source = FIXTURE.read_text()
    candidate_source = (
        source.replace("Sub-Self   Gage-B Sub-Self", "Sub-Self   Gage-A Sub-Self")
        .replace(
            "Gage-B INTENSITY 0:01 1.0 TIMESERIES RainSeries",
            "Gage-B INTENSITY 0:00:30 1.0 TIMESERIES RainSeriesFast",
        )
        .replace(
            "RainSeries 00:20 0",
            "RainSeries 00:20 0\n"
            "RainSeriesFast 00:00 0\n"
            "RainSeriesFast 00:01 0.2\n"
            "RainSeriesFast 00:20 0",
        )
    )
    reference_source = candidate_source.replace(
        "Sub-To-Sub Gage-A Sub-Node",
        "Sub-To-Sub Gage-B Sub-Node",
    )
    candidate_input = tmp_path / "candidate-relationship.inp"
    reference_input = tmp_path / "reference-relationship.inp"
    candidate_input.write_text(candidate_source)
    reference_input.write_text(reference_source)

    def run_trace(
        input_path: Path, role: str, edit: bool
    ) -> tuple[list[tuple[float, float]], int]:
        simulation = swmmrs.Simulation(
            input_path,
            tmp_path / f"{role}.rpt",
            tmp_path / f"{role}.out",
        )
        subcatchment = simulation.subcatchments["Sub-To-Sub"]
        if edit:
            subcatchment.settings.rain_gage = simulation.rain_gages["Gage-B"]
            assert simulation.configuration_dirty
            assert subcatchment.settings.rain_gage == simulation.rain_gages["Gage-B"]
        simulation.start(save_results=False)
        trace: list[tuple[float, float]] = []
        for _ in range(12):
            if simulation.step() is None:
                break
            trace.append((subcatchment.rainfall, subcatchment.runoff))
        warning_count = simulation.warning_count
        simulation.end()
        simulation.close()
        return trace, warning_count

    candidate_trace, candidate_warnings = run_trace(candidate_input, "candidate", True)
    reference_trace, reference_warnings = run_trace(reference_input, "reference", False)
    assert candidate_trace == pytest.approx(reference_trace, abs=1e-12, rel=0)
    assert any(rainfall > 0.0 for rainfall, _ in candidate_trace)
    # Initial-open WARN09/WARN01 are retained in the warning baseline, so the
    # equivalent prepared declaration does not replay duplicate report effects.
    assert candidate_warnings == 2
    assert reference_warnings == 2


def test_source_switch_and_coupled_scale_forcing_use_named_operations(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "named-forcing")
    gage = simulation.rain_gages["Gage-A"]
    subcatchment = simulation.subcatchments["Sub-To-Sub"]

    with pytest.raises(AttributeError):
        gage.external_precipitation_rate = 1.0  # type: ignore[misc]
    with pytest.raises(AttributeError):
        subcatchment.rain_scale_factor = 1.0  # type: ignore[misc]
    with pytest.raises(AttributeError):
        subcatchment.snow_scale_factor = 1.0  # type: ignore[misc]

    gage.use_external_precipitation(rate=1.25)
    assert gage.external_precipitation_rate == 1.25
    subcatchment.set_precipitation_scale_factors(rainfall=1.25, snowfall=0.75)
    assert subcatchment.rain_scale_factor == 1.25
    assert subcatchment.snow_scale_factor == 0.75

    before = (subcatchment.rain_scale_factor, subcatchment.snow_scale_factor)
    with pytest.raises(swmmrs.ValidationError):
        subcatchment.set_precipitation_scale_factors(rainfall=2.0, snowfall=0.0)
    assert [subcatchment.rain_scale_factor, subcatchment.snow_scale_factor] == list(before)

    candidates = ((2.0, 3.0), (4.0, 5.0))
    with ThreadPoolExecutor(max_workers=2) as executor:
        list(
            executor.map(
                lambda values: subcatchment.set_precipitation_scale_factors(
                    rainfall=values[0], snowfall=values[1]
                ),
                candidates,
            )
        )
    assert [subcatchment.rain_scale_factor, subcatchment.snow_scale_factor] in [
        list(candidate) for candidate in candidates
    ]

    simulation.start(save_results=False)
    gage.use_external_precipitation(rate=1.5)
    subcatchment.set_precipitation_scale_factors(rainfall=1.5, snowfall=2.5)
    simulation.end()
    gage.use_external_precipitation(rate=1.75)
    assert simulation.state is swmmrs.SimulationState.OPEN
    simulation.close()


def test_subcatchment_exposes_stable_declarations_only_through_settings(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "settings-only")
    subcatchment = simulation.subcatchments["Sub-To-Sub"]

    for name in (
        "configuration",
        "configure",
        "tag",
        "area",
        "rain_gage",
        "included_in_report",
        "width",
        "slope",
        "curb_length",
        "impervious_fraction",
        "zero_impervious_fraction",
        "impervious_roughness",
        "pervious_roughness",
        "impervious_depression_storage",
        "pervious_depression_storage",
        "outlet",
        "outlet_kind",
        "infiltration_configuration",
        "groundwater_configuration",
        "groundwater",
        "snowpack",
        "initial_buildup",
        "coverage_fractions",
    ):
        assert not hasattr(subcatchment, name)

    assert subcatchment.settings.width == pytest.approx(75.0)
    for name in (
        "SubcatchmentConfiguration",
        "SubcatchmentGroundwaterConfiguration",
        "SubcatchmentSnowpack",
        "SubcatchmentSnowpackConfiguration",
        "HortonInfiltrationConfiguration",
        "ModifiedHortonInfiltrationConfiguration",
        "GreenAmptInfiltrationConfiguration",
        "ModifiedGreenAmptInfiltrationConfiguration",
        "CurveNumberInfiltrationConfiguration",
    ):
        assert not hasattr(objects, name)
    assert dict(subcatchment.settings.initial_buildup) == {}
    assert dict(subcatchment.settings.coverage_fractions) == {}
    simulation.close()


def test_subcatchment_kinds_accept_literals_and_return_enums(tmp_path: Path) -> None:
    assert [kind.value for kind in InfilKind] == [
        "horton",
        "modified_horton",
        "green_ampt",
        "modified_green_ampt",
        "curve_number",
    ]
    assert [kind.value for kind in OutKind] == ["node", "subcatchment"]
    assert InfilKind("horton") is InfilKind.HORTON
    assert InfilKind(InfilKind.GREEN_AMPT) is InfilKind.GREEN_AMPT
    assert OutKind("node") is OutKind.NODE
    assert OutKind(OutKind.SUBCATCHMENT) is OutKind.SUBCATCHMENT

    simulation = _simulation(tmp_path, "kind-enums")
    upstream = simulation.subcatchments["Sub-To-Sub"]
    downstream = simulation.subcatchments["Sub-Node"]

    assert upstream.settings.outlet_kind is OutKind.SUBCATCHMENT
    assert downstream.settings.outlet_kind is OutKind.NODE

    settings = upstream.settings.infiltration
    assert settings is not None
    assert settings.kind is InfilKind.HORTON
    simulation.close()

def test_absent_infiltration_is_none(tmp_path: Path) -> None:
    text = FIXTURE.read_text()
    start = text.index("[INFILTRATION]")
    end = text.index("\n[", start)
    input_path = tmp_path / "no-infiltration.inp"
    input_path.write_text(text[:start] + text[end:])

    simulation = swmmrs.Simulation(input_path, tmp_path / "no-infiltration.rpt")
    assert simulation.subcatchments["Sub-To-Sub"].settings.infiltration is None
    simulation.close()



def test_subcatchment_settings_is_a_truthful_live_view_with_atomic_update(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "settings-update")
    settings = simulation.subcatchments["Sub-To-Sub"].settings

    assert isinstance(settings, SubcatchmentSettings)
    assert settings.width == pytest.approx(75.0)
    assert settings.slope == pytest.approx(0.02)
    settings.update()
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.ValidationError):
        settings.update(width=True)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(unknown=1.0)
    assert not simulation.configuration_dirty

    settings.update(
        tag="calibration",
        area=1.0,
        rain_gage=simulation.rain_gages["Gage-A"],
        included_in_report=False,
        width=100.0,
        slope=0.03,
        curb_length=5.0,
        impervious_fraction=0.4,
        zero_impervious_fraction=0.2,
        impervious_roughness=0.01,
        pervious_roughness=0.1,
        impervious_depression_storage=0.05,
        pervious_depression_storage=0.06,
        outlet=simulation.subcatchments["Sub-Node"],
    )
    assert settings.tag == "calibration"
    assert settings.area == pytest.approx(1.0)
    assert settings.rain_gage == simulation.rain_gages["Gage-A"]
    assert not settings.included_in_report
    assert settings.width == pytest.approx(100.0)
    assert settings.slope == pytest.approx(0.03)
    assert settings.curb_length == pytest.approx(5.0)
    assert settings.impervious_fraction == pytest.approx(0.4)
    assert settings.zero_impervious_fraction == pytest.approx(0.2)
    assert settings.impervious_roughness == pytest.approx(0.01)
    assert settings.pervious_roughness == pytest.approx(0.1)
    assert settings.impervious_depression_storage == pytest.approx(0.05)
    assert settings.pervious_depression_storage == pytest.approx(0.06)
    assert settings.outlet == simulation.subcatchments["Sub-Node"]

    with pytest.raises(swmmrs.ValidationError):
        settings.update(width=125.0, slope=True)
    assert settings.width == pytest.approx(100.0)
    assert settings.slope == pytest.approx(0.03)


def test_live_infiltration_views_follow_same_kind_and_reject_wrong_kind(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "infiltration-settings")
    settings = simulation.subcatchments["Sub-To-Sub"].settings
    retained = settings.infiltration
    assert retained.kind is InfilKind.HORTON
    retained.update()
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.ValidationError):
        retained.update(initial_rate=True)
    with pytest.raises(swmmrs.ValidationError):
        retained.update(unknown=1.0)
    assert not simulation.configuration_dirty

    same_kind = HortonInfiltrationSettings(3.2, 0.5, 4.0, 7.0, 1.5)
    settings.infiltration = same_kind
    assert [
        retained.initial_rate,
        retained.minimum_rate,
        retained.decay_coefficient,
        retained.drying_time_days,
        retained.maximum_infiltration,
    ] == pytest.approx([3.2, 0.5, 4.0, 7.0, 1.5])
    for field, value in {
        "initial_rate": 3.4,
        "minimum_rate": 0.6,
        "decay_coefficient": 4.5,
        "drying_time_days": 8.0,
        "maximum_infiltration": 1.6,
    }.items():
        setattr(retained, field, value)
        assert getattr(settings.infiltration, field) == pytest.approx(value)
    assert same_kind.initial_rate == pytest.approx(3.2)
    with pytest.raises(swmmrs.ValidationError):
        retained.update(initial_rate=3.6, minimum_rate=-1.0)
    assert retained.initial_rate == pytest.approx(3.4)
    assert retained.minimum_rate == pytest.approx(0.6)
    retained.maximum_infiltration = None
    assert retained.maximum_infiltration is None

    variants = (
        ModifiedHortonInfiltrationSettings(3.1, 0.5, 4.0, 7.0),
        GreenAmptInfiltrationSettings(3.5, 0.5, 0.2),
        ModifiedGreenAmptInfiltrationSettings(3.5, 0.5, 0.2),
        CurveNumberInfiltrationSettings(75.0, 7.0),
    )
    for replacement in variants:
        previous = settings.infiltration
        settings.infiltration = replacement
        current = settings.infiltration
        assert current.kind is replacement.kind
        if isinstance(replacement, ModifiedHortonInfiltrationSettings):
            for field, value in {
                "initial_rate": 3.3,
                "minimum_rate": 0.6,
                "decay_coefficient": 4.5,
                "drying_time_days": 8.0,
            }.items():
                setattr(current, field, value)
                assert getattr(current, field) == pytest.approx(value)
        elif isinstance(
            replacement,
            (GreenAmptInfiltrationSettings, ModifiedGreenAmptInfiltrationSettings),
        ):
            for field, value in {
                "suction_head": 3.6,
                "hydraulic_conductivity": 0.6,
                "initial_moisture_deficit": 0.25,
            }.items():
                setattr(current, field, value)
                assert getattr(current, field) == pytest.approx(value)
        else:
            current.curve_number = 80.0
            current.drying_time_days = 8.0
            assert current.curve_number == pytest.approx(80.0)
            assert current.drying_time_days == pytest.approx(8.0)
        with pytest.raises(swmmrs.StaleViewError):
            _ = previous.kind
        with pytest.raises(swmmrs.StaleViewError):
            previous.update()

    with pytest.raises(swmmrs.StaleViewError):
        retained.initial_rate = 4.0

    curve_number = settings.infiltration
    curve_number.curve_number = 80.0
    assert curve_number.curve_number == pytest.approx(80.0)
    simulation.close()


def test_retained_root_settings_view_reads_current_owner(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "retained-root-settings")
    subcatchment = simulation.subcatchments["Sub-To-Sub"]
    retained = subcatchment.settings
    current = subcatchment.settings

    current.width = 88.0
    assert retained.width == pytest.approx(88.0)

    retained.slope = 0.03
    assert current.slope == pytest.approx(0.03)
    simulation.close()


def test_horton_mutation_reads_back_canonical_owner_values(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "canonical-horton-settings")
    settings = simulation.subcatchments["Sub-To-Sub"].settings

    settings.infiltration = HortonInfiltrationSettings(3.0, 0.5, 4.0, 0.0, 0.0)
    canonical = settings.infiltration
    assert canonical.drying_time_days == pytest.approx(0.000001)
    assert canonical.maximum_infiltration is None
    simulation.close()


def test_subcatchment_scale_factors_are_running_forcing(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "scale-factors")
    subcatchment = simulation.subcatchments["Sub-To-Sub"]
    settings = subcatchment.settings
    subcatchment.set_precipitation_scale_factors(rainfall=1.25, snowfall=0.75)
    assert subcatchment.rain_scale_factor == 1.25
    assert subcatchment.snow_scale_factor == 0.75
    assert settings.width == 75.0
    assert not simulation.configuration_dirty

    simulation.start(save_results=False)
    subcatchment.set_precipitation_scale_factors(rainfall=2.0, snowfall=0.75)
    assert subcatchment.rain_scale_factor == 2.0
    assert settings.width == 75.0
    with pytest.raises(swmmrs.LifecycleError):
        settings.width = 80.0
    assert simulation.state is swmmrs.SimulationState.RUNNING
    simulation.end()
    subcatchment.set_precipitation_scale_factors(rainfall=1.5, snowfall=0.75)
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert not simulation.configuration_dirty
    simulation.close()


def test_failed_live_settings_update_preserves_model_and_dirty_state(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "failed-settings")
    subcatchment = simulation.subcatchments["Sub-To-Sub"]
    settings = subcatchment.settings
    infiltration = settings.infiltration
    before_initial_rate = infiltration.initial_rate
    simulation.start(save_results=False)

    with pytest.raises(swmmrs.LifecycleError):
        infiltration.initial_rate = 3.3

    with pytest.raises(swmmrs.LifecycleError):
        settings.update(width=90.0, outlet=simulation.nodes["J1"])

    assert infiltration.initial_rate == before_initial_rate
    assert not simulation.configuration_dirty
    simulation.end()
    simulation.close()


def test_current_results_and_persistent_external_forcing_lifecycle(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    subcatchment = simulation.subcatchments["Sub-Node"]
    upstream = simulation.subcatchments["Sub-To-Sub"]
    settings = subcatchment.settings

    for name in (
        "rainfall",
        "evaporation",
        "infiltration",
        "runon",
        "runoff",
        "snow_depth",
    ):
        with pytest.raises(swmmrs.LifecycleError):
            getattr(subcatchment, name)

    subcatchment.external_rainfall = 0.4
    upstream.external_rainfall = 10.0
    subcatchment.external_snowfall = 0.2
    assert subcatchment.external_rainfall == 0.4
    assert subcatchment.external_snowfall == 0.2
    assert_invalid_unchanged(simulation, subcatchment, "external_rainfall", -0.1)
    assert_invalid_unchanged(simulation, subcatchment, "external_snowfall", math.inf)

    simulation.start(save_results=False)
    simulation.step()
    assert subcatchment.rainfall == pytest.approx(0.6, abs=1e-7, rel=0)
    assert subcatchment.evaporation >= 0.0
    assert subcatchment.infiltration >= 0.0
    assert subcatchment.runon >= 0.0
    assert subcatchment.runoff >= 0.0
    assert subcatchment.snow_depth >= 0.0
    assert settings.width == 100.0
    assert simulation.subcatchments.snapshot([subcatchment.id]).runoff[0] >= 0.0
    simulation.step()
    simulation.step()
    assert subcatchment.runon > 0.0
    completed_rainfall = subcatchment.rainfall

    subcatchment.external_rainfall = 0.7
    subcatchment.external_snowfall = 0.3
    simulation.end()
    assert subcatchment.rainfall == pytest.approx(completed_rainfall, abs=1e-7, rel=0)
    simulation.start(save_results=False)
    simulation.step()
    assert subcatchment.rainfall == pytest.approx(1.0, abs=1e-7, rel=0)
    simulation.end()
    simulation.close()


def test_project_unit_conversion_and_complete_ended_windows(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(
        METRIC_FIXTURE,
        tmp_path / "metric.rpt",
        tmp_path / "metric.out",
    )
    gage = simulation.rain_gages["RG1"]
    subcatchment = simulation.subcatchments["1"]
    settings = subcatchment.settings
    assert settings.area == pytest.approx(10.0, abs=1e-7, rel=0)
    assert settings.width == pytest.approx(500.0, abs=1e-7, rel=0)
    assert settings.slope == pytest.approx(0.0001, abs=1e-7, rel=0)
    assert settings.impervious_fraction == 0.5

    gage.use_external_precipitation(5.08)
    subcatchment.external_rainfall = 2.54
    simulation.start(save_results=False)
    simulation.step()
    assert gage.total_precip == pytest.approx(5.08, abs=1e-7, rel=0)
    assert subcatchment.rainfall == pytest.approx(7.62, abs=1e-7, rel=0)

    while simulation.step() is not None:
        pass
    assert simulation.state is swmmrs.SimulationState.COMPLETE
    assert gage.total_precip == pytest.approx(5.08, abs=1e-7, rel=0)
    with pytest.raises(swmmrs.LifecycleError):
        gage.use_external_precipitation(1.0)
    with pytest.raises(swmmrs.LifecycleError):
        subcatchment.external_rainfall = 1.0
    with pytest.raises(swmmrs.LifecycleError):
        subcatchment.set_precipitation_scale_factors(rainfall=1.0, snowfall=1.0)
    with pytest.raises(swmmrs.LifecycleError):
        gage.rainfall_override = 1.0

    simulation.end()
    assert gage.total_precip == pytest.approx(5.08, abs=1e-7, rel=0)
    assert subcatchment.runoff >= 0.0
    simulation.close()


def test_current_result_reads_are_safe_from_concurrent_callers(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    gage = simulation.rain_gages["Gage-A"]
    subcatchment = simulation.subcatchments["Sub-Node"]
    simulation.start(save_results=False)
    simulation.step()

    def read_results(_: int) -> tuple[float, float]:
        return gage.total_precip, subcatchment.runoff

    with ThreadPoolExecutor(max_workers=4) as executor:
        results = list(executor.map(read_results, range(40)))
    assert results == [results[0]] * 40
    simulation.end()
    simulation.close()


def test_live_view_access_revalidates_generation_and_identity(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "first")
    gage = simulation.rain_gages["Gage-A"]
    subcatchment = simulation.subcatchments["Sub-Node"]
    simulation.close()
    simulation.open(
        FIXTURE,
        tmp_path / "second.rpt",
        tmp_path / "second.out",
    )

    with pytest.raises(swmmrs.StaleViewError):
        _ = gage.rainfall_override
    with pytest.raises(swmmrs.StaleViewError):
        _ = subcatchment.settings.width
    with pytest.raises(swmmrs.StaleViewError):
        subcatchment.external_rainfall = 1.0
    simulation.close()
