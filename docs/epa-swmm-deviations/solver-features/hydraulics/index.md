# Hydraulic solver features

An option that changes a pipe's geometry also changes what the water can do
inside it. Hydraulic solver features can affect conveyance, storage, flow, or
depth, so each requires explicit selection in addition to the
[default solver baseline](../../../epa-swmm-deviations.md#default-solver-baseline).

## True-ellipse hydraulics

| Property | Behavior |
| --- | --- |
| Default | `EPA_LEGACY` |
| Opt-in value | `TRUE_ELLIPSE` |
| Input declaration | `CUSTOM_ELLIPSE_MODEL TRUE_ELLIPSE` in `[OPTIONS]` |
| Scope | Custom `HORIZ_ELLIPSE` and `VERT_ELLIPSE` sections with positive rise and span |
| Unchanged | Standard catalog ellipse sections |
| Report marker | `Custom Ellipse Hydraulics  TRUE ELLIPSE (NON-EPA)` |

The custom ellipse has two dimensions. It is reasonable to expect both to take
part in the calculation. The EPA-compatible default uses released
custom-ellipse formulas and partial-flow tables. `TRUE_ELLIPSE` derives full
and partial area, width, wetted
perimeter, hydraulic radius, and section factor from one mathematical ellipse.
Because those quantities feed routing, enabling the feature can change conduit
capacity, depth, velocity, and flow-regime transitions.

Read [True-ellipse hydraulics](true-ellipse-behavior.md) for the
motivation, equations, comparison with EPA behavior, selection APIs, modeling
implications, and verification evidence.

<!-- Not yet implemented.
## Modified Preissmann-slot momentum

| Property | Behavior |
| --- | --- |
| Default | `SLOT` |
| Opt-in value | `MODIFIED_SLOT` |
| Input declaration | `SURCHARGE_METHOD MODIFIED_SLOT` in `[OPTIONS]` |
| Python selection | `simulation.options.surcharge_method = SurchargeMethod.MODIFIED_SLOT` |
| Scope | Friction, pressure-gradient, inertial, local-loss, and `dqdh` terms |
| Unchanged | Slot area, surface width, storage, continuity, Froude number, and evaporation/seepage losses |
| Report marker | `Surcharge Method ......... MODIFIED_SLOT` |

`MODIFIED_SLOT` retains EPA SWMM's Preissmann-slot geometry and storage. It
uses the physical conveyance area `min(slot-inclusive area, full-pipe area)`
for momentum velocity, friction, transient and convective inertia, and local
losses. The pressure-gradient and `dqdh` terms use the same physical area.
The default `SLOT` method keeps EPA SWMM behavior.
-->

Return to the [solver feature overview](../index.md).
