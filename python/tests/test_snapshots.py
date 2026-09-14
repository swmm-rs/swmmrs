from __future__ import annotations

# pyright: reportMissingImports=false
import gc
from datetime import timedelta
from pathlib import Path
from types import GetSetDescriptorType

import pytest

import swmmrs
from swmmrs.snapshots import LinkSnapshot, NodeSnapshot, SubcatchmentSnapshot

FIXTURE = Path(__file__).resolve().parent / "data" / "snapshots.inp"


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
    representation = repr(record)
    positions = [representation.index(f"{name}=") for name in expected]
    assert positions == sorted(positions)


def open_running(directory: Path) -> swmmrs.Simulation:
    simulation = swmmrs.Simulation(FIXTURE, directory / "snapshots.rpt")
    simulation.start(save_results=False)
    simulation.step()
    return simulation


def test_exact_fixed_shapes_full_order_units_and_mixed_subtypes(tmp_path: Path) -> None:
    simulation = open_running(tmp_path)

    subcatchments = simulation.subcatchments.snapshot()
    assert type(subcatchments) is SubcatchmentSnapshot
    assert_native_schema(
        subcatchments,
        (
            "object_ids",
            "rainfall",
            "evaporation",
            "infiltration",
            "runon",
            "runoff",
            "snow_depth",
        ),
    )
    assert subcatchments.object_ids == tuple(simulation.subcatchments)
    for position, identifier in enumerate(subcatchments.object_ids):
        subcatchment = simulation.subcatchments[identifier]
        for field_name in native_fields(subcatchments):
            if field_name != "object_ids":
                assert getattr(subcatchments, field_name)[position] == getattr(
                    subcatchment, field_name
                )

    nodes = simulation.nodes.snapshot()
    assert type(nodes) is NodeSnapshot
    assert_native_schema(
        nodes,
        (
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
    )
    assert nodes.object_ids == tuple(simulation.nodes)
    assert len(nodes.hydraulic_retention_time) == len(nodes.object_ids)
    for position, identifier in enumerate(nodes.object_ids):
        node = simulation.nodes[identifier]
        for field_name in native_fields(nodes):
            if field_name != "object_ids":
                assert getattr(nodes, field_name)[position] == getattr(node, field_name, None)
    assert sum(value is not None for value in nodes.hydraulic_retention_time) == 2
    assert all(
        value is None or isinstance(value, timedelta) for value in nodes.hydraulic_retention_time
    )

    links = simulation.links.snapshot()
    assert type(links) is LinkSnapshot
    assert_native_schema(
        links,
        (
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
    )
    assert links.object_ids == tuple(simulation.links)
    assert len(links.top_width) == len(links.object_ids)
    for position, identifier in enumerate(links.object_ids):
        link = simulation.links[identifier]
        for field_name in native_fields(links):
            if field_name != "object_ids":
                assert getattr(links, field_name)[position] == getattr(link, field_name, None)
    assert any(value is None for value in links.top_width)
    assert any(value is not None for value in links.top_width)
    simulation.close()


def test_native_records_are_nonconstructible_frozen_value_objects(tmp_path: Path) -> None:
    simulation = open_running(tmp_path)
    first = simulation.nodes.snapshot(["J1"])
    second = simulation.nodes.snapshot(["j1"])

    assert first == second
    expected_order = (
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
    )
    expected_repr = (
        "NodeSnapshot("
        + ", ".join(f"{field_name}={getattr(first, field_name)!r}" for field_name in expected_order)
        + ")"
    )
    assert repr(first) == expected_repr
    assert first != simulation.nodes.snapshot(["S1"])
    assert first != object()
    assert not hasattr(first, "__dict__")
    with pytest.raises(TypeError):
        getattr(NodeSnapshot, "__new__")(NodeSnapshot)
    with pytest.raises(AttributeError):
        first.depth = ()  # type: ignore[misc]

    simulation.close()


def test_subset_scalar_empty_and_atomic_identity_validation(tmp_path: Path) -> None:
    simulation = open_running(tmp_path)

    subset = simulation.nodes.snapshot([123, "j1"])
    assert subset.object_ids == ("123", "J1")
    assert simulation.subcatchments.snapshot("sub-a").object_ids == ("Sub-A",)
    assert simulation.links.snapshot(identifier for identifier in ("Pump-A", "C1")).object_ids == (
        "Pump-A",
        "C1",
    )

    for snapshot in (
        simulation.subcatchments.snapshot([]),
        simulation.nodes.snapshot(()),
        simulation.links.snapshot(iter(())),
    ):
        for field_name in native_fields(snapshot):
            assert getattr(snapshot, field_name) == ()

    for collection, ordered_ids in (
        (simulation.subcatchments, ["Sub-A"]),
        (simulation.nodes, ["J1", "S1"]),
        (simulation.links, ["Pump-A", "C1"]),
    ):
        with pytest.raises(KeyError) as caught:
            collection.snapshot([ordered_ids[0], "missing", *ordered_ids[1:]])
        assert caught.value.args == ("missing",)
    assert simulation.nodes.snapshot("J1").object_ids == ("J1",)

    simulation.close()


@pytest.mark.parametrize(
    ("family", "duplicate"),
    [
        pytest.param("subcatchments", ["Sub-A", "sub-a"], id="subcatchment-alias"),
        pytest.param("nodes", ["J1", "j1"], id="node-text-alias"),
        pytest.param("nodes", [123, "123"], id="node-numeric-alias"),
        pytest.param("links", ["C1", "c1"], id="link-alias"),
    ],
)
def test_snapshot_rejects_duplicate_identity(
    tmp_path: Path, family: str, duplicate: list[str | int]
) -> None:
    simulation = open_running(tmp_path)
    with pytest.raises(swmmrs.ValidationError):
        getattr(simulation, family).snapshot(duplicate)
    simulation.close()


@pytest.mark.parametrize("family", ["subcatchments", "nodes", "links"])
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
def test_snapshot_rejects_invalid_selection(tmp_path: Path, family: str, invalid: object) -> None:
    simulation = open_running(tmp_path)
    with pytest.raises(swmmrs.ValidationError):
        getattr(simulation, family).snapshot(invalid)
    simulation.close()


def test_snapshots_are_deeply_immutable_historical_and_owner_free(tmp_path: Path) -> None:
    simulation = open_running(tmp_path)
    collection = simulation.links
    historical = collection.snapshot(["Pump-A", "C1"])
    original = {
        field_name: getattr(historical, field_name) for field_name in native_fields(historical)
    }

    with pytest.raises(AttributeError):
        historical.flow = ()  # type: ignore[misc]
    with pytest.raises(TypeError):
        historical.object_ids[0] = "changed"  # type: ignore[index]
    assert_native_schema(
        historical,
        (
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
    )
    assert not hasattr(historical, "_simulation")
    assert not hasattr(historical, "_generation")

    assert {
        field_name: getattr(historical, field_name) for field_name in native_fields(historical)
    } == original
    simulation.step()
    assert historical.flow == original["flow"]
    simulation.end()
    assert historical.time_open == original["time_open"]
    simulation.close()
    assert historical.top_width == original["top_width"]

    with pytest.raises(swmmrs.StaleViewError):
        collection.snapshot([])
    simulation.open(FIXTURE, tmp_path / "reopened.rpt")
    with pytest.raises(swmmrs.StaleViewError):
        collection.snapshot(["Pump-A"])
    assert historical.target_setting == original["target_setting"]
    simulation.close()
    del simulation
    gc.collect()
    assert {
        field_name: getattr(historical, field_name) for field_name in native_fields(historical)
    } == original


def test_acquisition_requires_started_generation_even_for_empty_selection(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "lifecycle.rpt")
    for collection in (
        simulation.subcatchments,
        simulation.nodes,
        simulation.links,
    ):
        with pytest.raises(swmmrs.LifecycleError):
            collection.snapshot([])
    simulation.start(save_results=False)
    while simulation.step() is not None:
        pass
    for collection in (
        simulation.subcatchments,
        simulation.nodes,
        simulation.links,
    ):
        assert len(collection.snapshot().object_ids) == len(collection)
    simulation.end()
    for collection in (
        simulation.subcatchments,
        simulation.nodes,
        simulation.links,
    ):
        assert len(collection.snapshot().object_ids) == len(collection)
    simulation.close()
