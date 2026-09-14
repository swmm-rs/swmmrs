# Build and run your first model

This tutorial builds the browser package, runs an INP file, and reads its report.
You need a repository checkout with the solver source, Node.js 22 or later with
npm, Rust managed by rustup, and `wasm-pack`.

## Build the browser package

Install `wasm-pack` if it is not already available:

```sh
cargo install wasm-pack --locked
```

From the repository root:

```sh
cd js
npm ci
npm run build
npm start
```

The build produces threaded WASM in `dist/` and serial WASM in `dist/serial/`
with the pinned Rust nightly toolchain. Open <http://127.0.0.1:8080/> and select
an INP file in the demo. Supply every supporting file referenced by that model.

For development, `npm run build:wasm -- --dev` produces both WASM builds without
optimization. `npm run build:ts` rebuilds the JavaScript and declarations.

The server enables cross-origin isolation by default. To try serial execution
without isolation headers, run `npm start -- --no-isolation`. Omitted `threads`
defaults to one on that page. The examples below select the serial build
explicitly, so they work in either server mode.

## Run from a browser

Open the browser console on the demo page and run this example. It adds a file
picker at the bottom of the page:

```typescript
const { runSwmm } = await import("./index.js");
const picker = document.createElement("input");
picker.type = "file";
picker.accept = ".inp";
picker.onchange = async () => {
	const input = picker.files?.[0];
	if (!input) return;
	try {
		const { report, output } = await runSwmm(input, {}, { threads: 1 });
		console.log(report);
		console.log(`Binary output: ${output.byteLength} bytes`);
	} catch (error) {
		console.error(error);
	}
};
document.body.append(picker);
```

The browser and Node APIs accept file contents, not paths. Browser `File` values,
strings, `ArrayBuffer`, and typed arrays are valid inputs. `runSwmm()` finalizes
the report and binary output, then closes the owner before returning.

## Run from Node.js

The package supports Node.js 22 and later through `node:worker_threads`. Read
files yourself and pass their bytes. Node defaults to one solver thread; pass an
explicit `threads` count when parallel workers are useful.

```javascript
import { readFile } from "node:fs/promises";
import { runSwmm } from "@swmmrs/swmmrs";

const input = await readFile("model.inp");
const files = { "rain.dat": await readFile("rain.dat") };
const { report, output } = await runSwmm(input, files, { threads: 1 });
console.log(report, output.byteLength);
```

Supporting-file keys are relative paths referenced by the INP. The runner does
not fetch URLs or read host filesystem paths.

## Use the package in a TypeScript application

Build `js/` first. From your application's directory, install it as a local
dependency, substituting your checkout path:

```sh
npm install /path/to/swmmrs/js
```

Application code can import `Simulation`, `runSwmm`, and `OutputReader` from
`"@swmmrs/swmmrs"`. Import record types with `import type`. The package uses ES
modules and includes its own declarations.

Installing the dependency does not configure worker assets or HTTP headers.
Follow [integrate with a frontend](guides/frontend.md) before running it in a
browser application. Opening the demo through `file://` does not work.

## Next steps

- [Choose a run workflow](workflows.md) for progress, controls, and lifecycle stages.
- [Load supporting files and query output](guides/files.md).
- [Inspect and configure a model](guides/configure-model.md), including AMM, RTK,
  and LID families.
- [Collect quality and statistics](guides/collect-results.md).
- Read about [checkpoints, hotstarts, and forks](concepts/lifecycle.md).
