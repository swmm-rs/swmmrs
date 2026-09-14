# Rain gages

Call `use_external_precipitation(rate)` to make your application the persistent
precipitation source. It selects the API source, marks the gage used, and
unlinks any co-gage. Read `external_precipitation_rate` to check the selected
value.

Use `rainfall_override` to mask the underlying source with a persistent scalar
value. Assigning `None` clears that mask and resumes the source. Assigning zero
means dry weather; it does not hand control back.

```python
from swmmrs import Simulation

with Simulation("model.inp", "model.rpt") as simulation:
    gage = simulation.rain_gages["GAGE-1"]

    # Make the host application the persistent precipitation source.
    gage.use_external_precipitation(1.25)
    assert gage.external_precipitation_rate == 1.25

    # Temporarily mask that source, including with an explicitly dry rate.
    gage.rainfall_override = 2.0
    gage.rainfall_override = 0.0
    gage.rainfall_override = None  # Resume the 1.25 external source.

    simulation.start()
    while simulation.step() is not None:
        print(gage.rainfall, gage.snowfall, gage.total_precip)
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - RainGage
