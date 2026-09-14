"""Public exception hierarchy for Python simulation operations."""

from __future__ import annotations as _annotations

from dataclasses import dataclass as _dataclass
from typing import final as _final

from ._prettier import pretty_dataclass as _pretty_dataclass
from .enums import SolverErrorCode as _SolverErrorCode

_STANDARD_EXCEPTION_ATTRIBUTES = frozenset(
    (
        "args",
        "__cause__",
        "__context__",
        "__notes__",
        "__suppress_context__",
        "__traceback__",
    )
)


class SwmmError(Exception):
    """Base class for supported package failures.

    Notes
    -----
    Catch this class to handle all public binding failures without catching
    unrelated Python exceptions.
    """

    __slots__ = ("native_code", "operation", "detail", "semantic_code")

    native_code: int | None
    operation: str | None
    detail: str | None
    semantic_code: str | None

    def __init__(
        self,
        message: str = "",
        native_code: int | None = None,
        operation: str | None = None,
        detail: str | None = None,
        semantic_code: str | None = None,
    ) -> None:
        self.native_code = native_code
        self.operation = operation
        self.detail = detail
        self.semantic_code = semantic_code
        super().__init__(message)

    def __setattr__(self, name: str, value: object) -> None:
        slots = {
            slot
            for class_ in type(self).__mro__
            for slot in class_.__dict__.get("__slots__", ())
        }
        if name not in _STANDARD_EXCEPTION_ATTRIBUTES and name not in slots:
            raise AttributeError(f"{type(self).__name__} has no attribute {name!r}")
        super().__setattr__(name, value)


class LifecycleError(SwmmError):
    """Report an operation that is invalid in the current lifecycle state."""

    __slots__ = ()


@_final
class StaleViewError(LifecycleError):
    """Report access through a Live View from an inactive Project Generation."""

    __slots__ = ()


@_final
class ValidationError(SwmmError):
    """Report a public argument rejected before native state mutation."""

    __slots__ = ()


@_pretty_dataclass(frozen=True, slots=True)
class ConfigurationObjectIdentity:
    """Identify one configured object named by a configuration diagnostic."""

    object_type: str
    id: str
    index: int


@_pretty_dataclass(frozen=True, slots=True)
class ConfigurationDiagnostic:
    """Describe one ordered post-open configuration violation."""

    object: ConfigurationObjectIdentity
    property_path: str
    rule_code: str
    message: str
    conflicting_object: ConfigurationObjectIdentity | None


@_final
class ConfigurationError(SwmmError):
    """Report post-open preparation diagnostics without invalidating the owner."""

    __slots__ = ("diagnostics",)

    diagnostics: tuple[ConfigurationDiagnostic, ...]

    def __init__(self, diagnostics: tuple[ConfigurationDiagnostic, ...]) -> None:
        self.diagnostics = diagnostics
        super().__init__(
            f"start: post-open preparation rejected {len(diagnostics)} configuration violation(s)"
        )

    def __str__(self) -> str:
        """Include every ordered structured diagnostic in traceback text."""

        summary = super().__str__()
        if not self.diagnostics:
            return summary
        lines = [summary + ":"]
        for diagnostic in self.diagnostics:
            identity = diagnostic.object
            line = (
                f"- {identity.object_type} {identity.id!r} (index {identity.index}), "
                f"property {diagnostic.property_path!r}, rule {diagnostic.rule_code!r}: "
                f"{diagnostic.message}"
            )
            if diagnostic.conflicting_object is not None:
                conflicting = diagnostic.conflicting_object
                line += (
                    f"; conflicts with {conflicting.object_type} {conflicting.id!r} "
                    f"(index {conflicting.index})"
                )
            lines.append(line)
        return "\n".join(lines)


@_final
class SolverError(SwmmError):
    """Report a project or lifecycle operation rejected by the native solver.

    Attributes
    ----------
    code : SolverErrorCode
        Stable category for the native failure.
    native_code : int
        Numeric SWMM error code.
    operation : str
        Public operation that triggered the failure.
    detail : str or None
        Optional solver-provided diagnostic detail, including any secondary
        cleanup failure retained by the solver.
    semantic_code : str
        Stable semantic failure code.
    """

    __slots__ = ("code",)

    code: _SolverErrorCode
    operation: str
    detail: str | None
    semantic_code: str

    def __init__(
        self,
        code: _SolverErrorCode,
        operation: str,
        detail: str | None = None,
        native_code: int | None = None,
        semantic_code: str | None = None,
    ) -> None:
        self.code = code
        semantic_code = semantic_code or code.value
        message = f"{operation}: {code.message}"
        super().__init__(
            message if detail is None else f"{message}: {detail}",
            native_code,
            operation,
            detail,
            semantic_code,
        )

@_final
class InternalSimulationError(SwmmError):
    """Report an isolated native Simulation Owner that is permanently unavailable."""

    __slots__ = ()


__all__ = (
    "ConfigurationDiagnostic",
    "ConfigurationError",
    "ConfigurationObjectIdentity",
    "InternalSimulationError",
    "LifecycleError",
    "SolverError",
    "StaleViewError",
    "SwmmError",
    "ValidationError",
)
