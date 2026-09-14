# Run a model

`Simulation` owns one isolated SWMM project and its files. Prefer a context
manager for every interactive or stepped run: it ends an active run and closes
its project even when the body raises. Pass explicit report and output paths for
batch jobs so every artifact has one unambiguous owner.

```python
from pathlib import Path

from swmmrs import Simulation

input_path = Path("model.inp")
report_path = Path("model.rpt")
output_path = Path("model.out")

with Simulation(input_path, report_path, output_path) as simulation:
    for current_time in simulation:
        # Inspect or change supported live model state here.
        print(current_time, simulation.percent_complete)

    # Natural iterator completion leaves the run COMPLETE.
    simulation.end()
    simulation.report()

assert report_path.is_file()
assert output_path.is_file()
```

Construction opens and validates the project; entering `with` does **not**
start it. The first iteration step starts the run with `save_results=True`.
When the iterator reaches the model end, it leaves the run in `COMPLETE` so
final statistics remain available. Call `end()` before [`report()`][report].
Context exit finalizes an active or complete run, closes the project, and
releases its files; it does not call [`report()`][report] for you.


[report]: ../python/api/simulation.md#swmmrs.Simulation.report "Write saved report-period results to the detailed report."

## Simulation lifecycle

Use `simulation.state` when manual code needs to decide what to do. The
states, their transitions, and their useful operations are:

| State | Reached by | Meaning | Next operation |
| --- | --- | --- | --- |
| `OPEN` | `Simulation(...)` succeeds, or `open(...)` succeeds after `CLOSED` | Input is parsed and configuration is mutable; no routing has run. | Configure, `start()`, iterate, `execute()`, or `close()`. |
| `RUNNING` | `start()`, or the first `next(simulation)` / iteration step | SWMM is initialized and routing can advance. | `step()`, `stride()`, iteration, `sleep_workers()`, `end()`, or `close()`. |
| `COMPLETE` | Manual or iterator advancement reaches natural completion | Routing has reached the configured end time, but SWMM is not finalized. | Read final statistics, then call `end()` or `close()`. Repeated advancement remains exhausted. |
| `ENDED` | `end()` succeeds, or a termination request finalizes the active iterator | Run finalization is complete and the project remains open. | `report()` if results were saved, configure an allowed option, restart with `start()` or `execute()`, or `close()`. |
| `FAILED` | `start()`, `step()`, `stride()`, `end()`, or `report()` gets a solver error | The failed generation preserves its original `SolverError`. | Record the error, then `close()`. Do not reuse this generation. |
| `CLOSED` | `close()`, context exit, or successful/failed accepted `execute()` cleanup | Project files and native state are released. Views from that project are stale. | `open(...)` creates a fresh Project Generation. |

`close()` and `end()` are idempotent where their state permits them. A fresh
`open(...)` is valid only from `CLOSED`; it replaces the Project Generation, so
collections and node/link views obtained from the prior generation must not be
retained.

Iteration has one special ownership rule: once an iterator has started,
iteration exclusively owns advancement. Do not mix that iterator with
`step()`, `stride()`, `start()`, or `execute()`. For early iterator
termination, call `terminate()` from another thread or between callbacks; the
iterator stops and ends at its next checkpoint.

## Choose an advancement loop

All advancing forms run the same SWMM processors, but an exact host cadence can
shorten the routing step at a cadence boundary. Choose by lifecycle ownership
and the numerical behavior you need:

| Form | Starts / ends the run | Python callback cadence | Use when |
| --- | --- | --- | --- |
| `for current_time in simulation` | Iterator starts on first advance; caller ends after exhaustion. | Every routing advance. | The default interactive loop. |
| `simulation.step_advance(...); for current_time in simulation` | Iterator starts; caller ends after exhaustion. | Exact requested positive whole-second cadence by default; `strict=False` preserves ordinary routing steps and may return after the target. | Python needs regular, coarser callbacks. |
| `while (current_time := simulation.step()) is not None` | Caller must call `start()` and `end()`. | Every routing advance. | The host owns setup, cleanup, or its stepping loop. |
| `while (current_time := simulation.stride(...)) is not None` | Caller must call `start()` and `end()`. | Exact requested cadence with the default `strict=True`, or the first ordinary routing step at or beyond it with `strict=False`. | The host owns lifecycle and needs coarser callbacks. |
| `simulation.execute()` | `execute()` starts, ends, reports when results are saved, and closes. | None. | Batch runs with no intermediate work. |

Use a `for` loop only with the iterator. Its iterator owns advancement, so do
not call `start()`, `step()`, `stride()`, or `execute()` while it is active.
Use a `while` loop with `step()` or `stride()` only after an explicit `start()`;
`None` means the model is complete and must be followed by `end()`. Always test
`is not None` rather than truthiness.

```python
from datetime import timedelta
from swmmrs import Simulation

# Iterator-owned: simplest per-routing-advance callback.
with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    for current_time in simulation:
        inspect(current_time)
    simulation.end()
    simulation.report()

# Iterator-owned: callbacks every 15 simulated minutes.
with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    simulation.step_advance(timedelta(minutes=15))
    for current_time in simulation:
        inspect(current_time)
    simulation.end()
    simulation.report()

# Caller-owned: one callback per routing advance.
simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    simulation.start()
    while (current_time := simulation.step()) is not None:
        inspect(current_time)
    simulation.end()
finally:
    simulation.close()

# Caller-owned: callbacks at or after each 15-minute target without shortening
# SWMM's routing timestep.
simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    simulation.start()
    while (
        current_time := simulation.stride(
            timedelta(minutes=15),
            strict=False,
        )
    ) is not None:
        inspect(current_time)
    simulation.end()
finally:
    simulation.close()
```

## Manage the states yourself

Use explicit lifecycle calls when the host application controls each routing
step. The `try`/`finally` is the non-context-manager equivalent of `with`.

```python
from swmmrs import Simulation

simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    # save_results indicates to save data to binary output file
    simulation.start(save_results=True)  # OPEN -> RUNNING,

    while (current_time := simulation.step()) is not None:
        update_dashboard(current_time, simulation.nodes["J1"].depth)

    # step() returning None leaves a manually advanced model COMPLETE.
    simulation.end()  # COMPLETE -> ENDED
    simulation.report()  # writes timeseries data to model.rpt
finally:
    simulation.close()  # safe after an error or a normal run
```



`start(save_results=False)` avoids retaining binary report-period results.
Use it for live-only work, but then `report_period_count` remains zero and
`report()` is invalid. The report path still belongs to the simulation, so use
distinct artifact paths even for a live-only run.

Use `strict=False` when callback timing must not alter the model's
routing calculation:

```python
from datetime import timedelta

simulation.start()
while (
    current_time := simulation.stride(
        timedelta(minutes=15),
        strict=False,
    )
) is not None:
    publish(current_time, simulation.percent_complete)
simulation.end()
simulation.report()
```

This mode repeatedly uses the ordinary `step()` path until model time reaches
or passes the requested interval. Returned timestamps can therefore be later
than the target and need not be multiples of 15 minutes. With the default
`strict=True`, `stride()` stops exactly at the target and can shorten the final
routing step.

`step_advance(timedelta(minutes=15))` applies the exact-target behavior to
ordinary iteration, with the same keyword-only `strict=True` default as `stride()`.
Use `step_advance(timedelta(minutes=15), strict=False)` for timestep-preserving
iteration instead. `step_advance(None)` restores ordinary routing-step iteration;
`strict` has no effect when the duration is `None`.
`stride()` and `step_advance()` accept only positive whole seconds.
`step_advance(300)` is also valid and requests a cadence of 300 seconds.
## Run without iterating

For a non-interactive batch run, `execute()` performs
`start()` → routing to completion → `end()` → `report()` 
:material-alert-circle-outline:{ title="when results are saved and output is scratch" }
 → `close()`. It is the shortest correct lifecycle when no intermediate state is needed.




```python
from swmmrs import Simulation, SimulationState

simulation = Simulation("model.inp", "model.rpt", "model.out")
simulation.execute()

assert simulation.state is SimulationState.CLOSED
```

`execute(save_results=False)` still starts, runs, ends, and closes, but skips
the detailed report and report-period output. `execute()` closes its project
itself. A context manager is still the default for code that may switch to
inspection or manual stepping; do not call `execute()` on a `RUNNING` model or
while its iterator owns advancement.

## Let Dynamic Wave workers sleep between callbacks

swmmrs gives users explicit control over Dynamic Wave worker parking between
routing callbacks. For callbacks that perform slower external work,
`sleep_workers()` provides an optional power-saving hint that parks active
worker threads. The next `step()`, `stride()`, or iterator advancement wakes
them for its next routing operation. Calling it does not advance model time or
change results, and it is unnecessary for normal tight loops.

Use it when a callback does substantial non-SWMM work, such as waiting for a
GUI, network I/O, or a slow external controller:

```python
from swmmrs import Simulation

with Simulation("threaded-model.inp", "model.rpt", "model.out") as simulation:
    simulation.start()
    while (current_time := simulation.step()) is not None:
        simulation.sleep_workers()
        wait_for_external_controller(current_time)

    simulation.end()
    simulation.report()
```

Call it only in `RUNNING`. Calling it before `start()`, after completion, or
after `end()` raises `LifecycleError`. It is harmless for configurations with
no active Dynamic Wave worker pool, and it is not a replacement for selecting
an appropriate `THREADS` value in the input model.

## Run independent simulations concurrently

Each `Simulation` has its own native owner and can run concurrently with other
owners. Keep one owner per task and give every run a distinct report and output
path. Operations on the **same** `Simulation` serialize; never use multiple
threads to advance one model.

```python
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from swmmrs import Simulation


def run_one(input_path: Path, run_directory: Path) -> Path:
    report_path = run_directory / f"{input_path.stem}.rpt"
    output_path = run_directory / f"{input_path.stem}.out"
    with Simulation(input_path, report_path, output_path) as simulation:
        simulation.execute()
    return output_path


models = [Path("scenario-a.inp"), Path("scenario-b.inp")]
with ThreadPoolExecutor(max_workers=len(models)) as executor:
    outputs = list(executor.map(lambda path: run_one(path, Path("runs")), models))
```

The binding releases Python while a native lifecycle operation runs, so separate
owners can overlap. Dynamic Wave threads are also per simulation. Budget the
total requested solver threads across concurrent runs (e.g., two models with
`THREADS 4` need up to eight caller-inclusive solver threads) rather than
oversubscribing the machine.

## Errors and common gotchas

Catch `SolverError` for an input, solver, or output failure. Its `code`,
`operation`, and optional `detail` identify the rejected operation.

```python
from swmmrs import Simulation, SolverError

try:
    with Simulation("model.inp", "model.rpt", "model.out") as simulation:
        simulation.execute()
except SolverError as error:
    print(error.code, error.operation, error.detail)
    raise
```

`ValidationError` means the Python arguments are invalid; `LifecycleError`
means the arguments may be valid but the operation is wrong for the current
state. Common invalid arguments and sequences:

| Input | Rejected case |
| --- | --- |
| `input_path`, `report_path`, `output_path` | Must be `str` or `os.PathLike[str]`; byte paths are rejected. Relative paths are based on the current working directory. |
| Project artifact paths | The native owner resolves call-time relative paths, retains successful-open metadata, and requires input, report, output, checkpoint, and configured hotstart destinations to be distinct where applicable. A missing `output_path` uses scratch output, so `simulation.output_path` is `None`. |
| `save_results` | Must be exactly `True` or `False`; `1` is not accepted. |
| `stride()` / `step_advance()` | Require a `timedelta` or numeric positive whole seconds, no more than the native signed 32-bit maximum. `True`, fractions, zero, negatives, NaN, and infinity are invalid. Both methods require an exact `bool` for `strict` (default `True`). `step_advance(None)` clears the cadence. |
| Time option setters | Require timezone-naive `datetime` values with whole-second resolution. |
| `step()` / `stride()` | Invalid before `start()`, after `end()`, or while an iterator owns advancement. |
| `report()` | Valid only after `end()` and only when that run used `save_results=True`. |
| `open(...)` | Valid only after `close()`; it creates a new generation. |

Finally, `Simulation` cannot be copied, deep-copied, or pickled. Retaining a
node, link, collection, or other live view after `close()` or a replacement
`open()` is also invalid: reacquire it from the current simulation generation.
