from __future__ import annotations

import math
from datetime import timedelta
from pathlib import Path
from typing import cast

import pytest  # pyright: ignore[reportMissingImports]

import swmmrs
from swmmrs.enums import (
    LinkKind,
    OrificeKind,
    OutletHeadBasis,
    RoadSurface,
    StandardCrossSectionShape,
    WeirKind,
)
from swmmrs.objects import (
    CircularCrossSection,
    Conduit,
    CustomCrossSection,
    CustomShape,
    FunctionalOutletRating,
    IrregularCrossSection,
    Orifice,
    Outlet,
    Pump,
    StandardCrossSection,
    StreetCrossSection,
    TabularOutletRating,
    Weir,
)

ROOT = Path(__file__).resolve().parents[2]
COLLECTIONS_FIXTURE = Path(__file__).resolve().parent / "data" / "collections.inp"
REGULATOR_FIXTURE = Path(__file__).resolve().parent / "data" / "regulator_configurations.inp"
CONDUIT_FIXTURE = ROOT / "crates" / "solver" / "tests" / "data" / "test_ex1_metric.inp"
ORIFICE_FIXTURE = (
    ROOT / "crates" / "solver" / "tests" / "data" / "node_constantinflow_constanteffluent.inp"
)
PUMP_FIXTURE = (
    ROOT
    / "crates"
    / "run"
    / "tests"
    / "data"
    / "regression-suite"
    / "routing"
    / "pump-storage-table-matrix.inp"
)


def open_simulation(
    fixture: Path, directory: Path, report_name: str = "model.rpt"
) -> swmmrs.Simulation:
    return swmmrs.Simulation(fixture, directory / report_name)


def assert_common_results(link: Conduit | Pump | Orifice) -> None:
    assert math.isfinite(link.setting)
    assert math.isfinite(link.target_setting)
    assert isinstance(link.time_open, timedelta)
    assert isinstance(link.time_closed, timedelta)
    assert math.isfinite(link.flow)
    assert math.isfinite(link.depth)
    assert math.isfinite(link.velocity)
    assert math.isfinite(link.volume)
    assert math.isfinite(link.capacity)
    assert math.isfinite(link.upstream_surface_area)
    assert math.isfinite(link.downstream_surface_area)
    assert math.isfinite(link.froude_number)


def test_link_settings_namespace_and_removed_configuration_surface(tmp_path: Path) -> None:
    simulation = open_simulation(COLLECTIONS_FIXTURE, tmp_path)
    conduit = simulation.links["J-Pipe"]
    pump = simulation.links["Pump-A"]
    orifice = simulation.links["Orifice-A"]
    weir = simulation.links["Weir-A"]
    outlet = simulation.links["Outlet-A"]

    assert isinstance(conduit, Conduit)
    assert isinstance(pump, Pump)
    assert isinstance(orifice, Orifice)
    assert isinstance(weir, Weir)
    assert isinstance(outlet, Outlet)
    assert conduit.kind is LinkKind.CONDUIT
    assert conduit.settings.inlet_node.id == "J1"
    assert conduit.settings.outlet_node.id == "J-Out"
    assert conduit.settings.flow_direction == 1
    assert conduit.settings.length == 100.0
    assert conduit.settings.slope == pytest.approx(0.10050378152592121, abs=1e-7, rel=0)
    assert conduit.settings.full_depth == 2.0
    assert conduit.settings.full_flow > 0.0
    assert conduit.settings.tag == ""
    assert conduit.settings.roughness == pytest.approx(0.013)
    assert conduit.settings.barrels == 1
    assert conduit.settings.cross_section == CircularCrossSection(2.0)
    assert pump.settings.curve == simulation.curves["PumpCurve"]
    assert orifice.settings.kind is OrificeKind.SIDE
    assert weir.settings.kind is WeirKind.TRANSVERSE
    assert weir.settings.roadway_surface is RoadSurface.UNSPECIFIED
    assert isinstance(outlet.settings.rating, FunctionalOutletRating)

    for link in (conduit, pump, orifice, weir, outlet):
        assert not hasattr(link, "configuration")
        assert not hasattr(link, "configure")
        assert not hasattr(link, "tag")
        assert not hasattr(link, "included_in_report")
    assert not hasattr(conduit, "length")
    assert not hasattr(pump, "pump_configuration")
    assert not hasattr(orifice, "orifice_configuration")
    assert not hasattr(weir, "weir_configuration")
    assert not hasattr(outlet, "rating")

    simulation.close()


def test_conduit_settings_mixed_update_is_atomic_and_round_trips_cross_sections(
    tmp_path: Path,
) -> None:
    simulation = open_simulation(COLLECTIONS_FIXTURE, tmp_path)
    conduit = simulation.links["J-Pipe"]
    assert isinstance(conduit, Conduit)

    conduit.settings.update(
        tag="calibration-link",
        included_in_report=False,
        inlet_offset=1.0,
        outlet_offset=1.5,
        initial_flow=-0.25,
        inlet_loss_coefficient=0.1,
        outlet_loss_coefficient=0.2,
        average_loss_coefficient=0.3,
        seepage_rate=0.025,
        has_flap_gate=True,
        length=125.0,
        roughness=0.014,
        barrels=2,
        cross_section=CircularCrossSection(2.5),
    )
    assert conduit.settings.tag == "calibration-link"
    assert not conduit.settings.included_in_report
    assert conduit.settings.inlet_offset == 1.0
    assert conduit.settings.outlet_offset == 1.5
    assert conduit.settings.initial_flow == -0.25
    assert conduit.settings.inlet_loss_coefficient == pytest.approx(0.1)
    assert conduit.settings.outlet_loss_coefficient == pytest.approx(0.2)
    assert conduit.settings.average_loss_coefficient == pytest.approx(0.3)
    assert conduit.settings.seepage_rate == pytest.approx(0.025)
    assert conduit.settings.has_flap_gate
    assert conduit.settings.length == 125.0
    assert conduit.settings.roughness == pytest.approx(0.014)
    assert conduit.settings.barrels == 2
    assert conduit.settings.cross_section == CircularCrossSection(2.5)

    before = (conduit.settings.tag, conduit.settings.length, conduit.settings.barrels)
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.update(tag="should-not-commit", length=float("nan"))
    for barrels in (True, 128, 255):
        with pytest.raises(swmmrs.ValidationError):
            conduit.settings.update(tag="should-not-commit", barrels=barrels)
        assert (conduit.settings.tag, conduit.settings.length, conduit.settings.barrels) == before

    declarations = (
        StandardCrossSection(
            StandardCrossSectionShape.RECT_OPEN,
            (2.75, 3.0, 0.0, 0.0),
            culvert_code=1,
        ),
        CustomCrossSection(simulation.shapes["Shape-A"], 3.0, culvert_code=2),
        IrregularCrossSection(simulation.transects["Transect-A"], culvert_code=3),
        StreetCrossSection(simulation.streets["Street-A"], culvert_code=4),
        CircularCrossSection(3.25),
    )
    for declaration in declarations:
        conduit.settings.cross_section = declaration
        assert conduit.settings.cross_section == declaration

    simulation.close()


def test_pump_mode_operations_preserve_scalars_and_validate_relationships(tmp_path: Path) -> None:
    simulation = open_simulation(REGULATOR_FIXTURE, tmp_path)
    pump = simulation.links["Pump-A"]
    assert isinstance(pump, Pump)

    pump.settings.update(initial_setting=0.75, startup_depth=2.0, shutoff_depth=1.0)
    initial = (
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
    )
    pump.settings.use_ideal()
    assert pump.settings.curve is None
    assert (
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
    ) == initial
    pump.settings.use_curve(simulation.curves.by_index(0))
    assert pump.settings.curve == simulation.curves.by_index(0)
    assert (
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
    ) == initial

    with pytest.raises(swmmrs.ValidationError):
        pump.settings.update(curve=simulation.curves.by_index(0))
    other = open_simulation(REGULATOR_FIXTURE, tmp_path, "other.rpt")
    with pytest.raises(swmmrs.ValidationError):
        pump.settings.use_curve(other.curves.by_index(0))
    other.close()

    simulation.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        pump.settings.use_ideal()
    simulation.end()
    pump.settings.use_ideal()
    simulation.start(save_results=False)
    simulation.end()
    simulation.close()
    with pytest.raises(swmmrs.StaleViewError):
        pump.settings.use_ideal()


def test_regulator_settings_and_outlet_rating_round_trip(tmp_path: Path) -> None:
    simulation = open_simulation(REGULATOR_FIXTURE, tmp_path)
    orifice = simulation.links["Orifice-A"]
    weir = simulation.links["Weir-A"]
    outlet = simulation.links["Outlet-A"]
    assert isinstance(orifice, Orifice)
    assert isinstance(weir, Weir)
    assert isinstance(outlet, Outlet)

    orifice.settings.update(
        kind=OrificeKind.BOTTOM,
        discharge_coefficient=0.65,
        opening_time_hours=0.5,
        tag="orifice",
    )
    assert orifice.settings.kind is OrificeKind.BOTTOM
    assert orifice.settings.discharge_coefficient == pytest.approx(0.65)
    with pytest.raises(swmmrs.ValidationError):
        orifice.settings.kind = "side"  # type: ignore[assignment]
    assert orifice.settings.kind is OrificeKind.BOTTOM
    assert orifice.settings.opening_time_hours == pytest.approx(0.5)
    assert orifice.settings.tag == "orifice"

    coefficient_curve = simulation.curves.by_index(0)
    weir.settings.update(
        kind=WeirKind.ROADWAY,
        discharge_coefficient=3.0,
        end_discharge_coefficient=1.0,
        end_contractions=2.0,
        can_surcharge=False,
        roadway_width=12.0,
        roadway_surface=RoadSurface.PAVED,
        coefficient_curve=coefficient_curve,
    )
    assert weir.settings.kind is WeirKind.ROADWAY
    assert weir.settings.roadway_surface is RoadSurface.PAVED
    assert weir.settings.coefficient_curve == coefficient_curve
    weir.settings.coefficient_curve = None
    assert weir.settings.coefficient_curve is None

    functional = FunctionalOutletRating(2.0, 1.5, OutletHeadBasis.HEAD)
    outlet.settings.rating = functional
    assert outlet.settings.rating == functional
    tabular = TabularOutletRating(simulation.curves.by_index(2), OutletHeadBasis.DEPTH)
    outlet.settings.rating = tabular
    assert outlet.settings.rating == tabular

    before = (orifice.settings.tag, orifice.settings.discharge_coefficient)
    with pytest.raises(swmmrs.ValidationError):
        orifice.settings.update(tag="bad", discharge_coefficient=-1.0)
    assert (orifice.settings.tag, orifice.settings.discharge_coefficient) == before
    with pytest.raises(swmmrs.ValidationError):
        outlet.settings.update(rating=object())
    simulation.close()


def test_regulator_relationship_semantics_remain_post_open_diagnostics(tmp_path: Path) -> None:
    simulation = open_simulation(REGULATOR_FIXTURE, tmp_path)
    pump = simulation.links["Pump-A"]
    outlet = simulation.links["Outlet-A"]
    assert isinstance(pump, Pump)
    assert isinstance(outlet, Outlet)

    pump.settings.use_curve(simulation.curves.by_index(1))
    outlet.settings.rating = TabularOutletRating(simulation.curves.by_index(0))
    with pytest.raises(swmmrs.ConfigurationError) as error:
        simulation.start(save_results=False)
    assert [diagnostic.rule_code for diagnostic in error.value.diagnostics] == [
        "link.pump.curve_type",
        "link.outlet.curve_type",
    ]

    pump.settings.use_curve(simulation.curves.by_index(0))
    outlet.settings.rating = TabularOutletRating(simulation.curves.by_index(2))
    simulation.start(save_results=False)
    assert not simulation.configuration_dirty
    simulation.end()
    simulation.close()


def test_link_settings_lifecycle_empty_unknown_and_foreign_topology(tmp_path: Path) -> None:
    simulation = open_simulation(CONDUIT_FIXTURE, tmp_path)
    conduit = simulation.links["1"]
    assert isinstance(conduit, Conduit)
    conduit.settings.update()
    assert not simulation.configuration_dirty

    before = conduit.settings.length
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.update(unknown_field=1.0)
    assert conduit.settings.length == before

    other = open_simulation(CONDUIT_FIXTURE, tmp_path, "other.rpt")
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.update(inlet_node=other.nodes.by_index(0))
    other.close()

    simulation.start(save_results=False)
    with pytest.raises(swmmrs.LifecycleError):
        conduit.settings.length = 125.0
    simulation.end()
    conduit.settings.length = 125.0
    simulation.start(save_results=False)
    assert conduit.settings.length == 125.0
    simulation.end()
    simulation.close()
    with pytest.raises(swmmrs.StaleViewError):
        conduit.settings.update()


def test_link_declarations_survive_failed_start_repair_and_rerun(tmp_path: Path) -> None:
    simulation = open_simulation(CONDUIT_FIXTURE, tmp_path, "declaration-retry.rpt")
    conduit = simulation.links["1"]
    assert isinstance(conduit, Conduit)
    conduit.settings.update(
        inlet_offset=1.0,
        outlet_offset=1.5,
        initial_flow=-0.25,
        inlet_loss_coefficient=0.1,
        outlet_loss_coefficient=0.2,
        average_loss_coefficient=0.3,
        seepage_rate=0.025,
        has_flap_gate=True,
    )
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.inlet_offset = -1.0
    assert conduit.settings.inlet_offset == pytest.approx(1.0)

    conduit.settings.outlet_node = simulation.nodes["9"]
    assert conduit.settings.outlet_node == simulation.nodes["9"]
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start(save_results=False)
    assert conduit.settings.inlet_offset == pytest.approx(1.0)
    assert conduit.settings.outlet_offset == pytest.approx(1.5)
    assert conduit.settings.initial_flow == pytest.approx(-0.25)
    assert conduit.settings.has_flap_gate
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start(save_results=False)

    conduit.settings.outlet_node = simulation.nodes["10"]
    simulation.start(save_results=False)
    simulation.end()
    assert conduit.settings.outlet_node == simulation.nodes["10"]
    simulation.start(save_results=False)
    simulation.end()
    simulation.close()


def test_all_link_declarations_survive_failed_start_repair_and_rerun(tmp_path: Path) -> None:
    conduit_sim = open_simulation(CONDUIT_FIXTURE, tmp_path, "all-conduit.rpt")
    conduit = conduit_sim.links["1"]
    assert isinstance(conduit, Conduit)
    conduit.settings.update(
        length=125.0,
        roughness=0.014,
        barrels=2,
        cross_section=CircularCrossSection(2.5),
        inlet_offset=1.0,
        outlet_offset=1.5,
        initial_flow=-0.25,
    )
    conduit.settings.outlet_node = conduit_sim.nodes["9"]
    expected_conduit = (
        conduit.settings.length,
        conduit.settings.roughness,
        conduit.settings.barrels,
        conduit.settings.cross_section,
        conduit.settings.inlet_offset,
        conduit.settings.outlet_offset,
        conduit.settings.initial_flow,
    )
    with pytest.raises(swmmrs.ConfigurationError):
        conduit_sim.start(save_results=False)
    with pytest.raises(swmmrs.ConfigurationError):
        conduit_sim.start(save_results=False)
    assert (
        conduit.settings.length,
        conduit.settings.roughness,
        conduit.settings.barrels,
        conduit.settings.cross_section,
        conduit.settings.inlet_offset,
        conduit.settings.outlet_offset,
        conduit.settings.initial_flow,
    ) == expected_conduit
    conduit.settings.outlet_node = conduit_sim.nodes["10"]
    conduit_sim.start(save_results=False)
    conduit_sim.end()
    assert (
        conduit.settings.length,
        conduit.settings.roughness,
        conduit.settings.barrels,
        conduit.settings.cross_section,
        conduit.settings.inlet_offset,
        conduit.settings.outlet_offset,
        conduit.settings.initial_flow,
    ) == expected_conduit
    conduit_sim.start(save_results=False)
    conduit_sim.end()
    conduit_sim.close()

    simulation = open_simulation(REGULATOR_FIXTURE, tmp_path, "all-regulators.rpt")
    pump = simulation.links["Pump-A"]
    orifice = simulation.links["Orifice-A"]
    weir = simulation.links["Weir-A"]
    outlet = simulation.links["Outlet-A"]
    assert isinstance(pump, Pump)
    assert isinstance(orifice, Orifice)
    assert isinstance(weir, Weir)
    assert isinstance(outlet, Outlet)

    pump.settings.update(initial_setting=0.75, startup_depth=2.0, shutoff_depth=1.0)
    pump.settings.use_curve(simulation.curves.by_index(1))
    orifice.settings.update(
        kind=OrificeKind.BOTTOM,
        discharge_coefficient=0.65,
        opening_time_hours=0.5,
        inlet_offset=1.0,
        outlet_offset=1.0,
    )
    weir.settings.update(
        kind=WeirKind.ROADWAY,
        discharge_coefficient=3.0,
        end_discharge_coefficient=1.0,
        end_contractions=2.0,
        can_surcharge=False,
        roadway_width=12.0,
        roadway_surface=RoadSurface.PAVED,
        coefficient_curve=None,
    )
    outlet.settings.rating = TabularOutletRating(simulation.curves.by_index(0))
    assert orifice.settings.inlet_offset == 1.0
    assert orifice.settings.outlet_offset == 1.0
    before = (
        pump.settings.curve,
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
        orifice.settings.kind,
        orifice.settings.discharge_coefficient,
        orifice.settings.opening_time_hours,
        orifice.settings.inlet_offset,
        orifice.settings.outlet_offset,
        weir.settings.kind,
        weir.settings.discharge_coefficient,
        weir.settings.end_discharge_coefficient,
        weir.settings.end_contractions,
        weir.settings.can_surcharge,
        weir.settings.roadway_width,
        weir.settings.roadway_surface,
        outlet.settings.rating,
    )
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start(save_results=False)
    with pytest.raises(swmmrs.ConfigurationError):
        simulation.start(save_results=False)
    assert (
        pump.settings.curve,
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
        orifice.settings.kind,
        orifice.settings.discharge_coefficient,
        orifice.settings.opening_time_hours,
        orifice.settings.inlet_offset,
        orifice.settings.outlet_offset,
        weir.settings.kind,
        weir.settings.discharge_coefficient,
        weir.settings.end_discharge_coefficient,
        weir.settings.end_contractions,
        weir.settings.can_surcharge,
        weir.settings.roadway_width,
        weir.settings.roadway_surface,
        outlet.settings.rating,
    ) == before
    assert orifice.settings.inlet_offset == 1.0
    assert orifice.settings.outlet_offset == 1.0

    pump.settings.use_curve(simulation.curves.by_index(0))
    outlet.settings.rating = TabularOutletRating(simulation.curves.by_index(2))
    simulation.start(save_results=False)
    simulation.end()
    assert orifice.settings.inlet_offset == 1.0
    assert orifice.settings.outlet_offset == 1.0
    after_first = (
        pump.settings.curve,
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
        orifice.settings.kind,
        orifice.settings.discharge_coefficient,
        orifice.settings.opening_time_hours,
        orifice.settings.inlet_offset,
        orifice.settings.outlet_offset,
        weir.settings.kind,
        weir.settings.discharge_coefficient,
        weir.settings.end_discharge_coefficient,
        weir.settings.end_contractions,
        weir.settings.can_surcharge,
        weir.settings.roadway_width,
        weir.settings.roadway_surface,
        outlet.settings.rating,
    )
    simulation.start(save_results=False)
    simulation.end()
    assert (
        pump.settings.curve,
        pump.settings.initial_setting,
        pump.settings.startup_depth,
        pump.settings.shutoff_depth,
        orifice.settings.kind,
        orifice.settings.discharge_coefficient,
        orifice.settings.opening_time_hours,
        orifice.settings.inlet_offset,
        orifice.settings.outlet_offset,
        weir.settings.kind,
        weir.settings.discharge_coefficient,
        weir.settings.end_discharge_coefficient,
        weir.settings.end_contractions,
        weir.settings.can_surcharge,
        weir.settings.roadway_width,
        weir.settings.roadway_surface,
        outlet.settings.rating,
    ) == after_first
    assert orifice.settings.inlet_offset == 1.0
    assert orifice.settings.outlet_offset == 1.0
    simulation.close()


def test_settings_topology_and_relationship_failures_are_atomic(tmp_path: Path) -> None:
    simulation = open_simulation(COLLECTIONS_FIXTURE, tmp_path)
    conduit = simulation.links["J-Pipe"]
    assert isinstance(conduit, Conduit)

    conduit.settings.inlet_node = simulation.nodes["123"]
    assert conduit.settings.inlet_node == simulation.nodes["123"]
    before = conduit.settings.cross_section

    other = open_simulation(COLLECTIONS_FIXTURE, tmp_path, "foreign.rpt")
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.cross_section = CustomCrossSection(other.shapes["Shape-A"], 3.0)
    assert conduit.settings.cross_section == before
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.cross_section = CustomCrossSection(
            cast(CustomShape, simulation.curves["Shape-A"]), 3.0
        )
    assert conduit.settings.cross_section == before
    other.close()
    simulation.close()


def test_partial_regulator_updates_preserve_omitted_siblings(tmp_path: Path) -> None:
    simulation = open_simulation(REGULATOR_FIXTURE, tmp_path)
    orifice = simulation.links["Orifice-A"]
    weir = simulation.links["Weir-A"]
    assert isinstance(orifice, Orifice)
    assert isinstance(weir, Weir)

    orifice_before = (orifice.settings.kind, orifice.settings.opening_time_hours)
    orifice.settings.discharge_coefficient = 0.71
    assert (orifice.settings.kind, orifice.settings.opening_time_hours) == orifice_before
    assert orifice.settings.discharge_coefficient == pytest.approx(0.71)

    weir_before = (
        weir.settings.kind,
        weir.settings.end_discharge_coefficient,
        weir.settings.roadway_surface,
    )
    weir.settings.discharge_coefficient = 3.25
    assert (
        weir.settings.kind,
        weir.settings.end_discharge_coefficient,
        weir.settings.roadway_surface,
    ) == weir_before
    assert weir.settings.discharge_coefficient == pytest.approx(3.25)
    simulation.close()


def test_native_update_rejects_declaration_and_enum_impostors_atomically(tmp_path: Path) -> None:
    simulation = open_simulation(COLLECTIONS_FIXTURE, tmp_path)
    conduit = simulation.links["J-Pipe"]
    orifice = simulation.links["Orifice-A"]
    outlet = simulation.links["Outlet-A"]
    assert isinstance(conduit, Conduit)
    assert isinstance(orifice, Orifice)
    assert isinstance(outlet, Outlet)

    fake_cross_section_type = type(
        "CircularCrossSection",
        (),
        {"__module__": "swmmrs.objects", "diameter": 4.0},
    )
    fake_rating_type = type(
        "FunctionalOutletRating",
        (),
        {
            "__module__": "swmmrs.objects",
            "coefficient": 2.0,
            "exponent": 1.5,
            "head_basis": OutletHeadBasis.DEPTH,
        },
    )
    fake_orifice_kind_type = type("OrificeKind", (str,), {"__module__": "swmmrs.enums"})
    before = (
        conduit.settings.cross_section,
        orifice.settings.kind,
        outlet.settings.rating,
        simulation.configuration_dirty,
    )
    with pytest.raises(swmmrs.ValidationError):
        conduit.settings.cross_section = fake_cross_section_type()  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        outlet.settings.rating = fake_rating_type()  # type: ignore[assignment]
    with pytest.raises(swmmrs.ValidationError):
        orifice.settings.kind = fake_orifice_kind_type("side")  # type: ignore[assignment]
    assert (
        conduit.settings.cross_section,
        orifice.settings.kind,
        outlet.settings.rating,
        simulation.configuration_dirty,
    ) == before
    simulation.close()


def test_relationship_reads_preserve_numeric_identity_and_stale_views(tmp_path: Path) -> None:
    model = tmp_path / "numeric-relationships.inp"
    # pi-lens-ignore: python-path-traversal
    model.write_text(
        REGULATOR_FIXTURE.read_text().replace("PumpCurve", "101").replace("RatingCurve", "303")
    )
    simulation = open_simulation(model, tmp_path, "numeric-relationships.rpt")
    pump = simulation.links["Pump-A"]
    outlet = simulation.links["Outlet-A"]
    assert isinstance(pump, Pump)
    assert isinstance(outlet, Outlet)

    assert pump.settings.curve is not pump.settings.curve
    pump_curve = pump.settings.curve
    assert pump_curve is not None
    assert pump_curve.id == "101"
    assert pump_curve == simulation.curves["101"]
    outlet.settings.rating = TabularOutletRating(simulation.curves["303"])
    rating = outlet.settings.rating
    assert isinstance(rating, TabularOutletRating)
    assert rating.curve is not outlet.settings.rating.curve
    assert rating.curve.id == "303"
    assert rating.curve == simulation.curves["303"]

    simulation.close()
    with pytest.raises(swmmrs.StaleViewError):
        _ = pump_curve.id
    with pytest.raises(swmmrs.StaleViewError):
        _ = rating.curve.id


def test_obsolete_link_configuration_names_are_not_exported() -> None:
    import swmmrs.objects as objects

    for name in (
        "LinkConfiguration",
        "ConduitConfiguration",
        "IdealPumpConfiguration",
        "CurvePumpConfiguration",
        "OrificeConfiguration",
        "WeirConfiguration",
    ):
        assert not hasattr(objects, name)


def test_runtime_controls_results_and_settings_remain_separate(tmp_path: Path) -> None:
    conduit_sim = open_simulation(CONDUIT_FIXTURE, tmp_path, "conduit.rpt")
    conduit = conduit_sim.links["1"]
    assert isinstance(conduit, Conduit)
    conduit.flow_limit = 1.75
    assert conduit.flow_limit == 1.75
    with pytest.raises(swmmrs.ValidationError):
        conduit.flow_limit = False
    assert conduit.flow_limit == 1.75
    assert conduit.settings.tag == ""
    # A stable dirty edit must not overwrite the persistent flow-limit owner during preparation.
    conduit.settings.inlet_offset = conduit.settings.inlet_offset
    with pytest.raises(swmmrs.LifecycleError):
        _ = conduit.flow

    conduit_sim.start(save_results=False)
    assert conduit.flow_limit == 1.75
    assert_common_results(conduit)
    assert math.isfinite(conduit.top_width)
    conduit.flow_limit = 2.25
    assert conduit.flow_limit == 2.25
    with pytest.raises(AttributeError):
        conduit.flow = 1.0  # type: ignore[misc]
    conduit_sim.end()
    assert math.isfinite(conduit.flow)
    assert math.isfinite(conduit.settings.inlet_offset)
    conduit_sim.close()

    pump_sim = open_simulation(PUMP_FIXTURE, tmp_path, "pump.rpt")
    pump = pump_sim.links["P1"]
    assert isinstance(pump, Pump)
    pump_sim.start(save_results=False)
    assert_common_results(pump)
    pump_sim.close()

    regulator_sim = open_simulation(ORIFICE_FIXTURE, tmp_path, "orifice.rpt")
    orifice = regulator_sim.links["Valve"]
    assert isinstance(orifice, Orifice)
    regulator_sim.start(save_results=False)
    assert_common_results(orifice)
    orifice.target_setting = 0.5
    assert orifice.target_setting == 0.5
    regulator_sim.close()
