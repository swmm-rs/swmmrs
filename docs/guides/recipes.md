# Recipes

These examples use the standard library and the public `swmmrs` API. Replace object IDs and paths with values from the model being run.

## Export selected values to CSV

Collect only the fields needed at the host callback cadence:

```python
import csv
from datetime import timedelta

from swmmrs import Simulation

with (
    Simulation("model.inp", "model.rpt", "model.out") as simulation,
    open("results.csv", "w", newline="") as result_file,
):
    writer = csv.writer(result_file)
    writer.writerow(("time", "node_depth", "link_flow"))
    node = simulation.nodes["J1"]
    link = simulation.links["C1"]
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        writer.writerow((current_time.isoformat(), node.depth, link.flow))

    simulation.end()
    simulation.report()
```

Keep the model's `flow_units` and `unit_system` beside exported columns.

## Run independent scenarios concurrently

Each worker owns one simulation and unique artifacts:

```python
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from swmmrs import Simulation


def run_scenario(item: tuple[int, Path]) -> Path:
    scenario_index, input_path = item
    report_path = Path("runs") / f"scenario-{scenario_index}.rpt"
    output_path = Path("runs") / f"scenario-{scenario_index}.out"
    Simulation(input_path, report_path, output_path).execute()
    return output_path


Path("runs").mkdir(exist_ok=True)
models = [Path("baseline.inp"), Path("design.inp")]
with ThreadPoolExecutor(max_workers=len(models)) as executor:
    outputs = list(executor.map(run_scenario, enumerate(models)))
```

Budget Python workers together with the `THREADS` setting inside each Dynamic Wave model to avoid oversubscribing the machine.

## Retain final continuity statistics

Manual completion exposes final statistics before `end()`:

```python
from swmmrs import Simulation

simulation = Simulation("model.inp", "model.rpt")
try:
    simulation.start(save_results=False)
    while simulation.step() is not None:
        pass

    statistics = simulation.statistics
    routing_error = statistics.routing_totals.continuity_error
    runoff_error = statistics.runoff_totals.continuity_error
    simulation.end()
finally:
    simulation.close()

print(routing_error, runoff_error)
```

Apply project-specific acceptance limits in host code; the copied record remains valid after close.

## Sample water quality for selected nodes

```python
from datetime import timedelta

from swmmrs import Simulation

records = []
with Simulation("quality.inp", "quality.rpt") as simulation:
    simulation.step_advance(timedelta(minutes=15))
    for current_time in simulation:
        quality = simulation.nodes.quality_snapshot(["J1", "J2"])
        records.append(
            (
                current_time,
                quality.object_ids,
                quality.pollutant_ids,
                quality.concentrations,
            )
        )
```

Keep `object_ids` and `pollutant_ids` with the pollutant-major matrix so every value retains its identity and reporting-unit context.

See [Runtime forcings](runtime-forcings.md) for control, calibration, and sensitivity patterns.
