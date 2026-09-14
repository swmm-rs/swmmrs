from __future__ import annotations

import math
from pathlib import Path
from typing import NamedTuple

import pytest

import swmmrs

EPA_SPAN_RISE = 1.56363
RISE_METERS = 2.0
SPAN_METERS = RISE_METERS * EPA_SPAN_RISE
BARRELS = 2
EPA_HORIZONTAL_AREA = (
    0.000,
    0.015,
    0.040,
    0.065,
    0.095,
    0.130,
    0.165,
    0.205,
    0.250,
    0.300,
    0.355,
    0.415,
    0.480,
    0.520,
    0.585,
    0.645,
    0.700,
    0.750,
    0.795,
    0.835,
    0.870,
    0.905,
    0.935,
    0.960,
    0.985,
    1.000,
)


class LinkResult(NamedTuple):
    flow_lps: float
    depth_meters: float
    velocity_meters_per_second: float
    capacity: float
    top_width_meters: float


def model(flow_lps: int) -> str:
    return f"""[OPTIONS]
FLOW_UNITS LPS
FLOW_ROUTING DYNWAVE
START_DATE 01/01/2020
START_TIME 00:00:00
REPORT_START_DATE 01/01/2020
REPORT_START_TIME 00:00:00
END_DATE 01/01/2020
END_TIME 06:00:00
REPORT_STEP 00:15:00
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
TEST HORIZ_ELLIPSE {RISE_METERS} {SPAN_METERS} 0 0 {BARRELS}
DOWNSTREAM RECT_CLOSED 3 3 0 0 1

[INFLOWS]
UP FLOW "" FLOW 1 1 {flow_lps}
"""


def run_case(
    directory: Path,
    flow_lps: int,
    ellipse_model: swmmrs.CustomEllipseModel,
) -> LinkResult:
    directory.mkdir()
    input_path = directory / "model.inp"
    input_path.write_text(model(flow_lps))
    simulation = swmmrs.Simulation(input_path, directory / "model.rpt")
    simulation.options.custom_ellipse_model = ellipse_model
    simulation.start(save_results=False)
    while simulation.stride(900) is not None:
        pass
    link = simulation.links["TEST"]
    result = LinkResult(
        link.flow,
        link.depth,
        link.velocity,
        link.capacity,
        link.top_width,
    )
    simulation.end()
    simulation.close()
    return result


def area_from_results(result: LinkResult) -> float:
    """Return one barrel's inferred flow area in square meters."""
    return result.flow_lps / 1000.0 / result.velocity_meters_per_second / BARRELS


def ellipse_area(depth: float) -> float:
    """Return one mathematical ellipse's area below a given depth."""
    depth_fraction = depth / RISE_METERS
    phi = math.acos(1.0 - 2.0 * depth_fraction)
    return RISE_METERS * SPAN_METERS / 4.0 * (phi - math.sin(phi) * math.cos(phi))


def ellipse_width(depth: float) -> float:
    """Return one mathematical ellipse's width at a given depth."""
    depth_fraction = depth / RISE_METERS
    return SPAN_METERS * math.sqrt(1.0 - (1.0 - 2.0 * depth_fraction) ** 2)


def lookup(values: tuple[float, ...], fraction: float) -> float:
    """Linearly interpolate an EPA table over zero through one."""
    position = fraction * (len(values) - 1)
    lower = min(int(position), len(values) - 2)
    weight = position - lower
    return values[lower] + weight * (values[lower + 1] - values[lower])


def test_public_partial_depth_geometry_matches_each_selected_model(tmp_path: Path) -> None:
    legacy = run_case(
        tmp_path / "legacy-partial",
        10_000,
        swmmrs.CustomEllipseModel.EPA_LEGACY,
    )
    true = run_case(
        tmp_path / "true-partial",
        10_000,
        swmmrs.CustomEllipseModel.TRUE_ELLIPSE,
    )

    assert legacy.capacity < 1.0
    assert true.capacity < 1.0
    assert area_from_results(true) == pytest.approx(ellipse_area(true.depth_meters), rel=1e-3)
    assert true.top_width_meters == pytest.approx(ellipse_width(true.depth_meters), rel=1e-3)

    legacy_area = (
        1.2692
        * RISE_METERS**2
        * lookup(
            EPA_HORIZONTAL_AREA,
            legacy.depth_meters / RISE_METERS,
        )
    )
    assert area_from_results(legacy) == pytest.approx(legacy_area, rel=1e-3)
    assert area_from_results(legacy) > area_from_results(true)


def test_public_full_depth_geometry_uses_selected_full_area(tmp_path: Path) -> None:
    legacy = run_case(
        tmp_path / "legacy-full",
        15_000,
        swmmrs.CustomEllipseModel.EPA_LEGACY,
    )
    true = run_case(
        tmp_path / "true-full",
        15_000,
        swmmrs.CustomEllipseModel.TRUE_ELLIPSE,
    )

    assert legacy.capacity == pytest.approx(1.0)
    assert true.capacity == pytest.approx(1.0)
    assert area_from_results(legacy) == pytest.approx(1.2692 * RISE_METERS**2, rel=1e-5)
    assert area_from_results(true) == pytest.approx(
        math.pi / 4.0 * RISE_METERS * SPAN_METERS,
        rel=1e-5,
    )
    assert legacy.velocity_meters_per_second < true.velocity_meters_per_second
