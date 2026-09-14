from __future__ import annotations

import copy
import gc
import pickle
import warnings
from collections.abc import Mapping
from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs
import swmmrs.objects as objects
from swmmrs.enums import LinkKind, NodeKind
from swmmrs.objects import Conduit, Junction, NodeCollection, ObjectCollection, Outfall

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "solver"
    / "tests"
    / "data"
    / "test_ex1_metric.inp"
)
ALL_FAMILY_FIXTURE = Path(__file__).resolve().parent / "data" / "collections.inp"


def open_simulation(directory: Path) -> swmmrs.Simulation:
    return swmmrs.Simulation(FIXTURE, directory / "model.rpt")


def test_mapping_lookup_preserves_ids_order_and_subtypes(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    nodes = simulation.nodes

    assert isinstance(nodes, Mapping)
    assert isinstance(nodes, NodeCollection)
    assert len(nodes) == 14
    assert list(nodes)[:4] == ["9", "10", "13", "14"]
    assert list(nodes.keys())[:4] == ["9", "10", "13", "14"]
    assert [view.id for view in list(nodes.values())[:2]] == ["9", "10"]
    assert [(identifier, view.id) for identifier, view in list(nodes.items())[:2]] == [
        ("9", "9"),
        ("10", "10"),
    ]
    expected_node_collection = (
        "NodeCollection(['9', '10', '13', '14', '15', '16', '17', '19', "
        "'20', '21', '22', '23', '24', '18'])"
    )
    assert repr(nodes) == expected_node_collection
    assert repr(nodes.keys()) == f"KeysView({expected_node_collection})"
    assert repr(nodes.values()) == f"ValuesView({expected_node_collection})"
    assert repr(nodes.items()) == f"ItemsView({expected_node_collection})"

    by_text = nodes["9"]
    assert isinstance(by_text, Junction)
    assert by_text.id == "9"
    assert by_text.index == 0
    assert repr(by_text) == "Junction(id='9', index=0)"
    assert (
        repr(list(nodes.values())[:2]) == "[Junction(id='9', index=0), Junction(id='10', index=1)]"
    )
    assert nodes[9] == by_text
    assert nodes.by_index(0) == by_text
    assert nodes["9"] is not nodes["9"]

    class StringKey(str):
        pass

    class IntegerKey(int):
        pass

    assert "9" in nodes
    assert 9 in nodes
    assert StringKey("9") in nodes
    assert nodes[StringKey("9")] == by_text
    assert IntegerKey(9) not in nodes
    assert "missing" not in nodes
    assert True not in nodes

    assert isinstance(simulation.nodes["18"], Outfall)
    assert isinstance(simulation.links["1"], Conduit)
    assert isinstance(simulation.pollutants, ObjectCollection)
    assert list(simulation.pollutants) == ["TSS", "Lead"]
    assert simulation.pollutants["lead"].id == "Lead"

    with pytest.raises(KeyError) as missing:
        nodes["missing"]
    assert missing.value.args == ("missing",)
    with pytest.raises(TypeError):
        nodes[True]
    with pytest.raises(TypeError):
        nodes[IntegerKey(9)]  # type: ignore[index]
    with pytest.raises(TypeError):
        nodes[1.0]  # type: ignore[index]
    with pytest.raises(TypeError):
        nodes.by_index(True)
    with pytest.raises(TypeError):
        nodes.by_index(1.0)  # type: ignore[arg-type]
    with pytest.raises(IndexError) as negative:
        nodes.by_index(-1)
    assert negative.value.args == (-1,)
    with pytest.raises(IndexError) as past_end:
        nodes.by_index(len(nodes))
    assert past_end.value.args == (len(nodes),)

    simulation.close()


def test_complete_configured_model_is_discoverable(tmp_path: Path) -> None:
    expected = (
        ("rain_gages", "Gage-A", objects.RainGage),
        ("subcatchments", "Sub-A", objects.Subcatchment),
        ("nodes", "J1", objects.Junction),
        ("links", "Divider-Main", objects.Conduit),
        ("pollutants", "TSS", objects.Pollutant),
        ("land_uses", "Land-A", objects.LandUse),
        ("time_patterns", "Pattern-A", objects.TimePattern),
        ("curves", "PumpCurve", objects.Curve),
        ("time_series", "RainSeries", objects.TimeSeries),
        ("controls", "Control-A", objects.ControlRule),
        ("transects", "Transect-A", objects.Transect),
        ("aquifers", "Aquifer-A", objects.Aquifer),
        ("unit_hydrographs", "Hydrograph-A", objects.UnitHydrograph),
        ("snowmelt_sets", "Snow-A", objects.SnowmeltParameterSet),
        ("shapes", "Shape-A", objects.CustomShape),
        ("lid_controls", "Lid-A", objects.LidControl),
        ("streets", "Street-A", objects.Street),
        ("inlet_designs", "Inlet-A", objects.InletDesign),
    )
    simulation = swmmrs.Simulation(ALL_FAMILY_FIXTURE, tmp_path / "all.rpt")
    for family, identifier, view_type in expected:
        collection = getattr(simulation, family)
        by_index = collection.by_index(0)
        by_key = collection[identifier.lower()]
        assert type(by_index) is view_type
        assert type(by_key) is view_type
        assert by_index.id == identifier
        assert by_key is not collection[identifier]

    assert [type(view) for view in simulation.nodes.values()] == [
        objects.Junction,
        objects.Junction,
        objects.Divider,
        objects.StorageNode,
        *([objects.Outfall] * 8),
    ]
    assert [type(view) for view in simulation.links.values()] == [
        objects.Conduit,
        objects.Conduit,
        objects.Conduit,
        objects.Conduit,
        objects.Pump,
        objects.Orifice,
        objects.Weir,
        objects.Outlet,
    ]
    assert simulation.nodes["divider-a"].kind is NodeKind.DIVIDER
    assert simulation.links["pump-a"].kind is LinkKind.PUMP
    assert list(simulation.curves) == ["PumpCurve", "Shape-A"]
    assert not hasattr(swmmrs, "ObjectKind")
    assert not hasattr(objects, "ObjectKind")
    simulation.close()


def test_custom_shape_lookup_uses_unicode_case_insensitive_ids(tmp_path: Path) -> None:
    model = tmp_path / "unicode-shape.inp"
    # pi-lens-ignore: python-path-traversal
    model.write_text(ALL_FAMILY_FIXTURE.read_text().replace("Shape-A", "ÄShape"))
    simulation = swmmrs.Simulation(model, tmp_path / "unicode-shape.rpt")

    assert list(simulation.shapes) == ["ÄShape"]
    assert "äshape" in simulation.shapes
    assert simulation.shapes["äshape"].id == "ÄShape"

    simulation.close()


def test_view_retains_owner_without_forming_an_owner_cycle(tmp_path: Path) -> None:

    def lookup() -> objects.Node:
        simulation = open_simulation(tmp_path)
        return simulation.nodes["9"]

    view = lookup()
    gc.collect()
    assert view.id == "9"
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", ResourceWarning)
        del view
        gc.collect()


def test_views_never_rebind_and_retain_the_facade(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    old_nodes = simulation.nodes
    old_view = old_nodes["9"]
    equivalent = old_nodes["9"]
    assert old_view == equivalent
    assert hash(old_view) == hash(equivalent)

    del equivalent
    gc.collect()
    assert old_view.id == "9"

    simulation.close()
    with pytest.raises(swmmrs.StaleViewError):
        _ = old_view.id
    with pytest.raises(swmmrs.StaleViewError):
        _ = old_view.index
    with pytest.raises(swmmrs.StaleViewError):
        len(old_nodes)
    with pytest.raises(swmmrs.StaleViewError):
        _ = "9" in old_nodes

    missing = tmp_path / "missing.inp"
    with pytest.raises(swmmrs.SolverError):
        simulation.open(missing)
    with pytest.raises(swmmrs.StaleViewError):
        _ = old_view.id

    simulation.open(FIXTURE, tmp_path / "second.rpt")
    replacement = simulation.nodes["9"]
    assert old_view != replacement
    with pytest.raises(swmmrs.StaleViewError):
        _ = old_view.id
    simulation.close()


def test_facades_and_views_reject_extension_and_serialization(tmp_path: Path) -> None:
    simulation = open_simulation(tmp_path)
    collection = simulation.nodes
    view = collection["9"]

    assert getattr(type(collection), "__final__", False)
    assert getattr(type(view), "__final__", False)
    for value in (collection, view):
        with pytest.raises(AttributeError):
            value.extra = 1  # type: ignore[attr-defined]

    assert simulation.nodes is not simulation.nodes
    simulation.close()
