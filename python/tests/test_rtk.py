from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs
from swmmrs.objects import (
    RdiiAssignment,
    UnitHydrograph,
    UnitHydrographMonth,
    UnitHydrographResponse,
)

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "crates"
    / "run"
    / "tests"
    / "data"
    / "regression-suite"
    / "save_interfaces"
    / "save_rdii.inp"
)


def test_rtk_models_and_assignments_are_editable(tmp_path: Path) -> None:
    input_path = tmp_path / "rtk.inp"
    input_path.write_text(FIXTURE.read_text().replace('SAVE RDII "rdii-interface.bin"', ""))

    with swmmrs.Simulation(input_path, tmp_path / "rtk.rpt") as simulation:
        unit_hydrograph = simulation.unit_hydrographs["UH1"]
        assert type(unit_hydrograph) is UnitHydrograph
        assert unit_hydrograph.rain_gage == simulation.rain_gages["GAGE"]

        monthly_responses = unit_hydrograph.monthly_responses
        assert len(monthly_responses) == 12
        january = monthly_responses[0]
        assert type(january) is UnitHydrographMonth
        assert type(january.short) is UnitHydrographResponse
        assert type(january.medium) is UnitHydrographResponse
        assert type(january.long) is UnitHydrographResponse
        january_short = january.short
        assert january_short.rainfall_fraction == pytest.approx(0.20)
        assert january_short.time_to_peak_hours == pytest.approx(0.5)
        assert january_short.recession_ratio == pytest.approx(2.0)

        responses = monthly_responses
        responses[0].short.rainfall_fraction = 0.10
        responses[0].short.time_to_peak_hours = 0.75
        responses[0].short.recession_ratio = 3.0
        responses[0].short.maximum_initial_abstraction = 0.25
        responses[0].short.initial_abstraction_recovery_rate = 0.05
        responses[0].short.initial_abstraction_at_start = 0.10
        unit_hydrograph.update(
            rain_gage=simulation.rain_gages["GAGE"],
            monthly_responses=responses,
        )

        updated = unit_hydrograph.monthly_responses[0].short
        assert updated.rainfall_fraction == pytest.approx(0.10)
        assert updated.time_to_peak_hours == pytest.approx(0.75)
        assert updated.recession_ratio == pytest.approx(3.0)
        assert updated.maximum_initial_abstraction == pytest.approx(0.25)
        assert updated.initial_abstraction_recovery_rate == pytest.approx(0.05)
        assert updated.initial_abstraction_at_start == pytest.approx(0.10)

        invalid = unit_hydrograph.monthly_responses
        invalid[0].short.rainfall_fraction = 1.02
        with pytest.raises(swmmrs.ValidationError):
            unit_hydrograph.monthly_responses = invalid
        assert unit_hydrograph.monthly_responses[0].short.rainfall_fraction == pytest.approx(0.10)

        assignment = simulation.rdii_assignments[0]
        assert type(assignment) is RdiiAssignment
        assert assignment.node == simulation.nodes["J1"]
        assert assignment.unit_hydrograph == unit_hydrograph
        assert assignment.area == pytest.approx(1.0)
        assignment.area = 2.0
        simulation.rdii_assignments = [assignment]
        assert simulation.rdii_assignments[0].area == pytest.approx(2.0)

        with pytest.raises(swmmrs.ValidationError):
            simulation.rdii_assignments = [RdiiAssignment(assignment.node, unit_hydrograph, -1.0)]
        assert simulation.rdii_assignments[0].area == pytest.approx(2.0)

        simulation.start(save_results=False)
        with pytest.raises(swmmrs.LifecycleError):
            unit_hydrograph.rain_gage = simulation.rain_gages["GAGE"]
        while simulation.step() is not None:
            pass
        simulation.end()
