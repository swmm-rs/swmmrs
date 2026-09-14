# Python user guide

Pick the job you need to do. These guides cover the Python side of `swmmrs`;
the EPA SWMM manuals still handle the hydrology and hydraulics. Learning a new
API is quite enough without asking you to relearn rainfall.

## Build and run

| Task | Guide | Covers |
| --- | --- | --- |
| Inspect configured definitions | [Inspect model definitions](../guides/inspect-model-definitions.md) | Collections, relationships, editable definitions, and attached inlets |
| Configure a project | [Configure a model](../guides/configure-model.md) | Options, dates, geometry, initial conditions, diagnostics, lifecycle, and units |
| Choose advancement ownership | [Choose a run workflow](workflows.md) | `execute()`, iteration, `step()`, and `stride()` |
| Understand owner state | [Lifecycle and ownership](concepts/lifecycle.md) | States, cleanup, project generations, and stale views |
| Run the solver | [Run a model](../guides/run-model.md) | Batch runs, callbacks, fixed cadences, termination, and concurrency |

## Observe results

| Task | Guide | Covers |
| --- | --- | --- |
| Read current objects | [Property getters and live views](concepts/property-access.md) | Fresh reads, retained views, validity, and acquisition cost |
| Choose a result shape | [Live views, snapshots, and statistics](concepts/results.md) | Scalar reads, coherent batches, and cumulative records |
| Build output datasets | [Collect results and statistics](../guides/collect-results.md) | Time series, native records, cumulative statistics, quality matrices, continuity, and retention |
| Query a `.out` file | [Read binary output](../guides/read-output.md) | Metadata, exact names, ranges, bulk series, I/O strategies, automatic incomplete-file recovery, and pandas conversion |
| Align time and units | [Units and time](concepts/units-and-time.md) | Routing, reporting, callback cadences, and project units |

## Control and compare

| Task | Guide | Covers |
| --- | --- | --- |
| Apply live inputs | [Runtime forcings and control](../guides/runtime-forcings.md) | Inflow, rain, stage, pollutants, settings, and persistence |
| Preserve full continuation or branch | [Simulation checkpoints and forks](../guides/checkpoints-and-forks.md) | Durable resume, in-process fork, State Load, artifacts, and limitations |
| Transfer EPA-compatible warm state | [EPA hotstarts and chained runs](../guides/hotstarts.md) | `.hsf` compatibility, warm-up, and independent owners |
| Automate analysis | [Recipes](../guides/recipes.md) | CSV export, concurrent scenarios, QA, and quality sampling |
| Handle failures | [Errors and recovery](concepts/errors-and-recovery.md) | State effects, diagnostics, retry, and cleanup |
| Move from PySWMM | [Migration guide](migrating-from-pyswmm.md) | Lifecycle, errors, outputs, and API differences |

## Five rules

These are worth knowing before a small script grows into an application:

1. **One owner, one generation.** Reopening invalidates old collections and views.
2. **One advancement owner.** Iterator-owned and caller-owned advancement do not mix.
3. **Live views do not cache.** Retain a view for fresh reads; retain a scalar or snapshot for history.
4. **Choose cadence semantics deliberately.** Exact `step_advance()` and `stride(..., strict=True)` cadence can shorten a routing step; `step_advance(..., strict=False)` and `stride(..., strict=False)` return at the first unchanged routing step at or beyond the target.
5. **Choose live or file-backed results deliberately.** Read live state and snapshots during routing; use `OutputReader` for immutable binary series after routing stops.

Use the [API reference](api/index.md) for exact classes, properties, values, and exceptions.
