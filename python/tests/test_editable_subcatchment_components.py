from __future__ import annotations

# pyright: reportMissingImports=false
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Barrier

import pytest

import swmmrs
from swmmrs.objects import (
    SnowmeltParameterSet,
    SnowmeltSurface,
    SubcatchmentGroundwater,
    SubcatchmentGroundwaterSettings,
)

FIXTURE = Path(__file__).resolve().parent / "data" / "quality.inp"


def _simulation(directory: Path, name: str, *, with_pattern: bool = False) -> swmmrs.Simulation:
    input_path = directory / f"{name}.inp"
    pattern_id = " Pattern-A" if with_pattern else ""
    patterns = (
        """
[PATTERNS]
Pattern-A MONTHLY 1 1 1 1 1 1
Pattern-A 1 1 1 1 1 1
"""
        if with_pattern
        else ""
    )
    input_path.write_text(
        FIXTURE.read_text().replace(
            "[REPORT]",
            f"""[AQUIFERS]
Aquifer-A 0.48 0.25 0.37 5 10 15 0.35 14 0.002 0 10 0.3{pattern_id}
{patterns}
[SNOWPACKS]
Snow-A PLOWABLE 0.003 0.01 31 0.03 0.1 0 0
Snow-A IMPERVIOUS 0.003 0.01 32 0.05 0.1 0 1
Snow-A PERVIOUS 0.003 0.01 32 0.05 0.2 0 1

[REPORT]""",
        )
    )
    return swmmrs.Simulation(input_path, directory / f"{name}.rpt", directory / f"{name}.out")


def _groundwater_settings(simulation: swmmrs.Simulation) -> SubcatchmentGroundwaterSettings:
    return SubcatchmentGroundwaterSettings(
        aquifer=simulation.aquifers["Aquifer-A"],
        node=simulation.nodes["J1"],
        surface_elevation=20.0,
        groundwater_coefficient=1.0,
        groundwater_exponent=1.0,
        surface_coefficient=0.0,
        surface_exponent=0.0,
        interaction_coefficient=0.0,
        fixed_surface_depth=0.0,
        bottom_elevation=0.0,
        water_table_elevation=10.0,
        upper_moisture=0.3,
    )


def test_live_optional_components_set_clear_and_reject_foreign_references(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "bound-components")
    foreign = _simulation(tmp_path, "foreign-components")
    settings = simulation.subcatchments["S-A"].settings
    groundwater_declaration = _groundwater_settings(simulation)
    snowpack_declaration = simulation.snowmelt_sets["Snow-A"]

    settings.groundwater = groundwater_declaration
    settings.snowpack = snowpack_declaration
    groundwater = settings.groundwater
    snowpack = settings.snowpack
    assert isinstance(groundwater, SubcatchmentGroundwater)
    assert isinstance(snowpack, SnowmeltParameterSet)
    groundwater.aquifer = simulation.aquifers["Aquifer-A"]
    groundwater.node = simulation.nodes["J1"]
    assert groundwater.aquifer == simulation.aquifers["Aquifer-A"]
    assert groundwater.node == simulation.nodes["J1"]
    for field, value in {
        "surface_elevation": 21.0,
        "groundwater_coefficient": 2.0,
        "groundwater_exponent": 1.2,
        "surface_coefficient": 0.1,
        "surface_exponent": 1.1,
        "interaction_coefficient": 0.2,
        "fixed_surface_depth": 0.5,
        "bottom_elevation": 1.0,
        "water_table_elevation": 10.0,
        "upper_moisture": 0.4,
    }.items():
        setattr(groundwater, field, value)
        assert getattr(groundwater, field) == pytest.approx(value)
    current_groundwater = settings.groundwater
    assert current_groundwater is not None
    assert current_groundwater.surface_elevation == pytest.approx(21.0)
    assert groundwater_declaration.surface_elevation == pytest.approx(20.0)

    foreign_groundwater = _groundwater_settings(foreign)
    with pytest.raises(swmmrs.ValidationError):
        settings.groundwater = foreign_groundwater
    current_groundwater = settings.groundwater
    assert current_groundwater is not None
    assert current_groundwater.surface_elevation == pytest.approx(21.0)
    assert foreign_groundwater.surface_elevation == pytest.approx(20.0)

    settings.groundwater = None
    with pytest.raises(swmmrs.StaleViewError):
        _ = groundwater.surface_elevation
    with pytest.raises(swmmrs.StaleViewError):
        groundwater.surface_elevation = 22.0
    assert settings.snowpack is not None

    settings.snowpack = None
    assert settings.snowpack is None
    assert snowpack == simulation.snowmelt_sets["Snow-A"]

    settings.groundwater = _groundwater_settings(simulation)
    settings.snowpack = snowpack_declaration
    assert groundwater.surface_elevation == pytest.approx(20.0)
    assert settings.snowpack == simulation.snowmelt_sets["Snow-A"]
    simulation.start(save_results=False)
    simulation.end()
    foreign.close()
    simulation.close()


def test_settings_native_rejects_wrong_families_and_numeric_bools(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "settings-native-validation")
    settings = simulation.subcatchments["S-A"].settings
    original_rain_gage = settings.rain_gage
    original_outlet = settings.outlet
    original_width = settings.width

    with pytest.raises(swmmrs.ValidationError):
        settings.rain_gage = simulation.nodes["J1"]  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        settings.outlet = simulation.rain_gages["G1"]  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        settings.width = True  # type: ignore[assignment]

    settings.groundwater = _groundwater_settings(simulation)
    groundwater = settings.groundwater
    assert groundwater is not None
    with pytest.raises(swmmrs.ValidationError):
        groundwater.aquifer = simulation.nodes["J1"]  # type: ignore[assignment]

    settings.snowpack = simulation.snowmelt_sets["Snow-A"]
    snowpack = settings.snowpack
    assert snowpack is not None
    with pytest.raises(swmmrs.ValidationError):
        settings.snowpack = simulation.nodes["J1"]  # type: ignore[assignment]

    with pytest.raises(swmmrs.ValidationError):
        settings.initial_buildup["TSS"] = True

    assert settings.rain_gage == original_rain_gage
    assert settings.outlet == original_outlet
    assert settings.width == original_width
    assert groundwater.aquifer == simulation.aquifers["Aquifer-A"]
    assert snowpack == simulation.snowmelt_sets["Snow-A"]
    assert settings.initial_buildup["TSS"] == 0.0
    simulation.close()


def test_live_named_mappings_are_atomic_ordered_and_reusable(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "mapping-settings")
    subcatchment = simulation.subcatchments["S-A"]
    settings = subcatchment.settings
    initial_buildup = settings.initial_buildup
    coverage_fractions = settings.coverage_fractions

    assert list(initial_buildup) == ["TSS", "Count"]
    assert list(coverage_fractions) == ["Land-A"]
    assert initial_buildup["tss"] == pytest.approx(0.0)
    assert coverage_fractions["land-a"] == pytest.approx(1.0)
    initial_buildup.update()
    assert not simulation.configuration_dirty
    with pytest.raises(KeyError):
        initial_buildup["unknown"] = 3.0
    with pytest.raises(TypeError):
        initial_buildup[simulation.pollutants["TSS"]] = 3.0  # type: ignore[index]
    with pytest.raises(swmmrs.ValidationError):
        initial_buildup["TSS"] = float("nan")
    with pytest.raises(swmmrs.ValidationError):
        coverage_fractions["Land-A"] = 1.5
    assert not simulation.configuration_dirty

    initial_buildup["tss"] = 2.5
    assert initial_buildup["TSS"] == pytest.approx(2.5)
    initial_buildup.update([("Count", 3.0), ("TSS", 4.0)])
    assert dict(initial_buildup) == {"TSS": 4.0, "Count": 3.0}

    before = dict(initial_buildup)
    with pytest.raises(swmmrs.ValidationError):
        initial_buildup.update([("TSS", 5.0), ("tss", 6.0)])
    assert dict(initial_buildup) == before
    with pytest.raises(swmmrs.ValidationError):
        initial_buildup.update([("TSS", 5.0), ("Count", -1.0)])
    assert dict(initial_buildup) == before
    with pytest.raises(KeyError):
        del initial_buildup["unknown"]
    assert dict(initial_buildup) == before

    settings.initial_buildup = {"TSS": 7.0}
    assert dict(initial_buildup) == {"TSS": 7.0, "Count": 0.0}
    settings.coverage_fractions = {"Land-A": 0.4}
    assert coverage_fractions["Land-A"] == pytest.approx(0.4)

    settings.update(
        initial_buildup={"TSS": 8.0},
        coverage_fractions={"Land-A": 0.5},
    )
    assert dict(initial_buildup) == {"TSS": 8.0, "Count": 0.0}
    assert coverage_fractions["Land-A"] == pytest.approx(0.5)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(
            initial_buildup={"TSS": 9.0},
            coverage_fractions={"Land-A": 1.5},
        )
    assert dict(initial_buildup) == {"TSS": 8.0, "Count": 0.0}
    assert coverage_fractions["Land-A"] == pytest.approx(0.5)

    del initial_buildup["TSS"]
    assert initial_buildup["TSS"] == pytest.approx(0.0)
    coverage_fractions.clear()
    assert coverage_fractions["Land-A"] == pytest.approx(0.0)
    simulation.close()


def test_live_groundwater_cross_field_failure_is_deferred_and_repairable(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "groundwater-settings-deferred")
    settings = simulation.subcatchments["S-A"].settings
    settings.groundwater = _groundwater_settings(simulation)
    simulation.start(save_results=False)
    simulation.end()

    groundwater = settings.groundwater
    assert groundwater is not None
    groundwater.update()
    groundwater.update(groundwater_coefficient=2.0, surface_elevation=5.0)
    assert groundwater.groundwater_coefficient == pytest.approx(2.0)
    assert groundwater.surface_elevation == pytest.approx(5.0)
    assert simulation.configuration_dirty

    with pytest.raises(swmmrs.ConfigurationError) as caught:
        simulation.start(save_results=False)
    assert [diagnostic.rule_code for diagnostic in caught.value.diagnostics] == [
        "subcatchment.groundwater.elevation"
    ]
    diagnostic = caught.value.diagnostics[0]
    assert diagnostic.property_path == "groundwater.surface_elevation"
    assert (
        diagnostic.message
        == "groundwater surface elevation must not be below water-table elevation"
    )
    assert str(caught.value).endswith(
        "property 'groundwater.surface_elevation', rule "
        "'subcatchment.groundwater.elevation': groundwater surface elevation must not be below "
        "water-table elevation"
    )
    assert groundwater.groundwater_coefficient == pytest.approx(2.0)
    assert groundwater.surface_elevation == pytest.approx(5.0)
    assert simulation.configuration_dirty

    groundwater.surface_elevation = 20.0
    assert groundwater.groundwater_coefficient == pytest.approx(2.0)
    assert groundwater.surface_elevation == pytest.approx(20.0)
    assert simulation.configuration_dirty
    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    assert groundwater.groundwater_coefficient == pytest.approx(2.0)
    assert groundwater.surface_elevation == pytest.approx(20.0)
    simulation.end()
    assert groundwater.surface_elevation == pytest.approx(20.0)

    groundwater.surface_elevation = 21.0
    assert groundwater.surface_elevation == pytest.approx(21.0)
    assert simulation.configuration_dirty
    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    assert groundwater.surface_elevation == pytest.approx(21.0)
    simulation.end()
    assert groundwater.surface_elevation == pytest.approx(21.0)
    simulation.close()


def test_concurrent_component_field_updates_preserve_both_writes(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "concurrent-component-settings")
    settings = simulation.subcatchments["S-A"].settings
    settings.groundwater = _groundwater_settings(simulation)
    first = settings.groundwater
    second = settings.groundwater
    assert first is not None
    assert second is not None
    barrier = Barrier(2)

    def set_coefficient() -> None:
        barrier.wait()
        first.groundwater_coefficient = 2.0

    def set_exponent() -> None:
        barrier.wait()
        second.groundwater_exponent = 3.0

    with ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(set_coefficient), executor.submit(set_exponent)]
        for future in futures:
            future.result()

    current = settings.groundwater
    assert current is not None
    assert current.groundwater_coefficient == pytest.approx(2.0)
    assert current.groundwater_exponent == pytest.approx(3.0)
    simulation.close()


def test_aquifer_properties_read_and_write_through_owner(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "aquifer-properties", with_pattern=True)
    foreign = _simulation(tmp_path, "foreign-aquifer-properties", with_pattern=True)
    aquifer = simulation.aquifers["Aquifer-A"]
    pattern = simulation.time_patterns["Pattern-A"]

    assert aquifer.porosity == pytest.approx(0.48)
    assert aquifer.upper_evaporation_pattern == pattern
    assert not simulation.configuration_dirty

    aquifer.update()
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.ValidationError):
        aquifer.update(porosity=0.52, field_capacity=True)
    with pytest.raises(swmmrs.ValidationError):
        aquifer.update(unknown_field=1.0)
    with pytest.raises(swmmrs.ValidationError):
        aquifer.upper_evaporation_pattern = aquifer  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        aquifer.upper_evaporation_pattern = foreign.time_patterns["Pattern-A"]
    assert aquifer.porosity == pytest.approx(0.48)
    assert aquifer.field_capacity == pytest.approx(0.37)
    assert aquifer.upper_evaporation_pattern == pattern
    assert not simulation.configuration_dirty

    values = {
        "porosity": 0.5,
        "wilting_point": 0.2,
        "field_capacity": 0.35,
        "hydraulic_conductivity": 6.0,
        "conductivity_slope": 11.0,
        "tension_slope": 16.0,
        "upper_evaporation_fraction": 0.4,
        "lower_evaporation_depth": 15.0,
        "lower_loss_coefficient": 0.003,
        "bottom_elevation": 1.0,
        "water_table_elevation": 9.0,
        "upper_moisture": 0.32,
    }
    for name, value in values.items():
        setattr(aquifer, name, value)
        assert getattr(aquifer, name) == pytest.approx(value)

    aquifer.update(porosity=0.51, field_capacity=0.36)
    assert aquifer.porosity == pytest.approx(0.51)
    assert aquifer.field_capacity == pytest.approx(0.36)
    assert aquifer.upper_evaporation_pattern == pattern

    aquifer.upper_evaporation_pattern = None
    aquifer.update(porosity=0.52)
    assert aquifer.upper_evaporation_pattern is None
    aquifer.upper_evaporation_pattern = pattern
    assert aquifer.upper_evaporation_pattern == pattern

    input_path = simulation.input_path
    report_path = simulation.report_path
    output_path = simulation.output_path
    retained_pattern = pattern
    simulation.close()
    simulation.open(input_path, report_path, output_path)
    reopened_aquifer = simulation.aquifers["Aquifer-A"]
    with pytest.raises(swmmrs.ValidationError):
        reopened_aquifer.upper_evaporation_pattern = retained_pattern
    assert reopened_aquifer.upper_evaporation_pattern == simulation.time_patterns["Pattern-A"]
    simulation.close()
    foreign.close()


def test_snowmelt_properties_and_atomic_updates_read_through_owner(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "snowmelt-properties")
    foreign = _simulation(tmp_path, "foreign-snowmelt-properties")
    snowmelt = simulation.snowmelt_sets["Snow-A"]
    subcatchment = simulation.subcatchments["S-A"]

    assert snowmelt.plowable_fraction == pytest.approx(0.0)
    assert snowmelt.removal_fractions == pytest.approx((0.0, 0.0, 0.0, 0.0, 0.0))
    assert snowmelt.removal_subcatchment is None
    snowmelt.update()
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.update(plowable_fraction=True)
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.update(unknown=1.0)
    assert not simulation.configuration_dirty

    plowable = snowmelt.plowable
    plowable.update()
    assert not simulation.configuration_dirty
    assert isinstance(plowable, SnowmeltSurface)
    assert plowable.minimum_melt_coefficient == pytest.approx(0.003)
    plowable.update(
        minimum_melt_coefficient=0.004,
        maximum_melt_coefficient=0.02,
        base_temperature=30.0,
        free_water_fraction=0.04,
        initial_snow_depth=0.2,
        initial_free_water=0.01,
        snow_depth_for_full_coverage=0.0,
    )
    assert plowable.minimum_melt_coefficient == pytest.approx(0.004)
    assert plowable.maximum_melt_coefficient == pytest.approx(0.02)
    assert plowable.base_temperature == pytest.approx(30.0)
    assert plowable.free_water_fraction == pytest.approx(0.04)
    assert plowable.initial_snow_depth == pytest.approx(0.2)
    assert plowable.initial_free_water == pytest.approx(0.01)
    assert plowable.snow_depth_for_full_coverage == pytest.approx(0.0)
    with pytest.raises(swmmrs.ValidationError):
        plowable.snow_depth_for_full_coverage = 0.12

    before_surface = (
        plowable.minimum_melt_coefficient,
        plowable.maximum_melt_coefficient,
        plowable.base_temperature,
        plowable.free_water_fraction,
        plowable.initial_snow_depth,
        plowable.initial_free_water,
        plowable.snow_depth_for_full_coverage,
    )
    for invalid_surface in (
        {"base_temperature": float("nan")},
        {"initial_snow_depth": float("inf")},
        {"snow_depth_for_full_coverage": float("-inf")},
    ):
        with pytest.raises(swmmrs.ValidationError):
            plowable.update(**invalid_surface)
        assert [
            plowable.minimum_melt_coefficient,
            plowable.maximum_melt_coefficient,
            plowable.base_temperature,
            plowable.free_water_fraction,
            plowable.initial_snow_depth,
            plowable.initial_free_water,
            plowable.snow_depth_for_full_coverage,
        ] == list(before_surface)

    # Melt-coefficient ordering is relational and remains repairable until start().
    plowable.maximum_melt_coefficient = 0.001
    assert simulation.configuration_dirty
    with pytest.raises(swmmrs.ConfigurationError) as error:
        simulation.start(save_results=False)
    assert [diagnostic.rule_code for diagnostic in error.value.diagnostics] == [
        "snowmelt.parameters.range",
    ]
    assert plowable.maximum_melt_coefficient == pytest.approx(0.001)
    plowable.maximum_melt_coefficient = 0.02

    snowmelt.update(
        plowable_fraction=0.25,
        plow_depth=0.2,
        removal_fractions=(0.0, 0.1, 0.2, 0.3, 0.4),
        removal_subcatchment=subcatchment,
    )
    assert snowmelt.plowable_fraction == pytest.approx(0.25)
    assert snowmelt.plow_depth == pytest.approx(0.2)
    assert snowmelt.removal_fractions == pytest.approx((0.0, 0.1, 0.2, 0.3, 0.4))
    assert snowmelt.removal_subcatchment == subcatchment

    before = snowmelt.removal_fractions
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.update(
            plowable_fraction=0.5,
            removal_fractions=(0.0, 0.1, float("nan"), 0.3, 0.4),
        )
    assert snowmelt.plowable_fraction == pytest.approx(0.25)
    assert snowmelt.removal_fractions == before
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.removal_subcatchment = foreign.subcatchments["S-A"]
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.removal_subcatchment = simulation.nodes["J1"]  # type: ignore[assignment]
    assert snowmelt.removal_subcatchment == subcatchment

    stale_owner = _simulation(tmp_path, "stale-snowmelt-relationship")
    stale_subcatchment = stale_owner.subcatchments["S-A"]
    stale_owner.close()
    with pytest.raises(swmmrs.ValidationError):
        snowmelt.removal_subcatchment = stale_subcatchment
    assert snowmelt.removal_subcatchment == subcatchment

    snowmelt.removal_subcatchment = None
    assert snowmelt.removal_subcatchment is None


def test_definition_patches_and_settings_prepare_after_repair(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "definitions")
    aquifer = simulation.aquifers["Aquifer-A"]
    snowmelt = simulation.snowmelt_sets["Snow-A"]
    subcatchment = simulation.subcatchments["S-A"]

    with pytest.raises(swmmrs.ValidationError):
        aquifer.update(porosity=float("nan"))
    assert not simulation.configuration_dirty

    aquifer.update(porosity=0.0)
    for snow_surface in (snowmelt.plowable, snowmelt.impervious, snowmelt.pervious):
        snow_surface.update(
            minimum_melt_coefficient=0.003,
            maximum_melt_coefficient=0.01,
            base_temperature=31.0,
            free_water_fraction=0.03,
            initial_snow_depth=0.1,
            initial_free_water=0.0,
        )
    snowmelt.update(
        plowable_fraction=0.25,
        plow_depth=0.2,
        removal_fractions=(0.0, 0.1, 0.2, 0.3, 0.4),
        removal_subcatchment=subcatchment,
    )
    settings = subcatchment.settings
    settings.groundwater = _groundwater_settings(simulation)
    settings.initial_buildup["TSS"] = 2.5
    settings.coverage_fractions["Land-A"] = 0.4

    with pytest.raises(swmmrs.ConfigurationError) as error:
        simulation.start(save_results=False)
    assert [diagnostic.rule_code for diagnostic in error.value.diagnostics] == [
        "aquifer.parameters.range",
    ]

    aquifer.update(
        porosity=0.48,
        wilting_point=0.25,
        field_capacity=0.37,
        hydraulic_conductivity=5.0,
        upper_moisture=0.3,
    )
    subcatchment.settings.groundwater = _groundwater_settings(simulation)
    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.LifecycleError):
        aquifer.porosity = 0.49
    with pytest.raises(swmmrs.LifecycleError):
        snowmelt.plowable_fraction = 0.3
    assert aquifer.porosity == pytest.approx(0.48)
    assert snowmelt.plowable_fraction == pytest.approx(0.25)
    simulation.end()

    aquifer.porosity = 0.49
    snowmelt.plowable_fraction = 0.3
    subcatchment.settings.width = 80.0
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert subcatchment.settings.width == pytest.approx(80.0)
    simulation.start(save_results=False)
    simulation.end()
    simulation.close()


def test_retained_component_views_follow_replacement_and_reactivation(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path, "component-views")
    settings = simulation.subcatchments["S-A"].settings
    settings.groundwater = _groundwater_settings(simulation)
    settings.snowpack = simulation.snowmelt_sets["Snow-A"]
    groundwater = settings.groundwater
    snowpack = settings.snowpack
    assert groundwater is not None
    assert snowpack is not None

    replacement = _groundwater_settings(simulation)
    replacement.surface_elevation = 21.0
    settings.groundwater = replacement
    assert groundwater.surface_elevation == pytest.approx(21.0)
    assert snowpack == simulation.snowmelt_sets["Snow-A"]

    settings.groundwater = None
    settings.snowpack = None
    with pytest.raises(swmmrs.StaleViewError):
        _ = groundwater.surface_elevation
    with pytest.raises(swmmrs.StaleViewError):
        groundwater.update()
    assert snowpack == simulation.snowmelt_sets["Snow-A"]

    settings.groundwater = _groundwater_settings(simulation)
    settings.snowpack = simulation.snowmelt_sets["Snow-A"]
    assert groundwater.surface_elevation == pytest.approx(20.0)
    assert settings.snowpack == simulation.snowmelt_sets["Snow-A"]
    simulation.close()
