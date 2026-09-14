#!/usr/bin/env -S uv run --project python --with matplotlib==3.11.1 python
"""Compare a circular conduit with an equal-rise-and-span custom ellipse."""

from __future__ import annotations

from datetime import timedelta
from pathlib import Path
from tempfile import TemporaryDirectory

import matplotlib.pyplot as plt

import swmmrs

OUTPUT = Path(__file__).parent / "_static/img/true-ellipse-circle-routing.svg"
CIRCULAR = "CIRCULAR 1 0 0 0 1"
ELLIPSE = "HORIZ_ELLIPSE 1 1 0 0 1"


def model(xsection: str) -> str:
    """Return one Dynamic Wave model with a replaceable test cross section."""
    return f"""[OPTIONS]
FLOW_UNITS LPS
FLOW_ROUTING DYNWAVE
START_DATE 01/01/2020
START_TIME 00:00:00
REPORT_START_DATE 01/01/2020
REPORT_START_TIME 00:00:00
END_DATE 01/01/2020
END_TIME 06:00:00
REPORT_STEP 00:05:00
ROUTING_STEP 00:00:20
VARIABLE_STEP 0.75
MINIMUM_STEP 0.5
THREADS 1

[JUNCTIONS]
UP 9.3 5 0 0 0
IN 9.15 5 0 0 0
OUT 9.1 5 0 0 0

[OUTFALLS]
DOWN 8.95 NORMAL NO

[CONDUITS]
UPSTREAM UP IN 50 0.016 0 0 0 0
TEST IN OUT 45 0.024 0 0 0 0
DOWNSTREAM OUT DOWN 85 0.016 0 0 0 0

[XSECTIONS]
UPSTREAM RECT_CLOSED 3 3 0 0 1
TEST {xsection}
DOWNSTREAM RECT_CLOSED 3 3 0 0 1

[INFLOWS]
UP FLOW Inflow FLOW 1 1 0

[TIMESERIES]
Inflow 00:00 0
Inflow 00:30 100
Inflow 01:30 1600
Inflow 02:30 1600
Inflow 03:30 100
Inflow 04:00 0
Inflow 06:00 0
"""


def run(
    directory: Path,
    xsection: str,
    ellipse_model: swmmrs.CustomEllipseModel,
) -> tuple[list[float], list[float]]:
    """Run one model and return elapsed hours and test-conduit depth."""
    directory.mkdir()
    input_path = directory / "model.inp"
    input_path.write_text(model(xsection), encoding="utf-8")

    hours = [0.0]
    depths = [0.0]
    with swmmrs.Simulation(input_path, directory / "model.rpt") as simulation:
        simulation.options.custom_ellipse_model = ellipse_model
        simulation.start(save_results=False)
        conduit = simulation.links["TEST"]
        while (current_time := simulation.stride(timedelta(minutes=5))) is not None:
            hours.append((current_time - simulation.start_time).total_seconds() / 3600)
            depths.append(conduit.depth)
        simulation.end()
    return hours, depths


def plot(
    hours: list[float],
    circular: list[float],
    true_ellipse: list[float],
    legacy_ellipse: list[float],
) -> None:
    """Plot circular depth against each custom-ellipse model."""
    plt.rcParams["svg.fonttype"] = "none"
    plt.rcParams["svg.hashsalt"] = "swmmrs-circle-ellipse"
    figure, axes = plt.subplots(1, 2, figsize=(12, 4.8), sharex=True, sharey=True)
    maximum_depth = max(circular + true_ellipse + legacy_ellipse)
    comparisons = (
        (axes[0], true_ellipse, "TRUE_ELLIPSE", "#2563eb"),
        (axes[1], legacy_ellipse, "EPA_LEGACY", "#c2410c"),
    )
    for axis, ellipse_depth, label, color in comparisons:
        difference = max(abs(circle - ellipse) for circle, ellipse in zip(circular, ellipse_depth))
        axis.plot(hours, circular, color="#64748b", linewidth=4, label="CIRCULAR")
        axis.plot(hours, ellipse_depth, color=color, linestyle="--", linewidth=2, label=label)
        axis.set_title(f"{label} vs CIRCULAR")
        axis.set_xlabel("Elapsed time (hours)")
        axis.set_ylim(0, maximum_depth * 1.15)
        axis.grid(color="#cbd5e1", alpha=0.7)
        axis.legend()
        axis.text(
            0.03,
            0.94,
            f"maximum |Δ depth| = {difference:.4f} m",
            transform=axis.transAxes,
            bbox={"facecolor": "white", "edgecolor": "none", "alpha": 0.8},
        )
    axes[0].set_ylabel("TEST conduit depth (m)")
    figure.suptitle("1 m circular conduit versus 1 m × 1 m custom ellipse")
    figure.tight_layout(rect=(0, 0, 1, 0.93))
    figure.savefig(OUTPUT, format="svg", metadata={"Date": None})
    plt.close(figure)


def main() -> None:
    """Run all three cases, check the expected behavior, and write the plot."""
    with TemporaryDirectory() as temporary:
        root = Path(temporary)
        hours, circular = run(
            root / "circular",
            CIRCULAR,
            swmmrs.CustomEllipseModel.EPA_LEGACY,
        )
        true_hours, true_ellipse = run(
            root / "true-ellipse",
            ELLIPSE,
            swmmrs.CustomEllipseModel.TRUE_ELLIPSE,
        )
        legacy_hours, legacy_ellipse = run(
            root / "legacy-ellipse",
            ELLIPSE,
            swmmrs.CustomEllipseModel.EPA_LEGACY,
        )

    assert hours == true_hours == legacy_hours
    true_difference = max(abs(circle - ellipse) for circle, ellipse in zip(circular, true_ellipse))
    legacy_difference = max(
        abs(circle - ellipse) for circle, ellipse in zip(circular, legacy_ellipse)
    )
    assert true_difference < 0.001
    assert legacy_difference > 0.05

    plot(hours, circular, true_ellipse, legacy_ellipse)
    print(f"TRUE_ELLIPSE maximum depth difference: {true_difference:.6f} m")
    print(f"EPA_LEGACY maximum depth difference: {legacy_difference:.6f} m")


if __name__ == "__main__":
    main()
