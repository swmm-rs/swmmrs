# Solver features

This is where the equations get a vote. Solver features are alternatives to
released EPA SWMM behavior that you explicitly select, and they can change
your results. Changes to ownership and integration have their own
[software architecture](../software-architecture.md) page.

!!! info "You have to turn these on"
    These features are disabled unless selected. The default solver already
    includes unpublished EPA 5.3 improvements and corrections to that
    development version, as described in the
    [default solver baseline](../../epa-swmm-deviations.md#default-solver-baseline).
    Feature selections are identified in reports and compatibility documentation.

## Available features

| Domain | Feature | Default | Opt-in effect |
| --- | --- | --- | --- |
| [Hydraulics](hydraulics/index.md) | [True-ellipse hydraulics](hydraulics/true-ellipse-behavior.md) | `EPA_LEGACY` | Uses one mathematical ellipse for custom horizontal and vertical sections |
| [Hydrology](hydrology/index.md) | [Antecedent Moisture Model RDII](hydrology/amm-rdii.md) | EPA behavior | Generates temperature-sensitive RDII from standard and baseflow AMM components and combines it with native RTK at assigned nodes |
<!-- | [Hydraulics](hydraulics/index.md) | [Modified Preissmann-slot momentum](hydraulics/index.md#modified-preissmann-slot-momentum) | `SLOT` | Excludes fictitious slot area from pressurized-pipe momentum area | -->

## Hydraulics

Hydraulic features can change conduit geometry, conveyance, storage, flow, or
depth. `TRUE_ELLIPSE` changes custom horizontal and vertical ellipse sections.
<!-- `MODIFIED_SLOT` preserves Preissmann-slot storage while excluding fictitious
slot area from conduit momentum and local-loss calculations. -->

See the [hydraulic feature index](hydraulics/index.md) for selection and scope,
or the [true-ellipse detail page](hydraulics/true-ellipse-behavior.md) for its
equations, modeling implications, and verification.

## Hydrology

The [Antecedent Moisture Model RDII extension](hydrology/amm-rdii.md) generates
rainfall-dependent infiltration/inflow from reusable standard and baseflow AMM
components. It uses existing rain gages and project temperature data, and its
flow can be combined with native RTK RDII at the same node.

Models enable AMM with `[AMM_MODELS]`, `[AMM_COMPONENTS]`, and `[AMM_RDII]`.
Released EPA SWMM rejects these sections.

## Requirements for non-EPA solver features

An alternative formulation needs more than an interesting idea and an options
flag. It must be:

- explicitly enabled;
- documented as non-EPA behavior;
- covered by focused behavior and compatibility tests;
- visible in reports or public configuration; and
- excluded from the default EPA-compatible path.

Treat a feature selection as a model change. Read its detailed page before
enabling it, and compare critical projects against a trusted EPA SWMM release.
