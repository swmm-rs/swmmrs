# Load files and download output

The API accepts file contents. It does not fetch URLs or read host filesystem
paths. `FileContents` accepts strings, `Blob` values such as browser `File`,
`ArrayBuffer`, and array-buffer views.

## Supply model dependencies

Pass supporting files in the second argument to `Simulation.open()` or
`runSwmm()`. Keys are slash-separated paths relative to the INP:

```typescript
import { runSwmm } from "@swmmrs/swmmrs";

export async function runUpload(model: File, rainfall: File) {
	return runSwmm(model, { "rain/rain.dat": rainfall }, { threads: 1 });
}
```

In Node.js 22 or later, read files with `node:fs/promises` and pass the returned
bytes. The runner keeps the same byte-oriented contract in both runtimes:

```javascript
import { readFile } from "node:fs/promises";
import { runSwmm } from "@swmmrs/swmmrs";

const input = await readFile("model.inp");
const rainfall = await readFile("rain/rain.dat");
const result = await runSwmm(input, { "rain/rain.dat": rainfall }, { threads: 1 });
console.log(result.report, result.output.byteLength);
```

Check `fetch()` responses before passing their bodies. An HTTP error page passed
as a string is still input text to the parser. Supporting paths must be relative
and slash-separated. Absolute paths, backslashes, NUL characters, and empty,
`.` or `..` segments are rejected. `model.inp`, `model.rpt`, and `model.out` are
reserved by the runner.

All files are staged before the model opens. There is no public method to add a
new supporting file to an already-open owner.

## Download finalized files

`runSwmm()` and `simulation.finish()` return a `RunResults` record with
`report: string` and `output: Uint8Array`. The output bytes are caller-owned and
remain usable after `close()`:

```typescript
import type { RunResults } from "@swmmrs/swmmrs";

export function showDownloads(results: RunResults, container: HTMLElement) {
	const files = [
		{ name: "model.rpt", blob: new Blob([results.report], { type: "text/plain" }) },
		{
			name: "model.out",
			blob: new Blob([new Uint8Array(results.output)], {
				type: "application/octet-stream",
			}),
		},
	];
	const links = files.map(({ name, blob }) => {
		const link = document.createElement("a");
		link.href = URL.createObjectURL(blob);
		link.download = name;
		link.textContent = `Download ${name}`;
		container.append(link);
		return link;
	});
	return () => {
		for (const link of links) {
			URL.revokeObjectURL(link.href);
			link.remove();
		}
	};
}
```

`saveResults` defaults to `true` on `start()`, `run()`, and `steps()`. With
`saveResults: false`, the run does not write report-period binary results and
`RunResults.output` is an empty `Uint8Array`. `finish()` still finalizes report
text, but detailed report tables are written only when results are enabled and
the model's report-selection settings request them.

## Read project files

`await simulation.readFile(name)` returns a copy of a relative project file from
the worker. Read generated files before closing the owner. `readFile()` does not
finalize or flush an ongoing run; call `finish()` first for completed report or
binary output.

## Query binary output

`OutputReader` is a standalone byte-backed async owner. It accepts finalized
output and valid incomplete output, so it can inspect a file captured before a
run ended. It never reads a host path:

```typescript
import { OutputReader, type FileContents } from "@swmmrs/swmmrs";

export async function readNodeDepth(output: FileContents) {
	const reader = await OutputReader.open(output);
	try {
		return {
			finalized: reader.isFinalized,
			metadata: reader.metadata,
			series: await reader.nodeSeries("J1", "invert_depth"),
		};
	} finally {
		await reader.close();
	}
}
```

Use integer period bounds or `ModelTime` bounds in `start` and `end`. Bounds are
half-open, so `end` is excluded. Element names are exact-case stored names;
integer element indexes are also accepted. `readBulkSeries()` preserves the
selection order, including duplicate selections and empty selections. Set
`lowMemory: true` for its selective adjacent-run read strategy. The reader
exposes immutable metadata, report-period `times`, family series methods, system
series, and stored report dates. Close it when finished; it owns a separate
worker and supports `Symbol.asyncDispose`.

For the complete method and record list, see [the binary output reader reference](../api/output.md).
