# Lifecycle and ownership

A `Simulation` owns one isolated native SWMM project. Opening it, advancing it,
and closing it are separate jobs. Its state tells you which job is available
now, and which values you can read.

```text
OPEN -- start() or first iterator advance --> RUNNING
RUNNING -- manual or iterator advance reaches model end --> COMPLETE
COMPLETE -- end() -------------------------------------------> ENDED
ENDED -- reset or non-metadata stable/persistent write ------> OPEN
OPEN or ENDED -- start() -------------------------------------> RUNNING
OPEN -- retryable State Load startup failure ----------------> OPEN
Ordinary start/routing/finalization failure -----------------> FAILED
Any closeable state -- close() -------------------------------> CLOSED
CLOSED -- open() ---------------------------------------------> OPEN
```

Input validation and invalid lifecycle requests reject the operation without poisoning a healthy Project Generation. A poisoning operational `SolverError` can move the owner to `FAILED`; preserve that original error and close the failed generation.

## States

| State | Meaning | Normal next operations |
| --- | --- | --- |
| `OPEN` | Input is parsed; static configuration is mutable. | Configure, start, iterate, execute, or close. |
| `RUNNING` | Routing is active. | Advance, inspect, control, end, or close. |
| `COMPLETE` | Manual or iterator advancement reached the end; finalization has not run. | Copy final statistics, then end. |
| `ENDED` | Finalization is complete and the project remains open. | Report, restart, reconfigure, or close. |
| `FAILED` | A poisoning start, routing, finalization, report, or flush failure established an unusable generation. | Record the original error and close. |
| `CLOSED` | Native state and files are released. | Open a fresh project generation. |

## One advancement owner

Iteration owns advancement from its first `next()` until exhaustion or termination. Do not call `start()`, `step()`, `stride()`, or `execute()` during that interval.

If a loop may stop early, use `step()` or `stride()`. A `break` leaves that
iterator as the advancement owner; call `terminate()`, then advance the same
iterator once to observe `StopIteration` before switching workflows.

When `step()`, `stride()`, or the iterator reaches the model end, the model is
`COMPLETE`. This is your chance to copy final statistics before `end()` makes
statistics acquisition unavailable. Repeated `next()` stays exhausted until
an existing lifecycle reset. Asking again does not create more simulation.

`simulation.status` acquires state, timing, progress, counters, and warnings
together. Prefer it when values must describe the same solver instant.

## Project generations

Calling `open()` after `close()` creates a new project generation. Collections
and live object views remember which generation they belong to. Even if the
new model uses the same node names, the old views do not transfer:

```python
node = simulation.nodes["J1"]
simulation.close()
simulation.open("next.inp", "next.rpt")

# Reacquire from the new generation; `node` now raises StaleViewError.
node = simulation.nodes["J1"]
```

Snapshots and statistics already acquired are owned native value objects;
numbers, tuples, and dictionaries already copied into Python are host-owned
values. All remain usable after closing, while collections and Live Views do
not.

## Checkpoint boundaries

After a manual `step()` or `stride()`, a running or complete owner is at a
quiescent checkpoint boundary. `save_checkpoint()` publishes its complete
runtime/resource continuation state. `Simulation.resume()` and `fork()` return
independent owners with fresh report and binary-output paths;
`load_checkpoint_state()` performs Enhanced Hotstart into a compatible `OPEN`
receiver. It imports classified warm physical/numerical state while keeping the
receiver's dates, forcing, inputs, declarations, fresh accounting, and output
history. It does not continue source RDII/input progress or promise the same
next step as Checkpoint Resume. A preparation-dependent edit made after State Load is accepted but makes the pending warm state stale; repeat `load_checkpoint_state()` before calling `start()`.

Iteration continues to own advancement, but `save_checkpoint()` and `fork()` are permitted between advances. The owner lock serializes either operation after an active `next()` finishes, so each captures a quiescent boundary. State Load and lifecycle-changing operations remain invalid while iteration owns advancement.

Durable checkpoints restore editable declarations, so a resumed owner can be
ended, edited, prepared, and rerun.
See [Simulation checkpoints and forks](../../guides/checkpoints-and-forks.md).

## Context managers

Use a context manager for interactive runs. It closes the project and ends an active run if the body raises, but it does not generate the detailed report:

```python
with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    for _ in simulation:
        pass
    simulation.end()
    simulation.report()
```

The native owner performs lifecycle guards, cadence and exhaustion tracking,
path/checkpoint metadata retention, collision checks, and cleanup sequencing.
Python retains `Path`, `datetime`, and `timedelta` presentation plus context-body
exception precedence: a cleanup failure is raised when there is no body error,
or attached as a note without replacing the body error.

See [Choose a run workflow](../workflows.md) for the shortest lifecycle for each use case and [Errors and recovery](errors-and-recovery.md) for state effects.
