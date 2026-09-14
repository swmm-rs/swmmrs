# Links

To change what a link is, use its typed `settings` Live View. To control it or
read what it is doing, use the link itself. Runtime controls, forcing, quality,
current results, snapshots, and statistics live there.

```python
from swmmrs import Simulation
from swmmrs.objects import CircularCrossSection, Conduit, Pump

with Simulation("model.inp", "model.rpt") as simulation:
    conduit = simulation.links["C-1"]
    if isinstance(conduit, Conduit):
        conduit.settings.update(
            length=350.0,
            roughness=0.013,
            cross_section=CircularCrossSection(diameter=2.0),
        )

    pump = simulation.links["PUMP-1"]
    if isinstance(pump, Pump):
        pump.settings.use_curve(simulation.curves["PUMP-CURVE"])

    # Runtime controls stay on the link, not its stable settings view.
    pump.target_setting = 0.75
```

Look for an attached inlet on the link that owns it. The optional `link.inlet`
provides access; there is no global `simulation.inlets` collection.
`link.inlet.settings` exposes read-only count, local geometry, and the reusable
design from `simulation.inlet_designs`. Persistent `percent_clogged` and
`flow_limit` controls remain top-level on the inlet, while capture/backflow
values are runtime results. See [Inspect model
definitions](../../guides/inspect-model-definitions.md) for more detail.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    link = simulation.links["C-1"]
    inlet = link.inlet
    if inlet is not None:
        print(inlet.settings.count, inlet.settings.design.id)
        print(inlet.settings.local_depression, inlet.settings.local_width)

        inlet.update(percent_clogged=15.0, flow_limit=2.0)

    simulation.start()
    while simulation.step() is not None:
        if inlet is not None:
            print(inlet.captured_flow, inlet.backflow)

    flows = simulation.links.snapshot(["C-1"])
    statistics = simulation.links.statistics()
```

`pollut_quality`, reactor concentration, and total pollutant load are immutable
current-result mappings. `override_pollutant_concentrations({...})` atomically
queues nonnegative concentrations for the next quality-routing step only.
`external_pollutant_mass_flux` is a persistent live mutable mapping with atomic
item assignment and sparse `update()`. Deletion resets one configured pollutant
to zero, `clear()` resets all configured pollutants, and whole-property
assignment is unsupported.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    link = simulation.links["C-1"]
    flux = link.external_pollutant_mass_flux
    flux.update({"TSS": 1.5, "Lead": 0.05})
    del flux["Lead"]  # Reset one pollutant to zero.

    simulation.start()
    link.override_pollutant_concentrations({"TSS": 80.0})
    simulation.step()
    print(dict(link.pollut_quality))
    print(dict(link.reactor_pollutant_concentration))

    flux.clear()  # Reset all configured pollutant fluxes to zero.
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - LinkCollection
            - Link
            - LinkSettings
            - Conduit
            - ConduitSettings
            - CircularCrossSection
            - StandardCrossSection
            - CustomCrossSection
            - IrregularCrossSection
            - StreetCrossSection
            - Pump
            - PumpSettings
            - Orifice
            - OrificeSettings
            - Weir
            - WeirSettings
            - Outlet
            - OutletSettings
            - FunctionalOutletRating
            - TabularOutletRating
            - Inlet
            - InletSettings
            - InletDesign
