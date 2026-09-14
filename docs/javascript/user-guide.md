# User guide

Start with [your first run](get-started.md) if you have not built the package.
These guides assume an application that can load the worker and WASM assets, or
Node.js 22 or later for `worker_threads` execution.

## Build and run

| Task | Guide |
| --- | --- |
| Open a model, inspect IDs, and edit declarations | [Inspect and configure a model](guides/configure-model.md) |
| Advance, pause, resume, and finalize | [Run a model](guides/run-model.md) |
| Configure browser hosting and UI progress | [Integrate with a frontend](guides/frontend.md) |
| Choose batch, interactive, manual, or scenario workflows | [Choose a run workflow](workflows.md) |

## Observe and control

| Task | Guide |
| --- | --- |
| Sample hydraulic, runoff, and quality values | [Collect results, quality, and statistics](guides/collect-results.md) |
| Read per-object and system statistics | [Collect results, quality, and statistics](guides/collect-results.md) |
| Supply uploads or Node byte inputs and save files | [Load files and download output](guides/files.md) |
| Query finalized or incomplete binary output | [Read binary output](guides/read-output.md) |
| Apply hydraulic, rainfall, pollutant, or catchment controls | [Runtime forcings](guides/runtime-forcings.md) |
| Continue or branch a run | [Hotstarts, checkpoints, and forks](guides/checkpoints-and-forks.md) |

## Understand the API

- [Lifecycle and ownership](concepts/lifecycle.md): valid states, finalization,
  cooperative termination, reset, sleep, and continuation mechanisms.
- [Handles, records, and snapshots](concepts/results.md): what stays owner-bound
  and what is copied.
- [Units and time](concepts/units-and-time.md): project units, calendar strings,
  seconds, and cadence.
- [Errors and recovery](concepts/errors-and-recovery.md): validation, solver
  failures, and cleanup.

For signatures and every exposed record field, use the [API reference](api/index.md).
