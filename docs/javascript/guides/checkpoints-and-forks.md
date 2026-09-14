# Hotstarts, checkpoints, and forks

Choose the mechanism according to what the next run should retain:

- **Hotstart:** carry hydraulic state into a separately configured run.
- **Checkpoint resume:** recreate an independent owner with the checkpoint's
  model and continuation state.
- **Checkpoint State Load:** stage physical and numerical state in an already
  open compatible model while retaining that receiver's declarations and files.
- **Fork:** capture a checkpoint and resume it in a new independent owner.

See [Continuation and branching](../../concepts/continuation-and-branching.md)
for the underlying distinctions.

## Continue a compatible model from a hotstart

Capture hotstart bytes while the source is running or complete, before ending it.
Configure the receiving owner before starting its run:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function continueHydraulics(input: FileContents, hotstart: Uint8Array) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		await simulation.useHotstart(hotstart);
		return await simulation.run();
	} finally {
		await simulation.close();
	}
}
```

Hotstart bytes are caller-owned. They are not a complete project archive and do
not replace the receiving model's declarations.

## Branch at an observation boundary

Stop advancing before taking a checkpoint or fork. The source remains independent
of the branch; close both owners when finished:

```typescript
import type { Simulation } from "@swmmrs/swmmrs";

export async function runBranch(source: Simulation) {
	// The caller has advanced source to a running or complete, quiescent boundary.
	const branch = await source.fork({ threads: 1 });
	try {
		for await (const time of branch.steps({ seconds: 300 })) {
			console.log(time);
		}
		return await branch.finish();
	} finally {
		await branch.close();
	}
}
```

For persistence, retain the complete `saveCheckpoint()` bundle, including its
manifest and all sidecars. Resume it with `Simulation.resume(bundle)`. Bundle
paths belong to the archive, not the host filesystem. Do not discard sidecars or
substitute files from another model; validation checks identities and dependencies.

To apply state to your own declarations instead, open a compatible model, edit it,
and call `loadCheckpointState(bundle)` before starting. An ended receiver must
first return to `open`, for example through `resetSolver()`. Resetting again after
loading clears the staged state. Loading does not merge checkpoint declarations
into the receiver.

See the [Simulation reference](../api/simulation.md) for exact states, bundle
fields, validation, and return contracts.
