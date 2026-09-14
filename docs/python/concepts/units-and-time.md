# Units and time

The input file chooses the units, and Python keeps them. `swmmrs` does not
silently convert project values to SI. A float offers no clue whether it means
feet or metres, so check before combining it with external data.

## Identify project units

```python
print(simulation.unit_system)
print(simulation.flow_units)
```

`unit_system` is `UnitSystem.US` or `UnitSystem.SI`. `flow_units` identifies the configured flow code, such as CFS, GPM, CMS, or LPS.

| Value family | Units |
| --- | --- |
| Node and link depth, head, and dimensions | Configured project length units. |
| Flow, inflow, runoff, and link flow | Configured project flow units. |
| Rainfall, evaporation, infiltration, and snow depth | SWMM rainfall or report rain-depth units for the project. |
| Volume and area | Project volume and area units. |
| Pollutant concentrations and loads | Reporting units configured for each pollutant. |
| Durations exposed as Python values | `datetime.timedelta` unless the property documents a scalar rate or count. |

Keep unit labels with exported data, especially when comparing models with
different `FLOW_UNITS` settings. Future you should not have to reconstruct the
units from the size of a suspicious number.

## Simulation dates

Simulation date properties use timezone-naive `datetime` values with whole-second precision:

```python
from datetime import timedelta

simulation.report_start = simulation.start_time + timedelta(hours=1)
simulation.end_time = simulation.start_time + timedelta(hours=6)
```

Timezone-aware values and fractional seconds are rejected with `ValidationError`. Convert external timestamps deliberately before assigning them.

## Three independent time scales

The solver, the output file, and your Python loop each have a schedule. They
need not tick together:

| Time scale | Controls |
| --- | --- |
| Routing step | How SWMM advances its numerical solution. Dynamic Wave routing can vary this step. |
| Report step | How often SWMM stores report-period results in the binary output. |
| Python callback cadence | How often iteration or `stride()` returns control to Python. |

By default, `step_advance()` and `stride()` stop exactly at each positive
whole-second cadence boundary. SWMM can shorten the final routing step to hit
that boundary, which can change the numerical solution.

Both methods default to `strict=True`. `step_advance(..., strict=False)` or
manual `stride(..., strict=False)` instead advances only through
ordinary `step()` calls until model time reaches or passes the target. This
preserves the routing timestep and step-by-step numerical solution, but the
returned timestamp can be later than the target and need not be a cadence
multiple.

The timestamp and live values observed in a loop body describe the state after
the preceding routing advance. A control written in that body affects a later
routing operation.

## External data

Before applying telemetry or forecasts:

1. Inspect `unit_system` and `flow_units`.
2. Convert the external value into the model's configured units.
3. Align timestamps to the chosen callback cadence.
4. Define missing-data behavior in the host application.

See [Runtime forcings](../../guides/runtime-forcings.md) for persistent input semantics and [Collect results](../../guides/collect-results.md) for result-field units.
