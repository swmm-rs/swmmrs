from __future__ import annotations

# pyright: reportMissingImports=false
import gc
import inspect
import sys
from collections.abc import MutableMapping
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from types import GetSetDescriptorType, MappingProxyType

import pytest

import swmmrs
from swmmrs.snapshots import LinkQualitySnapshot, NodeQualitySnapshot, SubcatchmentQualitySnapshot

DATA = Path(__file__).resolve().parent / "data"
QUALITY_FIXTURE = DATA / "quality.inp"
ONE_FIXTURE = DATA / "quality_one.inp"
ALL_LINK_TYPES_FIXTURE = DATA / "collections.inp"
ZERO_FIXTURE = DATA / "snapshots.inp"


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


def _simulation(fixture: Path, directory: Path, name: str) -> swmmrs.Simulation:
    return swmmrs.Simulation(fixture, directory / f"{name}.rpt")


def _open_running(directory: Path) -> swmmrs.Simulation:
    simulation = _simulation(QUALITY_FIXTURE, directory, "quality")
    simulation.start(save_results=False)
    simulation.step()
    return simulation


def test_live_mappings_are_canonical_immutable_and_converted(tmp_path: Path) -> None:
    simulation = _open_running(tmp_path)
    node = simulation.nodes["j1"]
    link = simulation.links["c1"]
    subcatchment = simulation.subcatchments["s-a"]

    result_mappings = (
        node.pollut_quality,
        node.inflow_pollutant_concentration,
        node.reactor_pollutant_concentration,
        link.pollut_quality,
        link.reactor_pollutant_concentration,
        link.pollutant_total_load,
        subcatchment.runoff_pollutant_concentration,
        subcatchment.ponded_pollutant_concentration,
        subcatchment.pollutant_buildup,
        subcatchment.pollutant_total_load,
    )
    for mapping in result_mappings:
        assert type(mapping) is MappingProxyType
        assert list(mapping) == ["TSS", "Count"]
        assert all(type(value) is float for value in mapping.values())
        with pytest.raises(TypeError):
            mapping["TSS"] = 1.0  # type: ignore[index]

    for mapping in (
        node.external_pollutant_mass_flux,
        link.external_pollutant_mass_flux,
        subcatchment.external_pollutant_buildup_increment,
    ):
        assert isinstance(mapping, MutableMapping)
        assert list(mapping) == ["TSS", "Count"]

    assert subcatchment.pollutant_buildup["TSS"] >= 0.0
    assert link.pollutant_total_load["Count"] == 0.0
    simulation.close()


def test_node_runtime_quality_uses_named_overrides_and_a_live_persistent_mapping(
    tmp_path: Path,
) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "node-runtime-quality")
    node = simulation.nodes["J1"]
    mass_flux = node.external_pollutant_mass_flux

    assert isinstance(mass_flux, MutableMapping)
    assert list(mass_flux) == ["TSS", "Count"]
    assert mass_flux["tss"] == 0.0
    mass_flux.update([("count", 125.0), ("tss", 2.5)])
    assert dict(mass_flux) == {"TSS": 2.5, "Count": 125.0}

    before = dict(mass_flux)
    with pytest.raises(swmmrs.ValidationError):
        mass_flux["TSS"] = sys.float_info.max
    assert dict(mass_flux) == before
    with pytest.raises(KeyError):
        mass_flux.update([("TSS", 9.0), ("missing", 7.0)])
    with pytest.raises(swmmrs.ValidationError):
        mass_flux.update([("TSS", 9.0), ("tss", 7.0)])
    with pytest.raises(swmmrs.ValidationError):
        mass_flux.update([("TSS", 9.0), ("Count", float("nan"))])
    assert dict(mass_flux) == before

    del mass_flux["tss"]
    assert mass_flux["TSS"] == 0.0
    mass_flux["Count"] = -4.0
    assert dict(mass_flux) == {"TSS": 0.0, "Count": -4.0}
    mass_flux.clear()
    assert dict(mass_flux) == {"TSS": 0.0, "Count": 0.0}
    mass_flux.update()
    assert simulation.state is swmmrs.SimulationState.OPEN

    with ThreadPoolExecutor(max_workers=2) as executor:
        list(
            executor.map(
                lambda item: mass_flux.update([item]),
                (("TSS", 7.0), ("Count", 8.0)),
            )
        )
    assert dict(mass_flux) == {"TSS": 7.0, "Count": 8.0}

    with pytest.raises(AttributeError):
        node.external_pollutant_mass_flux = ("TSS", 1.0)  # type: ignore[misc]
    with pytest.raises(swmmrs.LifecycleError):
        node.override_pollutant_concentrations({"TSS": 123.0})

    simulation.start(save_results=False)
    node.override_pollutant_concentrations([("tss", 123.0), ("Count", 456.0)])
    with pytest.raises(swmmrs.ValidationError):
        node.override_pollutant_concentrations([("TSS", 1.0), ("tss", 2.0)])
    simulation.step()
    assert node.pollut_quality == {"TSS": 123.0, "Count": 456.0}
    simulation.step()
    assert node.pollut_quality["TSS"] != 123.0
    assert node.pollut_quality["Count"] != 456.0
    simulation.close()
    with pytest.raises(swmmrs.StaleViewError):
        _ = mass_flux["TSS"]
    with pytest.raises(swmmrs.StaleViewError):
        mass_flux.update()


def test_link_runtime_quality_uses_named_overrides_and_a_live_persistent_mapping(
    tmp_path: Path,
) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "link-runtime-quality")
    link = simulation.links["C1"]
    mass_flux = link.external_pollutant_mass_flux

    assert isinstance(mass_flux, MutableMapping)
    mass_flux.update(TSS=2.5, Count=125.0)
    assert list(mass_flux) == ["TSS", "Count"]
    assert mass_flux["count"] == 125.0
    before = dict(mass_flux)
    with pytest.raises(swmmrs.ValidationError):
        mass_flux["TSS"] = sys.float_info.max
    with pytest.raises(swmmrs.ValidationError):
        mass_flux.update([("TSS", 8.0), ("Count", True)])
    assert dict(mass_flux) == before
    mass_flux.clear()
    mass_flux["tss"] = -2.0
    assert dict(mass_flux) == {"TSS": -2.0, "Count": 0.0}
    del mass_flux["Count"]
    mass_flux.clear()

    with pytest.raises(AttributeError):
        setattr(link, "pollut_quality", ("TSS", 1.0))
    simulation.start(save_results=False)
    link.override_pollutant_concentrations({"TSS": 456.0, "Count": 123.0})
    simulation.step()
    assert link.pollut_quality == {"TSS": 456.0, "Count": 123.0}
    simulation.step()
    expired_values = list(link.pollut_quality.values())
    assert 456.0 not in expired_values
    assert 123.0 not in expired_values
    simulation.close()


def test_link_pollutant_mass_flux_rejects_links_without_a_conduit_reactor(
    tmp_path: Path,
) -> None:
    simulation = _simulation(ALL_LINK_TYPES_FIXTURE, tmp_path, "unsupported-link-flux")

    for link_id in ("Pump-A", "Orifice-A", "Weir-A", "Outlet-A"):
        mass_flux = simulation.links[link_id].external_pollutant_mass_flux
        before = dict(mass_flux)

        with pytest.raises(swmmrs.ValidationError, match="non-dummy conduit"):
            mass_flux["TSS"] = 1.0

        assert dict(mass_flux) == before
        mass_flux.clear()
        assert dict(mass_flux) == {"TSS": 0.0}

    simulation.close()


def test_subcatchment_persistent_buildup_forcing_is_a_live_mapping(
    tmp_path: Path,
) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "subcatch-runtime-quality")
    subcatchment = simulation.subcatchments["S-A"]
    buildup = subcatchment.external_pollutant_buildup_increment

    assert isinstance(buildup, MutableMapping)
    buildup["tss"] = 3.25
    buildup.update(Count=-2.0)
    assert list(buildup) == ["TSS", "Count"]
    assert dict(buildup) == {"TSS": 3.25, "Count": -2.0}
    before = dict(buildup)
    with pytest.raises(KeyError):
        buildup.update([("TSS", 4.0), ("missing", 2.0)])
    assert dict(buildup) == before
    del buildup["Count"]
    assert buildup["Count"] == 0.0
    buildup.clear()
    buildup["Count"] = 7.0
    assert dict(buildup) == {"TSS": 0.0, "Count": 7.0}
    buildup.clear()
    assert dict(buildup) == {"TSS": 0.0, "Count": 0.0}

    with pytest.raises(AttributeError):
        subcatchment.external_pollutant_buildup_increment = (  # type: ignore[misc]
            "TSS",
            1.0,
        )
    simulation.close()


def test_obsolete_quality_tuple_setters_are_absent(
    tmp_path: Path,
) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "removed-quality-setters")
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]
    subcatchment = simulation.subcatchments["S-A"]

    assert inspect.getattr_static(type(node), "pollut_quality").fset is None
    assert inspect.getattr_static(type(link), "pollut_quality").fset is None
    assert inspect.getattr_static(type(node), "external_pollutant_mass_flux").fset is None
    assert inspect.getattr_static(type(link), "external_pollutant_mass_flux").fset is None
    assert (
        inspect.getattr_static(type(subcatchment), "external_pollutant_buildup_increment").fset
        is None
    )
    simulation.close()


def test_subcatchment_total_load_is_not_converted_twice(tmp_path: Path) -> None:
    input_path = tmp_path / "nonzero-load.inp"
    input_path.write_text(
        QUALITY_FIXTURE.read_text()
        .replace("[BUILDUP]", "[LOADINGS]\nS-A TSS 100\n\n[BUILDUP]")
        .replace("Land-A TSS EXP 0 0 0 0", "Land-A TSS EXP 1 1 0 0")
        .replace("Rain 00:00 0.1", "Rain 00:00 10")
        .replace("Rain 00:10 0.1", "Rain 00:10 10")
    )
    simulation = _simulation(input_path, tmp_path, "nonzero-load")
    simulation.start(save_results=False)
    simulation.step()

    total = simulation.subcatchments["S-A"].pollutant_total_load["TSS"]
    snapshot_total = simulation.subcatchments.quality_snapshot(["S-A"]).total_washoff_loads[0][0]
    assert total == pytest.approx(1.1183105408662597, abs=1e-7, rel=0)
    assert snapshot_total == pytest.approx(1.1183105408662597, abs=1e-7, rel=0)
    simulation.close()


def test_zero_count_totals_remain_finite_in_detailed_report(tmp_path: Path) -> None:
    report_path = tmp_path / "zero-count.rpt"
    simulation = swmmrs.Simulation(QUALITY_FIXTURE, report_path)

    simulation.execute(save_results=True)

    report = report_path.read_text().lower()
    assert "-inf" not in report
    assert " nan" not in report


def test_ponded_concentration_live_mapping_and_snapshot_clear_when_dry(
    tmp_path: Path,
) -> None:
    input_path = tmp_path / "wet-dry.inp"
    input_path.write_text(
        QUALITY_FIXTURE.read_text()
        .replace("TSS  MG/L 0", "TSS  MG/L 100")
        .replace("Rain 00:10 0.1", "Rain 00:01 0.0")
        .replace("[REPORT]", "[EVAPORATION]\nCONSTANT 1000\n\n[REPORT]")
    )
    simulation = _simulation(input_path, tmp_path, "wet-dry")
    subcatchment = simulation.subcatchments["S-A"]
    simulation.start(save_results=False)

    seen_wet = False
    for _ in range(10):
        simulation.step()
        live = subcatchment.ponded_pollutant_concentration["TSS"]
        snapshot = simulation.subcatchments.quality_snapshot(["S-A"])
        assert snapshot.ponded_concentrations[0][0] == live
        seen_wet = seen_wet or live > 0.0
        if seen_wet and live == 0.0:
            break

    assert seen_wet
    assert subcatchment.ponded_pollutant_concentration["TSS"] == 0.0
    assert simulation.subcatchments.quality_snapshot(["S-A"]).ponded_concentrations[0][0] == 0.0
    simulation.close()


def test_persistent_sources_round_trip_atomically_and_reapply(tmp_path: Path) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "persistent")
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]
    subcatchment = simulation.subcatchments["S-A"]

    node.external_pollutant_mass_flux["tss"] = 2.5
    link.external_pollutant_mass_flux["Count"] = 125.0
    subcatchment.external_pollutant_buildup_increment["TSS"] = 3.25
    assert node.external_pollutant_mass_flux["TSS"] == 2.5
    assert link.external_pollutant_mass_flux["Count"] == 125.0
    assert subcatchment.external_pollutant_buildup_increment["TSS"] == 3.25

    invalid_updates = (
        (node.external_pollutant_mass_flux, {"missing": 7.0}),
        (link.external_pollutant_mass_flux, {"TSS": float("nan")}),
        (subcatchment.external_pollutant_buildup_increment, {"TSS": True}),
    )
    for mapping, values in invalid_updates:
        with pytest.raises((KeyError, swmmrs.ValidationError)):
            mapping.update(values)
    assert node.external_pollutant_mass_flux["TSS"] == 2.5
    assert link.external_pollutant_mass_flux["Count"] == 125.0
    assert subcatchment.external_pollutant_buildup_increment["TSS"] == 3.25

    simulation.start(save_results=False)
    assert node.external_pollutant_mass_flux["TSS"] == 2.5
    assert link.external_pollutant_mass_flux["Count"] == 125.0
    simulation.step()
    first_node_concentration = node.pollut_quality["TSS"]
    assert first_node_concentration > 0.0
    assert link.pollut_quality["Count"] > 0.0
    assert subcatchment.pollutant_buildup["TSS"] > 0.0007
    simulation.end()
    node.external_pollutant_mass_flux.update()
    assert simulation.state is swmmrs.SimulationState.ENDED
    node.external_pollutant_mass_flux["TSS"] = 4.5
    assert simulation.state == swmmrs.SimulationState.OPEN
    simulation.start(save_results=False)
    assert node.external_pollutant_mass_flux["TSS"] == 4.5
    assert subcatchment.external_pollutant_buildup_increment["TSS"] == 3.25
    simulation.step()
    assert node.pollut_quality["TSS"] > first_node_concentration
    simulation.close()


def test_persistent_quality_mappings_share_lifecycle_windows(tmp_path: Path) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "mapping-lifecycle")
    mappings = (
        simulation.nodes["J1"].external_pollutant_mass_flux,
        simulation.links["C1"].external_pollutant_mass_flux,
        simulation.subcatchments["S-A"].external_pollutant_buildup_increment,
    )

    for mapping in mappings:
        mapping["TSS"] = 1.0
    simulation.start(save_results=False)
    for mapping in mappings:
        mapping["Count"] = 2.0
    while simulation.step() is not None:
        pass
    assert simulation.state is swmmrs.SimulationState.COMPLETE
    for mapping in mappings:
        with pytest.raises(swmmrs.LifecycleError):
            mapping.update({"TSS": 3.0})

    simulation.end()
    for mapping in mappings:
        mapping.update()
        assert simulation.state is swmmrs.SimulationState.ENDED
    for mapping in mappings:
        mapping["TSS"] = 4.0
    assert simulation.state is swmmrs.SimulationState.OPEN

    simulation.close()
    for mapping in mappings:
        with pytest.raises(swmmrs.StaleViewError):
            list(mapping)
        with pytest.raises(swmmrs.StaleViewError):
            mapping.clear()


def test_one_step_overrides_require_running_and_expire(tmp_path: Path) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "override")
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]
    with pytest.raises(swmmrs.LifecycleError):
        node.override_pollutant_concentrations({"TSS": 123.0})

    simulation.start(save_results=False)
    for owner in (node, link):
        with pytest.raises(swmmrs.ValidationError):
            owner.override_pollutant_concentrations(["TSS", 1.0])  # type: ignore[arg-type]
        with pytest.raises(KeyError):
            owner.override_pollutant_concentrations({"missing": 1.0})
        with pytest.raises(swmmrs.ValidationError):
            owner.override_pollutant_concentrations({"TSS": float("inf")})

    node.override_pollutant_concentrations({"tss": 123.0})
    link.override_pollutant_concentrations({"TSS": 456.0})
    simulation.step()
    assert node.pollut_quality["TSS"] == 123.0
    assert link.pollut_quality["TSS"] == 456.0
    simulation.step()
    assert node.pollut_quality["TSS"] != 123.0
    assert link.pollut_quality["TSS"] != 456.0
    simulation.end()
    with pytest.raises(swmmrs.LifecycleError):
        link.override_pollutant_concentrations({"TSS": 1.0})
    simulation.close()


def test_one_step_override_applies_to_every_public_link_subtype(tmp_path: Path) -> None:
    input_path = tmp_path / "link-subtypes.inp"
    input_path.write_text(
        ALL_LINK_TYPES_FIXTURE.read_text().replace(
            "FLOW_ROUTING         KINWAVE",
            "FLOW_ROUTING         DYNWAVE",
        )
    )
    simulation = _simulation(input_path, tmp_path, "link-subtypes")
    simulation.start(save_results=False)
    links = [
        simulation.links[link_id]
        for link_id in (
            "J-Pipe",
            "Pump-A",
            "Orifice-A",
            "Weir-A",
            "Outlet-A",
        )
    ]
    for link in links:
        link.override_pollutant_concentrations({"TSS": 9876.0})

    simulation.step()

    assert [link.pollut_quality["TSS"] for link in links] == [9876.0] * len(links)
    simulation.close()


def test_one_step_override_applies_to_dummy_conduit(tmp_path: Path) -> None:
    input_path = tmp_path / "dummy.inp"
    input_path.write_text(
        QUALITY_FIXTURE.read_text()
        .replace("FLOW_ROUTING         KINWAVE", "FLOW_ROUTING         DYNWAVE")
        .replace("J1 10 5 0 0 0", "J1 10 5 0 0 0\nJD 10 5 0 0 0")
        .replace("O1 0 FREE NO", "O1 0 FREE NO\nOD 0 FREE NO")
        .replace(
            "C1 J1 O1 100 0.013 0 0 0 0",
            "C1 J1 O1 100 0.013 0 0 0 0\nD1 JD OD 1 0.01 0 0 0 0",
        )
        .replace(
            "C1 CIRCULAR 2 0 0 0 1",
            "C1 CIRCULAR 2 0 0 0 1\nD1 DUMMY 0 0 0 0 1",
        )
    )
    simulation = _simulation(input_path, tmp_path, "dummy")
    simulation.start(save_results=False)
    dummy = simulation.links["D1"]
    dummy.override_pollutant_concentrations({"TSS": 9876.0})

    simulation.step()

    assert dummy.pollut_quality["TSS"] == 9876.0
    simulation.close()


def test_unconsumed_overrides_do_not_cross_end_and_restart(tmp_path: Path) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "restart")
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]
    simulation.start(save_results=False)
    node.override_pollutant_concentrations({"TSS": 9876.0})
    link.override_pollutant_concentrations({"TSS": 9876.0})
    simulation.end()

    simulation.start(save_results=False)
    simulation.step()
    assert node.pollut_quality["TSS"] != 9876.0
    assert link.pollut_quality["TSS"] != 9876.0
    simulation.close()


def test_quality_snapshots_have_exact_shapes_order_and_scalar_parity(tmp_path: Path) -> None:
    simulation = _open_running(tmp_path)
    subcatchments = simulation.subcatchments.quality_snapshot()
    nodes = simulation.nodes.quality_snapshot()
    links = simulation.links.quality_snapshot()

    assert type(subcatchments) is SubcatchmentQualitySnapshot
    assert_native_schema(
        subcatchments,
        (
            "object_ids",
            "pollutant_ids",
            "runoff_concentrations",
            "ponded_concentrations",
            "buildup_loads",
            "total_washoff_loads",
        ),
    )
    assert type(nodes) is NodeQualitySnapshot
    assert_native_schema(
        nodes,
        (
            "object_ids",
            "pollutant_ids",
            "concentrations",
            "inflow_concentrations",
            "reactor_concentrations",
        ),
    )
    assert type(links) is LinkQualitySnapshot
    assert_native_schema(
        links,
        (
            "object_ids",
            "pollutant_ids",
            "concentrations",
            "reactor_concentrations",
            "total_loads",
        ),
    )

    for snapshot in (subcatchments, nodes, links):
        assert snapshot.pollutant_ids == ("TSS", "Count")
        assert not hasattr(snapshot, "_simulation")
        with pytest.raises(AttributeError):
            snapshot.object_ids = ()  # type: ignore[misc]
        for field_name in native_fields(snapshot):
            if field_name in {"object_ids", "pollutant_ids"}:
                continue
            matrix = getattr(snapshot, field_name)
            assert len(matrix) == 2
            assert all(len(row) == len(snapshot.object_ids) for row in matrix)
            with pytest.raises(TypeError):
                matrix[0][0] = 0.0  # type: ignore[index]

    assert nodes.object_ids == tuple(simulation.nodes)
    assert links.object_ids == tuple(simulation.links)
    assert subcatchments.object_ids == tuple(simulation.subcatchments)
    node_position = nodes.object_ids.index("J1")
    link_position = links.object_ids.index("C1")
    subcatch_position = subcatchments.object_ids.index("S-A")
    for pollutant_position, pollutant in enumerate(nodes.pollutant_ids):
        assert (
            nodes.concentrations[pollutant_position][node_position]
            == simulation.nodes["J1"].pollut_quality[pollutant]
        )
        assert (
            nodes.inflow_concentrations[pollutant_position][node_position]
            == simulation.nodes["J1"].inflow_pollutant_concentration[pollutant]
        )
        assert (
            nodes.reactor_concentrations[pollutant_position][node_position]
            == simulation.nodes["J1"].reactor_pollutant_concentration[pollutant]
        )
        assert (
            links.concentrations[pollutant_position][link_position]
            == simulation.links["C1"].pollut_quality[pollutant]
        )
        assert (
            links.reactor_concentrations[pollutant_position][link_position]
            == simulation.links["C1"].reactor_pollutant_concentration[pollutant]
        )
        assert (
            links.total_loads[pollutant_position][link_position]
            == simulation.links["C1"].pollutant_total_load[pollutant]
        )
        subcatchment = simulation.subcatchments["S-A"]
        for field_name, live_name in (
            ("runoff_concentrations", "runoff_pollutant_concentration"),
            ("ponded_concentrations", "ponded_pollutant_concentration"),
            ("buildup_loads", "pollutant_buildup"),
            ("total_washoff_loads", "pollutant_total_load"),
        ):
            assert (
                getattr(subcatchments, field_name)[pollutant_position][subcatch_position]
                == getattr(subcatchment, live_name)[pollutant]
            )
    simulation.close()


def test_snapshot_selection_zero_one_multiple_errors_and_history(tmp_path: Path) -> None:
    simulation = _open_running(tmp_path)
    historical = simulation.nodes.quality_snapshot(["O1", "j1"])
    original = {
        field_name: getattr(historical, field_name) for field_name in native_fields(historical)
    }
    assert historical.object_ids == ("O1", "J1")
    assert simulation.links.quality_snapshot("c1").object_ids == ("C1",)
    empty = simulation.subcatchments.quality_snapshot([])
    assert empty.object_ids == ()
    assert empty.runoff_concentrations == ((), ())
    assert empty.ponded_concentrations == ((), ())
    assert empty.buildup_loads == ((), ())
    assert empty.total_washoff_loads == ((), ())
    with pytest.raises(swmmrs.ValidationError):
        simulation.nodes.quality_snapshot(["J1", "j1"])
    with pytest.raises(KeyError) as caught:
        simulation.nodes.quality_snapshot(["J1", "missing", "O1"])
    assert caught.value.args == ("missing",)
    simulation.step()
    assert historical.concentrations == original["concentrations"]
    simulation.end()
    assert historical.inflow_concentrations == original["inflow_concentrations"]
    simulation.close()
    simulation.open(QUALITY_FIXTURE, tmp_path / "quality-reopened.rpt")
    simulation.close()
    del simulation
    gc.collect()
    assert {
        field_name: getattr(historical, field_name) for field_name in native_fields(historical)
    } == original

    zero = _simulation(ZERO_FIXTURE, tmp_path, "zero")
    zero.start(save_results=False)
    zero.step()
    zero_node = zero.nodes["J1"]
    zero_link = zero.links.by_index(0)
    zero_subcatchment = zero.subcatchments.by_index(0)
    assert zero_node.pollut_quality == {}
    for mapping in (
        zero_node.external_pollutant_mass_flux,
        zero_link.external_pollutant_mass_flux,
        zero_subcatchment.external_pollutant_buildup_increment,
    ):
        assert dict(mapping) == {}
        mapping.update()
        mapping.clear()
        with pytest.raises(KeyError):
            mapping["missing"] = 1.0
    zero_node.override_pollutant_concentrations({})
    zero_link.override_pollutant_concentrations({})
    with pytest.raises(KeyError):
        zero_node.override_pollutant_concentrations({"missing": 1.0})
    with pytest.raises(KeyError):
        zero_link.override_pollutant_concentrations({"missing": 1.0})
    snapshot = zero.nodes.quality_snapshot()
    assert not snapshot.pollutant_ids
    assert not snapshot.concentrations
    assert not snapshot.inflow_concentrations
    assert not snapshot.reactor_concentrations
    zero.close()

    one = _simulation(ONE_FIXTURE, tmp_path, "one")
    one.start(save_results=False)
    one.step()
    snapshot = one.nodes.quality_snapshot([])
    assert snapshot.pollutant_ids == ("TSS",)
    assert snapshot.concentrations == ((),)
    assert snapshot.inflow_concentrations == ((),)
    assert snapshot.reactor_concentrations == ((),)
    one.close()


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
def test_quality_snapshot_rejects_malformed_native_selections(
    tmp_path: Path, family: str, invalid: object
) -> None:
    simulation = _open_running(tmp_path)
    collection = getattr(simulation, family)
    with pytest.raises(swmmrs.ValidationError):
        collection.quality_snapshot(invalid)
    simulation.close()


def test_quality_snapshot_order_duplicates_and_unknowns_cover_all_families(
    tmp_path: Path,
) -> None:
    simulation = _open_running(tmp_path)
    acquisitions = (
        (simulation.subcatchments.quality_snapshot, ["S-A"]),
        (simulation.nodes.quality_snapshot, ["O1", "J1"]),
        (simulation.links.quality_snapshot, ["C1"]),
    )
    for acquisition, ordered_ids in acquisitions:
        assert acquisition(iter(ordered_ids)).object_ids == tuple(ordered_ids)
        with pytest.raises(swmmrs.ValidationError):
            acquisition([ordered_ids[0], ordered_ids[0].lower()])
        with pytest.raises(KeyError) as caught:
            acquisition([ordered_ids[0], "missing", *ordered_ids[1:]])
        assert caught.value.args == ("missing",)
    simulation.close()


def test_quality_snapshot_requires_started_generation_even_when_empty(tmp_path: Path) -> None:
    simulation = _simulation(QUALITY_FIXTURE, tmp_path, "lifecycle")
    for collection in (
        simulation.subcatchments,
        simulation.nodes,
        simulation.links,
    ):
        with pytest.raises(swmmrs.LifecycleError):
            collection.quality_snapshot([])
    simulation.close()
