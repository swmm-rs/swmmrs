# How swmmrs differs from EPA SWMM

Porting SWMM to Rust sounds straightforward until someone asks which SWMM.
`swmmrs` incorporates improvements from EPA's unpublished 5.3 development line,
along with corrections to regressions and implementation bugs in that version.
Its defaults therefore differ from released EPA SWMM 5.2.4 in the areas listed
in the [solver compatibility changelog](solver-changelog.md).

This page sorts out what changed, what you have to select yourself, and what
merely moved to a different part of the engine room.

!!! info "What the comparisons tell us"

    `swmmrs` [passes broad regression comparisons against EPA SWMM](regression-validation.md)
    and the [real-world benchmarks](benchmark-results.md) measure runtime and
    result differences against a named EPA SWMM 5.2.4 build. In the published
    twelve-model comparison, `swmmrs` ran faster on eleven models and tied on
    one. It provides isolated simulations, typed Python APIs,
    interactive controls, checkpoints, structured results, and a command-line
    runner.

    The project remains pre-release. Published results are strong parity
    evidence, not proof for every model, routing condition, engine version, or
    computer. Compare critical production projects against a trusted EPA SWMM
    release.

## Difference categories

"Different" covers quite a lot of territory. A new owner for a file and a new
equation for a pipe deserve different amounts of your attention:

| Category | What differs | Expected result effect |
| --- | --- | --- |
| [Default solver changes](solver-changelog.md) | Adopted unpublished EPA 5.3 improvements and corrections to that development version | Can change results from EPA 5.2.4 without an opt-in |
| [Software architecture](epa-swmm-deviations/software-architecture.md) | State ownership, isolation, execution resources, continuation, APIs, lifecycle checks, and packaging | No intentional numerical change on the default EPA path |
| [Solver features](epa-swmm-deviations/solver-features/index.md) | Explicit alternatives to released EPA hydraulic or hydrologic formulations | Results can change only when the feature is explicitly selected |

### Software architecture

EPA SWMM historically organizes the solver around process-wide mutable state.
`swmmrs` gives each simulation an owner containing its model, runtime state,
files, and execution resources. This enables independent simulations,
checkpoints, forks, typed APIs, and stricter lifecycle checks while preserving
the default EPA calculation path.

Read [Software architecture](epa-swmm-deviations/software-architecture.md) for
the ownership, performance, continuation, interface, and failure-handling
differences.

### Solver features

Solver features change the model only when you select them. Choose a department:

- [Hydraulics](epa-swmm-deviations/solver-features/hydraulics/index.md);
- [Hydrology](epa-swmm-deviations/solver-features/hydrology/index.md).

The [solver feature overview](epa-swmm-deviations/solver-features/index.md)
summarizes every available opt-in and links to its detailed page.

## Default solver baseline

The source audit compares EPA SWMM 5.2.4 with the unpublished 5.3 `bug_fixes`
history through commit `b8863c2`. `swmmrs` incorporates changes from that line
while correcting identified regressions and implementation bugs. It does not
promise identical results to either EPA 5.2.4 or the uncorrected development
build. See the [solver compatibility changelog](solver-changelog.md) for adopted
changes and [known EPA 5.3 issues](epa-swmm-5.3-known-issues.md) for their status.

The port keeps solver functions, domain terminology, calculations, branch
behavior, and numerically sensitive evaluation order recognizable beside the
EPA source. Optional formulations such as `TRUE_ELLIPSE` and AMM remain
explicit selections, separate from these default solver changes.

A non-EPA solver feature must be:

- explicitly enabled;
- documented as non-EPA behavior;
- covered by focused behavior and compatibility tests;
- visible in public configuration and reports; and
- excluded from the default EPA-compatible path.

You should not need to care how Rust arranges its furniture. The relevant
question is whether a change affects the modeled result. Implementation-only
changes preserve deterministic ordering and are validated against trusted EPA
outputs.

## Compatibility boundaries

| Artifact or behavior | Compatibility intent |
| --- | --- |
| EPA `.inp` input | Preserve EPA interpretation; documented extensions require explicit declarations |
| Text report and binary `.out` output | Preserve familiar SWMM artifacts, subject to ongoing parity validation |
| EPA `.hsf` hotstart | Supported for compatible physical-state handoff |
| `swmmrs` checkpoint | `swmmrs`-specific continuation artifact; not EPA-compatible |
| Python API | Native `swmmrs` interface; not a drop-in PySWMM replacement |
| Concurrent simulations | Independent solver state; callers own file-path and resource coordination |

## Transparency and validation

The project passes an end-to-end suite orchestrated by
[`swmm-bench`](https://github.com/swmm-rs/swmm-bench). The suite compares parsed
report tables and binary-output time series across hydrology, hydraulics,
controls, routing, water quality, and interface-file workflows. Its
[EPA SWMM coverage report](https://swmm-rs.github.io/swmm-bench/epa-swmm-coverage.html)
shows that the model corpus exercises more than 87% of upstream solver lines and
67% of branches.

Read [Regression validation](regression-validation.md) for the evidence and its
limits, [Known EPA SWMM 5.3 issues](epa-swmm-5.3-known-issues.md) for upstream
behavior relevant to parity, and the
[Solver compatibility changelog](solver-changelog.md) for reviewed
upstream-version differences.

Every difference needs an explanation someone else can check:

1. Default numerical behavior is compared with EPA SWMM.
2. Intentional semantic changes require documentation and focused tests.
3. Concurrency must be deterministic and must not reorder floating-point
   reductions.
4. Performance changes must preserve the validated result path.
5. Pre-release limitations remain visible in the
   [compatibility guide](python/compatibility.md).

Private source maps and internal solver records are intentionally excluded from
the public site. These pages describe the practical differences users should
consider when choosing or validating `swmmrs`.
