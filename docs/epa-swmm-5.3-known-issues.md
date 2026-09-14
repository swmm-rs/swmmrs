# Known EPA SWMM 5.3 issues

This page tracks defects and their modeling impacts in the audited EPA SWMM
5.3 source and shows whether `swmmrs` has corrected them. Some issues originated
before 5.3 but remain in the 5.3 `bug_fixes` branch. Each heading identifies its
origin: a **5.3 regression** was introduced or exposed by a 5.3-line change,
a **5.3 implementation bug** is a defect in behavior newly added in 5.3, while a
**prior SWMM 5 bug** predates that line.

The audit covers `bug_fixes` through commit `b8863c2`. Source links below target
the latest [`bug_fixes`](https://github.com/USEPA/Stormwater-Management-Model/tree/bug_fixes)
branch so each claim can be checked against the current 5.3 code. Completed
corrections also appear under the applicable release in the
[solver compatibility changelog](solver-changelog.md).

| Marker | Status | Meaning |
| --- | --- | --- |
| ✅ | **Resolved in `swmmrs`** | `swmmrs` has a correction and focused regression coverage. |
| ⚠️ | **Open in `swmmrs`** | `swmmrs` still mirrors or only partially corrects the affected EPA behavior. |

## ✅ Resolved in swmmrs

### Positive buildup remains eligible for washoff *(5.3 regression)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Commit [`b91f310`](https://github.com/USEPA/Stormwater-Management-Model/commit/b91f310) changes the no-buildup guard in `landuse_getWashoffQual` from `buildup == 0.0` to `buildup >= 0.0`. | [`landuse.c` L632–634](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/landuse.c#L632-L634) | Because buildup mass is normally nonnegative, the changed guard returns before calculating exponential, rating-curve, or EMC washoff. Pollutant accumulates on the land surface, but runoff pollutant loads and downstream concentrations are understated. | Retains the EPA 5.2.4 zero-only guard. |

### Steady-flow pollutant forcing addresses the current link *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| `findSFLinkQual` assigns the upstream node index to `j` and then reads `Link[j].apiExtQualMassFlux` and updates `Link[j].totalLoad`, although the intended object is the current link `i`. | [`qualrout.c` L447–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L447-L469) | API forcing and pollutant load can be applied to the wrong link. A node index larger than the link array can also cause an invalid memory access instead of producing results. | Uses the current `link_index` consistently for mass flux and total load. |

### Signed-zero external link forcing is a dry-link no-op *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Ordinary and steady-flow link-quality paths can divide signed-zero external flux by zero available water. | [`qualrout.c` ordinary path L383–393](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L383-L393); [steady-flow path L447–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L447-L469) | The default zero flux can produce non-finite link concentration, pollutant load, and continuity values on a dry, zero-inflow link even though the model has no effective external forcing. | Keeps the calculations inline and changes the steady-flow `else` to `else if c_out != 0.0`, matching the ordinary path so either signed zero skips division, load, and mass-balance updates. |

### Nonzero external pollutant forcing into dry links *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Link-quality paths divide nonzero API mass flux by `old volume + inflow × timestep` even when the conduit ends dry and the available-water denominator is zero. | [`qualrout.c` L340–469](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L340-L469) | Positive forcing can produce infinite concentration and non-finite pollutant loads or continuity totals. Negative forcing has no aqueous mass to withdraw. Pumps, regulators, outlets, and dummy conduits also bypass the conduit reactor in which the forcing calculation is defined. | Restricts nonzero link mass flux to non-dummy conduits. For a conduit classified dry at the end of the quality step, positive forcing is recorded as external inflow, link load, and non-remobilizing final dry storage without changing concentration; negative forcing is a no-op. Wet-conduit arithmetic and signed-zero behavior are unchanged. A complete zero candidate remains valid for clearing unsupported links. |

### Positive external pollutant forcing into dry nodes *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Routing adds each positive API node pollutant mass flux to both the node's temporary quality-inflow accumulator and external-inflow continuity. With negligible hydraulic inflow, a dry non-storage node clears that accumulator, while a dry storage node's mixing path ignores it and adds only `concentration × new volume` to final storage. A completely dry node therefore retains none of the supplied mass. | [`routing.c` L498–513](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/routing.c#L498-L513); [`qualrout.c` L239–339](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L239-L339) | Positive external mass appears in inflow totals but not in node concentration, outflow, reaction loss, or final storage. The model silently loses the source and reports a quality-continuity error. | When any node type ends a quality-routing step dry with negligible hydraulic inflow, deposits the positive API mass-flux rate into non-remobilizing final dry storage and leaves concentration zero. Routing has already recorded the same rate as external inflow, so the correction adds only the continuity sink. Negative and signed-zero forcing are no-ops on dry nodes; wet-node mixing is unchanged. |

### Dry no-inflow node quality is cleared *(5.3 regression)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| For a non-storage node with negligible inflow, EPA restores old concentration only when depth is positive and no longer explicitly clears the dry branch. | [`qualrout.c` L258–262](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L258-L262) | A pollutant mass rate left in `newQual` can survive as though it were a concentration, then appear as a false concentration or pollutant pulse when the node rewets. | Restores the EPA 5.2.4 dry branch: wet nodes retain `old_qual`, while dry nodes clear the temporary mass-inflow accumulator without changing continuity totals. |

### Covered rain barrels return rainfall from their local footprint *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Inside the per-unit loop, every covered barrel adds rainfall times the aggregate `subcatch->lidArea` instead of that unit's area. | [`lid.c` L1678–1679](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1678-L1679) | Multiple covered-barrel records or mixed LID groups can return too much rainfall, overstating flow to the pervious area and distorting subcatchment runoff and water balance. | Uses the loop-local replicated `lid_area`, preserving each unit's coverage and replication. |

### Partial-area LIDs exclude full-subcatchment runon *(5.3 regression)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| Commit [`9d3bb8d`](https://github.com/USEPA/Stormwater-Management-Model/commit/9d3bb8d) changes the full-area LID check to `subcatch->area >= subcatch->lidArea`, a relation true for nearly every valid partial-area LID. | [`lid.c` L1687–1691](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/lid.c#L1687-L1691) | Full-subcatchment upstream runon enters the partial-area LID while the non-LID area also accounts for it. This double-counts runon and distorts LID inflow, storage, infiltration, treatment, and runoff. | Uses the defensive full-area relation `subcatch.area <= subcatch.lid_area`. Validation rejects material overallocation and snaps accepted allocations above 99.9% to full coverage. |

### Averaged maximum depth uses reportable-node indexing *(5.3 regression)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| EPA stores averaged results only for reportable nodes using a compact cursor, then indexes that array with the model-wide node index when computing maximum depth. | [`output.c` accumulation L829–862](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L829-L862); [maximum update L918–923](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L918-L923) | Selecting only some nodes for reporting can show another node's maximum depth or read beyond the averaged-results array. The hydraulic trajectory itself is unchanged. | Keeps the reportable-result cursor separate from the model node index. |

### Averaged maximum depth retains report units in metric projects *(5.3 regression)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| EPA applies `UCF(LENGTH)` to averaged depth even though `node_getResults` already accumulated report-unit values. | [`output.c` accumulation L851–860](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L851-L860); [first conversion `node.c` L483](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/node.c#L483); [second conversion L918–923](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/output.c#L918-L923) | Metric projects apply a second length conversion to the averaged maximum-depth statistic. Binary averaged depths and the hydraulic trajectory are unchanged, but the summary statistic is wrong. | Uses the averaged report-unit `NodeDepth` directly when updating the statistic. |

### Scheduled hotstarts use the absolute routing clock *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` resolution |
| --- | --- | --- | --- |
| The parser stores a requested calendar date and time in decimal days, while the save loop compares that absolute value directly with elapsed routing time in milliseconds. | [`iface.c` L82–105](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/iface.c#L82-L105); [`hotstart.c` L99–116](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/hotstart.c#L99-L116) | A scheduled hotstart can be written near the beginning of a run instead of at the requested instant. A later run then resumes from the wrong hydraulic and water-quality state. | Compares the absolute schedule with the absolute date/time returned by the routing clock. EPA 5.3's early emission is intentionally not mirrored. |

## ⚠️ Open in swmmrs

### Land-use coverage can exceed 100% ([#230](https://github.com/USEPA/Stormwater-Management-Model/issues/230)) *(prior SWMM 5 bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` status |
| --- | --- | --- | --- |
| The engine stores every `[COVERAGES]` percentage independently and `subcatch_validate` never checks their aggregate. The EPA desktop interface rejects the condition, but engine and API callers can still create it. | [`subcatch.c` reader L287–313](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L287-L313); [validation L352–406](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L352-L406); [`surfqual.c` area use L105–118](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/surfqual.c#L105-L118) | Two land uses at 100% each spend the subcatchment area twice, silently doubling buildup and potentially washoff loading. Totals below 100% remain valid. | ⚠️ Not corrected. Input parsing has no aggregate check; programmatic preparation bounds each individual fraction to `[0, 1]` but does not bound their sum. |

### Subcatchment pollutographs use concentration interpolation ([#229](https://github.com/USEPA/Stormwater-Management-Model/issues/229)) *(prior SWMM 5 bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` status |
| --- | --- | --- | --- |
| `subcatch_getResults` linearly interpolates old and new concentrations, while routing interpolates old and new pollutant loads (`runoff × concentration`) and divides by interpolated flow. | [`subcatch.c` reporting L819–884](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/subcatch.c#L819-L884); [`surfqual.c` load interpolation L347–351](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/surfqual.c#L347-L351); [`routing.c` inflow L619–633](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/routing.c#L619-L633); [`qualrout.c` concentration L239–243](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/qualrout.c#L239-L243) | At report times inside a wet runoff step, the reported subcatchment concentration can differ sharply from the concentration routed to a directly connected outfall. Flow and mass continuity can remain correct. Aligning `REPORT_STEP` with `WET_STEP`, or shortening the wet step, only masks the discrepancy. | ⚠️ Not corrected; `swmmrs` mirrors the two interpolation paths. A fix must also define how LID drain flow and load participate in the reported concentration. |

### Long station names displace NCEI rainfall columns ([#224](https://github.com/USEPA/Stormwater-Management-Model/issues/224)) *(prior SWMM 5 bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` status |
| --- | --- | --- | --- |
| For the attached `COOP:`/`HPCP` export, `findNWSOnlineFormat` takes `ValueOffset` from the short header label but derives `DataOffset` from the actual data row. A station name wider than its nominal field shifts the row's rainfall value without shifting the saved header offset. | [`rain.c` offset discovery L584–634](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/rain.c#L584-L634); [row parsing L762–817](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/rain.c#L762-L817) | Date and time can parse from the row-relative offset while precipitation is read from the wrong character position, causing records to be discarded or the file to fail ingestion. Although the issue calls the download GHCN-Daily, the attached `HPCP` data selects the NWS Online rainfall parser, not the GHCND climate parser. | ⚠️ Not corrected; the Rust port retains the split header-derived `value_offset` and row-derived `data_offset`. Its growable strings remove the C line-buffer limit but not the displaced-column defect. |

### API-provided buildup and washoff accounting *(5.3 implementation bug)*

| EPA bug | EPA source | Modeling impact | `swmmrs` status |
| --- | --- | --- | --- |
| The EPA 5.3 source header records a TODO to track API-provided buildup and washoff separately in the mass balance. | [`surfqual.c` L18–25](https://github.com/USEPA/Stormwater-Management-Model/blame/bug_fixes/src/solver/surfqual.c#L18-L25) | Continuity accounting for callers using these API paths has not been fully validated. | ⚠️ Requires targeted mass-balance validation before it can be treated as resolved. |

## Updating issue status

When an issue is corrected in `swmmrs`:

1. add focused regression coverage;
2. record the correction in the translated source file's update history;
3. move the issue from **⚠️ Open** to **✅ Resolved** on this page;
4. add the correction to the applicable `swmmrs` release in the
   [solver compatibility changelog](solver-changelog.md); and
5. validate representative results against an explicitly identified EPA source
   revision.
