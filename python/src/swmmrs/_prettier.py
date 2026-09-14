"""Vendored multiline formatting for public dataclass representations.

Adapted from ``devtools.prettier`` in Samuel Colvin's ``python-devtools``:
https://github.com/samuelcolvin/python-devtools/blob/main/devtools/prettier.py

The upstream implementation is MIT licensed. This reduced, dependency-free
adaptation supports the standard-library values used by swmmrs and dataclasses
with slots. It deliberately omits optional syntax highlighting, SQLAlchemy,
and generator support.

Copyright (c) 2017 to present Samuel Colvin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

from __future__ import annotations

import io
from collections import OrderedDict
from collections.abc import Iterable, Mapping
from dataclasses import dataclass, fields, is_dataclass
from typing import Any, Callable, TypeVar, cast, dataclass_transform, overload

__all__ = ("PrettyFormat", "pformat", "pretty_dataclass")

_T = TypeVar("_T")
_DEFAULT_WIDTH = 120


class PrettyFormat:
    """Format standard Python values and dataclasses across multiple lines."""

    def __init__(
        self,
        *,
        indent_step: int = 4,
        indent_char: str = " ",
        simple_cutoff: int = 10,
        width: int = _DEFAULT_WIDTH,
    ) -> None:
        """Initialize a formatter with the requested layout controls.

        Parameters
        ----------
        indent_step : int, default=4
            Number of indent characters per nesting level.
        indent_char : str, default=" "
            Character used for indentation.
        simple_cutoff : int, default=10
            Maximum representation length rendered without structural formatting.
        width : int, default=120
            Maximum preferred output width before wrapping raw representations.
        """

        self._indent_step = indent_step
        self._indent_char = indent_char
        self._simple_cutoff = simple_cutoff
        self._width = width
        self._stream = io.StringIO()
        self._active_ids: set[int] = set()

    def __call__(self, value: object, *, indent: int = 0) -> str:
        """Format one value.

        Parameters
        ----------
        value : object
            Value to format.
        indent : int, default=0
            Initial indentation level.

        Returns
        -------
        str
            Deterministic, human-readable value representation.
        """

        self._stream = io.StringIO()
        self._active_ids.clear()
        self._format(value, indent_current=indent, indent_first=False)
        return self._stream.getvalue()

    def format_dataclass(self, value: object) -> str:
        """Format a dataclass directly without first calling its ``repr``.

        Parameters
        ----------
        value : object
            Dataclass instance to format.

        Returns
        -------
        str
            Multiline dataclass representation.

        Raises
        ------
        TypeError
            If ``value`` is not a dataclass instance.
        """

        if not is_dataclass(value) or isinstance(value, type):
            raise TypeError("value must be a dataclass instance")
        self._stream = io.StringIO()
        self._active_ids.clear()
        self._format_dataclass(value, indent_current=0, indent_new=self._indent_step)
        return self._stream.getvalue()

    def _format(self, value: object, *, indent_current: int, indent_first: bool) -> None:
        if indent_first:
            self._stream.write(indent_current * self._indent_char)

        if is_dataclass(value) and not isinstance(value, type):
            value_id = id(value)
            if value_id in self._active_ids:
                self._stream.write(f"<Recursion on {type(value).__name__}>")
                return
            self._format_dataclass(
                value,
                indent_current=indent_current,
                indent_new=indent_current + self._indent_step,
            )
            return

        value_repr = repr(value)
        if len(value_repr) <= self._simple_cutoff:
            self._stream.write(value_repr)
        elif isinstance(value, Mapping):
            self._format_mapping(
                cast(Mapping[object, object], value),
                indent_current,
                indent_current + self._indent_step,
            )
        elif isinstance(value, tuple):
            self._format_tuple(value, indent_current, indent_current + self._indent_step)
        elif isinstance(value, (list, set, frozenset)):
            self._format_sequence(
                cast(list[object] | set[object] | frozenset[object], value),
                indent_current,
                indent_current + self._indent_step,
            )
        else:
            self._format_raw(value_repr, indent_current, indent_current + self._indent_step)

    def _format_dataclass(self, value: object, *, indent_current: int, indent_new: int) -> None:
        value_id = id(value)
        if value_id in self._active_ids:
            self._stream.write(f"<Recursion on {type(value).__name__}>")
            return

        self._active_ids.add(value_id)
        try:
            self._stream.write(f"{type(value).__name__}(\n")
            for field in fields(cast(Any, value)):
                if not field.repr:
                    continue
                self._stream.write(indent_new * self._indent_char)
                self._stream.write(f"{field.name}=")
                self._format(
                    getattr(value, field.name),
                    indent_current=indent_new,
                    indent_first=False,
                )
                self._stream.write(",\n")
            self._stream.write(indent_current * self._indent_char)
            self._stream.write(")")
        finally:
            self._active_ids.remove(value_id)

    def _format_mapping(
        self, value: Mapping[object, object], indent_current: int, indent_new: int
    ) -> None:
        if isinstance(value, OrderedDict):
            open_, close_ = "OrderedDict([\n", "])"
            prefix, separator = "(", ", "
        elif type(value) is dict:
            open_, close_ = "{\n", "}"
            prefix, separator = "", ": "
        else:
            open_, close_ = f"<{type(value).__name__}({{\n", "})>"
            prefix, separator = "", ": "

        self._stream.write(open_)
        for key, item in value.items():
            self._stream.write(indent_new * self._indent_char + prefix)
            self._format(key, indent_current=indent_new, indent_first=False)
            self._stream.write(separator)
            self._format(item, indent_current=indent_new, indent_first=False)
            self._stream.write(",\n")
        self._stream.write(indent_current * self._indent_char + close_)

    def _format_tuple(
        self, value: tuple[object, ...], indent_current: int, indent_new: int
    ) -> None:
        fields_ = getattr(value, "_fields", None)
        if fields_ is not None:
            self._format_fields(
                type(value).__name__, zip(fields_, value), indent_current, indent_new
            )
            return
        self._format_sequence(value, indent_current, indent_new)

    def _format_sequence(
        self,
        value: list[object] | tuple[object, ...] | set[object] | frozenset[object],
        indent_current: int,
        indent_new: int,
    ) -> None:
        if isinstance(value, list):
            open_, close_ = "[", "]"
        elif isinstance(value, set):
            open_, close_ = "{", "}"
        elif isinstance(value, frozenset):
            open_, close_ = "frozenset({", "})"
        else:
            open_, close_ = "(", ")"

        self._stream.write(open_ + "\n")
        for item in value:
            self._format(item, indent_current=indent_new, indent_first=True)
            self._stream.write(",\n")
        self._stream.write(indent_current * self._indent_char + close_)

    def _format_raw(self, value_repr: str, indent_current: int, indent_new: int) -> None:
        if len(value_repr) + indent_current < self._width and "\n" not in value_repr:
            self._stream.write(value_repr)
            return

        self._stream.write("(\n")
        line_width = self._width - indent_new
        for line in value_repr.splitlines() or [value_repr]:
            for start in range(0, len(line), line_width):
                self._stream.write(indent_new * self._indent_char)
                self._stream.write(line[start : start + line_width])
                self._stream.write("\n")
        self._stream.write(indent_current * self._indent_char + ")")

    def _format_fields(
        self,
        name: str,
        values: Iterable[tuple[str, object]],
        indent_current: int,
        indent_new: int,
    ) -> None:
        self._stream.write(f"{name}(\n")
        for field_name, value in values:
            self._stream.write(indent_new * self._indent_char + f"{field_name}=")
            self._format(value, indent_current=indent_new, indent_first=False)
            self._stream.write(",\n")
        self._stream.write(indent_current * self._indent_char + ")")


def pformat(value: object, *, width: int = _DEFAULT_WIDTH) -> str:
    """Format a value with a fresh formatter instance.

    Parameters
    ----------
    value : object
        Value to format.
    width : int, default=120
        Maximum preferred output width.

    Returns
    -------
    str
        Deterministic, human-readable value representation.
    """

    return PrettyFormat(width=width)(value)


@overload
def pretty_dataclass(cls: _T, **kwargs: Any) -> _T: ...


@overload
def pretty_dataclass(cls: None = None, **kwargs: Any) -> Callable[[_T], _T]: ...


@dataclass_transform()
def pretty_dataclass(cls: _T | None = None, **kwargs: Any) -> _T | Callable[[_T], _T]:
    """Create a dataclass with multiline ``repr`` and ``str`` methods.

    The decorator accepts the same keyword arguments as :func:`dataclasses.dataclass`.
    It also accepts an existing dataclass directly for compatibility with the original
    two-decorator form.

    Parameters
    ----------
    cls : type, optional
        Existing dataclass type when used without parentheses.
    **kwargs : object
        Keyword arguments forwarded to :func:`dataclasses.dataclass`.

    Returns
    -------
    type or callable
        Decorated dataclass, or a decorator when called with keyword arguments.

    Raises
    ------
    TypeError
        If `cls` is already a dataclass and keyword arguments are also supplied.
    """

    def decorate(target: _T) -> _T:
        result = target
        if is_dataclass(target):
            if kwargs:
                raise TypeError("pretty_dataclass cannot reconfigure an existing dataclass")
        else:
            result = cast(_T, dataclass(**kwargs)(cast(Any, target)))

        def representation(self: object) -> str:
            return PrettyFormat().format_dataclass(self)

        result_type = cast(Any, result)
        result_type.__repr__ = representation
        result_type.__str__ = representation
        return result

    return decorate if cls is None else decorate(cls)
