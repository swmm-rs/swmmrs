# Snapshots and statistics

Snapshots and statistics keep the values the solver had when you acquired
them. These frozen native PyO3 records come from one Simulation Owner
acquisition and remain usable after the simulation advances, ends, closes, or
opens a different project generation.

Only the solver creates these records. Callers cannot construct, mutate,
shallow-copy, or deep-copy them. To work with editable data, extract the fields
you need into Python containers.

| Acquisition | Record family |
| --- | --- |
| `nodes/links/subcatchments.snapshot(ids=None)` | Aligned current hydraulic snapshots |
| `nodes/links/subcatchments.quality_snapshot(ids=None)` | Pollutant-major quality snapshots |
| `nodes/links/subcatchments.statistics(ids=None)` | Aligned cumulative statistics |
| Object `.statistics` properties | Scalar node, link, or subcatchment statistics |
| Storage `.storage_statistics`, outfall `.outfall_statistics`, pump `.pump_statistics` | Subtype statistics |
| `subcatchment.lid_snapshot()` and `subcatchment.lid_units[index].snapshot()` | LID group and unit runtime snapshots |
| `simulation.statistics` | Coherent totals, diagnostics, and quality balances |

The principal record names are:

| Acquisition family | Record classes |
| --- | --- |
| Hydraulic snapshots | `NodeSnapshot`, `LinkSnapshot`, `SubcatchmentSnapshot` |
| Quality snapshots | `NodeQualitySnapshot`, `LinkQualitySnapshot`, `SubcatchmentQualitySnapshot` |
| Batch statistics | `NodeStatisticsSnapshot`, `LinkStatisticsSnapshot`, `SubcatchmentStatisticsSnapshot` |
| Scalar statistics | `NodeStatistics`, `LinkStatistics`, `SubcatchmentStatistics`, `StorageStatistics`, `OutfallStatistics`, `PumpStatistics` |
| LID runtime | `SubcatchmentLidSnapshot`, `LidUnitSnapshot` |
| System accounting | `SimulationStatistics`, `RunoffTotals`, `RoutingTotals`, `RoutingDiagnostics`, `QualityBalance` |

Aligned tuple fields follow `object_ids`; quality matrix rows follow
`pollutant_ids`. Read-only pollutant mappings preserve configured order.
`datetime` properties are naive whole-second model times and `timedelta`
properties use Python duration rounding. Record equality compares exposed field
values, and reprs list fields in canonical schema order.

To edit detached data, explicitly convert the needed tuple or mapping with
`list(...)` or `dict(...)`. This never changes the original record or solver.
See [Collect results and statistics](../../guides/collect-results.md) for selection rules,
lifecycle behavior, units, and complete examples.

::: swmmrs.snapshots
    options:
        show_root_heading: false
