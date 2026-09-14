"""Shared implementation for generation-bound SWMM object views."""

from __future__ import annotations as _annotations

from collections.abc import Mapping as _Mapping
from enum import IntEnum as _IntEnum
from types import MappingProxyType as _MappingProxyType
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    TypeVar as _TypeVar,
)

from ..exceptions import StaleViewError as _StaleViewError

if _TYPE_CHECKING:
    from .. import Simulation


class _ObjectFamily(_IntEnum):
    RAIN_GAGE = 0
    SUBCATCHMENT = 1
    NODE = 2
    LINK = 3
    POLLUTANT = 4
    LAND_USE = 5
    TIME_PATTERN = 6
    CURVE = 7
    TIME_SERIES = 8
    CONTROL = 9
    TRANSECT = 10
    AQUIFER = 11
    UNIT_HYDROGRAPH = 12
    SNOWMELT_SET = 13
    SHAPE = 14
    LID_CONTROL = 15
    STREET = 16
    INLET_DESIGN = 17
    AMM_MODEL = 18


_T = _TypeVar("_T", bound="_LiveView")
_VIEW_TOKEN = object()
_TRUSTED_VIEW_TOKEN = _VIEW_TOKEN
_INLET_TOKEN = object()
_OPTIONS_TOKEN = object()
_COLLECTION_TOKEN = object()


def _pollutant_mapping(
    values: tuple[list[str], list[float]],
) -> _Mapping[str, float]:
    identifiers, numbers = values
    return _MappingProxyType(dict(zip(identifiers, numbers, strict=True)))


class _LiveView:
    __slots__ = ("_family", "_generation", "_id", "_index", "_simulation", "_subtype")

    def __init__(
        self,
        simulation: Simulation,
        generation: int,
        family: _ObjectFamily,
        index: int,
        identifier: str,
        subtype: str,
        token: object,
    ) -> None:
        if token is not _TRUSTED_VIEW_TOKEN:
            raise TypeError("Live Views are created by Simulation collections")
        self._simulation = simulation
        self._generation = generation
        self._family = family
        self._index = index
        self._id = identifier
        self._subtype = subtype

    def _validate(self) -> None:
        self._simulation._owner.validate_identity(
            self._generation, self._family, self._index, self._id, self._subtype
        )

    @property
    def id(self) -> str:
        self._validate()
        return self._id

    @property
    def index(self) -> int:
        self._validate()
        return self._index

    def __repr__(self) -> str:
        return f"{type(self).__name__}(id={self._id!r}, index={self._index})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, _LiveView):
            return NotImplemented
        return self._identity == other._identity

    def __hash__(self) -> int:
        return hash(self._identity)

    @property
    def _identity(self) -> tuple[int, int, int, int, str]:
        return (
            id(self._simulation),
            self._generation,
            self._family,
            self._index,
            self._subtype,
        )

    def __copy__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")
