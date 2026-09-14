# Errors and recovery

An exception does not always mean the simulation is beyond repair. Start with
the narrowest exception type: it tells you what was rejected and whether you
can keep using the current Simulation Owner.

## State effects

| Failure | Typical cause | Owner effect | Recovery |
| --- | --- | --- | --- |
| `ValidationError` | Invalid type, value, time, path, or atomic candidate | No accepted mutation; lifecycle remains reusable | Correct the call and retry. |
| `LifecycleError` | Valid operation in the wrong owner state or conflicting advancement owner | Rejected without changing lifecycle | Move to the required state or stop mixing advancement styles. |
| `ConfigurationError` | Deferred relational or cross-object preparation failure | Requested declarations and dirty state remain; prior prepared state is unchanged | Inspect diagnostics, repair declarations, retry `start()`. |
| `StaleViewError` | Collection or Live View belongs to an earlier Project Generation | Current owner is unaffected | Reacquire the object from the current `Simulation`. |
| `KeyError` / `IndexError` | Unknown configured identity or invalid index | Current owner is unaffected | Correct the lookup. |
| `SolverError` | Native open, routing, finalization, resource, or checkpoint failure | An operational failure can move the generation to `FAILED` | Preserve the error; if state is `FAILED`, close the generation. |
| `InternalSimulationError` | Binding/native contract inconsistency | Continued use is not supported | Preserve details, close if possible, and report a bug. |
| `swmmrs.output.OutputError` | Invalid metadata, corrupt finalized output, incompatible schema, or invalid query | Independent of any live `Simulation` lifecycle | Correct the file/query or regenerate the output. |

With `ValidationError`, `LifecycleError`, or a repairable `ConfigurationError`,
a healthy generation stays healthy. Correct the problem and reuse the owner.
There is no need to evacuate the whole project over one rejected value.

## Inspect configuration diagnostics

A value can make sense on its own and disagree with the rest of the model.
`start()` collects those conflicts into ordered diagnostics in one
`ConfigurationError`. You can read them in the traceback or inspect the typed
records in `.diagnostics`:

```python
from swmmrs import ConfigurationError

try:
    simulation.start()
except ConfigurationError as error:
    for diagnostic in error.diagnostics:
        identity = diagnostic.object
        print(identity.object_type, identity.id, identity.index)
        print(diagnostic.property_path)
        print(diagnostic.rule_code)
        print(diagnostic.message)
        if diagnostic.conflicting_object is not None:
            print("conflicts with:", diagnostic.conflicting_object)

    # Requested declarations remain readable. Repair them, then retry start().
```

For example, a subcatchment groundwater surface elevation below its initial
water-table elevation reports
`subcatchment.groundwater.elevation` against
`groundwater.surface_elevation`. Repair the subcatchment groundwater declaration
rather than treating the shared aquifer's fields as instance-owned values.

A failed preparation is atomic:

- requested declarations remain authoritative;
- `simulation.configuration_dirty` remains true;
- the prior effective solver projection is unchanged;
- unrelated runtime, forcing, statistics, and continuation state are retained;
- repair and retry are supported.

## Inspect operational solver failures

```python
from swmmrs import Simulation, SimulationState, SolverError

simulation = Simulation("model.inp", "model.rpt", "model.out")
try:
    simulation.execute()
except SolverError as error:
    print("code:", error.code)
    print("operation:", error.operation)
    print("detail:", error.detail)
    if simulation.state is SimulationState.FAILED:
        simulation.close()
    raise
```

Preserve the first operational error. Cleanup may attach secondary details but
must not replace the original failure. Do not continue stepping a `FAILED`
generation.

## Context manager cleanup

If your code raises inside a context manager, that exception keeps priority.
A cleanup failure is attached as a note so the original problem remains visible.
If your code succeeds and cleanup fails, the cleanup exception is raised
normally. The context manager closes resources. Request the detailed report
explicitly if you need one.

## Related guidance

- [Lifecycle and ownership](lifecycle.md)
- [Configure a model](../../guides/configure-model.md)
- [Troubleshooting](../troubleshooting.md)
- [Exception API](../api/exceptions.md)
- [Binary output errors](../api/output.md)
