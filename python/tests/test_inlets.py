from __future__ import annotations

import math
from copy import copy
from datetime import timedelta
from pathlib import Path

import pytest  # pyright: ignore[reportMissingImports]

import swmmrs
import swmmrs.objects._base as object_base
from swmmrs.objects import CircularCrossSection, Conduit, Inlet, InletDesign, StreetCrossSection

ROOT = Path(__file__).resolve().parents[2]
INLET_FIXTURE = (
    ROOT
    / "crates"
    / "run"
    / "tests"
    / "data"
    / "regression-suite"
    / "hydraulics"
    / "inlets-onsag-slotted-custom.inp"
)
SHARED_DESIGN_FIXTURE = (
    ROOT
    / "crates"
    / "run"
    / "tests"
    / "data"
    / "regression-suite"
    / "hydraulics"
    / "inlets-inlet_capture_test.inp"
)
ABSENT_FIXTURE = Path(__file__).resolve().parent / "data" / "collections.inp"


def test_link_exposes_optional_local_inlet_and_settings_identity(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(object_base, "_INLET_TOKEN", object())
    simulation = swmmrs.Simulation(INLET_FIXTURE, tmp_path / "present.rpt")
    link = simulation.links["C_SLOT"]
    inlet = link.inlet

    assert isinstance(inlet, Inlet)
    assert inlet is not None
    assert (
        repr(inlet)
        == "Inlet(link_id='C_SLOT', link_index=0, design_id='SlotInlet', design_index=0)"
    )
    assert inlet.settings.count == 2
    assert inlet.settings.local_depression == 0.1
    assert inlet.settings.local_width == 2.0
    assert isinstance(inlet.settings.design, InletDesign)
    assert inlet.settings.design == simulation.inlet_designs["SlotInlet"]
    assert not hasattr(inlet, "count")
    assert not hasattr(inlet, "design")
    assert inlet == link.inlet
    assert inlet is not link.inlet
    assert not hasattr(simulation, "inlets")
    simulation.close()

    absent = swmmrs.Simulation(ABSENT_FIXTURE, tmp_path / "absent.rpt")
    assert absent.links["J-Pipe"].inlet is None
    absent.close()


def test_persistent_update_is_atomic_and_survives_repeated_runs(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(INLET_FIXTURE, tmp_path / "persistent.rpt")
    inlet = simulation.links["C_SLOT"].inlet
    assert inlet is not None

    assert inlet.percent_clogged == pytest.approx(10.0, abs=1e-7, rel=0)
    assert inlet.flow_limit == 6.0
    inlet.update()
    assert not simulation.configuration_dirty

    before = (inlet.percent_clogged, inlet.flow_limit)
    with pytest.raises(swmmrs.ValidationError):
        inlet.update(percent_clogged=25.0, flow_limit=-1.0)
    assert (inlet.percent_clogged, inlet.flow_limit) == before
    with pytest.raises(swmmrs.ValidationError):
        inlet.update(percent_clogged=25.0, count=3)
    assert (inlet.percent_clogged, inlet.flow_limit) == before

    for name, invalid in (
        ("percent_clogged", -0.1),
        ("percent_clogged", 100.1),
        ("percent_clogged", float("nan")),
        ("percent_clogged", False),
        ("flow_limit", -1.0),
        ("flow_limit", float("inf")),
        ("flow_limit", False),
    ):
        before = (inlet.percent_clogged, inlet.flow_limit)
        with pytest.raises(swmmrs.ValidationError):
            setattr(inlet, name, invalid)
        assert (inlet.percent_clogged, inlet.flow_limit) == before

    inlet.update(percent_clogged=37.5, flow_limit=4.25)
    assert inlet.percent_clogged == 37.5
    assert inlet.flow_limit == 4.25
    simulation.start()
    inlet.update(percent_clogged=25.0, flow_limit=5.5)
    assert not simulation.configuration_dirty
    simulation.stride(timedelta(days=1))
    assert inlet.settings.count == 2
    inlet.update()
    with pytest.raises(swmmrs.LifecycleError):
        _ = inlet.percent_clogged
    with pytest.raises(swmmrs.LifecycleError):
        inlet.update(flow_limit=8.0)
    simulation.end()
    assert inlet.percent_clogged == 25.0
    assert inlet.flow_limit == 5.5

    inlet.update(flow_limit=7.25)
    assert simulation.state == swmmrs.SimulationState.OPEN
    simulation.start()
    simulation.stride(timedelta(days=1))
    simulation.end()
    assert inlet.percent_clogged == 25.0
    assert inlet.flow_limit == 7.25
    simulation.close()


def test_invalid_geometry_diagnostic_retains_inlet_and_persistent_controls(
    tmp_path: Path,
) -> None:
    simulation = swmmrs.Simulation(INLET_FIXTURE, tmp_path / "invalid-placement.rpt")
    conduit = simulation.links["C_SLOT"]
    assert isinstance(conduit, Conduit)
    inlet = conduit.inlet
    assert inlet is not None
    inlet.update(percent_clogged=25.0, flow_limit=4.25)

    conduit.settings.cross_section = CircularCrossSection(2.0)
    with pytest.raises(swmmrs.ConfigurationError) as caught:
        simulation.start()
    assert [diagnostic.rule_code for diagnostic in caught.value.diagnostics] == [
        "inlet.placement.compatibility"
    ]
    assert conduit.inlet == inlet
    assert inlet.settings.design.id == "SlotInlet"
    assert inlet.percent_clogged == 25.0
    assert inlet.flow_limit == 4.25

    conduit.settings.cross_section = StreetCrossSection(simulation.streets["Street1"])
    simulation.start()
    assert inlet.flow_factor == pytest.approx(2.3270139939788844e-06, abs=1e-18, rel=0)
    assert inlet.percent_clogged == 25.0
    assert inlet.flow_limit == 4.25
    simulation.stride(timedelta(days=1))
    simulation.end()
    assert inlet.percent_clogged == 25.0
    assert inlet.flow_limit == 4.25
    simulation.close()


def test_geometry_edit_rebuilds_flow_factor_like_parsed_reference(tmp_path: Path) -> None:
    reference_input = tmp_path / "reference.inp"
    source = INLET_FIXTURE.read_text()
    original = "C_SLOT   SLOT_UP    SLOT_SAG  100     0.016"
    replacement = "C_SLOT   SLOT_UP    SLOT_SAG  200     0.016"
    assert source.count(original) == 1
    # pi-lens-ignore: python-path-traversal
    reference_input.write_text(source.replace(original, replacement, 1))

    edited = swmmrs.Simulation(INLET_FIXTURE, tmp_path / "edited.rpt")
    reference = swmmrs.Simulation(reference_input, tmp_path / "reference.rpt")
    edited_conduit = edited.links["C_SLOT"]
    assert isinstance(edited_conduit, Conduit)
    edited_inlet = edited_conduit.inlet
    reference_inlet = reference.links["C_SLOT"].inlet
    assert edited_inlet is not None and reference_inlet is not None
    edited_conduit.settings.length = 200.0

    edited.start()
    reference.start()
    assert edited_inlet.flow_factor == reference_inlet.flow_factor
    assert edited_inlet.flow_factor == pytest.approx(
        1.6454165212810583e-06, abs=1e-18, rel=0
    )
    edited.stride(timedelta(minutes=45))
    reference.stride(timedelta(minutes=45))
    assert edited_inlet.captured_flow == reference_inlet.captured_flow
    assert edited_inlet.backflow_ratio == reference_inlet.backflow_ratio
    edited.stride(timedelta(days=1))
    reference.stride(timedelta(days=1))
    edited.end()
    reference.end()
    edited.close()
    reference.close()


def test_capture_results_have_exact_windows_and_stale_validation(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(INLET_FIXTURE, tmp_path / "results.rpt")
    inlet = simulation.links["C_SLOT"].inlet
    assert inlet is not None

    for name in ("flow_factor", "captured_flow", "backflow", "backflow_ratio"):
        with pytest.raises(swmmrs.LifecycleError):
            getattr(inlet, name)

    simulation.start()
    simulation.stride(timedelta(minutes=45))
    assert inlet.flow_factor > 0.0
    assert inlet.captured_flow > 0.0
    assert math.isfinite(inlet.backflow)
    assert math.isfinite(inlet.backflow_ratio)
    simulation.stride(timedelta(days=1))
    complete_results = (
        inlet.flow_factor,
        inlet.captured_flow,
        inlet.backflow,
        inlet.backflow_ratio,
    )
    simulation.end()
    assert (
        inlet.flow_factor,
        inlet.captured_flow,
        inlet.backflow,
        inlet.backflow_ratio,
    ) == complete_results

    for name in ("flow_factor", "captured_flow", "backflow", "backflow_ratio"):
        with pytest.raises(AttributeError):
            setattr(inlet, name, 1)
    for name in ("count", "local_depression", "local_width", "design"):
        with pytest.raises(AttributeError):
            setattr(inlet.settings, name, 1)
    with pytest.raises(TypeError):
        copy(inlet)

    simulation.close()
    for operation in (
        lambda: inlet.settings.count,
        lambda: inlet.settings.design,
        lambda: inlet.captured_flow,
        lambda: inlet.update(),
    ):
        with pytest.raises(swmmrs.StaleViewError):
            operation()


def test_inlet_settings_and_controls_round_trip_metric_units(tmp_path: Path) -> None:
    metric_fixture = tmp_path / "metric.inp"
    # pi-lens-ignore: python-path-traversal
    metric_fixture.write_text(
        INLET_FIXTURE.read_text().replace("FLOW_UNITS           CFS", "FLOW_UNITS           CMS")
    )
    simulation = swmmrs.Simulation(metric_fixture, tmp_path / "metric.rpt")
    inlet = simulation.links["C_SLOT"].inlet
    assert inlet is not None
    assert inlet.settings.local_depression == pytest.approx(0.1, abs=1e-7, rel=0)
    assert inlet.settings.local_width == pytest.approx(2.0, abs=1e-7, rel=0)
    assert inlet.flow_limit == pytest.approx(6.0, abs=1e-7, rel=0)
    inlet.update(flow_limit=1.25)
    assert inlet.flow_limit == pytest.approx(1.25, abs=1e-7, rel=0)
    simulation.close()


def test_identity_includes_parent_link_for_shared_design(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(SHARED_DESIGN_FIXTURE, tmp_path / "identity.rpt")
    first = simulation.links["C1"].inlet
    second = simulation.links["C2"].inlet
    assert first is not None and second is not None
    assert first.settings.design == second.settings.design
    assert first != second
    assert hash(first) != hash(second)
    simulation.close()
