from dataclasses import astuple
from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs
from swmmrs.objects import AmmAssignment, AmmBaseflowComponent, AmmModel, AmmStandardComponent

FIXTURE = (
    Path(__file__).resolve().parents[2] / "crates" / "solver" / "tests" / "data" / "amm-us.inp"
)


def test_amm_models_are_editable_live_views(tmp_path: Path) -> None:
    simulation = swmmrs.Simulation(FIXTURE, tmp_path / "amm.rpt")
    model = simulation.amm_models["m1"]

    assert type(model) is AmmModel
    assert model.rain_gage == simulation.rain_gages["GAGE1"]
    assert model.cold_temperature == 30.0
    assert model.hot_temperature == 70.0

    assignment = simulation.amm_assignments[0]
    assert type(assignment) is AmmAssignment
    assert assignment.node == simulation.nodes["J1"]
    assert assignment.model == model
    assert assignment.area == pytest.approx(1.0)
    assignment.area = 2.0
    simulation.amm_assignments = [assignment]
    assert simulation.amm_assignments[0].area == pytest.approx(2.0)
    with pytest.raises(swmmrs.ValidationError):
        simulation.amm_assignments = [AmmAssignment(assignment.node, model, -1.0)]
    assert simulation.amm_assignments[0].area == pytest.approx(2.0)
    components = model.components
    standard = components[0]
    assert isinstance(standard, AmmStandardComponent)
    assert standard.id == "C1"
    assert astuple(standard)[1:] == pytest.approx((0.1, 0.0, 1.0, 0.0, 24.0, 0.0, 0.1, 0.05, 0.1))
    baseflow = components[1]
    assert isinstance(baseflow, AmmBaseflowComponent)
    assert baseflow.id == "BASE"
    assert astuple(baseflow)[1:] == pytest.approx((0.0, 2.0, 0.0, 0.04, 0.02, 0.04))
    standard.hot_shcf = 0.06
    model.update(cold_temperature=25.0, hot_temperature=75.0, components=components)
    assert model.cold_temperature == 25.0
    assert model.hot_temperature == 75.0
    updated = model.components[0]
    assert isinstance(updated, AmmStandardComponent)
    assert updated.hot_shcf == pytest.approx(0.06)

    with pytest.raises(swmmrs.ValidationError):
        model.cold_temperature = 80.0
    assert model.cold_temperature == 25.0

    simulation.start(save_results=False)
    while simulation.step() is not None:
        pass
    simulation.end()
    simulation.close()
