# Solver compatibility changelog

This changelog tracks solver changes made after EPA SWMM 5.2.4 and the
additional compatibility corrections made by `swmmrs`. Entries are ordered
from newest to oldest.

The EPA source audit compares tag
[`v5.2.4`](https://github.com/USEPA/Stormwater-Management-Model/releases/tag/v5.2.4)
with the 5.3 `bug_fixes` development history through commit `b8863c2`. Source
links target the latest
[`bug_fixes`](https://github.com/USEPA/Stormwater-Management-Model/tree/bug_fixes)
branch for traceability. Some 5.3 builds still print a 5.2.4 text-report banner,
so identify an executable by its source revision or build metadata rather than
by the report banner alone.

This page records completed changes. The status of both resolved and unresolved
EPA 5.3 defects is tracked in
[Known EPA SWMM 5.3 issues](epa-swmm-5.3-known-issues.md).

## swmmrs 0.1.0

### Corrections beyond the audited EPA 5.3 source

Corrections in this section are either **5.3 regressions** introduced or exposed by a 5.3-line change or **5.3 implementation bugs** in behavior added by 5.3. None corrects a prior SWMM 5 defect.

| Area | Classification | EPA 5.3 bug | EPA `bug_fixes` source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- | --- | --- |
| Land-use washoff | **5.3 regression** | Commit [`b91f310`](https://github.com/USEPA/Stormwater-Management-Model/commit/b91f310) changed the no-buildup guard from `buildup == 0.0` to `buildup >= 0.0`. Because buildup is normally nonnegative, the guard returned before calculating washoff. | [`landuse.c` L632–634](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/landuse.c#L632-L634) | Pollutant could accumulate on the land surface but exponential, rating-curve, and EMC washoff remained zero. Runoff pollutant loads and downstream concentrations were understated. | Retains the EPA 5.2.4 zero-only guard. |
| Steady-flow pollutant forcing | **5.3 implementation bug** | `findSFLinkQual` assigned the upstream node index to `j`, then used `Link[j]` for API mass flux and total load instead of the current link `i`. | [`qualrout.c` L447–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L447-L469) | API forcing and pollutant load could be applied to the wrong link. A node index larger than the link array could also cause an invalid memory access instead of producing results. | Uses the current `link_index` consistently for mass flux and total load. |
| Dry-link pollutant forcing | **5.3 implementation bug** | Link-quality paths divided nonzero API mass flux by `old volume + inflow × timestep` when a conduit ended dry and that denominator was zero. They also allowed forcing on link types that bypass the conduit reactor. | [`qualrout.c` L340–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L340-L469) | Positive forcing could produce infinite concentration and non-finite pollutant loads or continuity totals. Negative forcing attempted to remove pollutant from no water, and forcing on unsupported link types had no defined reactor behavior. | Restricts nonzero flux to non-dummy conduits. Positive flux into a conduit that ends dry is recorded as external inflow, link load, and non-remobilizing final dry storage. Negative dry-link flux is a no-op; wet-conduit calculations are unchanged. |
| Dry-node pollutant forcing | **5.3 implementation bug** | Routing counted positive API node mass flux as external inflow, but a node that ended dry with negligible hydraulic inflow discarded the temporary pollutant mass instead of storing or discharging it. | [`routing.c` L498–513](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/routing.c#L498-L513); [`qualrout.c` L239–339](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L239-L339) | The source appeared in external-inflow totals but not in concentration, outflow, reaction loss, or final storage. This silently lost pollutant mass and created a quality-continuity error. | Deposits positive flux into non-remobilizing final dry storage and leaves concentration zero. Negative and signed-zero forcing remain dry-node no-ops; wet-node mixing is unchanged. |
| Partial-area LID runon | **5.3 regression** | Commit [`9d3bb8d`](https://github.com/USEPA/Stormwater-Management-Model/commit/9d3bb8d) changed the full-area LID check to `subcatch->area >= subcatch->lidArea`, which is true for nearly every valid partial-area LID. | [`lid.c` L1687–1691](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1687-L1691) | Full-subcatchment upstream runon entered the partial-area LID while the non-LID area also accounted for it. This double-counted runon and distorted LID inflow, storage, infiltration, treatment, and runoff. | Uses the defensive full-area relation `subcatch.area <= subcatch.lid_area`. Validation still rejects material overallocation and snaps accepted allocations above 99.9% to full coverage. |
| Metric averaged maximum node depth | **5.3 regression** | EPA applied `UCF(LENGTH)` to averaged `NodeDepth` values that `node_getResults` had already converted to report units. | [`output.c` accumulation L851–860](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L851-L860); [first conversion `node.c` L483](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/node.c#L483); [second conversion L918–923](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L918-L923) | Metric projects reported an averaged maximum-node-depth statistic with a second length conversion. Binary averaged depths and the hydraulic trajectory were unchanged, but the summary statistic was wrong. | Uses the averaged report-unit `NodeDepth` directly when updating the statistic. |
| Averaged maximum node depth | **5.3 regression** | EPA stored averaged results in a compact array of reportable nodes, then indexed that array with the model-wide node index when updating maximum depth. | [`output.c` accumulation L829–862](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L829-L862); [maximum update L918–923](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L918-L923) | Selecting only some nodes for output could report another node's maximum depth or read beyond the averaged-results array. The hydraulic trajectory itself was unchanged. | Keeps the reportable-result cursor separate from the model-wide node index. |
| Scheduled hotstart clock | **5.3 implementation bug** | The parser stored each requested calendar date and time in decimal days, but the save loop compared that absolute value with elapsed routing time in milliseconds. | [`iface.c` L82–105](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/iface.c#L82-L105); [`hotstart.c` L99–116](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/hotstart.c#L99-L116) | A scheduled hotstart could be written near the beginning of a run instead of at the requested instant. A later run resumed from the wrong hydraulic and water-quality state. | Compares the absolute schedule with the absolute routing date and time, then writes on the first routing step that reaches the requested instant. |
| Signed-zero link pollutant forcing | **5.3 implementation bug** | Dry-link quality paths could divide the default `+0.0` or `-0.0` API mass flux by zero available water. | [`qualrout.c` L340–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L340-L469) | A model with no effective external forcing could still produce non-finite link concentration, pollutant load, and continuity values in Steady, Kinematic Wave, or Dynamic Wave routing. | Makes the steady-flow positive-flux branch conditional on `c_out != 0.0`, matching the ordinary path so either signed zero is a no-op. |
| Dry no-inflow node quality | **5.3 regression** | For a non-storage node with negligible inflow, EPA restored old concentration only when depth was positive and no longer cleared the temporary mass-inflow accumulator when the node was dry. | [`qualrout.c` L230–263](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L230-L263) | A pollutant mass rate could survive as though it were a concentration, then appear as a false concentration or pollutant pulse after the node rewetted. | Restores the EPA 5.2.4 dry branch. Wet nodes retain `old_qual`; dry nodes clear the temporary accumulator without changing continuity totals. |
| Covered rain-barrel area | **5.3 implementation bug** | Inside the per-unit loop, every covered rain barrel returned rainfall from the aggregate LID area instead of that unit's replicated area. | [`lid.c` L1649–1685](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1649-L1685) | Repeated covered barrels or mixed LID groups could return too much rainfall, overstating flow to the pervious area and distorting subcatchment runoff and water balance. | Uses the current unit's replicated `lid_area` when returning rainfall. |

### New features

| Area | `swmmrs` feature | Reference | Compatibility effect | Coverage |
| --- | --- | --- | --- | --- |
| Custom elliptical cross sections | Adds an opt-in `true_ellipse` model that uses mathematical ellipse area, width, and wetted perimeter at every depth. Released EPA formulas and partial-flow tables remain the default. | [True Ellipse Doc](epa-swmm-deviations/solver-features/hydraulics/true-ellipse-behavior.md) | The alternative is deliberately non-EPA and affects only custom ellipses with explicit rise and span. Catalog sections are unchanged. | Geometry tests cover legacy formulas, catalog invariance, exact circle behavior, interpolation convergence, interfaces, reporting, and checkpoints. |
| Antecedent Moisture Model RDII | Adds reusable standard and baseflow AMM declarations, area-scaled node assignments, and combined RTK+AMM RDII interface generation. | [AMM Doc](epa-swmm-deviations/solver-features/hydrology/amm-rdii.md) | Existing EPA input is unchanged. Models using `[AMM_MODELS]`, `[AMM_COMPONENTS]`, and `[AMM_RDII]` use a non-EPA hydrologic formulation. | Per-step equation oracles, US/SI stream equivalence, mixed-node aggregation, temperature rollback, external RDII replay, checkpoint, and fork tests. |


## EPA SWMM 5.3

### Numerical and behavioral fixes

| Area | EPA 5.3 change | EPA `bug_fixes` source | Practical effect |
| --- | --- | --- | --- |
| Dynamic-wave storage | Enables isolated, surcharged storage nodes to drain when the flow derivative is zero ([#149](https://github.com/USEPA/Stormwater-Management-Model/issues/149)). | [`dynwave.c` L681–702](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/dynwave.c#L681-L702) | Prevents storage depth from remaining artificially fixed when exfiltration or another loss should drain it. |
| Modified Horton infiltration | Corrects maximum cumulative infiltration handling and limits cumulative infiltration to the configured maximum ([#59](https://github.com/USEPA/Stormwater-Management-Model/issues/59), [#158](https://github.com/USEPA/Stormwater-Management-Model/issues/158)). | [`infil.c` L501–574](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/infil.c#L501-L574) | Can reduce infiltration and increase rainfall excess after the maximum is reached. It does not affect models using another infiltration method. |
| FREE and NORMAL outfalls | Adds the connected-link offset to normal or critical outlet depth instead of forcing zero depth for a nonzero offset ([#154](https://github.com/USEPA/Stormwater-Management-Model/issues/154), commit [`4606020`](https://github.com/USEPA/Stormwater-Management-Model/commit/4606020)). | [`node.c` L1395–1403](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/node.c#L1395-L1403) | Removes an artificial head difference that could cause extreme flow, flooding, and pump activation. |
| LID rain barrels | Routes rainfall on a covered rain barrel to return flow instead of admitting it into the covered barrel ([#166](https://github.com/USEPA/Stormwater-Management-Model/issues/166), commit [`9d3bb8d`](https://github.com/USEPA/Stormwater-Management-Model/commit/9d3bb8d)). | [`lid.c` L1660–1685](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1660-L1685) | Restores footprint rainfall in the simple single-covered-barrel case. The same commit introduced the partial-area runon regression corrected in `swmmrs` 0.1.0. |
| Node-only routing | Runs the routing step when at least one node or link exists. | [`routing.c` L406–428](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/routing.c#L406-L428) | Ensures nodal seepage is applied in models that contain nodes but no links. |
| Custom elliptical cross sections | The audited, unreleased `bug_fixes` branch replaces the released one-dimension formulas with empirical coefficients using both rise and span ([#144](https://github.com/USEPA/Stormwater-Management-Model/issues/144)). | [`xsect.c` L556–610](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/xsect.c#L556-L610) | This is not mathematical ellipse geometry and is not a released EPA default. `swmmrs` keeps released behavior by default and provides a separate true-ellipse opt-in. |
| Simulation duration | Computes date and time differences separately to avoid precision loss ([#152](https://github.com/USEPA/Stormwater-Management-Model/issues/152)). | [`project.c` L155–188](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/project.c#L155-L188) | Avoids a lost or added final step for susceptible date ranges. |
| Table input | Detects a comment after the first token and before collecting the rest of the line ([#165](https://github.com/USEPA/Stormwater-Management-Model/issues/165)). | [`table.c` L849–886](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/table.c#L849-L886) | Prevents token-array overflow on unusually long comment lines. |
| Averaged output statistics | Attempts to correct maximum node-depth reporting when output averaging is enabled ([#188](https://github.com/USEPA/Stormwater-Management-Model/issues/188)). | [`output.c` L918–923](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L918-L923) | Changes only the reported maximum, not the hydraulic trajectory. `swmmrs` 0.1.0 corrects the EPA indexing defect and avoids a second metric length conversion of already converted reported depth. |

### API and operational changes

| Area | EPA 5.3 change | EPA `bug_fixes` source | Scope |
| --- | --- | --- | --- |
| Hotstarts | Adds saving of multiple hotstart files at specified simulation times and expands the prescribed-hotstart API. | [`iface.c` L82–139](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/iface.c#L82-L139); [`hotstart.c` L77–158](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/hotstart.c#L77-L158) | Opt-in API and operational behavior. |
| Object APIs | Exposes additional object attributes and public API error codes. | [`swmm5.h` properties L104–261](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/include/swmm5.h#L104-L261); [errors L374–404](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/include/swmm5.h#L374-L404) | API compatibility; no default numerical change. |
| Runtime precipitation | Adds API-provided rainfall and snowfall to subcatchment precipitation. | [`subcatch.c` L747–761](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L747-L761) | Results change only when callers provide runtime precipitation. |
| Runtime water quality | Adds API-provided pollutant mass fluxes and buildup/washoff adjustments. | [`surfqual.c` L160–173](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/surfqual.c#L160-L173); [`routing.c` L498–513](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/routing.c#L498-L513); [`qualrout.c` L239–250](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L239-L250) | Results change only when callers use the new water-quality forcing paths. |
| Shared constants | Replaces duplicated numerical constants with definitions from `consts.h` in LID, link, subcatchment, and transect calculations. | [`lid.c` L1045–1063](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1045-L1063); [`link.c` L1143–1147](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/link.c#L1143-L1147); [`subcatch.c` L396–400](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L396-L400); [`transect.c` L459–466](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/transect.c#L459-L466) | Primarily source organization; no intended formula change was identified. |

### Additional final-tree changes found by source audit

| Area | EPA 5.3 change | EPA `bug_fixes` source | Practical effect |
| --- | --- | --- | --- |
| Binary-output metadata | Commit `b91f310` splits generic invert, maximum-depth, and offset property identifiers into input/output identifiers. Numeric codes change, and downstream link offset receives its own code. | [`output.c` identifiers L56–66](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L56-L66); [preamble L252–313](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L252-L313) | The `.out` preamble is not metadata-compatible with 5.2.4 readers that assume the old codes. `swmmrs` mirrors the 5.3 schema. |
| External time-series tables | Accepts commas as token separators in addition to whitespace. | [`consts.h` L402–411](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/consts.h#L402-L411); [`table.c` L856–886](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/table.c#L856-L886) | Adds support for comma-delimited external records. `swmmrs` currently supports whitespace-delimited records but has not adopted this input extension. |
| OpenMP node updates | Commit `f9fa0ed` restores parallel dynamic-wave node updating when multiple threads are requested. | [`dynwave.c` L590–617](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/dynwave.c#L590-L617) | Primarily changes threaded execution and performance; no formula change was identified. |
| Overallocated LID area | Clamps `subcatchment area - LID area` to zero during runoff calculations. | [`subcatch.c` L623–655](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L623-L655); [`lid.c` L1787–1819](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1787-L1819) | Invalid or API-mutated overallocated models enter the full-area-LID branch instead of using negative-area bookkeeping. `swmmrs` mirrors the clamp and rejects material overallocation during preparation. |

## EPA SWMM 5.2.5

These changes occur after the `v5.2.4` tag even though their source headers
identify them as Build 5.2.5.

### Fixes documented in source headers

| Area | EPA 5.2.5 change | EPA `bug_fixes` source | Practical effect |
| --- | --- | --- | --- |
| Hotstarts and water quality | Corrects the `fwrite` item count used for land-use pollutant buildup. | [`hotstart.c` L549–558](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/hotstart.c#L549-L558) | Prevents malformed or incomplete buildup state in saved hotstart files. |
| Flow-summary reporting | Changes formatting for very large positive or negative flow values. | [`statsrpt.c` L87–100](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/statsrpt.c#L87-L100) | Prevents adjacent report columns from merging; this is a reporting-only change. |

## Maintaining this changelog

When adopting another upstream change:

1. Add it under the version that introduced it, keeping versions newest first.
2. Link both the EPA issue or commit and the implementing line in the latest
   `bug_fixes` branch.
3. State whether it affects ordinary calculations, reporting, or an opt-in API
   path.
4. Add focused regression coverage for every material calculation branch.
5. Put a correction completed by `swmmrs` in the applicable `swmmrs` release
   section and update its status on the
   [known-issues page](epa-swmm-5.3-known-issues.md).
6. Add newly discovered defects to the known-issues page even when they are not
   yet corrected.
7. Re-run representative comparisons against an explicitly identified EPA
   source revision.
