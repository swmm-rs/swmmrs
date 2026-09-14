# Choose a run workflow

All workflows execute the solver in a worker. Pick one advancement owner for a
simulation and await each operation before starting another.

| Workflow | Use when | Completion and cleanup |
| --- | --- | --- |
| `runSwmm(input, files, options)` | You only need a complete batch result | Returns `RunResults` and closes the owner |
| `Simulation.open()` then `run()` | You need configuration before a batch run or final object reads afterward | Returns `RunResults`; close the owner yourself |
| `simulation.steps(options)` | You need progress, observations, or controls | Exhaustion leaves `complete`; read statistics, finish, then close |
| `start()` then `step()` or `stride()` | Your application owns every advance | `null` means completion; finish and close explicitly |

## Batch execution

`run()` and `runSwmm()` start the model, advance it to completion, write the
requested report and binary output, and return one `RunResults` record. They use
one worker request for the native batch operation. There is no progress callback,
`AbortSignal`, or hard interrupt while that native call is running.

`saveResults` defaults to `true` on `run()` and `start()`. Pass
`{ saveResults: false }` when report-period binary data is not needed. The
returned `output` is then an empty `Uint8Array`; detailed report tables are not
written. Report text still contains the finalized native report content.

Use `Simulation.open()` and `run()` after configuration edits. Use iteration when
you need progress, intermediate results, quality reads, statistics, or controls.

## Interactive execution

`for await (const time of simulation)` observes each routing step. Pass an
interval to `steps()` for fewer observations:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function observe(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		for await (const time of simulation.steps({ seconds: 60, strict: true })) {
			const status = await simulation.status();
			console.log(time, status.percentComplete);
		}
		const statistics = await simulation.statistics();
		return { statistics, files: await simulation.finish() };
	} finally {
		await simulation.close();
	}
}
```

`strict` defaults to `true` for both `steps()` and `stride()`. Exact observation
boundaries can shorten the final routing step in an interval. Pass
`strict: false` explicitly to use whole routing steps that may reach or pass the
requested interval. Observation cadence is separate from report cadence.

Read object results and apply controls inside the loop. Do not call another
advance or `finish()` while the iterator owns advancement. An early `break`
releases the iterator without finalizing the owner, so you can resume, finish a
partial run, or close it.

To stop an active iterator from another controller, call `terminate()`. It is a
synchronous request, not a native interrupt. The iterator ends at its next
observation boundary and the owner remains available for reads or cleanup:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function stopAtFirstObservation(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		const iterator = simulation.steps({ seconds: 60, strict: true });
		const first = await iterator.next();
		if (!first.done) simulation.terminate();
		await iterator.next();
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

## Native lifecycle controls

Use the lower-level controls when an application needs to separate native stages:

- `end()` closes an active run and enters `ended` without releasing the owner.
- `report()` writes requested detailed report tables for an ended run when
  `saveResults` is enabled. It is idempotent.
- `resetSolver()` clears run, result, and output state and returns an `open` owner
  while retaining declared configuration.
- `sleepWorkers()` asks active Dynamic Wave workers to sleep. It is valid only
  while `running`.
- `close()` releases workers and files. It does not interrupt an in-flight native
  operation.

For valid states and transitions, see [lifecycle and ownership](concepts/lifecycle.md).

## Compare scenarios

Open a separate owner for each independent model. A rerun after `finish()` starts
at the configured model start time, not at the previous endpoint. Configuration
and persistent forcings remain until changed.

Use the scenario methods when the starting state matters:

- `saveHotstart()` and `useHotstart(bytes)` exchange selected EPA `.hsf` physical
  state with a new compatible run.
- `saveCheckpoint()` returns a complete `CheckpointBundle`; keep its manifest,
  sidecars, and dependency bytes together. `Simulation.resume(bundle)` creates a
  new owner with fuller continuation state.
- `fork()` creates an independent in-process child at a quiescent running or
  complete boundary. Later controls and results do not share state.
- `loadCheckpointState(bundle)` loads classified warm state into an already open
  compatible receiver. The receiver keeps its own dates, declarations, forcing,
  accounting, and output history. It is not the same operation as resume.

See [hotstarts, checkpoints, and forks](guides/checkpoints-and-forks.md)
for the restoration differences.

Each simulation owns its worker. In Node.js 22 or later, `threads: 1` is the
default and explicit thread counts use `worker_threads`. In a browser,
`threads: 1` works without isolation; multiple threads require COOP/COEP and
`SharedArrayBuffer`. Set explicit budgets when running several owners.
