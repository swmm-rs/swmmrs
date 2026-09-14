# Continuation and branching

A continuation mechanism answers: **which parts of this run should become the
starting point for another run?** `swmmrs` supports four related mechanisms.
They are intentionally different because hydraulic state, model declarations,
run clocks, forcing, accounting, and file progress have different meanings.

## Choose by modeling intent

| Need | Use | Main interpretation |
| --- | --- | --- |
| Exchange warm hydraulic conditions with EPA SWMM or another compatible implementation | EPA hotstart | A portable SWMM physical-state handoff into a new compatible run. |
| Pause a swmmrs run and continue its fuller runtime and resource history later | Save checkpoint, then resume | A durable continuation of the same swmmrs run boundary with fresh destination files. |
| Split one active simulation into independent futures in the same process | Fork | A scenario branch that copies the current boundary and editable declarations without a disk round trip. |
| Apply saved warm conditions to a separately opened compatible model while keeping that receiver's dates, forcing, and accounting | Enhanced Hotstart through Checkpoint State Load | An initial-condition transfer, not a continuation of the source's run history. |

## EPA hotstart

An EPA hotstart (`.hsf`) stores selected dynamic SWMM state. It is appropriate
for familiar H&H workflows such as:

- warm-up followed by a forecast period;
- initializing several compatible alternatives from common antecedent
  conditions;
- exchanging state with software that understands the EPA format.

The receiving simulation remains a new run. It owns its own dates, report,
output, forcing policy, and accounting. A hotstart does not preserve Python
objects, report/output progress, or the complete swmmrs runtime environment.
Compatibility checking is limited; keep the hotstart paired with the exact
compatible input model and object ordering used to create it.

Save a hotstart while the source is `RUNNING` or `COMPLETE`, before `end()`.
Within one opened project generation, a hotstart selected with `use_hotstart()`
remains selected for later starts until it is explicitly cleared. Clear it before
a cold-start comparison; otherwise a repeated run will warm-start again.

## swmmrs checkpoint and resume

A checkpoint is a fuller swmmrs continuation boundary. It is intended to
continue the current run rather than merely warm-start another model. It can
preserve physical and numerical continuation, clocks, accounting, statistics,
persistent API forcing, controls, and the completed portions and positions of
managed resources.

A checkpoint is an artifact set: a JSON manifest plus any required sidecars and
references to validated external inputs. Treat the set as one unit. Do not edit,
replace, or move individual members independently.

`resume()` creates a new independent simulation at the stored `RUNNING` or
`COMPLETE` boundary. It requires fresh report and output destinations. The
source simulation and resumed simulation do not share future hydraulic state.

Checkpoint format 1.2 restores prepared state, runtime continuation, and the
authenticated editable-declaration projection. The full-resume fingerprint binds
that projection to the checkpoint's canonical state, so it cannot be substituted
from another artifact. The resumed owner can be ended, reconfigured, prepared,
and rerun.

## In-process fork

A fork creates an independent child at the parent's current quiescent boundary.
Parent and child begin from the same modeled condition, but later steps,
controls, forcing, and results are separate.

Fork is the natural choice for a decision tree evaluated inside one application:
advance to a forecast issue time, fork one child per operating strategy, then run
the branches independently. It preserves editable declarations, but it is not a
durable artifact for a later process. Each branch needs unique mutable output
paths.

## Checkpoint State Load

State Load opens a different modeling question: **what if this compatible model
started from the saved physical condition?**

The receiving simulation keeps its own:

- dates and simulation clocks;
- input declarations and editable configuration;
- statistics and continuity accounting ownership;
- report and binary-output history;
- external-input and output-resource policy.

It imports the compatible warm physical subset and indispensable numerical
memory from the checkpoint. The receiver keeps all nine persistent-forcing
families, including rain-gage sources, rain/snow additions, node and link
forcing, outfall stage overrides, and pollutant forcing. It also starts its own
rain, climate, RDII, report, output, accounting, and statistics processors.

Elapsed ages are preserved, source-calendar timestamps are rebased to the
receiver calendar, and receiver-calendar selectors are recomputed during
ordinary receiver execution. Source RDII progress and residual source-input response are
not transferred. The receiver stays `OPEN`, then reapplies the staged warm
condition after normal initialization at `start()`.

Prefer to finish preparation-dependent scenario edits before State Load. Loading can stage and commit dirty edits that do not change State Load interpretation. A later preparation-dependent edit is accepted but makes the pending warm state stale, so State Load must be repeated before `start()`. Receiver forcing can still be changed without reloading the warm state.

This distinction is useful for data-assimilation-like workflows, forecast
cycling, or transferring initial conditions into a separately managed run. It
does not resume the source run's accounting history or promise the same next
step as Checkpoint Resume.

## Side-by-side interpretation

For a subsystem-level comparison of exactly what comes from the source and what
remains receiver-owned, see the [EPA Hotstart, Enhanced Hotstart, and Checkpoint
Resume restoration matrix](../guides/checkpoints-and-forks.md#what-each-mechanism-restores).

| Question | EPA Hotstart | Checkpoint Resume | Fork | Enhanced Hotstart |
| --- | --- | --- | --- | --- |
| EPA-compatible format? | Yes | No | No | No |
| Durable for another process? | Yes | Yes | No | Yes, using checkpoint artifact |
| Continues source clocks/accounting? | No | Yes | Yes | No |
| Preserves fuller swmmrs forcing/resource progress? | No | Yes | Yes | No; receiver forcing and resources start normally |
| Retains editable declarations for later scenario edits? | Receiver supplies them | Yes | Yes | Receiver supplies them |
| Produces an independent owner? | New receiver | Yes | Yes | Existing receiver remains independent |

## Safe-boundary expectations

Checkpoint save and fork are performed between routing advances at a quiescent
`RUNNING` or `COMPLETE` boundary. They do not split an in-progress hydraulic
calculation. Saving a checkpoint or hotstart is explicit; neither mechanism is
automatic crash recovery for work that was never successfully published.

For operational examples, see [Simulation checkpoints and forks](../guides/checkpoints-and-forks.md)
and [EPA hotstarts and chained runs](../guides/hotstarts.md).
