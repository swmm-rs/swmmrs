# Collect results and statistics during a run

`swmmrs` has four result-access patterns:

- **Live scalar views** for one or a few current values, such as
  `simulation.nodes["J1"].depth`.
- **Immutable native snapshots** for a coherent batch of selected objects, such
  as `simulation.nodes.snapshot(["J1", "J2"])`.
- **Cumulative statistics** for system continuity, object extremes, and subtype
  summaries acquired in `RUNNING` or `COMPLETE`.
- **Binary output queries** through `OutputReader` after routing stops, including
  automatic recovery of complete records when finalization did not finish.

The first three read in-memory solver state; `OutputReader` independently reads
the binary artifact.
Collect a time series while advancing the model, then retain ordinary Python
data or snapshots after the simulation closes.

```python
from datetime import timedelta

from swmmrs import Simulation

records = []
with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    monitor = simulation.nodes["J1"]
    conduit = simulation.links["C1"]
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        records.append(
            (current_time, monitor.depth, monitor.total_inflow, conduit.flow)
        )

    simulation.end()
    simulation.report()

# `records` is ordinary Python data and remains usable.
```

The timestamp and values in an iterator body describe the state after that
iterator advance. `step_advance()` defaults to `strict=True`, setting an exact
Python callback cadence that can shorten the final routing step at each boundary.
If step-equivalent numerical results matter more than exact callback timestamps,
use `step_advance(..., strict=False)` or a caller-owned
`stride(..., strict=False)` loop instead.

## Statistics acquisition map

| Acquisition | Result |
| --- | --- |
| `simulation.statistics` | Coherent system totals, continuity, diagnostics, and quality balances |
| `node.statistics`, `link.statistics`, `subcatchment.statistics` | One object's cumulative summary |
| `nodes/links/subcatchments.statistics(ids=None)` | Aligned immutable batch statistics |
| `storage.storage_statistics` | Storage-specific cumulative summary |
| `outfall.outfall_statistics` | Outfall flow frequency and pollutant loads |
| `pump.pump_statistics` | Pump utilization, flow, volume, energy, and off-curve time |
| `subcatchment.lid_snapshot()` and LID unit `.snapshot()` | Current LID runtime records rather than cumulative statistics |

Acquire final statistics after natural completion places the owner in
`COMPLETE`, and before `end()`. Existing records remain usable afterward.

## How this relates to PySWMM

The live loop is familiar to PySWMM users: advance the simulation, read a node
or link, and save the value in host-owned data. `swmmrs` keeps that workflow
but offers typed immutable snapshots for a network-wide or selected-object
read.

PySWMM also has a separate [Output module](https://pyswmm.github.io/pyswmm/reference/output.html)
for post-run time-series extraction from an `.out` file. `swmmrs` provides the
independent [`OutputReader`](read-output.md) for the same stage of a workflow.
It detects finalized output or recovers complete records from an incomplete
artifact, then returns typed metadata, single-family time series, structured
bulk series, and exact Stored Report Dates. Keep collecting live values during
routing when Python must inspect or control the active simulation; use
`OutputReader` after routing stops when the report-period binary record is the
required source.

## When data is valid

The lifecycle determines whether an acquisition is allowed. A view's identity
and configuration are separate from its current hydraulic result.

| Data | `OPEN` | `RUNNING` | `COMPLETE` | `ENDED` | After `close()` or a later `open()` |
| --- | --- | --- | --- | --- | --- |
| Live current hydraulic and quality properties | Not valid | Valid; changes after each advance | Valid final state | Valid final state | View is stale |
| `snapshot()` and `quality_snapshot()` | Not valid | Valid | Valid final state | Valid final state | Existing snapshot remains valid; collection is stale |
| `simulation.statistics`, object `.statistics`, and collection `.statistics()` | Not valid | Valid cumulative values | Valid final cumulative values | Not valid | Not valid |
| Host-owned tuples, lists, dicts, and snapshot/statistics records already copied | Valid | Valid | Valid | Valid | Valid |

`COMPLETE` is reached when manual or iterator advancement reaches the model
end. Iterator exhaustion remains observable as `COMPLETE`; acquire final
statistics there, then call `end()` before `report()`. Context exit can perform
finalization and close when no report is needed.

```python
from swmmrs import Simulation

simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    simulation.start(save_results=True)
    while simulation.step() is not None:
        pass

    # The model is COMPLETE: final hydraulics and statistics are available.
    final_nodes = simulation.nodes.snapshot()
    final_links = simulation.links.snapshot()
    continuity = simulation.statistics
    node_extremes = simulation.nodes.statistics()

    simulation.end()
    simulation.report()
finally:
    simulation.close()

# These copied records remain usable after close().
print(continuity.routing_totals.continuity_error)
```

After `end()`, final scalar hydraulics and hydraulic/quality snapshots remain
available, but statistics cannot be acquired again. Copy statistics before
ending. A failed simulation has no valid current-result acquisition path:
catch its `SolverError`, preserve any records already copied, then close it.

## Live views and stale views

`simulation.nodes["J1"]`, `simulation.links["C1"]`, and their collections are
generation-bound live views. A view does not cache its result; each current
property asks its owner for the present solver value. It is safe to retain the
view across routing advances in the same generation, but its values can change
at every advance.

Do not retain a live view or collection after either of these boundaries:

```python
node = simulation.nodes["J1"]
nodes = simulation.nodes

simulation.close()  # `node` and `nodes` are now stale.
simulation.open("another-model.inp", "another-model.rpt")
# Reacquire: simulation.nodes["J1"]
```

Using stale views raises `StaleViewError`. This prevents a node identity from
silently referring to a different project generation. It is also why a
snapshot is the right value to send to another thread, write after context
exit, or keep as a historical record.

## Scalar reads: monitoring and control

Use live scalar properties when the workflow needs a small number of values at
each callback:

- dashboard or SCADA-style display of one monitored node;
- real-time control threshold or hysteresis decision;
- calibration residual for one observation location;
- online peak, volume, or exceedance calculation.

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    monitor = simulation.nodes["J1"]
    peak_depth = 0.0
    exceedance_seconds = 0.0
    simulation.step_advance(timedelta(minutes=5))

    for _ in simulation:
        peak_depth = max(peak_depth, monitor.depth)
        if monitor.depth > 3.0:
            exceedance_seconds += 5 * 60

    # Iteration is COMPLETE; `peak_depth` is already host-owned.
    # Context exit finalizes and closes the run.
```

Each scalar property is a live acquisition. Do not build a network-wide
"same-time" record by separately reading hundreds of object properties,
especially if another thread might advance the same owner. Use a snapshot for
that. Better still, do not advance one `Simulation` from multiple threads;
use independent owners for concurrent scenarios.

## Batch snapshots: coherent network samples

`nodes.snapshot()`, `links.snapshot()`, and `subcatchments.snapshot()` copy
all requested current hydraulic columns in one owner acquisition. The result is
a frozen native PyO3 record whose public columns are immutable tuples. Its
columns are aligned with `object_ids`, and a requested ID sequence determines
the output order.

```python
from datetime import timedelta

from swmmrs import Simulation

rows = []
with Simulation("model.inp", "model.rpt") as simulation:
    simulation.step_advance(timedelta(minutes=15))

    for current_time in simulation:
        nodes = simulation.nodes.snapshot(["J1", "J2"])
        for object_id, depth, inflow in zip(
            nodes.object_ids,
            nodes.depth,
            nodes.total_inflow,
            strict=True,
        ):
            rows.append((current_time, object_id, depth, inflow))
```

Pass `None` (the default) for all configured objects, one string or exact
non-boolean integer for one object, or an ordered iterable of IDs for a subset.
IDs are case-insensitive; returned IDs use their canonical configured spelling.
Duplicate IDs after case-insensitive resolution raise `ValidationError`; an
unknown ID raises `KeyError`. Booleans, bytes, mappings, sets, non-iterables, and
iterables containing unsupported members raise `ValidationError` before object
lookup. Select only the objects needed for high-frequency sampling, one narrow
snapshot is cheaper and clearer than a full-network snapshot followed by
filtering in Python.

Choose snapshots when a consumer needs a coherent cross-section, historical
record, or data to hand off after the solver advances. Existing snapshots do
not change when a target setting changes, the model advances, the run ends, or
the owner closes.

### Native record behavior and mutable copies

Snapshot and statistics values are Rust-backed Python classes created only by
solver acquisitions. Callers cannot construct them directly or assign to their
properties. Their tuples, tuple matrices, and pollutant mappings are read-only;
`datetime` and `timedelta` properties are detached ordinary Python values.
Records compare by their exposed values and use a deterministic field-named
`repr`.

`copy.copy()` and `copy.deepcopy()` are unsupported and raise `TypeError`; they
do not create an editable record. Copy the fields needed by the downstream task
into mutable host containers instead:

```python
sample = simulation.nodes.snapshot(["J1", "J2"])
editable = {
    "object_ids": list(sample.object_ids),
    "depth": list(sample.depth),
}
editable["depth"][0] = 2.5
```

The edit changes only `editable`. It does not change `sample` or the simulation.
Likewise, use `dict(outfall.pollutant_loads)` or
`dict(simulation.statistics.quality_balances)` when a mutable mapping is needed.
Use the live settings or forcing APIs to change solver state, then acquire a new
record.

### Snapshot fields and units

| Snapshot | Main fields | Units |
| --- | --- | --- |
| `subcatchments.snapshot()` | rainfall, evaporation, infiltration, runon, runoff, snow depth | Rainfall/evaporation/rain-depth and project flow units as named; snow depth uses report rain-depth units. |
| `nodes.snapshot()` | depth, head, volume, lateral/total inflow, outflow, losses, flooding, hydraulic retention time | Project length, volume, and flow units; retention time is `timedelta` or `None` for node kinds that do not report it. |
| `links.snapshot()` | setting, target setting, open/closed time, flow, depth, velocity, width, volume, capacity, surface areas, Froude number | Project flow, length, length/sec, volume, and area units; times are `timedelta`; unavailable subtype values such as `top_width` are `None`. |

`simulation.flow_units` and `simulation.unit_system` identify the project's
flow and unit system. No hidden conversion to SI occurs. Keep the model unit
labels beside exported columns, especially when combining runs from projects
with different `FLOW_UNITS`.

## Quality snapshots

Use `quality_snapshot()` when the same pollutant set is needed for multiple
objects. It returns immutable pollutant-major matrices: the first index is the
position in `pollutant_ids`, and the second is the position in `object_ids`.

```python
with Simulation("model.inp", "model.rpt") as simulation:
    next(simulation)  # Starts and advances once.

    quality = simulation.nodes.quality_snapshot(["J1", "J2"])
    for pollutant_index, pollutant_id in enumerate(quality.pollutant_ids):
        for node_index, node_id in enumerate(quality.object_ids):
            concentration = quality.concentrations[pollutant_index][node_index]
            print(node_id, pollutant_id, concentration)
```

Node quality includes current, inflow, and reactor concentrations. Link quality
includes current and reactor concentrations plus total routed loads.
Subcatchment quality includes runoff and ponded concentrations, buildup loads,
and total washoff loads. Concentrations and loads use the reporting units
configured for each pollutant; retain `pollutant_ids` with exported values
because different pollutants can have different reporting conventions.

For one object and one pollutant, scalar mappings such as
`simulation.nodes["J1"].pollut_quality["TSS"]` are simpler. Use a quality
snapshot when extracting several pollutants or several objects together.

## LID snapshots

LID runtime state has two owned record shapes. The subcatchment-level record
summarizes the LID group, while a unit record contains its water balance,
depths, moisture, drain flow, and layer fluxes:

```python
subcatchment = simulation.subcatchments["With-Lids"]
group = subcatchment.lid_snapshot()
unit = subcatchment.lid_units[0].snapshot()

print(group.pervious_area, group.new_drain_flow)
print(unit.surface_depth, unit.soil_moisture, unit.dry_time)
```

Both records follow hydraulic snapshot lifecycle and ownership rules. They are
read-only and remain usable after the run advances or the owner closes.

## Statistics: cumulative and final QA

Statistics are owned, immutable cumulative records, not an `.out`-file query.
They are useful for continuity checks, peak values, flooding duration, pump
use, and final scenario summaries.

```python
# While RUNNING or COMPLETE:
system = simulation.statistics
links = simulation.links.statistics(["C1", "P1"])

print(system.routing_totals.continuity_error)
print(links.maximum_flow)
print(links.time_surcharged)
```

`simulation.statistics` contains routing and runoff continuity totals plus
groundwater and quality continuity errors. Its pollutant quality balances are
available through an insertion-ordered, read-only mapping. Collection
statistics are aligned batch records just like hydraulic snapshots. Object
properties such as `simulation.nodes["J1"].statistics` return one immutable
record. Subtype-specific records are explicit:

```python
storage = simulation.nodes["S1"].storage_statistics
outfall = simulation.nodes["O1"].outfall_statistics
pump = simulation.links["P1"].pump_statistics
```

Statistics can be sampled while running for a progress display, but they are
cumulative to the current model time; copy them at `COMPLETE` for final QA
before `end()`.

`save_results=False` disables saved report-period binary results and
`report()`, but it does **not** disable live results, snapshots, quality
snapshots, or statistics. This is the lean choice for calibration and
sensitivity runs that compute their own objective or summary in memory.

## Practical collection workflows

| Goal | Recommended acquisition | Retain |
| --- | --- | --- |
| One gauge, a control threshold, or a peak | Scalar live properties each callback | Numbers or tuples in a list. |
| Wide network dashboard or CSV-style export | Selected hydraulic snapshot each callback | Rows built by zipping `object_ids` with aligned columns. |
| Quality monitoring | Selected quality snapshot | `pollutant_ids`, `object_ids`, and pollutant-major values together. |
| Calibration against observations | Scalar or narrow snapshot at observation cadence | Timestamped residuals and objective components. |
| Final continuity and design QA | Manual completion, then statistics before `end()` | `SimulationStatistics` and object statistics snapshots. |
| Post-run binary analysis | `OutputReader` over an explicit `.out` path | Immutable metadata, nominal times, exact stored dates, and structured result series. |

Keep collection simple: append tuples or dictionaries during the loop, then
write CSV or hand the completed records to a dataframe library after the
context closes. No extra dependency is required to collect correct output.
