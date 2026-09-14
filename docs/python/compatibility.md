# Compatibility and limitations

Before you build around an API, it helps to know which parts are ready and
which parts might move. Here are the supported runtimes, available capabilities,
and limits to account for.

!!! info "What the comparisons show"

    The native solver [passes broad regression comparisons against EPA SWMM](../regression-validation.md) and, among the [real-world benchmarks](../benchmark-results.md), is faster than EPA SWMM
    while maintaining a small result distance from the original engine. Python adds isolated simulations, typed model access, interactive controls, checkpoints, snapshots, statistics, and structured binary-output queries.
    The solver port is complete, but `swmmrs` and its public APIs remain pre-release. Published results are strong parity evidence, not proof for every model or routing condition; compare critical production projects against a trusted EPA SWMM release.

## Requirements and distribution

| Item | Current contract |
| --- | --- |
| Python | Regular CPython 3.11+ with `abi3` wheels; free-threaded 3.14 with a `cp314t` wheel. Free-threaded 3.15+ release wheels are deferred pending Python 3.15 support in the ABI audit tools. Free-threaded 3.13 is not supported. |
| Python runtime dependencies | None for an installed wheel. |
| Package version in this repository | `0.3.0`. |
| PyPI and native wheels | Published native wheels are the supported installation artifacts. |
| Public API stability | Early-stage; supported names and behavior may change before a stable release. |

Inspect package and embedded-solver identities independently, and include all three in bug reports:

```python
import swmmrs

print(swmmrs.__version__)
print(swmmrs.solver_version)
print(swmmrs.solver_build_id)
```

See [Package metadata](api/package-metadata.md) for the identity contract.

## Capability matrix

| Capability | Status |
| --- | --- |
| Complete open, start, step, end, report, and close lifecycle | Available. |
| Steady, kinematic, and Dynamic Wave routing | Available. |
| Runoff, groundwater, snowmelt, infiltration, LID, and quality calculations | Available. |
| Mathematical custom-ellipse hydraulics | Available through `CUSTOM_ELLIPSE_MODEL` in `[OPTIONS]` or `simulation.options.custom_ellipse_model`; released EPA behavior is the default. |
| Controls, external forcings, EPA hotstarts, Simulation Checkpoints/forks, statistics, and continuity results | Available with the continuation limitations below. |
| Text report and SWMM binary output writing | Available. |
| Python live object access and immutable snapshots | Available for the documented object families. |
| Independent concurrent simulation owners | Available; paths and thread budgets remain caller responsibilities. |
| Python post-run binary-output reader | Available for finalized files and automatic recovery of complete records from incomplete files. |
| PySWMM drop-in compatibility | Not provided. |
| Registered Python callback API | Not provided; use an explicit advancement loop. |

## Parity status

The Rust implementation intentionally preserves the C solver's calculations, evaluation order, control flow, and domain terminology. It passes a reasonably rigorous end-to-end suite orchestrated by [`swmm-bench`](https://github.com/swmm-rs/swmm-bench), which compares parsed report tables and binary-output time series across hydrology, hydraulics, controls, routing, water quality, and interface-file workflows.

The suite's [EPA SWMM coverage report](https://swmm-rs.github.io/swmm-bench/epa-swmm-coverage.html) shows that its model corpus exercises more than 87% of upstream solver lines and 67% of branches. See [Regression validation](../regression-validation.md) for scope and interpretation, and open the [latest HTML regression report](../regression-report.html) for the current results.

Dynamic Wave routing is the largest structural departure from upstream C because its parallel shared-state implementation was redesigned around Rust ownership and deterministic workers. It remains the highest-priority parity area.

The suite gives us strong evidence for the cases it exercises. Your model still
deserves its own comparison against a trusted EPA SWMM baseline before you rely
on the results. Public users should install a released wheel when available.

Custom horizontal and vertical ellipse sections use released EPA formulas by
default. The optional `TRUE_ELLIPSE` model is intentionally non-EPA, affects
full and partial depths, leaves catalog sections unchanged, and is identified
in the text report. `CUSTOM_ELLIPSE_MODEL` is a `swmmrs` input extension that
EPA SWMM does not accept. Read or replace the selection through
`simulation.options.custom_ellipse_model`; assignment takes effect when the next
`start()` prepares configuration. See
[true-ellipse hydraulics](../epa-swmm-deviations/solver-features/hydraulics/true-ellipse-behavior.md).

## Units and files

- Values use the unit system configured by the input model; no automatic SI conversion occurs.
- Input, report, and retained output paths must be distinct.
- Omitting `output_path` in Python uses a scratch output artifact and makes `simulation.output_path` `None`.
- `save_results=False` permits live reads, snapshots, and statistics but disables detailed reporting and saved report-period results.
- EPA hotstart files require compatible model structure, object ordering, and configuration.
- Simulation Checkpoints preserve fuller runtime/resource continuation and editable declarations.

## Choosing another tool

Use EPA SWMM when you need a validated upstream executable. If your application
depends on PySWMM's callbacks or broader ecosystem, staying with PySWMM is a
reasonable decision. Evaluate `swmmrs` when isolated ownership, typed access,
structured binary-output queries, deterministic concurrent simulations, or the
Rust implementation solve a problem you actually have.
