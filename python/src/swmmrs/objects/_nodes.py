"""Node views with typed stable settings namespaces."""

from __future__ import annotations as _annotations

from collections.abc import (
    Iterable as _Iterable,
    Iterator as _Iterator,
    Mapping as _Mapping,
    MutableMapping as _MutableMapping,
)
from datetime import timedelta as _timedelta
from typing import TYPE_CHECKING as _TYPE_CHECKING, cast as _cast, final as _final

from .. import snapshots as _snapshots
from .._prettier import pretty_dataclass as _pretty_dataclass
from ..enums import NodeKind as _NodeKind
from ._base import _LiveView, _pollutant_mapping
from ._collections import _Collection

if _TYPE_CHECKING:
    from ._definitions import Curve, TimeSeries
    from ._links import Link
    from ._subcatchments import Subcatchment


class _NodeExternalPollutantMassFlux(_MutableMapping[str, float]):
    """Expose persistent node pollutant forcing as a canonical live mapping."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Node) -> None:
        self._owner = owner

    def _snapshot(self) -> tuple[list[str], list[float]]:
        owner = self._owner
        return owner._simulation._owner.node_external_pollutant_mass_flux(
            owner._generation, owner._index, owner._id, owner._subtype
        )

    def _mutate(
        self,
        replace: bool,
        args: tuple[object, ...],
        kwargs: dict[str, float],
    ) -> None:
        owner = self._owner
        owner._simulation._owner.update_node_external_pollutant_mass_flux(
            owner._generation,
            owner._index,
            owner._id,
            owner._subtype,
            replace,
            args,
            kwargs,
        )

    def __getitem__(self, key: str) -> float:
        owner = self._owner
        return owner._simulation._owner.node_external_pollutant_mass_flux_value(
            owner._generation, owner._index, owner._id, owner._subtype, key
        )

    def __setitem__(self, key: str, value: float) -> None:
        self._mutate(False, ([(key, value)],), {})

    def __delitem__(self, key: str) -> None:
        self._mutate(False, ([(key, 0.0)],), {})

    def __iter__(self) -> _Iterator[str]:
        return iter(self._snapshot()[0])

    def __len__(self) -> int:
        return len(self._snapshot()[0])

    def update(self, *args: object, **kwargs: float) -> None:
        """Apply one atomic sparse update without resetting omitted pollutants."""
        self._mutate(False, args, kwargs)

    def clear(self) -> None:
        """Atomically reset every configured pollutant value to zero."""
        self._mutate(True, (), {})

    def __repr__(self) -> str:
        return repr(dict(zip(*self._snapshot(), strict=True)))


class NodeSettings:
    """Expose stable common-node settings through one atomic owner."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Node) -> None:
        self._owner = owner

    def _read(self, field: str) -> object:
        return self._owner._configuration_value(field)

    def update(self, **changes: object) -> None:
        """Atomically update supplied stable node settings in project units."""
        self._owner._simulation._owner.update_node_settings(
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
            changes,
        )

    @property
    def tag(self) -> str:
        """Return the metadata tag."""
        return _cast(str, self._read("tag"))

    @tag.setter
    def tag(self, value: str) -> None:
        self.update(tag=value)

    @property
    def invert_elevation(self) -> float:
        """Return node invert elevation in configured project length units."""
        return _cast(float, self._read("invert_elevation"))

    @invert_elevation.setter
    def invert_elevation(self, value: float) -> None:
        self.update(invert_elevation=value)

    @property
    def full_depth(self) -> float:
        """Return node full depth in configured project length units."""
        return _cast(float, self._read("full_depth"))

    @full_depth.setter
    def full_depth(self, value: float) -> None:
        self.update(full_depth=value)

    @property
    def surcharge_depth(self) -> float:
        """Return node surcharge depth in configured project length units."""
        return _cast(float, self._read("surcharge_depth"))

    @surcharge_depth.setter
    def surcharge_depth(self, value: float) -> None:
        self.update(surcharge_depth=value)

    @property
    def ponded_area(self) -> float:
        """Return node ponded area in configured project area units."""
        return _cast(float, self._read("ponded_area"))

    @ponded_area.setter
    def ponded_area(self, value: float) -> None:
        self.update(ponded_area=value)

    @property
    def initial_depth(self) -> float:
        """Return node initial depth in configured project length units."""
        return _cast(float, self._read("initial_depth"))

    @initial_depth.setter
    def initial_depth(self, value: float) -> None:
        self.update(initial_depth=value)

    @property
    def included_in_report(self) -> bool:
        """Return whether detailed output includes this node."""
        return _cast(bool, self._read("included_in_report"))

    @included_in_report.setter
    def included_in_report(self, value: bool) -> None:
        self.update(included_in_report=value)


class Node(_LiveView):
    """Expose node identity, forcing, hydraulics, quality, and statistics.

    Notes
    -----
    The concrete view is `Junction`, `Outfall`, `StorageNode`, or `Divider`
    according to the configured node kind.
    """

    __slots__ = ()

    @property
    def kind(self) -> _NodeKind:
        """Return the configured node subtype."""
        self._validate()
        return _NodeKind(self._subtype)

    def _configuration_value(self, field: str) -> object:
        return self._simulation._owner.node_configuration_value(
            self._simulation,
            self._generation,
            self._index,
            self._id,
            self._subtype,
            field,
        )

    def _result(self, field: str) -> float:
        return self._simulation._owner.node_result(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def settings(self) -> NodeSettings:
        """Return stable common-node settings."""
        return NodeSettings(self)

    @property
    def external_inflow(self) -> float:
        """Additive external inflow. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return self._simulation._owner.node_external_inflow(
            self._generation, self._index, self._id, self._subtype
        )

    @external_inflow.setter
    def external_inflow(self, value: float) -> None:
        self._simulation._owner.set_node_external_inflow(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def depth(self) -> float:
        """Return current node depth in configured project length units."""
        return self._result("depth")

    @property
    def head(self) -> float:
        """Return current node head in configured project length units."""
        return self._result("head")

    @property
    def volume(self) -> float:
        """Return current node volume in configured project volume units."""
        return self._result("volume")

    @property
    def lateral_inflow(self) -> float:
        """Return current total lateral inflow in configured project flow units."""
        return self._result("lateral_inflow")

    @property
    def total_inflow(self) -> float:
        """Return current total node inflow in configured project flow units."""
        return self._result("total_inflow")

    @property
    def total_outflow(self) -> float:
        """Return current total node outflow in configured project flow units."""
        return self._result("total_outflow")

    @property
    def losses(self) -> float:
        """Return current node losses in configured project flow units."""
        return self._result("losses")

    @property
    def flooding(self) -> float:
        """Return current node flooding in configured project flow units."""
        return self._result("flooding")

    def _quality(self, field: str) -> tuple[list[str], list[float]]:
        return self._simulation._owner.node_quality_mapping(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def pollut_quality(self) -> _Mapping[str, float]:
        """Return immutable current concentrations by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("concentrations"))

    def override_pollutant_concentrations(
        self,
        values: _Mapping[str, float] | _Iterable[tuple[str | int, float]],
    ) -> None:
        """Atomically override concentrations for the next quality step. Lifecycle: `RUNNING`."""
        self._simulation._owner.override_node_pollutant_concentrations(
            self._generation, self._index, self._id, self._subtype, values
        )

    @property
    def inflow_pollutant_concentration(self) -> _Mapping[str, float]:
        """Return current inflow concentrations by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("inflow_concentrations"))

    @property
    def reactor_pollutant_concentration(self) -> _Mapping[str, float]:
        """Return current reactor concentrations by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("reactor_concentrations"))

    @property
    def external_pollutant_mass_flux(self) -> _MutableMapping[str, float]:
        """Return canonical persistent mass fluxes as a live mutable mapping.

        Item assignment and `update()` are atomic sparse mutations. Deletion resets
        one configured pollutant to zero; `clear()` atomically resets every pollutant.
        Mutation lifecycle: `OPEN`, `RUNNING`, `ENDED`.
        """
        return _NodeExternalPollutantMassFlux(self)

    @property
    def statistics(self) -> _snapshots.NodeStatistics:
        """Return immutable cumulative common-node statistics."""
        return self._simulation._owner.node_statistics(
            self._generation, self._index, self._id, self._subtype
        )

    @property
    def total_inflow_volume(self) -> float:
        """Return cumulative node inflow in configured project volume units."""
        return self._simulation._owner.node_total_inflow_volume(
            self._generation, self._index, self._id, self._subtype
        )


@_final
class Junction(Node):
    """Expose a generation-bound junction node."""

    __slots__ = ()


@_final
@_pretty_dataclass(frozen=True, slots=True)
class OutfallBoundary:
    """Store one mutually exclusive outfall boundary declaration."""

    kind: str
    stage: float = 0.0
    reference: Curve | TimeSeries | None = None


@_final
class OutfallSettings(NodeSettings):
    """Expose stable outfall settings and boundary replacement."""

    __slots__ = ()

    @property
    def boundary(self) -> OutfallBoundary:
        """Return the stable boundary declaration in project units."""
        kind, stage, reference = _cast(
            "tuple[str, float, Curve | TimeSeries | None]", self._read("boundary")
        )
        return OutfallBoundary(kind=kind, stage=stage, reference=reference)

    @boundary.setter
    def boundary(self, value: OutfallBoundary) -> None:
        self.update(boundary=value)

    @property
    def has_flap_gate(self) -> bool:
        """Return whether the outfall has a flap gate."""
        return _cast(bool, self._read("has_flap_gate"))

    @has_flap_gate.setter
    def has_flap_gate(self, value: bool) -> None:
        self.update(has_flap_gate=value)

    @property
    def route_to_subcatchment(self) -> Subcatchment | None:
        """Return the optional receiving subcatchment relationship."""
        return _cast("Subcatchment | None", self._read("route_to_subcatchment"))

    @route_to_subcatchment.setter
    def route_to_subcatchment(self, value: Subcatchment | None) -> None:
        self.update(route_to_subcatchment=value)


@_final
class Outfall(Node):
    """Expose a generation-bound outfall node and its runtime capabilities."""

    __slots__ = ()

    @property
    def settings(self) -> OutfallSettings:
        """Return stable outfall settings."""
        return OutfallSettings(self)

    @property
    def outfall_statistics(self) -> _snapshots.OutfallStatistics:
        """Return immutable cumulative outfall statistics."""
        return self._simulation._owner.outfall_statistics(
            self._generation, self._index, self._id, self._subtype
        )

    @property
    def fixed_stage(self) -> float | None:
        """Current fixed-stage override without replacing the stable boundary declaration."""
        return self._simulation._owner.outfall_fixed_stage(
            self._generation, self._index, self._id, self._subtype
        )

    @fixed_stage.setter
    def fixed_stage(self, value: float) -> None:
        self._simulation._owner.set_outfall_fixed_stage(
            self._generation, self._index, self._id, self._subtype, value
        )


@_final
@_pretty_dataclass(frozen=True, slots=True)
class StorageShape:
    """Store one round-trippable canonical storage surface declaration.

    `coefficients` are SWMM's stored C-compatible `a0`, `a1`, and `a2` values
    in configured project units. Tabular shapes use zero coefficients and a
    `Curve` Live View.
    """

    kind: str
    coefficients: tuple[float, float, float] = (0.0, 0.0, 0.0)
    curve: Curve | None = None


@_final
@_pretty_dataclass(frozen=True, slots=True)
class StorageExfiltration:
    """Store one storage Green-Ampt or seepage declaration in project units."""

    conductivity: float
    suction_head: float = 0.0
    moisture_deficit: float = 0.0


@_final
class StorageSettings(NodeSettings):
    """Expose stable storage-node settings and declarations."""

    __slots__ = ()

    @property
    def shape(self) -> StorageShape:
        """Return the canonical storage surface declaration."""
        kind, coefficients, curve = _cast(
            "tuple[str, tuple[float, float, float], Curve | None]",
            self._read("shape"),
        )
        return StorageShape(
            kind=kind,
            coefficients=(coefficients[0], coefficients[1], coefficients[2]),
            curve=curve,
        )

    @shape.setter
    def shape(self, value: StorageShape) -> None:
        self.update(shape=value)

    @property
    def evaporation_fraction(self) -> float:
        """Return the realized evaporation fraction."""
        return _cast(float, self._read("evaporation_fraction"))

    @evaporation_fraction.setter
    def evaporation_fraction(self, value: float) -> None:
        self.update(evaporation_fraction=value)

    @property
    def exfiltration(self) -> StorageExfiltration | None:
        """Return the optional storage exfiltration declaration."""
        values = _cast(tuple[float, float, float] | None, self._read("exfiltration"))
        if values is None:
            return None
        return StorageExfiltration(
            suction_head=values[0],
            conductivity=values[1],
            moisture_deficit=values[2],
        )

    @exfiltration.setter
    def exfiltration(self, value: StorageExfiltration | None) -> None:
        self.update(exfiltration=value)


@_final
class StorageNode(Node):
    """Expose a generation-bound storage node and its runtime capabilities."""

    __slots__ = ()

    @property
    def settings(self) -> StorageSettings:
        """Return stable storage-node settings."""
        return StorageSettings(self)

    @property
    def storage_statistics(self) -> _snapshots.StorageStatistics:
        """Return immutable cumulative storage statistics."""
        return self._simulation._owner.storage_statistics(
            self._generation, self._index, self._id, self._subtype
        )

    @property
    def hydraulic_retention_time(self) -> _timedelta:
        """Return current storage hydraulic retention time."""
        return _timedelta(seconds=self._result("hydraulic_retention_seconds"))


@_final
@_pretty_dataclass(frozen=True, slots=True)
class DividerRule:
    """Store one mutually exclusive divider rule in project units."""

    kind: str
    values: tuple[float, float, float] = (0.0, 0.0, 0.0)
    curve: Curve | None = None


@_final
class DividerSettings(NodeSettings):
    """Expose stable flow-divider settings and rule replacement."""

    __slots__ = ()

    @property
    def rule(self) -> DividerRule:
        """Return the stable divider rule in project units."""
        kind, values, curve = _cast(
            "tuple[str, tuple[float, float, float], Curve | None]",
            self._read("rule"),
        )
        return DividerRule(
            kind=kind,
            values=(values[0], values[1], values[2]),
            curve=curve,
        )

    @rule.setter
    def rule(self, value: DividerRule) -> None:
        self.update(rule=value)

    @property
    def diverted_link(self) -> Link | None:
        """Return the optional diverted-link relationship."""
        return _cast("Link | None", self._read("diverted_link"))

    @diverted_link.setter
    def diverted_link(self, value: Link | None) -> None:
        self.update(diverted_link=value)


@_final
class Divider(Node):
    """Expose a generation-bound flow-divider node."""

    __slots__ = ()

    @property
    def settings(self) -> DividerSettings:
        """Return stable divider settings."""
        return DividerSettings(self)


@_final
class NodeCollection(_Collection[Node]):
    """Provide node lookup and aligned hydraulic, quality, and statistics snapshots."""

    __slots__ = ()

    def __getitem__(self, key: str | int) -> Node:
        """Return a node view by case-insensitive configured ID."""
        return super().__getitem__(key)

    def by_index(self, index: int) -> Node:
        """Return a node view by its zero-based configured position."""
        return super().by_index(index)

    def snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.NodeSnapshot:
        """Copy current node hydraulics in canonical or requested order."""
        return self._simulation._owner.node_snapshot(self._generation, ids)

    def quality_snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.NodeQualitySnapshot:
        """Copy current node quality in canonical or requested order."""
        return self._simulation._owner.node_quality_snapshot(self._generation, ids)

    def statistics(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.NodeStatisticsSnapshot:
        """Copy cumulative node statistics in canonical or requested order."""
        return self._simulation._owner.node_statistics_snapshot(self._generation, ids)
