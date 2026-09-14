<p align="center">
  <img src="../docs/_static/img/logo_text.png" alt="swmmrs" width="75%">
</p>

# swmmrs for Python

Native, typed Python bindings for the Rust port of the EPA Storm Water Management Model (SWMM) solver.

`swmmrs` gives each Python `Simulation` its own solver state, supports interactive stepping and runtime control, and exposes live model objects, immutable snapshots, and statistics without a separate C-library installation.

> [!NOTE]
> **Run SWMM workloads faithfully, faster, and through a modern Python API.**
>
> The native solver [passes broad regression comparisons against EPA SWMM](https://swmm-rs.github.io/swmmrs/regression-validation/). Python adds isolated simulations, typed model access, interactive controls, checkpoints, snapshots, statistics, and structured binary-output queries.
>
> The solver port is complete, but `swmmrs` and its public APIs remain pre-release. Published results are strong parity evidence, not proof for every model or routing condition.

## Why swmmrs?

The main [swmmrs project](https://github.com/swmm-rs/swmmrs) is a parity-first, line-by-line Rust port of EPA SWMM. It preserves upstream behavior before pursuing refactoring or performance work. The Python package puts a typed, lifecycle-aware interface over that native solver.

Key goals:

- isolate all mutable solver state inside each `Simulation`;
- make lifecycle and invalid operations explicit;
- support deterministic interactive and batch execution;
- expose Python-friendly, typed object views and immutable result records;
- allow independent simulations to run concurrently;
- borrow the familiar, productive Python workflow pioneered by PySWMM.

## With gratitude to PySWMM

`swmmrs` owes a substantial debt to [PySWMM](https://github.com/pyswmm/pyswmm). PySWMM showed what an approachable, interactive Python interface to SWMM could be and made workflows such as stepping through a simulation, inspecting model objects, applying real-time controls, and collecting results feel natural in Python. Its design, documentation, examples, and years of practical use are a major inspiration for this package.

Many of the most recognizable parts of the `swmmrs` API deliberately borrow from PySWMM: the `Simulation` context manager and iterator, object collections, node and link access, step-advance workflows, and the general shape of interactive control loops. Where `swmmrs` feels familiar to a PySWMM user, that is usually because PySWMM established a clear and effective pattern worth following.

Thank you to the PySWMM maintainers and contributors for building and sustaining such an important tool for the SWMM community. Their work significantly lowered the barrier to programmatic SWMM modeling and provided the practical API vocabulary that informed much of `swmmrs`. PySWMM remains the mature, established choice for Python users who need its broader feature set, ecosystem, callback system, or binary-output tools. Please visit the [PySWMM documentation](https://pyswmm.github.io/pyswmm/) and [PySWMM repository](https://github.com/pyswmm/pyswmm) to learn more, support their work, and cite the [PySWMM JOSS paper](https://doi.org/10.21105/joss.02292) when appropriate.

This influence is not a compatibility promise. `swmmrs` embeds a Rust port of the solver and uses a different ownership and lifecycle model; it is largely **not** a drop-in replacement for PySWMM. Imports, classes, callbacks, supported properties, error behavior, and output capabilities can differ. Existing PySWMM applications should expect an intentional migration rather than a package-name substitution. `swmmrs` is an independent project and is not affiliated with or endorsed by the PySWMM project.

## Features

- Batch execution or step-by-step simulation
- Context-manager cleanup and typed lifecycle states
- Configurable simulation options, dates, and supported model properties
- Live access to rain gages, subcatchments, nodes, links, and LIDs
- Runtime inflow, rainfall, stage, pollutant, and link-setting controls
- Configurable Python callback cadence with `step_advance()` and `stride()`
- Immutable hydraulic, water-quality, and statistics snapshots
- Hotstart input and checkpoint output
- Independent concurrent simulation owners
- Type annotations and a `py.typed` marker
- No Python runtime dependencies

## Requirements

- Python 3.11 or newer
- A supported native wheel

Released wheels are the supported installation artifact: they include the
package-private native extension and do not require Rust, a compiler, or a
solver checkout. The package version documented by this repository is `0.3.0`.

The release workflow builds two wheel families for Linux, macOS, and Windows
on x86-64 and ARM64:

- `cp311-abi3` for regular CPython 3.11+.
- `cp314-cp314t` for free-threaded CPython 3.14.

Free-threaded CPython 3.15+ release wheels (`cp315-abi3.abi3t`) are deferred
until the ABI audit tools support Python 3.15. The `abi3t` build target remains
available for local testing; it starts at 3.15 and cannot replace the 3.14t wheel.
Free-threaded 3.13 is not supported by the current PyO3 dependency.

> [!NOTE]
> The native solver source is private. To request access to the `swmm-rs`
> organization, email [admin@swmm.rs](mailto:admin@swmm.rs) with your GitHub
> username and a short description of the contribution you want to make.

## Install a released wheel

Install the published wheel into a clean virtual environment:

```bash
python -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install swmmrs
```

For an authorized artifact-only installation, pass a downloaded wheel directly:

```bash
python -m pip install /path/to/swmmrs-0.3.0-cp311-abi3-manylinux_2_28_x86_64.whl
```

## Quick start

### Run a model to completion

Use `execute()` when Python does not need to inspect or modify the model during routing. It starts the run, advances to completion, ends it, writes the report when results are enabled, and closes the project.

```python
from swmmrs import Simulation

simulation = Simulation("model.inp", "model.rpt", "model.out")
simulation.execute()
```

### Inspect a model while it runs

Iteration starts the simulation on the first advance. Natural exhaustion leaves it in `COMPLETE`, where final statistics remain available; call `end()` before `report()`. The context manager finalizes an active or complete run and closes the project even if the body raises, but it does not generate the report for you.

```python
from datetime import timedelta

from swmmrs import Simulation

records = []
with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    node = simulation.nodes["J1"]
    conduit = simulation.links["C1"]
    simulation.step_advance(timedelta(minutes=5))

    for current_time in simulation:
        records.append((current_time, node.depth, conduit.flow))

    simulation.end()
    simulation.report()
```

`step_advance()` changes how often control returns to Python. Like `stride()`, it defaults to keyword-only `strict=True`: exact cadence boundaries can shorten the final routing step. Use `step_advance(..., strict=False)` to preserve ordinary routing steps and return at or beyond each target. `step_advance(None)` restores routing-step iteration, ignoring `strict`. It does not replace or enlarge SWMM's internal routing time step.

### Apply a runtime control

Live object views can read current solver state and write supported runtime inputs. Values use the unit system configured by the model.

```python
from datetime import timedelta

from swmmrs import Simulation

with Simulation("model.inp", "model.rpt", "model.out") as simulation:
    basin = simulation.nodes["BASIN"]
    gate = simulation.links["OUTLET_GATE"]
    simulation.step_advance(timedelta(minutes=5))

    for _ in simulation:
        if basin.depth >= 4.0:
            gate.target_setting = 1.0
        elif basin.depth <= 2.0:
            gate.target_setting = 0.0

    simulation.end()
    simulation.report()
```

### Read binary output

`OutputReader` exposes immutable, typed series from a SWMM binary output. It
detects a valid final trailer automatically. Family queries share the structured
bulk reader and accept exact indexes or names plus optional `start` and `end`
bounds:

```python
from swmmrs.output import OutputName, OutputReader, PollutantAttribute

reader = OutputReader("model.out")
depth = reader.node_series("J1", "depth", start=0, end=12)
print(depth.selection, depth.times, depth.values)
pollutant = reader.node_series("J1", PollutantAttribute("TSS"))
```

If a run failed before writing the final trailer, the same
`OutputReader("model.out")` call recovers complete records already visible on
disk. `reader.is_finalized` is then `False`, `run_status.code` is `None`, and an
incomplete trailing record is ignored. This is a fixed snapshot, not live tailing.

`SeriesSelection` has the fields `element_type`, `element`, and `attribute`.
Names match stored bytes exactly; use `bytes` or `OutputName` for non-UTF-8 names.
Pollutants must use `PollutantAttribute`, so a pollutant named `depth` never
silently replaces the explicit static node-depth attribute. Results expose tuples
of times and values and raise stable `OutputError` categories for missing or
ambiguous names and attributes.

`read_bulk_series(..., low_memory=False)` reads one complete payload per
selected period by default. Set `low_memory=True` to read only selected
adjacent cell runs with smaller scratch memory. The same keyword is available
on all four family methods; both strategies return identical structured data.

## Core concepts

### One owner, one project generation

A `Simulation` owns one isolated native SWMM project at a time. Object collections and live views belong to that project generation. They remain valid across routing advances, but become stale after `close()` or after `open()` creates a replacement generation. Using a stale view raises `StaleViewError`.

Do not copy, deep-copy, or pickle a `Simulation`. Use a separate owner for each independent model or scenario.

### Lifecycle

| State | Meaning | Typical next operation |
| --- | --- | --- |
| `OPEN` | The input is parsed and configuration can be changed. | Configure, start, iterate, execute, or close. |
| `RUNNING` | Routing is active. | Step, stride, iterate, inspect, control, or end. |
| `COMPLETE` | Manual or iterator advancement reached the model end but finalization has not run. | Read final statistics, then end. |
| `ENDED` | Run finalization is complete and the project remains open. | Report, reconfigure, restart, or close. |
| `FAILED` | A native lifecycle operation failed. | Preserve the error and close. |
| `CLOSED` | Native state and project files are released. | Open a fresh project generation. |

Choose one advancement style per run:

- `for current_time in simulation` for iterator-owned stepping;
- `start()` plus `step()` or `stride()` for caller-owned stepping;
- `execute()` for an unattended batch run.

Do not mix iterator-owned advancement with `start()`, `step()`, `stride()`, or `execute()`.

The native Simulation Owner is authoritative for path resolution and retained
metadata, collision checks, checkpoint metadata, iterator cadence/exhaustion,
lifecycle cleanup, canonical collection identity, related-view construction,
and final public exception classification. Python retains presentation of
`Path`, `datetime`, and `timedelta`, uncached identity-only Live Views, and
context-manager body-exception precedence.

### Checkpoints and branches

At a manual-step boundary, `save_checkpoint(path)` publishes an immutable
Simulation Checkpoint. Resume it into fresh output files, import only its
physical continuation state into a compatible open model, or fork the live
owner without replay:

```python
simulation.save_checkpoint("split.json")
resumed = Simulation.resume("split.json", "resumed.rpt", "resumed.out")
branch = simulation.fork("branch.rpt", "branch.out")
receiver.load_checkpoint_state("split.json")
```

Resume and fork create independent owners and file handles. State Load keeps
the receiver's dates, inputs, statistics, and outputs.

### Model objects

Collections are available directly from a simulation:

```python
node = simulation.nodes["J1"]
link = simulation.links["C1"]
subcatchment = simulation.subcatchments["S1"]
rain_gage = simulation.rain_gages["RG1"]
```

IDs are case-insensitive. Collections preserve configured project order and provide `by_index()` where index-based access is required. Invalid IDs use normal `KeyError` behavior; invalid indices raise `IndexError`.

Configure supported static properties while the simulation is `OPEN` or `ENDED`:

```python
from datetime import timedelta

simulation.options.update(
    report_step=timedelta(minutes=15),
    allow_ponding=True,
)
simulation.nodes["J1"].settings.full_depth = 4.5
simulation.links["C1"].settings.roughness = 0.015
simulation.subcatchments["S1"].settings.width = 250.0
```

AMM models use the same editable live-view pattern. Component values are
detached dataclasses; replace the model's component sequence to apply changes:

```python
from swmmrs.objects import AmmAssignment

model = simulation.amm_models["M1"]
components = list(model.components)
components[0].hot_shcf = 0.06
model.update(
    cold_temperature=25.0,
    hot_temperature=75.0,
    components=components,
)
simulation.amm_assignments = [
    AmmAssignment(simulation.nodes["J1"], model, area=1.0),
]
```

Temperature values use project units, component windows use hours, SHCF values
use reciprocal project rain-depth units, and assignment areas use project
land-area units.

Inspect `simulation.unit_system` and `simulation.flow_units` before combining model values with external data. The API does not silently convert project units to SI.

### Live values, snapshots, and statistics

Use a live scalar view for a small number of current values:

```python
depth = simulation.nodes["J1"].depth
flow = simulation.links["C1"].flow
```

Use a snapshot for a coherent, immutable batch acquisition:

```python
nodes = simulation.nodes.snapshot(["J1", "J2"])
for node_id, depth, inflow in zip(
    nodes.object_ids,
    nodes.depth,
    nodes.total_inflow,
    strict=True,
):
    print(node_id, depth, inflow)
```

Snapshots and statistics are solver-created, immutable native PyO3 records and
remain usable after the simulation closes. They cannot be constructed or
mutated by callers; copy selected fields into lists or dictionaries when an
editable host representation is needed. Live views do not.

Statistics are available while `RUNNING` and after manual or iterator advancement reaches `COMPLETE`. Acquire final statistics before calling `end()`:

```python
simulation.start(save_results=False)
while simulation.step() is not None:
    pass

continuity = simulation.statistics
node_statistics = simulation.nodes.statistics()
simulation.end()
simulation.close()
```

`save_results=False` skips report-period binary results but still permits live reads, snapshots, quality snapshots, and statistics.

## Output files

`swmmrs` can write SWMM `.rpt` and `.out` artifacts. `OutputReader` detects and
queries finalized or incomplete binary `.out` files independently of a live simulation.

Collect live values or snapshots while Python must inspect or control the
advancing model. For post-run report-period analysis, pass an explicit
`output_path`, run with `save_results=True`, and open the completed artifact with
`OutputReader`.

Input, report, and output paths must be distinct. If `output_path` is omitted, SWMM uses a scratch output artifact and `simulation.output_path` is `None`.

## Concurrent simulations

Different `Simulation` owners can run concurrently, and native lifecycle operations release Python while they execute. Give every run distinct report and output paths. Operations on the same owner serialize; never use several threads to advance one simulation.

Also account for SWMM Dynamic Wave worker threads. For example, four concurrent models configured with `THREADS 2` can use up to eight caller-inclusive solver threads.

## Errors

All package-specific exceptions inherit from `SwmmError`:

- `ValidationError` — invalid Python arguments, values, or paths;
- `LifecycleError` — an operation is invalid in the current state;
- `StaleViewError` — a collection or object view belongs to an old project generation;
- `SolverError` — the native solver rejected an input or operation;
- `InternalSimulationError` — an unexpected binding or solver failure.

`SolverError` exposes `code`, `operation`, and optional `detail` fields:

```python
from swmmrs import Simulation, SolverError

try:
    Simulation("model.inp", "model.rpt", "model.out").execute()
except SolverError as error:
    print(error.code, error.operation, error.detail)
    raise
```

## Package and solver versions

The Python distribution and embedded SWMM solver have separate identities:

```python
import swmmrs

print(swmmrs.__version__)
print(swmmrs.solver_version)
print(swmmrs.solver_build_id)
```

## Documentation

- [Configure a model](https://github.com/swmm-rs/swmmrs/blob/main/docs/guides/configure-model.md)
- [Run a model](https://github.com/swmm-rs/swmmrs/blob/main/docs/guides/run-model.md)
- [Runtime forcings and controls](https://github.com/swmm-rs/swmmrs/blob/main/docs/guides/runtime-forcings.md)
- [Collect results](https://github.com/swmm-rs/swmmrs/blob/main/docs/guides/collect-results.md)
- [Hotstarts](https://github.com/swmm-rs/swmmrs/blob/main/docs/guides/hotstarts.md)
- [Python API reference sources](https://github.com/swmm-rs/swmmrs/tree/main/docs/python/api)
- [Main swmmrs project README](https://github.com/swmm-rs/swmmrs#readme)

## Licensing and third-party notices

The original Python interface source is licensed under Apache-2.0. Official
wheels include the separately licensed native solver under the permissive
[`swmmrs Binary Runtime License`](licenses/SWMMRS-BINARY-RUNTIME.txt), which
allows use, modification, and redistribution of compiled artifacts for any
purpose without granting access to the private solver source. The native
component incorporates EPA SWMM public-domain material and MIT-licensed Open
Water Analytics contributions. See [`LICENSE`](LICENSE) and the bundled
[third-party notices](THIRD_PARTY_NOTICES.md) for licensing and attribution.

## Development

With [Just](https://just.systems/) and uv installed, build local wheels from the
repository root without changing your Python environment:

```bash
just python-debug     # Cargo dev profile; python/target/wheels/debug/
just python-release   # Optimized wheel; python/target/wheels/release/
```

These builds use the pinned maturin backend and locked Cargo dependencies.
They require Rust and authorized access to the solver submodules. They do not
install the wheel or replace an editable installation.

For an editable environment and tests, run from `python/`:

```bash
uv sync --locked
uv run pytest
uv run ruff check src
```

Rust and native-extension development requires authorized access to the private
solver submodule. From the repository root, initialize all dependencies and run
the workspace tests:

```bash
cd ..
git submodule update --init --recursive
cargo test --workspace --exclude swmmrs-parallel --locked --all-features
```

### Test free-threaded Python

From the repository root:

```bash
uv run --no-project python python/tests/run_free_threaded.py
uv run --no-project python python/tests/run_free_threaded.py --python 3.15t
```

Each command installs the selected free-threaded CPython version with uv,
builds a temporary wheel with native `test-support` hooks, and tests the
installed wheel in an isolated environment. The default is 3.14t.
The project virtual environment stays unchanged; temporary wheels are deleted.

The first check runs without forcing `gil=0`, so it catches an import that
silently re-enables the GIL. It compares two concurrent simulation outputs
against a sequential reference and checks concurrent queries on a shared
`OutputReader`. The full Python suite then runs with `-X gil=0`, including
independent-owner overlap, same-owner lock contention, and shared/independent
output readers using both read strategies, stored dates, and the lazy time cache.
Two unrelated tests remain explicitly skipped.

The release workflow runs the 3.14t suite on Linux x86-64. The optional
`--python 3.15t` command is for local testing only, not release CI.
Every production wheel also runs `tests/wheel_smoke.py` on its target runner,
without test hooks.
Independent owners can run concurrently; operations on one owner remain
serialized. Callers must still sequence multi-call workflows and use distinct
output paths.

To request maintainer access, email [admin@swmm.rs](mailto:admin@swmm.rs) with
your GitHub username and a short description of the contribution you want to
make.

Contributions should preserve upstream SWMM behavior first. Structural cleanup and performance changes come after parity can be demonstrated with tests.
