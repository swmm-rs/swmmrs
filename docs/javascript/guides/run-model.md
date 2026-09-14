# Run a model

Use async iteration when the application needs progress, intermediate results, or
runtime controls. The iterator starts an `open` or `ended` simulation lazily.

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function observeRun(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		for await (const time of simulation.steps({ seconds: 60, strict: true })) {
			const status = await simulation.status();
			console.log(time, status.percentComplete);
		}

		const statistics = await simulation.statistics();
		const files = await simulation.finish();
		return { statistics, ...files };
	} finally {
		await simulation.close();
	}
}
```

`seconds` must be an integer from 1 through 2,147,483,647. Omit it to observe
every routing step. `strict` defaults to `true`; exact intervals can shorten a
routing step. Use `strict: false` only when whole routing-step advancement is the
required behavior. See [units and time](../concepts/units-and-time.md).

`steps()` also accepts `{ saveResults }`, which defaults to `true`. Set
`saveResults: false` when report-period binary results are not needed. The final
`RunResults.output` is then empty, while live reads and snapshots still work.

## Advance manually

Call `start()` before the first manual `step()` or `stride()`:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function manualRun(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		await simulation.start({ saveResults: true });
		while (true) {
			const time = await simulation.stride(60); // strict defaults to true
			if (time === null) break;
			console.log(time);
		}
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

`step()` advances one routing step. `stride(seconds, strict = true)` advances an
observation interval. Both return `null` when the solver completes. A `null`
result is completion, not another timestamp.

## Stop or resume an interactive run

Breaking from an iterator releases its advancement ownership. It does not call
`finish()` or `close()`. After the break, you can iterate again, manually advance
a `running` model, read statistics, finish partial output, or close the owner.

To request a cooperative stop from another controller, call `terminate()` while
the iterator is active. It ends the iterator at its next observation boundary.
It does not interrupt a native operation already in progress:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function stopAfterOneSample(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		const iterator = simulation.steps({ seconds: 60, strict: true });
		const sample = await iterator.next();
		if (!sample.done) simulation.terminate();
		await iterator.next();
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

Read results and apply controls inside the loop, but do not call `step()`,
`stride()`, `start()`, `run()`, `end()`, `finalizeReport()`, `report()`, or
`finish()` there. An active iterator owns advancement, so those operations
reject with `LifecycleError`.

## Finalize and release

Natural exhaustion leaves the model `complete`, with system and per-object
statistics available. `finish()` ends the run, flushes binary output, writes the
requested detailed report tables when results are enabled, finalizes report text,
and returns `{ report, output }`. Repeating `finish()` in `ended` is safe.

`end()`, `finalizeReport()`, and `report()` expose those native stages
separately. `end()` advances to `ended` with summary output; `finalizeReport()`
only appends the runtime footer, and `report()` adds requested detailed tables.
`resetSolver()` returns an `open` owner while retaining declared configuration.
`sleepWorkers()` can put active Dynamic Wave workers to sleep while `running`.

The runtime footer is normally written at project close. JavaScript exposes
`finalizeReport()` so it can flush that text before `close()` removes the worker's
in-memory files. Python's caller-owned report path remains available after close,
so it does not need the same explicit operation.

For a new owner that continues or branches a run, see
[Hotstarts, checkpoints, and forks](checkpoints-and-forks.md).

Always await `close()` in a `finally` block. `runSwmm()` is the batch convenience
that opens, runs, finalizes, and closes for you. Batch execution has no progress
callback, `AbortSignal`, or hard cancellation. See the full
[lifecycle contract](../concepts/lifecycle.md).
