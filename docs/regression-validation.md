# Regression validation

Numerical solvers are remarkably willing to agree with themselves. That is why
`swmmrs` runs a numerical regression suite against EPA SWMM instead. The suite
provides broad, repeatable parity evidence across representative solver
behavior. It does not provide cosmic certainty, which remains difficult to fit
into CI.

[View the latest HTML regression report](regression-report.html){ .md-button .md-button--primary }

## How the suite works

[`swmm-bench`](https://github.com/swmm-rs/swmm-bench) handles the repetitive and
necessary work. Its `swmm-test` runner sends the same model suite through
compatible SWMM engines and compares:

- parsed tables from SWMM text reports.
- report-period time series from SWMM binary output files.
- rainfall, runoff, hotstart, RDII, and routing interface-file production and
  consumption.

The suite covers hydrology, hydraulics, controls, water quality, interface
workflows, and all three routing methods: steady, kinematic, and dynamic wave.
The HTML report is the flight recorder. It keeps the current results, including
the awkward models that a morale officer might prefer to omit.

## EPA SWMM code exercised

The model suite exercises more than 87% of EPA SWMM solver lines and 67% of
branches. The published [EPA SWMM coverage
report](https://swmm-rs.github.io/swmm-bench/epa-swmm-coverage.html) shows the
details.

Those numbers measure the upstream C solver while the suite runs. They do not
measure `swmmrs` source coverage, and they certainly do not mean that 87% of all
possible hydraulic universes have been proven equivalent. Coverage tells us
where the probes went. It cannot tell us what exists everywhere else.

## How to interpret a pass

A passing report means that `swmmrs` agrees closely with EPA SWMM across a broad,
deliberately varied suite. That is strong evidence. It is not a certificate
from the Galactic Hydraulics Council covering every input, platform, routing
condition, numerical edge case, and future release.

Dynamic Wave is still the largest structural departure from the upstream C
code. It remains the part of the engine room with the most instruments pointed
at it.

`swmmrs` remains pre-release software. Compare critical production projects
with a trusted EPA SWMM release. If the results differ without explanation,
report the model, both engine versions, and the generated artifacts.

## Why `slot-surcharge_tunnelmh.inp` differs from EPA 5.2.4

The filename points accusingly at slot surcharge. The evidence points to an
outfall bug instead.

The latest report gives this model a 0.028021 report distance and 0.023333
output distance. Its terminal side orifice, `OverflowGate`, has a 13 ft offset.
SWMM applies that offset at both link ends, including the `Overflow` FREE
outfall. EPA 5.2.4 has a [known outfall
bug](https://github.com/USEPA/Stormwater-Management-Model/issues/154) that forces
a FREE or NORMAL outfall to zero depth when its connected link has a nonzero
offset. EPA's [5.3
fix](https://github.com/USEPA/Stormwater-Management-Model/commit/4606020) and
`swmmrs` 0.1.0 include the offset.

The bug sends the engines into two small but measurable timelines. EPA 5.2.4
holds `Overflow` at 0 ft depth and -50 ft hydraulic head. `swmmrs` holds it at
13 ft and -37 ft for all 1,211 reporting periods. Those are the only two nonzero
differences among the 54 binary-output series. The remaining report differences
are flow-summary changes no larger than 0.7 cfs.

The displayed distance comes from one documented version difference, not a
broader slot-surcharge mismatch.

## Relationship to real-world benchmarks

The regression suite sends many compact probes through individual solver
features and useful combinations of them. The [real-world
benchmarks](benchmark-results.md) ask a different question. They use fewer,
larger hydraulic models to compare runtime and result similarity under workloads
closer to day-to-day modelling practice.
