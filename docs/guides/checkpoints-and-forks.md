# Simulation checkpoints and forks

Simulation Checkpoints preserve a fuller swmmrs continuation boundary than an
EPA hotstart. Use them to resume a running owner in another process, create an
independent in-process branch, or import physical state into a compatible open
receiver.

These operations are distinct from [`save_hotstart()` and
`use_hotstart()`](hotstarts.md), which read and write EPA's `.hsf` format.

## Choose an operation

| Need | Operation | Result |
| --- | --- | --- |
| Publish a durable continuation boundary | `simulation.save_checkpoint(path)` | JSON manifest and immutable sidecars beside it |
| Continue that artifact as an independent owner | `Simulation.resume(...)` | Restored `RUNNING` or `COMPLETE` owner with fresh report/output paths |
| Branch immediately in the same process | `simulation.fork(...)` | Independent child with cloned editable declarations |
| Enhanced Hotstart into a parsed compatible model | `receiver.load_checkpoint_state(path)` | Receiver stays `OPEN`, imports classified warm state, and retains its dates, forcing, inputs, accounting, and outputs |

Save, fork, and resume operate at quiescent `RUNNING` or `COMPLETE` boundaries.
They may be called between iterator advances after `next()` returns.

## What each mechanism restores

The three durable mechanisms answer different modeling questions. In the table,
**source** means that the value recorded at the saved boundary is restored;
**receiver** means that the newly opened model keeps or initializes its own
value. **Selected** means that the artifact carries only the listed state, not
the complete state of that subsystem.

| Model state or run property | EPA Hotstart (`.hsf`) | Enhanced Hotstart (`load_checkpoint_state`) | Checkpoint Resume (`resume`) |
| --- | --- | --- | --- |
| Model declarations, object identities, and topology | **Receiver.** The caller supplies a compatible input model; identity and complete topology are not verified. | **Receiver.** Ordered structure, units, routing family, pollutants, subsystem dimensions, and imported-value domains must be compatible. | **Source.** Authenticated declarations, prepared configuration, and native relationships are restored. |
| Simulation dates, elapsed clocks, and lifecycle boundary | **Receiver.** A new run starts on the receiver schedule. | **Receiver.** A new run starts on the receiver schedule; imported ages and deadlines are translated to that schedule. | **Source.** The saved `RUNNING` or `COMPLETE` boundary and its clocks are restored. |
| Node and link hydraulic condition | **Selected.** Current node depth and lateral flow, storage hydraulic residence time, and current link flow, depth, and setting are restored. | **Source warm state.** Classified old/new node, link, conduit, storage, and inlet condition is imported. | **Source continuation.** The complete checkpoint hydraulic projection is restored. |
| Subcatchment surface and runoff state | **Selected.** Subarea ponded depth and current runoff are restored. | **Source warm state.** Surface storage and runoff lag state are imported; source rainfall and shared runoff-step totals are excluded. | **Source continuation.** Runoff state, flags, totals, and progress are restored. |
| Infiltration memory | **Source warm state.** The active Horton, Modified Horton, Green-Ampt, Modified Green-Ampt, or Curve Number state is restored. | **Source warm state.** Active infiltration memory is imported and checked against receiver parameters; shared runoff-step accumulators are excluded. | **Source continuation.** Method configuration, state, and balances are restored. |
| Groundwater and snowpack condition | **Selected source state.** Groundwater-zone and snow-surface state are restored where configured. | **Source warm state.** Compatible groundwater and snow physical condition is imported; receiver-derived limits and calendar selectors remain authoritative. | **Source continuation.** Configuration, physical state, coupling state, and accounting are restored. |
| LID condition | **Receiver.** EPA hotstart files do not carry LID layer or clogging state. | **Source warm state.** Layer state, clogging, regeneration timing, and embedded Green-Ampt memory are imported; LID rates, balances, and report history start fresh. | **Source continuation.** LID physical state, balances, and detailed-report continuation are restored. |
| Water-quality condition | **Selected.** Current node, link, runoff, and ponded quality plus surface buildup and street-sweeping date are restored. | **Source warm state.** Compatible concentrations, reactor state, and physical buildup stocks are imported; API forcing, cumulative loads, and quality accounting are excluded. | **Source continuation.** Quality configuration, treatment state, concentrations, loads, balances, and statistics are restored. |
| Control-rule and PID numerical memory | **Receiver.** A saved link setting is applied, but rule evaluation history, PID memory, and routing-event progress are not carried. | **Source warm state.** Compatible PID and control memory is imported; rule-clock phase is translated and receiver event position is recomputed. | **Source continuation.** Control structure, PID memory, action history, and routing-event position are restored. |
| Dynamic Wave numerical memory | **Receiver initialization.** Saved current hydraulic values seed an ordinarily initialized solver. | **Selected source memory.** Indispensable variable-step and node/link continuation is imported; process-local workers and scratch state are rebuilt. | **Source continuation.** Durable Dynamic Wave continuation is restored and process-local workspace is rebuilt. |
| Persistent API forcing | **Receiver.** The new owner supplies its own forcing. | **Receiver.** All nine forcing families remain receiver-owned and can be edited after State Load. | **Source.** All nine forcing families are restored from the checkpoint. |
| Rain, climate, time-series, RDII, and runoff/routing interface progress | **Receiver initialization.** Input processors start according to the receiving model. | **Receiver initialization.** Checkpoint cursors, lookahead, residual source-input response, and sidecars are not imported. | **Source continuation.** Resource modes, validated dependencies or sidecars, cursors, and parser lookahead recorded by the checkpoint are restored. |
| Continuity accounting, cumulative loads, and statistics | **Fresh receiver history.** | **Fresh receiver history.** | **Source history.** |
| Report, binary output, interface output, and detailed-LID report history | **Fresh receiver files.** | **Fresh receiver files.** Checkpoint sidecars are not read. | **Source prefixes and cursors** are copied into caller-supplied fresh destinations, then continued. |
| Scheduled hotstart progress and output policy | **Receiver.** | **Receiver.** | **Source.** Pending/emitted schedule state is restored at fresh derived destinations. |
| Next-step and whole-run relationship to the saved source | A warm initial condition; uninterrupted equality is not promised. | A richer warm initial condition; first-step and whole-run equality with Resume are not promised. | An uninterrupted-equivalent continuation at the saved quiescent boundary. |

The matrix describes the current `swmmrs` contracts rather than every byte in a
live process. Raw file handles, buffers, worker threads, and scratch workspaces
are never serialized; Resume rebuilds them from durable state. EPA Hotstart has
limited count and flow-unit checks, Enhanced Hotstart performs strict receiver
compatibility and scalar-domain checks, and Resume validates the complete
checkpoint artifact set.

## Save and resume

```python
from pathlib import Path

from swmmrs import Simulation

checkpoint = Path("continuation/state.json")

source = Simulation("model.inp", "source.rpt", "source.out")
source.start()
source.step()
source.save_checkpoint(checkpoint)

resumed = Simulation.resume(
    checkpoint,
    "resumed.rpt",
    "resumed.out",
)

# Both owners are independent and continue from the saved boundary.
source_time = source.step()
resumed_time = resumed.step()

for simulation in (source, resumed):
    simulation.end()
    simulation.close()
```

Published generation-addressed sidecars are immutable. Saving again to the same
manifest destination atomically replaces that manifest with a new generation;
use a unique manifest path for every boundary that must remain retained.
`resume()` requires fresh report and output destinations and rejects collisions
before publishing an owner.

## Fork an in-process scenario

`fork()` avoids a durable round trip and preserves the source owner's prepared
configuration coordinator and editable declarations:

```python
from swmmrs import Simulation

source = Simulation("model.inp", "baseline.rpt", "baseline.out")
source.start()
source.step()

child = source.fork("scenario.rpt", "scenario.out")
child.links["GATE"].target_setting = 0.5

while source.step() is not None:
    pass
while child.step() is not None:
    pass

for simulation in (source, child):
    simulation.end()
    simulation.report()
    simulation.close()
```

Source and child do not share hydraulic state. Give every owner unique mutable
artifact paths.

## Load physical state into an open receiver

Use State Load when the receiver must keep its parsed configuration and later
support declaration editing:

```python
from swmmrs import Simulation

receiver = Simulation("compatible.inp", "receiver.rpt", "receiver.out")
receiver.load_checkpoint_state("continuation/state.json")

# Classified warm state is staged now. Receiver forcing and input processors
# remain authoritative when normal startup initializes the new run.
receiver.start()
while receiver.step() is not None:
    pass
receiver.end()
receiver.report()
receiver.close()
```

The receiver must be `OPEN` and structurally compatible. State Load stages dirty receiver declarations without mutation and accepts them only when they leave State Load structure and scalar interpretation unchanged. It validates the imported physical values, commits preparation and the classified physical/numerical subset together, and retains a pending copy so `start()` can reapply it after processor initialization.

The receiver keeps its dates, all persistent-forcing families, configured
inputs and their progress, iterator cadence, fresh statistics/accounting,
output history, and parsed declarations. Checkpoint sidecars are not read.
Source rain/RDII progress and residual source-input response are not imported.
Enhanced Hotstart makes no first-step or whole-run equality promise with
Checkpoint Resume, even when receiver forcing, dates, and inputs match.

Prefer to complete preparation-dependent edits before loading. If you make one while warm state is pending, the edit is accepted but `start()` reports that the pending State Load is stale. Call `load_checkpoint_state()` again to prepare the receiver and revalidate the import. Persistent-forcing and metadata-only updates do not stale the pending warm state.

## Editable declarations after durable resume

Checkpoint format 1.2 preserves the source owner's editable declarations and
warning baseline as an authenticated payload. The full-resume fingerprint binds
that payload to the checkpoint's canonical state; do not copy configuration
between manifests, even if the models appear structurally compatible. After
`resume()`, you can finish and end the restored run, make stable edits, and start
a fully prepared rerun. State Load remains different: it keeps the open
receiver's parsed declarations.

## Artifacts and failures

A Simulation Checkpoint is not one opaque file. The JSON manifest identifies
immutable sidecars for active report/output prefixes and generated resources,
and validated external dependencies for immutable inputs. Do not move, replace,
or edit members independently.

Publication is atomic from the caller's perspective: the manifest appears only
after its complete generation is usable. Resume and fork preflight all requested
destinations. Destination and preflight failures publish no partial owner and
leave the source unchanged. An operational failure while flushing source-owned
resources can establish `FAILED`, even though no child or partial destination is
published.

`ValidationError` reports Python/path alias mistakes known before native
resource planning; `LifecycleError` reports an invalid owner state;
`SolverError` reports non-fresh destinations, integrity or compatibility
failures, capture, resource validation, and publication failures. See [Errors and
recovery](../python/concepts/errors-and-recovery.md).
