# Runtime forcings and scenario workflows

Runtime forcing is a value supplied by the host application while a project is
open or running. Use it for a measured inflow, forecast precipitation, an
operator action, or a scenario input that should not require rewriting the
`.inp` file. `swmmrs` retains supported forcing values in the owning
`Simulation` and reapplies them when SWMM initialization resets native fields.

All values use the model's configured units. Check `simulation.flow_units` and
`simulation.unit_system` before passing telemetry or forecast data into a
model.

## The familiar PySWMM control loop

The usual PySWMM real-time-control pattern also works in `swmmrs`: enter a
context manager, advance to a decision cadence, read results, and write a
control. The object access is direct from the simulation rather than through
separate `Nodes` and `Links` collections.

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    basin = simulation.nodes["BASIN"]
    gate = simulation.links["OUTLET_GATE"]
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        if basin.depth >= 4.0:
            gate.target_setting = 1.0
        elif basin.depth <= 2.0:
            gate.target_setting = 0.0

    simulation.end()
    simulation.report()
```

The first iterator advancement starts the model. Each loop body observes the
state after its preceding routing advance, so a decision affects the next
routing operation. `step_advance()` defaults to `strict=True`, returning control
at exact cadence boundaries and potentially shortening the final routing step.
`step_advance(..., strict=False)` or a caller-owned `stride(..., strict=False)`
loop preserves ordinary routing steps but can return after the requested target.

The control loop is deliberately explicit. PySWMM offers callback registration;
`swmmrs` uses the loop body as the scheduler. This keeps the lifecycle and the
point at which host code changes a model visible. `swmmrs` also exposes typed
`SimulationState` values and permits explicit `start()`, `step()`, `end()`, and
`close()` when a host needs manual lifecycle control. See the
[PySWMM Simulation reference](https://pyswmm.github.io/pyswmm/reference/api/pyswmm.simulation.Simulation.html)
for its callback-oriented API and [Run a model](run-model.md) for the `swmmrs`
lifecycle.

## Supported runtime inputs

| Need | Property | Semantics and write state |
| --- | --- | --- |
| Open or close a regulator; set pump speed | `simulation.links["ID"].target_setting` | Active-run target setting. Valid only in `RUNNING`; regulator values clamp to `[0, 1]`, while pump speed is nonnegative. |
| Add measured or forecast flow at a node | `simulation.nodes["ID"].external_inflow` | Persistent **additive** external inflow in project flow units. Valid in `OPEN`, `RUNNING`, and `ENDED`. |
| Supply catchment precipitation | `simulation.subcatchments["ID"].external_rainfall` or `.external_snowfall` | Persistent additive rainfall or snowfall in project rainfall units. Valid in `OPEN`, `RUNNING`, and `ENDED`. |
| Make the host application a rain gage's precipitation source | `simulation.rain_gages["ID"].use_external_precipitation(rate)` | Named persistent source switch in project rainfall units. Selects the API source, marks the gage used, and unlinks any co-gage. |
| Mask a rain gage's current source | `simulation.rain_gages["ID"].rainfall_override` | Persistent highest-priority rainfall override. Assign `None` to clear it and resume the underlying source. |
| Force boundary stage | `simulation.nodes["OUTFALL"].fixed_stage` | Persistent fixed outfall stage in project length units. |
| Supply quality mass or buildup | `node.external_pollutant_mass_flux["TSS"] = value`, `link.external_pollutant_mass_flux.update(...)`, or `subcatchment.external_pollutant_buildup_increment[...] = value` | Persistent live mappings keyed by canonical pollutant ID. Item assignment is sparse; `update()` is one atomic multi-pollutant mutation. |
| Override current node or link quality | `node.override_pollutant_concentrations({...})` or `link.override_pollutant_concentrations({...})` | Atomic nonnegative concentration overrides consumed by the next routing-quality step. Valid only in `RUNNING`. |
| Scale a subcatchment's gage precipitation | `subcatchment.set_precipitation_scale_factors(rainfall=..., snowfall=...)` | Named atomic update of both positive multipliers. Valid in `OPEN`, `RUNNING`, and `ENDED`. |

`target_setting` is a live control, not a persistent forcing source. Set it
when the control decision changes and set it again in every restarted run that
needs a non-default target. The other entries are retained until replaced; set
them to `0.0` to remove an inflow, external rainfall, snowfall, or quality
source. Rain-gage forcing has distinct clearing behavior described below.
Persistent forcings are discarded on `close()` and are not carried into a
later `open()` generation. Checkpoint Resume and Fork copy the source owner's
forcing; Enhanced Hotstart through `load_checkpoint_state()` keeps the already
open receiver's forcing instead.

Subcatchment `external_rainfall` is additive: reported catchment rainfall can
include both gage precipitation and the external amount. Rain-gage
`external_precipitation_rate` and `rainfall_override` instead determine the
gage's effective precipitation, with the override taking precedence.
`total_precip` is the read-only current effective rain-plus-snow result.

## Forcing rain-gage precipitation

Rain gages expose two persistent forcing mechanisms because `swmmrs` preserves
both OWASWMM's external API precipitation source and the canonical expanded-EPA
rainfall override. Both values are rates, not one-step pulses: the last assigned
value applies to every subsequent routing step and repeated `start()` on the same
open project generation until it is replaced, cleared where supported, or
discarded by `close()`.

Use `use_external_precipitation()` when the host owns the complete precipitation
stream:

```python
gage = simulation.rain_gages["FORECAST_GAGE"]
gage.use_external_precipitation(forecast_rate)
```

Calling it changes the gage source to the API, marks the gage as used, and
unlinks any co-gage. Call it only when the external measurement or forecast
changes. A value of `0.0` keeps the API selected with a dry rate; it does not
restore the former file or time-series source. The read-only
`external_precipitation_rate` property is `None` before this source is selected.

Use `rainfall_override` when the gage already has a valid source that should
remain configured but be masked for a period:

```python
gage.rainfall_override = 1.5  # persists and masks every source
gage.rainfall_override = 0.0  # persists and forces dry precipitation
gage.rainfall_override = None  # clear and resume the underlying source
```

The override has higher priority than `external_precipitation_rate` and does
not change the gage's `data_source`. Setting an override unlinks any co-gage;
clearing it does not reconstruct that relationship. Therefore, use the
external rate for an authoritative telemetry or forecast feed and the override
for a reversible value mask over an already valid direct gage source.

Read `total_precip`, `rainfall`, and `snowfall` for current effective results.
SWMM partitions the selected gage precipitation into rain or snow from the
model temperature and snow settings, so `total_precip` need not equal
`rainfall`.

## Pollutant forcing and one-step overrides

Current concentration and load properties are immutable result mappings.
`pollut_quality` therefore has no setter. To override one or more node or link
concentrations for exactly the next routing-quality step, use the named atomic
operation:

```python
node.override_pollutant_concentrations({"TSS": 100.0, "Lead": 0.25})
link.override_pollutant_concentrations([("TSS", 80.0), ("Lead", 0.10)])
```

The complete payload is resolved and validated before any override is queued.
Pollutant IDs are case-insensitive, duplicate aliases are rejected, values must
be finite and nonnegative, and the operation is valid only in `RUNNING`.
Omitted pollutants preserve any override already queued for that same step.
The next quality-routing advance consumes the values; an ordinary end/restart
also clears unconsumed overrides.

Persistent pollutant sources use dense live mappings in configured pollutant
order:

```python
flux = node.external_pollutant_mass_flux
flux["TSS"] = 2.5
flux.update({"TSS": 3.0, "Lead": 0.1})  # one atomic sparse write
del flux["Lead"]  # reset one value to 0.0
flux.clear()  # atomically reset every value to 0.0
```

The same protocol applies to `link.external_pollutant_mass_flux` and
`subcatchment.external_pollutant_buildup_increment`. Deletion and `clear()`
reset values but do not remove configured pollutant keys. Whole-property
assignment is unsupported; the mapping itself is the mutation interface.
Failed multi-key writes leave every value and the owner lifecycle unchanged.
Persistent mappings remain applied across reruns and are discarded on close.

Nonzero `link.external_pollutant_mass_flux` values are supported only for real
conduits. Pumps, orifices, weirs, outlets, and dummy conduits have no conduit
quality reactor, so a candidate containing any nonzero value is rejected
atomically; `clear()` and other all-zero replacements remain valid. When a
conduit ends a quality-routing step dry, positive forcing is counted as link
load and external inflow and deposited into final dry storage without producing
an aqueous concentration. That deposited mass is a terminal continuity sink, it
is not remobilized if the conduit later rewets. Negative forcing on a dry
conduit is a no-op because there is no aqueous mass to remove. Wet-conduit
positive and negative forcing retains the ordinary mixing and bounded-removal
behavior.

When any node type ends a quality-routing step dry with negligible hydraulic
inflow, positive `node.external_pollutant_mass_flux` is counted as external
inflow and deposited into final dry storage without producing an aqueous
concentration. As with dry conduits, this is a terminal continuity sink and is
not remobilized if the node later rewets. Negative forcing on a dry node is a
no-op, while wet-node mixing and bounded removal are unchanged.

## Persistence and timing

Persistent forcing is intentionally held between routing calls and across
`start()` calls on the same open generation. It is appropriate to set an
initial value before starting, then replace it only when the host has a new
measurement or forecast.

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    inflow = simulation.nodes["MONITORED_INLET"]
    gage = simulation.rain_gages["FORECAST_GAGE"]

    # These values affect the first routing advance.
    inflow.external_inflow = latest_measured_flow()
    gage.use_external_precipitation(forecast_rainfall(simulation.start_time))
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        # These replacements persist through the next callback and beyond.
        inflow.external_inflow = latest_measured_flow()
        gage.use_external_precipitation(forecast_rainfall(current_time))

    simulation.end()
    simulation.report()
```

A persistent write is valid in `OPEN`, `RUNNING`, and `ENDED`, but not in
`COMPLETE`, `FAILED`, or `CLOSED`. Writing after `end()` changes `ENDED` back
to `OPEN` and resets the completed run's clock and report-period bookkeeping.
That makes the next `start()` a clean run with the retained forcing. End a
manually or iterator-advanced run before changing a forcing after natural completion.

## Hydraulic real-time control

A hydraulic RTC loop normally has four parts:

1. Set a decision cadence that is short enough for the controlled asset.
2. Read the measured model state after each advance.
3. Apply a bounded, deterministic control decision.
4. Feed new telemetry or forecasts through persistent inputs only when they
   change.

Use `target_setting` for a gate, regulator, or pump response. Avoid writing it
unconditionally if the target has not changed; the model already retains the
current target. Use a small pure function for the controller so its behavior is
testable outside SWMM.

```python
from datetime import timedelta

from swmmrs import Simulation


def gate_target(depth: float, previous: float) -> float:
    if depth >= 4.0:
        return 1.0
    if depth <= 2.0:
        return 0.0
    return previous


with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    storage = simulation.nodes["STORAGE"]
    gate = simulation.links["GATE"]
    target = 0.0
    simulation.step_advance(timedelta(minutes=5))

    for _ in simulation:
        target = gate_target(storage.depth, target)
        if gate.target_setting != target:
            gate.target_setting = target

    simulation.end()
    simulation.report()
```

This hysteresis prevents the gate from repeatedly toggling when depth hovers
near one threshold. It is host policy, not a replacement for SWMM control
rules already encoded in the input file. Pick one source of authority for an
asset: host RTC and input-file rules can both change the same link, making the
result dependent on their execution order and difficult to audit.

## Calibration runs

Calibration is usually a sequence of independent candidate runs: set static
parameters before a run, advance at the observation cadence, accumulate an
objective in memory, then start the next candidate from a clean state.

Configuration properties such as a subcatchment `width` are not live controls.
They are valid in `OPEN` and `ENDED`, but not `RUNNING`. A configuration write
from `ENDED` returns the project to `OPEN`, which makes a single owner useful
for serial candidate evaluation. This example skips binary results because the
objective is computed in memory.

```python
from datetime import timedelta

from swmmrs import Simulation


def score_candidates(candidates: list[float]) -> list[tuple[float, float]]:
    scores = []
    with Simulation("model.inp", "calibration.rpt") as simulation:
        subcatchment = simulation.subcatchments["S1"]
        monitor = simulation.nodes["J1"]
        simulation.step_advance(timedelta(minutes=5))

        for width in candidates:
            subcatchment.width = width
            simulation.start(save_results=False)
            squared_error = 0.0
            sample_count = 0

            while (current_time := simulation.stride(timedelta(minutes=5))) is not None:
                residual = monitor.depth - observed_depth_at(current_time)
                squared_error += residual * residual
                sample_count += 1

            simulation.end()
            scores.append((width, squared_error / sample_count))
    return scores
```

Collect values during the run, not from stale live views after `close()`. Keep
the objective calculation deterministic: align observation timestamps to the
chosen callback cadence, define missing-data treatment once, and avoid changing
both forcing and physical parameters in the same candidate unless that is the
experiment.

## Sensitivity analyses

For sensitivity analysis, prefer one isolated `Simulation` per candidate.
This produces independent initial state, forcing storage, report files, and
native ownership. It also lets the standard library run candidates in parallel
without sharing a mutable owner.

```python
from concurrent.futures import ThreadPoolExecutor
from datetime import timedelta
from pathlib import Path

from swmmrs import Simulation


def peak_depth(
    candidate: tuple[int, float], run_directory: Path
) -> tuple[float, float]:
    candidate_index, width = candidate
    report_path = run_directory / f"candidate-{candidate_index}.rpt"
    with Simulation("model.inp", report_path) as simulation:
        subcatchment = simulation.subcatchments["S1"]
        monitor = simulation.nodes["J1"]
        subcatchment.width = width
        simulation.start(save_results=False)

        peak = 0.0
        while simulation.stride(timedelta(minutes=5)) is not None:
            peak = max(peak, monitor.depth)

        simulation.end()
    return width, peak


run_directory = Path("sensitivity-runs")
run_directory.mkdir(exist_ok=True)
candidates = [200.0, 250.0, 300.0, 350.0]
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(
        executor.map(
            lambda candidate: peak_depth(candidate, run_directory),
            enumerate(candidates),
        )
    )
```

Every concurrent owner needs a distinct report path; omit `output_path` when
binary output is unnecessary and `swmmrs` uses scratch output. Do not share one
`Simulation`, its live views, or its artifact paths between workers. Budget
Dynamic Wave `THREADS` across all candidates. For example, four concurrent
models with `THREADS 2` can use up to eight caller-inclusive solver threads.

## Gotchas

- Runtime values must be finite. Rainfall, snowfall, external precipitation
  rates, and pollutant concentrations that require nonnegative values reject
  negatives. Finite mass fluxes whose native unit conversion would overflow are
  also rejected before mutation; use `ValidationError` as an input-data boundary,
  not as flow control.
- `target_setting` is invalid before `start()` and after natural completion.
  Call it only from `RUNNING` control code.
- A manually or iterator-advanced model becomes `COMPLETE` at natural
  exhaustion. Call `end()` before changing a persistent forcing or starting the
  next candidate.
- `report()` is available only for a run started with `save_results=True`.
  Calibration and sensitivity loops commonly use `False` and retain their
  metrics in memory instead.
- Simulation and input timestamps are timezone-naive. Convert external
  telemetry to the model's clock before indexing a forecast or observation.
- `close()` invalidates every live view. Reacquire objects after `open()` and
  never copy or pickle a `Simulation` to distribute a workload.
