# EPA hotstarts and chained runs

An EPA hotstart file captures selected SWMM dynamic state in the interoperable
`.hsf` format so a compatible model can start warm instead of cold. Use one for
warm-up followed by forecasting or state transfer to another SWMM-compatible
run.

An EPA hotstart is not a results file and does not preserve a Python
`Simulation` owner, report/output history, persistent forcing policy, or the
fuller swmmrs runtime/resource continuation boundary. Start a new compatible owner, configure the
hotstart with `use_hotstart()`, and let `start()` load it. For fuller swmmrs
continuation, use [Simulation checkpoints and forks](checkpoints-and-forks.md).
Its [restoration matrix](checkpoints-and-forks.md#what-each-mechanism-restores)
compares EPA Hotstart, Enhanced Hotstart, and Checkpoint Resume subsystem by
subsystem. That guide also covers Enhanced Hotstart through
`load_checkpoint_state()`, which transfers more classified physical/numerical
state while retaining the receiver's dates, forcing, inputs, and fresh
accounting. EPA `.hsf` behavior is unchanged by that separate operation.

## Warm up, save a hotstart, then continue

Write a hotstart while the producer is `RUNNING` or `COMPLETE`. The method
waits until the file is complete and usable before returning.

```python
from datetime import timedelta
from pathlib import Path

from swmmrs import Simulation

hotstart = Path("warmup.hsf")

# Produce a warm-up hotstart. A scratch output is sufficient here.
with Simulation("model.inp", "warmup.rpt") as warmup:
    warmup.start(save_results=False)
    warmup.stride(timedelta(days=30))
    warmup.save_hotstart(hotstart)
    warmup.end()

# Continue from the hotstart in an independent model owner.
with Simulation("model.inp", "forecast.rpt", "forecast.out") as forecast:
    forecast.use_hotstart(hotstart)
    forecast.start(save_results=True)
    while forecast.stride(timedelta(minutes=15)) is not None:
        pass
    forecast.end()
    forecast.report()
```

`stride()` may return `None` if the producer reaches its configured end time;
that leaves it `COMPLETE`, where `save_hotstart()` remains valid. A hotstart is
most useful before the end time, when it represents a warm state that a later
run will extend.

## Lifecycle rules

| Operation | Valid states | Effect |
| --- | --- | --- |
| `use_hotstart(path)` | `OPEN`, `ENDED` | Configures an EPA hotstart to load before every later `start()` in this Project Generation. A write from `ENDED` returns it to `OPEN`. |
| `use_hotstart(None)` | `OPEN`, `ENDED` | Clears the configured hotstart; the next start is cold. |
| `save_hotstart(path)` | `RUNNING`, `COMPLETE` | Writes current state in EPA hotstart format. |
| `start()` | `OPEN`, `ENDED` | Loads the configured hotstart, if any, then initializes the run. |
| `close()` or a fresh `open(...)` | Any closeable state | Clears the generation's configured hotstart input. |

The input configuration persists across repeated starts of one project
generation. Clear it deliberately when comparing a cold run with a continued
run:

```python
simulation.use_hotstart(hotstart)
simulation.start(save_results=False)  # Starts from the EPA hotstart.
# ... run and end ...

simulation.use_hotstart(None)
simulation.start(save_results=False)  # Starts cold.
```

## Branch scenarios safely

Produce the hotstart once, then create one fresh `Simulation` per branch.
Each branch gets independent native state and artifacts while sharing the
read-only hotstart.

```python
from pathlib import Path

from swmmrs import Simulation


def run_branch(name: str, hotstart: Path, target_setting: float) -> Path:
    directory = Path("branches")
    directory.mkdir(exist_ok=True)
    report_path = directory / f"{name}.rpt"
    output_path = directory / f"{name}.out"

    with Simulation("model.inp", report_path, output_path) as simulation:
        simulation.use_hotstart(hotstart)
        simulation.start(save_results=True)
        simulation.links["GATE"].target_setting = target_setting
        while simulation.step() is not None:
            pass
        simulation.end()
        simulation.report()
    return output_path
```

A branch can run concurrently with other branches; give every owner unique
report and output paths. Do not share one `Simulation` or write the same
hotstart from multiple owners. The hotstart file is safe to reuse only once
its producer's `save_hotstart()` call has returned.

## Compatibility and artifact ownership

Treat a hotstart and its input file as a matched pair. Malformed or truncated
files and detectable object-count or flow-unit mismatches are rejected when
`start()` loads them. Object identities, ordering, subtypes, and complete
configuration compatibility are not verified; a mismatched model can silently
receive state for the wrong object. The caller must preserve the matched input
and hotstart pair. A detected loading failure raises `SolverError` and moves the
generation to `FAILED`; record the error and close it.

`use_hotstart()` and `save_hotstart()` reject byte paths and paths that collide
with the owner's input, report, or output artifact. A hotstart output also
cannot be the same path as the owner's configured hotstart input. Use a
different file for a new hotstart, then configure that file for a later owner.

Avoid reusing a hotstart path that the input model itself schedules through
its `[FILES]` hotstart directives. Keep host-managed hotstarts in a dedicated
directory with an explicit scenario or timestamp in the filename.

## Common patterns and gotchas

- **Warm-up then forecast:** finish a dry-weather or long-term stabilization
  period, save a hotstart, then start forecast owners from it.
- **Scenario tree:** create one baseline hotstart, branch with independent
  forcings or `target_setting` values, and compare host-collected results.
- **Resume after a planned boundary:** save after a routing callback, close the
  producer, and hand the hotstart to a later process or scheduled job.
- **Not crash recovery:** a hotstart only exists after `save_hotstart()` has
  completed. It does not preserve unsaved Python records, report output, or an
  in-progress call.
- **No hotstart after `end()`:** save before finalizing. If the run is
  complete, save in `COMPLETE`, then call `end()`.
- **Hotstart does not replace output:** use `save_results=True` and an explicit
  `output_path` when the continued run also needs a binary results artifact.

For the ordinary state machine and error model, see [Run a model](run-model.md).
For fuller runtime/resource continuation, see [Simulation checkpoints and forks](checkpoints-and-forks.md).
For live forcing and branch-control patterns, see [Runtime forcings](runtime-forcings.md).
