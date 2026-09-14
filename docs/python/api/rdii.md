# RDII objects

The RDII API exposes both EPA SWMM RTK unit hydrographs and the
[`swmmrs` Antecedent Moisture Model (AMM) extension](../../epa-swmm-deviations/solver-features/hydrology/amm-rdii.md).
RTK uses `Simulation.unit_hydrographs` and `Simulation.rdii_assignments`; AMM
uses `Simulation.amm_models` and `Simulation.amm_assignments`. Python can edit
existing models and replace assignment lists, but it cannot create or delete
top-level RTK or AMM models.

Model views stay connected to one project generation. Their parameter and
assignment objects are detached, so editing one is only the first half of the
job. Assign the complete parameter or assignment sequence back to apply it.
Edits are allowed while a simulation is Open or Ended, not while it is Running.
The next `Simulation.start()` validates and prepares the changed configuration.

## RTK unit hydrographs

RTK models come from `[HYDROGRAPHS]`. `UnitHydrograph.rain_gage` and
`monthly_responses` are writable individually or together through `update()`.
The returned tuple contains January through December. Each
`UnitHydrographMonth` exposes named `short`, `medium`, and `long` responses.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    unit_hydrograph = simulation.unit_hydrographs["UH1"]
    responses = unit_hydrograph.monthly_responses

    january_short = responses[0].short
    january_short.rainfall_fraction = 0.15
    january_short.time_to_peak_hours = 0.75
    january_short.recession_ratio = 2.0

    unit_hydrograph.update(
        rain_gage=simulation.rain_gages["GAGE"],
        monthly_responses=responses,
    )
```

| Python field | SWMM parameter | Units |
| --- | --- | --- |
| `rainfall_fraction` | R | Fraction |
| `time_to_peak_hours` | T | Hours |
| `recession_ratio` | K | Ratio |
| `maximum_initial_abstraction` | IAmax | Project rain-depth units |
| `initial_abstraction_recovery_rate` | IArec | Project rain-depth units per day |
| `initial_abstraction_at_start` | IAinit | Project rain-depth units |

Every month must provide short, medium, and long responses. Monthly rainfall
must be nonnegative and sum to no more than 1.01.

### Replace RTK node assignments

`Simulation.rdii_assignments` represents `[RDII]` rows and returns detached
`RdiiAssignment` values in node order. Each node can have at most one RTK
assignment.

```python
from swmmrs import Simulation
from swmmrs.objects import RdiiAssignment

with Simulation("model.inp", "model.rpt") as simulation:
    simulation.rdii_assignments = [
        RdiiAssignment(
            node=simulation.nodes["J1"],
            unit_hydrograph=simulation.unit_hydrographs["UH1"],
            area=3.5,
        )
    ]
```

## Antecedent Moisture Model

AMM models come from `[AMM_MODELS]` and `[AMM_COMPONENTS]`. `AmmModel.update()`
changes the rain gage, temperature bounds, and complete component list
atomically.

```python
from swmmrs import Simulation
from swmmrs.objects import AmmStandardComponent

with Simulation("model.inp", "model.rpt") as simulation:
    model = simulation.amm_models["SAN1"]
    print(model.rain_gage.id, model.cold_temperature, model.hot_temperature)

    components = list(model.components)
    standard = next(
        component
        for component in components
        if isinstance(component, AmmStandardComponent)
    )
    standard.hot_shcf = 0.06

    model.update(
        cold_temperature=25.0,
        hot_temperature=75.0,
        components=components,
    )
```

Temperatures use the project's temperature units. Component averaging windows
and half-lives use hours. See the
[AMM input reference](../../epa-swmm-deviations/solver-features/hydrology/amm-rdii.md#input-sections)
for every AMM parameter's meaning and units.

### Replace AMM node assignments

`Simulation.amm_assignments` represents `[AMM_RDII]` rows. Unlike RTK, multiple
AMM assignments may target one node.

```python
from swmmrs import Simulation
from swmmrs.objects import AmmAssignment

with Simulation("model.inp", "model.rpt") as simulation:
    assignments = list(simulation.amm_assignments)
    assignments.append(
        AmmAssignment(
            node=simulation.nodes["J2"],
            model=simulation.amm_models["SAN1"],
            area=3.5,
        )
    )
    simulation.amm_assignments = assignments
```

RTK and AMM assignment areas use acres in US projects and hectares in SI
projects.

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - UnitHydrograph
            - UnitHydrographMonth
            - UnitHydrographResponse
            - RdiiAssignment
            - AmmModel
            - AmmStandardComponent
            - AmmBaseflowComponent
            - AmmAssignment
