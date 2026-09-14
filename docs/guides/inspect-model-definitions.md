# Inspect model definitions

A `Simulation` exposes named project configuration through both **model-element
collections** and **definition collections**. In this guide, a *definition* is
a named, project-level configuration record outside the primary hydraulic and
runoff element collections. Definitions often provide shared data that model
elements refer to. For example, a pump can refer to a curve and multiple
subcatchments can refer to one aquifer, but some (such as control rules) instead
configure the project as a whole.

A *model element* is an instance placed in the modeled system, such as a node,
link, or subcatchment. Those objects are available from `simulation.nodes`,
`simulation.links`, and `simulation.subcatchments` and are documented in
[Configure a model](configure-model.md).

Whether an object is a definition or a model element is separate from whether
it is editable. Some definitions are editable and others are read-only, just as
model elements expose a mix of editable settings and read-only data. Therefore,
the table below intentionally omits nodes, links, and subcatchments: it lists
only definition collections, not every collection that supports updates.

All of these collections are generation-bound and preserve project order. Use
them to discover IDs, inspect relationships, and select typed objects without
parsing the input file in Python.

## Definition collections and editability

| Collection | Examples | Configuration support |
| --- | --- | --- |
| `pollutants`, `land_uses` | Quality identities and configured definitions | Read-only |
| `time_patterns`, `curves`, `time_series` | Reusable temporal and tabular definitions | Read-only |
| `controls` | Parsed control-rule identities | Read-only |
| `transects`, `shapes`, `streets` | Reusable hydraulic geometry | Read-only |
| `unit_hydrographs` | EPA SWMM RTK definitions | Editable through atomic properties and `update()`; see [RDII objects](../python/api/rdii.md) |
| `inlet_designs` | Reusable inlet-design identities | Read-only |
| `aquifers` | Shared groundwater definitions | Editable through atomic declaration-backed properties and `update()` |
| `snowmelt_sets` | Shared snowmelt parameter sets and surfaces | Editable through atomic declaration-backed properties and `update()` |
| `lid_controls` | LID process definitions and optional layers | Editable through the documented LID process/layer operations |

The absence of a setter is intentional. Read-only definitions do not become
mutable merely because a relationship points to them.

## Discover identities and preserve project order

```python
from swmmrs import Simulation
from swmmrs.objects import Pump

with Simulation("model.inp", "model.rpt") as simulation:
    print(tuple(simulation.curves))
    print(tuple(simulation.time_series))
    print(tuple(simulation.aquifers))

    # IDs resolve case-insensitively; objects report canonical configured IDs.
    aquifer = simulation.aquifers["upper-aquifer"]
    print(aquifer.id)
```

Collections support string identity and configured integer index lookup. Unknown
IDs raise `KeyError`; invalid indices raise `IndexError`. Collections and their
objects become stale after `close()` followed by a later `open()`, so reacquire
them from the new Project Generation.

## Follow typed relationships

Relationships return another typed Live View or `None`, rather than an untyped
index. Check optional relationships explicitly:

```python
with Simulation("model.inp", "model.rpt") as simulation:
    aquifer = simulation.aquifers["AQUIFER-1"]
    pattern = aquifer.upper_evaporation_pattern
    if pattern is not None:
        print("evaporation pattern:", pattern.id)

    pump = simulation.links["PUMP-1"]
    if isinstance(pump, Pump):
        curve = pump.settings.curve
        print("pump curve:", None if curve is None else curve.id)
```

The exact available relationship depends on the object's subtype and input
model. Prefer typed properties from the family API over reconstructing
relationships by matching IDs yourself.

## Shared definitions versus instance settings

An aquifer is a reusable shared definition. A subcatchment's groundwater
component owns instance-specific values such as its surface elevation and
initial water-table elevation. Editing `simulation.aquifers[id]` changes the
shared requested definition; editing
`simulation.subcatchments[id].settings.groundwater` changes that subcatchment's
requested groundwater component.

Aquifer and snowmelt edits use deferred Configuration Preparation. Requested
values read back immediately, while relational problems are reported by
`start()` as `ConfigurationError` diagnostics. See [Configure a
model](configure-model.md).

## Link-attached inlets

Inlet placement belongs to a link, so there is no global `simulation.inlets`
collection. Reusable designs are available from `simulation.inlet_designs`, and
an eligible link exposes an optional attached inlet:

```python
with Simulation("model.inp", "model.rpt") as simulation:
    link = simulation.links["CONDUIT-1"]
    inlet = link.inlet
    if inlet is not None:
        settings = inlet.settings
        print(settings.count, settings.local_depression, settings.local_width)
        print("design:", settings.design.id)

        # Persistent controls are updated atomically.
        inlet.update(percent_clogged=10.0, flow_limit=2.5)
```

Placement settings (`count`, local geometry, and design) are stable read-only
configuration. `percent_clogged` and `flow_limit` are persistent controls.
Runtime `captured_flow`, `backflow`, and `backflow_ratio` require a valid result
lifecycle.

## Reference pages

- [Model definitions](../python/api/definitions.md)
- [Time series](../python/api/time_series.md)
- [Links and attached inlets](../python/api/link.md)
- [LID definitions](../python/api/lid.md)
- [Collections](../python/api/generics.md)
