from __future__ import annotations

# pyright: reportMissingImports=false
import gc
from datetime import timedelta
from pathlib import Path
from types import GetSetDescriptorType

import pytest

import swmmrs
from swmmrs.enums import UnitSystem
from swmmrs.snapshots import LidUnitSnapshot, SubcatchmentLidSnapshot

FIXTURE = Path(__file__).resolve().parent / "data" / "lids.inp"


def native_fields(record: object) -> tuple[str, ...]:
    """Return the explicit public native get-set schema."""
    return tuple(
        name
        for name, descriptor in vars(type(record)).items()
        if isinstance(descriptor, GetSetDescriptorType)
    )


def assert_native_schema(record: object, expected: tuple[str, ...]) -> None:
    assert set(native_fields(record)) == set(expected)
    representation = repr(record)
    positions = [representation.index(f"{name}=") for name in expected]
    assert positions == sorted(positions)


def test_exact_records_cover_representative_lid_types_and_optional_layers(
    tmp_path: Path,
) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "lids.rpt")
    simulation.start(save_results=False)
    simulation.step()
    subcatchment = simulation.subcatchments["With-Lids"]

    snapshots = tuple(subcatchment.lid_units[index].snapshot() for index in (0, 1, 3, 4, 7))
    assert all(type(snapshot) is LidUnitSnapshot for snapshot in snapshots)
    assert_native_schema(
        snapshots[0],
        (
            "inflow",
            "evaporation",
            "infiltration",
            "surface_outflow",
            "drain_outflow",
            "initial_volume",
            "final_volume",
            "surface_depth",
            "pavement_depth",
            "storage_depth",
            "soil_moisture",
            "dry_time",
            "old_drain_flow",
            "new_drain_flow",
            "evaporation_rate",
            "maximum_native_infiltration_rate",
            "surface_inflow_rate",
            "surface_infiltration_rate",
            "surface_evaporation_rate",
            "surface_outflow_rate",
            "pavement_evaporation_rate",
            "pavement_percolation_rate",
            "soil_evaporation_rate",
            "soil_percolation_rate",
            "storage_inflow_rate",
            "storage_exfiltration_rate",
            "storage_evaporation_rate",
            "storage_drain_rate",
            "surface_flux_rate",
            "soil_flux_rate",
            "storage_flux_rate",
            "pavement_flux_rate",
        ),
    )
    assert all(isinstance(snapshot.dry_time, timedelta) for snapshot in snapshots)
    assert subcatchment.lid_units[0].control.soil is not None
    assert subcatchment.lid_units[1].control.drainage_mat is not None
    assert subcatchment.lid_units[3].control.pavement is not None
    assert subcatchment.lid_units[4].control.soil is None
    assert subcatchment.lid_units[7].control.storage is None

    group = subcatchment.lid_snapshot()
    assert type(group) is SubcatchmentLidSnapshot
    assert_native_schema(
        group,
        (
            "pervious_area",
            "flow_to_pervious_area",
            "old_drain_flow",
            "new_drain_flow",
        ),
    )
    simulation.close()


def test_acquisition_requires_running_or_complete_and_rejects_missing_group(
    tmp_path: Path,
) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "lifecycle.rpt")
    subcatchment = simulation.subcatchments["With-Lids"]
    unit = subcatchment.lid_units[0]
    try:
        with pytest.raises(swmmrs.LifecycleError):
            unit.snapshot()
        with pytest.raises(swmmrs.LifecycleError):
            subcatchment.lid_snapshot()

        simulation.start(save_results=False)
        with pytest.raises(swmmrs.ValidationError):
            simulation.subcatchments["Receiver"].lid_snapshot()
        while simulation.step() is not None:
            pass
        assert type(unit.snapshot()) is LidUnitSnapshot
        assert type(subcatchment.lid_snapshot()) is SubcatchmentLidSnapshot

        simulation.end()
        with pytest.raises(swmmrs.LifecycleError):
            unit.snapshot()
        with pytest.raises(swmmrs.LifecycleError):
            subcatchment.lid_snapshot()
    finally:
        simulation.close()


def test_snapshots_are_immutable_owner_free_historical_values(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "history.rpt")
    simulation.start(save_results=False)
    simulation.step()
    subcatchment = simulation.subcatchments["With-Lids"]
    unit = subcatchment.lid_units[0]
    historical_unit = unit.snapshot()
    historical_group = subcatchment.lid_snapshot()
    original_unit = {
        field_name: getattr(historical_unit, field_name)
        for field_name in native_fields(historical_unit)
    }
    original_group = {
        field_name: getattr(historical_group, field_name)
        for field_name in native_fields(historical_group)
    }

    with pytest.raises(AttributeError):
        historical_unit.inflow = -1.0  # type: ignore[misc]
    assert not hasattr(historical_unit, "_owner")
    assert not hasattr(historical_unit, "_generation")
    assert not hasattr(historical_group, "_owner")
    assert not hasattr(historical_group, "_generation")

    simulation.step()
    assert historical_unit.inflow == original_unit["inflow"]
    assert historical_group.new_drain_flow == original_group["new_drain_flow"]
    simulation.end()
    assert historical_unit.dry_time == original_unit["dry_time"]
    simulation.close()
    assert historical_group.pervious_area == original_group["pervious_area"]

    simulation.open(FIXTURE, tmp_path / "reopened.rpt")
    with pytest.raises(swmmrs.StaleViewError):
        unit.snapshot()
    with pytest.raises(swmmrs.StaleViewError):
        subcatchment.lid_snapshot()
    simulation.close()
    del unit, subcatchment, simulation
    gc.collect()
    assert {
        field_name: getattr(historical_unit, field_name)
        for field_name in native_fields(historical_unit)
    } == original_unit
    assert {
        field_name: getattr(historical_group, field_name)
        for field_name in native_fields(historical_group)
    } == original_group


def test_snapshots_use_each_project_unit_system(tmp_path: Path) -> None:
    metric = tmp_path / "metric.inp"
    metric.write_text(
        FIXTURE.read_text().replace("FLOW_UNITS           CFS", "FLOW_UNITS           CMS", 1)
    )
    for name, input_path, unit_system in (
        ("us", FIXTURE, UnitSystem.US),
        ("si", metric, UnitSystem.SI),
    ):
        simulation = swmmrs.Simulation(input_path, tmp_path / f"{name}.rpt")
        simulation.start(save_results=False)
        simulation.step()
        subcatchment = simulation.subcatchments["With-Lids"]
        assert simulation.unit_system is unit_system
        assert subcatchment.lid_snapshot().pervious_area == 300.0
        assert type(subcatchment.lid_units[0].snapshot()) is LidUnitSnapshot
        simulation.close()
