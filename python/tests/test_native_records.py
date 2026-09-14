from __future__ import annotations

# pyright: reportMissingImports=false
from pathlib import Path
from types import GetSetDescriptorType

import pytest

import swmmrs
import swmmrs.snapshots as snapshots

EXPECTED_SCHEMAS: dict[str, tuple[str, ...]] = {
    "LidUnitSnapshot": (
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
    "LinkQualitySnapshot": (
        "object_ids",
        "pollutant_ids",
        "concentrations",
        "reactor_concentrations",
        "total_loads",
    ),
    "LinkSnapshot": (
        "object_ids",
        "setting",
        "target_setting",
        "time_open",
        "time_closed",
        "flow",
        "depth",
        "velocity",
        "top_width",
        "volume",
        "capacity",
        "upstream_surface_area",
        "downstream_surface_area",
        "froude_number",
    ),
    "LinkStatistics": (
        "maximum_flow",
        "maximum_flow_time",
        "maximum_velocity",
        "maximum_depth",
        "maximum_street_fill_fraction",
        "time_normal_flow",
        "time_inlet_control",
        "time_surcharged",
        "time_full_upstream",
        "time_full_downstream",
        "time_full_flow",
        "time_capacity_limited",
        "time_in_flow_class",
        "time_courant_critical",
        "flow_turns",
        "flow_turn_sign",
    ),
    "LinkStatisticsSnapshot": (
        "object_ids",
        "maximum_flow",
        "maximum_flow_time",
        "maximum_velocity",
        "maximum_depth",
        "maximum_street_fill_fraction",
        "time_normal_flow",
        "time_inlet_control",
        "time_surcharged",
        "time_full_upstream",
        "time_full_downstream",
        "time_full_flow",
        "time_capacity_limited",
        "time_in_flow_class",
        "time_courant_critical",
        "flow_turns",
        "flow_turn_sign",
    ),
    "NodeQualitySnapshot": (
        "object_ids",
        "pollutant_ids",
        "concentrations",
        "inflow_concentrations",
        "reactor_concentrations",
    ),
    "NodeSnapshot": (
        "object_ids",
        "depth",
        "head",
        "volume",
        "lateral_inflow",
        "total_inflow",
        "total_outflow",
        "losses",
        "flooding",
        "hydraulic_retention_time",
    ),
    "NodeStatistics": (
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
    "NodeStatisticsSnapshot": (
        "object_ids",
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
    "OutfallStatistics": (
        "average_flow",
        "maximum_flow",
        "pollutant_loads",
        "period_count",
    ),
    "PumpStatistics": (
        "time_utilized",
        "minimum_flow",
        "average_flow",
        "maximum_flow",
        "pumped_volume",
        "energy_consumed",
        "off_curve_low",
        "off_curve_high",
        "startup_count",
        "period_count",
    ),
    "QualityBalance": ("continuity_error", "seepage_loss"),
    "RoutingDiagnostics": (
        "average_time_step",
        "minimum_time_step",
        "maximum_time_step",
        "step_count",
        "nonconverged_step_count",
        "nonconverged_step_percentage",
        "average_iterations",
    ),
    "RoutingTotals": (
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
    "RunoffTotals": (
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
    "SimulationStatistics": (
        "routing_totals",
        "runoff_totals",
        "groundwater_continuity_error",
        "quality_continuity_error",
        "routing_diagnostics",
        "quality_balances",
    ),
    "StorageStatistics": (
        "initial_volume",
        "average_volume",
        "maximum_volume",
        "maximum_inflow",
        "evaporation_losses",
        "exfiltration_losses",
        "maximum_volume_time",
    ),
    "SubcatchmentLidSnapshot": (
        "pervious_area",
        "flow_to_pervious_area",
        "old_drain_flow",
        "new_drain_flow",
    ),
    "SubcatchmentQualitySnapshot": (
        "object_ids",
        "pollutant_ids",
        "runoff_concentrations",
        "ponded_concentrations",
        "buildup_loads",
        "total_washoff_loads",
    ),
    "SubcatchmentSnapshot": (
        "object_ids",
        "rainfall",
        "evaporation",
        "infiltration",
        "runon",
        "runoff",
        "snow_depth",
    ),
    "SubcatchmentStatistics": (
        "precipitation",
        "runon_volume",
        "evaporation_volume",
        "infiltration_volume",
        "runoff_volume",
        "maximum_runoff",
        "impervious_runoff_volume",
        "pervious_runoff_volume",
    ),
    "SubcatchmentStatisticsSnapshot": (
        "object_ids",
        "precipitation",
        "runon_volume",
        "evaporation_volume",
        "infiltration_volume",
        "runoff_volume",
        "maximum_runoff",
        "impervious_runoff_volume",
        "pervious_runoff_volume",
    ),
}

EXPECTED_RECORD_NAMES = tuple(EXPECTED_SCHEMAS)
DATA = Path(__file__).resolve().parent / "data"

DOC_EXPECTATIONS: dict[tuple[str, str], tuple[str, ...]] = {
    ("LinkSnapshot", "capacity"): ("dimensionless", "conduit-full-area ratio"),
    ("LidUnitSnapshot", "inflow"): ("equivalent rain depth", "inches or millimeters"),
    ("LidUnitSnapshot", "dry_time"): ("datetime.timedelta",),
    ("NodeStatistics", "maximum_depth_time"): ("datetime.datetime",),
    ("QualityBalance", "seepage_loss"): ("pollutant mass", "pounds", "kilograms"),
    ("RoutingTotals", "continuity_error"): ("dimensionless", "percent"),
    ("RunoffTotals", "rainfall"): ("rainfall depth", "inches or millimeters"),
    ("StorageStatistics", "evaporation_losses"): (
        "evaporation loss volume",
        "cubic feet or cubic meters",
    ),
    ("StorageStatistics", "exfiltration_losses"): (
        "exfiltration loss volume",
        "cubic feet or cubic meters",
    ),
    ("SubcatchmentQualitySnapshot", "pollutant_ids"): ("outer matrix axis",),
    ("SubcatchmentQualitySnapshot", "runoff_concentrations"): (
        "pollutant-major",
        "pollutant_ids",
        "object_ids",
    ),
}


def test_all_native_record_names_schemas_docs_and_stub_registration() -> None:
    assert snapshots.__all__ == EXPECTED_RECORD_NAMES
    stub = (Path(__file__).resolve().parents[1] / "src" / "swmmrs" / "_swmmrs.pyi").read_text()

    for name, expected_fields in EXPECTED_SCHEMAS.items():
        record_type = getattr(snapshots, name)
        assert record_type.__module__ == "swmmrs.snapshots"
        assert record_type.__final__ is True
        assert record_type.__doc__ is not None
        assert record_type.__doc__.startswith(
            (
                "Aligned",
                "Cumulative",
                "Current",
                "Routing",
                "System",
                "Continuity",
                "Coherent",
                "Pollutant-major",
            )
        )
        descriptor_names = tuple(
            field_name
            for field_name, descriptor in vars(record_type).items()
            if isinstance(descriptor, GetSetDescriptorType)
        )
        # PyO3 builds ``tp_getset`` through a HashMap, so normalize its implementation-defined
        # type-dictionary order through the explicit public schema before comparing exact tuples.
        actual_fields = tuple(
            field_name for field_name in expected_fields if field_name in descriptor_names
        )
        unexpected_fields = tuple(
            field_name for field_name in descriptor_names if field_name not in expected_fields
        )
        assert actual_fields == expected_fields
        assert unexpected_fields == ()
        for field_name in expected_fields:
            field_doc = getattr(record_type, field_name).__doc__
            assert field_doc is not None
            assert field_doc.startswith("Returns ")
            assert "# Returns" in field_doc
        assert f"class {name}:" in stub

    for (name, field_name), expected_fragments in DOC_EXPECTATIONS.items():
        field_doc = getattr(getattr(snapshots, name), field_name).__doc__
        assert field_doc is not None
        assert all(fragment in field_doc for fragment in expected_fragments)
    storage_loss_doc = snapshots.StorageStatistics.evaporation_losses.__doc__
    routing_error_doc = snapshots.RoutingTotals.continuity_error.__doc__
    assert storage_loss_doc is not None
    assert routing_error_doc is not None
    assert "percentage" not in storage_loss_doc
    assert "project units" not in routing_error_doc


def test_acquired_representative_records_are_reflexive(tmp_path: Path) -> None:
    statistics = swmmrs.Simulation(DATA / "statistics.inp", tmp_path / "statistics.rpt")
    quality = swmmrs.Simulation(DATA / "quality.inp", tmp_path / "quality.rpt")
    lids = swmmrs.Simulation(DATA / "lids.inp", tmp_path / "lids.rpt")
    try:
        for simulation in (statistics, quality, lids):
            simulation.start(save_results=False)
            simulation.step()

        system = statistics.statistics
        acquired = {
            "LidUnitSnapshot": lids.subcatchments["With-Lids"].lid_units[0].snapshot(),
            "LinkQualitySnapshot": quality.links.quality_snapshot(),
            "LinkSnapshot": statistics.links.snapshot(),
            "LinkStatistics": statistics.links["C2"].statistics,
            "LinkStatisticsSnapshot": statistics.links.statistics(),
            "NodeQualitySnapshot": quality.nodes.quality_snapshot(),
            "NodeSnapshot": statistics.nodes.snapshot(),
            "NodeStatistics": statistics.nodes["J1"].statistics,
            "NodeStatisticsSnapshot": statistics.nodes.statistics(),
            "OutfallStatistics": getattr(statistics.nodes["O1"], "outfall_statistics"),
            "PumpStatistics": getattr(statistics.links["Pump-A"], "pump_statistics"),
            "QualityBalance": next(iter(system.quality_balances.values())),
            "RoutingDiagnostics": system.routing_diagnostics,
            "RoutingTotals": system.routing_totals,
            "RunoffTotals": system.runoff_totals,
            "SimulationStatistics": system,
            "StorageStatistics": getattr(statistics.nodes["S1"], "storage_statistics"),
            "SubcatchmentLidSnapshot": lids.subcatchments["With-Lids"].lid_snapshot(),
            "SubcatchmentQualitySnapshot": quality.subcatchments.quality_snapshot(),
            "SubcatchmentSnapshot": statistics.subcatchments.snapshot(),
            "SubcatchmentStatistics": statistics.subcatchments["Sub-A"].statistics,
            "SubcatchmentStatisticsSnapshot": statistics.subcatchments.statistics(),
        }
    finally:
        statistics.close()
        quality.close()
        lids.close()

    assert tuple(acquired) == EXPECTED_RECORD_NAMES
    for name, record in acquired.items():
        assert type(record) is getattr(snapshots, name)
        assert record == record


def test_all_native_records_reject_normal_constructor_forms() -> None:
    for name in EXPECTED_RECORD_NAMES:
        record_type = getattr(snapshots, name)
        for args, kwargs in (
            ((), {}),
            ((None,), {}),
            ((), {"value": None}),
            ((1, 2, 3), {}),
        ):
            with pytest.raises(TypeError, match="cannot create"):
                record_type(*args, **kwargs)
