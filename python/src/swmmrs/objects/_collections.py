"""Shared mapping adapters for configured SWMM object views."""

from __future__ import annotations as _annotations

from collections.abc import Iterable as _Iterable, Iterator as _Iterator, Mapping as _Mapping
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    Generic as _Generic,
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    cast as _cast,
    final as _final,
    overload as _overload,
)

from ._base import _COLLECTION_TOKEN, _T, _ObjectFamily

if _TYPE_CHECKING:
    from .. import Simulation
    from ._links import Link, LinkCollection
    from ._nodes import Node, NodeCollection
    from ._subcatchments import Subcatchment, SubcatchmentCollection


class _Collection(_Mapping[str, _T], _Generic[_T]):
    __slots__ = ("_family", "_generation", "_simulation")

    def __init__(
        self,
        simulation: Simulation,
        generation: int,
        family: _ObjectFamily,
        token: object,
    ) -> None:
        if token is not _COLLECTION_TOKEN:
            raise TypeError("collections are created by Simulation")
        self._simulation = simulation
        self._generation = generation
        self._family = family

    def __getitem__(self, key: str | int) -> _T:
        """Return an object view by case-insensitive configured ID.

        Parameters
        ----------
        key
            Object ID as a string or non-boolean integer.

        Returns
        -------
        _T
            Generation-bound object view.

        Raises
        ------
        KeyError
            If no configured object has the requested ID.
        TypeError
            If `key` is not a string or non-boolean integer.
        """
        return _cast(
            _T,
            self._simulation._owner.collection_view(
                self._simulation, self._generation, self._family, key
            ),
        )

    def by_index(self, index: int) -> _T:
        """Return an object view by its zero-based configured position.

        Parameters
        ----------
        index
            Zero-based position in configured project order.

        Returns
        -------
        _T
            Generation-bound object view.

        Raises
        ------
        IndexError
            If `index` is outside the configured collection.
        TypeError
            If `index` is not a non-boolean integer.
        """
        return _cast(
            _T,
            self._simulation._owner.collection_view_at(
                self._simulation, self._generation, self._family, index
            ),
        )

    def __iter__(self) -> _Iterator[str]:
        """Return an iterator over object IDs in configured project order."""
        return iter(self._simulation._owner.collection_ids(self._generation, self._family))

    def __len__(self) -> int:
        """Return the number of configured objects in this collection."""
        return self._simulation._owner.collection_count(self._generation, self._family)

    def __repr__(self) -> str:
        identifiers = self._simulation._owner.collection_ids(self._generation, self._family)
        return f"{type(self).__name__}({identifiers!r})"

    def __contains__(self, key: object) -> bool:
        """Return whether a case-insensitive object ID is configured.

        Parameters
        ----------
        key
            Candidate object ID.

        Returns
        -------
        bool
            `True` when `key` identifies a configured object; otherwise `False`.
        """
        return self._simulation._owner.collection_contains(self._generation, self._family, key)

    def __copy__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError(f"{type(self).__name__} cannot be pickled")


@_final
class ObjectCollection(_Collection[_T], _Generic[_T]):
    """Provide mapping-style access to configured project objects.

    Notes
    -----
    Object IDs are case-insensitive. Use `by_index()` for configured order.
    """

    __slots__ = ()


@_overload
def _new_collection(
    simulation: Simulation,
    generation: int,
    family: _ObjectFamily,
    collection_type: type[ObjectCollection[_T]],
) -> ObjectCollection[_T]: ...


@_overload
def _new_collection(
    simulation: Simulation,
    generation: int,
    family: _ObjectFamily,
    collection_type: type[NodeCollection],
) -> NodeCollection: ...


@_overload
def _new_collection(
    simulation: Simulation,
    generation: int,
    family: _ObjectFamily,
    collection_type: type[LinkCollection],
) -> LinkCollection: ...


@_overload
def _new_collection(
    simulation: Simulation,
    generation: int,
    family: _ObjectFamily,
    collection_type: type[SubcatchmentCollection],
) -> SubcatchmentCollection: ...


def _new_collection(
    simulation: Simulation,
    generation: int,
    family: _ObjectFamily,
    collection_type: type[_Collection[_T]],
) -> _Collection[_T]:
    return collection_type(simulation, generation, family, _COLLECTION_TOKEN)
