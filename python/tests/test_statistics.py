from __future__ import annotations

# pyright: reportMissingImports=false
import gc
import math
from datetime import datetime, timedelta
from pathlib import Path
from types import GetSetDescriptorType, MappingProxyType

import pytest

import swmmrs
from swmmrs.enums import FlowClass
from swmmrs.snapshots import (
    LinkStatistics,
    LinkStatisticsSnapshot,
    NodeStatistics,
    NodeStatisticsSnapshot,
    OutfallStatistics,
    PumpStatistics,
    QualityBalance,
    RoutingDiagnostics,
    RoutingTotals,
    RunoffTotals,
    SimulationStatistics,
    StorageStatistics,
    SubcatchmentStatistics,
    SubcatchmentStatisticsSnapshot,
)

DATA = Path(__file__).resolve().parent / "data"
STATISTICS = DATA / "statistics.inp"
NODES = DATA / "nodes.inp"


def native_fields(record: object) -> tuple[str, ...]:
    """Return the explicit public native get-set schema."""
    record_type = record if isinstance(record, type) else type(record)
    return tuple(
        name
        for name, descriptor in vars(record_type).items()
        if isinstance(descriptor, GetSetDescriptorType)
    )


def assert_native_schema(record: object, expected: tuple[str, ...]) -> None:
    assert set(native_fields(record)) == set(expected)
    if not isinstance(record, type):
        representation = repr(record)
        positions = [representation.index(f"{name}=") for name in expected]
        assert positions == sorted(positions)


def running(fixture: Path, directory: Path) -> swmmrs.Simulation:
    simulation = swmmrs.Simulation(fixture, directory / "statistics.rpt")
    simulation.start(save_results=False)
    simulation.step()
    return simulation


def test_exact_scalar_records_subtypes_types_and_canonical_pollutants(tmp_path: Path) -> None:
    assert tuple(FlowClass) == (
        FlowClass.DRY,
        FlowClass.UPSTREAM_DRY,
        FlowClass.DOWNSTREAM_DRY,
        FlowClass.SUBCRITICAL,
        FlowClass.SUPERCRITICAL,
        FlowClass.UPSTREAM_CRITICAL,
        FlowClass.DOWNSTREAM_CRITICAL,
    )
    simulation = running(STATISTICS, tmp_path)

    node = simulation.nodes["J1"].statistics
    assert type(node) is NodeStatistics
    assert_native_schema(
        node,
        (
            "average_depth",
            "maximum_depth",
            "maximum_depth_time",
            "maximum_reported_depth",
            "flooded_volume",
            "time_flooded",
            "time_surcharged",
            "time_courant_critical",
            "total_lateral_inflow",
            "maximum_lateral_inflow",
            "maximum_inflow",
            "maximum_overflow",
            "maximum_ponded_volume",
            "nonconverged_count",
            "maximum_inflow_time",
            "maximum_overflow_time",
        ),
    )
    assert isinstance(node.maximum_depth_time, datetime)
    assert node.maximum_depth_time.tzinfo is None
    assert isinstance(node.time_flooded, timedelta)
    assert not hasattr(simulation.nodes["J1"], "storage_statistics")

    assert type(simulation.nodes["S1"].statistics) is NodeStatistics
    storage = simulation.nodes["S1"].storage_statistics  # type: ignore[attr-defined]
    assert type(storage) is StorageStatistics
    assert isinstance(storage.maximum_volume_time, datetime)
    assert type(simulation.nodes["O1"].statistics) is NodeStatistics
    outfall = simulation.nodes["O1"].outfall_statistics  # type: ignore[attr-defined]
    assert type(outfall) is OutfallStatistics
    assert type(outfall.pollutant_loads) is MappingProxyType
    assert tuple(outfall.pollutant_loads) == ("TSS", "Bacteria")
    with pytest.raises(TypeError):
        outfall.pollutant_loads["TSS"] = 1.0  # type: ignore[index]
    assert outfall.pollutant_loads["Bacteria"] == 0.0
    assert math.isfinite(outfall.pollutant_loads["Bacteria"])
    assert type(simulation.links["Pump-A"].statistics) is LinkStatistics
    pump = simulation.links["Pump-A"].pump_statistics  # type: ignore[attr-defined]
    assert type(pump) is PumpStatistics
    assert isinstance(pump.time_utilized, timedelta)
    assert pump.time_utilized == timedelta(seconds=30)
    assert pump.startup_count == 1
    assert pump.period_count == 1
    assert pump.average_flow == pytest.approx(0.377358784, abs=1e-7, rel=0)
    assert pump.minimum_flow == 0.0
    assert not hasattr(simulation.links["C2"], "pump_statistics")

    link = simulation.links["C2"].statistics
    assert type(link) is LinkStatistics
    assert len(link.time_in_flow_class) == len(FlowClass)
    assert all(isinstance(value, timedelta) for value in link.time_in_flow_class)
    assert link.maximum_street_fill_fraction is None
    street = simulation.links["C1"].statistics
    assert street.maximum_street_fill_fraction is not None
    assert street.maximum_street_fill_fraction >= 0.0
    assert street.maximum_street_fill_fraction <= 1.0
    subcatchment = simulation.subcatchments["Sub-A"].statistics
    assert type(subcatchment) is SubcatchmentStatistics
    assert subcatchment.precipitation == pytest.approx(1.0 / 600.0, abs=1e-7, rel=0)
    simulation.close()


def test_specialized_batches_are_ordered_atomic_immutable_and_historical(
    tmp_path: Path,
) -> None:
    simulation = running(STATISTICS, tmp_path)
    node_scalar = simulation.nodes["J1"].statistics
    original_scalar = {
        field_name: getattr(node_scalar, field_name) for field_name in native_fields(node_scalar)
    }
    node_snapshot = simulation.nodes.statistics(["J1", 123, "s1"])
    assert type(node_snapshot) is NodeStatisticsSnapshot
    assert node_snapshot.object_ids == ("J1", "123", "S1")
    assert all(isinstance(value, datetime) for value in node_snapshot.maximum_depth_time)
    assert all(value.microsecond == 0 for value in node_snapshot.maximum_depth_time)
    assert all(isinstance(value, timedelta) for value in node_snapshot.time_flooded)
    for position, identifier in enumerate(node_snapshot.object_ids):
        scalar = simulation.nodes[identifier].statistics
        for field_name in native_fields(node_snapshot):
            if field_name != "object_ids":
                assert getattr(node_snapshot, field_name)[position] == getattr(scalar, field_name)

    link_snapshot = simulation.links.statistics(identifier for identifier in ("Pump-A", "C1"))
    assert type(link_snapshot) is LinkStatisticsSnapshot
    assert link_snapshot.object_ids == ("Pump-A", "C1")
    assert len(link_snapshot.time_in_flow_class[0]) == len(FlowClass)
    for position, identifier in enumerate(link_snapshot.object_ids):
        scalar = simulation.links[identifier].statistics
        for field_name in native_fields(link_snapshot):
            if field_name != "object_ids":
                assert getattr(link_snapshot, field_name)[position] == getattr(scalar, field_name)
    assert None in link_snapshot.maximum_street_fill_fraction
    assert any(value is not None for value in link_snapshot.maximum_street_fill_fraction)
    subcatchment_snapshot = simulation.subcatchments.statistics()
    assert type(subcatchment_snapshot) is SubcatchmentStatisticsSnapshot
    assert subcatchment_snapshot.object_ids == tuple(simulation.subcatchments)
    for position, identifier in enumerate(subcatchment_snapshot.object_ids):
        scalar = simulation.subcatchments[identifier].statistics
        for field_name in native_fields(subcatchment_snapshot):
            if field_name != "object_ids":
                assert getattr(subcatchment_snapshot, field_name)[position] == getattr(
                    scalar, field_name
                )

    for empty in (
        simulation.nodes.statistics([]),
        simulation.links.statistics(()),
        simulation.subcatchments.statistics(iter(())),
    ):
        assert all(getattr(empty, field_name) == () for field_name in native_fields(empty))
    for duplicate in (["J1", "j1"], [123, "123"]):
        with pytest.raises(swmmrs.ValidationError):
            simulation.nodes.statistics(duplicate)
    with pytest.raises(KeyError) as caught:
        simulation.nodes.statistics(["J1", "missing", "S1"])
    assert caught.value.args == ("missing",)

    original = {
        field_name: getattr(node_snapshot, field_name)
        for field_name in native_fields(node_snapshot)
    }
    with pytest.raises(AttributeError):
        node_snapshot.maximum_depth = ()  # type: ignore[misc]
    simulation.step()
    assert node_snapshot.maximum_depth == original["maximum_depth"]
    simulation.end()
    simulation.close()
    simulation.open(STATISTICS, tmp_path / "statistics-reopened.rpt")
    simulation.close()
    del simulation
    gc.collect()
    assert {
        field_name: getattr(node_snapshot, field_name)
        for field_name in native_fields(node_snapshot)
    } == original
    assert {
        field_name: getattr(node_scalar, field_name) for field_name in native_fields(node_scalar)
    } == original_scalar


def test_simulation_statistics_exact_nested_shape_units_and_observation(
    tmp_path: Path,
) -> None:
    simulation = running(STATISTICS, tmp_path)
    first = simulation.statistics
    second = simulation.statistics
    assert first == second
    assert type(first) is SimulationStatistics
    assert_native_schema(
        first,
        (
            "routing_totals",
            "runoff_totals",
            "groundwater_continuity_error",
            "quality_continuity_error",
            "routing_diagnostics",
            "quality_balances",
        ),
    )
    assert type(first.routing_diagnostics) is RoutingDiagnostics
    assert_native_schema(
        first.routing_diagnostics,
        (
            "average_time_step",
            "minimum_time_step",
            "maximum_time_step",
            "step_count",
            "nonconverged_step_count",
            "nonconverged_step_percentage",
            "average_iterations",
        ),
    )
    assert type(first.quality_balances) is MappingProxyType
    assert tuple(first.quality_balances) == ("TSS", "Bacteria")
    assert all(type(balance) is QualityBalance for balance in first.quality_balances.values())
    assert set(native_fields(QualityBalance)) == {"continuity_error", "seepage_loss"}
    assert all(
        math.isfinite(value)
        for balance in first.quality_balances.values()
        for value in (balance.continuity_error, balance.seepage_loss)
    )
    assert not hasattr(first.quality_balances["TSS"], "__dict__")
    with pytest.raises(AttributeError):
        first.quality_balances["TSS"].seepage_loss = 1.0  # type: ignore[misc]
    with pytest.raises(TypeError):
        first.quality_balances["TSS"] = first.quality_balances["TSS"]  # type: ignore[index]
    assert not hasattr(first.quality_balances["TSS"], "evaporation_loss")
    assert first.routing_diagnostics.step_count > 0
    assert type(first.routing_diagnostics.step_count) is int
    assert type(first.routing_diagnostics.nonconverged_step_count) is int
    assert all(
        isinstance(value, timedelta)
        for value in (
            first.routing_diagnostics.average_time_step,
            first.routing_diagnostics.minimum_time_step,
            first.routing_diagnostics.maximum_time_step,
        )
    )
    assert math.isfinite(first.routing_diagnostics.nonconverged_step_percentage)
    assert math.isfinite(first.routing_diagnostics.average_iterations)
    first_step_count = first.routing_diagnostics.step_count
    assert not hasattr(first.routing_diagnostics, "__dict__")
    with pytest.raises(AttributeError):
        first.routing_diagnostics.step_count = 1  # type: ignore[misc]
    assert not hasattr(first.routing_diagnostics, "maximum_courant_number")
    assert "conveyed_volume" not in native_fields(LinkStatistics)
    assert "maximum_filling_ratio" not in native_fields(LinkStatistics)
    assert not hasattr(simulation, "mass_balance")
    assert not hasattr(simulation, "quality_continuity_error")
    assert not hasattr(simulation, "quality_seepage_loss")
    assert type(first.routing_totals) is RoutingTotals
    assert_native_schema(
        first.routing_totals,
        (
            "dry_weather_inflow",
            "wet_weather_inflow",
            "groundwater_inflow",
            "rdii_inflow",
            "external_inflow",
            "flooding",
            "outflow",
            "evaporation_loss",
            "seepage_loss",
            "reaction_loss",
            "initial_storage",
            "final_storage",
            "continuity_error",
        ),
    )
    assert type(first.runoff_totals) is RunoffTotals
    assert_native_schema(
        first.runoff_totals,
        (
            "rainfall",
            "evaporation_loss",
            "infiltration_loss",
            "runoff",
            "lid_drain_flow",
            "outfall_runon",
            "initial_surface_storage",
            "final_surface_storage",
            "initial_snow_cover",
            "final_snow_cover",
            "snow_removed",
            "continuity_error",
        ),
    )
    assert not hasattr(first, "flow_continuity_error")
    assert not hasattr(first, "runoff_continuity_error")
    assert first.runoff_totals.rainfall >= 0.0
    assert first.routing_totals.initial_storage == pytest.approx(100.0, abs=1e-7, rel=0)
    assert first.routing_totals.final_storage == pytest.approx(94.339696, abs=1e-7, rel=0)
    assert first.runoff_totals.rainfall == pytest.approx(1.0 / 600.0, abs=1e-7, rel=0)
    first_quality_balances = {
        identifier: (balance.continuity_error, balance.seepage_loss)
        for identifier, balance in first.quality_balances.items()
    }
    first_concrete_values = (
        repr(first.routing_totals),
        repr(first.runoff_totals),
        first.groundwater_continuity_error,
        first.quality_continuity_error,
        first.routing_diagnostics.average_time_step,
        first.routing_diagnostics.step_count,
    )

    while simulation.step() is not None:
        pass
    completed = simulation.statistics
    assert completed != first
    assert completed.runoff_totals.rainfall > 0.0
    assert first.routing_diagnostics.step_count == first_step_count
    assert {
        identifier: (balance.continuity_error, balance.seepage_loss)
        for identifier, balance in first.quality_balances.items()
    } == first_quality_balances
    simulation.end()
    simulation.close()
    simulation.open(STATISTICS, tmp_path / "system-reopened.rpt")
    simulation.close()
    del simulation
    gc.collect()
    assert (
        repr(first.routing_totals),
        repr(first.runoff_totals),
        first.groundwater_continuity_error,
        first.quality_continuity_error,
        first.routing_diagnostics.average_time_step,
        first.routing_diagnostics.step_count,
    ) == first_concrete_values
    assert {
        identifier: (balance.continuity_error, balance.seepage_loss)
        for identifier, balance in first.quality_balances.items()
    } == first_quality_balances


def test_maximum_reported_depth_uses_project_length_units_once(tmp_path: Path) -> None:
    root = tmp_path
    input_path = root / "statistics-si.inp"
    input_path.write_text(
        STATISTICS.read_text().replace("FLOW_UNITS           CFS", "FLOW_UNITS           CMS")
    )
    simulation = swmmrs.Simulation(
        input_path, root / "statistics-si.rpt", root / "statistics-si.out"
    )
    simulation.start()
    while simulation.step() is not None:
        pass

    node = simulation.nodes["J1"].statistics
    assert node.maximum_reported_depth == pytest.approx(node.maximum_depth, abs=1e-8, rel=0)
    simulation.close()


def test_statistics_are_finite_before_first_period_and_for_zero_area(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(STATISTICS, tmp_path / "zero-denominators.rpt")
    simulation.start(save_results=False)

    diagnostics = simulation.statistics.routing_diagnostics
    assert diagnostics.average_time_step == timedelta(0)
    assert diagnostics.minimum_time_step == timedelta(0)
    assert diagnostics.maximum_time_step == timedelta(0)
    assert diagnostics.step_count == 0
    assert diagnostics.nonconverged_step_count == 0
    assert diagnostics.nonconverged_step_percentage == 0.0
    assert diagnostics.average_iterations == 0.0
    assert simulation.nodes["J1"].statistics.average_depth == 0.0
    assert simulation.nodes["S1"].storage_statistics.average_volume == 0.0  # type: ignore[attr-defined]
    assert simulation.subcatchments["Zero-A"].statistics.precipitation == 0.0
    simulation.close()


def test_routing_only_project_has_zero_runoff_depth_totals(tmp_path: Path) -> None:
    simulation = running(NODES, tmp_path)
    runoff = simulation.statistics.runoff_totals
    assert all(
        getattr(runoff, field_name) == 0.0
        for field_name in native_fields(runoff)
        if field_name != "continuity_error"
    )
    simulation.close()


@pytest.mark.parametrize("family", ["nodes", "links", "subcatchments"])
@pytest.mark.parametrize(
    "invalid",
    [
        pytest.param(True, id="boolean"),
        pytest.param(b"J1", id="bytes"),
        pytest.param({"J1": 1}, id="mapping"),
        pytest.param({"J1"}, id="set"),
        pytest.param(object(), id="non-iterable"),
        pytest.param([True], id="boolean-item"),
        pytest.param([object()], id="object-item"),
    ],
)
def test_statistics_reject_malformed_native_selections(
    tmp_path: Path, family: str, invalid: object
) -> None:
    simulation = running(STATISTICS, tmp_path)
    collection = getattr(simulation, family)
    with pytest.raises(swmmrs.ValidationError):
        collection.statistics(invalid)
    simulation.close()


def test_statistics_order_duplicates_and_unknowns_cover_all_families(tmp_path: Path) -> None:
    simulation = running(STATISTICS, tmp_path)
    acquisitions = (
        (simulation.nodes.statistics, ["S1", "J1"]),
        (simulation.links.statistics, ["C1", "Pump-A"]),
        (simulation.subcatchments.statistics, ["Zero-A", "Sub-A"]),
    )
    for acquisition, ordered_ids in acquisitions:
        assert acquisition(iter(ordered_ids)).object_ids == tuple(ordered_ids)
        with pytest.raises(swmmrs.ValidationError):
            acquisition([ordered_ids[0], ordered_ids[0].lower()])
        with pytest.raises(KeyError) as caught:
            acquisition([ordered_ids[0], "missing", ordered_ids[1]])
        assert caught.value.args == ("missing",)
    simulation.close()


def test_statistics_acquisition_is_running_or_complete_only_even_when_empty(
    tmp_path: Path,
) -> None:
    simulation = swmmrs.Simulation(STATISTICS, tmp_path / "lifecycle.rpt")
    stale_generation = simulation._generation()
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.statistics
    for collection in (
        simulation.nodes,
        simulation.links,
        simulation.subcatchments,
    ):
        with pytest.raises(swmmrs.LifecycleError):
            collection.statistics([])
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.nodes["J1"].statistics

    simulation.start(save_results=False)
    while simulation.step() is not None:
        pass
    assert isinstance(simulation.statistics, SimulationStatistics)
    assert len(simulation.nodes.statistics().object_ids) == len(simulation.nodes)
    simulation.end()
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.statistics
    for acquisition in (
        lambda: simulation.nodes["J1"].statistics,
        lambda: simulation.nodes.statistics([]),
        lambda: simulation.links.statistics([]),
        lambda: simulation.subcatchments.statistics([]),
    ):
        with pytest.raises(swmmrs.LifecycleError):
            acquisition()
    simulation.close()
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.statistics
    simulation.open(STATISTICS, tmp_path / "reopened.rpt")
    simulation.start(save_results=False)
    with pytest.raises(swmmrs.StaleViewError):
        simulation._owner.simulation_statistics(stale_generation)
    simulation.close()
