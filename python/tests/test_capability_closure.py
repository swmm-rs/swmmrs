from __future__ import annotations

# pyright: reportMissingImports=false
import inspect
from enum import StrEnum
from pathlib import Path
from types import GetSetDescriptorType

import pytest

import swmmrs
import swmmrs.enums as enums
import swmmrs.exceptions as exceptions
import swmmrs.objects as objects
import swmmrs.snapshots as snapshots


def test_public_import_contract_is_complete_and_native_free() -> None:
    assert enums.SimulationState is swmmrs.SimulationState
    assert enums.SolverErrorCode is swmmrs.SolverErrorCode
    assert "HOTSTART_FILE_WRITE" not in enums.SolverErrorCode.__members__
    for name in exceptions.__all__:
        assert getattr(exceptions, name) is getattr(swmmrs, name)

    forbidden = {
        "NativeSimulation",
        "Output",
        "PyO3",
        "SimulationStateRaw",
        "SwmmState",
        "callback_registry",
        "get_value",
        "set_value",
    }
    for module in (swmmrs, enums, exceptions, objects, snapshots):
        assert forbidden.isdisjoint(module.__all__)
        assert not hasattr(module, "_swmmrs")

    for removed in (
        "AquiferConfiguration",
        "AquiferPatch",
        "SnowmeltConfiguration",
        "SnowmeltPatch",
        "SnowmeltSurfaceConfiguration",
        "SubcatchmentSnowpackSettings",
        "NodeConfiguration",
        "StorageConfiguration",
        "OutfallConfiguration",
        "DividerConfiguration",
    ):
        assert removed not in objects.__all__
        assert not hasattr(objects, removed)


def test_every_concrete_public_class_is_final_and_attribute_closed() -> None:
    concrete = [
        swmmrs.Simulation,
        swmmrs.StaleViewError,
        swmmrs.ValidationError,
        swmmrs.SolverError,
        swmmrs.InternalSimulationError,
    ]
    concrete.extend(getattr(enums, name) for name in enums.__all__)
    concrete.extend(
        getattr(objects, name)
        for name in objects.__all__
        if name not in {"Node", "NodeSettings", "Link", "LinkSettings"}
    )
    concrete.extend(getattr(snapshots, name) for name in snapshots.__all__)

    native_records = {getattr(snapshots, name) for name in snapshots.__all__}
    for class_ in concrete:
        assert getattr(class_, "__final__", False)
        if class_ in native_records:
            assert class_.__dictoffset__ == 0
        elif not issubclass(class_, StrEnum):
            assert hasattr(class_, "__slots__")

    errors: tuple[swmmrs.SwmmError, ...] = (
        swmmrs.StaleViewError(),
        swmmrs.ValidationError(),
        swmmrs.SolverError(swmmrs.SolverErrorCode.INPUT, "open"),
        swmmrs.InternalSimulationError(),
    )
    for error in errors:
        with pytest.raises(AttributeError):
            error.unsupported = True  # type: ignore[attr-defined]

    for record_type in native_records:
        with pytest.raises(TypeError):
            getattr(record_type, "__new__")(record_type)


def test_snapshot_field_inventory_is_exact() -> None:
    expected = {
        snapshots.SubcatchmentSnapshot: (
            "object_ids",
            "rainfall",
            "evaporation",
            "infiltration",
            "runon",
            "runoff",
            "snow_depth",
        ),
        snapshots.NodeSnapshot: (
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
        snapshots.LinkQualitySnapshot: (
            "object_ids",
            "pollutant_ids",
            "concentrations",
            "reactor_concentrations",
            "total_loads",
        ),
        snapshots.RunoffTotals: (
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
        snapshots.RoutingDiagnostics: (
            "average_time_step",
            "minimum_time_step",
            "maximum_time_step",
            "step_count",
            "nonconverged_step_count",
            "nonconverged_step_percentage",
            "average_iterations",
        ),
        snapshots.QualityBalance: (
            "continuity_error",
            "seepage_loss",
        ),
        snapshots.SimulationStatistics: (
            "routing_totals",
            "runoff_totals",
            "groundwater_continuity_error",
            "quality_continuity_error",
            "routing_diagnostics",
            "quality_balances",
        ),
    }
    for record, names in expected.items():
        public_fields = {
            name
            for name, descriptor in vars(record).items()
            if isinstance(descriptor, GetSetDescriptorType)
        }
        assert public_fields == set(names)


def test_percent_complete_is_a_derived_started_lifecycle_fact(tmp_path: Path) -> None:
    fixture = (
        Path(__file__).resolve().parents[2]
        / "crates"
        / "solver"
        / "tests"
        / "data"
        / "test_ex1_metric.inp"
    )
    simulation = swmmrs.Simulation(fixture, tmp_path / "progress.rpt")
    with pytest.raises(swmmrs.LifecycleError):
        _ = simulation.percent_complete
    simulation.start(save_results=False)
    assert simulation.percent_complete == 0.0
    assert simulation.stride(3600) is not None
    assert simulation.percent_complete > 0.0
    assert simulation.percent_complete < 100.0
    while simulation.stride(86400) is not None:
        pass
    assert simulation.percent_complete == 100.0
    simulation.end()
    assert simulation.percent_complete == 100.0
    simulation.close()


def test_public_modules_export_only_declared_names() -> None:
    for module in (enums, exceptions, objects, snapshots):
        public_names = {name for name in vars(module) if not name.startswith("_")}
        assert public_names == set(module.__all__)
        for name in module.__all__:
            if inspect.isclass(getattr(module, name)):
                assert getattr(module, name).__module__ == module.__name__
