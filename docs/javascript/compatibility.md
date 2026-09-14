# Compatibility and limitations

## Runtime and distribution

`@swmmrs/swmmrs` is a private package built from `js/` in this repository. It
runs in a dedicated module worker in a browser and in `node:worker_threads` on
Node.js 22 or later. Node callers do not need browser globals or cross-origin
isolation. Node defaults to `threads: 1`; an explicit count greater than one is
supported when it fits the host capacity.

Browser serial execution with `threads: 1` uses unshared WASM memory and does
not need `SharedArrayBuffer` or isolation headers. Browser execution with more
than one thread requires `globalThis.crossOriginIsolated === true`,
`SharedArrayBuffer`, and the usual COOP/COEP response headers. The browser
integration checks use Chromium. They do not qualify every browser, bundler, or
asset-hosting setup.

All model and output inputs are bytes or text contents. The runner does not
fetch URLs or read host paths. In Node, use `fs.readFile()` and pass its
`Uint8Array` result. Supporting-file keys remain relative paths from the INP.

## API coverage

| Area | Exposed today | Boundary to keep in mind |
| --- | --- | --- |
| Execution | Batch runs, routing steps, strict strides, async iteration, status, `end()`, `finalizeReport()`, `report()`, `resetSolver()`, `sleepWorkers()`, and iterator `terminate()` | Batch runs have no progress callback, `AbortSignal`, or hard interrupt of native work |
| Objects | Node, link, subcatchment, and rain-gage collections; kind-tagged node/link configuration; hydraulic, runoff, quality, and statistics reads | Handles come from collections. There are no public subtype constructors or object create/delete methods |
| Configuration | Sparse model and object patches, node/link subtype and cross-section records, infiltration, groundwater, snowmelt, pollutant loading, coverage, AMM, RTK, and LID configuration | Object IDs and collection membership are fixed for an opened owner. Time-series definitions are identity handles, not editable records |
| Specialized families | Aquifers, `SnowmeltParameterSet`, `AmmModel`, `UnitHydrograph`, `LidControl`, and indexed `LidUnit` handles | Relationship fields use canonical string IDs. They are not handles owned by another collection |
| Quality and statistics | Per-object quality reads, family quality snapshots, per-object statistics, family statistics snapshots, and system continuity totals | Quality and result reads need a started run. Statistics reads are available while `running` or `complete`, before `finish()` |
| Files | Staged input dependencies, finalized report text, binary output bytes, `readFile()`, and standalone `OutputReader` queries | `OutputReader` also accepts valid incomplete output, owns a worker, and must be closed when finished |
| Scenarios | EPA hotstarts, `CheckpointBundle` save/resume, in-process `fork()`, and checkpoint State Load | A checkpoint is a manifest plus sidecar and dependency bytes. The native checkpoint JSON alone is not a complete JavaScript bundle |

An INP can still contain solver features that an object API does not edit. For
example, a time-series definition can be present in the model while
`simulation.timeSeries` exposes only its identity and ID. Change structure or
definitions that the object API does not edit in the INP and open a new owner. For
definitions and scenario rules, see [specialized families](api/collections.md#specialized-families)
and the [scenario reference](api/simulation.md#hotstarts-checkpoints-and-forks).

## Operational limits

Each owner has its own worker and in-memory project files. Threaded owners also
allocate a solver worker pool. Input staging, snapshots, retained sample
histories, checkpoint sidecars, and downloaded output all consume memory. Use
explicit thread budgets when running several owners and release owners with
`close()`.

`run()` and `runSwmm()` occupy their worker until the native batch call returns.
`close()` queues cleanup but does not interrupt an in-flight native call. For
cooperative progress or stopping, advance with `steps()` and call
`terminate()` from the controlling task. The iterator then ends at its next
observation boundary.

`finish()` ends the active run, flushes binary output, writes the requested
detailed report tables when `saveResults` is enabled, and finalizes report text.
`saveResults` defaults to `true` on `start()`, `run()`, and `steps()`. With
`saveResults: false`, `RunResults.output` is an empty `Uint8Array`; use live
reads and snapshots instead of expecting report-period binary data.

`finalizeReport()` is a lighter post-stage when you only need the footer,
while `report()` writes the requested detailed tables and finalizes report
text. `report()` requires detailed output to be enabled. `finalizeReport()` can
still be used after `end()` with `saveResults: false`.

For numerical evidence, see the project's [regression evidence](../regression-validation.md).
That evidence does not qualify every browser, model, or deployment configuration.
