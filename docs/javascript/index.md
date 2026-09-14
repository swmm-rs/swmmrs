---
hide:
  - toc
---

# JavaScript / TypeScript

`@swmmrs/swmmrs` runs SWMM in a dedicated worker from a browser or from
Node.js 22 and later. Each `Simulation` owns its worker, solver state, and
in-memory project files. Solver calls are asynchronous; collection metadata is
local and synchronous.

The package is private and is built from `js/` in this repository. The browser
build has serial and threaded WASM modes. Node uses `worker_threads`, defaults to
one solver thread, and can use explicit thread counts without browser isolation.
In a browser, `threads: 1` needs no cross-origin isolation. Browser runs with
multiple threads require `crossOriginIsolated` and `SharedArrayBuffer`, normally
provided by COOP/COEP headers. The tested browser setup does not qualify every
browser or bundler combination.

## Start here

- [Build and run your first model](get-started.md).
- [Choose a run workflow](workflows.md) for batch execution, progress, or controls.
- [Follow the user guide](user-guide.md) for configuration and result collection.
- [Browse the API reference](api/index.md) for signatures and field definitions.
- [Move from Python](migrating-from-python.md) for translation rules.
- Check [compatibility and limitations](compatibility.md) before deployment.

## Choose an entry point

| Need | Entry point |
| --- | --- |
| Run once and receive report text and binary output | `runSwmm(input, files, options)` |
| Configure a model, then run it in one request | `Simulation.open()` followed by `run()` |
| Observe progress, read results, or apply controls | `simulation.steps(...)` or `for await` |
| Own every routing advance | `start()`, then `step()` or `stride()` |
| Query finalized or incomplete binary output | `OutputReader.open(outputBytes)` |

`input` and supporting-file values are contents, not filenames or URLs. Accepted
values are strings, `Blob`, `ArrayBuffer`, and array-buffer views. In Node, pass
`Uint8Array` values returned by `fs.readFile`; the runner does not read host
paths for you. See [file handling](guides/files.md).

## Important behavior

- Handles and collections are tied to one owner. Lookups use canonical IDs and
  do not return foreign owner objects.
- `configuration()` and `results()` return detached, frozen records. Call the
  asynchronous method again after an advance; do not expect Python-style live
  scalar properties.
- `steps()` and `stride()` default to `strict: true`. Pass whole seconds, not a
  Python `timedelta`.
- `start()`, `run()`, and `steps()` default to `saveResults: true`. With
  `saveResults: false`, report-period binary output is not written and the
  returned `RunResults.output` is empty. `finish()` writes the requested detailed
  report tables and finalizes the report and binary output when results are
  enabled.
- `end()`, `finalizeReport()`, `report()`, and `resetSolver()` expose the native lifecycle stages.
  `sleepWorkers()` puts active Dynamic Wave workers to sleep. `terminate()` ends
  an active iterator at its next observation boundary.
- `run()` and `runSwmm()` are single worker requests. They do not provide a
  progress callback, `AbortSignal`, or hard interruption of native batch work.
  Use `steps()` for cooperative progress and stopping.

The public API includes hydraulic and runoff results, water-quality reads,
per-object and family statistics, model options, kind-tagged node and link
configuration, specialized definitions, AMM, RTK, LID, EPA hotstarts, swmmrs
checkpoint bundles, and the standalone [OutputReader](api/output.md).
See [specialized families](api/collections.md#specialized-families) and the
[scenario methods](api/simulation.md#hotstarts-checkpoints-and-forks) for the
full list and their lifecycle rules.
