# Subcatchments

Edit stable configuration through `SubcatchmentSettings`. After a successful
setter or `update()` call, the Live View returns your requested value
immediately. The solver rebuilds surface, relationship, infiltration,
groundwater, snowpack, loading, and coverage records only when
`Simulation.start()` succeeds.

Type, identity, finiteness, range, and unit errors fail at the setter. Conflicts
with other declarations wait until preparation. An outlet cycle, a LID area
conflict, or a ground surface below the declared water table makes
`Simulation.start()` raise `ConfigurationError` with ordered diagnostics.
Your accepted declarations remain readable. Correct them and retry without
reopening the project.

Tags and report inclusion are metadata and do not mark configuration dirty.
Precipitation scaling and external inputs are forcing, separate from stable
declarations.

`SubcatchmentSettings.infiltration` is `None` when the input has no infiltration
declaration. A present declaration returns a generation-bound live view.

```python
from swmmrs import ConfigurationError, Simulation
from swmmrs.objects import HortonInfiltrationSettings

with Simulation("model.inp", "model.rpt") as simulation:
    subcatchment = simulation.subcatchments["S-1"]
    settings = subcatchment.settings
    settings.update(
        area=12.5,
        width=250.0,
        outlet=simulation.nodes["J-1"],
        infiltration=HortonInfiltrationSettings(
            initial_rate=3.0,
            minimum_rate=0.5,
            decay_coefficient=4.0,
            drying_time_days=7.0,
        ),
    )
    assert settings.width == 250.0  # Requested declaration reads back now.

    # Cross-field errors are retained until preparation.
    groundwater = settings.groundwater
    if groundwater is not None:
        groundwater.surface_elevation = groundwater.water_table_elevation - 1.0

    try:
        simulation.start()  # Validate relationships and rebuild derived state.
    except ConfigurationError as error:
        for diagnostic in error.diagnostics:
            print(diagnostic.property_path, diagnostic.message)
        if groundwater is not None:
            groundwater.surface_elevation = groundwater.water_table_elevation
        simulation.start()  # Repair and retry without reopening.
```

Changing `SubcatchmentSettings.rain_gage` also rebuilds configured rain-gage
usage, duplicate time-series sharing, and effective wet/dry/routing steps at the
next successful `start()`. The setting continues to return the requested gage
while dirty or after failed preparation. Persistent external precipitation and
rainfall overrides remain runtime forcing and take precedence without changing
the configured relationship.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    subcatchment = simulation.subcatchments["S-1"]
    gage = simulation.rain_gages["GAGE-2"]
    subcatchment.settings.rain_gage = gage
    assert subcatchment.settings.rain_gage == gage

    # Forcing changes effective rain without replacing the configured relationship.
    gage.use_external_precipitation(1.25)
    gage.rainfall_override = 0.0
    gage.rainfall_override = None  # Resume the external source.
```

`SubcatchmentSettings.initial_buildup` and `coverage_fractions` are canonical
live mappings. Item assignment and `update()` are atomic sparse mutations;
item deletion resets one configured value to zero. `clear()` resets every
configured value to zero, while whole-property assignment completely replaces
the mapping and resets omitted identities to zero.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    settings = simulation.subcatchments["S-1"].settings
    settings.initial_buildup["TSS"] = 2.5
    settings.initial_buildup.update({"Lead": 0.10})
    del settings.initial_buildup["Lead"]  # Reset Lead to zero.

    settings.coverage_fractions.update({"Residential": 0.6, "Commercial": 0.4})
    settings.coverage_fractions.clear()  # Reset all configured land uses to zero.

    # Whole-property assignment replaces all values; omitted pollutants become zero.
    settings.initial_buildup = {"TSS": 4.0}
```

Runtime forcing remains top-level. `external_rainfall` and
`external_snowfall` are additive persistent scalar properties.
`set_precipitation_scale_factors(rainfall=..., snowfall=...)` atomically changes
both coupled gage multipliers without reading a sibling value in Python.
`external_pollutant_buildup_increment` is a separate persistent live mapping:
item assignment and `update()` are sparse atomic writes, deletion resets one
pollutant to zero, and `clear()` resets all configured pollutants. Unlike the
stable settings mappings above, whole-property assignment is unsupported.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    subcatchment = simulation.subcatchments["S-1"]
    subcatchment.external_rainfall = 0.25
    subcatchment.external_snowfall = 0.0
    subcatchment.set_precipitation_scale_factors(rainfall=1.10, snowfall=0.80)

    buildup = subcatchment.external_pollutant_buildup_increment
    buildup["TSS"] = 1.0
    buildup.update({"Lead": 0.05})
    del buildup["Lead"]
    buildup.clear()
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - SubcatchmentCollection
            - Subcatchment
            - SubcatchmentSettings
            - SubcatchmentInfiltration
            - HortonInfiltrationSettings
            - ModifiedHortonInfiltrationSettings
            - GreenAmptInfiltrationSettings
            - ModifiedGreenAmptInfiltrationSettings
            - CurveNumberInfiltrationSettings
            - SubcatchmentGroundwater
            - SubcatchmentGroundwaterSettings
