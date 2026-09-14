# Nodes

Read current node values through a Live View tied to the current project
generation. Edit stable declarations through `node.settings`, using individual
properties or the atomic `settings.update(**changes)` operation.

`StorageSettings`, `OutfallSettings`, and `DividerSettings` inherit common node
settings. You can update common and subtype fields together, and the whole
update succeeds or fails as one operation. Junctions use `NodeSettings`
directly.

```python
from swmmrs import Simulation
from swmmrs.objects import Outfall

with Simulation("model.inp", "model.rpt") as simulation:
    node = simulation.nodes["OUT-1"]
    if isinstance(node, Outfall):
        # Common and outfall-specific declarations form one atomic update.
        node.settings.update(
            invert_elevation=102.5,
            surcharge_depth=1.0,
            has_flap_gate=True,
        )
```

`StorageShape`, `StorageExfiltration`, `OutfallBoundary`, and `DividerRule` are
frozen caller values. Their related curves and time series are Live Views, as
are `Outfall.settings.route_to_subcatchment` and
`Divider.settings.diverted_link`; public object
indexes are not accepted. Storage shapes expose SWMM's canonical stored
coefficients, which makes reads and writes round-trip without attempting to
reconstruct geometry discarded by the parser.

```python
from swmmrs import Simulation
from swmmrs.objects import (
    Divider,
    DividerRule,
    Outfall,
    OutfallBoundary,
    StorageNode,
    StorageShape,
)

with Simulation("model.inp", "model.rpt") as simulation:
    storage = simulation.nodes["STORAGE-1"]
    if isinstance(storage, StorageNode):
        storage.settings.shape = StorageShape("cylindrical", (6.5, 0.0, 0.0))

    outfall = simulation.nodes["OUT-1"]
    if isinstance(outfall, Outfall):
        outfall.settings.update(
            boundary=OutfallBoundary(
                "timeseries",
                reference=simulation.time_series["OUTFALL-STAGE"],
            ),
            route_to_subcatchment=simulation.subcatchments["S-1"],
        )

    divider = simulation.nodes["DIV-1"]
    if isinstance(divider, Divider):
        divider.settings.update(
            rule=DividerRule("tabular", curve=simulation.curves["DIVERSION"]),
            diverted_link=simulation.links["C-2"],
        )
```

The settings keep your complete requested declaration. Reads return the
requested invert elevation, full depth, surcharge, ponding, and initial-depth
values even while configuration is dirty or after a failed `start()`.

The solver prepares the consequences separately: link crowns and slopes,
storage volume and exfiltration geometry, outfall buffers, and divider flow
limits. A successful `start()` installs those changes together. Runtime
hydraulics, routed loads, and forcing retain their separate state.

```python
from swmmrs import ConfigurationError, Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    node = simulation.nodes["J-1"]
    # This relationally invalid candidate is retained for repair at start().
    node.settings.update(full_depth=4.5, initial_depth=6.0)
    assert node.settings.initial_depth == 6.0  # Requested value is visible now.

    try:
        simulation.start()  # Rebuild and validate dependent geometry atomically.
    except ConfigurationError as error:
        for diagnostic in error.diagnostics:
            print(diagnostic.property_path, diagnostic.message)
        node.settings.initial_depth = 0.25
        simulation.start()  # Retry without reopening the project.
```

Node identity, external inflow, fixed-stage runtime override, current quality,
results, snapshots, and statistics remain top-level runtime capabilities and
are not accepted by `node.settings.update()`.

```python
from swmmrs import Simulation
from swmmrs.objects import Outfall

with Simulation("model.inp", "model.rpt") as simulation:
    node = simulation.nodes["J-1"]
    node.external_inflow = 0.25

    outfall = simulation.nodes["OUT-1"]
    if isinstance(outfall, Outfall):
        outfall.fixed_stage = 101.5  # Runtime override, not a settings edit.

    simulation.start()
    while simulation.step() is not None:
        print(node.id, node.depth, node.total_inflow)

    hydraulics = simulation.nodes.snapshot(["J-1", "OUT-1"])
    statistics = simulation.nodes.statistics()
```

`pollut_quality`, inflow concentration, and reactor concentration are immutable
current-result mappings. `override_pollutant_concentrations({...})` atomically
queues nonnegative concentrations for the next quality-routing step only.
`external_pollutant_mass_flux` is instead a persistent live mutable mapping:
item assignment and `update()` are sparse atomic mutations, deletion resets one
configured pollutant to zero, and `clear()` resets all configured pollutants.
Whole-property assignment is unsupported.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    node = simulation.nodes["J-1"]
    flux = node.external_pollutant_mass_flux
    flux["TSS"] = 2.5
    flux.update({"Lead": 0.10})
    del flux["Lead"]  # Reset just Lead to zero.

    simulation.start()
    node.override_pollutant_concentrations({"TSS": 100.0})
    simulation.step()  # The override is consumed by this quality-routing step.
    print(dict(node.pollut_quality))

    flux.clear()  # Atomically reset every configured pollutant to zero.
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - NodeCollection
            - Node
            - NodeSettings
            - Junction
            - Divider
            - DividerSettings
            - DividerRule
            - Outfall
            - OutfallSettings
            - OutfallBoundary
            - StorageNode
            - StorageSettings
            - StorageExfiltration
            - StorageShape
