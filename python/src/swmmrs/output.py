"""Read finalized or incomplete SWMM binary output without solver ownership."""

from __future__ import annotations

import math
import os
from bisect import bisect_left
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import StrEnum
from pathlib import Path
from threading import Lock
from typing import Final, TypeVar, cast, final, overload

from ._prettier import pretty_dataclass as _pretty_dataclass
from ._swmmrs import (
    NativeOutputError as _NativeOutputError,
    NativeOutputReader as _NativeOutputReader,
)
from .enums import FlowUnits, LinkKind, NodeKind, UnitSystem

__all__ = [
    "ConcentrationUnits",
    "FlowUnits",
    "LinkKind",
    "LinkMetadata",
    "LinkResultAttribute",
    "NodeKind",
    "NodeMetadata",
    "NodeResultAttribute",
    "OutputError",
    "OutputMetadata",
    "OutputName",
    "OutputReader",
    "OutputTimeSeries",
    "OutputValueSeries",
    "BulkSeriesResult",
    "PollutantMetadata",
    "ReportTiming",
    "ResultAttributeCode",
    "ResultElementType",
    "ResultSchema",
    "RunStatus",
    "SeriesSelection",
    "SubcatchmentMetadata",
    "SubcatchmentResultAttribute",
    "SystemResultAttribute",
    "UnknownCode",
    "UnitSystem",
]

_PathInput = str | os.PathLike[str]
_T = TypeVar("_T")
_StrEnumT = TypeVar("_StrEnumT", bound=StrEnum)
_I32_MIN: Final = -(2**31)
_I32_MAX: Final = 2**31 - 1
_SWMM_EPOCH = datetime(1899, 12, 30)


@final
class ResultElementType(StrEnum):
    """Identify one physical output-result family."""

    SUBCATCHMENT = "subcatchment"
    NODE = "node"
    LINK = "link"
    SYSTEM = "system"


@final
class ConcentrationUnits(StrEnum):
    """Identify pollutant concentration units stored in the output file."""

    MILLIGRAMS_PER_LITER = "milligrams_per_liter"
    MICROGRAMS_PER_LITER = "micrograms_per_liter"
    COUNTS_PER_LITER = "counts_per_liter"


@final
class SubcatchmentResultAttribute(StrEnum):
    """Identify one known SWMM/PySWMM subcatchment result attribute."""

    RAINFALL = "rainfall"
    SNOW_DEPTH = "snow_depth"
    EVAP_LOSS = "evap_loss"
    INFIL_LOSS = "infil_loss"
    RUNOFF_RATE = "runoff_rate"
    GW_OUTFLOW_RATE = "gw_outflow_rate"
    GW_TABLE_ELEV = "gw_table_elev"
    SOIL_MOISTURE = "soil_moisture"


@final
class NodeResultAttribute(StrEnum):
    """Identify one known SWMM/PySWMM node result attribute."""

    INVERT_DEPTH = "invert_depth"
    HYDRAULIC_HEAD = "hydraulic_head"
    PONDED_VOLUME = "ponded_volume"
    LATERAL_INFLOW = "lateral_inflow"
    TOTAL_INFLOW = "total_inflow"
    FLOODING_LOSSES = "flooding_losses"


@final
class LinkResultAttribute(StrEnum):
    """Identify one known SWMM/PySWMM link result attribute."""

    FLOW_RATE = "flow_rate"
    FLOW_DEPTH = "flow_depth"
    FLOW_VELOCITY = "flow_velocity"
    FLOW_VOLUME = "flow_volume"
    CAPACITY = "capacity"


@final
class SystemResultAttribute(StrEnum):
    """Identify one known SWMM/PySWMM system result attribute."""

    AIR_TEMP = "air_temp"
    RAINFALL = "rainfall"
    SNOW_DEPTH = "snow_depth"
    EVAP_INFIL_LOSS = "evap_infil_loss"
    RUNOFF_FLOW = "runoff_flow"
    DRY_WEATHER_INFLOW = "dry_weather_inflow"
    GW_INFLOW = "gw_inflow"
    RDII_INFLOW = "rdii_inflow"
    DIRECT_INFLOW = "direct_inflow"
    TOTAL_LATERAL_INFLOW = "total_lateral_inflow"
    FLOOD_LOSSES = "flood_losses"
    OUTFALL_FLOWS = "outfall_flows"
    VOLUME_STORED = "volume_stored"
    EVAP_RATE = "evap_rate"
    PTNL_EVAP_RATE = "ptnl_evap_rate"


def _signed_code(value: object, name: str) -> int:
    """Validate one exact signed 32-bit code."""

    if type(value) is not int:
        raise TypeError(f"{name} must be an int")
    if not _I32_MIN <= value <= _I32_MAX:
        raise ValueError(f"{name} must fit a signed 32-bit integer")
    return value


def _nonnegative_index(value: object, name: str) -> int:
    """Validate one exact nonnegative integer identity."""

    if type(value) is not int:
        raise TypeError(f"{name} must be an int")
    if value < 0:
        raise ValueError(f"{name} must be nonnegative")
    return value


def _enum_value(value: object, enum: type[_StrEnumT], name: str) -> _StrEnumT:
    """Normalize one exact canonical enum value."""

    if isinstance(value, enum):
        return value
    if type(value) is not str:
        raise TypeError(f"{name} must be {enum.__name__} or its canonical string")
    try:
        return enum(value)
    except ValueError as error:
        raise ValueError(f"unknown {name} {value!r}") from error


class OutputError(Exception):
    """Report one structured binary-output failure.

    Attributes
    ----------
    category : str
        Stable high-level failure category.
    """

    __slots__ = ("category",)

    def __init__(self, category: str, message: str) -> None:
        """Create an output error.

        Parameters
        ----------
        category : str
            Stable high-level failure category.
        message : str
            Descriptive native failure text.

        Examples
        --------
        >>> error = OutputError("invalid_format", "bad output file")
        >>> error.category
        'invalid_format'
        """

        super().__init__(message)
        self.category = category


def _nominal_datetime(serial_days: float) -> datetime:
    """Convert one finite SWMM serial day to a rounded naive datetime."""

    total_seconds = serial_days * 86_400.0
    if not math.isfinite(total_seconds):
        raise OutputError(
            "datetime_out_of_range",
            "nominal report date is outside Python datetime range",
        )
    try:
        rounded_seconds = round(total_seconds)
        return _SWMM_EPOCH + timedelta(seconds=rounded_seconds)
    except (OverflowError, ValueError) as error:
        raise OutputError(
            "datetime_out_of_range",
            "nominal report date is outside Python datetime range",
        ) from error


@dataclass(frozen=True, slots=True)
class ReportTiming:
    """Immutable report schedule and available complete period count."""

    report_schedule_origin: float
    report_step_seconds: int
    period_count: int

    def __repr__(self) -> str:
        """Summarize the schedule with compact field names."""

        return (
            f"{type(self).__name__}(origin={self.report_schedule_origin!r}, "
            f"step={self.report_step_seconds}, periods={self.period_count})"
        )

    def _nominal_serial(self, period: int) -> float:
        value = (
            self.report_schedule_origin
            + float(period + 1) * float(self.report_step_seconds) / 86_400.0
        )
        if not math.isfinite(value):
            raise OutputError(
                "datetime_out_of_range",
                "nominal report date is outside Python datetime range",
            )
        return value

    def nominal_date(self, period: int) -> datetime | None:
        """Return the rounded naive date for a zero-based report period.

        Parameters
        ----------
        period : int
            Zero-based report-period offset.

        Returns
        -------
        datetime or None
            Nominal Report Date rounded to a whole second using round-half-to-even,
            or ``None`` when ``period`` is outside the available report range.

        Raises
        ------
        TypeError
            If ``period`` is not an integer.
        OutputError
            If the nominal date is outside Python's datetime range.

        Examples
        --------
        >>> timing = ReportTiming(0.0, 300, 2)
        >>> timing.nominal_date(0)
        datetime.datetime(1899, 12, 30, 0, 5)
        >>> timing.nominal_date(2) is None
        True
        """

        if type(period) is not int:
            raise TypeError("period must be an int")
        if period < 0 or period >= self.period_count:
            return None
        return _nominal_datetime(self._nominal_serial(period))


@_pretty_dataclass
@dataclass(frozen=True, slots=True)
class UnknownCode:
    """Preserve one unknown signed 32-bit categorical code."""

    code: int

    def __post_init__(self) -> None:
        """Reject booleans and values outside the stored signed range."""

        _signed_code(self.code, "code")


@_pretty_dataclass
@dataclass(frozen=True, slots=True)
class ResultAttributeCode:
    """Preserve one unknown signed 32-bit result-attribute code."""

    code: int

    def __post_init__(self) -> None:
        """Reject booleans and values outside the stored signed range."""

        _signed_code(self.code, "code")


@_pretty_dataclass
@dataclass(frozen=True, slots=True)
class RunStatus:
    """Preserve a finalized status code or mark an unfinalized output."""

    code: int | None

    def __post_init__(self) -> None:
        """Validate a stored signed code when one is available."""

        if self.code is not None:
            _signed_code(self.code, "code")

    @property
    def is_finalized(self) -> bool:
        """Report whether the output contained a final trailer."""

        return self.code is not None

    @property
    def is_success(self) -> bool:
        """Report whether the finalized run completed without a warning.

        Returns
        -------
        bool
            ``True`` only when the exact stored status code is zero.

        Examples
        --------
        >>> RunStatus(0).is_success
        True
        >>> RunStatus(None).is_success
        False
        """

        return self.code == 0


@dataclass(frozen=True, slots=True)
class OutputName:
    """Preserve exact stored bytes with an optional lossless UTF-8 view."""

    raw: bytes

    def __post_init__(self) -> None:
        """Require exact immutable bytes without decoding or normalization."""

        if type(self.raw) is not bytes:
            raise TypeError("raw must be bytes")

    @property
    def text(self) -> str | None:
        """Decode the exact stored name when it is valid UTF-8.

        Returns
        -------
        str or None
            Lossless UTF-8 text, or ``None`` when the stored bytes are not UTF-8.

        Examples
        --------
        >>> OutputName(b"J1").text
        'J1'
        >>> OutputName(b"\\xff").text is None
        True
        """

        try:
            return self.raw.decode("utf-8")
        except UnicodeDecodeError:
            return None

    def __repr__(self) -> str:
        """Render decoded names compactly while preserving invalid bytes."""

        return f"{type(self).__name__}({self.text if self.text is not None else self.raw!r})"


_PollutantSelector = int | str | bytes | OutputName


@_pretty_dataclass
@dataclass(frozen=True, slots=True)
class PollutantAttribute:
    """Select one pollutant result attribute."""

    selector: _PollutantSelector
    _code: int | None = field(default=None, init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        """Validate an index or exact stored-name selector."""

        selector = self.selector
        if type(selector) is int:
            _nonnegative_index(selector, "pollutant index")
        elif type(selector) is str:
            try:
                selector.encode("utf-8")
            except UnicodeEncodeError as error:
                raise ValueError("selector must be valid UTF-8 text") from error
        elif type(selector) is bytes or isinstance(selector, OutputName):
            pass
        else:
            raise TypeError("selector must be a nonnegative int, exact str, bytes, or OutputName")


@dataclass(frozen=True, slots=True)
class SubcatchmentMetadata:
    """Exact stored subcatchment identity and area."""

    index: int
    name: OutputName
    area: float

    def __repr__(self) -> str:
        """Summarize the subcatchment identity and area."""

        return f"{type(self).__name__}(index={self.index}, name={self.name!r}, area={self.area!r})"


@dataclass(frozen=True, slots=True)
class NodeMetadata:
    """Exact stored node identity, kind, and static properties."""

    index: int
    name: OutputName
    kind: NodeKind | UnknownCode
    invert_elevation: float
    maximum_depth: float

    def __repr__(self) -> str:
        """Summarize the node identity and kind."""

        return f"{type(self).__name__}(index={self.index}, name={self.name!r}, kind={self.kind!r})"


@dataclass(frozen=True, slots=True)
class LinkMetadata:
    """Exact stored link identity, kind, and static properties."""

    index: int
    name: OutputName
    kind: LinkKind | UnknownCode
    inlet_offset: float
    outlet_offset: float
    maximum_depth: float
    length: float

    def __repr__(self) -> str:
        """Summarize the link identity and kind."""

        return f"{type(self).__name__}(index={self.index}, name={self.name!r}, kind={self.kind!r})"


@dataclass(frozen=True, slots=True)
class PollutantMetadata:
    """Exact stored pollutant identity, name, and concentration units."""

    index: int
    name: OutputName
    concentration_units: ConcentrationUnits | UnknownCode

    def __repr__(self) -> str:
        """Summarize the pollutant identity and concentration units."""

        return (
            f"{type(self).__name__}(index={self.index}, name={self.name!r}, "
            f"concentration_units={self.concentration_units!r})"
        )


_SubcatchmentSchemaEntry = SubcatchmentResultAttribute | PollutantAttribute | ResultAttributeCode
_NodeSchemaEntry = NodeResultAttribute | PollutantAttribute | ResultAttributeCode
_LinkSchemaEntry = LinkResultAttribute | PollutantAttribute | ResultAttributeCode
_SystemSchemaEntry = SystemResultAttribute | ResultAttributeCode


@dataclass(frozen=True, slots=True)
class ResultSchema:
    """Ordered, immutable family-specific result schema entries."""

    subcatchment: tuple[_SubcatchmentSchemaEntry, ...]
    node: tuple[_NodeSchemaEntry, ...]
    link: tuple[_LinkSchemaEntry, ...]
    system: tuple[_SystemSchemaEntry, ...]
    _codes: tuple[tuple[int | None, ...], ...] = field(
        default=(),
        init=False,
        repr=False,
        compare=False,
    )

    def __post_init__(self) -> None:
        """Freeze collections and reject attributes from another family."""

        families: tuple[
            tuple[str, tuple[object, ...], type[object], bool],
            ...,
        ] = (
            (
                "subcatchment",
                tuple(self.subcatchment),
                SubcatchmentResultAttribute,
                True,
            ),
            ("node", tuple(self.node), NodeResultAttribute, True),
            ("link", tuple(self.link), LinkResultAttribute, True),
            ("system", tuple(self.system), SystemResultAttribute, False),
        )
        codes: list[tuple[int | None, ...]] = []
        for name, entries, family, pollutants_allowed in families:
            family_codes: list[int | None] = []
            for entry in entries:
                valid = isinstance(entry, family) or isinstance(entry, ResultAttributeCode)
                if pollutants_allowed:
                    valid = valid or isinstance(entry, PollutantAttribute)
                if not valid:
                    raise TypeError(f"{name} schema contains a wrong-family entry")
                if isinstance(entry, PollutantAttribute) and type(entry.selector) is not int:
                    raise TypeError(f"{name} schema pollutant entries require a resolved index")
                family_codes.append(_entry_code(entry))
            object.__setattr__(self, name, entries)
            codes.append(tuple(family_codes))
        object.__setattr__(self, "_codes", tuple(codes))

    def __repr__(self) -> str:
        """Summarize result families by entry count."""

        return (
            f"{type(self).__name__}(subcatchment={len(self.subcatchment)}, "
            f"node={len(self.node)}, link={len(self.link)}, system={len(self.system)})"
        )


@dataclass(frozen=True, slots=True)
class OutputMetadata:
    """Complete immutable metadata parsed from one SWMM output file."""

    solver_release: int
    run_status: RunStatus
    flow_units: FlowUnits | UnknownCode
    unit_system: UnitSystem | None
    report_timing: ReportTiming
    subcatchments: tuple[SubcatchmentMetadata, ...]
    nodes: tuple[NodeMetadata, ...]
    links: tuple[LinkMetadata, ...]
    pollutants: tuple[PollutantMetadata, ...]
    result_schema: ResultSchema
    source_path: Path

    def __post_init__(self) -> None:
        """Freeze metadata collections and preserve the source path."""

        if type(self.solver_release) is not int:
            raise TypeError("solver_release must be an int")
        if not isinstance(self.run_status, RunStatus):
            raise TypeError("run_status must be a RunStatus")
        if not isinstance(self.flow_units, (FlowUnits, UnknownCode)):
            raise TypeError("flow_units must be FlowUnits or UnknownCode")
        if self.unit_system is not None and not isinstance(self.unit_system, UnitSystem):
            raise TypeError("unit_system must be UnitSystem or None")
        if not isinstance(self.report_timing, ReportTiming):
            raise TypeError("report_timing must be a ReportTiming")
        if not isinstance(self.result_schema, ResultSchema):
            raise TypeError("result_schema must be ResultSchema")
        if not isinstance(self.source_path, Path):
            raise TypeError("source_path must be a Path")
        object.__setattr__(self, "subcatchments", tuple(self.subcatchments))
        object.__setattr__(self, "nodes", tuple(self.nodes))
        object.__setattr__(self, "links", tuple(self.links))
        object.__setattr__(self, "pollutants", tuple(self.pollutants))

    def __repr__(self) -> str:
        """Summarize output identity, run details, and collection sizes."""

        flow_units = (
            self.flow_units.value
            if isinstance(self.flow_units, FlowUnits)
            else self.flow_units.code
        )
        unit_system = self.unit_system.value if self.unit_system is not None else None
        return (
            f"{type(self).__name__}(source={self.source_path.name!r}, "
            f"solver_release={self.solver_release}, status={self.run_status.code}, "
            f"flow_units={flow_units!r}, unit_system={unit_system!r}, "
            f"periods={self.report_timing.period_count}, "
            f"subcatchments={len(self.subcatchments)}, nodes={len(self.nodes)}, "
            f"links={len(self.links)}, pollutants={len(self.pollutants)})"
        )


_ElementSelector = int | str | bytes | OutputName
_SelectionAttribute = (
    SubcatchmentResultAttribute
    | NodeResultAttribute
    | LinkResultAttribute
    | SystemResultAttribute
    | PollutantAttribute
    | ResultAttributeCode
)


def _entry_code(entry: object) -> int | None:
    """Return one stored result code when the entry carries one."""

    if isinstance(entry, ResultAttributeCode):
        return entry.code
    if isinstance(entry, PollutantAttribute):
        return entry._code
    if isinstance(entry, SubcatchmentResultAttribute):
        return tuple(SubcatchmentResultAttribute).index(entry)
    if isinstance(entry, NodeResultAttribute):
        return tuple(NodeResultAttribute).index(entry)
    if isinstance(entry, LinkResultAttribute):
        return tuple(LinkResultAttribute).index(entry)
    if isinstance(entry, SystemResultAttribute):
        return tuple(SystemResultAttribute).index(entry)
    raise TypeError("attribute must match the selected result family")


def _element_bytes(selector: str | bytes | OutputName, name: str) -> bytes:
    if type(selector) is str:
        try:
            return selector.encode("utf-8")
        except UnicodeEncodeError as error:
            raise ValueError(f"{name} must be valid UTF-8 text") from error
    if type(selector) is bytes:
        return selector
    if isinstance(selector, OutputName):
        return selector.raw
    raise TypeError(f"{name} must be an exact str, bytes, or OutputName")


def _validate_element(selector: object) -> None:
    if type(selector) is int:
        _nonnegative_index(selector, "element")
    elif type(selector) is str:
        _element_bytes(selector, "element")
    elif type(selector) is bytes or isinstance(selector, OutputName):
        return
    else:
        raise TypeError("element must be a nonnegative int, exact str, bytes, or OutputName")


@_pretty_dataclass
@dataclass(frozen=True, slots=True)
class SeriesSelection:
    """Select one stored result attribute for one output element."""

    element_type: ResultElementType | str
    element: _ElementSelector | None
    attribute: _SelectionAttribute | str

    def __post_init__(self) -> None:
        """Normalize and validate one family, element, and attribute."""

        element_type = _enum_value(self.element_type, ResultElementType, "element_type")
        object.__setattr__(self, "element_type", element_type)
        if element_type is ResultElementType.SYSTEM:
            if self.element is not None:
                raise ValueError("system selections do not carry an element")
        else:
            _validate_element(self.element)
        family = {
            ResultElementType.SUBCATCHMENT: SubcatchmentResultAttribute,
            ResultElementType.NODE: NodeResultAttribute,
            ResultElementType.LINK: LinkResultAttribute,
            ResultElementType.SYSTEM: SystemResultAttribute,
        }[element_type]
        attribute = self.attribute
        if type(attribute) is str:
            attribute = _enum_value(attribute, family, "attribute")
        elif isinstance(attribute, PollutantAttribute):
            if element_type is ResultElementType.SYSTEM:
                raise ValueError("system selections do not support pollutants")
        elif isinstance(attribute, ResultAttributeCode) or isinstance(attribute, family):
            pass
        else:
            raise TypeError("attribute must match the selected result family")
        object.__setattr__(self, "attribute", attribute)

    def _native(self, metadata: OutputMetadata) -> tuple[str, int, int]:
        """Resolve this selection to the package-private native tuple."""

        element_type = self.element_type
        if not isinstance(element_type, ResultElementType):
            raise TypeError("element_type is not normalized")
        if element_type is ResultElementType.SYSTEM:
            index = 0
        else:
            if self.element is None:
                raise TypeError("non-system selections require an element")
            index = _resolve_element(metadata, element_type, self.element)
        code = _resolve_attribute(metadata, element_type, self.attribute)
        return element_type.value, index, code


def _schema_entries(
    metadata: OutputMetadata,
    element_type: ResultElementType,
) -> tuple[tuple[object, int | None], ...]:
    schemas: tuple[tuple[object, ...], ...] = (
        metadata.result_schema.subcatchment,
        metadata.result_schema.node,
        metadata.result_schema.link,
        metadata.result_schema.system,
    )
    index = tuple(ResultElementType).index(element_type)
    entries = schemas[index]
    codes = metadata.result_schema._codes[index]
    return tuple(zip(entries, codes, strict=True))


def _resolve_element(
    metadata: OutputMetadata,
    element_type: ResultElementType,
    selector: _ElementSelector,
) -> int:
    if type(selector) is int:
        return selector
    if not isinstance(selector, (str, bytes, OutputName)):
        raise TypeError("element must be an exact str, bytes, or OutputName")
    raw = _element_bytes(selector, "element")
    if element_type is ResultElementType.SUBCATCHMENT:
        matches = [
            index for index, record in enumerate(metadata.subcatchments) if record.name.raw == raw
        ]
    elif element_type is ResultElementType.NODE:
        matches = [index for index, record in enumerate(metadata.nodes) if record.name.raw == raw]
    elif element_type is ResultElementType.LINK:
        matches = [index for index, record in enumerate(metadata.links) if record.name.raw == raw]
    else:
        raise KeyError(element_type)
    if not matches:
        raise OutputError(
            "element_not_found",
            f"{element_type.value} element name {raw!r} was not found",
        )
    if len(matches) > 1:
        raise OutputError(
            "ambiguous_element",
            f"{element_type.value} element name {raw!r} matched {len(matches)} elements",
        )
    return matches[0]


def _resolve_pollutant(metadata: OutputMetadata, attribute: PollutantAttribute) -> int:
    selector = attribute.selector
    if type(selector) is int:
        index = selector
    else:
        if not isinstance(selector, (str, bytes, OutputName)):
            raise TypeError("pollutant must be an exact str, bytes, or OutputName")
        raw = _element_bytes(selector, "pollutant")
        matches = [
            index
            for index, pollutant in enumerate(metadata.pollutants)
            if pollutant.name.raw == raw
        ]
        if not matches:
            raise OutputError(
                "pollutant_not_found",
                f"pollutant name {raw!r} was not found",
            )
        if len(matches) > 1:
            raise OutputError(
                "ambiguous_pollutant",
                f"pollutant name {raw!r} matched {len(matches)} pollutants",
            )
        index = matches[0]
    if index >= len(metadata.pollutants):
        raise OutputError(
            "pollutant_not_found",
            f"pollutant index {index} was not found",
        )
    return index


def _resolve_attribute(
    metadata: OutputMetadata,
    element_type: ResultElementType,
    attribute: object,
) -> int:
    if isinstance(attribute, PollutantAttribute):
        if element_type is ResultElementType.SYSTEM:
            raise ValueError("system selections do not support pollutants")
        pollutant_index = _resolve_pollutant(metadata, attribute)
        matches = [
            (code, entry)
            for entry, code in _schema_entries(metadata, element_type)
            if isinstance(entry, PollutantAttribute)
            and type(entry.selector) is int
            and entry.selector == pollutant_index
        ]
        if not matches:
            raise OutputError(
                "attribute_not_found",
                f"pollutant {pollutant_index} is absent from {element_type.value} schema",
            )
        if len(matches) > 1:
            raise OutputError(
                "ambiguous_attribute",
                f"pollutant {pollutant_index} occurs {len(matches)} times in "
                f"{element_type.value} schema",
            )
        code = matches[0][0]
    else:
        code = _entry_code(attribute)
        code_matches = [
            stored_code
            for _, stored_code in _schema_entries(metadata, element_type)
            if stored_code == code
        ]
        if not code_matches:
            raise OutputError(
                "attribute_not_found",
                f"result code {code} is absent from {element_type.value} schema",
            )
        if len(code_matches) > 1:
            raise OutputError(
                "ambiguous_attribute",
                f"result code {code} occurs {len(code_matches)} times in "
                f"{element_type.value} schema",
            )
    if code is None:
        raise OutputError(
            "attribute_not_found",
            f"pollutant selector is absent from {element_type.value} schema",
        )
    return _signed_code(code, "result code")


@dataclass(frozen=True, slots=True)
class OutputValueSeries:
    """Immutable values aligned to one canonical selection."""

    selection: SeriesSelection
    values: tuple[float, ...]

    def __post_init__(self) -> None:
        """Freeze the value sequence and validate its selection."""

        if not isinstance(self.selection, SeriesSelection):
            raise TypeError("selection must be a SeriesSelection")
        object.__setattr__(self, "values", tuple(self.values))

    def __repr__(self) -> str:
        """Summarize the selection and period count without rendering values."""

        return f"{type(self).__name__}(selection={self.selection!r}, periods={len(self.values)})"


@dataclass(frozen=True, slots=True)
class OutputTimeSeries:
    """Immutable dates and values aligned to one canonical selection."""

    selection: SeriesSelection
    times: tuple[datetime, ...]
    values: tuple[float, ...]

    def __post_init__(self) -> None:
        """Freeze dimensions and enforce series alignment."""

        if not isinstance(self.selection, SeriesSelection):
            raise TypeError("selection must be a SeriesSelection")
        times = tuple(self.times)
        for value in times:
            if not isinstance(value, datetime):
                raise TypeError("times must contain datetime values")
            if value.tzinfo is not None and value.utcoffset() is not None:
                raise ValueError("times must contain naive datetimes")
        values = tuple(self.values)
        if len(values) != len(times):
            raise ValueError("series values must align with times")
        object.__setattr__(self, "times", times)
        object.__setattr__(self, "values", values)

    def __repr__(self) -> str:
        """Summarize the selection and period count without rendering data."""

        return f"{type(self).__name__}(selection={self.selection!r}, periods={len(self.times)})"


@dataclass(frozen=True, slots=True)
class BulkSeriesResult:
    """Immutable column-oriented result for an ordered bulk request."""

    times: tuple[datetime, ...]
    series: tuple[OutputValueSeries, ...]

    def __post_init__(self) -> None:
        """Freeze dimensions and enforce shared-axis alignment."""

        times = tuple(self.times)
        for value in times:
            if not isinstance(value, datetime):
                raise TypeError("times must contain datetime values")
            if value.tzinfo is not None and value.utcoffset() is not None:
                raise ValueError("times must contain naive datetimes")
        series = tuple(self.series)
        for item in series:
            if not isinstance(item, OutputValueSeries):
                raise TypeError("series must contain OutputValueSeries values")
            if len(item.values) != len(times):
                raise ValueError("series values must align with times")
        object.__setattr__(self, "times", times)
        object.__setattr__(self, "series", series)

    def __repr__(self) -> str:
        """Summarize both result dimensions without rendering data."""

        return f"{type(self).__name__}(periods={len(self.times)}, series={len(self.series)})"

    def value(self, period_offset: int, selection_offset: int) -> float:
        """Return one value using offsets local to this result.

        Parameters
        ----------
        period_offset : int
            Zero-based position in :attr:`times`.
        selection_offset : int
            Zero-based position in :attr:`series`.

        Returns
        -------
        float
            Selected stored result value promoted exactly from ``f32``.

        Raises
        ------
        IndexError
            If either offset is not an integer or is outside its result dimension.

        Examples
        --------
        >>> selection = SeriesSelection("system", None, "rainfall")
        >>> values = OutputValueSeries(selection, (1.0, 2.0))
        >>> result = BulkSeriesResult(
        ...     (datetime(2020, 1, 1), datetime(2020, 1, 2)),
        ...     (values,),
        ... )
        >>> result.value(1, 0)
        2.0
        """

        if type(period_offset) is not int or period_offset < 0 or period_offset >= len(self.times):
            raise IndexError("period offset is outside this result")
        if (
            type(selection_offset) is not int
            or selection_offset < 0
            or selection_offset >= len(self.series)
        ):
            raise IndexError("selection offset is outside this result")
        return self.series[selection_offset].values[period_offset]


def _canonical_attribute(
    metadata: OutputMetadata,
    element_type: ResultElementType,
    code: int,
) -> (
    SubcatchmentResultAttribute
    | NodeResultAttribute
    | LinkResultAttribute
    | SystemResultAttribute
    | PollutantAttribute
    | ResultAttributeCode
):
    for attribute, stored_code in _schema_entries(metadata, element_type):
        if stored_code == code:
            return cast(
                SubcatchmentResultAttribute
                | NodeResultAttribute
                | LinkResultAttribute
                | SystemResultAttribute
                | PollutantAttribute
                | ResultAttributeCode,
                attribute,
            )
    return ResultAttributeCode(code)


def _canonical_selection(metadata: OutputMetadata, raw: tuple[str, int, int]) -> SeriesSelection:
    element_type = _enum_value(raw[0], ResultElementType, "element_type")
    attribute = _canonical_attribute(metadata, element_type, _signed_code(raw[2], "result code"))
    if element_type is ResultElementType.SYSTEM:
        return SeriesSelection(element_type, None, attribute)
    return SeriesSelection(element_type, _nonnegative_index(raw[1], "element"), attribute)


def _bulk_result(payload: Mapping[str, object], metadata: OutputMetadata) -> BulkSeriesResult:
    times = tuple(_nominal_datetime(serial) for serial in cast(list[float], payload["times"]))
    series = tuple(
        OutputValueSeries(
            _canonical_selection(metadata, (family, index, code)),
            tuple(values),
        )
        for family, index, code, values in cast(
            list[tuple[str, int, int, list[float]]], payload["series"]
        )
    )
    return BulkSeriesResult(times, series)


def _validate_period_bound(bound: object, name: str, period_count: int) -> None:
    if bound is None:
        return
    if type(bound) is int:
        if bound < 0:
            raise ValueError(f"{name} must be nonnegative")
        if bound > period_count:
            raise ValueError(f"{name} must not exceed period_count")
        return
    if isinstance(bound, datetime):
        if bound.tzinfo is not None and bound.utcoffset() is not None:
            raise ValueError(f"{name} must be a naive datetime")
        return
    raise TypeError(f"{name} must be an int, naive datetime, or None")


def _bound_index(
    bound: int | datetime | None,
    *,
    default: int,
    axis: tuple[datetime, ...] | None,
) -> int:
    if bound is None:
        return default
    if type(bound) is int:
        return bound
    if axis is None:
        raise RuntimeError("datetime bounds require a nominal time axis")
    return bisect_left(axis, bound)


@final
class OutputReader:
    """Own and query one local SWMM binary output file.

    A valid final trailer is detected automatically. Without one, the reader
    exposes every complete result record visible when the file is opened.

    Parameters
    ----------
    source_path : str or os.PathLike[str]
        SWMM binary output path.

    Raises
    ------
    OutputError
        If the file cannot be opened, has incomplete metadata, has an unsupported
        schema, or fails structural validation.

    Examples
    --------
    >>> reader = OutputReader("model.out")
    >>> reader.metadata.report_timing.period_count > 0
    True
    """

    __slots__ = ("_metadata", "_native", "_source_path", "_times", "_times_lock")

    def __init__(self, source_path: _PathInput) -> None:
        """Open the file and eagerly parse immutable output metadata.

        Parameters
        ----------
        source_path : str or os.PathLike[str]
            SWMM binary output path.

        Raises
        ------
        OutputError
            If opening, reading, allocation, or file validation fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> reader.source_path.name
        'model.out'
        """

        path = Path(source_path)
        try:
            native = _NativeOutputReader(path)
            payload = native.metadata()
        except _NativeOutputError as error:
            raise _public_error(error) from error
        self._native = native
        self._source_path = path
        self._metadata = _metadata(payload, path)
        self._times: tuple[datetime, ...] | None = None
        self._times_lock = Lock()

    @property
    def source_path(self) -> Path:
        """Return the path supplied when this reader was constructed.

        Returns
        -------
        pathlib.Path
            Informational local source path.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> reader.source_path.name
        'model.out'
        """

        return self._source_path

    @property
    def is_finalized(self) -> bool:
        """Return whether the file contained a valid final trailer when opened."""

        return self._metadata.run_status.is_finalized

    @property
    def metadata(self) -> OutputMetadata:
        """Return eagerly parsed immutable output metadata.

        Returns
        -------
        OutputMetadata
            Physical identities, schemas, units, status, and report timing.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> reader.metadata.nodes[0].index
        0
        """

        return self._metadata

    @property
    def times(self) -> tuple[datetime, ...]:
        """Return the cached Nominal Report Date axis.

        Returns
        -------
        tuple of datetime
            Naive datetimes rounded to whole seconds using round-half-to-even.

        Raises
        ------
        OutputError
            If a nominal report date is outside Python's datetime range.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> len(reader.times) == reader.metadata.report_timing.period_count
        True

        Notes
        -----
        The tuple is built on first access and then reused by identity. Stored Report
        Dates remain separately available through :meth:`read_stored_dates`.
        """

        cached = self._times
        if cached is not None:
            return cached
        with self._times_lock:
            cached = self._times
            if cached is None:
                timing = self._metadata.report_timing
                cached = tuple(
                    cast(datetime, timing.nominal_date(period))
                    for period in range(timing.period_count)
                )
                self._times = cached
        assert cached is not None
        return cached

    def _resolve_bounds(
        self, start: int | datetime | None, end: int | datetime | None
    ) -> tuple[int, int]:
        period_count = self._metadata.report_timing.period_count
        _validate_period_bound(start, "start", period_count)
        _validate_period_bound(end, "end", period_count)
        if isinstance(start, datetime) and isinstance(end, datetime) and start > end:
            raise OutputError(
                "invalid_period_range",
                "start bound must not be later than end bound",
            )
        axis = self.times if isinstance(start, datetime) or isinstance(end, datetime) else None
        resolved_start = _bound_index(start, default=0, axis=axis)
        resolved_end = _bound_index(end, default=period_count, axis=axis)
        if resolved_start > resolved_end:
            raise OutputError(
                "invalid_period_range",
                "resolved period range is inverted",
            )
        return resolved_start, resolved_end

    def _native_selections(
        self,
        selections: Sequence[SeriesSelection],
    ) -> list[tuple[str, int, int]]:
        if not isinstance(selections, Sequence):
            raise TypeError("selections must be a sequence")
        result: list[tuple[str, int, int]] = []
        for selection in selections:
            if not isinstance(selection, SeriesSelection):
                raise TypeError("selections must contain SeriesSelection values")
            result.append(selection._native(self._metadata))
        return result

    def read_bulk_series(
        self,
        selections: Sequence[SeriesSelection],
        start: int | datetime | None = None,
        end: int | datetime | None = None,
        *,
        low_memory: bool = False,
    ) -> BulkSeriesResult:
        """Read ordered result series using one of two physical I/O strategies.

        Parameters
        ----------
        selections : Sequence of SeriesSelection
            Ordered selections. Duplicate selections remain duplicate result columns.
        start : int, datetime, or None, default=None
            Inclusive local period offset or naive Nominal Report Date lower bound.
        end : int, datetime, or None, default=None
            Exclusive local period offset or naive Nominal Report Date lower bound.
        low_memory : bool, default=False
            If ``False``, read one complete result payload per selected period. If
            ``True``, read only selected adjacent cell runs with smaller scratch space.

        Returns
        -------
        BulkSeriesResult
            Shared nominal time axis and ordered immutable value series.

        Raises
        ------
        TypeError
            If selections, bounds, or ``low_memory`` have invalid types.
        ValueError
            If a bound is negative, out of range, or timezone-aware.
        OutputError
            If a selection is invalid, bounds are inverted, or native I/O fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> selections = (
        ...     SeriesSelection("node", "J1", "invert_depth"),
        ...     SeriesSelection("link", "C1", "flow_rate"),
        ... )
        >>> result = reader.read_bulk_series(selections, start=0, end=12)
        >>> len(result.series)
        2
        >>> narrow = reader.read_bulk_series(
        ...     selections,
        ...     start=0,
        ...     end=12,
        ...     low_memory=True,
        ... )
        >>> narrow == result
        True

        Notes
        -----
        Both strategies return bit-identical values. ``low_memory=False`` usually
        favors wide requests by reducing physical read calls. ``low_memory=True``
        limits scratch memory and avoids reading unselected payload cells.
        """

        if type(low_memory) is not bool:
            raise TypeError("low_memory must be bool")
        if low_memory:
            return self._read_bulk_series(selections, start, end)
        return self._read_bulk_series_by_period(selections, start, end)

    def _read_bulk_series(
        self,
        selections: Sequence[SeriesSelection],
        start: int | datetime | None,
        end: int | datetime | None,
    ) -> BulkSeriesResult:
        """Read requested cells through selective, low-memory I/O."""

        first, last = self._resolve_bounds(start, end)
        payload = cast(
            Mapping[str, object],
            self._call(
                self._native.read_bulk_series,
                self._native_selections(selections),
                first,
                last,
            ),
        )
        return _bulk_result(payload, self._metadata)

    def _single_series(
        self,
        selection: SeriesSelection,
        start: int | datetime | None,
        end: int | datetime | None,
        low_memory: bool,
    ) -> OutputTimeSeries:
        """Read one selection through the requested bulk memory strategy."""

        result = self.read_bulk_series(
            (selection,),
            start,
            end,
            low_memory=low_memory,
        )
        series = result.series[0]
        return OutputTimeSeries(series.selection, result.times, series.values)

    def subcatchment_series(
        self,
        element: _ElementSelector,
        attribute: (SubcatchmentResultAttribute | PollutantAttribute | ResultAttributeCode | str),
        start: int | datetime | None = None,
        end: int | datetime | None = None,
        *,
        low_memory: bool = False,
    ) -> OutputTimeSeries:
        """Read one subcatchment result series.

        Parameters
        ----------
        element : int, str, bytes, or OutputName
            Zero-based subcatchment position or exact stored name.
        attribute : SubcatchmentResultAttribute, PollutantAttribute, ResultAttributeCode, or str
            Family-specific typed attribute or exact canonical string.
        start : int, datetime, or None, default=None
            Inclusive period offset or naive nominal-date lower bound.
        end : int, datetime, or None, default=None
            Exclusive period offset or naive nominal-date lower bound.
        low_memory : bool, default=False
            Selective-read strategy when true; by-period strategy when false.

        Returns
        -------
        OutputTimeSeries
            Canonical selection, nominal times, and aligned values.

        Raises
        ------
        TypeError
            If a selector, bound, or ``low_memory`` has an invalid type.
        ValueError
            If a selector or bound has an invalid value.
        OutputError
            If the element or attribute is absent or ambiguous, the range is
            inverted, or native I/O fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> runoff = reader.subcatchment_series("S1", "runoff_flow")
        >>> len(runoff.times) == len(runoff.values)
        True
        """

        return self._single_series(
            SeriesSelection(ResultElementType.SUBCATCHMENT, element, attribute),
            start,
            end,
            low_memory,
        )

    def node_series(
        self,
        element: _ElementSelector,
        attribute: NodeResultAttribute | PollutantAttribute | ResultAttributeCode | str,
        start: int | datetime | None = None,
        end: int | datetime | None = None,
        *,
        low_memory: bool = False,
    ) -> OutputTimeSeries:
        """Read one node result series.

        Parameters
        ----------
        element : int, str, bytes, or OutputName
            Zero-based node position or exact stored name.
        attribute : NodeResultAttribute, PollutantAttribute, ResultAttributeCode, or str
            Family-specific typed attribute or exact canonical string.
        start : int, datetime, or None, default=None
            Inclusive period offset or naive nominal-date lower bound.
        end : int, datetime, or None, default=None
            Exclusive period offset or naive nominal-date lower bound.
        low_memory : bool, default=False
            Selective-read strategy when true; by-period strategy when false.

        Returns
        -------
        OutputTimeSeries
            Canonical selection, nominal times, and aligned values.

        Raises
        ------
        TypeError
            If a selector, bound, or ``low_memory`` has an invalid type.
        ValueError
            If a selector or bound has an invalid value.
        OutputError
            If the element or attribute is absent or ambiguous, the range is
            inverted, or native I/O fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> depth = reader.node_series("J1", "invert_depth", start=0, end=12)
        >>> depth.selection.element_type is ResultElementType.NODE
        True
        """

        return self._single_series(
            SeriesSelection(ResultElementType.NODE, element, attribute),
            start,
            end,
            low_memory,
        )

    def link_series(
        self,
        element: _ElementSelector,
        attribute: LinkResultAttribute | PollutantAttribute | ResultAttributeCode | str,
        start: int | datetime | None = None,
        end: int | datetime | None = None,
        *,
        low_memory: bool = False,
    ) -> OutputTimeSeries:
        """Read one link result series.

        Parameters
        ----------
        element : int, str, bytes, or OutputName
            Zero-based link position or exact stored name.
        attribute : LinkResultAttribute, PollutantAttribute, ResultAttributeCode, or str
            Family-specific typed attribute or exact canonical string.
        start : int, datetime, or None, default=None
            Inclusive period offset or naive nominal-date lower bound.
        end : int, datetime, or None, default=None
            Exclusive period offset or naive nominal-date lower bound.
        low_memory : bool, default=False
            Selective-read strategy when true; by-period strategy when false.

        Returns
        -------
        OutputTimeSeries
            Canonical selection, nominal times, and aligned values.

        Raises
        ------
        TypeError
            If a selector, bound, or ``low_memory`` has an invalid type.
        ValueError
            If a selector or bound has an invalid value.
        OutputError
            If the element or attribute is absent or ambiguous, the range is
            inverted, or native I/O fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> flow = reader.link_series("C1", "flow_rate", low_memory=True)
        >>> len(flow.times) == len(flow.values)
        True
        """

        return self._single_series(
            SeriesSelection(ResultElementType.LINK, element, attribute),
            start,
            end,
            low_memory,
        )

    def system_series(
        self,
        attribute: SystemResultAttribute | ResultAttributeCode | str,
        start: int | datetime | None = None,
        end: int | datetime | None = None,
        *,
        low_memory: bool = False,
    ) -> OutputTimeSeries:
        """Read one system result series.

        Parameters
        ----------
        attribute : SystemResultAttribute, ResultAttributeCode, or str
            Typed system attribute or exact canonical string.
        start : int, datetime, or None, default=None
            Inclusive period offset or naive nominal-date lower bound.
        end : int, datetime, or None, default=None
            Exclusive period offset or naive nominal-date lower bound.
        low_memory : bool, default=False
            Selective-read strategy when true; by-period strategy when false.

        Returns
        -------
        OutputTimeSeries
            Canonical system selection, nominal times, and aligned values.

        Raises
        ------
        TypeError
            If an attribute, bound, or ``low_memory`` has an invalid type.
        ValueError
            If an attribute or bound has an invalid value.
        OutputError
            If the attribute is absent or ambiguous, the range is inverted, or
            native I/O fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> rainfall = reader.system_series("rainfall")
        >>> rainfall.selection.element is None
        True
        """

        return self._single_series(
            SeriesSelection(ResultElementType.SYSTEM, None, attribute),
            start,
            end,
            low_memory,
        )

    def _read_bulk_series_by_period(
        self,
        selections: Sequence[SeriesSelection],
        start: int | datetime | None,
        end: int | datetime | None,
    ) -> BulkSeriesResult:
        """Read one complete result payload for each selected period."""

        first, last = self._resolve_bounds(start, end)
        payload = cast(
            Mapping[str, object],
            self._call(
                self._native.read_bulk_series_by_period,
                self._native_selections(selections),
                first,
                last,
            ),
        )
        return _bulk_result(payload, self._metadata)

    def read_stored_dates(
        self,
        start: int | datetime | None = None,
        end: int | datetime | None = None,
    ) -> list[float]:
        """Read exact Stored Report Date serial values.

        Parameters
        ----------
        start : int, datetime, or None, default=None
            Inclusive period offset or naive Nominal Report Date lower bound.
        end : int, datetime, or None, default=None
            Exclusive period offset or naive Nominal Report Date lower bound.

        Returns
        -------
        list of float
            Exact finite SWMM serial-day values in resolved period order.

        Raises
        ------
        TypeError
            If a bound has an invalid type.
        ValueError
            If a bound is negative, out of range, or timezone-aware.
        OutputError
            If bounds are inverted, a stored date is non-finite, or native I/O
            fails.

        Examples
        --------
        >>> reader = OutputReader("model.out")
        >>> stored = reader.read_stored_dates(start=0, end=2)
        >>> len(stored)
        2

        Notes
        -----
        Stored Report Dates are file facts and never select, round, or replace the
        nominal datetime axis. Integer-only reads do not populate :attr:`times`.
        """

        first, last = self._resolve_bounds(start, end)
        return self._call(self._native.read_stored_dates, first, last)

    @staticmethod
    def _call(operation: Callable[..., _T], *args: object) -> _T:
        """Translate one package-private native output exception."""

        try:
            return operation(*args)
        except _NativeOutputError as error:
            raise _public_error(error) from error


def _categorical(
    value: object,
    known: dict[int, _T],
    name: str,
) -> _T | UnknownCode:
    """Convert one exact signed category code without discarding unknowns."""

    code = _signed_code(value, name)
    return known.get(code, UnknownCode(code))


def _unit_system(value: object) -> UnitSystem | None:
    """Convert the private native unit-system spelling."""

    if value is None:
        return None
    if isinstance(value, UnitSystem):
        return value
    if type(value) is str and value in {"US", "SI"}:
        return UnitSystem(value.lower())
    raise ValueError(f"unknown unit_system {value!r}")


def _schema_code(value: object, name: str) -> tuple[int, int | None]:
    if type(value) is int:
        return _signed_code(value, name), None
    if type(value) is tuple and len(value) == 2:
        code_value, marker = value
    elif type(value) is list and len(value) == 2:
        code_value, marker = value
    else:
        raise TypeError(f"{name} must be a signed code and pollutant marker")
    code = _signed_code(code_value, name)
    if marker is None:
        return code, None
    return code, _nonnegative_index(marker, f"{name} pollutant index")


def _schema_entry(
    value: object,
    family: type[_T],
    known: Mapping[int, _T],
    pollutants: tuple[PollutantMetadata, ...],
    name: str,
) -> tuple[_T | PollutantAttribute | ResultAttributeCode, int]:
    code, pollutant_index = _schema_code(value, name)
    if pollutant_index is not None:
        if pollutant_index >= len(pollutants):
            raise ValueError(f"{name} references an unknown pollutant")
        attribute = PollutantAttribute(pollutant_index)
        object.__setattr__(attribute, "_code", code)
        return attribute, code
    return known.get(code, ResultAttributeCode(code)), code


@overload
def _parse_schema(
    payload: Mapping[str, object],
    key: str,
    family: type[_T],
    known: Mapping[int, _T],
    pollutants: tuple[()],
) -> tuple[tuple[_T | ResultAttributeCode, ...], tuple[int, ...]]: ...


@overload
def _parse_schema(
    payload: Mapping[str, object],
    key: str,
    family: type[_T],
    known: Mapping[int, _T],
    pollutants: tuple[PollutantMetadata, ...],
) -> tuple[tuple[_T | PollutantAttribute | ResultAttributeCode, ...], tuple[int, ...]]: ...
def _parse_schema(
    payload: Mapping[str, object],
    key: str,
    family: type[_T],
    known: Mapping[int, _T],
    pollutants: tuple[PollutantMetadata, ...],
) -> tuple[tuple[_T | PollutantAttribute | ResultAttributeCode, ...], tuple[int, ...]]:
    entries = tuple(
        _schema_entry(value, family, known, pollutants, f"{key} code")
        for value in cast(list[object], payload[key])
    )
    return tuple(entry for entry, _ in entries), tuple(code for _, code in entries)


def _metadata(payload: Mapping[str, object], source_path: Path) -> OutputMetadata:
    """Convert one trusted private metadata payload to immutable public records."""

    timing = ReportTiming(
        report_schedule_origin=cast(float, payload["report_schedule_origin"]),
        report_step_seconds=cast(int, payload["report_step_seconds"]),
        period_count=cast(int, payload["period_count"]),
    )
    run_status = cast(int | None, payload["run_status"])
    subcatchments = tuple(
        SubcatchmentMetadata(index, OutputName(bytes(name)), area)
        for index, name, area in cast(
            list[tuple[int, Sequence[int], float]], payload["subcatchments"]
        )
    )
    nodes = tuple(
        NodeMetadata(
            index,
            OutputName(bytes(name)),
            _categorical(
                kind,
                {
                    0: NodeKind.JUNCTION,
                    1: NodeKind.OUTFALL,
                    2: NodeKind.STORAGE,
                    3: NodeKind.DIVIDER,
                },
                "node kind",
            ),
            invert,
            maximum_depth,
        )
        for index, name, kind, invert, maximum_depth in cast(
            list[tuple[int, Sequence[int], int, float, float]], payload["nodes"]
        )
    )
    links = tuple(
        LinkMetadata(
            index,
            OutputName(bytes(name)),
            _categorical(
                kind,
                {
                    0: LinkKind.CONDUIT,
                    1: LinkKind.PUMP,
                    2: LinkKind.ORIFICE,
                    3: LinkKind.WEIR,
                    4: LinkKind.OUTLET,
                },
                "link kind",
            ),
            inlet,
            outlet,
            maximum_depth,
            length,
        )
        for index, name, kind, inlet, outlet, maximum_depth, length in cast(
            list[tuple[int, Sequence[int], int, float, float, float, float]],
            payload["links"],
        )
    )
    pollutants = tuple(
        PollutantMetadata(
            index,
            OutputName(bytes(name)),
            _categorical(
                units,
                {
                    0: ConcentrationUnits.MILLIGRAMS_PER_LITER,
                    1: ConcentrationUnits.MICROGRAMS_PER_LITER,
                    2: ConcentrationUnits.COUNTS_PER_LITER,
                },
                "concentration units",
            ),
        )
        for index, name, units in cast(list[tuple[int, Sequence[int], int]], payload["pollutants"])
    )
    subcatchment_schema, subcatchment_codes = _parse_schema(
        payload,
        "subcatchment_schema",
        SubcatchmentResultAttribute,
        {
            0: SubcatchmentResultAttribute.RAINFALL,
            1: SubcatchmentResultAttribute.SNOW_DEPTH,
            2: SubcatchmentResultAttribute.EVAP_LOSS,
            3: SubcatchmentResultAttribute.INFIL_LOSS,
            4: SubcatchmentResultAttribute.RUNOFF_RATE,
            5: SubcatchmentResultAttribute.GW_OUTFLOW_RATE,
            6: SubcatchmentResultAttribute.GW_TABLE_ELEV,
            7: SubcatchmentResultAttribute.SOIL_MOISTURE,
        },
        pollutants,
    )
    node_schema, node_codes = _parse_schema(
        payload,
        "node_schema",
        NodeResultAttribute,
        {
            0: NodeResultAttribute.INVERT_DEPTH,
            1: NodeResultAttribute.HYDRAULIC_HEAD,
            2: NodeResultAttribute.PONDED_VOLUME,
            3: NodeResultAttribute.LATERAL_INFLOW,
            4: NodeResultAttribute.TOTAL_INFLOW,
            5: NodeResultAttribute.FLOODING_LOSSES,
        },
        pollutants,
    )
    link_schema, link_codes = _parse_schema(
        payload,
        "link_schema",
        LinkResultAttribute,
        {
            0: LinkResultAttribute.FLOW_RATE,
            1: LinkResultAttribute.FLOW_DEPTH,
            2: LinkResultAttribute.FLOW_VELOCITY,
            3: LinkResultAttribute.FLOW_VOLUME,
            4: LinkResultAttribute.CAPACITY,
        },
        pollutants,
    )
    system_schema, system_codes = _parse_schema(
        payload,
        "system_schema",
        SystemResultAttribute,
        {
            0: SystemResultAttribute.AIR_TEMP,
            1: SystemResultAttribute.RAINFALL,
            2: SystemResultAttribute.SNOW_DEPTH,
            3: SystemResultAttribute.EVAP_INFIL_LOSS,
            4: SystemResultAttribute.RUNOFF_FLOW,
            5: SystemResultAttribute.DRY_WEATHER_INFLOW,
            6: SystemResultAttribute.GW_INFLOW,
            7: SystemResultAttribute.RDII_INFLOW,
            8: SystemResultAttribute.DIRECT_INFLOW,
            9: SystemResultAttribute.TOTAL_LATERAL_INFLOW,
            10: SystemResultAttribute.FLOOD_LOSSES,
            11: SystemResultAttribute.OUTFALL_FLOWS,
            12: SystemResultAttribute.VOLUME_STORED,
            13: SystemResultAttribute.EVAP_RATE,
            14: SystemResultAttribute.PTNL_EVAP_RATE,
        },
        (),
    )
    result_schema = ResultSchema(
        subcatchment=subcatchment_schema,
        node=node_schema,
        link=link_schema,
        system=system_schema,
    )
    object.__setattr__(
        result_schema,
        "_codes",
        (subcatchment_codes, node_codes, link_codes, system_codes),
    )
    return OutputMetadata(
        solver_release=_signed_code(payload["solver_release"], "solver_release"),
        run_status=RunStatus(
            None if run_status is None else _signed_code(run_status, "run_status")
        ),
        flow_units=_categorical(
            payload["flow_units"],
            {
                0: FlowUnits.CFS,
                1: FlowUnits.GPM,
                2: FlowUnits.MGD,
                3: FlowUnits.CMS,
                4: FlowUnits.LPS,
                5: FlowUnits.MLD,
            },
            "flow units",
        ),
        unit_system=_unit_system(payload["unit_system"]),
        report_timing=timing,
        subcatchments=subcatchments,
        nodes=nodes,
        links=links,
        pollutants=pollutants,
        result_schema=result_schema,
        source_path=source_path,
    )


def _public_error(error: _NativeOutputError) -> OutputError:
    """Convert one package-private native exception to `OutputError`."""

    category, message = error.args
    return OutputError(category, message)
