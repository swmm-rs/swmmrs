# Configure a model

Set options, dates, geometry, and initial conditions while the project is `OPEN`, before routing starts.

## Choose the right kind of input

| Value | Examples | Lifecycle |
| --- | --- | --- |
| Project configuration | Dates, routing policy, geometry, initial state | Write in `OPEN` or `ENDED` |
| Runtime forcing | Measured inflow, forecast rain, stage, target setting | Use the states documented by each property, including `RUNNING` |
| Current result | Depth, flow, runoff, quality | Read in `RUNNING` or `COMPLETE` |
| Owned result | Snapshots and statistics | Copy while valid; retain after advancement or closure |

Use [runtime forcings](runtime-forcings.md) for data or decisions that change during a run.

## Inspect the project

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    print(simulation.unit_system, simulation.flow_units)
    print(tuple(simulation.nodes))

    first_node = simulation.nodes.by_index(0)
    assert simulation.nodes[first_node.id.lower()].id == first_node.id
```

Collections:

- iterate over IDs in configured order;
- use case-insensitive string lookup;
- use zero-based `by_index()` lookup;
- return typed, generation-bound views;
- become stale after the owner closes or reopens.

## Set options and model inputs

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    # Solver policy and dates atomically commit requested declarations. Effective
    # values and dependent geometry are prepared at the next start().
    simulation.options.update(
        routing_step=timedelta(seconds=30),
        report_step=timedelta(minutes=15),
        allow_ponding=True,
        requested_threads=4,
    )
    simulation.update_schedule(
        end_time=simulation.start_time + timedelta(hours=6)
    )

    # Selected object inputs.
    simulation.nodes["J1"].settings.update(full_depth=4.5, initial_depth=0.25)
    simulation.links["C1"].settings.initial_flow = 0.1
    simulation.links["C1"].flow_limit = 8.0
    simulation.subcatchments["S1"].settings.update(width=250.0, slope=0.012)

    # Aquifer and snowmelt definitions retain requested project-unit values;
    # pattern/subcatchment relationships and effective native normalization wait
    # for the next start().
    simulation.aquifers["AQUIFER1"].update(hydraulic_conductivity=5.0)
    simulation.snowmelt_sets["SNOW1"].impervious.update(
        initial_snow_depth=0.2,
        initial_free_water=0.01,
        snow_depth_for_full_coverage=0.1,
    )
```

### Time settings

| Setting | Controls |
| --- | --- |
| `routing_step` | Solver routing policy |
| `report_step` | Saved report periods |
| `step_advance()` / `stride()` | When Python regains control; exact cadence can shorten the final routing step |

Use `step_advance(..., strict=False)` or `stride(..., strict=False)` when the numerical routing steps must
match repeated `step()` calls. Python then regains control at the first
unchanged routing step at or beyond the requested interval. `requested_threads`
is also a request; read `simulation.effective_threads` after startup for the
actual caller-inclusive team size.

### Units

| Property | Unit family |
| --- | --- |
| Node elevation and depth | Project length |
| Link flow and flow limit | Project flow |
| Subcatchment width and curb length | Project length |
| Subcatchment area | Project land area |
| Subcatchment slope | Fraction |
| Aquifer elevations and tension/evaporation depths | Project length |
| Aquifer conductivity and lower loss | Project rainfall rate |
| Snow initial, plow, and full-coverage depths | Project rain depth |
| Snow base temperature and melt coefficients | Project temperature/snowmelt units |

Public declarations remain in project units. Configuration Preparation performs
the single conversion to native solver units at Derived-State Commit; callers
do not convert SI values themselves.

## Declaration commit and effective preparation

An option or schedule update is atomic at the declaration boundary: either the
complete requested candidate is retained and the owner becomes dirty, or no
field changes. It does not mean that effective native solver fields change
immediately. Local type, unit, finiteness, representability, and intrinsic-domain
errors are rejected by the setter. Relational schedule/step errors are retained
so edits can be made in any order and are reported by `start()` as ordered
configuration diagnostics. A failed start leaves the prior effective native
state unchanged and keeps the requested declaration available for repair/retry.

After a successful start, Configuration Preparation has normalized zero-count
flags, Dynamic Wave defaults/conversions, effective threads, wet/dry/routing
steps, aquifer/snowmelt relationships and derived values, and all dependent
link/node/inlet geometry before the solver runs. Curves, time series, patterns,
land uses, pollutants, climate, DWF patterns, and report selectors remain
read-only definitions; a time-pattern can be selected by an aquifer but is not
mutated through this interface.

## Repair deferred configuration diagnostics

When calling `start()` explicitly, relational failures are returned together as
`ConfigurationError`. Tracebacks include concise ordered diagnostic lines,
while `.diagnostics` exposes typed
object identity, property path, rule code, message, and any conflicting object:

```python
from swmmrs import ConfigurationError

try:
    simulation.start()
except ConfigurationError as error:
    for diagnostic in error.diagnostics:
        print(
            diagnostic.object,
            diagnostic.property_path,
            diagnostic.rule_code,
            diagnostic.message,
        )
        if diagnostic.conflicting_object is not None:
            print("conflicts with", diagnostic.conflicting_object)

    groundwater = simulation.subcatchments["S1"].settings.groundwater
    if groundwater is not None:
        groundwater.surface_elevation = groundwater.water_table_elevation
    simulation.start()  # Retry after repairing retained requested intent.
```

A failed explicit `start()` leaves requested declarations authoritative,
`configuration_dirty` true, and the prior effective solver state unchanged. It
does not partially reset unrelated runtime, forcing, statistics, or continuation
state. Use explicit lifecycle calls for diagnostic repair/retry workflows.
Batch `execute()` instead surfaces deferred preparation rejection as
`ValidationError` and closes during its owned cleanup. See [Errors and
recovery](../python/concepts/errors-and-recovery.md).

## Configure link-attached inlets

Inlet placement is optional and link-local; there is no global
`simulation.inlets` collection. Reusable designs are available from
`simulation.inlet_designs`:

```python
link = simulation.links["C1"]
inlet = link.inlet
if inlet is not None:
    print(inlet.settings.design.id, inlet.settings.count)
    inlet.update(percent_clogged=15.0, flow_limit=2.0)
```

Placement settings are stable read-only configuration. `percent_clogged` and
`flow_limit` are persistent controls, while captured flow and backflow values
are runtime results. See [Inspect model definitions](inspect-model-definitions.md)
and the [link reference](../python/api/link.md).

## Reconfigure after a run

!!! warning
    A configuration write in `ENDED` moves the owner back to `OPEN` and resets completed-run clock and report-period bookkeeping.

Before reconfiguring:

1. copy final statistics;
2. call `end()`;
3. write the report if required;
4. apply the next candidate values.

Use one fresh `Simulation` per concurrent candidate or hotstart branch.

## Errors

| Error | Meaning |
| --- | --- |
| `ValidationError` | Invalid type, non-finite value, range, assignment, or path |
| `LifecycleError` | Valid operation in the wrong owner state |
| `ConfigurationError` | Deferred relational preparation failure with ordered, repairable diagnostics |
| `KeyError` / `IndexError` | Unknown ID or invalid configured index |
| `StaleViewError` | Collection or object belongs to an old project generation |

See the [node reference](../python/api/node.md), [link reference](../python/api/link.md), [subcatchment reference](../python/api/subcatchment.md), [LID reference](../python/api/lid.md), and [option reference](../python/api/options.md) for supported setters.
