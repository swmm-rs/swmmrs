# Migrating from PySWMM

Your PySWMM experience travels well, though changing the import is only the
first step. `swmmrs` borrows familiar workflows and embeds a different solver.
Ownership, lifecycle states, typed exceptions, and views tied to one project
generation need attention during migration. It is not a drop-in replacement.

## Common translations

| Task | PySWMM shape | `swmmrs` shape |
| --- | --- | --- |
| Open a model | `Simulation("model.inp")` | `Simulation("model.inp")` |
| Get a node | `Nodes(simulation)["J1"]` | `simulation.nodes["J1"]` |
| Get a link | `Links(simulation)["C1"]` | `simulation.links["C1"]` |
| Get a subcatchment | `Subcatchments(simulation)["S1"]` | `simulation.subcatchments["S1"]` |
| Iterate | `for step in simulation` | `for current_time in simulation` |
| Set callback cadence | `simulation.step_advance(300)` | `simulation.step_advance(300)` or a `timedelta` |
| Apply a link control | `link.target_setting = value` | `link.target_setting = value` while `RUNNING` |
| Post-run binary query | `Output(...)` | `OutputReader("model.out")` with typed family or bulk queries. |

Start with object access. The collections now belong to the simulation:

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]

    for current_time in simulation:
        if node.depth > 4.0:
            link.target_setting = 1.0

    simulation.end()
    simulation.report()
```

## Important differences

### Collections belong to the simulation

Do not construct separate `Nodes`, `Links`, or `Subcatchments` adapters. Access collections through the owning `Simulation`. IDs are case-insensitive, and collections preserve configured order.

### The loop body is the callback

Put inspection and controls directly in the iterator or caller-owned stepping
loop. There is no callback-registration system to wire up. `step_advance()`
controls how often that Python logic runs.

### Lifecycle failures are typed

Catch `ValidationError` for rejected Python inputs, `LifecycleError` for invalid operation order, and `SolverError` for native solver failures. A failed project generation should be closed, not restarted.

### Live views can become stale

A node, link, or collection remains valid across advances in one project generation. It raises `StaleViewError` after `close()` or after `open()` creates a replacement generation. Reacquire views from the current simulation.

### Output reading uses typed structured results

`OutputReader` opens a finalized or incomplete `.out` file independently of `Simulation`.
Family methods return immutable `OutputTimeSeries` values; bulk reads return a
shared nominal time axis plus ordered `OutputValueSeries` columns. Exact names
are case-sensitive, ranges are half-open, and pollutant columns use explicit
`PollutantAttribute` selectors. See [Read binary output](../guides/read-output.md).

## Migration checklist

1. Replace standalone collection constructors with `simulation.<collection>` access.
2. Choose one advancement owner: iterator, manual stepping, or `execute()`.
3. Move registered callbacks into the advancement loop.
4. Check units before assigning or combining external values.
5. Copy statistics before `end()` and snapshots before closing when later code needs them.
6. Replace PySWMM `Output` calls with typed family methods or `SeriesSelection` bulk queries, and choose `low_memory` deliberately.
7. Give every concurrent simulation distinct report and output paths.

PySWMM remains the mature choice when an application requires its callback ecosystem or broader compatibility surface. See the [PySWMM documentation](https://pyswmm.github.io/pyswmm/) for its authoritative API.
