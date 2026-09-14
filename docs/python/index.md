---
hide:
  - toc
---

# swmmrs for Python

Run SWMM from Python, inspect it as it goes, and change a control when the water
has other ideas. `swmmrs` provides typed access to a native Rust solver, with
each simulation responsible for its own state and files.

!!! warning "Early-stage"
    Install the published wheel, expect API changes, and validate critical model results against EPA SWMM. See [compatibility and limitations](compatibility.md).

!!! info "Solver source access"
    The native solver source is private. Access starts with a conversation about
    contributing; read the [governance and contribution model](../governance.md).

## Start here

1. [Install and run your first model](get-started.md).
2. [Choose a run workflow](workflows.md).
3. Use the [Python user guide](user-guide.md) for task-oriented workflows.
4. Use the [API reference](api/index.md) for exact types and signatures.

## Choose by workflow

| Goal | Guide |
| --- | --- |
| Discover configured definitions and relationships | [Inspect model definitions](../guides/inspect-model-definitions.md) |
| Configure options, dates, geometry, or initial conditions | [Configure a model](../guides/configure-model.md) |
| Run interactively or own the stepping loop | [Run a model](../guides/run-model.md) |
| Apply measured inflow, forecast rain, boundary stages, or controls | [Runtime forcings](../guides/runtime-forcings.md) |
| Collect live values, snapshots, quality, or final statistics | [Collect results and statistics](../guides/collect-results.md) |
| Resume or branch fuller runtime/resource continuation | [Simulation checkpoints and forks](../guides/checkpoints-and-forks.md) |
| Transfer EPA-compatible warm state | [EPA hotstarts and chained runs](../guides/hotstarts.md) |
| Diagnose and recover from failures | [Errors and recovery](concepts/errors-and-recovery.md) |
| Port an existing application | [Migrating from PySWMM](migrating-from-pyswmm.md) |

## Interactive run

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    node = simulation.nodes["J1"]
    gate = simulation.links["OR1"]
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        if node.depth > 2.0:
            gate.target_setting = 0.5
        print(current_time, node.depth, gate.flow)

    statistics = simulation.statistics
    simulation.end()
    simulation.report()
```

The loop handles the control decisions. Here is who handles the housekeeping:

- iteration owns advancement and finalizes the run;
- the context manager closes the project;
- live views read fresh solver values;
- `statistics` is an immutable copy that remains valid after closure.

## Core behavior

- **One owner, one project generation.** Reopening invalidates old collections and live views.
- **One advancement owner.** Do not mix iteration with manual `start()`, `step()`, `stride()`, or `execute()`.
- **Project units stay unchanged.** No automatic SI conversion.
- **Host cadence can affect routing cadence.** Exact `step_advance()` and `stride(..., strict=True)` boundaries can shorten a routing step; `step_advance(..., strict=False)` and `stride(..., strict=False)` preserve ordinary steps and may return after the target.
- **Live and file-backed results use different interfaces.** Collect Live Views and snapshots during routing; use [`OutputReader`](../guides/read-output.md) for report-period `.out` queries after routing stops.

## With gratitude to PySWMM

If this feels familiar, [PySWMM](https://github.com/pyswmm/pyswmm) deserves the
credit. It established a practical way to step through SWMM, inspect results,
and apply controls from Python. `swmmrs` borrows from that work deliberately.

You will recognize:

- the `Simulation` context manager and iterator;
- object collections and live node/link access;
- step-advance workflows;
- the shape of real-time control loops.

PySWMM remains the mature choice for its broader ecosystem, callbacks, and binary-output tools. See its [documentation](https://pyswmm.github.io/pyswmm/), [repository](https://github.com/pyswmm/pyswmm), and [JOSS paper](https://doi.org/10.21105/joss.02292).

!!! note "Independent project"
    `swmmrs` is not affiliated with or endorsed by PySWMM, and it is not a drop-in replacement. 
    It embeds a different solver implementation with different ownership, lifecycle, errors, and 
    current capabilities.
