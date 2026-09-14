# Software architecture

Two simulations in one process should be able to mind their own business.
`swmmrs` changes how EPA SWMM state, execution resources, and integrations are
owned so they can do exactly that. These changes leave the default hydraulic
and hydrologic equations intact.

For alternatives that intentionally change results, see
[solver features](solver-features/index.md).

## Architecture differences at a glance

| Area | EPA SWMM | swmmrs | Expected effect on model results |
| --- | --- | --- | --- |
| State ownership | Process-wide mutable solver state | One Simulation Owner contains each model and run | Independent simulations do not share solver state |
| Execution | Upstream execution and storage design | Selected internal work, storage, and execution paths are reorganized | No intentional numerical change; sensitive operation order remains deterministic |
| Continuation | EPA hotstart files preserve selected physical state | EPA hotstarts plus checkpoints, resume, forks, and State Load | New workflows; EPA hotstart behavior remains available |
| Interfaces | Executable and C toolkit | Command-line runner, Rust owner, typed Python API, live views, snapshots, and finalized-output reader | Integration changes, not alternate default equations |
| Failure handling | Global status, integer codes, and caller-managed resources | Explicit lifecycle checks, owned resources, structured errors, and stale-view detection | Invalid operations can fail earlier and more specifically |
| Packaging | Native EPA executable or shared library | Native Rust binaries and a private Python extension | Deployment boundary changes only |

## Isolated simulation state

EPA SWMM historically stores model objects, options, files, counters, and
runtime values in process-wide mutable state. `swmmrs` places that information
inside a Simulation Owner.

With each simulation responsible for its own belongings, you can:

- keep baseline and alternative scenarios open at the same time;
- run independent simulations concurrently without a shared solver lock;
- contain lifecycle and resource failures within one simulation;
- give every live object view a specific project identity; and
- branch a scenario without letting parent and child share future state.

!!! warning "System Resources Matter"
    The simulations still inhabit the same computer. Give concurrent runs unique
    output paths and sensible CPU and memory budgets. Owning your state does not
    entitle you to all the RAM.

## Performance-oriented implementation changes

Some work can be avoided. Some resources can be reused. Both are worth trying
before asking the computer to work harder. To reduce overhead while preserving
deterministic behavior, `swmmrs`:

- avoids repeating internal work when an exact, already-validated result is
  available;
- reuses simulation-owned execution resources;
- applies concurrency only where ownership and calculation order remain
  explicit; and
- uses bounded buffering and managed resource lifetimes for frequent file I/O.

These are implementation choices, not alternate runoff or hydraulic equations.
Parallel work is joined at defined boundaries, and numerically sensitive
reductions retain deterministic ordering.

The [real-world benchmark report](../benchmark-report.html) compares runtime
and result similarity across SWMM-compatible engines. See
[Real-world benchmarks](../benchmark-results.md) for its scope and limits.

## Checkpoints, resume, and branching

For a compatible physical-state handoff, EPA hotstarts remain available.
If you want to stop a run, come back later, recover from a model interuption 
(e.g. system crash, invasive windows update 🤬), or explore what would have happened
with a different control decision, `swmmrs` has a few more options. Branching
timelines are easier to manage when each one owns its files:

- **Checkpoint and resume** preserve numerical continuation, clocks,
  accounting, persistent forcing, and managed-resource progress. Really helpful
  for recovering a long-running simulation that stopped for some unexpected reason.
- **Fork** creates an independent in-process branch from a quiescent simulation
  boundary. Useful for exploring in-process alternatives.
- **Checkpoint State Load** imports compatible physical state while the receiver
  keeps its own dates, declarations, accounting, and output ownership. Simply a 
  more fully featured hotstart the loads more than the EPA hotstart 
  (but not yet feature complete).

!!! warning
    `swmmrs` checkpoints are not EPA files. Use an EPA hotstart when format
    interoperability is required. See
    [Continuation and branching](../concepts/continuation-and-branching.md).

## Lifecycle and observation boundaries

The Simulation Owner validates open, start, step, end, report, checkpoint, and
close transitions. It owns solver state and managed files, and reports failures
through structured errors while retaining EPA-compatible numeric error identity
where applicable.

The Python interface provides:

- **Live Views**, which read current values from one open project generation;
- **Snapshots and statistics**, which remain valid after the simulation advances
  or closes.

A live view from yesterday's project cannot quietly become a view of today's
project just because both happen to contain a node called `J1`. Stale views
are rejected. A valid run follows the same numerical trajectory.

## Intentional integration behavior

A small number of differences extend integration behavior without selecting a
new solver formulation:

- Date parsing first uses the configured EPA format, then accepts another
  supported ordering only when that parse fails.
- Python can apply a persistent fixed-stage outfall override without replacing
  the stable model declaration.
- Invalid lifecycle operations, stale views, conflicting resources, and
  incompatible continuation artifacts can be rejected earlier than equivalent
  low-level C usage.

These behaviors are explicit and tested. They do not silently enable an
alternative hydraulic or hydrologic model.
