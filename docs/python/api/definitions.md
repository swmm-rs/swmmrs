# Model definitions

These views expose the definitions behind the model, including aquifers,
snowmelt parameters, curves, and patterns. This is where you look when a node
or subcatchment refers to something configured elsewhere.

Edit aquifer and snowmelt configuration while the owner is `OPEN` or `ENDED`.
Assignments check types and finite numbers immediately, then retain the complete
requested values in project units.

The related checks happen together at `start()`: aquifer ordering and positivity,
snowmelt coefficient ordering and removal-sum limits, pattern and subcatchment
relationships, and native normalization. If preparation fails, both your
requested values and the previous prepared native state remain intact. Repair
the declaration and retry without starting the whole project over.

```python
from swmmrs import ConfigurationError, Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    aquifer = simulation.aquifers["UPPER"]
    aquifer.update(
        porosity=0.45,
        field_capacity=0.30,
        wilting_point=0.12,
        upper_evaporation_pattern=simulation.time_patterns["MONTHLY-EVAP"],
    )

    snowmelt = simulation.snowmelt_sets["SNOW-1"]
    snowmelt.update(plowable_fraction=0.25, plow_depth=0.5)
    snowmelt.impervious.update(
        minimum_melt_coefficient=0.05,
        maximum_melt_coefficient=0.01,  # Retained for validation at start().
    )

    try:
        simulation.start()
    except ConfigurationError as error:
        for diagnostic in error.diagnostics:
            print(diagnostic.property_path, diagnostic.message)
        snowmelt.impervious.maximum_melt_coefficient = 0.20
        simulation.start()
```

Curves, time series, patterns, land uses, pollutants, controls, transects,
custom shapes, streets, inlet designs, and other generated
model definitions are generation-bound read-only views unless this page
explicitly documents an edit operation. A time pattern may be selected by an
aquifer, but this interface does not edit the pattern itself.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    print(tuple(simulation.curves))
    curve = simulation.curves["PUMP-CURVE"]
    first_pattern = simulation.time_patterns.by_index(0)

    aquifer = simulation.aquifers["UPPER"]
    pattern = aquifer.upper_evaporation_pattern
    print(curve.id, first_pattern.id)
    if pattern is not None:
        print("selected evaporation pattern:", pattern.id)
```

See [Inspect model definitions](../../guides/inspect-model-definitions.md) for
collection names, lookup behavior, relationship traversal, and the editable
versus read-only matrix.

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - Aquifer
            - ControlRule
            - Curve
            - CustomShape
            - LandUse
            - Pollutant
            - SnowmeltParameterSet
            - SnowmeltSurface
            - Street
            - TimePattern
            - Transect
            - InletDesign
