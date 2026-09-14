"""Typed editable definition-object views."""

from __future__ import annotations as _annotations

from collections.abc import Iterable as _Iterable
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    NamedTuple as _NamedTuple,
    cast as _cast,
    final as _final,
)

from .._prettier import pretty_dataclass as _pretty_dataclass
from ._base import _LiveView
from ._subcatchments import RainGage, Subcatchment

if _TYPE_CHECKING:
    from ._nodes import Node


@_final
class SnowmeltSurface:
    """Expose one fixed snowmelt surface as a live project-unit view."""

    __slots__ = ("_owner", "_surface")

    def __init__(self, owner: SnowmeltParameterSet, surface: str) -> None:
        self._owner = owner
        self._surface = surface

    def _read(self, field: str) -> float:
        return self._owner._surface_read(self._surface, field)

    def update(self, **changes: object) -> None:
        """Atomically update supplied surface fields in project units."""
        self._owner._update(self._surface, changes)

    @property
    def minimum_melt_coefficient(self) -> float:
        """Return the minimum melt coefficient in project snowmelt units."""
        return self._read("minimum_melt_coefficient")

    @minimum_melt_coefficient.setter
    def minimum_melt_coefficient(self, value: float) -> None:
        self.update(minimum_melt_coefficient=value)

    @property
    def maximum_melt_coefficient(self) -> float:
        """Return the maximum melt coefficient in project snowmelt units."""
        return self._read("maximum_melt_coefficient")

    @maximum_melt_coefficient.setter
    def maximum_melt_coefficient(self, value: float) -> None:
        self.update(maximum_melt_coefficient=value)

    @property
    def base_temperature(self) -> float:
        """Return the base temperature in project temperature units."""
        return self._read("base_temperature")

    @base_temperature.setter
    def base_temperature(self, value: float) -> None:
        self.update(base_temperature=value)

    @property
    def free_water_fraction(self) -> float:
        """Return the snowpack free-water fraction."""
        return self._read("free_water_fraction")

    @free_water_fraction.setter
    def free_water_fraction(self, value: float) -> None:
        self.update(free_water_fraction=value)

    @property
    def initial_snow_depth(self) -> float:
        """Return the initial snow depth in project rain-depth units."""
        return self._read("initial_snow_depth")

    @initial_snow_depth.setter
    def initial_snow_depth(self, value: float) -> None:
        self.update(initial_snow_depth=value)

    @property
    def initial_free_water(self) -> float:
        """Return the initial free-water depth in project rain-depth units."""
        return self._read("initial_free_water")

    @initial_free_water.setter
    def initial_free_water(self, value: float) -> None:
        self.update(initial_free_water=value)

    @property
    def snow_depth_for_full_coverage(self) -> float:
        """Return the snow depth for complete areal coverage."""
        return self._read("snow_depth_for_full_coverage")

    @snow_depth_for_full_coverage.setter
    def snow_depth_for_full_coverage(self, value: float) -> None:
        self.update(snow_depth_for_full_coverage=value)


@_final
class Pollutant(_LiveView):
    """Expose a generation-bound pollutant definition."""

    __slots__ = ()


@_final
class LandUse(_LiveView):
    """Expose a generation-bound land-use definition."""

    __slots__ = ()


@_final
class TimePattern(_LiveView):
    """Expose a generation-bound time-pattern definition."""

    __slots__ = ()


@_final
class Curve(_LiveView):
    """Expose a generation-bound curve definition."""

    __slots__ = ()


@_final
class TimeSeries(_LiveView):
    """Expose a generation-bound time-series definition."""

    __slots__ = ()


@_final
class ControlRule(_LiveView):
    """Expose a generation-bound control-rule definition."""

    __slots__ = ()


@_final
class Transect(_LiveView):
    """Expose a generation-bound transect definition."""

    __slots__ = ()


@_final
class Aquifer(_LiveView):
    """Expose a generation-bound editable aquifer definition."""

    __slots__ = ()

    def _read(self, field: str) -> object:
        """Read one current aquifer field through the native owner."""
        return self._simulation._owner.aquifer_configuration_value(
            self._simulation, self._generation, self._index, self._id, self._subtype, field
        )

    def update(self, **changes: object) -> None:
        """Atomically update supplied aquifer fields in project units."""
        self._simulation._owner.update_aquifer(
            self._generation, self._index, self._id, self._subtype, changes
        )

    @property
    def porosity(self) -> float:
        """Return aquifer porosity as a fraction."""
        return _cast(float, self._read("porosity"))

    @porosity.setter
    def porosity(self, value: float) -> None:
        """Set aquifer porosity as a fraction."""
        self.update(porosity=value)

    @property
    def wilting_point(self) -> float:
        """Return wilting-point moisture as a fraction."""
        return _cast(float, self._read("wilting_point"))

    @wilting_point.setter
    def wilting_point(self, value: float) -> None:
        """Set wilting-point moisture as a fraction."""
        self.update(wilting_point=value)

    @property
    def field_capacity(self) -> float:
        """Return field-capacity moisture as a fraction."""
        return _cast(float, self._read("field_capacity"))

    @field_capacity.setter
    def field_capacity(self, value: float) -> None:
        """Set field-capacity moisture as a fraction."""
        self.update(field_capacity=value)

    @property
    def hydraulic_conductivity(self) -> float:
        """Return saturated hydraulic conductivity in project rainfall units."""
        return _cast(float, self._read("hydraulic_conductivity"))

    @hydraulic_conductivity.setter
    def hydraulic_conductivity(self, value: float) -> None:
        """Set saturated hydraulic conductivity in project rainfall units."""
        self.update(hydraulic_conductivity=value)

    @property
    def conductivity_slope(self) -> float:
        """Return hydraulic conductivity slope."""
        return _cast(float, self._read("conductivity_slope"))

    @conductivity_slope.setter
    def conductivity_slope(self, value: float) -> None:
        """Set hydraulic conductivity slope."""
        self.update(conductivity_slope=value)

    @property
    def tension_slope(self) -> float:
        """Return soil moisture tension slope in project length units."""
        return _cast(float, self._read("tension_slope"))

    @tension_slope.setter
    def tension_slope(self, value: float) -> None:
        """Set soil moisture tension slope in project length units."""
        self.update(tension_slope=value)

    @property
    def upper_evaporation_fraction(self) -> float:
        """Return the upper-zone evaporation fraction."""
        return _cast(float, self._read("upper_evaporation_fraction"))

    @upper_evaporation_fraction.setter
    def upper_evaporation_fraction(self, value: float) -> None:
        """Set the upper-zone evaporation fraction."""
        self.update(upper_evaporation_fraction=value)

    @property
    def lower_evaporation_depth(self) -> float:
        """Return lower-zone evaporation depth in project length units."""
        return _cast(float, self._read("lower_evaporation_depth"))

    @lower_evaporation_depth.setter
    def lower_evaporation_depth(self, value: float) -> None:
        """Set lower-zone evaporation depth in project length units."""
        self.update(lower_evaporation_depth=value)

    @property
    def lower_loss_coefficient(self) -> float:
        """Return lower-zone loss coefficient in project rainfall units."""
        return _cast(float, self._read("lower_loss_coefficient"))

    @lower_loss_coefficient.setter
    def lower_loss_coefficient(self, value: float) -> None:
        """Set lower-zone loss coefficient in project rainfall units."""
        self.update(lower_loss_coefficient=value)

    @property
    def bottom_elevation(self) -> float:
        """Return aquifer bottom elevation in project length units."""
        return _cast(float, self._read("bottom_elevation"))

    @bottom_elevation.setter
    def bottom_elevation(self, value: float) -> None:
        """Set aquifer bottom elevation in project length units."""
        self.update(bottom_elevation=value)

    @property
    def water_table_elevation(self) -> float:
        """Return initial water-table elevation in project length units."""
        return _cast(float, self._read("water_table_elevation"))

    @water_table_elevation.setter
    def water_table_elevation(self, value: float) -> None:
        """Set initial water-table elevation in project length units."""
        self.update(water_table_elevation=value)

    @property
    def upper_moisture(self) -> float:
        """Return initial upper-zone moisture as a fraction."""
        return _cast(float, self._read("upper_moisture"))

    @upper_moisture.setter
    def upper_moisture(self, value: float) -> None:
        """Set initial upper-zone moisture as a fraction."""
        self.update(upper_moisture=value)

    @property
    def upper_evaporation_pattern(self) -> TimePattern | None:
        """Return the optional upper-zone evaporation pattern relationship."""
        return _cast(TimePattern | None, self._read("upper_evaporation_pattern"))

    @upper_evaporation_pattern.setter
    def upper_evaporation_pattern(self, value: TimePattern | None) -> None:
        """Set or clear the upper-zone evaporation pattern relationship."""
        self.update(upper_evaporation_pattern=value)


@_final
@_pretty_dataclass(slots=True)
class UnitHydrographResponse:
    """Store one detached short-, medium-, or long-response RTK parameter set."""

    rainfall_fraction: float
    time_to_peak_hours: float
    recession_ratio: float
    maximum_initial_abstraction: float
    initial_abstraction_recovery_rate: float
    initial_abstraction_at_start: float


@_final
class UnitHydrographMonth(_NamedTuple):
    """Store one month's detached short, medium, and long RTK responses."""

    short: UnitHydrographResponse
    medium: UnitHydrographResponse
    long: UnitHydrographResponse


@_final
class UnitHydrograph(_LiveView):
    """Expose a generation-bound editable RTK unit-hydrograph group."""

    __slots__ = ()

    def _configuration(
        self,
    ) -> tuple[
        RainGage,
        tuple[
            tuple[
                tuple[float, float, float, float, float, float],
                ...,
            ],
            ...,
        ],
    ]:
        return _cast(
            tuple[
                RainGage,
                tuple[
                    tuple[
                        tuple[float, float, float, float, float, float],
                        ...,
                    ],
                    ...,
                ],
            ],
            self._simulation._owner.unit_hydrograph_configuration(
                self._simulation,
                self._generation,
                self._index,
                self._id,
                self._subtype,
            ),
        )

    def update(self, **changes: object) -> None:
        """Atomically update supplied RTK fields in project units."""
        if "monthly_responses" in changes:
            monthly_responses = changes["monthly_responses"]
            if not isinstance(monthly_responses, _Iterable):
                raise TypeError("monthly_responses must be iterable")
            changes["monthly_responses"] = tuple(
                tuple(_response_to_native(response) for response in month)
                for month in monthly_responses
            )
        self._simulation._owner.update_unit_hydrograph(
            self._generation, self._index, self._id, self._subtype, changes
        )

    @property
    def rain_gage(self) -> RainGage:
        """Return the unit hydrograph's rain gage."""
        return self._configuration()[0]

    @rain_gage.setter
    def rain_gage(self, value: RainGage) -> None:
        self.update(rain_gage=value)

    @property
    def monthly_responses(self) -> tuple[UnitHydrographMonth, ...]:
        """Return January-through-December named response groups."""
        return tuple(
            UnitHydrographMonth(
                short=UnitHydrographResponse(*month[0]),
                medium=UnitHydrographResponse(*month[1]),
                long=UnitHydrographResponse(*month[2]),
            )
            for month in self._configuration()[1]
        )

    @monthly_responses.setter
    def monthly_responses(self, value: _Iterable[UnitHydrographMonth]) -> None:
        self.update(monthly_responses=value)


@_final
@_pretty_dataclass(slots=True)
class RdiiAssignment:
    """Store one detached RTK RDII node assignment in project units."""

    node: Node
    unit_hydrograph: UnitHydrograph
    area: float


def _response_to_native(response: object) -> tuple[float, float, float, float, float, float]:
    if not isinstance(response, UnitHydrographResponse):
        raise TypeError("monthly_responses must contain UnitHydrographResponse values")
    return (
        response.rainfall_fraction,
        response.time_to_peak_hours,
        response.recession_ratio,
        response.maximum_initial_abstraction,
        response.initial_abstraction_recovery_rate,
        response.initial_abstraction_at_start,
    )


@_final
class SnowmeltParameterSet(_LiveView):
    """Expose a generation-bound editable snowmelt parameter-set definition."""

    __slots__ = ()

    def _read(self, field: str) -> object:
        return self._simulation._owner.snowmelt_configuration_value(
            self._simulation, self._generation, self._index, self._id, self._subtype, field
        )

    def _surface_read(self, surface: str, field: str) -> float:
        return self._simulation._owner.snowmelt_surface_configuration_value(
            self._generation, self._index, self._id, self._subtype, surface, field
        )

    def _update(self, surface: str | None, changes: dict[str, object]) -> None:
        self._simulation._owner.update_snowmelt(
            self._generation, self._index, self._id, self._subtype, surface, changes
        )

    def update(self, **changes: object) -> None:
        """Atomically update supplied root snowmelt fields in project units."""
        self._update(None, changes)

    @property
    def plowable_fraction(self) -> float:
        """Return the fraction of impervious area that is plowable."""
        return _cast(float, self._read("plowable_fraction"))

    @plowable_fraction.setter
    def plowable_fraction(self, value: float) -> None:
        self.update(plowable_fraction=value)

    @property
    def plowable(self) -> SnowmeltSurface:
        """Return the fixed plowable-surface Live View."""
        return SnowmeltSurface(self, "plowable")

    @property
    def impervious(self) -> SnowmeltSurface:
        """Return the fixed impervious-surface Live View."""
        return SnowmeltSurface(self, "impervious")

    @property
    def pervious(self) -> SnowmeltSurface:
        """Return the fixed pervious-surface Live View."""
        return SnowmeltSurface(self, "pervious")

    @property
    def plow_depth(self) -> float:
        """Return the snow depth at which plowing begins in project units."""
        return _cast(float, self._read("plow_depth"))

    @plow_depth.setter
    def plow_depth(self, value: float) -> None:
        self.update(plow_depth=value)

    @property
    def removal_fractions(self) -> tuple[float, float, float, float, float]:
        """Return fractions routed to five plowed-snow destinations."""
        return _cast(tuple[float, float, float, float, float], self._read("removal_fractions"))

    @removal_fractions.setter
    def removal_fractions(self, value: tuple[float, float, float, float, float]) -> None:
        self.update(removal_fractions=value)

    @property
    def removal_subcatchment(self) -> Subcatchment | None:
        """Return the optional subcatchment receiving removed snow."""
        return _cast(Subcatchment | None, self._read("removal_subcatchment"))

    @removal_subcatchment.setter
    def removal_subcatchment(self, value: Subcatchment | None) -> None:
        self.update(removal_subcatchment=value)


@_final
class CustomShape(_LiveView):
    """Expose a generation-bound custom-shape definition."""

    __slots__ = ()


@_final
class Street(_LiveView):
    """Expose a generation-bound street definition."""

    __slots__ = ()


@_final
class InletDesign(_LiveView):
    """Expose a generation-bound inlet-design definition."""

    __slots__ = ()
