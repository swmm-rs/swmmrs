---
hide:
  - toc
---

# Keep the model. [*Change the ownership.*](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) { .swmm-hero-title }

Rain meets a city, discovers several million places it is not meant to be, and
becomes an engineering problem. `swmmrs` translates the EPA SWMM solver into
Rust. The hydraulic model stays familiar. Each simulation gets its own state,
and the compiler stands by the airlock asking who owns what.

<div class="swmm-proof-ledger">
  <div class="swmm-proof-item">
    <span class="swmm-proof-label">Hydraulic universe</span>
    <strong>Near-identical results</strong>
    <span>against EPA SWMM</span>
  </div>
  <div class="swmm-proof-item">
    <span class="swmm-proof-label">Engine room</span>
    <strong>Faster engine</strong>
    <span>in the published benchmarks</span>
  </div>
  <div class="swmm-proof-item">
    <span class="swmm-proof-label">Flight controls</span>
    <strong>Python, CLI, and browser</strong>
    <span>with isolated simulations</span>
  </div>
</div>

[Launch from terminal](cli/index.md){ .md-button .md-button--primary }
[Pilot it with Python](python/get-started.md){ .md-button }
[Build with JS](javascript/get-started.md){ .md-button }
[Open local workspace](https://app.swmm.rs){ .md-button }

!!! info "Evidence, not prophecy"
    **EPA-faithful results, faster runs, and APIs meant for this century.**

    `swmmrs` produces near-identical results to EPA SWMM. In the published
    real-world benchmarks, it also runs faster. Native Python APIs, isolated
    concurrent simulations, interactive controls, checkpoints, and typed
    results make the solver usable outside a single command-line voyage.

    The [regression evidence](regression-validation.md) and [real-world
    benchmark results](benchmark-results.md) contain the receipts.

    These claims apply to the published engine versions, workloads, hardware,
    and comparison method. No finite suite visits every possible timeline.
    Compare critical production results against a trusted EPA SWMM baseline.

## One prime directive

**Parity by default. Hydraulic innovation by explicit opt-in.**

The default path follows EPA SWMM. New hydraulic ideas can exist, but they live
in clearly marked alternate timelines that the user must choose.

| Default EPA path | Opt-in experiments |
| --- | --- |
| Existing EPA SWMM models should run with minimal result changes. | New hydraulic formulations and algorithms may join the project. |
| Calculations, evaluation order, edge cases, and input semantics stay intact. | The user must enable every alternative explicitly. |
| Differences remain reviewable beside the upstream C source. | An experiment never silently replaces a default. Its behavior, validation, and tradeoffs must be documented. |

**Parity is the baseline, not the ceiling.** A model uses non-EPA hydraulics only when its user asks for them. No wormholes,
no secret rerouting of the timeline. 




## Why put SWMM in Rust?

I began `swmmrs` to learn Rust on software I already use and respect. A to-do
list would have involved fewer differential equations, but it would also have
been a to-do list.

Rust makes each simulation own its files, objects, lifecycle, statistics, and
mutable solver state. One simulation no longer shares a giant control panel
with every other simulation in the process.

The translation keeps function order, names, comments, domain language, and
floating-point order recognizable beside EPA SWMM. That restraint is
intentional. Reviewers can compare the two implementations without decoding a
clever new civilization first.

The ownership work also gives the project typed APIs, isolated tests, and
independent concurrent runs. Those are useful consequences, not an excuse to
change the model.

## What changed under the hull

| Area | Improvement |
| --- | --- |
| State | Each `SwmmSimulation` owns one private `SwmmState` instead of process-global project state. |
| Python | Typed lifecycle, object collections, validated setters, runtime forcings, hotstarts, snapshots, and statistics. |
| Results | Live scalar reads for control, coherent snapshots for network sampling, and immutable cumulative records. |
| Concurrency | Independent simulations can run concurrently without sharing hydraulic state. |
| Extension policy | Non-EPA formulations may be developed, but remain strictly opt-in. |


## Regression validation

Matching one model proves approximately one model. So
[`swmm-bench`](https://github.com/swmm-rs/swmm-bench) runs an end-to-end suite
against EPA SWMM and compares parsed report tables with binary-output time
series. The suite covers hydrology, hydraulics, controls, routing, water
quality, and interface-file workflows.

The [EPA SWMM coverage
report](https://swmm-rs.github.io/swmm-bench/epa-swmm-coverage.html) shows that
the suite exercises more than 87% of upstream solver lines and 67% of branches.
Read [how to interpret the validation](regression-validation.md) or inspect the
[latest HTML regression report](regression-report.html).

## Real-world benchmarks

Accuracy matters, but so does getting the answer back before the deadline. The
companion `swmm-bench` benchmark runs SWMM-compatible engines on large hydraulic
models drawn from day-to-day modelling work. It reports simulation duration
beside report and binary-output similarity.

On the tested workloads, `swmmrs` is faster than EPA SWMM
while maintaining a small result distance from the original engine.

[Read the benchmark scope and interpretation](benchmark-results.md) or inspect
the [latest HTML benchmark report](benchmark-report.html). No galactic standards
committee has issued a universal speed ranking. The findings belong to the
engine versions, workloads, hardware, and run conditions in that report.

!!! info "Known limits, not plot twists"
    - Dynamic Wave is the largest structural departure from upstream C and remains the highest-priority parity area.
    - During routing, collect live values and snapshots. After routing stops, use [`OutputReader`](guides/read-output.md) for report-period output.
    - See [compatibility and limitations](python/compatibility.md) before production evaluation.

## Choose an interface

<div class="grid cards" markdown>

- **Python**

    Pause time, inspect a node, change a control, or run several scenarios
    without making them share a brain.

    [Python docs →](python/index.md)

- **JavaScript / TypeScript**

    Put the solver in a worker, keep it off the UI thread, and retain typed
    collections, controls, snapshots, and output downloads.

    [JavaScript / TypeScript docs →](javascript/index.md)

- **Command line**

    Give the runner one input file. It returns a text report and SWMM binary
    output with very little ceremony.

    [CLI guide →](cli/index.md)

- **Concepts**

    Read how isolated state, post-open declarations, lifecycle, and continuation
    affect programmatic modeling before inventing a paradox.

    [Concepts overview →](concepts/index.md)

- **Solver differences**

    See where state isolation, performance work, checkpoints, and typed
    interfaces differ from EPA SWMM while the default hydraulics stay put.

    [Compare with EPA SWMM →](epa-swmm-deviations.md)

</div>


## Concepts and contributions

Start with [Concepts](concepts/index.md) for a modeller-focused explanation of
programmatic state, post-open declarations, solver lifecycle, and continuation.
When you are ready to send something back across the subspace relay, see
[Contributing](contributing/index.md).

## Upstream projects

There was no alien archive behind `swmmrs`, only decades of work by the [EPA
SWMM](https://www.epa.gov/water-research/storm-water-management-model-swmm)
developers and the [Open Water
Analytics](https://github.com/OpenWaterAnalytics/Stormwater-Management-Model)
community. Keeping the translation recognizable acknowledges that work and
helps with parity review. The repository's [third-party
notices](https://github.com/swmm-rs/swmmrs/blob/main/THIRD_PARTY_NOTICES.md)
reproduce the OWA MIT license, the EPA public-domain statement, and contributor
attribution.
