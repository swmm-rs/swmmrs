# Troubleshooting

Start with the exception type. A rejected argument and a failed solver run need
different responses. `ValidationError` and `LifecycleError` leave a healthy
owner reusable. `ConfigurationError` keeps your requested declarations so you
can repair them. `SolverError` reports an operational native failure.

## Common failures

| Symptom | Cause and action |
| --- | --- |
| `import swmmrs` fails | Install a supported released wheel in a clean environment. |
| Construction raises `SolverError` | The solver could not open or validate the input or its referenced files. Inspect `error.code`, `error.operation`, and `error.detail`. |
| Paths raise `ValidationError` | The native owner requires retained input, report, output, checkpoint, and configured hotstart destinations to be distinct where applicable. Byte paths are not accepted. Relative paths resolve against the working directory at call time. |
| `simulation.output_path` is `None` | No output path was supplied, so the simulation owns a scratch binary artifact. Pass an explicit path to retain the `.out` file. |
| `report()` raises `LifecycleError` | Ensure the run started with `save_results=True`, then call `end()`. Natural iterator exhaustion leaves the run in `COMPLETE`, just like manual completion. |
| Statistics are unavailable after `end()` | Statistics acquisition is valid only in `RUNNING` or `COMPLETE`. Copy final statistics after manual or iterator completion and before `end()`. |
| A node or collection raises `StaleViewError` | It belongs to a closed or replaced project generation. Reacquire it from the current `Simulation`. |
| `step()` or `stride()` raises `LifecycleError` | Call `start()` first and do not mix manual advancement with an active iterator. |
| A time or stride value raises `ValidationError` | Use a timezone-naive, whole-second `datetime`, or a positive whole-second stride. Booleans, fractions, zero, negatives, NaN, and infinity are rejected. |
| EPA hotstart loading raises `SolverError` | Use a complete `.hsf` produced by a structurally compatible model. Close the generation if the failure established `FAILED`. |
| `start()` raises `ConfigurationError` | Read the traceback lines or iterate `error.diagnostics`, repair the retained declarations, and retry. |
| Simulation Checkpoint resume fails | Keep the manifest and sidecars together, use fresh destinations, and inspect the typed error. See the checkpoint guide. |
| CPU remains active while host code waits | Dynamic Wave workers stay ready between callbacks. Call `sleep_workers()` while `RUNNING` before slow external work. |
| Concurrent runs overwrite artifacts | Assign each `Simulation` unique report and output paths. Do not advance one owner from several threads. |

## Inspect solver failures

```python
from swmmrs import Simulation, SolverError

try:
    Simulation("model.inp", "model.rpt", "model.out").execute()
except SolverError as error:
    print("code:", error.code)
    print("operation:", error.operation)
    print("detail:", error.detail)
    raise
```

Native code supplies the exception type and structured fields directly. Python
does not have to guess what happened from the wording of an error message.
Validation, lifecycle, and configuration rejections leave a healthy owner
reusable. If an operational failure puts the generation in `FAILED`, preserve
the original error and close it. Do not continue stepping.

## Check the current state

```python
print(simulation.state)
print(simulation.input_path)
print(simulation.report_path)
print(simulation.output_path)
```

Use [Errors and recovery](concepts/errors-and-recovery.md) for retry policy and [Lifecycle and ownership](concepts/lifecycle.md) to choose the next valid operation. Use [Compatibility and limitations](compatibility.md) before treating a missing capability as a local installation problem.
