# Install and run your first model

Install the published `swmmrs` wheel from PyPI. It includes the native solver
and has no runtime Python dependencies, so installation does not begin with a
side quest through a compiler toolchain. The solver source is private; the
wheel is the supported installation route for public users.

```bash
python -m pip install swmmrs
```

Import the public `swmmrs` interface in your code. The package-private native
extension comes with it.

## Run a complete simulation

The repository includes a small ten-minute model at `tests/data/snapshots.inp`. From the `python` directory, save this as `first_model.py`:

```python
from pathlib import Path

from swmmrs import Simulation, SimulationState

report_path = Path("first-model.rpt")
output_path = Path("first-model.out")
simulation = Simulation("tests/data/snapshots.inp", report_path, output_path)
simulation.execute()

assert simulation.state is SimulationState.CLOSED
assert report_path.is_file()
assert output_path.is_file()
```

Run it:

```bash
python first_model.py
```

If the script finishes without an assertion error, both result files exist and
the project is closed. `execute()` handles the full run: start, route to
completion, end, write the detailed report, and close.

## Inspect values while the model runs

If you want a say in what happens along the way, use iteration. Python can read
or change the model between routing advances:

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation(
    "tests/data/snapshots.inp",
    "interactive.rpt",
    "interactive.out",
) as simulation:
    node = simulation.nodes["J1"]
    conduit = simulation.links["C1"]
    simulation.step_advance(timedelta(minutes=1))

    for current_time in simulation:
        print(current_time, node.depth, conduit.flow)

    simulation.end()
    simulation.report()
```

Values use the unit system configured by the input model. `step_advance()` defaults to `strict=True`, returning control at exact cadence boundaries and potentially shortening the final routing step. Use `step_advance(..., strict=False)` or `stride(..., strict=False)` in a caller-owned loop when routing steps must match repeated `step()` calls.

## Next steps

- [Choose a run workflow](workflows.md) before building a host application.
- [Configure a model](../guides/configure-model.md) before its first run.
- [Collect results and statistics](../guides/collect-results.md) during or after each routing advance.
- [Read binary output](../guides/read-output.md) after routing stops, including complete records from an incomplete file.
- [Review compatibility and limitations](compatibility.md) before production evaluation.
