# Troubleshooting

## Opening rejects with a worker or isolation error

In a browser, a request for `threads > 1` requires
`globalThis.crossOriginIsolated === true` and `SharedArrayBuffer`. Use HTTPS or
localhost, verify the COOP and COEP headers, and check the browser console. For
a local threaded baseline, run `npm start` in `js/`.

For browser execution without isolation, use `threads: 1` or omit `threads`. The
serial build needs neither isolation headers nor `SharedArrayBuffer`. An
explicit request for multiple browser threads rejects on a non-isolated page.
Try `npm start -- --no-isolation` locally. See [frontend hosting](guides/frontend.md).

Node.js 22 and later uses `node:worker_threads`; it does not need browser
isolation headers. Node defaults to `threads: 1`, and explicit counts above one
use the threaded build when within host capacity.

JavaScript `options.threads` is separate from the INP `THREADS` setting. Serial
WASM accepts inputs such as `THREADS 4` and clamps the solver count to one. If
threaded initialization rejects an INP request above capacity, reduce the INP
`THREADS` setting or increase JavaScript `options.threads` within the runtime's
supported maximum.

## Worker, WASM, or snippet requests fail

Inspect the network panel for missing assets, MIME errors, blocked cross-origin
resources, or responses containing HTML instead of JavaScript/WASM. Deploy
`lib/swmmrs/` and `dist/`, including `dist/snippets/` and `dist/serial/`, from the
same build. Keep their relative paths intact and set `workerUrl` if your server
places the worker at a different public URL.

## The solver reports an invalid input file

Pass INP contents, not a path or URL string. In Node, read the file with
`node:fs/promises` and pass the returned bytes. In a browser, check the fetch
response status before using its body. For supporting files, match each relative
INP reference to a supplied key, including directory components. Read the
error's `report` when available; some early parsing errors only return a generic
message.

## An operation throws `LifecycleError`

Check `await simulation.getState()`. Configure before starting or after
finalization. Read object results and quality after starting. Read system and
per-object statistics before `finish()`. Do not manually advance or finalize
inside an active iterator. `terminate()` is valid only while that iterator is
active. Closed owners, collections, and handles cannot be reused. See the
[state table](concepts/lifecycle.md).

## Values do not change in the UI

A `results()`, `quality()`, or `configuration()` record is a detached copy. Call
the method again after advancing, or collect a fresh snapshot. Assignments to
records do not edit the solver. Use `configure()` or a runtime control method.
Related objects in configuration records are represented by IDs, not foreign
handles.

Check project units as well: subcatchment fractions are 0 to 1, slopes are ratios,
and model timestamps are calendar strings without a timezone. See [units and
time](concepts/units-and-time.md).

## Progress or stopping does not respond during a run

`run()` and `runSwmm()` occupy the simulation worker for the entire native batch
request. They have no progress callback, `AbortSignal`, or hard interrupt. Use
`steps()` for periodic UI updates and call `terminate()` for cooperative
stopping at the next observation boundary. `close()` is cleanup and does not
interrupt an in-flight native operation.

## Rainfall does not return after clearing an override

`setRainfallOverride(null)` clears the override only. If you previously called
`setPrecipitation()`, the API source remains selected, even when its rate is zero.
Open a fresh model to restore its original source and co-gage setup. See
[Runtime forcings](guides/runtime-forcings.md).

## Binary output cannot be queried

`OutputReader.open()` needs output bytes. With `saveResults: false`,
`RunResults.output` is intentionally empty. Use live snapshots instead, or run
with `saveResults: true` and query the bytes through [OutputReader](api/output.md).
A reader can inspect valid incomplete output, but it still owns a worker and must
be closed.

## Types are missing or cannot be imported as values

Build the package before consuming it, and import from the public root
`"@swmmrs/swmmrs"`. Obtain `Node`, `Link`, `Subcatchment`, `RainGage`, and
specialized handles from their collections; they have no public constructors.
Import record interfaces and string unions with `import type`. `Simulation`,
`runSwmm`, `OutputReader`, and exception classes are runtime exports.

## Validate a local build

From `js/`, run `npm run typecheck` for the public TypeScript contract, `npm test`
for browser integration tests, and `npm run test:package` to inspect packaged
assets. Browser tests require Playwright Chromium or `CHROMIUM_PATH` pointing to
an installed Chromium binary.

`npm run test:docs` type-checks documentation TypeScript examples against the
built public declarations. It does not replace browser integration tests.
