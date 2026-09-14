# Choose a run workflow

Decide who advances the simulation and how often Python needs a turn. All
advancement styles use the same SWMM calculations, but each run needs one
driver. Letting an iterator and a manual loop both take the controls is where
the paperwork starts.

| Need | Use | Lifecycle owner |
| --- | --- | --- |
| Run to completion without inspecting state | `execute()` | `execute()` starts, ends, reports, and closes. |
| Inspect or control every routing advance | `for current_time in simulation` | The iterator starts; the caller ends; the context manager closes. |
| Inspect or control at a fixed host cadence | `step_advance(...)` plus iteration | The iterator starts; the caller ends; the context manager closes. |
| Integrate with an application-owned event loop | `start()` plus `step()` | The caller starts, ends, and closes. |
| Use an application-owned loop at a coarser cadence | `start()` plus `stride(...)` | The caller starts, ends, and closes. |

## Batch execution

```python
from swmmrs import Simulation

Simulation("model.inp", "model.rpt", "model.out").execute()
```

Use this when Python has nothing to do during routing. `execute()` takes care
of the run and cleanup.

## Iterator-owned execution

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    simulation.step_advance(timedelta(minutes=5))
    for current_time in simulation:
        inspect(current_time, simulation.nodes["J1"].depth)
    simulation.end()
    simulation.report()
```

The first iterator advance starts the run. When the loop finishes naturally,
the model is `COMPLETE`. Copy any final statistics you need, then call `end()`
before the explicit `report()`.

## Caller-owned execution

```python
from swmmrs import Simulation

simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    simulation.start()
    while (current_time := simulation.step()) is not None:
        inspect(current_time)
    simulation.end()
    simulation.report()
finally:
    simulation.close()
```

Manual `step()` or `stride()` returns `None` when the model finishes. The model
is then `COMPLETE`. Call `end()` before reporting or restarting.

## Rules that prevent lifecycle bugs

- Do not mix iterator advancement with `start()`, `step()`, `stride()`, or `execute()`.
- Test a manual step result with `is not None`; a timestamp is not a completion flag.
- Use `save_results=False` only when live values and statistics are sufficient. That run cannot generate a detailed report.
- Give concurrent simulations distinct report and output paths.

See [Run a model](../guides/run-model.md) for every lifecycle state and [Lifecycle and ownership](concepts/lifecycle.md) for view validity and project generations.
