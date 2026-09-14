# pyright: reportMissingImports=false
from __future__ import annotations

from datetime import timedelta
from pathlib import Path

import pytest

import swmmrs
import swmmrs.objects as objects
from swmmrs.enums import NodeKind
from swmmrs.objects import (
    Divider,
    DividerRule,
    DividerSettings,
    Junction,
    NodeSettings,
    Outfall,
    OutfallBoundary,
    OutfallSettings,
    StorageExfiltration,
    StorageNode,
    StorageSettings,
    StorageShape,
)

FIXTURE = Path(__file__).resolve().parent / "data" / "nodes.inp"
COLLECTION_FIXTURE = Path(__file__).resolve().parent / "data" / "collections.inp"


def open_simulation(directory: Path, fixture: Path = FIXTURE) -> swmmrs.Simulation:
    return swmmrs.Simulation(fixture, directory / "nodes.rpt")


def test_node_collection_selects_all_concrete_views(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    for identifier, expected_type in (
        ("J1", Junction),
        ("S1", StorageNode),
        ("O1", Outfall),
        ("D1", Divider),
    ):
        view = simulation.nodes[identifier]
        assert type(view) is expected_type
        assert type(simulation.nodes.by_index(view.index)) is expected_type
    simulation.close()


def test_common_properties_and_atomic_update(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    node = simulation.nodes["J1"]
    assert type(node) is Junction
    assert node.kind is NodeKind.JUNCTION
    settings = node.settings
    assert type(settings) is NodeSettings
    settings.update()
    assert not simulation.configuration_dirty
    assert (
        settings.invert_elevation,
        settings.full_depth,
        settings.surcharge_depth,
        settings.ponded_area,
        settings.initial_depth,
        settings.included_in_report,
        settings.tag,
    ) == (10.0, 5.0, 0.0, 0.0, 0.0, True, "")

    settings.update(
        tag="monitoring-node",
        invert_elevation=11,
        full_depth=6,
        surcharge_depth=1,
        ponded_area=25,
        initial_depth=0.5,
        included_in_report=False,
    )
    assert (
        settings.tag,
        settings.invert_elevation,
        settings.full_depth,
        settings.surcharge_depth,
        settings.ponded_area,
        settings.initial_depth,
        settings.included_in_report,
    ) == ("monitoring-node", 11.0, 6.0, 1.0, 25.0, 0.5, False)

    before = (settings.tag, settings.full_depth)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(tag="not-committed", full_depth=-1)
    assert (settings.tag, settings.full_depth) == before
    with pytest.raises(swmmrs.ValidationError):
        settings.update(full_depth=True)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(included_in_report=1)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(unknown=1)
    settings.update()

    settings.tag = "setter"
    settings.full_depth = 7
    assert (settings.tag, settings.full_depth) == ("setter", 7.0)
    for removed in ("update", "tag", "invert_elevation", "full_depth"):
        assert not hasattr(node, removed)
    simulation.close()


def test_common_declarations_round_trip_for_all_concrete_node_views(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    values = {
        "J1": (11.0, 6.0, 1.0, 25.0, 0.5),
        "S1": (12.0, 7.0, 1.0, 26.0, 0.5),
        "O1": (13.0, 8.0, 1.0, 27.0, 0.5),
        "D1": (14.0, 9.0, 1.0, 28.0, 0.5),
    }
    for identifier, expected in values.items():
        settings = simulation.nodes[identifier].settings
        settings.update(
            invert_elevation=expected[0],
            full_depth=expected[1],
            surcharge_depth=expected[2],
            ponded_area=expected[3],
            initial_depth=expected[4],
        )
        assert (
            settings.invert_elevation,
            settings.full_depth,
            settings.surcharge_depth,
            settings.ponded_area,
            settings.initial_depth,
        ) == expected
    assert simulation.configuration_dirty
    simulation.close()


def test_project_si_units_round_trip_without_python_conversion(tmp_path: Path) -> None:
    inp = tmp_path / "si.inp"
    inp.write_text(
        FIXTURE.read_text().replace("FLOW_UNITS           CFS", "FLOW_UNITS           CMS")
    )
    simulation = open_simulation(tmp_path, inp)
    node = simulation.nodes["J1"]
    node.settings.update(invert_elevation=3.25, full_depth=1.75, ponded_area=12.5)
    assert node.settings.invert_elevation == pytest.approx(3.25)
    assert node.settings.full_depth == pytest.approx(1.75)
    assert node.settings.ponded_area == pytest.approx(12.5)
    simulation.close()


def test_storage_partial_updates_preserve_canonical_siblings_and_clear(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    storage = simulation.nodes["S1"]
    assert type(storage) is StorageNode
    settings = storage.settings
    assert type(settings) is StorageSettings
    original_shape = settings.shape
    assert original_shape == StorageShape("functional", (0.0, 100.0, 0.0))

    signed_functional = StorageShape("functional", (2.0, -0.5, 1.0))
    settings.shape = signed_functional
    assert settings.shape == signed_functional

    settings.update(
        evaporation_fraction=0.25,
        exfiltration=StorageExfiltration(conductivity=0.1),
    )
    assert settings.shape == signed_functional
    assert settings.evaporation_fraction == 0.25
    assert settings.exfiltration == StorageExfiltration(conductivity=0.1)

    replacement = StorageShape("cylindrical", (6.5, 0.0, 0.0))
    settings.update(shape=replacement, initial_depth=0.25)
    assert settings.shape == replacement
    assert settings.evaporation_fraction == 0.25
    assert settings.exfiltration == StorageExfiltration(conductivity=0.1)

    before = (settings.shape, settings.evaporation_fraction, settings.exfiltration)
    with pytest.raises(swmmrs.ValidationError):
        settings.update(evaporation_fraction=2.0, exfiltration=None)
    assert (settings.shape, settings.evaporation_fraction, settings.exfiltration) == before
    settings.exfiltration = None
    assert settings.exfiltration is None
    for removed in ("shape", "evaporation_fraction", "exfiltration"):
        assert not hasattr(storage, removed)
    simulation.close()


def test_live_relationships_round_trip_and_partial_subtype_updates(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path, COLLECTION_FIXTURE)
    storage = simulation.nodes["Storage-A"]
    outfall = simulation.nodes["Main-Out"]
    divider = simulation.nodes["Divider-A"]
    assert type(storage) is StorageNode
    assert type(outfall) is Outfall
    assert type(divider) is Divider

    curve = simulation.curves["Shape-A"]
    time_series = simulation.time_series["RainSeries"]
    subcatchment = simulation.subcatchments["Sub-A"]
    link = simulation.links["Divider-Div"]

    assert type(storage.settings) is StorageSettings
    assert type(outfall.settings) is OutfallSettings
    assert type(divider.settings) is DividerSettings
    storage.settings.shape = StorageShape("tabular", curve=curve)
    assert storage.settings.shape.curve == curve

    outfall.settings.update(
        boundary=OutfallBoundary("timeseries", reference=time_series),
        has_flap_gate=True,
        route_to_subcatchment=subcatchment,
    )
    assert outfall.settings.boundary == OutfallBoundary("timeseries", reference=time_series)
    assert outfall.settings.has_flap_gate
    assert outfall.settings.route_to_subcatchment == subcatchment
    outfall.settings.boundary = OutfallBoundary("tidal", reference=curve)
    assert outfall.settings.has_flap_gate
    assert outfall.settings.route_to_subcatchment == subcatchment

    divider.settings.update(rule=DividerRule("tabular", curve=curve), diverted_link=link)
    assert divider.settings.rule == DividerRule("tabular", curve=curve)
    assert divider.settings.diverted_link == link
    divider.settings.rule = DividerRule("weir", (1.5, 2.0, 3.0))
    assert divider.settings.diverted_link == link

    outfall.settings.route_to_subcatchment = None
    divider.settings.diverted_link = None
    assert outfall.settings.route_to_subcatchment is None
    assert divider.settings.diverted_link is None
    simulation.close()


def test_relationship_validation_is_atomic(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path, COLLECTION_FIXTURE)
    (tmp_path / "other").mkdir()
    other = open_simulation(tmp_path / "other", COLLECTION_FIXTURE)
    outfall = simulation.nodes["Main-Out"]
    divider = simulation.nodes["Divider-A"]
    assert type(outfall) is Outfall
    assert type(divider) is Divider

    before = outfall.settings.boundary
    with pytest.raises(swmmrs.ValidationError):
        outfall.settings.update(
            boundary=OutfallBoundary("tidal", reference=other.curves["Shape-A"]),
            has_flap_gate=True,
        )
    assert outfall.settings.boundary == before
    assert not outfall.settings.has_flap_gate

    with pytest.raises(swmmrs.ValidationError):
        divider.settings.rule = DividerRule(
            "tabular",
            curve=simulation.time_series["RainSeries"],  # type: ignore[arg-type]
        )
    with pytest.raises(swmmrs.ValidationError):
        divider.settings.diverted_link = simulation.nodes["J1"]  # type: ignore[assignment]

    other.close()
    stale_curve = simulation.curves["Shape-A"]
    simulation.close()
    simulation.open(COLLECTION_FIXTURE, tmp_path / "second.rpt")
    current_outfall = simulation.nodes["Main-Out"]
    assert type(current_outfall) is Outfall
    current_boundary = current_outfall.settings.boundary
    with pytest.raises(swmmrs.ValidationError):
        current_outfall.settings.boundary = OutfallBoundary("tidal", reference=stale_curve)
    assert current_outfall.settings.boundary == current_boundary
    simulation.close()


def test_detached_node_declarations_require_exact_public_classes(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    storage = simulation.nodes["S1"]
    outfall = simulation.nodes["O1"]
    divider = simulation.nodes["D1"]
    assert type(storage) is StorageNode
    assert type(outfall) is Outfall
    assert type(divider) is Divider

    class FakeShape:
        kind = "functional"
        coefficients = (0.0, 100.0, 0.0)
        curve = None

    class FakeExfiltration:
        conductivity = 0.1
        suction_head = 0.0
        moisture_deficit = 0.0

    class FakeBoundary:
        kind = "fixed"
        stage = 1.0
        reference = None

    class FakeRule:
        kind = "overflow"
        values = (0.0, 0.0, 0.0)
        curve = None

    for settings, changes in (
        (storage.settings, {"shape": FakeShape()}),
        (storage.settings, {"exfiltration": FakeExfiltration()}),
        (outfall.settings, {"boundary": FakeBoundary()}),
        (divider.settings, {"rule": FakeRule()}),
    ):
        with pytest.raises(swmmrs.ValidationError):
            settings.update(**changes)
    simulation.close()


def test_wrong_subtype_fields_and_active_updates_are_rejected(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    junction = simulation.nodes["J1"]
    storage = simulation.nodes["S1"]
    before = junction.settings.full_depth
    with pytest.raises(swmmrs.ValidationError):
        junction.settings.update(evaporation_fraction=0.5)
    assert junction.settings.full_depth == before

    simulation.start()
    with pytest.raises(swmmrs.LifecycleError):
        junction.settings.update(full_depth=6.0)
    with pytest.raises(swmmrs.LifecycleError):
        storage.settings.update(evaporation_fraction=0.5)
    simulation.end()
    simulation.close()


def test_deferred_diagnostics_repair_retry_and_ended_rerun(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    junction = simulation.nodes["J1"]
    storage = simulation.nodes["S1"]
    assert isinstance(storage, StorageNode)
    junction.settings.update(initial_depth=6.0)
    storage.settings.update(initial_depth=6.0, evaporation_fraction=0.25)

    with pytest.raises(swmmrs.ConfigurationError) as raised:
        simulation.start()
    assert [record.rule_code for record in raised.value.diagnostics] == [
        "node.initial_depth.maximum_depth",
        "node.initial_depth.maximum_depth",
    ]
    assert simulation.state is swmmrs.SimulationState.OPEN
    assert (junction.settings.initial_depth, storage.settings.initial_depth) == (6.0, 6.0)

    junction.settings.initial_depth = 1.0
    storage.settings.initial_depth = 1.0
    simulation.start()
    simulation.end()
    storage.settings.update(surcharge_depth=1.0, evaporation_fraction=0.5)
    assert simulation.state is swmmrs.SimulationState.OPEN
    simulation.start()
    simulation.end()
    simulation.close()


def test_ended_owner_accepts_subtype_updates_before_rerun(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    outfall = simulation.nodes["O1"]
    divider = simulation.nodes["D1"]
    assert type(outfall) is Outfall
    assert type(divider) is Divider
    simulation.start()
    while simulation.step() is not None:
        pass
    simulation.end()

    outfall.settings.update(boundary=OutfallBoundary("fixed", stage=1.25), has_flap_gate=True)
    divider.settings.update(
        rule=DividerRule("weir", (1.5, 2.0, 3.0)),
        diverted_link=simulation.links["C3"],
    )
    assert outfall.settings.boundary == OutfallBoundary("fixed", stage=1.25)
    assert divider.settings.rule == DividerRule("weir", (1.5, 2.0, 3.0))

    simulation.start()
    assert simulation.step() is not None
    simulation.end()
    assert outfall.settings.boundary == OutfallBoundary("fixed", stage=1.25)
    assert divider.settings.diverted_link == simulation.links["C3"]
    simulation.close()


def test_persistent_external_inflow_is_additive_and_reapplied_each_run(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    node = simulation.nodes["J1"]
    assert node.external_inflow == 0.0
    node.external_inflow = 1
    with pytest.raises(swmmrs.ValidationError):
        node.external_inflow = True  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        node.external_inflow = float("nan")
    assert node.external_inflow == 1.0
    for _ in range(2):
        simulation.start()
        assert node.external_inflow == 1.0
        assert simulation.step() is not None
        assert node.lateral_inflow == pytest.approx(3.0, abs=1e-7, rel=0)
        simulation.end()
    simulation.close()


def test_outfall_fixed_stage_runtime_override_persists_without_replacing_stable_boundary(
    tmp_path: Path,
) -> None:
    inp = tmp_path / "runtime-fixed-stage.inp"
    inp.write_text(
        FIXTURE.read_text()
        + """
[RAINGAGES]
G1 INTENSITY 0:05 1.0 TIMESERIES RainSeries

[TIMESERIES]
RainSeries 00:00 0.0
RainSeries 00:10 0.0

[SUBCATCHMENTS]
Sub1 G1 J1 1.0 0 100 1.0 0

[SUBAREAS]
Sub1 0.01 0.1 0.05 0.05 25 OUTLET

[INFILTRATION]
Sub1 3.0 0.5 4.0 7.0 0
"""
    )
    simulation = open_simulation(tmp_path, inp)
    outfall = simulation.nodes["O1"]
    assert type(outfall) is Outfall
    boundary = OutfallBoundary("timeseries", reference=simulation.time_series["BaseFlow"])
    route = simulation.subcatchments["Sub1"]
    outfall.settings.update(boundary=boundary, has_flap_gate=True, route_to_subcatchment=route)
    assert outfall.fixed_stage is None
    with pytest.raises(swmmrs.ValidationError):
        setattr(outfall, "fixed_stage", None)

    outfall.fixed_stage = 4.25
    with pytest.raises(swmmrs.ValidationError):
        outfall.fixed_stage = True  # type: ignore[assignment]
    assert outfall.fixed_stage == 4.25
    assert (
        outfall.settings.boundary,
        outfall.settings.has_flap_gate,
        outfall.settings.route_to_subcatchment,
    ) == (boundary, True, route)
    outfall.settings.has_flap_gate = False
    assert outfall.fixed_stage == 4.25
    assert (
        outfall.settings.boundary,
        outfall.settings.has_flap_gate,
        outfall.settings.route_to_subcatchment,
    ) == (boundary, False, route)

    simulation.start()
    assert outfall.fixed_stage == 4.25
    assert (
        outfall.settings.boundary,
        outfall.settings.has_flap_gate,
        outfall.settings.route_to_subcatchment,
    ) == (boundary, False, route)
    outfall.fixed_stage = 5
    assert outfall.fixed_stage == 5.0
    simulation.end()
    simulation.start()
    assert outfall.fixed_stage == 5.0
    assert (
        outfall.settings.boundary,
        outfall.settings.has_flap_gate,
        outfall.settings.route_to_subcatchment,
    ) == (boundary, False, route)
    simulation.close()


def test_current_results_are_targeted_and_keep_lifecycles(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    junction = simulation.nodes["J1"]
    storage = simulation.nodes["S1"]
    assert type(storage) is StorageNode
    assert not hasattr(junction, "hydraulic_retention_time")
    assert not hasattr(junction, "fixed_stage")
    assert not hasattr(storage, "fixed_stage")
    for name in (
        "depth",
        "head",
        "volume",
        "lateral_inflow",
        "total_inflow",
        "total_outflow",
        "losses",
        "flooding",
        "total_inflow_volume",
    ):
        with pytest.raises(swmmrs.LifecycleError):
            getattr(junction, name)

    simulation.start()
    assert simulation.step() is not None
    assert storage.hydraulic_retention_time == timedelta(0)
    assert junction.head == pytest.approx(
        junction.settings.invert_elevation + junction.depth, abs=1e-7
    )
    for value in (
        junction.depth,
        junction.volume,
        junction.lateral_inflow,
        junction.total_inflow,
        junction.total_outflow,
        junction.losses,
        junction.flooding,
        junction.total_inflow_volume,
    ):
        assert type(value) is float
    while simulation.step() is not None:
        pass
    simulation.end()
    assert type(junction.depth) is float
    with pytest.raises(swmmrs.LifecycleError):
        _ = junction.total_inflow_volume
    simulation.close()


def test_each_access_revalidates_generation_and_identity(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    old_node = simulation.nodes["J1"]
    simulation.close()
    simulation.open(FIXTURE, tmp_path / "second.rpt")
    old_settings = old_node.settings
    for operation in (
        lambda: old_settings.invert_elevation,
        lambda: old_node.depth,
        lambda: old_settings.update(full_depth=6.0),
        lambda: setattr(old_node, "external_inflow", 1.0),
    ):
        with pytest.raises(swmmrs.StaleViewError):
            operation()
    simulation.close()


def test_removed_configuration_names_are_absent() -> None:
    for name in (
        "NodeConfiguration",
        "StorageConfiguration",
        "OutfallConfiguration",
        "DividerConfiguration",
    ):
        assert not hasattr(objects, name)
    assert not hasattr(Junction, "configuration")
    assert not hasattr(Junction, "configure")
    assert not hasattr(StorageNode, "storage_configuration")
    assert not hasattr(Outfall, "outfall_configuration")
    assert not hasattr(Divider, "divider_configuration")
    for view_type, removed in (
        (
            Junction,
            (
                "update",
                "tag",
                "invert_elevation",
                "full_depth",
                "surcharge_depth",
                "ponded_area",
                "initial_depth",
                "included_in_report",
            ),
        ),
        (StorageNode, ("shape", "evaporation_fraction", "exfiltration")),
        (Outfall, ("boundary", "has_flap_gate", "route_to_subcatchment")),
        (Divider, ("rule", "diverted_link")),
    ):
        for name in removed:
            assert not hasattr(view_type, name)
