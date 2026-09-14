"""Link and inlet views with stable settings namespaces."""

from __future__ import annotations as _annotations

from collections.abc import (
    Iterable as _Iterable,
    Iterator as _Iterator,
    Mapping as _Mapping,
    MutableMapping as _MutableMapping,
)
from dataclasses import dataclass as _dataclass
from datetime import timedelta as _timedelta
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    Literal as _Literal,
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    cast as _cast,
    final as _final,
)

from .. import snapshots as _snapshots
from .._prettier import pretty_dataclass as _pretty_dataclass
from ..enums import (
    LinkKind as _LinkKind,
    OrificeKind,
    OutletHeadBasis,
    RoadSurface,
    StandardCrossSectionShape,
    WeirKind,
)
from ._base import _INLET_TOKEN, _LiveView, _pollutant_mapping
from ._collections import _Collection
from ._definitions import Curve, CustomShape, InletDesign, Street, Transect
from ._nodes import Node

if _TYPE_CHECKING:
    from .. import Simulation


@_final
@_pretty_dataclass(frozen=True, slots=True)
class FunctionalOutletRating:
    """Store a functional outlet rating in project units."""

    coefficient: float
    exponent: float
    head_basis: OutletHeadBasis = OutletHeadBasis.DEPTH


@_final
@_pretty_dataclass(frozen=True, slots=True)
class TabularOutletRating:
    """Store a curve-driven outlet rating."""

    curve: Curve
    head_basis: OutletHeadBasis = OutletHeadBasis.DEPTH


@_final
@_pretty_dataclass(frozen=True, slots=True)
class CircularCrossSection:
    """Store a circular conduit cross-section in project length units."""

    diameter: float


@_final
@_pretty_dataclass(frozen=True, slots=True)
class StandardCrossSection:
    """Store a standard SWMM cross-section declaration.

    Geometry follows SWMM's four XSECTION parameters. Dimensional entries use
    project length units; side counts, side slopes, power exponents, catalog
    size codes, force-main roughness/C-factors, and unused entries do not.
    """

    shape: StandardCrossSectionShape
    geometry: tuple[float, float, float, float]
    culvert_code: int = 0


@_final
@_pretty_dataclass(frozen=True, slots=True)
class CustomCrossSection:
    """Store a custom-shape cross-section relationship and full depth."""

    shape: CustomShape
    full_depth: float
    culvert_code: int = 0


@_final
@_pretty_dataclass(frozen=True, slots=True)
class IrregularCrossSection:
    """Store an irregular-transect cross-section relationship."""

    transect: Transect
    culvert_code: int = 0


@_final
@_pretty_dataclass(frozen=True, slots=True)
class StreetCrossSection:
    """Store a street cross-section relationship."""

    street: Street
    culvert_code: int = 0


_CrossSection = (
    CircularCrossSection
    | StandardCrossSection
    | CustomCrossSection
    | IrregularCrossSection
    | StreetCrossSection
)
_OutletRating = FunctionalOutletRating | TabularOutletRating

_ORIFICE_KINDS = (OrificeKind.SIDE, OrificeKind.BOTTOM)
_WEIR_KINDS = (
    WeirKind.TRANSVERSE,
    WeirKind.SIDEFLOW,
    WeirKind.V_NOTCH,
    WeirKind.TRAPEZOIDAL,
    WeirKind.ROADWAY,
)
_ROAD_SURFACES = (RoadSurface.UNSPECIFIED, RoadSurface.PAVED, RoadSurface.GRAVEL)
_STANDARD_SHAPES: tuple[StandardCrossSectionShape | None, ...] = (
    StandardCrossSectionShape.DUMMY,
    StandardCrossSectionShape.CIRCULAR,
    StandardCrossSectionShape.FILLED_CIRCULAR,
    StandardCrossSectionShape.RECT_CLOSED,
    StandardCrossSectionShape.RECT_OPEN,
    StandardCrossSectionShape.TRAPEZOIDAL,
    StandardCrossSectionShape.TRIANGULAR,
    StandardCrossSectionShape.PARABOLIC,
    StandardCrossSectionShape.POWER_FUNCTION,
    StandardCrossSectionShape.RECT_TRIANGULAR,
    StandardCrossSectionShape.RECT_ROUND,
    StandardCrossSectionShape.MODIFIED_BASKET,
    StandardCrossSectionShape.HORIZONTAL_ELLIPSE,
    StandardCrossSectionShape.VERTICAL_ELLIPSE,
    StandardCrossSectionShape.ARCH,
    StandardCrossSectionShape.EGG_SHAPED,
    StandardCrossSectionShape.HORSESHOE,
    StandardCrossSectionShape.GOTHIC,
    StandardCrossSectionShape.CATENARY,
    StandardCrossSectionShape.SEMI_ELLIPTICAL,
    StandardCrossSectionShape.BASKET_HANDLE,
    StandardCrossSectionShape.SEMI_CIRCULAR,
    None,
    None,
    StandardCrossSectionShape.FORCE_MAIN,
    None,
)


class LinkSettings:
    """Expose stable common-link settings through one atomic owner."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Link) -> None:
        self._owner = owner

    def _read(self, field: str) -> object:
        return self._owner._configuration_value(field)

    def update(self, **changes: object) -> None:
        """Atomically update supplied stable link settings."""
        self._owner._simulation._owner.update_link_settings(
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
    def inlet_node(self) -> Node:
        """Return the generation-bound upstream node relationship."""
        return _cast(Node, self._read("inlet_node"))

    @inlet_node.setter
    def inlet_node(self, value: Node) -> None:
        self.update(inlet_node=value)

    @property
    def outlet_node(self) -> Node:
        """Return the generation-bound downstream node relationship."""
        return _cast(Node, self._read("outlet_node"))

    @outlet_node.setter
    def outlet_node(self, value: Node) -> None:
        self.update(outlet_node=value)

    @property
    def flow_direction(self) -> _Literal[-1, 1]:
        """Return the configured numeric orientation sign."""
        return _cast(_Literal[-1, 1], self._read("flow_direction"))

    @property
    def full_depth(self) -> float:
        """Return full depth in project length units."""
        return _cast(float, self._read("full_depth"))

    @property
    def full_flow(self) -> float:
        """Return full flow in project flow units."""
        return _cast(float, self._read("full_flow"))

    @property
    def included_in_report(self) -> bool:
        """Return whether detailed output includes this link."""
        return _cast(bool, self._read("included_in_report"))

    @included_in_report.setter
    def included_in_report(self, value: bool) -> None:
        self.update(included_in_report=value)

    @property
    def inlet_offset(self) -> float:
        """Return inlet offset in project length units."""
        return _cast(float, self._read("inlet_offset"))

    @inlet_offset.setter
    def inlet_offset(self, value: float) -> None:
        self.update(inlet_offset=value)

    @property
    def outlet_offset(self) -> float:
        """Return outlet offset in project length units."""
        return _cast(float, self._read("outlet_offset"))

    @outlet_offset.setter
    def outlet_offset(self, value: float) -> None:
        self.update(outlet_offset=value)

    @property
    def initial_flow(self) -> float:
        """Return initial flow in project flow units."""
        return _cast(float, self._read("initial_flow"))

    @initial_flow.setter
    def initial_flow(self, value: float) -> None:
        self.update(initial_flow=value)

    @property
    def inlet_loss_coefficient(self) -> float:
        """Return the dimensionless inlet loss coefficient."""
        return _cast(float, self._read("inlet_loss_coefficient"))

    @inlet_loss_coefficient.setter
    def inlet_loss_coefficient(self, value: float) -> None:
        self.update(inlet_loss_coefficient=value)

    @property
    def outlet_loss_coefficient(self) -> float:
        """Return the dimensionless outlet loss coefficient."""
        return _cast(float, self._read("outlet_loss_coefficient"))

    @outlet_loss_coefficient.setter
    def outlet_loss_coefficient(self, value: float) -> None:
        self.update(outlet_loss_coefficient=value)

    @property
    def average_loss_coefficient(self) -> float:
        """Return the dimensionless average loss coefficient."""
        return _cast(float, self._read("average_loss_coefficient"))

    @average_loss_coefficient.setter
    def average_loss_coefficient(self, value: float) -> None:
        self.update(average_loss_coefficient=value)

    @property
    def seepage_rate(self) -> float:
        """Return seepage rate in project rainfall units."""
        return _cast(float, self._read("seepage_rate"))

    @seepage_rate.setter
    def seepage_rate(self, value: float) -> None:
        self.update(seepage_rate=value)

    @property
    def has_flap_gate(self) -> bool:
        """Return whether the link has a flap gate."""
        return _cast(bool, self._read("has_flap_gate"))

    @has_flap_gate.setter
    def has_flap_gate(self, value: bool) -> None:
        self.update(has_flap_gate=value)


@_final
class ConduitSettings(LinkSettings):
    """Expose stable conduit settings and cross-section replacement."""

    __slots__ = ()

    @property
    def length(self) -> float:
        """Return conduit length in project length units."""
        return _cast(float, self._read("length"))

    @length.setter
    def length(self, value: float) -> None:
        self.update(length=value)

    @property
    def roughness(self) -> float:
        """Return conduit Manning roughness."""
        return _cast(float, self._read("roughness"))

    @roughness.setter
    def roughness(self, value: float) -> None:
        self.update(roughness=value)

    @property
    def barrels(self) -> int:
        """Return the positive number of identical conduit barrels."""
        return _cast(int, self._read("barrels"))

    @barrels.setter
    def barrels(self, value: int) -> None:
        self.update(barrels=value)

    @property
    def cross_section(self) -> _CrossSection | None:
        """Return the complete typed cross-section declaration, when retained."""
        parts = _cast(
            tuple[
                str,
                int | None,
                tuple[float, float, float, float],
                CustomShape | Transect | Street | None,
                float | None,
                int,
            ]
            | None,
            self._read("cross_section"),
        )
        if parts is None:
            return None
        kind, shape_code, geometry, reference, full_depth, culvert_code = parts
        if kind == "circular":
            return CircularCrossSection(geometry[0])
        if kind == "standard":
            if shape_code is None or _STANDARD_SHAPES[shape_code] is None:
                raise RuntimeError("native standard cross-section shape is invalid")
            return StandardCrossSection(
                _cast(StandardCrossSectionShape, _STANDARD_SHAPES[shape_code]),
                (geometry[0], geometry[1], geometry[2], geometry[3]),
                culvert_code,
            )
        if kind == "custom":
            return CustomCrossSection(
                _cast(CustomShape, reference),
                _cast(float, full_depth),
                culvert_code,
            )
        if kind == "irregular":
            return IrregularCrossSection(
                _cast(Transect, reference),
                culvert_code,
            )
        if kind == "street":
            return StreetCrossSection(
                _cast(Street, reference),
                culvert_code,
            )
        raise RuntimeError("native cross-section kind is invalid")

    @cross_section.setter
    def cross_section(self, value: _CrossSection) -> None:
        self.update(cross_section=value)

    @property
    def slope(self) -> float:
        """Return conduit slope as a fraction."""
        return _cast(float, self._read("slope"))


@_final
class PumpSettings(LinkSettings):
    """Expose stable pump settings and explicit operating-mode transitions."""

    __slots__ = ()

    @property
    def curve(self) -> Curve | None:
        """Return the pump curve, or `None` while operating as an ideal pump."""
        return _cast(Curve | None, self._read("curve"))

    def use_curve(self, curve: Curve) -> None:
        """Switch to curve-driven operation while preserving scalar settings."""
        self._owner._simulation._owner.use_pump_curve(
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
            curve,
        )

    def use_ideal(self) -> None:
        """Switch to ideal operation while preserving scalar settings."""
        self._owner._simulation._owner.use_ideal_pump(
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
        )

    @property
    def initial_setting(self) -> float:
        """Return the initial dimensionless pump speed setting."""
        return _cast(float, self._read("initial_setting"))

    @initial_setting.setter
    def initial_setting(self, value: float) -> None:
        self.update(initial_setting=value)

    @property
    def startup_depth(self) -> float:
        """Return pump startup depth in project length units."""
        return _cast(float, self._read("startup_depth"))

    @startup_depth.setter
    def startup_depth(self, value: float) -> None:
        self.update(startup_depth=value)

    @property
    def shutoff_depth(self) -> float:
        """Return pump shutoff depth in project length units."""
        return _cast(float, self._read("shutoff_depth"))

    @shutoff_depth.setter
    def shutoff_depth(self, value: float) -> None:
        self.update(shutoff_depth=value)


@_final
class OrificeSettings(LinkSettings):
    """Expose stable orifice settings."""

    __slots__ = ()

    @property
    def kind(self) -> OrificeKind:
        """Return the orifice orientation."""
        return _ORIFICE_KINDS[_cast(int, self._read("orifice_kind"))]

    @kind.setter
    def kind(self, value: OrificeKind) -> None:
        self.update(kind=value)

    @property
    def discharge_coefficient(self) -> float:
        """Return the dimensionless orifice discharge coefficient."""
        return _cast(float, self._read("orifice_discharge_coefficient"))

    @discharge_coefficient.setter
    def discharge_coefficient(self, value: float) -> None:
        self.update(discharge_coefficient=value)

    @property
    def opening_time_hours(self) -> float:
        """Return the hours required to fully open or close the orifice."""
        return _cast(float, self._read("opening_time_hours"))

    @opening_time_hours.setter
    def opening_time_hours(self, value: float) -> None:
        self.update(opening_time_hours=value)


@_final
class WeirSettings(LinkSettings):
    """Expose stable weir settings."""

    __slots__ = ()

    @property
    def kind(self) -> WeirKind:
        """Return the weir geometry and flow family."""
        return _WEIR_KINDS[_cast(int, self._read("weir_kind"))]

    @kind.setter
    def kind(self, value: WeirKind) -> None:
        self.update(kind=value)

    @property
    def discharge_coefficient(self) -> float:
        """Return the primary weir discharge coefficient."""
        return _cast(float, self._read("weir_discharge_coefficient"))

    @discharge_coefficient.setter
    def discharge_coefficient(self, value: float) -> None:
        self.update(discharge_coefficient=value)

    @property
    def end_discharge_coefficient(self) -> float:
        """Return the weir end-section discharge coefficient."""
        return _cast(float, self._read("end_discharge_coefficient"))

    @end_discharge_coefficient.setter
    def end_discharge_coefficient(self, value: float) -> None:
        self.update(end_discharge_coefficient=value)

    @property
    def end_contractions(self) -> float:
        """Return the number of weir end contractions."""
        return _cast(float, self._read("end_contractions"))

    @end_contractions.setter
    def end_contractions(self, value: float) -> None:
        self.update(end_contractions=value)

    @property
    def can_surcharge(self) -> bool:
        """Return whether the weir may surcharge."""
        return _cast(bool, self._read("can_surcharge"))

    @can_surcharge.setter
    def can_surcharge(self, value: bool) -> None:
        self.update(can_surcharge=value)

    @property
    def roadway_width(self) -> float:
        """Return roadway width in project length units."""
        return _cast(float, self._read("roadway_width"))

    @roadway_width.setter
    def roadway_width(self, value: float) -> None:
        self.update(roadway_width=value)

    @property
    def roadway_surface(self) -> RoadSurface:
        """Return the roadway-weir surface material."""
        return _ROAD_SURFACES[_cast(int, self._read("roadway_surface"))]

    @roadway_surface.setter
    def roadway_surface(self, value: RoadSurface) -> None:
        self.update(roadway_surface=value)

    @property
    def coefficient_curve(self) -> Curve | None:
        """Return the optional weir coefficient-curve relationship."""
        return _cast(Curve | None, self._read("coefficient_curve"))

    @coefficient_curve.setter
    def coefficient_curve(self, value: Curve | None) -> None:
        self.update(coefficient_curve=value)


@_final
class OutletSettings(LinkSettings):
    """Expose stable outlet-rating settings."""

    __slots__ = ()

    @property
    def rating(self) -> _OutletRating:
        """Return the complete functional or tabular outlet rating."""
        kind, coefficient, exponent, curve, head_basis = _cast(
            tuple[str, float | None, float | None, Curve | None, str],
            self._read("rating"),
        )
        basis = OutletHeadBasis(head_basis)
        if kind == "functional":
            return FunctionalOutletRating(
                _cast(float, coefficient), _cast(float, exponent), basis
            )
        return TabularOutletRating(_cast(Curve, curve), basis)

    @rating.setter
    def rating(self, value: _OutletRating) -> None:
        self.update(rating=value)


class _LinkExternalPollutantMassFlux(_MutableMapping[str, float]):
    """Expose persistent link pollutant forcing as a canonical live mapping."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Link) -> None:
        self._owner = owner

    def _snapshot(self) -> tuple[list[str], list[float]]:
        owner = self._owner
        return owner._simulation._owner.link_external_pollutant_mass_flux(
            owner._generation, owner._index, owner._id, owner._subtype
        )

    def _mutate(
        self,
        replace: bool,
        args: tuple[object, ...],
        kwargs: dict[str, float],
    ) -> None:
        owner = self._owner
        owner._simulation._owner.update_link_external_pollutant_mass_flux(
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
        return owner._simulation._owner.link_external_pollutant_mass_flux_value(
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


class Link(_LiveView):
    """Expose link identity, stable settings, runtime controls, and current results."""

    __slots__ = ()

    def _configuration_value(self, field: str) -> object:
        return self._simulation._owner.link_configuration_value(
            self._simulation,
            self._generation,
            self._index,
            self._id,
            self._subtype,
            field,
        )

    def _result(self, field: str) -> float | None:
        return self._simulation._owner.link_result(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def settings(self) -> LinkSettings:
        """Return stable common-link settings."""
        return LinkSettings(self)

    @property
    def kind(self) -> _LinkKind:
        """Return the configured link subtype."""
        self._validate()
        return _LinkKind(self._subtype)

    def _quality(self, field: str) -> tuple[list[str], list[float]]:
        return self._simulation._owner.link_quality_mapping(
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
        self._simulation._owner.override_link_pollutant_concentrations(
            self._generation, self._index, self._id, self._subtype, values
        )

    @property
    def reactor_pollutant_concentration(self) -> _Mapping[str, float]:
        """Return current pre-override reactor concentrations."""
        return _pollutant_mapping(self._quality("reactor_concentrations"))

    @property
    def pollutant_total_load(self) -> _Mapping[str, float]:
        """Return current total routed loads in reporting units."""
        return _pollutant_mapping(self._quality("total_loads"))

    @property
    def external_pollutant_mass_flux(self) -> _MutableMapping[str, float]:
        """Return canonical persistent mass fluxes as a live mutable mapping.

        Item assignment and `update()` are atomic sparse mutations. Deletion resets
        one configured pollutant to zero; `clear()` atomically resets every pollutant.
        Nonzero values require a non-dummy conduit; other link types accept only an
        all-zero candidate.
        Mutation lifecycle: `OPEN`, `RUNNING`, `ENDED`.
        """
        return _LinkExternalPollutantMassFlux(self)

    @property
    def inlet(self) -> Inlet | None:
        """Return this link's generation-bound inlet placement, when configured."""
        return _cast(
            Inlet | None,
            self._simulation._owner.link_inlet(
                self._simulation,
                self._generation,
                self._index,
                self._id,
                self._subtype,
            ),
        )

    @property
    def flow_limit(self) -> float:
        """Flow limit. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return _cast(float, self._configuration_value("flow_limit"))

    @flow_limit.setter
    def flow_limit(self, value: float) -> None:
        self._simulation._owner.set_link_flow_limit(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def setting(self) -> float:
        """Return the current setting or pump speed factor."""
        return _cast(float, self._result("setting"))

    @property
    def target_setting(self) -> float:
        """Target setting or pump speed. Setter lifecycle: `RUNNING`."""
        return _cast(float, self._result("target_setting"))

    @target_setting.setter
    def target_setting(self, value: float) -> None:
        self._simulation._owner.set_link_target_setting(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def time_open(self) -> _timedelta:
        """Return current continuously-open duration."""
        return _timedelta(seconds=_cast(float, self._result("time_open_seconds")))

    @property
    def time_closed(self) -> _timedelta:
        """Return current continuously-closed duration."""
        return _timedelta(seconds=_cast(float, self._result("time_closed_seconds")))

    @property
    def flow(self) -> float:
        """Return signed current flow in project flow units."""
        return _cast(float, self._result("flow"))

    @property
    def depth(self) -> float:
        """Return current depth in project length units."""
        return _cast(float, self._result("depth"))

    @property
    def velocity(self) -> float:
        """Return current velocity in project length units per second."""
        return _cast(float, self._result("velocity"))

    @property
    def volume(self) -> float:
        """Return current volume in project volume units."""
        return _cast(float, self._result("volume"))

    @property
    def capacity(self) -> float:
        """Return native subtype capacity fraction or current setting."""
        return _cast(float, self._result("capacity"))

    @property
    def upstream_surface_area(self) -> float:
        """Return current upstream surface area in project area units."""
        return _cast(float, self._result("upstream_surface_area"))

    @property
    def downstream_surface_area(self) -> float:
        """Return current downstream surface area in project area units."""
        return _cast(float, self._result("downstream_surface_area"))

    @property
    def froude_number(self) -> float:
        """Return the current dimensionless Froude number."""
        return _cast(float, self._result("froude_number"))

    @property
    def statistics(self) -> _snapshots.LinkStatistics:
        """Return immutable cumulative common-link statistics."""
        return self._simulation._owner.link_statistics(
            self._generation, self._index, self._id, self._subtype
        )


@_final
class Conduit(Link):
    """Expose a conduit link with stable settings and current hydraulics."""

    __slots__ = ()

    @property
    def settings(self) -> ConduitSettings:
        """Return stable conduit settings."""
        return ConduitSettings(self)

    @property
    def top_width(self) -> float:
        """Return current conduit top width in project length units."""
        return _cast(float, self._result("top_width"))


@_final
class Pump(Link):
    """Expose a pump link with stable settings and current hydraulics."""

    __slots__ = ()

    @property
    def settings(self) -> PumpSettings:
        """Return stable pump settings."""
        return PumpSettings(self)

    @property
    def pump_statistics(self) -> _snapshots.PumpStatistics:
        """Return immutable cumulative pump statistics."""
        return self._simulation._owner.pump_statistics(
            self._generation, self._index, self._id, self._subtype
        )


@_final
class Orifice(Link):
    """Expose an orifice link with stable settings and current hydraulics."""

    __slots__ = ()

    @property
    def settings(self) -> OrificeSettings:
        """Return stable orifice settings."""
        return OrificeSettings(self)


@_final
class Weir(Link):
    """Expose a weir link with stable settings and current hydraulics."""

    __slots__ = ()

    @property
    def settings(self) -> WeirSettings:
        """Return stable weir settings."""
        return WeirSettings(self)


@_final
class Outlet(Link):
    """Expose an outlet link with stable settings and current hydraulics."""

    __slots__ = ()

    @property
    def settings(self) -> OutletSettings:
        """Return stable outlet settings."""
        return OutletSettings(self)


@_final
class InletSettings:
    """Expose the stable read-only placement settings of one link-local inlet."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Inlet) -> None:
        self._owner = owner

    @property
    def count(self) -> int:
        """Return the positive number of identical inlet structures."""
        return _cast(int, self._owner._configuration_value("count"))

    @property
    def local_depression(self) -> float:
        """Return local depression depth in project length units."""
        return _cast(float, self._owner._configuration_value("local_depression"))

    @property
    def local_width(self) -> float:
        """Return local depression width in project length units."""
        return _cast(float, self._owner._configuration_value("local_width"))

    @property
    def design(self) -> InletDesign:
        """Return the generation-bound inlet-design relationship."""
        return _cast(InletDesign, self._owner._configuration_value("design"))


@_final
class Inlet:
    """Expose persistent controls and current results for one link-local inlet."""

    __slots__ = (
        "_design_id",
        "_design_index",
        "_generation",
        "_link_id",
        "_link_index",
        "_link_subtype",
        "_simulation",
    )

    def __init__(
        self,
        simulation: Simulation,
        generation: int,
        link_index: int,
        link_identifier: str,
        link_subtype: str,
        design_index: int,
        design_identifier: str,
        token: object,
    ) -> None:
        if token is not _INLET_TOKEN:
            raise TypeError("Inlet views are created by Link")
        self._simulation = simulation
        self._generation = generation
        self._link_index = link_index
        self._link_id = link_identifier
        self._link_subtype = link_subtype
        self._design_index = design_index
        self._design_id = design_identifier

    def _configuration_value(self, field: str) -> object:
        return self._simulation._owner.inlet_configuration_value(
            self._simulation,
            self._generation,
            self._link_index,
            self._link_id,
            self._link_subtype,
            self._design_index,
            self._design_id,
            field,
        )

    def _persistent_configuration_value(self, field: str) -> object:
        return self._simulation._owner.inlet_persistent_configuration_value(
            self._simulation,
            self._generation,
            self._link_index,
            self._link_id,
            self._link_subtype,
            self._design_index,
            self._design_id,
            field,
        )

    def _result(self, field: str) -> float:
        return self._simulation._owner.inlet_result(
            self._generation,
            self._link_index,
            self._link_id,
            self._link_subtype,
            self._design_index,
            self._design_id,
            field,
        )

    @property
    def percent_clogged(self) -> float:
        """Inlet clogging percentage. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return _cast(float, self._persistent_configuration_value("percent_clogged"))

    @percent_clogged.setter
    def percent_clogged(self, value: float) -> None:
        self.update(percent_clogged=value)

    @property
    def flow_limit(self) -> float:
        """Per-inlet flow limit. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return _cast(float, self._persistent_configuration_value("flow_limit"))

    @flow_limit.setter
    def flow_limit(self, value: float) -> None:
        self.update(flow_limit=value)

    @property
    def flow_factor(self) -> float:
        """Return the current dimensionless capture flow factor."""
        return self._result("flow_factor")

    @property
    def captured_flow(self) -> float:
        """Return current captured flow in project flow units."""
        return self._result("captured_flow")

    @property
    def backflow(self) -> float:
        """Return current backflow in project flow units."""
        return self._result("backflow")

    @property
    def backflow_ratio(self) -> float:
        """Return the current dimensionless inlet backflow ratio."""
        return self._result("backflow_ratio")

    @property
    def _identity(self) -> tuple[object, ...]:
        return (
            id(self._simulation),
            self._generation,
            Inlet,
            self._link_index,
            self._link_id,
            self._link_subtype,
            self._design_index,
            self._design_id,
        )

    def __repr__(self) -> str:
        return (
            f"{type(self).__name__}(link_id={self._link_id!r}, "
            f"link_index={self._link_index}, design_id={self._design_id!r}, "
            f"design_index={self._design_index})"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Inlet):
            return NotImplemented
        return self._identity == other._identity

    def __hash__(self) -> int:
        return hash(self._identity)

    def __copy__(self) -> _Never:
        raise TypeError("Inlet cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError("Inlet cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError("Inlet cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError("Inlet cannot be pickled")

    @property
    def settings(self) -> InletSettings:
        """Return stable read-only inlet placement settings."""
        return InletSettings(self)

    def update(self, **changes: object) -> None:
        """Atomically update supplied persistent inlet controls."""
        self._simulation._owner.update_inlet(
            self._generation,
            self._link_index,
            self._link_id,
            self._link_subtype,
            self._design_index,
            self._design_id,
            changes,
        )


@_final
class LinkCollection(_Collection[Link]):
    """Provide link lookup and aligned hydraulic, quality, and statistics snapshots."""

    __slots__ = ()

    def __getitem__(self, key: str | int) -> Link:
        """Return a link view by case-insensitive configured ID."""
        return super().__getitem__(key)

    def by_index(self, index: int) -> Link:
        """Return a link view by its zero-based configured position."""
        return super().by_index(index)

    def snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.LinkSnapshot:
        """Copy current link hydraulics in canonical or requested order."""
        return self._simulation._owner.link_snapshot(self._generation, ids)

    def quality_snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.LinkQualitySnapshot:
        """Copy current link quality in canonical or requested order."""
        return self._simulation._owner.link_quality_snapshot(self._generation, ids)

    def statistics(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.LinkStatisticsSnapshot:
        """Copy cumulative link statistics in canonical or requested order."""
        return self._simulation._owner.link_statistics_snapshot(self._generation, ids)
