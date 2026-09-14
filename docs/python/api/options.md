# Simulation options

Use `SimulationOptions` to configure numerical and analysis choices while the
simulation is `OPEN` or `ENDED`. Individual assignments and
`options.update(**changes)` use the same atomic native update. Rust checks each
value locally, retains the complete requested configuration, and records what
needs preparation. The effective solver fields wait until `start()`.

This gives you room to edit related values in several calls. A schedule or step
relationship can be temporarily inconsistent while you work. Reads return your
requested values, including after a failed start. At `start()`, the solver
prepares effective defaults, step coupling, flags, threads, and dependent
hydraulic geometry, then installs them together if preparation succeeds.

If preparation fails, `start()` raises `ConfigurationError` with ordered
diagnostics and leaves the prior effective state intact. Repair the requested
values and retry. `requested_threads` is what you asked for;
`simulation.effective_threads` is the prepared worker count.
Assigning `custom_ellipse_model` while `OPEN` or `ENDED` marks configuration
dirty. The next `start()` revalidates and atomically rebuilds affected cross sections.


Update related dates together with `Simulation.update_schedule(**changes)`.
Omitted dates keep their requested values. The setter checks that dates are
finite and representable. At `start()`, the solver checks start/end ordering,
positive duration, whether the report start falls at or after the end, and
report step coupling. A report start earlier than the requested start is normalized
to the effective start instead of producing a diagnostic. Direct date setters
use the same operation.

```python
from datetime import datetime, timedelta

simulation.options.update(
    routing_step=timedelta(seconds=20),
    courant_factor=1.0,
    requested_threads=2,
)
simulation.update_schedule(
    start_time=datetime(2025, 1, 1),
    report_start=datetime(2025, 1, 1, 1),
    end_time=datetime(2025, 1, 2),
)
```

::: swmmrs.objects
    options:
        show_root_heading: false
        members:
            - SimulationOptions
