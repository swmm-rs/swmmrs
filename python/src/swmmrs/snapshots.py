"""Immutable native point-in-time and cumulative solver records.

Snapshot tuple fields are aligned by ``object_ids`` and all records remain valid
after their source :class:`Simulation` advances, ends, closes, or reopens.
Records are created only by solver acquisitions and cannot be constructed or
mutated by callers.
"""

from ._swmmrs import (
    LidUnitSnapshot as LidUnitSnapshot,
    LinkQualitySnapshot as LinkQualitySnapshot,
    LinkSnapshot as LinkSnapshot,
    LinkStatistics as LinkStatistics,
    LinkStatisticsSnapshot as LinkStatisticsSnapshot,
    NodeQualitySnapshot as NodeQualitySnapshot,
    NodeSnapshot as NodeSnapshot,
    NodeStatistics as NodeStatistics,
    NodeStatisticsSnapshot as NodeStatisticsSnapshot,
    OutfallStatistics as OutfallStatistics,
    PumpStatistics as PumpStatistics,
    QualityBalance as QualityBalance,
    RoutingDiagnostics as RoutingDiagnostics,
    RoutingTotals as RoutingTotals,
    RunoffTotals as RunoffTotals,
    SimulationStatistics as SimulationStatistics,
    StorageStatistics as StorageStatistics,
    SubcatchmentLidSnapshot as SubcatchmentLidSnapshot,
    SubcatchmentQualitySnapshot as SubcatchmentQualitySnapshot,
    SubcatchmentSnapshot as SubcatchmentSnapshot,
    SubcatchmentStatistics as SubcatchmentStatistics,
    SubcatchmentStatisticsSnapshot as SubcatchmentStatisticsSnapshot,
)

__all__ = (
    "LidUnitSnapshot",
    "LinkQualitySnapshot",
    "LinkSnapshot",
    "LinkStatistics",
    "LinkStatisticsSnapshot",
    "NodeQualitySnapshot",
    "NodeSnapshot",
    "NodeStatistics",
    "NodeStatisticsSnapshot",
    "OutfallStatistics",
    "PumpStatistics",
    "QualityBalance",
    "RoutingDiagnostics",
    "RoutingTotals",
    "RunoffTotals",
    "SimulationStatistics",
    "StorageStatistics",
    "SubcatchmentLidSnapshot",
    "SubcatchmentQualitySnapshot",
    "SubcatchmentSnapshot",
    "SubcatchmentStatistics",
    "SubcatchmentStatisticsSnapshot",
)
