"""LID control and layer views."""

from __future__ import annotations as _annotations

from datetime import timedelta as _timedelta
from typing import (
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    TypeVar as _TypeVar,
    cast as _cast,
    final as _final,
)

from ._base import _VIEW_TOKEN, _LiveView
from ._definitions import Curve

_LidLayerT = _TypeVar("_LidLayerT", bound="_LidLayer")


@_final
class LidControl(_LiveView):
    """Expose the configured layers of one generation-bound LID control.

    A layer property returns ``None`` when that layer is not part of the
    control's configured LID process.
    """

    __slots__ = ()

    def _layer(self, name: str, view_type: type[_LidLayerT]) -> _LidLayerT | None:
        exists = self._simulation._owner.lid_control_layer_exists(
            self._generation, self._index, self._id, self._subtype, name
        )
        return view_type(self, name, _VIEW_TOKEN) if exists else None

    @property
    def surface(self) -> LidSurfaceLayer | None:
        """Return the configured surface layer, if present."""
        return self._layer("surface", LidSurfaceLayer)

    @property
    def soil(self) -> LidSoilLayer | None:
        """Return the configured soil layer, if present."""
        return self._layer("soil", LidSoilLayer)

    @property
    def storage(self) -> LidStorageLayer | None:
        """Return the configured storage layer, if present."""
        return self._layer("storage", LidStorageLayer)

    @property
    def pavement(self) -> LidPavementLayer | None:
        """Return the configured pavement layer, if present."""
        return self._layer("pavement", LidPavementLayer)

    @property
    def drain(self) -> LidDrainLayer | None:
        """Return the configured underdrain layer, if present."""
        return self._layer("drain", LidDrainLayer)

    @property
    def drainage_mat(self) -> LidDrainageMatLayer | None:
        """Return the configured drainage-mat layer, if present."""
        return self._layer("drainage_mat", LidDrainageMatLayer)


class _LidLayer:
    __slots__ = ("_control", "_name")

    def __init__(self, control: LidControl, name: str, token: object) -> None:
        if token is not _VIEW_TOKEN:
            raise TypeError("LID layers are created by LidControl properties")
        self._control = control
        self._name = name

    def _value(self, name: str) -> object:
        return self._control._simulation._owner.lid_control_value(
            self._control._simulation,
            self._control._generation,
            self._control._index,
            self._control._id,
            self._control._subtype,
            self._name,
            name,
        )

    def update(self, **changes: object) -> None:
        """Commit requested layer values atomically in `OPEN` or `ENDED`.

        Intrinsic scalar errors fail immediately. Coupled layer, process, and
        LID-group relationships are validated during the next `start()`.
        """
        self._control._simulation._owner.update_lid_control_layer(
            self._control._generation,
            self._control._index,
            self._control._id,
            self._control._subtype,
            self._name,
            changes,
        )

    def _set(self, name: str, value: object) -> None:
        self.update(**{name: value})

    def __repr__(self) -> str:
        return (
            f"{type(self).__name__}(control_id={self._control._id!r}, "
            f"control_index={self._control._index})"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, _LidLayer):
            return NotImplemented
        return self._control == other._control and self._name == other._name

    def __hash__(self) -> int:
        return hash((self._control, self._name))

    def __copy__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")


@_final
class LidSurfaceLayer(_LidLayer):
    """Expose one configured LID surface layer."""

    __slots__ = ()

    @property
    def thickness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("thickness"))

    @thickness.setter
    def thickness(self, value: float) -> None:
        self._set("thickness", value)

    @property
    def vegetation_volume_fraction(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("vegetation_volume_fraction"))

    @vegetation_volume_fraction.setter
    def vegetation_volume_fraction(self, value: float) -> None:
        self._set("vegetation_volume_fraction", value)

    @property
    def roughness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("roughness"))

    @roughness.setter
    def roughness(self, value: float) -> None:
        self._set("roughness", value)

    @property
    def slope(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("slope"))

    @slope.setter
    def slope(self, value: float) -> None:
        self._set("slope", value)

    @property
    def side_slope(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("side_slope"))

    @side_slope.setter
    def side_slope(self, value: float) -> None:
        self._set("side_slope", value)

    @property
    def alpha(self) -> float:
        """Return the derived surface runoff coefficient in project units."""
        return _cast(float, self._value("alpha"))

    @property
    def immediate_overflow(self) -> bool:
        """Return whether ponded surface water overflows without delay."""
        return _cast(bool, self._value("immediate_overflow"))


@_final
class LidSoilLayer(_LidLayer):
    """Expose one configured LID soil layer."""

    __slots__ = ()

    @property
    def thickness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("thickness"))

    @thickness.setter
    def thickness(self, value: float) -> None:
        self._set("thickness", value)

    @property
    def porosity(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("porosity"))

    @porosity.setter
    def porosity(self, value: float) -> None:
        self._set("porosity", value)

    @property
    def field_capacity(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("field_capacity"))

    @field_capacity.setter
    def field_capacity(self, value: float) -> None:
        self._set("field_capacity", value)

    @property
    def wilting_point(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("wilting_point"))

    @wilting_point.setter
    def wilting_point(self, value: float) -> None:
        self._set("wilting_point", value)

    @property
    def saturated_conductivity(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("saturated_conductivity"))

    @saturated_conductivity.setter
    def saturated_conductivity(self, value: float) -> None:
        self._set("saturated_conductivity", value)

    @property
    def conductivity_slope(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("conductivity_slope"))

    @conductivity_slope.setter
    def conductivity_slope(self, value: float) -> None:
        self._set("conductivity_slope", value)

    @property
    def suction_head(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("suction_head"))

    @suction_head.setter
    def suction_head(self, value: float) -> None:
        self._set("suction_head", value)


@_final
class LidStorageLayer(_LidLayer):
    """Expose one configured LID storage layer."""

    __slots__ = ()

    @property
    def thickness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("thickness"))

    @thickness.setter
    def thickness(self, value: float) -> None:
        self._set("thickness", value)

    @property
    def void_ratio(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("void_ratio"))

    @void_ratio.setter
    def void_ratio(self, value: float) -> None:
        self._set("void_ratio", value)

    @property
    def saturated_conductivity(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("saturated_conductivity"))

    @saturated_conductivity.setter
    def saturated_conductivity(self, value: float) -> None:
        self._set("saturated_conductivity", value)

    @property
    def clogging_factor(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("clogging_factor"))

    @clogging_factor.setter
    def clogging_factor(self, value: float) -> None:
        self._set("clogging_factor", value)


@_final
class LidPavementLayer(_LidLayer):
    """Expose one configured LID pavement layer."""

    __slots__ = ()

    @property
    def thickness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("thickness"))

    @thickness.setter
    def thickness(self, value: float) -> None:
        self._set("thickness", value)

    @property
    def void_ratio(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("void_ratio"))

    @void_ratio.setter
    def void_ratio(self, value: float) -> None:
        self._set("void_ratio", value)

    @property
    def impervious_fraction(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("impervious_fraction"))

    @impervious_fraction.setter
    def impervious_fraction(self, value: float) -> None:
        self._set("impervious_fraction", value)

    @property
    def saturated_conductivity(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("saturated_conductivity"))

    @saturated_conductivity.setter
    def saturated_conductivity(self, value: float) -> None:
        self._set("saturated_conductivity", value)

    @property
    def clogging_factor(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("clogging_factor"))

    @clogging_factor.setter
    def clogging_factor(self, value: float) -> None:
        self._set("clogging_factor", value)

    @property
    def regeneration_interval(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=_cast(float, self._value("regeneration_interval")))

    @regeneration_interval.setter
    def regeneration_interval(self, value: _timedelta) -> None:
        self._set("regeneration_interval", value)

    @property
    def regeneration_fraction(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("regeneration_fraction"))

    @regeneration_fraction.setter
    def regeneration_fraction(self, value: float) -> None:
        self._set("regeneration_fraction", value)


@_final
class LidDrainLayer(_LidLayer):
    """Expose one configured LID drain layer."""

    __slots__ = ()

    @property
    def coefficient(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("coefficient"))

    @coefficient.setter
    def coefficient(self, value: float) -> None:
        self._set("coefficient", value)

    @property
    def exponent(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("exponent"))

    @exponent.setter
    def exponent(self, value: float) -> None:
        self._set("exponent", value)

    @property
    def offset(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("offset"))

    @offset.setter
    def offset(self, value: float) -> None:
        self._set("offset", value)

    @property
    def delay(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=_cast(float, self._value("delay")))

    @delay.setter
    def delay(self, value: _timedelta) -> None:
        self._set("delay", value)

    @property
    def open_head(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("open_head"))

    @open_head.setter
    def open_head(self, value: float) -> None:
        self._set("open_head", value)

    @property
    def close_head(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("close_head"))

    @close_head.setter
    def close_head(self, value: float) -> None:
        self._set("close_head", value)

    @property
    def control_curve(self) -> Curve | None:
        """Return the configured underdrain control curve, if one is assigned."""
        return _cast(Curve | None, self._value("control_curve_identity"))


@_final
class LidDrainageMatLayer(_LidLayer):
    """Expose one configured LID drainage-mat layer."""

    __slots__ = ()

    @property
    def thickness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("thickness"))

    @thickness.setter
    def thickness(self, value: float) -> None:
        self._set("thickness", value)

    @property
    def void_fraction(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("void_fraction"))

    @void_fraction.setter
    def void_fraction(self, value: float) -> None:
        self._set("void_fraction", value)

    @property
    def roughness(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(float, self._value("roughness"))

    @roughness.setter
    def roughness(self, value: float) -> None:
        self._set("roughness", value)

    @property
    def alpha(self) -> float:
        """Return the derived drainage-mat runoff coefficient in project units."""
        return _cast(float, self._value("alpha"))
