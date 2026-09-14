# Property getters and Live Views

Reading `node.depth` asks the solver how deep the water is now. The `Node` is a
**Live View**, a small reference to one configured object in one project
generation. Each property access asks its `Simulation` owner for the current
value and returns an ordinary Python scalar. That returned number stays put
even when the water does not.

```python
node = simulation.nodes["J1"]
# With simulation RUNNING:
simulation.step()
first = node.depth
simulation.step()
second = node.depth

print(first, second)
# `node` still refers to J1. `first` is still the earlier Python float.
```

The two values can be equal because the hydraulic state might not change. The
important distinction is ownership: `node` remains connected to the
simulation, while `first` and `second` are independent point-in-time values.

## What a getter does

An innocent-looking property read has a little work behind it. Each read:

1. crosses the Python/Rust boundary;
2. locks the Simulation Owner;
3. validates the Project Generation and typed object identity;
4. reads and converts the current native value; and
5. constructs the requested Python value.

Getters do not cache current results, so a retained view observes newer solver
state after each advance. A scalar getter converts only the requested value,
but every access still incurs the boundary, lock, and validation costs.

Native code handles case-insensitive collection lookup, configured order,
membership, bounds, and subtype identity. For relationships, it validates both
endpoints and creates the related Live View without caching it. Python stores
only the identity.

Configuration properties use the same view and validation mechanism, but read
model configuration rather than changing hydraulic results. For example,
`node.settings.invert_elevation` is stable until configuration changes, while
`node.depth` can change after every step. Setter docstrings state the lifecycle
states in which each configuration or forcing value may be changed.

## Retain the view, not the value

Resolve collections and object IDs before a hot loop:

```python
nodes = simulation.nodes
monitor = nodes["J1"]
conduit = simulation.links["C1"]

for current_time in simulation:
    print(current_time, monitor.depth, conduit.flow)
```

Avoid rebuilding the collection and resolving the ID on every iteration:

```python
for current_time in simulation:
    print(current_time, simulation.nodes["J1"].depth)  # avoid in a hot loop
```

On large models, repeated collection construction and ID resolution can cost
far more than a getter on an already-retained view. Retaining a view does not
retain native result storage and does not prevent the solver from advancing.

Reading the same property once is also clearer and cheaper than reading it
repeatedly during one callback:

```python
depth = monitor.depth
if depth > alarm_depth:
    alarms.append((current_time, depth))
```

## Lifecycle and Project Generations

A Live View belongs to the Project Generation in which it was created.
Advancing the same generation does not invalidate it. Closing the project, then
opening a project, creates a new generation and makes every earlier collection
and Live View stale.

```python
old_node = simulation.nodes["J1"]
simulation.close()
simulation.open("next.inp", "next.rpt")

# Raises StaleViewError even if next.inp also contains J1.
print(old_node.depth)
```

Reacquire collections and views after `open()`. The stale check prevents an old
index or ID from silently referring to an object in a different model.

Current hydraulic and quality getters are valid in `RUNNING`, `COMPLETE`, and
`ENDED`. They are not valid in `OPEN`, before routing state has been
initialized. `COMPLETE` is observable after manual or iterator advancement
reaches the model end. A failed generation has no valid current-result access
path: retain values already copied into Python, preserve the solver error, and
close it.

A scalar already returned by a getter is an ordinary Python value. It remains
valid after another step, `end()`, `close()`, or deletion of the simulation.
The same is true of completed snapshot and statistics records.

## Live getters or snapshots?

| Need | Use | Cost and behavior |
| --- | --- | --- |
| One or a few values per callback | Retained Live Views and scalar getters | One owner acquisition and one Python value per property. Reads current state. |
| Several fields for several objects | A selected `snapshot()` | One coherent owner acquisition; copies aligned columns into a new immutable record. |
| Current values for the whole network | An unfiltered `snapshot()` | Copies every supported column for every configured object; cost grows with model size. |
| One object and one pollutant | A Live View quality mapping | Copies pollutant IDs and the selected value vector. |
| Several objects or pollutants | `quality_snapshot()` | Amortizes acquisition and preserves aligned pollutant-major data. |
| Historical or cross-thread handoff | Scalars, tuples, or snapshots already copied into Python | Owner-free and unaffected by later solver state. |
| Cumulative or final summaries | Statistics properties or `statistics()` | Copies summary records; acquire final statistics before `end()`. |

Two reads next to each other in Python do not necessarily describe the same
solver instant. Each getter acquires the owner separately, and another thread
could advance the simulation between them. These values might come from
different steps:

```python
depth = monitor.depth
inflow = monitor.total_inflow
```

A snapshot reads all requested columns while holding one owner acquisition:

```python
sample = simulation.nodes.snapshot(["J1"])
depth = sample.depth[0]
inflow = sample.total_inflow[0]
```

Do not use snapshots indiscriminately. `snapshot()` without IDs copies the
complete configured collection. For frequent sampling, pass only the required
IDs and keep the callback cadence no finer than the monitoring or control task
requires.

## Allocation and performance tradeoffs

Live scalar getters allocate only the returned Python value and any data
required by that specific result. They are the lowest-overhead choice for a
small working set, but many independent getters repeat boundary, lock,
validation, conversion, and object-construction work.

Each snapshot call creates fresh native column vectors and fresh immutable
Python tuples. This is required by the public contract: an older snapshot must
not change when the model advances or closes. Reusing the Python-visible buffer
would make historical snapshots mutate. A private scratch buffer would save
only temporary native capacity while the final Python-owned columns would
still need to be copied; it is not used by the current implementation.

Practical rules:

- retain collections and Live Views outside loops;
- read a scalar once per callback when one value serves multiple decisions;
- use a selected snapshot when crossings or same-time alignment dominate;
- avoid full-network snapshots followed by Python-side filtering;
- store only the history the consumer needs; and
- benchmark the actual model and callback cadence when result collection is a
  material part of runtime.

There is no magic number of properties at which snapshots become cheaper.
Their cost depends on the selected objects and copied columns. Getter cost
depends on the number of acquisitions. Start with retained scalar getters for
a few values. Switch to a selected snapshot when you need its complete record
or measurements show that repeated acquisitions dominate.

See [Live views, snapshots, and statistics](results.md) for result forms and
[Lifecycle and ownership](lifecycle.md) for the complete state machine.
