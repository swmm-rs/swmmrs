# Antecedent Moisture Model RDII

For the Antecedent Moisture Model, yesterday's weather is part of today's
problem. `swmmrs` can use the reparameterized AMM to generate
rainfall-dependent infiltration/inflow (RDII), with antecedent moisture and
temperature affecting the response. You select it through three AMM input
sections. Released EPA SWMM does not recognize them.

AMM and native RTK unit-hydrograph RDII may target the same node. Their flows
are added once, written to the existing RDII interface stream, and routed and
reported through the normal wet-weather inflow path. A model can use AMM
without defining any subcatchments.

## Input sections

```swmm
[RAINGAGES]
RG1 VOLUME 0:05 1.0 TIMESERIES RAIN_TS

[TEMPERATURE]
TIMESERIES AIR_TEMP_TS

[AMM_MODELS]
;;Name RainGage ColdTemp HotTemp
SAN1   RG1      30.0     70.0

[AMM_COMPONENTS]
;;Model Component Type      RD     PAT  HHL  RW0    AMHL  TAT    SpringColdSHCF  HotSHCF  [FallColdSHCF]
SAN1    FAST      STANDARD  0.010  0.0  2.0  0.000  48.0  240.0  0.070           0.030    0.060

;;Model Component  Type      PAT    HHL    TAT    SpringColdR  HotR   [FallColdR]
SAN1    BASE       BASEFLOW  240.0  720.0  240.0  0.030        0.010  0.025
;comment 

[AMM_RDII]
;;Node  Model  SewershedArea
J1      SAN1   1000.0
```

Sections may appear in any order. Component IDs must be unique within their
model. You can reuse one model across assignments or send several assignments
to one node. Each model advances once per calculation time. Assignment flows
are scaled by area and summed, so reusing a model does not make its clock run
faster.

### Models

| Field | Meaning | US | SI |
| --- | --- | --- | --- |
| `Name` | Reusable AMM model ID | — | — |
| `RainGage` | Existing `[RAINGAGES]` ID | — | — |
| `ColdTemp` | Cold sigmoid calibration point | °F | °C |
| `HotTemp` | Hot sigmoid calibration point | °F | °C |

AMM uses the project `[TEMPERATURE]` source. Inline and external time series,
and USER, GHCND, TD-3200, and DLY02/04 climate files are supported. Temperature
history must cover the full averaging window before the first calculation.
The implementation samples history at `WET_STEP` intervals, so the required
start is `start - ceil(max_active_TAT / WET_STEP) * WET_STEP`, with both times
expressed in the same units. When TAT is a whole number of wet steps, this is
`start - max_active_TAT`.

The model needs a little prehistory before it can tell you what happens next.
For example, with TAT of 90 seconds and a wet step of 60 seconds, the first
average uses temperatures at the start, 60 seconds before, and 120 seconds
before. The oldest sample has half weight. Supply history from 120 seconds
before the start through the run. Missing history is an input error.

### Standard components

| Field | Meaning | Units |
| --- | --- | --- |
| `RD` | Dry-condition capture fraction | fraction |
| `PAT` | Precipitation averaging time | hours |
| `HHL` | Hydrograph half-life | hours |
| `RW0` | Initial wet-condition capture | fraction |
| `AMHL` | Antecedent-moisture half-life | hours |
| `TAT` | Temperature averaging time | hours |
| `SpringColdSHCF` | Cold factor, Jan 1–Jul 15 | 1/in US; 1/mm SI |
| `HotSHCF` | Hot factor | 1/in US; 1/mm SI |
| `FallColdSHCF` | Optional cold factor, Jul 16–Dec 31 | 1/in US; 1/mm SI |

When `FallColdSHCF` is omitted, the spring value applies all year.

### Baseflow components

| Field | Meaning | Units |
| --- | --- | --- |
| `PAT` | Precipitation averaging time | hours |
| `HHL` | Hydrograph half-life | hours |
| `TAT` | Temperature averaging time | hours |
| `SpringColdR` | Cold capture, Jan 1–Jul 15 | fraction |
| `HotR` | Hot capture | fraction |
| `FallColdR` | Optional cold capture, Jul 16–Dec 31 | fraction |

When `FallColdR` is omitted, the spring value applies all year.

### Assignments

`[AMM_RDII]` rows contain an existing node ID, an AMM model ID, and contributing
sewershed area. Area is acres in US projects and hectares in SI projects.

The parser rejects missing references, duplicate model or component IDs,
non-finite values, `HotTemp <= ColdTemp`, nonpositive half-lives, negative
windows or capture parameters, nonpositive assignment areas, and AMM use when
rainfall or RDII processing is ignored.

Seasonal SHCF and baseflow capture are clamped to zero before updating moisture
and flow, matching AMM for PCSWMM. This prevents negative seasonal extrapolation
from producing negative AMM contributions or reducing native RTK inflow.
Capture is not capped at 1. The report emits one warning for each component
that exceeds 1 during generation.

## Equations and timing

AMM uses `WET_STEP` as $\Delta t$. Rain at time $t$ is the incremental
start-of-interval depth for $[t,t+\Delta t)$. Rainfall before the simulation is
zero. A fractional averaging window $W$ uses $w=W/\Delta t$,
$m=\lfloor w\rfloor$, and $f=w-m$:

$$
MA_t=\frac{\sum_{j=0}^{m}x_{t-j}+f x_{t-m-1}}{w+1}.
$$

`PAT=0` and `TAT=0` therefore use the current value. Only the bounded history
needed by the largest active window is retained.

For a standard component:

$$
SF=0.5^{\Delta t/HHL},\qquad AMRF=0.5^{\Delta t/AMHL},
$$

$$
RW_t=\frac{AMRF-1}{\ln(AMRF)}SHCF_t MAP_t+AMRF RW_{t-1},
$$

$$
Q_t=A\left(RD+\frac{RW_t+RW_{t-1}}{2}\right)MAP_t
\frac{1-SF}{\Delta t}+SFQ_{t-1}.
$$

`RW[-1] = RW0` and `Q[-1] = 0`. The implementation evaluates the moisture
correction with `expm1(x) / x` for stability.

For moving-average temperature $MAT_t$, define:

$$
L=1.2(Cold-Hot),\quad k=\frac{4.7964}{ColdTemp-HotTemp},\quad
x_0=\frac{ColdTemp+HotTemp}{2},
$$

$$
Season_t=\max\left(0,\frac{L}{1+e^{-k(MAT_t-x_0)}}+Cold-\frac{11L}{12}\right).
$$

`Cold` and `Hot` are SHCF values for standard components and capture fractions
for baseflow components. The sigmoid is branch-stable at extreme temperatures.
The zero lower bound is an implementation choice from AMM for PCSWMM, rather
than part of C525's printed equation. Nonnegative calibration values alone do
not prevent the unbounded expression from becoming negative outside the
calibration temperatures.

Baseflow omits `RD` and the antecedent-moisture recurrence:

$$
Q_t=A\left(\frac{R_t+R_{t-1}}{2}\right)MAP_t
\frac{1-SF}{\Delta t}+SFQ_{t-1}.
$$

`R[-1]` is the seasonal value at the first available averaged temperature;
`Q[-1] = 0`.

## RDII files, checkpoints, and overrides

The generated binary RDII stream contains the ascending union of RTK and AMM
nodes. AMM calculations remain in double precision until the existing stream's
single-precision payload boundary.

```swmm
[FILES]
SAVE RDII generated.rdii
```

`SAVE RDII` retains the combined RTK+AMM stream. It can be replayed with:

```swmm
[FILES]
USE RDII generated.rdii
```

With `USE RDII`, the saved file has the final say. Configured AMM declarations
remain visible in the input summary, but the run bypasses both native RTK and
AMM generation. The report states that AMM generation is inactive.

Simulation checkpoints store AMM declarations in the configuration projection
and retain generated RDII progress through the existing RDII sidecar. Resume
and fork do not serialize or recompute AMM ring buffers because the complete
interface stream is generated before routing starts.

AMM model IDs and model objects must currently be declared in the input file.
After a project is opened, the Rust and Python model-definition APIs can update
an existing AMM model's rain gage, temperature bounds, and complete component
list, and can replace the AMM node-assignment list. Edits are allowed only while
the simulation is `Open` or `Ended`, not while it is `Running`. The APIs do not
create or delete top-level AMM model objects.

## Run an example

The repository includes equivalent
[`amm-us.inp`](https://github.com/swmm-rs/swmmrs/blob/main/crates/solver/tests/data/amm-us.inp)
and
[`amm-si.inp`](https://github.com/swmm-rs/swmmrs/blob/main/crates/solver/tests/data/amm-si.inp)
examples. Both omit subcatchments.

```sh
runswmmrs crates/solver/tests/data/amm-us.inp amm-us.rpt amm-us.out
runswmmrs crates/solver/tests/data/amm-si.inp amm-si.rpt amm-si.out
```

The report includes an **AMM Input Summary**, identifies RDII analysis as active,
and reports AMM inflow through the normal RDII and routing summaries.

## Sources and implementation choices

The equations follow Edgren, Czachorski, and Gonwa,
[“Reparameterizing the Antecedent Moisture Model”](https://doi.org/10.14796/JWMM.C525),
JWMM C525, published October 3, 2024 under CC BY 4.0.

Fractional windows, the July 15/16 seasonal boundary, and the zero lower bound
on seasonal SHCF and baseflow capture follow RJN Group's
MIT-licensed
[AMM for PCSWMM B.9](https://github.com/RJNGroup/AMM-for-PCSWMM/tree/b1d07e3f4a02ba5c874ae7e042177edd2583f573).
The baseflow recurrence includes its
[correction](https://github.com/RJNGroup/AMM-for-PCSWMM/commit/4e5d25580bd3211d586588590e54752bcbda7ad3).

The paper, the reference implementation, and the startup behavior do not agree
on every detail. Here is what `swmmrs` chooses, with independent tests for each
decision:

- C525's printed Table 1 flow values diverge from its printed equation after
  02:00. The equation-derived values are authoritative.
- Rain at the simulation start is consumed once; the PCSWMM startup path can
  duplicate that first value.
- Baseflow initializes `R[-1]` from the first temperature instead of zero.
- Seasonal SHCF and baseflow capture use PCSWMM's zero lower bound before the
  recurrences.
- The unversioned companion workbook has separate copyright terms; no workbook
  fixtures or formulas are copied.
