# Object collections

Use `ObjectCollection` to look up configured project definitions by name or
position. String lookup is case-insensitive, iteration follows configured
project order, and `by_index()` accepts a zero-based position. Collections and
their objects are Live Views tied to the current project generation. Reopening
a project means getting fresh views.

```python
from swmmrs import Simulation, StaleViewError

simulation = Simulation("model.inp", "model.rpt")
curves = simulation.curves
print(tuple(curves))  # Configured IDs in project order.
curve = curves["pump-curve"]  # Case-insensitive ID lookup.
assert curves.by_index(0).id == tuple(curves)[0]
assert "PUMP-CURVE" in curves

simulation.close()
try:
    print(curve.id)
except StaleViewError:
    print("Reopen the project and reacquire the collection and object views")
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - ObjectCollection
