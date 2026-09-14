"""Rain gage, subcatchment, and subcatchment-owned LID unit views."""

from __future__ import annotations as _annotations

from collections.abc import (
    Iterable as _Iterable,
    Iterator as _Iterator,
    Mapping as _Mapping,
    MutableMapping as _MutableMapping,
    Sequence as _Sequence,
)
from typing import (
    TYPE_CHECKING,
    ClassVar as _ClassVar,
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    cast as _cast,
    final as _final,
    overload as _overload,
)

from .. import snapshots as _snapshots
from .._prettier import pretty_dataclass as _pretty_dataclass
from ..enums import InfilKind as _InfilKind, OutKind as _OutKind
from ..exceptions import ValidationError as _ValidationError
from ._base import _COLLECTION_TOKEN, _VIEW_TOKEN, _LiveView, _pollutant_mapping
from ._collections import _Collection

if TYPE_CHECKING:
    from ._definitions import Aquifer, LandUse, Pollutant, SnowmeltParameterSet
    from ._lids import LidControl
    from ._nodes import Node, NodeCollection


@_pretty_dataclass(slots=True)
class _HortonInfiltrationSettings:
    """Store one detached Horton-family infiltration declaration."""

    initial_rate: float
    minimum_rate: float
    decay_coefficient: float
    drying_time_days: float
    maximum_infiltration: float | None = None


@_final
class HortonInfiltrationSettings(_HortonInfiltrationSettings):
    """Store detached Horton infiltration settings."""

    __slots__ = ()
    kind: _ClassVar[_InfilKind] = _InfilKind.HORTON


@_final
class ModifiedHortonInfiltrationSettings(_HortonInfiltrationSettings):
    """Store detached modified-Horton infiltration settings."""

    __slots__ = ()
    kind: _ClassVar[_InfilKind] = _InfilKind.MODIFIED_HORTON


@_pretty_dataclass(slots=True)
class _GreenAmptInfiltrationSettings:
    """Store one detached Green-Ampt-family infiltration declaration."""

    suction_head: float
    hydraulic_conductivity: float
    initial_moisture_deficit: float


@_final
class GreenAmptInfiltrationSettings(_GreenAmptInfiltrationSettings):
    """Store detached Green-Ampt infiltration settings."""

    __slots__ = ()
    kind: _ClassVar[_InfilKind] = _InfilKind.GREEN_AMPT


@_final
class ModifiedGreenAmptInfiltrationSettings(_GreenAmptInfiltrationSettings):
    """Store detached modified Green-Ampt infiltration settings."""

    __slots__ = ()
    kind: _ClassVar[_InfilKind] = _InfilKind.MODIFIED_GREEN_AMPT


@_final
@_pretty_dataclass(slots=True)
class CurveNumberInfiltrationSettings:
    """Store detached Curve Number infiltration settings."""

    curve_number: float
    drying_time_days: float
    kind: _ClassVar[_InfilKind] = _InfilKind.CURVE_NUMBER


_SubcatchmentInfiltrationSettings = (
    HortonInfiltrationSettings
    | ModifiedHortonInfiltrationSettings
    | GreenAmptInfiltrationSettings
    | ModifiedGreenAmptInfiltrationSettings
    | CurveNumberInfiltrationSettings
)


@_final
@_pretty_dataclass(slots=True)
class SubcatchmentGroundwaterSettings:
    """Store one detached subcatchment groundwater declaration."""

    aquifer: Aquifer
    node: Node
    surface_elevation: float
    groundwater_coefficient: float
    groundwater_exponent: float
    surface_coefficient: float
    surface_exponent: float
    interaction_coefficient: float
    fixed_surface_depth: float
    bottom_elevation: float
    water_table_elevation: float
    upper_moisture: float


@_final
class SubcatchmentSettings:
    """Expose declaration-backed stable configuration as a live project-unit view.

    Dependent edits remain readable immediately and are relationally validated by
    :meth:`Simulation.start`, which can raise :class:`swmmrs.ConfigurationError`.
    """

    __slots__ = ("_owner",)

    def __init__(self, owner: Subcatchment) -> None:
        self._owner = owner

    def _read(self, field: str) -> object:
        owner = self._owner
        return owner._simulation._owner.subcatchment_configuration_value(
            owner._simulation, owner._generation, owner._index, owner._id, owner._subtype, field
        )

    def update(self, **changes: object) -> None:
        """Atomically retain supplied stable fields in project units.

        Intrinsic errors fail here. Cross-object and aggregate relationships are
        checked when the simulation starts, and rejected declarations remain
        available for corrective edits and retry.
        """
        owner = self._owner
        owner._simulation._owner.update_subcatchment(
            owner._generation, owner._index, owner._id, owner._subtype, changes
        )

    @property
    def tag(self) -> str:
        """Return the metadata tag."""
        return _cast(str, self._read("tag"))

    @tag.setter
    def tag(self, value: str) -> None:
        self.update(tag=value)

    @property
    def area(self) -> float:
        """Return the area in project land-area units."""
        return _cast(float, self._read("area"))

    @area.setter
    def area(self, value: float) -> None:
        self.update(area=value)

    @property
    def rain_gage(self) -> RainGage:
        """Return the requested rain-gage relationship.

        Assignment is declaration-backed. Usage, duplicate-series sharing, and
        effective simulation steps are rebuilt at the next successful start.
        """
        return _cast(RainGage, self._read("rain_gage"))

    @rain_gage.setter
    def rain_gage(self, value: RainGage) -> None:
        self.update(rain_gage=value)

    @property
    def included_in_report(self) -> bool:
        """Return whether detailed results are reported."""
        return _cast(bool, self._read("included_in_report"))

    @included_in_report.setter
    def included_in_report(self, value: bool) -> None:
        self.update(included_in_report=value)

    @property
    def width(self) -> float:
        """Return characteristic overland-flow width in project length units."""
        return _cast(float, self._read("width"))

    @width.setter
    def width(self, value: float) -> None:
        self.update(width=value)

    @property
    def slope(self) -> float:
        """Return the dimensionless average surface slope."""
        return _cast(float, self._read("slope"))

    @slope.setter
    def slope(self, value: float) -> None:
        self.update(slope=value)

    @property
    def curb_length(self) -> float:
        """Return curb length in project length units."""
        return _cast(float, self._read("curb_length"))

    @curb_length.setter
    def curb_length(self, value: float) -> None:
        self.update(curb_length=value)

    @property
    def impervious_fraction(self) -> float:
        """Return the impervious area fraction."""
        return _cast(float, self._read("impervious_fraction"))

    @impervious_fraction.setter
    def impervious_fraction(self, value: float) -> None:
        self.update(impervious_fraction=value)

    @property
    def zero_impervious_fraction(self) -> float:
        """Return the zero-depression-storage impervious fraction."""
        return _cast(float, self._read("zero_impervious_fraction"))

    @zero_impervious_fraction.setter
    def zero_impervious_fraction(self, value: float) -> None:
        self.update(zero_impervious_fraction=value)

    @property
    def impervious_roughness(self) -> float:
        """Return impervious-area Manning roughness."""
        return _cast(float, self._read("impervious_roughness"))

    @impervious_roughness.setter
    def impervious_roughness(self, value: float) -> None:
        self.update(impervious_roughness=value)

    @property
    def pervious_roughness(self) -> float:
        """Return pervious-area Manning roughness."""
        return _cast(float, self._read("pervious_roughness"))

    @pervious_roughness.setter
    def pervious_roughness(self, value: float) -> None:
        self.update(pervious_roughness=value)

    @property
    def impervious_depression_storage(self) -> float:
        """Return impervious depression storage in project rain-depth units."""
        return _cast(float, self._read("impervious_depression_storage"))

    @impervious_depression_storage.setter
    def impervious_depression_storage(self, value: float) -> None:
        self.update(impervious_depression_storage=value)

    @property
    def pervious_depression_storage(self) -> float:
        """Return pervious depression storage in project rain-depth units."""
        return _cast(float, self._read("pervious_depression_storage"))

    @pervious_depression_storage.setter
    def pervious_depression_storage(self, value: float) -> None:
        self.update(pervious_depression_storage=value)

    @property
    def outlet(self) -> Node | Subcatchment:
        """Return the node or subcatchment outlet relationship."""
        return _cast("Node | Subcatchment", self._read("outlet"))

    @outlet.setter
    def outlet(self, value: Node | Subcatchment) -> None:
        self.update(outlet=value)

    @property
    def outlet_kind(self) -> _OutKind:
        """Return the typed outlet relationship kind."""
        return _OutKind(_cast(str, self._read("outlet_kind")))

    @property
    def infiltration(self) -> SubcatchmentInfiltration | None:
        """Return the current infiltration configuration view, if declared."""
        owner = self._owner
        kind = owner._simulation._owner.subcatchment_infiltration_value(
            owner._generation, owner._index, owner._id, owner._subtype, None, "kind"
        )
        return (
            None
            if kind is None
            else SubcatchmentInfiltration(owner, _InfilKind(_cast(str, kind)))
        )

    @infiltration.setter
    def infiltration(self, value: _SubcatchmentInfiltrationSettings) -> None:
        self.update(infiltration=value)

    @property
    def groundwater(self) -> SubcatchmentGroundwater | None:
        """Return the optional groundwater configuration slot view."""
        owner = self._owner
        present = owner._simulation._owner.subcatchment_groundwater_value(
            owner._simulation, owner._generation, owner._index, owner._id, owner._subtype, "present"
        )
        return SubcatchmentGroundwater(owner) if present else None

    @groundwater.setter
    def groundwater(self, value: SubcatchmentGroundwaterSettings | None) -> None:
        self.update(groundwater=value)

    @property
    def snowpack(self) -> SnowmeltParameterSet | None:
        """Return the optional snowmelt parameter-set relationship."""
        owner = self._owner
        return _cast(
            "SnowmeltParameterSet | None",
            owner._simulation._owner.subcatchment_snowpack(
                owner._simulation,
                owner._generation,
                owner._index,
                owner._id,
                owner._subtype,
            ),
        )

    @snowpack.setter
    def snowpack(self, value: SnowmeltParameterSet | None) -> None:
        self.update(snowpack=value)

    @property
    def initial_buildup(self) -> _MutableMapping[str, float]:
        """Return live buildup; deletion zeros one value and assignment replaces all."""
        return _LiveSubcatchmentNamedMapping(self._owner, "loading")

    @initial_buildup.setter
    def initial_buildup(self, value: _Mapping[str, float]) -> None:
        _LiveSubcatchmentNamedMapping(self._owner, "loading")._replace(value)

    @property
    def coverage_fractions(self) -> _MutableMapping[str, float]:
        """Return live coverage; deletion zeros one value and assignment replaces all."""
        return _LiveSubcatchmentNamedMapping(self._owner, "coverage")

    @coverage_fractions.setter
    def coverage_fractions(self, value: _Mapping[str, float]) -> None:
        _LiveSubcatchmentNamedMapping(self._owner, "coverage")._replace(value)


@_final
class SubcatchmentInfiltration:
    """Expose the current infiltration configuration slot as a live view."""

    __slots__ = ("_expected_kind", "_owner")

    def __init__(self, owner: Subcatchment, expected_kind: _InfilKind) -> None:
        self._owner = owner
        self._expected_kind = expected_kind

    def _read(self, field: str) -> object:
        owner = self._owner
        return owner._simulation._owner.subcatchment_infiltration_value(
            owner._generation,
            owner._index,
            owner._id,
            owner._subtype,
            self._expected_kind.value,
            field,
        )

    def update(self, **changes: object) -> None:
        """Atomically update supplied infiltration fields through the Simulation Owner."""
        owner = self._owner
        owner._simulation._owner.update_subcatchment_infiltration(
            owner._generation,
            owner._index,
            owner._id,
            owner._subtype,
            self._expected_kind.value,
            changes,
        )

    @property
    def kind(self) -> _InfilKind:
        """Return the current infiltration model kind."""
        self._read("kind")
        return self._expected_kind

    @property
    def initial_rate(self) -> float:
        """Return the initial Horton-family rate in project rainfall units."""
        return _cast(float, self._read("initial_rate"))

    @initial_rate.setter
    def initial_rate(self, value: float) -> None:
        self.update(initial_rate=value)

    @property
    def minimum_rate(self) -> float:
        """Return the minimum Horton-family rate in project rainfall units."""
        return _cast(float, self._read("minimum_rate"))

    @minimum_rate.setter
    def minimum_rate(self, value: float) -> None:
        self.update(minimum_rate=value)

    @property
    def decay_coefficient(self) -> float:
        """Return the Horton-family decay coefficient per hour."""
        return _cast(float, self._read("decay_coefficient"))

    @decay_coefficient.setter
    def decay_coefficient(self, value: float) -> None:
        self.update(decay_coefficient=value)

    @property
    def drying_time_days(self) -> float:
        """Return the infiltration recovery drying time in days."""
        return _cast(float, self._read("drying_time_days"))

    @drying_time_days.setter
    def drying_time_days(self, value: float) -> None:
        self.update(drying_time_days=value)

    @property
    def maximum_infiltration(self) -> float | None:
        """Return the optional maximum Horton-family infiltration depth."""
        return _cast(float | None, self._read("maximum_infiltration"))

    @maximum_infiltration.setter
    def maximum_infiltration(self, value: float | None) -> None:
        self.update(maximum_infiltration=value)

    @property
    def suction_head(self) -> float:
        """Return Green-Ampt-family suction head in project rain-depth units."""
        return _cast(float, self._read("suction_head"))

    @suction_head.setter
    def suction_head(self, value: float) -> None:
        self.update(suction_head=value)

    @property
    def hydraulic_conductivity(self) -> float:
        """Return Green-Ampt-family conductivity in project rainfall units."""
        return _cast(float, self._read("hydraulic_conductivity"))

    @hydraulic_conductivity.setter
    def hydraulic_conductivity(self, value: float) -> None:
        self.update(hydraulic_conductivity=value)

    @property
    def initial_moisture_deficit(self) -> float:
        """Return the Green-Ampt-family initial moisture deficit fraction."""
        return _cast(float, self._read("initial_moisture_deficit"))

    @initial_moisture_deficit.setter
    def initial_moisture_deficit(self, value: float) -> None:
        self.update(initial_moisture_deficit=value)

    @property
    def curve_number(self) -> float:
        """Return the Curve Number infiltration parameter."""
        return _cast(float, self._read("curve_number"))

    @curve_number.setter
    def curve_number(self, value: float) -> None:
        self.update(curve_number=value)


@_final
class SubcatchmentGroundwater:
    """Expose one optional groundwater configuration slot as a live view."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Subcatchment) -> None:
        self._owner = owner

    def _read(self, field: str) -> object:
        owner = self._owner
        return owner._simulation._owner.subcatchment_groundwater_value(
            owner._simulation, owner._generation, owner._index, owner._id, owner._subtype, field
        )

    def update(self, **changes: object) -> None:
        """Atomically retain supplied groundwater fields through the Simulation Owner.

        Identity, scalar range, and unit checks are immediate. Elevation ordering
        and other relational checks are deferred to :meth:`Simulation.start`.
        """
        owner = self._owner
        owner._simulation._owner.update_subcatchment_groundwater(
            owner._generation, owner._index, owner._id, owner._subtype, changes
        )

    @property
    def aquifer(self) -> Aquifer:
        """Return the groundwater aquifer relationship."""
        return _cast("Aquifer", self._read("aquifer"))

    @aquifer.setter
    def aquifer(self, value: Aquifer) -> None:
        self.update(aquifer=value)

    @property
    def node(self) -> Node:
        """Return the groundwater outlet-node relationship."""
        return _cast("Node", self._read("node"))

    @node.setter
    def node(self, value: Node) -> None:
        self.update(node=value)

    @property
    def surface_elevation(self) -> float:
        """Return ground surface elevation in project length units."""
        return _cast(float, self._read("surface_elevation"))

    @surface_elevation.setter
    def surface_elevation(self, value: float) -> None:
        self.update(surface_elevation=value)

    @property
    def groundwater_coefficient(self) -> float:
        """Return the groundwater flow coefficient."""
        return _cast(float, self._read("groundwater_coefficient"))

    @groundwater_coefficient.setter
    def groundwater_coefficient(self, value: float) -> None:
        self.update(groundwater_coefficient=value)

    @property
    def groundwater_exponent(self) -> float:
        """Return the groundwater flow exponent."""
        return _cast(float, self._read("groundwater_exponent"))

    @groundwater_exponent.setter
    def groundwater_exponent(self, value: float) -> None:
        self.update(groundwater_exponent=value)

    @property
    def surface_coefficient(self) -> float:
        """Return the surface-water flow coefficient."""
        return _cast(float, self._read("surface_coefficient"))

    @surface_coefficient.setter
    def surface_coefficient(self, value: float) -> None:
        self.update(surface_coefficient=value)

    @property
    def surface_exponent(self) -> float:
        """Return the surface-water flow exponent."""
        return _cast(float, self._read("surface_exponent"))

    @surface_exponent.setter
    def surface_exponent(self, value: float) -> None:
        self.update(surface_exponent=value)

    @property
    def interaction_coefficient(self) -> float:
        """Return the groundwater/surface-water interaction coefficient."""
        return _cast(float, self._read("interaction_coefficient"))

    @interaction_coefficient.setter
    def interaction_coefficient(self, value: float) -> None:
        self.update(interaction_coefficient=value)

    @property
    def fixed_surface_depth(self) -> float:
        """Return fixed surface-water depth in project length units."""
        return _cast(float, self._read("fixed_surface_depth"))

    @fixed_surface_depth.setter
    def fixed_surface_depth(self, value: float) -> None:
        self.update(fixed_surface_depth=value)

    @property
    def bottom_elevation(self) -> float:
        """Return aquifer bottom elevation in project length units."""
        return _cast(float, self._read("bottom_elevation"))

    @bottom_elevation.setter
    def bottom_elevation(self, value: float) -> None:
        self.update(bottom_elevation=value)

    @property
    def water_table_elevation(self) -> float:
        """Return initial water-table elevation in project length units."""
        return _cast(float, self._read("water_table_elevation"))

    @water_table_elevation.setter
    def water_table_elevation(self, value: float) -> None:
        self.update(water_table_elevation=value)

    @property
    def upper_moisture(self) -> float:
        """Return the initial upper-zone moisture fraction."""
        return _cast(float, self._read("upper_moisture"))

    @upper_moisture.setter
    def upper_moisture(self, value: float) -> None:
        self.update(upper_moisture=value)


@_final
class RainGage(_LiveView):
    """Expose precipitation state and persistent forcing for one configured rain gage.

    Notes
    -----
    Values use the project's rainfall units. `total_precip`, `rainfall`, and
    `snowfall` report current effective precipitation. `external_precipitation_rate`
    selects the API as the persistent source; `rainfall_override` persistently masks
    every source until cleared.
    """

    __slots__ = ()

    def _precipitation_value(self, field: str) -> float:
        return self._simulation._owner.rain_gage_precipitation_value(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def total_precip(self) -> float:
        """Return current effective rain plus snow in project rainfall units."""
        return self._precipitation_value("total_precip")

    @property
    def external_precipitation_rate(self) -> float | None:
        """Return the selected external source rate, or `None` before selection.

        `use_external_precipitation()` selects the API as this gage's source,
        unlinks any co-gage, and persists until replaced or the simulation closes.
        A rate of `0.0` keeps the API source selected with dry precipitation.
        A rainfall override takes precedence while present.
        """
        return self._simulation._owner.rain_gage_external_precipitation_rate(
            self._generation, self._index, self._id, self._subtype
        )

    def use_external_precipitation(self, rate: float) -> None:
        """Select persistent API precipitation. Lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        self._simulation._owner.use_rain_gage_external_precipitation(
            self._generation, self._index, self._id, self._subtype, rate
        )

    @property
    def rainfall(self) -> float:
        """Return current effective liquid rainfall in project rainfall units."""
        return self._precipitation_value("rainfall")

    @property
    def rainfall_override(self) -> float | None:
        """Rainfall override or `None`. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`.

        Assigning a nonnegative finite rate masks every configured source without
        changing it; `0.0` forces dry precipitation. Assigning `None` clears the
        override and resumes the underlying source. Setting an override unlinks any
        co-gage, and clearing it does not restore that relationship.
        """
        return self._simulation._owner.rain_gage_rainfall_override(
            self._generation, self._index, self._id, self._subtype
        )

    @rainfall_override.setter
    def rainfall_override(self, value: float | None) -> None:
        self._simulation._owner.set_rain_gage_rainfall_override(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def snowfall(self) -> float:
        """Return current effective snowfall in project rainfall units."""
        return self._precipitation_value("snowfall")


class _LiveSubcatchmentNamedMapping(_MutableMapping[str, float]):
    """Address current loading or coverage by configured string ID."""

    __slots__ = ("_owner", "_path")

    def __init__(self, owner: Subcatchment, path: str) -> None:
        self._owner = owner
        self._path = path

    def _snapshot(self) -> _Mapping[str, float]:
        owner = self._owner
        values = owner._simulation._owner.subcatchment_named_settings_mapping(
            owner._generation, owner._index, owner._id, owner._subtype, self._path
        )
        return dict(values)

    def _mutate(self, replace: bool, values: object) -> None:
        owner = self._owner
        owner._simulation._owner.update_subcatchment_named_settings(
            owner._generation, owner._index, owner._id, owner._subtype, self._path, replace, values
        )

    def _replace(self, value: object) -> None:
        self._mutate(True, value)

    def __getitem__(self, key: str) -> float:
        owner = self._owner
        return owner._simulation._owner.subcatchment_named_setting_value(
            owner._generation, owner._index, owner._id, owner._subtype, self._path, key
        )

    def __setitem__(self, key: str, value: float) -> None:
        self._mutate(False, {key: value})

    def __delitem__(self, key: str) -> None:
        self._mutate(False, {key: 0.0})

    def __iter__(self) -> _Iterator[str]:
        return iter(self._snapshot())

    def __len__(self) -> int:
        return len(self._snapshot())

    def update(self, *args: object, **kwargs: float) -> None:
        self._mutate(False, dict(*args, **kwargs))  # type: ignore[arg-type]

    def clear(self) -> None:
        self._mutate(True, {})

    def __repr__(self) -> str:
        return repr(self._snapshot())


@_final
class _SubcatchmentExternalPollutantBuildup(_MutableMapping[str, float]):
    """Expose persistent subcatchment buildup forcing as a canonical live mapping."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Subcatchment) -> None:
        self._owner = owner

    def _snapshot(self) -> tuple[list[str], list[float]]:
        owner = self._owner
        return owner._simulation._owner.subcatchment_external_pollutant_buildup_increment(
            owner._generation, owner._index, owner._id, owner._subtype
        )

    def _mutate(
        self,
        replace: bool,
        args: tuple[object, ...],
        kwargs: dict[str, float],
    ) -> None:
        owner = self._owner
        owner._simulation._owner.update_subcatchment_external_pollutant_buildup_increment(
            owner._generation, owner._index, owner._id, owner._subtype, replace, args, kwargs
        )

    def __getitem__(self, key: str) -> float:
        owner = self._owner
        return owner._simulation._owner.subcatchment_external_pollutant_buildup_increment_value(
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


@_final
class Subcatchment(_LiveView):
    """Expose bindable settings, forcing, results, quality, and LID state.

    Notes
    -----
    Stable declarations are available only through `settings`. Result properties
    reflect the current routing state, while snapshot and statistics methods
    return owned, immutable records.
    """

    __slots__ = ()

    def _precipitation_scale_factor(self, field: str) -> float:
        return self._simulation._owner.subcatchment_precipitation_scale_factor(
            self._generation, self._index, self._id, self._subtype, field
        )

    def _result(self, field: str) -> float:
        return self._simulation._owner.subcatchment_result(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def settings(self) -> SubcatchmentSettings:
        """Return a live view of stable subcatchment settings in project units."""
        return SubcatchmentSettings(self)

    @property
    def rain_scale_factor(self) -> float:
        """Return the persistent gage-rainfall multiplier."""
        return self._precipitation_scale_factor("rain_scale_factor")

    @property
    def snow_scale_factor(self) -> float:
        """Return the persistent gage-snowfall multiplier."""
        return self._precipitation_scale_factor("snow_scale_factor")

    def set_precipitation_scale_factors(
        self,
        *,
        rainfall: float,
        snowfall: float,
    ) -> None:
        """Set both persistent gage multipliers. Lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        self._simulation._owner.set_subcatchment_precipitation_scale_factors(
            self._generation, self._index, self._id, self._subtype, rainfall, snowfall
        )

    @property
    def external_rainfall(self) -> float:
        """External rainfall. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return self._simulation._owner.subcatchment_external_forcing_value(
            self._generation, self._index, self._id, self._subtype, "rainfall"
        )

    @external_rainfall.setter
    def external_rainfall(self, value: float) -> None:
        self._simulation._owner.set_subcatchment_external_rainfall(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def external_snowfall(self) -> float:
        """External snowfall. Setter lifecycle: `OPEN`, `RUNNING`, `ENDED`."""
        return self._simulation._owner.subcatchment_external_forcing_value(
            self._generation, self._index, self._id, self._subtype, "snowfall"
        )

    @external_snowfall.setter
    def external_snowfall(self, value: float) -> None:
        self._simulation._owner.set_subcatchment_external_snowfall(
            self._generation, self._index, self._id, self._subtype, value
        )

    @property
    def rainfall(self) -> float:
        """Return current total rainfall and snowfall in project rainfall units."""
        return self._result("rainfall")

    @property
    def evaporation(self) -> float:
        """Return current evaporation in project evaporation-rate units."""
        return self._result("evaporation")

    @property
    def infiltration(self) -> float:
        """Return current infiltration in project rainfall units."""
        return self._result("infiltration")

    @property
    def runon(self) -> float:
        """Return current runon in project flow units."""
        return self._result("runon")

    @property
    def runoff(self) -> float:
        """Return current runoff in project flow units."""
        return self._result("runoff")

    @property
    def snow_depth(self) -> float:
        """Return current snow depth in project rain-depth units."""
        return self._result("snow_depth")

    def _quality(self, field: str) -> tuple[list[str], list[float]]:
        return self._simulation._owner.subcatchment_quality_mapping(
            self._generation, self._index, self._id, self._subtype, field
        )

    @property
    def runoff_pollutant_concentration(self) -> _Mapping[str, float]:
        """Return current runoff concentrations by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("runoff_concentrations"))

    @property
    def ponded_pollutant_concentration(self) -> _Mapping[str, float]:
        """Return current ponded concentrations by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("ponded_concentrations"))

    @property
    def pollutant_buildup(self) -> _Mapping[str, float]:
        """Return current area-weighted buildup by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("buildup_loads"))

    @property
    def pollutant_total_load(self) -> _Mapping[str, float]:
        """Return current total washoff load by canonical pollutant ID."""
        return _pollutant_mapping(self._quality("total_washoff_loads"))

    @property
    def external_pollutant_buildup_increment(
        self,
    ) -> _MutableMapping[str, float]:
        """Return canonical persistent buildup increments as a live mutable mapping.

        Item assignment and `update()` are atomic sparse mutations. Deletion resets
        one configured pollutant to zero; `clear()` atomically resets every pollutant.
        Mutation lifecycle: `OPEN`, `RUNNING`, `ENDED`.
        """
        return _SubcatchmentExternalPollutantBuildup(self)

    @property
    def statistics(self) -> _snapshots.SubcatchmentStatistics:
        """Return immutable cumulative statistics for this subcatchment."""
        return self._simulation._owner.subcatchment_statistics(
            self._generation, self._index, self._id, self._subtype
        )

    @property
    def lid_units(self) -> LidUnitCollection:
        """Return locally indexed LID Units owned by this subcatchment."""
        return LidUnitCollection(self, _COLLECTION_TOKEN)

    def lid_snapshot(self) -> _snapshots.SubcatchmentLidSnapshot:
        """Copy current LID-group runtime values in configured project units."""
        return self._simulation._owner.subcatchment_lid_snapshot(
            self._generation, self._index, self._id, self._subtype
        )


@_final
class LidUnit:
    """Expose one LID unit owned by a subcatchment."""

    __slots__ = ("_index", "_owner")

    def __init__(self, owner: Subcatchment, index: int, token: object) -> None:
        if token is not _VIEW_TOKEN:
            raise TypeError("LID Unit views are created by Subcatchment.lid_units")
        self._owner = owner
        self._index = index

    def _value(self, field: str) -> object:
        return self._owner._simulation._owner.lid_unit_value(
            self._owner._simulation,
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
            self._index,
            field,
        )

    def snapshot(self) -> _snapshots.LidUnitSnapshot:
        """Copy current LID Unit runtime values in configured project units."""
        return self._owner._simulation._owner.lid_unit_snapshot(
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
            self._index,
        )

    def update(self, **changes: object) -> None:
        """Commit requested LID-unit values atomically in `OPEN` or `ENDED`.

        Intrinsic errors fail immediately. Group area, capture, routing, drain,
        infiltration, and subcatchment relationships are prepared at `start()`.
        """
        self._owner._simulation._owner.update_lid_unit(
            self._owner._generation,
            self._owner._index,
            self._owner._id,
            self._owner._subtype,
            self._index,
            changes,
        )

    @property
    def index(self) -> int:
        """Return the generation-bound position within the owning subcatchment."""
        self._value("count")
        return self._index

    @property
    def area(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("area"))

    @area.setter
    def area(self, value: float) -> None:
        self.update(area=value)

    @property
    def full_width(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("full_width"))

    @full_width.setter
    def full_width(self, value: float) -> None:
        self.update(full_width=value)

    @property
    def bottom_width(self) -> float:
        """Return the LID unit's bottom width in project length units."""
        return _cast(float, self._value("bottom_width"))

    @property
    def initial_saturation(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("initial_saturation"))

    @initial_saturation.setter
    def initial_saturation(self, value: float) -> None:
        self.update(initial_saturation=value)

    @property
    def impervious_runoff_treated(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("impervious_runoff_treated"))

    @impervious_runoff_treated.setter
    def impervious_runoff_treated(self, value: float) -> None:
        self.update(impervious_runoff_treated=value)

    @property
    def pervious_runoff_treated(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("pervious_runoff_treated"))

    @pervious_runoff_treated.setter
    def pervious_runoff_treated(self, value: float) -> None:
        self.update(pervious_runoff_treated=value)

    @property
    def control(self) -> LidControl:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast("LidControl", self._value("control_identity"))

    @control.setter
    def control(self, value: LidControl) -> None:
        self.update(control=value)

    @property
    def count(self) -> int:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(int, self._value("count"))

    @count.setter
    def count(self, value: int) -> None:
        self.update(count=value)

    @property
    def routes_to_pervious(self) -> bool:
        """Return requested routing intent; setter lifecycle: `OPEN`, `ENDED`.

        Preparation can force the native effective value off while a
        subcatchment has no pervious area without overwriting this request.
        """
        return _cast(bool, self._value("routes_to_pervious"))

    @routes_to_pervious.setter
    def routes_to_pervious(self, value: bool) -> None:
        self.update(routes_to_pervious=value)

    @property
    def drain_destination(self) -> Node | Subcatchment | None:
        """Return explicit drain intent; setter lifecycle: `OPEN`, `ENDED`.

        `None` requests an implicit drain that follows the prospective
        subcatchment outlet during Configuration Preparation.
        """
        return _cast("Node | Subcatchment | None", self._value("drain_destination"))

    @drain_destination.setter
    def drain_destination(self, value: Node | Subcatchment | None) -> None:
        self.update(drain_destination=value)

    @property
    def _identity(self) -> tuple[int, int, int, int]:
        return (
            id(self._owner._simulation),
            self._owner._generation,
            self._owner._index,
            self._index,
        )

    def __repr__(self) -> str:
        return (
            f"{type(self).__name__}(subcatchment_id={self._owner._id!r}, "
            f"subcatchment_index={self._owner._index}, index={self._index})"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, LidUnit):
            return NotImplemented
        return self._identity == other._identity

    def __hash__(self) -> int:
        return hash(self._identity)

    def __copy__(self) -> _Never:
        raise TypeError("LidUnit cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError("LidUnit cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError("LidUnit cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError("LidUnit cannot be pickled")


@_final
class LidUnitCollection(_Sequence[LidUnit]):
    """Provide indexed access to a subcatchment's LID units."""

    __slots__ = ("_owner",)

    def __init__(self, owner: Subcatchment, token: object) -> None:
        if token is not _COLLECTION_TOKEN:
            raise TypeError("LID Unit collections are created by Subcatchment.lid_units")
        self._owner = owner

    def __len__(self) -> int:
        return self._owner._simulation._owner.lid_unit_count(
            self._owner._generation, self._owner._index, self._owner._id, self._owner._subtype
        )

    @_overload
    def __getitem__(self, index: int) -> LidUnit: ...

    @_overload
    def __getitem__(self, index: slice) -> tuple[LidUnit, ...]: ...

    def __getitem__(self, index: int | slice) -> LidUnit | tuple[LidUnit, ...]:
        count = len(self)
        if isinstance(index, slice):
            return tuple(
                LidUnit(self._owner, position, _VIEW_TOKEN)
                for position in range(*index.indices(count))
            )
        if type(index) is not int:
            raise TypeError("LID Unit positions must be int or slice")
        if index < 0:
            index += count
        if not 0 <= index < count:
            raise IndexError("LID Unit index out of range")
        return LidUnit(self._owner, index, _VIEW_TOKEN)

    def __repr__(self) -> str:
        return (
            f"{type(self).__name__}(subcatchment_id={self._owner._id!r}, "
            f"subcatchment_index={self._owner._index})"
        )

    def __copy__(self) -> _Never:
        raise TypeError("LidUnitCollection cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError("LidUnitCollection cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError("LidUnitCollection cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError("LidUnitCollection cannot be pickled")


@_final
class SubcatchmentCollection(_Collection[Subcatchment]):
    """Provide subcatchment lookup and aligned hydraulic, quality, and statistics snapshots."""

    __slots__ = ()

    def __getitem__(self, key: str | int) -> Subcatchment:
        """Return a subcatchment view by case-insensitive configured ID."""
        return super().__getitem__(key)

    def by_index(self, index: int) -> Subcatchment:
        """Return a subcatchment view by its zero-based configured position."""
        return super().by_index(index)

    def snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.SubcatchmentSnapshot:
        """Copy current subcatchment hydraulics in canonical or requested order."""
        return self._simulation._owner.subcatchment_snapshot(self._generation, ids)

    def quality_snapshot(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.SubcatchmentQualitySnapshot:
        """Copy current subcatchment quality in canonical or requested order."""
        return self._simulation._owner.subcatchment_quality_snapshot(self._generation, ids)

    def statistics(
        self, ids: str | int | _Iterable[str | int] | None = None
    ) -> _snapshots.SubcatchmentStatisticsSnapshot:
        """Copy cumulative subcatchment statistics in canonical or requested order."""
        return self._simulation._owner.subcatchment_statistics_snapshot(self._generation, ids)
