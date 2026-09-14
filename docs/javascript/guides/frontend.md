# Integrate with a frontend

In a browser, the package runs SWMM in a dedicated module worker, keeping solver
work off the UI thread. Serial WASM uses unshared memory and works without
cross-origin isolation or `SharedArrayBuffer`.

## Configure the server

Serve the application over HTTPS or localhost. For browser runs with multiple
solver threads, enable cross-origin isolation with these headers:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Resource-Policy: same-origin
```

The application page must report `globalThis.crossOriginIsolated === true`, and
`SharedArrayBuffer` must be available. Third-party resources must satisfy the
page's embedding policy. The `npm start` server in `js/` provides these headers
for local development. Use `npm start -- --no-isolation` to try serial execution
without them. Serve WASM files with `Content-Type: application/wasm`.

`Simulation.open()` and `runSwmm()` default `threads` to
`navigator.hardwareConcurrency || 1` on an isolated page, or `1` otherwise.
Explicit `threads: 1` always selects serial WASM. An explicit count greater than
one rejects on a non-isolated page rather than silently reducing the count.

JavaScript `options.threads` is separate from the INP `THREADS` setting. Serial
WASM clamps an INP request above one solver thread, so an input containing
`THREADS 4` can run without isolation. Threaded WASM rejects an INP request above
the initialized worker capacity. `await simulation.info()` reports the effective
thread count.

Node.js 22 and later uses `node:worker_threads` instead of browser workers. It
defaults to `threads: 1` and supports explicit thread counts without COOP/COEP.
Node still receives model and supporting-file contents as bytes or text, not host
paths supplied to the runner.

## Deploy worker assets

A static-asset deployment avoids relying on a bundler to discover nested WASM
workers. Copy the built package under one asset directory, preserving its
relative layout:

```text
/swmmrs/
  index.js
  worker.js
  lib/swmmrs/          # compiled modules, including worker helpers
  dist/
    swmmrs.js
    swmmrs_bg.wasm
    snippets/           # generated thread-pool helpers
    serial/
      swmmrs.js
      swmmrs_bg.wasm
```

Include both WASM builds. The loader downloads only the build selected by
`threads`; both expose the same public API. Import the API from the matching
package build and set the public worker URL when needed:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export function openBrowserModel(input: FileContents) {
	return Simulation.open(input, {}, {
		threads: 1,
		workerUrl: new URL("/swmmrs/worker.js", location.href),
	});
}
```

The JavaScript entry module and static assets must come from the same build.
Without `workerUrl`, the API resolves the worker relative to the compiled module.
Automatic asset rewriting by individual bundlers is not qualified. Inspect the
browser network panel to confirm that worker, WASM, and snippet requests return
files rather than an application's HTML fallback.

## Report progress and stop cooperatively

Use `steps()` for periodic UI updates. Call `terminate()` when a stop request
arrives while the iterator is active:

```typescript
import {
	Simulation,
	type FileContents,
	type SimulationStatus,
} from "@swmmrs/swmmrs";

export async function runWithProgress(
	input: FileContents,
	onProgress: (status: SimulationStatus) => void,
	shouldStop: () => boolean,
) {
	const simulation = await Simulation.open(input, {}, {
		threads: 1,
		workerUrl: new URL("/swmmrs/worker.js", location.href),
	});
	try {
		for await (const _time of simulation.steps({ seconds: 60, strict: true })) {
			onProgress(await simulation.status());
			if (shouldStop()) simulation.terminate();
		}
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

`terminate()` is cooperative. The iterator ends at its next observation boundary;
it does not interrupt a native operation already in progress. Choose an
observation cadence that balances UI updates and worker-message overhead. For
network charts, collect a [snapshot](collect-results.md) per observation instead
of making one request per object.

`run()` and `runSwmm()` occupy the worker for the whole native batch request.
They provide no mid-run progress callback, `AbortSignal`, or hard cancellation.
`close()` queues cleanup but does not interrupt that request.

## Manage application ownership

Keep the simulation in the controller or component that owns the run. Coordinate
component teardown with the running task, and let that task's `finally` block
close the owner. Do not let a background iterator and a UI timer advance the same
simulation.

Importing the package is safe during server-side rendering, but opening a model
requires worker support. Open models only in client-side code. Use explicit
thread budgets for multiple scenarios. Each owner has its own worker, and
threaded owners also allocate a worker pool. Independent scenarios can run
concurrently with `threads: 1` in separate workers without shared memory.
