# Live views, snapshots, and statistics

The right result interface depends on what you want to keep. A live view follows
the simulation; a saved number remembers what you read. `swmmrs` offers four
result forms for different amounts of data and different lifetimes:

| Form | Use | Ownership |
| --- | --- | --- |
| Live scalar view | Read one or a few current properties. | Generation-bound; each property reads current native state. |
| Snapshot | Copy coherent values for several objects. | Immutable native record; remains valid after advancing or closing. |
| Statistics | Copy cumulative continuity or object summaries. | Immutable native record; acquisition is valid only while `RUNNING` or `COMPLETE`. |
| Binary series | Query stored report periods through `OutputReader`. | Independent reader over a finalized file or complete records recovered from an incomplete file. |

## Live values

```python
node = simulation.nodes["J1"]
link = simulation.links["C1"]

for current_time in simulation:
    print(current_time, node.depth, link.flow)
```

Retaining a live view across advances in one project generation is valid. Retaining it after `close()` or a replacement `open()` is not; access then raises `StaleViewError`.

See [Property getters and Live Views](property-access.md) for acquisition costs,
retention patterns, lifecycle boundaries, and the scalar-versus-snapshot
tradeoff.

## Coherent snapshots

```python
nodes = simulation.nodes.snapshot(["J1", "J2"])
for object_id, depth, inflow in zip(
    nodes.object_ids,
    nodes.depth,
    nodes.total_inflow,
    strict=True,
):
    print(object_id, depth, inflow)
```

Snapshot columns align with `object_ids`, in the order of your selected IDs.
For frequent sampling, select the objects you need. Copying the whole network
to inspect two nodes gives the computer a great deal of unnecessary employment.

Quality snapshots use pollutant-major matrices: the first index selects
`pollutant_ids`, and the second selects `object_ids`.

## Owned native records

Snapshot and statistics classes are frozen Rust-backed Python value objects.
Only the solver can construct them. Their properties, tuples, matrices, and
mappings cannot be mutated, and existing records do not retain a Simulation
Owner or project generation.

```python
sample = simulation.nodes.snapshot(["J1"])
print(repr(sample))

# Build an explicitly mutable host representation when needed.
editable_depths = list(sample.depth)
editable_depths[0] = 2.5
```

`copy.copy()` and `copy.deepcopy()` are unsupported because a copy would still
be the same immutable record shape. Extract the specific fields needed into
lists or dictionaries instead. Those host-side edits never change the record or
solver state.

## Final statistics

Manual or iterator advancement exposes `COMPLETE`, where final statistics are still available:

```python
simulation.start(save_results=False)
while simulation.step() is not None:
    pass

system_statistics = simulation.statistics
node_statistics = simulation.nodes.statistics()
simulation.end()
simulation.close()
```

Copy statistics before `end()`. Once you have the records, they remain valid
along with any snapshots you already collected.

## Data validity

| Acquisition | `OPEN` | `RUNNING` | `COMPLETE` | `ENDED` |
| --- | --- | --- | --- | --- |
| Current hydraulic or quality value | No | Yes | Yes | Yes |
| Hydraulic or quality snapshot | No | Yes | Yes | Yes |
| Statistics | No | Yes | Yes | No |
| Previously copied Python record | Yes | Yes | Yes | Yes |

See [Collect results and statistics](../../guides/collect-results.md) for field lists, acquisition routes, units, and complete collection workflows. Use [Read binary output](../../guides/read-output.md) when the stored report-period artifact is the source of truth.
