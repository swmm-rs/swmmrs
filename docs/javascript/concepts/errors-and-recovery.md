# Errors and recovery

Worker requests reject with JavaScript error objects. Native details cross the
worker boundary when available, including `code`, `operation`, `detail`,
`semanticCode`, configuration diagnostics, and partial `report` text.

Catch `SwmmError` for binding and native failures, or use a specific subclass
from the [exception reference](../api/exceptions.md):

```typescript
import { runSwmm, SwmmError, type FileContents } from "@swmmrs/swmmrs";

export async function runWithDiagnostics(input: FileContents) {
	try {
		return await runSwmm(input, {}, { threads: 1 });
	} catch (error) {
		if (error instanceof SwmmError) {
			console.error(error.operation, error.code, error.message);
			if (error.report) console.error(error.report);
		}
		throw error;
	}
}
```

`OutputReader` failures use `OutputError`, which has a `category` and optional
`operation`. It is independent of a `Simulation` owner.

## Recoverable errors

A rejected configuration or specialized patch does not partially commit that
patch. Correct the fields and retry in `open` or `ended`. Invalid lifecycle
operations, unknown IDs, duplicate snapshot selections, and competing
advancement also normally leave a healthy owner available.

Collection lookups can throw synchronously. Invalid file-content values and
boolean options use `TypeError`. Invalid thread counts, observation intervals,
collection indices, and other numeric bounds use `RangeError`. These JavaScript
errors are not subclasses of `SwmmError`.

## Failed runs

A solver failure can move the owner to `failed`. Preserve the rejected operation's
error and any partial report, then close the owner. Do not assume another read,
`finish()`, or scenario operation will succeed after a native solver failure.

Initialization failures, fatal worker failures, and WASM traps tear down worker
resources. Open a new simulation to retry. Some first-pass input errors contain
only a generic native input-error message; later parsing failures can include
more detailed report diagnostics.

## Preserve the primary failure

`runSwmm()` preserves a run failure if cleanup also fails, attaching the cleanup
failure as `cleanupError`. In manual ownership, a cleanup exception in a plain
`finally` block can replace an earlier error. Record the primary failure and
handle cleanup separately when that distinction matters. Always attempt cleanup;
do not reuse a failed owner.
