# Benchmarks from the engine room

Benchmarks are where fast software goes to acquire impressive adjectives. This
page makes the numbers do most of the talking instead.

The `swmmrs` benchmark complements the feature-oriented regression suite with
large hydraulic models drawn from the sort of work a modeller encounters day
to day.

[View the latest HTML benchmark report](benchmark-report.html){ .md-button .md-button--primary }

## What the benchmark measures

[`swmm-bench`](https://github.com/swmm-rs/swmm-bench) sends the same input models
through each SWMM-compatible engine. It records runtime, peak memory, reports,
and binary output. In the latest report, every engine and model pair gets three
interleaved runs. We report the median runtime so one unusually fast or slow
run does not dominate the result.

Two distance calculations measure result similarity:

- a cell-weighted relative-distance metric over parsed SWMM report tables.
- a composite metric over matching report-period time series in binary `.out`
  files, weighted 75% toward typical values and 25% toward event behavior.

Lower distance means a closer result. The report keeps speed and similarity
together for a reason. Arriving first is not useful if the solver lands in the
wrong hydraulic system.

## Latest result

The August 26, 2026 report puts EPA SWMM 5.2.4 (`runswmm-07c371f`) and
`swmmrs` 0.1.0 (`runswmmrs-56ce94c`) through twelve stress models:

| Model | EPA median | `swmmrs` median | Runtime reduction | Report distance | Output distance |
| --- | ---: | ---: | ---: | ---: | ---: |
| `10033-hydraulic.inp` | 121 s | 58 s | 52.1% | 0.015429 | 0.001891 |
| `10860-nodes.inp` | 76 s | 41 s | 46.1% | 0.001345 | 0.000763 |
| `126000-groundwater-lid.inp` | 121 s | 117 s | 3.3% | 0.007016 | 0.000987 |
| `17100-dummy-links.inp` | 150 s | 76 s | 49.3% | 0.008367 | 0.000775 |
| `4569-nodes.inp` | 12 s | 9 s | 25.0% | 0.004949 | 0.000000 |
| `46-pumps.inp` | 424 s | 243 s | 42.7% | 0.022239 | 0.028547 |
| `continuousmodelwq.inp` | 31 s | 31 s | 0.0% | 0.011193 | 0.001562 |
| `ddc-24hr-100yr.inp` | 11 s | 10 s | 9.1% | 0.009763 | 0.000991 |
| `fredericksburg.inp` | 151 s | 119 s | 21.2% | 0.016904 | 0.000666 |
| `greenville-snowmelt.inp` | 34 s | 23 s | 32.4% | 0.073754 | 0.034562 |
| `terreno.inp` | 180 s | 66 s | 63.3% | 0.002057 | 0.000386 |
| `xpswmm-collapsingweir.inp` | 11 s | 10 s | 9.1% | 0.003988 | 0.002532 |

All 72 measured executions returned safely. `swmmrs` was faster on eleven
models and tied EPA on one. Its mean per-model median runtime was 66.9 seconds,
compared with 110.2 seconds for EPA. That is a mean reduction of 29.5%.

Ten of the twelve binary-output distances are below 0.003. The two outliers,
`46-pumps` and `greenville-snowmelt`, are explained below.

This result has a passport. It belongs to these engine builds, models, hardware,
operating system, compiler settings, and run conditions. It is not a universal
ranking for every machine in this or any neighboring solar system.

Distance summarizes selected report and output values. It is evidence of
similarity, not proof of complete numerical equivalence.

## Why `46-pumps.inp` remains different

Both engines have a difficult time with this model, which makes the difference
more interesting and less useful as a verdict.

The published 30-day run gives this model a 0.022239 report distance and
0.028547 output distance. That run predates the exact relational-operator parser
fix in solver commit `8adbd42`. Pumps initialized exactly at a `DEPTH <=`
shutoff threshold could therefore run briefly.

After the fix, a two-day diagnostic used a fixed 0.25-second routing step and
100 Dynamic Wave trials. The false startup at `PMP1-222` and `PMP2-222`
disappeared. The pumping-summary distance fell to 0.000902, but the report and
output distances remained at 0.013373 and 0.012877.

Neither engine converged reliably in that diagnostic. EPA failed to converge on
26.86% of routing steps, and `swmmrs` failed on 31.02%. Their external outflow
and final stored volume agree to the report precision. The dominant
`GM-824_38` flow stays below 0.7 GPM in both engines. The typical output
distance is 0.002631. Event timing raises the composite distance through its
0.043617 event component.

This model is a useful numerical-stability stress case. It is a poor parity
oracle while both engines leave so many routing steps unconverged. The
difference stays in the report because hiding an awkward result makes a prettier
chart and a worse benchmark.

## Why `greenville-snowmelt.inp` differs from EPA 5.2.4

The filename has done nothing wrong, but it does point suspicion in the wrong
direction. The 0.073754 report distance and 0.034562 output distance are
hydraulic, not a snowmelt discrepancy. Initial snow cover, precipitation,
infiltration, and final snow cover match to the report precision. Surface runoff
differs by 0.033 acre-ft, or 0.0104%.

The model connects `STOR-10` to the FREE `NW_CSO_OUTFALL` through the ungated
`ORI-33`, which has a 30 ft offset. A [known EPA 5.2.4 outfall
bug](https://github.com/USEPA/Stormwater-Management-Model/issues/154) sets the
outfall depth to zero when the connected link has a nonzero offset. EPA's [5.3
fix](https://github.com/USEPA/Stormwater-Management-Model/commit/4606020) and
`swmmrs` produce the 30 ft outfall depth instead.

That 30 ft fork in the timeline creates 7,388.745 million gallons of reverse
boundary flow. It accounts for 99.99% of the 7,389.294-million-gallon increase
in external inflow and drives the downstream flooding, pumping, and outflow
differences. This is an expected EPA-version difference, not a snowmelt
implementation mismatch.

## Relationship to regression validation

One sensor cannot answer every question. The [regression
suite](regression-validation.md) uses compact models to exercise a broad range
of hydrology, hydraulics, controls, routing, water-quality, and interface
behavior. The benchmarks use fewer, larger, and more interconnected models to
represent realistic simulation scale and feature interaction. Together, they
show both breadth across solver features and practical behavior on complex
workloads.
