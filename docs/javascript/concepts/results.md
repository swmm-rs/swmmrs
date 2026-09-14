# Handles, records, and snapshots

The JavaScript API separates local object identity from asynchronous solver reads.
This is the main difference from Python's live scalar properties.

| Value | Represents | After an advance | After `close()` |
| --- | --- | --- | --- |
| Object handle, such as `simulation.nodes.get("J1")` | Stable owner-local ID and worker route | Reads observe the new state | Solver operations reject |
| `configuration()` result | Copy of declarations and derived fields | Does not change | Remains usable |
| `results()` result | Copy of one object's current hydraulic/runoff values | Does not change | Remains usable |
| `quality()` result | Copy of pollutant concentrations or loads | Does not change | Remains usable |
| Collection snapshot | Aligned family columns copied in one request | Does not change | Remains usable |
| Statistics result | Cumulative values at the time of the call | Does not change | Remains usable |
| `RunResults.output` or `readFile()` bytes | Caller-owned file copy | Does not change automatically | Remains usable |

## Local identity and asynchronous reads

`ids`, `size`, `has()`, `get()`, `at()`, and collection iteration use metadata
loaded at open time. They do not make worker requests. Lookup is case-insensitive,
and repeated lookup of one canonical ID returns the same handle.

`await node.results()` and `await node.configuration()` each make an asynchronous
worker request and return a new record. There is no synchronous `node.depth`
getter. Related configuration fields such as `inletNode`, `rainGage`, `control`,
and `unitHydrograph` are canonical string IDs. They are not handles from another
collection or owner.

## Detached records

Returned records and nested arrays are frozen. TypeScript marks their fields as
`readonly`. To edit UI state, copy the fields the UI needs. To change the solver,
call `configure()`, a specialized configuration method, or a runtime control.
Assigning to a returned record does not change the model.

`Uint8Array` file values are the exception. They are caller-owned mutable copies;
editing them does not edit the worker's project files.

## Current state, quality, and statistics

Object `results()` and `quality()` calls read the current solver state. They are
not report-period queries and contain no history. `NodeCollection`,
`LinkCollection`, and `SubcatchmentCollection` also provide `qualitySnapshot()`
and `statisticsSnapshot()` methods. Their pollutant matrices are pollutant-major,
and each inner array follows `objectIds`.

Use per-object statistics when one object needs cumulative run values:
`node.statistics()`, `node.storageStatistics()`, `node.outfallStatistics()`,
`link.statistics()`, `link.pumpStatistics()`, or `subcatchment.statistics()`.
Use family statistics snapshots to read several nodes, links, or subcatchments in
one worker request. These statistics require a `running` or `complete` run and
are not available after `finish()`.

`simulation.statistics()` returns system routing totals, runoff totals, routing
diagnostics, continuity errors, and pollutant quality balances. Capture it while
`running` or `complete`, before finalization. A detached statistics record remains
usable after `finish()` and `close()`.

## Snapshots and report periods

A family snapshot aligns every result column with `objectIds`. Pass IDs to choose
and order objects, omit IDs for the complete family, or pass `[]` for empty
columns. Duplicate and unknown IDs reject. Keep the iterator's `ModelTime` next
to each live snapshot if you need a time axis.

`reportStepSeconds` controls binary report periods independently from
`steps({ seconds })` and `stride(seconds)`. For report-period values, read the
finalized bytes with [OutputReader](../api/output.md).
The reader is separate from live handles and can also inspect valid incomplete
output.
