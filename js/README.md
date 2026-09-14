# swmmrs for JavaScript and TypeScript

`@swmmrs/swmmrs` runs SWMM in a dedicated worker from a browser or from Node.js
22 and later. A `Simulation` owns its worker, solver state, and in-memory project
files. The package is private and is built from this repository.

See the [JavaScript / TypeScript documentation](../docs/javascript/index.md) for
usage guides and the [API reference](../docs/javascript/api/index.md) for all
public signatures. After building the package, `npm run test:docs` type-checks
the documentation examples.

## Build the browser package

Install Node.js 22 or later, `wasm-pack`, and the Rust toolchain used by this
repository. Then run these commands in `js`:

```sh
npm ci
npm run build
npm start
```

Open `http://127.0.0.1:8080/`. `npm run build` and `npm run build:wasm` produce
threaded WASM in `dist/` and serial WASM with unshared memory in `dist/serial/`.
The pinned Rust nightly toolchain rebuilds the standard library for WASM threads.
`npm run build:wasm -- --dev` builds both modules without optimization, and
`npm run build:ts` rebuilds JavaScript and declarations after TypeScript changes.

Alternatively, with [Just](https://just.systems/) installed, use the shared
build recipes from the repository root:

```sh
just js-deps
just js-debug      # Debug threaded and serial WASM, then TypeScript.
just js-release    # Optimized threaded and serial WASM, then TypeScript.
```

Both profiles write to the same `js/dist/` and `js/lib/` package paths; the
latest build replaces those assets. Cargo dependencies are locked in both modes.

The package exports ES modules. Import `index.js` when serving this directory, or
install `@swmmrs/swmmrs` as a local package dependency. Node uses
`node:worker_threads`; it does not need browser isolation headers.

## Run a model

The API accepts file contents, not host paths or URLs. Valid values are strings,
`Blob`, `ArrayBuffer`, and array-buffer views. Supporting-file keys are relative
paths from the INP.

In Node.js, read files yourself and pass the resulting bytes:

```javascript
import { readFile } from "node:fs/promises";
import { runSwmm } from "@swmmrs/swmmrs";

const input = await readFile("model.inp");
const files = { "rain.dat": await readFile("rain.dat") };
const { report, output } = await runSwmm(input, files, { threads: 1 });
console.log(report, output.byteLength);
```

In a browser, fetch contents or pass `File` and `Blob` values:

```typescript
import { runSwmm } from "@swmmrs/swmmrs";

export async function runBrowserModel() {
	const response = await fetch("/models/model.inp");
	if (!response.ok) throw new Error(`INP request failed: ${response.status}`);
	const input = await response.text();
	const rainfall = await fetch("/models/rain.dat");
	if (!rainfall.ok) throw new Error(`Rainfall request failed: ${rainfall.status}`);
	const files = { "rain.dat": await rainfall.blob() };
	return runSwmm(input, files, { threads: 1 });
}
```

`runSwmm()` starts the model, runs it to completion, finalizes report text and
binary output, and closes the owner. Set `saveResults: false` in the third
argument when report-period binary results are not needed. The returned
`output` is then an empty `Uint8Array`; detailed report tables are not written.

## Inspect and control a run

Use `Simulation.open()` when the application needs configuration, progress,
quality reads, statistics, or runtime controls:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function inspectAndControl(
	input: FileContents,
	files: Readonly<Record<string, FileContents>> = {},
) {
	const simulation = await Simulation.open(input, files, { threads: 1 });
	try {
		const node = simulation.nodes.get("J1");
		const link = simulation.links.get("OR1");
		await node.configure({ fullDepth: 6 });
		await simulation.options.update({ reportStepSeconds: 60 });

		for await (const time of simulation.steps({ seconds: 60, strict: true })) {
			const { depth } = await node.results();
			await link.setTargetSetting(depth > 2 ? 1 : 0.25);
			console.log(time, depth, (await simulation.status()).percentComplete);
		}

		const statistics = await simulation.statistics();
		const files = await simulation.finish();
		return { statistics, ...files };
	} finally {
		await simulation.close();
	}
}
```

`steps()` and `stride()` default to `strict: true`, so interval observations
request exact boundaries. Pass positive whole seconds. Set `strict: false`
explicitly when whole routing-step boundaries are preferred. `step()` and
`stride()` return `null` at natural completion. Collection lookup is synchronous,
but configuration, results, quality, statistics, and controls are asynchronous.
Returned records are frozen copies. Handles remain owner-bound and expose
canonical IDs; related object references are IDs, not foreign handles.

`start()`, `run()`, and `steps()` default to `saveResults: true`. `finish()` ends
the run, flushes binary output, writes requested detailed report tables when
results are enabled, finalizes the report, and retains the owner for reads or
reruns. `end()` writes summary statistics, `finalizeReport()` appends only the
runtime footer, and `report()` appends the requested detailed tables and footer.

## Specialized configuration

Node and link handles do not have public subtype constructors. Their
configuration records carry a `kind`; link records also carry a kind-tagged
`subtype`. The package exposes sparse configuration for node, link,
subcatchment, options, aquifer, snowmelt, AMM, RTK, and LID families.

```typescript
import { type Simulation } from "@swmmrs/swmmrs";

export async function configureSpecialized(simulation: Simulation) {
	await simulation.aquifers.get("A1").configure({ waterTableElevation: 10 });
	await simulation.snowmeltSets.get("Snow").configureSurface("pervious", {
		baseTemperature: 0,
	});
	await simulation.ammModels.get("M1").configure({ hotTemperature: 30 });
	await simulation.unitHydrographs.get("UH1").configure({ rainGage: "Rain" });
	await simulation.replaceAmmAssignments([
		{ node: "J1", model: "M1", area: 1 },
	]);
	await simulation.replaceRdiiAssignments([
		{ node: "J1", unitHydrograph: "UH1", area: 1 },
	]);
	await simulation.lidControls.get("Bio").configure("surface", {
		roughness: 0.1,
	});
	const unit = await simulation.subcatchments.get("S1").lidUnits.access(0);
	await unit.configure({ count: 2 });
}
```

Definition collections such as `timeSeries` expose read-only identity handles.
They do not provide editable time-series definitions. Change those definitions
in the INP and open a new owner. See [specialized families](../docs/javascript/api/collections.md#specialized-families).

## Quality, statistics, and output

Use `node.quality()`, `link.quality()`, or `subcatchment.quality()` for current
pollutant values. The matching collections provide quality snapshots. Per-object
statistics and family statistics snapshots are available while the run is
`running` or `complete`. Capture `simulation.statistics()` before `finish()`
for system totals, diagnostics, continuity errors, and pollutant balances.

For report-period time series, pass finalized or valid incomplete output bytes to
the standalone `OutputReader`. It provides immutable metadata, exact-case name
selectors or integer indexes, integer or `ModelTime` half-open bounds,
`lowMemory` reads, duplicate selections, and system or family series:

```typescript
import { OutputReader, type FileContents } from "@swmmrs/swmmrs";

export async function readOutput(output: FileContents) {
	const reader = await OutputReader.open(output);
	try {
		return await reader.nodeSeries("J1", "invert_depth");
	} finally {
		await reader.close();
	}
}
```

See [binary output reader](../docs/javascript/api/output.md).

## Lifecycle and scenarios

`resetSolver()` clears stale run and output state while retaining declarations and
returns the owner to `open`. `sleepWorkers()` asks active Dynamic Wave workers to
sleep while `running`. `terminate()` cooperatively ends an active iterator at
its next observation boundary. `run()` and `runSwmm()` are single native worker
requests with no progress callback, `AbortSignal`, or hard interrupt. `close()`
is cleanup, not interruption.

For continuation, use `saveHotstart()` and `useHotstart(bytes)` for selected EPA
`.hsf` physical state. Use `saveCheckpoint()` for a complete `CheckpointBundle`
containing a manifest and sidecar/dependency bytes, then `Simulation.resume(bundle)`
for fuller continuation. `fork()` creates an independent in-process child.
`loadCheckpointState(bundle)` loads classified warm state into an already opened
compatible receiver, which keeps its own dates, declarations, forcings,
accounting, and output history. The native checkpoint JSON alone is not a complete
JavaScript bundle. See [hotstarts, checkpoints, and forks](../docs/javascript/guides/checkpoints-and-forks.md).

## Browser assets and threads

Serve `index.js`, `worker.js`, the generated `lib/` directory, and all of `dist/`,
including `dist/snippets/` and `dist/serial/`, from the same build. Serve `.wasm`
with `Content-Type: application/wasm`. Both WASM builds expose the same API.

Serial browser execution with `threads: 1` needs no cross-origin isolation.
Browser `threads > 1` requires these response headers and
`globalThis.crossOriginIsolated === true`:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Resource-Policy: same-origin
```

Browser defaults are `navigator.hardwareConcurrency || 1` on an isolated page and
`1` otherwise. Node defaults to `1`; explicit counts use `worker_threads` when
within host capacity. The browser setup has been tested with Chromium, not every
browser or bundler.

## Handle errors

Native failures preserve `code`, `operation`, and partial `report` when available.
Catch `SwmmError` or a specific `SolverError`, `LifecycleError`, `ValidationError`,
`ObjectNotFoundError`, `WorkerError`, or `OutputError`. JavaScript-side type and
range failures use `TypeError` or `RangeError`. Close failed owners and open a new
one for retries.

## Run checks

The commands below build their WASM prerequisites:

```sh
npm run typecheck
npx playwright install chromium
npm test
npm run test:package
npm run test:docs
```

Set `CHROMIUM_PATH` to use an installed Chromium executable. Browser tests run
WASM and include serial and parallel paths. The documentation checker type-checks
all `typescript` fences as standalone modules.

## Find the implementation

`src/swmmrs` holds the public TypeScript API, including simulation ownership,
objects, snapshots, output, and exceptions. `protocol.ts`, `client.ts`, and
`worker.ts` are private worker plumbing. `src/native` contains Rust adapters
organized by object family. The root `index.d.ts` re-exports declarations built
from the TypeScript source.
