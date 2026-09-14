# Isolated simulations and state

EPA SWMM was historically organized around one set of process-wide mutable
state. `swmmrs` instead places that state inside a `Simulation`. The important
modeling consequence is simple: **one simulation is one isolated working
project**.

## What the simulation contains

An opened simulation keeps several kinds of information together:

| State category | Examples | Modeling interpretation |
| --- | --- | --- |
| Requested configuration | Dates, options, node and link properties, aquifer and snowmelt declarations | The scenario the caller has asked to run. |
| Prepared configuration | Effective solver options, converted values, hydraulic geometry, dependent relationships | The solver-ready interpretation of the requested scenario. |
| Runtime state | Depths, flows, runoff, quality, controls, numerical history | The changing physical and numerical condition of this run. |
| Persistent forcing | API-supplied rain, inflow, stage, snow, and pollutant overrides | External conditions owned by this simulation and reapplied after solver initialization. |
| Results and accounting | Snapshots, statistics, continuity totals, report and binary-output progress | Products and records belonging to this run. |
| Continuation state | Checkpoint or hotstart configuration and pending imported conditions | Information used to begin or continue from a non-cold condition. |

These categories can coexist, but they are not interchangeable. Changing a
pipe declaration is not the same operation as changing a gate during routing.
Applying observed inflow is not the same as rewriting the input model. Loading
physical state is not the same as importing a previous run's dates, accounting,
and output history.

## Why isolation matters

Because mutable state is encapsulated:

- a baseline and an alternative can be held in memory at the same time;
- independent simulations can run concurrently without sharing hydraulic state;
- a scenario branch receives its own future state rather than a reference to
  its parent;
- lifecycle and unit checks can be applied consistently at one boundary;
- failed or closed simulations can be contained without invalidating unrelated
  simulations.

Isolation does not make output paths safe automatically. Every active owner must
have distinct report, binary-output, checkpoint, and other mutable destinations.
Two independent simulations writing the same file still conflict.

## Handles to the current simulation

Python collections and object views such as `simulation.nodes["J1"]` are
**live views**. They are convenient handles into the current project; they are
not copies of node or link records.

```python
node = simulation.nodes["J1"]

for current_time in simulation:
    print(current_time, node.depth)  # Reads the current state each time.
```

The view remains useful while the same opened project advances. It becomes
stale as soon as that simulation is closed. This prevents an old `"J1"` handle
from silently referring to a different `"J1"` if another project is later
opened. Reacquire live views from each newly opened project.

Snapshots, statistics records, and ordinary numbers already copied into Python
are different. They describe the acquisition moment and remain usable after
advancement or closure.

| Value | Changes as the solver advances? | Valid after close and reopen? |
| --- | --- | --- |
| Collection or live object view | Yes; properties read current state | No; reacquire it |
| Snapshot or statistics record | No; it is a detached record | Yes |
| Number, tuple, list, or dictionary already copied | No | Yes |

## Project generations

Each successful `open()` after `close()` begins a new **Project Generation**.
Think of a generation as one identity for an opened project, not as a hydraulic
time step. Live handles belong to that identity. This distinction matters in
long-running applications that reuse one Python `Simulation` variable for many
input files.

For most scenario studies, the least surprising pattern is one `Simulation`
object per model run or branch, with unique artifact paths and explicitly copied
results.

## Practical implications

1. Do not treat model objects as free-standing Python data classes; their live
   properties belong to a simulation.
2. Pass snapshots or copied values, not live views, across jobs or beyond a
   simulation's lifetime.
3. Use separate simulations for concurrent alternatives.
4. Distinguish declarations, forcing, controls, and results when deciding how a
   value should be changed.
5. Do not use `copy`, `deepcopy`, or pickling to duplicate a `Simulation`.
   Create another simulation, use `fork()`, or use a continuation artifact,
   depending on the modeling intent.
6. Close a failed generation rather than attempting to keep routing it.

Continue with [Declarations and preparation](declarations-and-preparation.md)
or see [Property getters and live views](../python/concepts/property-access.md)
for exact acquisition behavior.
