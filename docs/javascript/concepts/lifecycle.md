# Lifecycle and ownership

One `Simulation` owns one project, worker, solver state, and in-memory file
system. Open a separate owner for another model or scenario. Handles and
collections belong to the owner that created them.

## States and operations

| State | Meaning | Normal operations |
| --- | --- | --- |
| `open` | Input parsed and no run is active | Read or edit configuration, set persistent forcings, load checkpoint state, `start()`, iterate, or `run()` |
| `running` | A run is active | Read results and quality, read statistics, advance, apply controls, save hotstart/checkpoint, sleep workers, `end()`, or `finish()` |
| `complete` | Routing reached the configured end time | Read results, quality, and statistics, save hotstart/checkpoint, then `end()` or `finish()` |
| `ended` | Native processors ended and the owner remains open | Read final results and files, `finalizeReport()`, `report()`, edit configuration, set persistent forcings, start a fresh run, or `resetSolver()` |
| `failed` | Native execution entered an error state | Preserve the error and close the owner |
| `closed` | Worker, solver, and project files were released | Repeat `close()` or call `getState()` |

`close()` is idempotent and can be called from every closeable state. After
closing, collection access and solver operations reject. `getState()` still
returns `"closed"`.

## Advancement ownership

`steps()` creates an async generator but starts lazily on its first `next()`.
It starts an `open` or `ended` owner, continues a `running` owner, and returns
immediately for a `complete` owner. While an iterator is active, it owns
advancement. A second iterator, `start()`, `step()`, `stride()`, `run()`,
`end()`, `finalizeReport()`, `report()`, or `finish()` rejects with `LifecycleError`.

Result reads, status reads, and allowed control writes can run inside the loop.
Await each operation before requesting another advance. Natural exhaustion leaves
the owner `complete`; `break` releases the iterator without finalizing. If you
consume a generator manually and abandon it, call its `return()` method.

`terminate()` is valid only while an iterator owns advancement. It asks the
iterator to end at its next observation boundary. The request does not interrupt
a native call already in progress. The iterator then releases ownership and the
owner remains available for `finish()` (and `report()` / `finalizeReport()`), another run, or `close()`.

## Finalization and native stages

`finish()` performs the native end, flushes binary output, writes the requested
detailed report tables when the run started with `saveResults: true`, finalizes
the report, and returns `{ report, output }`. It can finalize a partial run and
is safe to repeat in `ended`.

`end()` performs only the native end and leaves the owner `ended`. Use
`report()` afterward when you need to invoke detailed report generation as a
separate stage. `report()` is valid in `ended` only and requires saved results.
`finalizeReport()` is a lighter separate stage that appends only the runtime
footer and is valid after `end()` regardless of whether detailed results were
requested. The report-selection flags on objects and the model's detailed-report
option control which tables are requested.

System `statistics()` is available in `running` and `complete`, not `ended`, so
capture it before finalization. Per-object statistics methods follow the same
active-run window. Result and quality records remain readable in `ended`.

`resetSolver()` is valid in `open` or `ended`. It clears stale run, result, and
output state without reparsing the retained declarations, then returns the owner
to `open`. An accepted configuration edit in `ended` also returns the owner to
`open` and invalidates the previous live run state. Detached records already
returned to the application remain usable.

`sleepWorkers()` is valid only in `running` and asks Dynamic Wave workers to enter
their sleep state. It does not end the run. `close()` releases resources but does
not provide hard cancellation for an in-flight native operation. It also removes
the worker's in-memory project files, so report text or output bytes must be
copied before cleanup. `finish()` does that automatically; `finalizeReport()`
exists for callers that need to flush the native close-time footer without yet
releasing those files.

## Reruns

After `finish()`, `start()` or iteration begins a fresh run at the configured model
start time. It does not continue from the previous endpoint. Configuration and
persistent forcings remain on the owner until changed. A new run replaces binary
output; report text can retain prior run content. Use a new owner for independently
named scenario artifacts.

## Hotstarts, checkpoints, and forks

The continuation mechanisms answer different questions:

| Need | Method | What it preserves |
| --- | --- | --- |
| Exchange selected physical state with an EPA-compatible run | `saveHotstart()` then `useHotstart(bytes)` | EPA `.hsf` warm state; the receiving run owns its dates, forcing, accounting, and files |
| Continue the same swmmrs run in another owner | `saveCheckpoint()` then `Simulation.resume(bundle)` | The saved run boundary, clocks, managed resource progress, configuration, forcing, accounting, and continuation state |
| Branch inside one process | `fork()` | An independent child at the source's quiescent boundary |
| Seed an already opened compatible model | `loadCheckpointState(bundle)` | Classified warm physical state; the receiver keeps its declarations, dates, forcing, accounting, and output history |

`saveHotstart()` is valid in `running` or `complete`; `useHotstart()` accepts
bytes or `null` in `open` or `ended` and stays selected for later starts until
cleared. It is a warm start, not a complete owner continuation.

A `CheckpointBundle` is one in-memory artifact. It contains a format/version,
worker-relative manifest path, canonical manifest bytes, and a map of sidecar and
validated dependency bytes. Keep all fields together. The native checkpoint JSON
alone is not a complete JavaScript checkpoint. `Simulation.resume(bundle)` opens
an independent owner from a saved `running` or `complete` boundary. `fork()`
provides the same independence without asking the application to publish a
bundle, but it is not a durable artifact.

`loadCheckpointState(bundle)` requires an already opened compatible receiver and
leaves it `open`. It imports classified warm state, then normal `start()` applies
that state after receiver initialization. The receiver keeps its own calendar,
declarations, persistent forcings, input progress, accounting, and output files.
State Load therefore does not promise the same next step or report history as
Checkpoint Resume.

Checkpoint save, resume, and fork operate between advances at a quiescent
`running` or `complete` boundary. See [hotstarts, checkpoints, and forks](../guides/checkpoints-and-forks.md)
for method signatures and compatibility details.
