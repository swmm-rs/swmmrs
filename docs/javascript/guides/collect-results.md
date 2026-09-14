# Collect results, quality, and statistics

Choose a result shape based on what the application needs:

| Need | Read |
| --- | --- |
| A few current hydraulic or runoff values | `await handle.results()` |
| Current pollutant values for one object | `await handle.quality()` |
| A coherent sample of one object family | `await collection.snapshot(ids?)` |
| Pollutant values for several objects | `await collection.qualitySnapshot(ids?)` |
| Per-object cumulative run values | `await handle.statistics()` or a family `statisticsSnapshot()` |
| System totals and continuity diagnostics | `await simulation.statistics()` |
| Report-period time series | `OutputReader.open(outputBytes)` |

## Collect network samples

Snapshots align one array per result field with `objectIds`. Retain the iterator
time next to each snapshot:

```typescript
import {
	Simulation,
	type FileContents,
	type ModelTime,
	type NodeSnapshot,
} from "@swmmrs/swmmrs";

export async function collectNodeDepths(input: FileContents) {
	const samples: Array<{ time: ModelTime; nodes: NodeSnapshot }> = [];
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		for await (const time of simulation.steps({ seconds: 300, strict: true })) {
			samples.push({ time, nodes: await simulation.nodes.snapshot() });
		}
		const statistics = await simulation.statistics();
		const files = await simulation.finish();
		return { samples, statistics, ...files };
	} finally {
		await simulation.close();
	}
}
```

Pass a subset of IDs for large models. Each snapshot is a new frozen copy, so an
unbounded sample history can consume substantial memory. `snapshot(["J2", "J1"])`
preserves that order, returns canonical IDs, and aligns `depth[i]` with
`objectIds[i]`. Omit IDs for the entire family; `snapshot([])` returns empty
columns. Duplicate and unknown IDs reject, including case-insensitive
duplicates.

Node, link, and subcatchment collections support hydraulic, quality, and
statistics snapshots. Rain-gage collections provide per-object result reads,
not family snapshots. Reads from different families are separate worker requests;
make them without advancing between requests when they must represent the same
state.

## Read quality

Quality records carry `pollutantIds` plus parallel arrays. Quality snapshots use
pollutant-major matrices: the inner row follows `objectIds`.

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function readQuality(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		await simulation.start({ saveResults: false });
		const node = simulation.nodes.get("J1");
		const one = await node.quality();
		const many = await simulation.nodes.qualitySnapshot(["J1"]);
		return { one, many };
	} finally {
		await simulation.close();
	}
}
```

A quality read observes the current solver state. It is not a historical or
report-period query. Live concentration overrides are active-run controls and
apply to the next quality step; see [runtime forcings](runtime-forcings.md).

## Retain statistics

Read system statistics while the model is `running` or `complete`, before
`finish()`. The record includes routing totals, runoff totals, routing
diagnostics, continuity errors, and pollutant quality balances. Per-object
statistics are available in the same active-run window:

- `node.statistics()`, `node.storageStatistics()`, and `node.outfallStatistics()`
  provide node-family records where applicable.
- `link.statistics()` and `link.pumpStatistics()` provide link and pump records.
- `subcatchment.statistics()` provides catchment totals.
- `nodes.statisticsSnapshot()`, `links.statisticsSnapshot()`, and
  `subcatchments.statisticsSnapshot()` read several objects in one request.

Statistics records are detached. A record already returned remains usable after
`finish()` and `close()`, but no statistics read starts after finalization.

## Read report-period series

`steps()` and `stride()` choose observation cadence. `reportStepSeconds` controls
binary report periods. When `saveResults` is true, pass the returned output bytes
to the standalone [OutputReader](../api/output.md) for
metadata and series queries. With `saveResults: false`, the output byte array is
empty, so use live snapshots instead. See [Read binary output](read-output.md)
for selecting columns, time ranges, and memory strategies.
