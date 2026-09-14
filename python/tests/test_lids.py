from __future__ import annotations

import copy
import pickle
from concurrent.futures import ThreadPoolExecutor
from datetime import timedelta
from pathlib import Path
from threading import Barrier

import pytest

import swmmrs
from swmmrs.objects import (
    Curve,
    LidControl,
    LidDrainageMatLayer,
    LidDrainLayer,
    LidPavementLayer,
    LidSoilLayer,
    LidStorageLayer,
    LidSurfaceLayer,
    LidUnit,
    LidUnitCollection,
)

FIXTURE = Path(__file__).resolve().parent / "data" / "lids.inp"


def _simulation(directory: Path, name: str = "model") -> swmmrs.Simulation:
    return swmmrs.Simulation(
        FIXTURE,
        directory / f"{name}.rpt",
        directory / f"{name}.out",
    )


def test_subcatchment_owned_units_are_an_immutable_local_sequence(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    subcatchment = simulation.subcatchments["With-Lids"]
    units = subcatchment.lid_units

    assert isinstance(units, LidUnitCollection)
    assert len(units) == 8
    assert repr(units) == "LidUnitCollection(subcatchment_id='With-Lids', subcatchment_index=0)"

    assert [unit.control.id for unit in units] == [
        "BC",
        "GR",
        "IT",
        "PP",
        "RB",
        "RD",
        "RG",
        "SWALE",
    ]
    assert units[0].index == 0
    assert units[-1].index == 7
    assert isinstance(units[0], LidUnit)
    assert repr(units[0]) == "LidUnit(subcatchment_id='With-Lids', subcatchment_index=0, index=0)"

    assert isinstance(units[1:3], tuple)
    assert [unit.index for unit in units[1:3]] == [1, 2]
    assert units[0] == subcatchment.lid_units[0]
    assert units[0] != units[1]
    assert hash(units[0]) == hash(subcatchment.lid_units[0])
    assert units[2].drain_destination is None
    with pytest.raises(IndexError):
        _ = units[8]
    with pytest.raises(TypeError):
        _ = units["0"]  # type: ignore[index]
    with pytest.raises(TypeError):
        copy.copy(units[0])
    with pytest.raises(TypeError):
        copy.deepcopy(units)
    with pytest.raises(TypeError):
        pickle.dumps(units)
    simulation.close()


def test_lid_unit_properties_and_relationships(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    unit = simulation.subcatchments["With-Lids"].lid_units[0]

    assert unit.area == 50.0
    assert unit.full_width == 10.0
    assert unit.bottom_width == 0.0
    assert unit.initial_saturation == 10.0
    assert unit.impervious_runoff_treated == 10.0
    assert unit.pervious_runoff_treated == 5.0
    assert unit.count == 1
    assert not unit.routes_to_pervious
    assert unit.control == simulation.lid_controls["BC"]
    assert unit.drain_destination == simulation.nodes["J1"]
    unit.area = 50.0
    once = unit.area
    unit.area = 50.0
    assert unit.area == once
    surface = unit.control.surface
    assert isinstance(surface, LidSurfaceLayer)
    surface.roughness = 0.2
    once = (surface.roughness, surface.alpha)
    surface.roughness = 0.2
    assert (surface.roughness, surface.alpha) == once
    simulation.close()


def test_lid_unit_update_is_atomic_and_relationships_remain_live_views(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    unit = simulation.subcatchments["With-Lids"].lid_units[0]

    assert not simulation.configuration_dirty
    unit.update()
    assert not simulation.configuration_dirty

    unit.update(
        area=40.0,
        full_width=12.0,
        initial_saturation=25.0,
        impervious_runoff_treated=8.0,
        pervious_runoff_treated=4.0,
        control=simulation.lid_controls["BC"],
        count=1,
        routes_to_pervious=True,
        drain_destination=simulation.nodes["J1"],
    )
    assert (
        unit.area,
        unit.full_width,
        unit.initial_saturation,
        unit.impervious_runoff_treated,
        unit.pervious_runoff_treated,
        unit.control,
        unit.count,
        unit.routes_to_pervious,
        unit.drain_destination,
    ) == (
        40.0,
        12.0,
        25.0,
        8.0,
        4.0,
        simulation.lid_controls["BC"],
        1,
        True,
        simulation.nodes["J1"],
    )

    before = (unit.area, unit.full_width, unit.count, unit.drain_destination)
    with pytest.raises(swmmrs.ValidationError):
        unit.update(area=35.0, count=0, drain_destination=None)
    assert (unit.area, unit.full_width, unit.count, unit.drain_destination) == before
    with pytest.raises(swmmrs.ValidationError):
        unit.update(bottom_width=1.0)
    for changes in (
        {"area": True},
        {"count": True},
        {"routes_to_pervious": 1},
        {"control": simulation.curves["DrainCurve"]},
        {"drain_destination": simulation.curves["DrainCurve"]},
    ):
        with pytest.raises(swmmrs.ValidationError):
            unit.update(**changes)
    assert (unit.area, unit.full_width, unit.count, unit.drain_destination) == before
    simulation.close()


def test_lid_layer_update_uses_one_coupled_candidate(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    control = simulation.lid_controls["BC"]
    soil = control.soil
    storage = control.storage
    assert soil is not None and storage is not None

    assert not simulation.configuration_dirty
    soil.update()
    assert not simulation.configuration_dirty
    soil.update(porosity=0.7, field_capacity=0.6, wilting_point=0.5)
    assert (soil.porosity, soil.field_capacity, soil.wilting_point) == (0.7, 0.6, 0.5)

    before = (soil.porosity, soil.field_capacity, soil.wilting_point, soil.suction_head)
    soil.update(porosity=0.4, suction_head=8.0)
    assert (soil.porosity, soil.field_capacity, soil.wilting_point, soil.suction_head) == (
        0.4,
        0.6,
        0.5,
        8.0,
    )
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start(save_results=False)
    soil.update(porosity=before[0], suction_head=before[3])
    with pytest.raises(swmmrs.ValidationError):
        soil.update(alpha=1.0)

    clogging_factor = storage.clogging_factor
    storage.update(thickness=storage.thickness * 1.5, void_ratio=storage.void_ratio / 2.0)
    assert storage.clogging_factor == clogging_factor
    simulation.close()


def test_every_lid_layer_update_extracts_native_types_and_rejects_read_only_fields(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path)
    bc = simulation.lid_controls["BC"]
    pp = simulation.lid_controls["PP"]
    gr = simulation.lid_controls["GR"]
    surface = bc.surface
    soil = bc.soil
    storage = bc.storage
    pavement = pp.pavement
    drain = bc.drain
    drainage_mat = gr.drainage_mat
    assert surface and soil and storage and pavement and drain and drainage_mat

    surface.update(
        thickness=7.0,
        vegetation_volume_fraction=0.3,
        roughness=0.15,
        slope=1.5,
        side_slope=4.0,
    )
    soil.update(
        thickness=18.0,
        porosity=0.6,
        field_capacity=0.4,
        wilting_point=0.2,
        saturated_conductivity=0.8,
        conductivity_slope=8.0,
        suction_head=4.0,
    )
    storage.update(
        thickness=10.0,
        void_ratio=0.5,
        saturated_conductivity=0.25,
        clogging_factor=1.5,
    )
    pavement.update(
        thickness=7.0,
        void_ratio=0.2,
        impervious_fraction=0.15,
        saturated_conductivity=0.75,
        clogging_factor=1.25,
        regeneration_interval=timedelta(days=3),
        regeneration_fraction=0.5,
    )
    drain.update(
        coefficient=0.75,
        exponent=0.6,
        offset=1.0,
        delay=timedelta(hours=2),
        open_head=2.0,
        close_head=1.0,
    )
    drainage_mat.update(thickness=4.0, void_fraction=0.6, roughness=0.2)

    assert (
        surface.thickness,
        surface.vegetation_volume_fraction,
        surface.roughness,
        surface.slope,
        surface.side_slope,
    ) == pytest.approx((7.0, 0.3, 0.15, 1.5, 4.0))
    assert (
        soil.thickness,
        soil.porosity,
        soil.field_capacity,
        soil.wilting_point,
        soil.saturated_conductivity,
        soil.conductivity_slope,
        soil.suction_head,
    ) == pytest.approx((18.0, 0.6, 0.4, 0.2, 0.8, 8.0, 4.0))
    assert (
        storage.thickness,
        storage.void_ratio,
        storage.saturated_conductivity,
        storage.clogging_factor,
    ) == pytest.approx((10.0, 0.5, 0.25, 1.5))
    assert (
        pavement.thickness,
        pavement.void_ratio,
        pavement.impervious_fraction,
        pavement.saturated_conductivity,
        pavement.clogging_factor,
        pavement.regeneration_fraction,
    ) == pytest.approx((7.0, 0.2, 0.15, 0.75, 1.25, 0.5))
    assert pavement.regeneration_interval == timedelta(days=3)
    assert (
        drain.coefficient,
        drain.exponent,
        drain.offset,
        drain.open_head,
        drain.close_head,
    ) == pytest.approx((0.75, 0.6, 1.0, 2.0, 1.0))
    assert drain.delay == timedelta(hours=2)
    assert (
        drainage_mat.thickness,
        drainage_mat.void_fraction,
        drainage_mat.roughness,
    ) == pytest.approx((4.0, 0.6, 0.2))

    for layer, changes in [
        (surface, {"alpha": 1.0}),
        (drain, {"control_curve": None}),
        (drainage_mat, {"alpha": 1.0}),
        (pavement, {"regeneration_interval": 1.0}),
    ]:
        with pytest.raises(swmmrs.ValidationError):
            layer.update(**changes)
    with pytest.raises(swmmrs.ValidationError):
        surface.update(roughness=True)
    simulation.close()

    with pytest.raises(swmmrs.StaleViewError):
        surface.update()


def test_retained_lid_layer_view_is_stale_while_optional_layer_is_absent(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path)
    control = simulation.lid_controls["BC"]
    storage = control.storage
    assert storage is not None

    storage.thickness = 0.0
    assert control.storage is None
    with pytest.raises(swmmrs.StaleViewError):
        _ = storage.thickness
    with pytest.raises(swmmrs.StaleViewError):
        storage.update()
    simulation.close()


def test_lid_updates_obey_lifecycle_relationship_and_owner_locking(
    tmp_path: Path,
) -> None:
    simulation = _simulation(tmp_path, "primary")
    foreign = _simulation(tmp_path, "foreign")
    unit = simulation.subcatchments["With-Lids"].lid_units[0]
    surface = simulation.lid_controls["BC"].surface
    drain = simulation.lid_controls["BC"].drain
    assert surface and drain

    before = (unit.control, unit.drain_destination)
    with pytest.raises(swmmrs.ValidationError):
        unit.update(control=foreign.lid_controls["BC"])
    with pytest.raises(swmmrs.ValidationError):
        unit.update(drain_destination=foreign.nodes["J1"])
    assert (unit.control, unit.drain_destination) == before

    first = simulation.lid_controls["BC"].soil
    second = simulation.lid_controls["BC"].soil
    assert first and second
    barrier = Barrier(2)

    def set_conductivity() -> None:
        barrier.wait()
        first.saturated_conductivity = 0.8

    def set_suction() -> None:
        barrier.wait()
        second.suction_head = 4.0

    with ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(set_conductivity), executor.submit(set_suction)]
        for future in futures:
            future.result()
    assert first.saturated_conductivity == 0.8
    assert first.suction_head == 4.0
    assert simulation.configuration_dirty

    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.LifecycleError):
        surface.update(roughness=0.25)
    with pytest.raises(swmmrs.LifecycleError):
        drain.update(coefficient=0.9, delay=timedelta(hours=1))
    with pytest.raises(swmmrs.LifecycleError):
        unit.update(drain_destination=None)
    with pytest.raises(swmmrs.LifecycleError):
        surface.update(roughness=0.3, slope=2.0)
    with pytest.raises(swmmrs.LifecycleError):
        unit.update(area=30.0, drain_destination=simulation.nodes["J1"])
    simulation.end()

    first.update(saturated_conductivity=0.9, suction_head=4.5)
    assert simulation.state is swmmrs.SimulationState.OPEN
    simulation.start(save_results=False)
    simulation.end()
    foreign.close()
    simulation.close()


def test_lid_declarations_rebuild_at_start_and_reject_running_writes(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    subcatchment = simulation.subcatchments["With-Lids"]
    unit = subcatchment.lid_units[2]
    assert unit.routes_to_pervious
    assert unit.drain_destination is None

    unit.routes_to_pervious = True
    subcatchment.settings.impervious_fraction = 0.999
    subcatchment.settings.outlet = simulation.nodes["Out"]
    assert simulation.configuration_dirty
    assert unit.routes_to_pervious
    assert unit.drain_destination is None

    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    with pytest.raises(swmmrs.LifecycleError):
        unit.update(area=40.0)
    surface = unit.control.surface
    assert surface is not None
    with pytest.raises(swmmrs.LifecycleError):
        surface.update(roughness=0.2)
    simulation.end()
    simulation.close()


def test_all_optional_layers_and_exact_public_conversions(tmp_path: Path) -> None:
    simulation = _simulation(tmp_path)
    bc = simulation.lid_controls["BC"]
    gr = simulation.lid_controls["GR"]
    pp = simulation.lid_controls["PP"]
    rb = simulation.lid_controls["RB"]

    assert isinstance(bc, LidControl)
    assert isinstance(bc.surface, LidSurfaceLayer)
    assert isinstance(bc.soil, LidSoilLayer)
    assert isinstance(bc.storage, LidStorageLayer)
    assert bc.pavement is None
    assert isinstance(bc.drain, LidDrainLayer)
    assert bc.drainage_mat is None
    assert isinstance(gr.drainage_mat, LidDrainageMatLayer)
    assert gr.storage is None
    assert gr.pavement is None
    assert gr.drain is None
    assert isinstance(pp.pavement, LidPavementLayer)
    assert rb.surface is None
    assert rb.soil is None
    assert rb.pavement is None
    assert rb.drainage_mat is None

    surface = bc.surface
    soil = bc.soil
    storage = bc.storage
    drain = bc.drain
    pavement = pp.pavement
    drainage_mat = gr.drainage_mat
    assert surface and soil and storage and drain and pavement and drainage_mat
    assert repr(surface) == "LidSurfaceLayer(control_id='BC', control_index=0)"

    assert (
        surface.thickness,
        surface.vegetation_volume_fraction,
        surface.roughness,
        surface.slope,
        surface.side_slope,
    ) == (6.0, 0.25, 0.1, 1.0, 5.0)
    assert surface.alpha > 0.0
    assert isinstance(surface.immediate_overflow, bool)
    assert (
        soil.thickness,
        soil.porosity,
        soil.field_capacity,
        soil.wilting_point,
        soil.saturated_conductivity,
        soil.conductivity_slope,
        soil.suction_head,
    ) == (12.0, 0.5, 0.2, 0.1, 0.5, 10.0, 3.5)
    assert storage.void_ratio == pytest.approx(0.75, abs=1e-7, rel=0)
    assert storage.clogging_factor == 2.0
    assert pavement.void_ratio == pytest.approx(0.15, abs=1e-7, rel=0)
    assert pavement.impervious_fraction == 0.1
    assert pavement.regeneration_interval == timedelta(days=2)
    assert pavement.regeneration_fraction == 0.4
    assert drain.delay == timedelta(hours=6)
    assert drain.control_curve == simulation.curves["DrainCurve"]
    assert isinstance(drain.control_curve, Curve)
    assert drainage_mat.void_fraction == 0.5
    assert drainage_mat.alpha > 0.0
    simulation.close()


def test_explicit_zero_thickness_layer_remains_present_and_writable(tmp_path: Path) -> None:
    input_path = tmp_path / "zero-soil.inp"
    input_path.write_text(
        FIXTURE.read_text().replace("PP      SOIL        12", "PP      SOIL        0", 1)
    )
    simulation = swmmrs.Simulation(input_path, tmp_path / "zero-soil.rpt")

    soil = simulation.lid_controls["PP"].soil
    assert soil is not None
    assert soil.thickness == 0.0
    soil.thickness = 1.0
    assert soil.thickness == 1.0
    simulation.close()

