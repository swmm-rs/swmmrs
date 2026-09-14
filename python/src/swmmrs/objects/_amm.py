"""Typed editable AMM model views."""

from __future__ import annotations as _annotations

from collections.abc import Iterable as _Iterable
from typing import TYPE_CHECKING as _TYPE_CHECKING, cast as _cast, final as _final

from .._prettier import pretty_dataclass as _pretty_dataclass
from ._base import _LiveView
from ._subcatchments import RainGage

if _TYPE_CHECKING:
    from ._nodes import Node


@_final
@_pretty_dataclass(slots=True)
class AmmStandardComponent:
    """Store one detached standard AMM component in project units."""

    id: str
    dry_capture: float
    precipitation_window_hours: float
    hydrograph_half_life_hours: float
    initial_wet_capture: float
    moisture_half_life_hours: float
    temperature_window_hours: float
    spring_cold_shcf: float
    hot_shcf: float
    fall_cold_shcf: float


@_final
@_pretty_dataclass(slots=True)
class AmmBaseflowComponent:
    """Store one detached baseflow AMM component in project units."""

    id: str
    precipitation_window_hours: float
    hydrograph_half_life_hours: float
    temperature_window_hours: float
    spring_cold_capture: float
    hot_capture: float
    fall_cold_capture: float


AmmComponent = AmmStandardComponent | AmmBaseflowComponent


@_final
class AmmModel(_LiveView):
    """Expose a generation-bound editable AMM model."""

    __slots__ = ()

    def _configuration(
        self,
    ) -> tuple[RainGage, float, float, tuple[tuple[object, ...], ...]]:
        return _cast(
            tuple[RainGage, float, float, tuple[tuple[object, ...], ...]],
            self._simulation._owner.amm_model_configuration(
                self._simulation,
                self._generation,
                self._index,
                self._id,
                self._subtype,
            ),
        )

    def update(self, **changes: object) -> None:
        """Atomically update supplied AMM model fields in project units."""
        if "components" in changes:
            components = changes["components"]
            if not isinstance(components, _Iterable):
                raise TypeError("components must be an iterable of AMM components")
            changes["components"] = tuple(
                _component_to_native(component) for component in components
            )
        self._simulation._owner.update_amm_model(
            self._generation, self._index, self._id, self._subtype, changes
        )

    @property
    def rain_gage(self) -> RainGage:
        """Return the model's rain gage."""
        return self._configuration()[0]

    @rain_gage.setter
    def rain_gage(self, value: RainGage) -> None:
        self.update(rain_gage=value)

    @property
    def cold_temperature(self) -> float:
        """Return the cold reference temperature in project units."""
        return self._configuration()[1]

    @cold_temperature.setter
    def cold_temperature(self, value: float) -> None:
        self.update(cold_temperature=value)

    @property
    def hot_temperature(self) -> float:
        """Return the hot reference temperature in project units."""
        return self._configuration()[2]

    @hot_temperature.setter
    def hot_temperature(self, value: float) -> None:
        self.update(hot_temperature=value)

    @property
    def components(self) -> tuple[AmmComponent, ...]:
        """Return detached component definitions in configured order."""
        return tuple(_component_from_native(component) for component in self._configuration()[3])

    @components.setter
    def components(self, value: _Iterable[AmmComponent]) -> None:
        self.update(components=value)


@_final
@_pretty_dataclass(slots=True)
class AmmAssignment:
    """Store one detached AMM node assignment in project units."""

    node: Node
    model: AmmModel
    area: float


def _component_from_native(value: tuple[object, ...]) -> AmmComponent:
    kind = value[0]
    if kind == "standard":
        (
            _,
            identifier,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        ) = _cast(
            tuple[str, str, float, float, float, float, float, float, float, float, float],
            value,
        )
        return AmmStandardComponent(
            identifier,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        )
    if kind == "baseflow":
        (
            _,
            identifier,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        ) = _cast(tuple[str, str, float, float, float, float, float, float], value)
        return AmmBaseflowComponent(
            identifier,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        )
    raise RuntimeError(f"unknown native AMM component kind: {kind!r}")


def _component_to_native(component: object) -> tuple[object, ...]:
    if isinstance(component, AmmStandardComponent):
        return (
            "standard",
            component.id,
            component.dry_capture,
            component.precipitation_window_hours,
            component.hydrograph_half_life_hours,
            component.initial_wet_capture,
            component.moisture_half_life_hours,
            component.temperature_window_hours,
            component.spring_cold_shcf,
            component.hot_shcf,
            component.fall_cold_shcf,
        )
    if isinstance(component, AmmBaseflowComponent):
        return (
            "baseflow",
            component.id,
            component.precipitation_window_hours,
            component.hydrograph_half_life_hours,
            component.temperature_window_hours,
            component.spring_cold_capture,
            component.hot_capture,
            component.fall_cold_capture,
        )
    raise TypeError("components must contain AmmStandardComponent or AmmBaseflowComponent")
