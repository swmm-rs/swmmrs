# Time series

Look up a `TimeSeries` in `simulation.time_series` by configured ID or position,
then pass it to a setting that accepts a time-series relationship. The view is
read-only and belongs to the current project generation. It represents the
configured definition.

```python
from swmmrs import Simulation
from swmmrs.objects import Outfall, OutfallBoundary

with Simulation("model.inp", "model.rpt") as simulation:
    series = simulation.time_series["OUTFALL-STAGE"]
    print(series.id, series.index)
    assert simulation.time_series.by_index(series.index) == series

    outfall = simulation.nodes["OUT-1"]
    if isinstance(outfall, Outfall):
        outfall.settings.boundary = OutfallBoundary(
            "timeseries",
            reference=series,
        )
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - TimeSeries
