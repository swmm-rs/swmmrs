# Low-impact development

Use `unit.update(**changes)` to edit a subcatchment's configured `LidUnit`
properties together. Every optional LID-control layer also has
`update(**changes)`, and individual property assignments use the same atomic
native operation. Invalid scalar values fail immediately. The next `start()`
checks process, group, and subcatchment relationships together.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    unit = simulation.subcatchments["S-1"].lid_units[0]
    unit.update(
        area=500.0,
        full_width=25.0,
        initial_saturation=0.20,
        routes_to_pervious=True,
        drain_destination=simulation.nodes["J-1"],
    )

    control = unit.control
    surface = control.surface
    if surface is not None:
        surface.update(thickness=0.25, roughness=0.15, slope=0.01)
        surface.roughness = 0.20  # Scalar assignment uses the same update approach.
```

The solver handles reads, value conversion, relationship identity, allowed
lifecycle states, and atomic updates. Derived layer values and runtime
snapshots are read-only. Keeping a view of an optional layer does not keep the
layer in existence: access raises `StaleViewError` while that layer is absent.

```python
from swmmrs import Simulation, StaleViewError

with Simulation("model.inp", "model.rpt") as simulation:
    control = simulation.lid_controls["BIORETENTION"]
    soil = control.soil
    if soil is not None:  # Layers depend on the configured LID process.
        soil.update(porosity=0.45, field_capacity=0.20, wilting_point=0.10)

    try:
        if soil is not None:
            print(soil.thickness)
    except StaleViewError:
        # Reacquire the optional slot after configuration changes.
        soil = control.soil
```

Stable LID declarations are writable only while the simulation is Open or
Ended; every LID write is rejected while Running. Requested
`routes_to_pervious` and implicit drain intent remain visible while dirty, while
prepared alpha, overflow, bottom width, and runtime snapshots continue to
represent the last successful preparation or run.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    unit = simulation.subcatchments["S-1"].lid_units[0]
    unit.update(routes_to_pervious=True, drain_destination=None)
    assert unit.routes_to_pervious is True  # Requested intent is visible while dirty.

    simulation.start()
    surface = unit.control.surface
    if surface is not None:
        print(surface.alpha, surface.immediate_overflow)  # Prepared, read-only values.
    print(unit.bottom_width)  # Prepared unit geometry is also read-only.

    while simulation.step() is not None:
        runtime = unit.snapshot()  # Owned point-in-time runtime values.
        print(runtime)

    simulation.end()
    unit.area = 450.0  # Stable declarations are writable again in Ended.
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - LidControl
            - LidSurfaceLayer
            - LidSoilLayer
            - LidStorageLayer
            - LidPavementLayer
            - LidDrainLayer
            - LidDrainageMatLayer
            - LidUnitCollection
            - LidUnit
