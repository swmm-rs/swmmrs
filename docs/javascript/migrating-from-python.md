# Moving from Python

The JavaScript API keeps Python's object families but crosses a worker boundary.
Methods that read or change solver state return promises. Configuration and result
records are detached snapshots, not live scalar properties.

## Common operations

| Python | JavaScript / TypeScript |
| --- | --- |
| `Simulation("model.inp", ...)` | `await Simulation.open(inputContents, files, options)` |
| `simulation.nodes["J1"]` | `simulation.nodes.get("J1")` |
| `node.depth` | `(await node.results()).depth` |
| `node.full_depth = value` | `await node.configure({ fullDepth: value })` |
| `node.external_inflow = flow` | `await node.setExternalInflow(flow)` |
| `link.target_setting = value` | `await link.setTargetSetting(value)` |
| `simulation.nodes.snapshot(ids)` | `await simulation.nodes.snapshot(ids)` |
| `for time in simulation` | `for await (const time of simulation)` |
| `simulation.step_advance(timedelta(seconds=60))` | `simulation.steps({ seconds: 60, strict: true })` |
| `simulation.stride(duration)` | `await simulation.stride(60)` |
| `simulation.statistics` | `await simulation.statistics()` |
| `simulation.start(save_results=False)` | `await simulation.start({ saveResults: false })` |
| `simulation.end(); simulation.report()` | `await simulation.end(); await simulation.report()` |
| no direct Python equivalent (run summary only) | `await simulation.end(); await simulation.finalizeReport()` |
| `simulation.reset_solver()` | `await simulation.resetSolver()` |
| `simulation.sleep_workers()` | `await simulation.sleepWorkers()` |
| `simulation.terminate()` | `simulation.terminate()` while an iterator is active |
| `with Simulation(...)` | `try/finally` with `await simulation.close()` |

`steps()` and `stride()` default to `strict: true`, which requests exact
observation boundaries. Pass a positive whole number of seconds. JavaScript does
not accept Python `timedelta` objects. Pass `strict: false` only when preserving
whole routing-step boundaries is more important than exact observation times.

## Pass contents, not paths

`FileContents` accepts a string, `Blob`, `ArrayBuffer`, or array-buffer view.
Supporting-file keys are relative paths from the INP. In Node.js 22 or later,
read files with `node:fs/promises` and pass the resulting bytes:

```javascript
import { readFile } from "node:fs/promises";
import { runSwmm } from "@swmmrs/swmmrs";

const input = await readFile("model.inp");
const rainfall = await readFile("rain.dat");
const { report, output } = await runSwmm(
  input,
  { "rain.dat": rainfall },
  { threads: 1 },
);
console.log(report, output.byteLength);
```

Browser callers can pass `File` or `Blob` values. The worker filesystem is
released by `close()`; returned report text and output bytes remain usable.

## Ownership and records

A collection returns a stable handle for its owner and canonical ID. It does not
return a foreign object handle for a related node, link, rain gage, curve, or
LID control. Configuration records expose those relationships as string IDs.
For example, `link.configuration().inletNode` and a LID unit's `control.id` are
identities, not handles from another collection.

`configuration()` and `results()` are asynchronous and return frozen copies. A
handle remains useful after an advance, but an earlier record never changes:
call `await node.results()` again for the new state. Quality reads, snapshots,
and statistics follow the same detached-record rule. See [handles, records, and
snapshots](concepts/results.md).

`ModelTime` values are timezone-free calendar strings such as
`"2020-01-01T00:01:00"`. Treat them as model calendar values, not UTC `Date`
objects. Numeric elapsed time and cadence use seconds.

## Specialized families and quality

Use the model's collections for specialized configuration and keep relationship
fields as IDs. Node and link handles do not become Python-style subtype
instances; their configuration records carry a `kind`, and link records carry a
kind-tagged `subtype`.

The API also exposes aquifer and snowmelt parameter-set patches, AMM and RTK
configuration and assignments, LID controls and units, pollutant quality reads,
and per-object statistics. See [specialized families](api/collections.md#specialized-families)
and [snapshots and statistics](api/snapshots.md).

`simulation.timeSeries` is a read-only identity collection. JavaScript does not
provide editable time-series definitions. Edit the INP when a definition itself
must change, then open a new simulation.

## Output and scenarios

`runSwmm()` and `simulation.finish()` return `{ report, output }`. `saveResults`
defaults to `true`. With `saveResults: false`, the report-period binary output is
not written and `output` is empty. `finish()` still finalizes the report text,
but detailed report tables are written only when results are enabled. Use
[OutputReader](api/output.md) with returned output bytes
to query report-period series, metadata, date bounds, or incomplete output.

EPA hotstarts use `.hsf` bytes:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function warmStart(input: FileContents, hotstart: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		await simulation.useHotstart(hotstart);
		await simulation.start({ saveResults: true });
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

Use `saveHotstart()` during a running or complete run. For fuller swmmrs
continuation, `saveCheckpoint()` returns a `CheckpointBundle` containing a
manifest and its sidecar/dependency bytes. Keep the complete bundle together;
the native checkpoint JSON by itself is not sufficient. `Simulation.resume(bundle)`
creates an independent owner with the saved continuation boundary. `fork()`
creates an independent in-process child. `loadCheckpointState(bundle)` instead
loads classified physical state into an already opened compatible receiver; the
receiver keeps its own dates, declarations, forcing, accounting, and output
history. See [hotstarts, checkpoints, and forks](guides/checkpoints-and-forks.md).

## Deliberate differences

- JavaScript has no synchronous live-property view. Await methods and keep the
  records you want to display.
- JavaScript exposes `finalizeReport()` because the native processing-time
  footer is normally written during project close. The JavaScript worker stores
  project files in its in-memory filesystem, and `close()` removes that
  filesystem, so callers could not read the completed report afterward.
  `finalizeReport()` writes and flushes the footer while the owner and report
  file are still readable; it also avoids detailed table generation when only
  the summary report is needed. Python writes to caller-owned host paths, which
  remain readable after `close()` writes the footer, so it does not need this
  extra public stage.
- JavaScript uses seconds and model-time strings instead of `timedelta` and
  timezone-aware Python dates.
- Batch `run()` cannot be interrupted in the middle of native execution. Use an
  iterator for cooperative stopping, then call `terminate()` at an observation
  boundary. `close()` is cleanup, not a hard interrupt.
- Collection IDs are local metadata. Related object references are IDs, not
  handles owned by another collection.
