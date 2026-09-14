"""Regression coverage for vendored dataclass pretty representations."""

from __future__ import annotations

from dataclasses import dataclass, is_dataclass
from importlib import import_module
from types import ModuleType

import swmmrs.exceptions as exceptions
import swmmrs.output as output
import swmmrs.snapshots as snapshots
from swmmrs.exceptions import (
    ConfigurationDiagnostic,
    ConfigurationError,
    ConfigurationObjectIdentity,
)
from swmmrs.objects import _definitions, _links, _nodes, _subcatchments
from swmmrs.output import OutputName, ReportTiming, RunStatus

_prettier = import_module("swmmrs._prettier")

_DATACLASS_MODULES: tuple[ModuleType, ...] = (
    exceptions,
    output,
    snapshots,
    _definitions,
    _links,
    _nodes,
    _subcatchments,
)

_CUSTOM_REPR_CLASSES = {
    "ReportTiming",
    "OutputName",
    "SubcatchmentMetadata",
    "NodeMetadata",
    "LinkMetadata",
    "PollutantMetadata",
    "ResultSchema",
    "OutputMetadata",
    "OutputValueSeries",
    "OutputTimeSeries",
    "BulkSeriesResult",
}


def test_pretty_dataclasses_render_nested_fields_and_str_matches_repr() -> None:
    diagnostic = ConfigurationDiagnostic(
        object=ConfigurationObjectIdentity("node", "J1", 0),
        property_path="inlet_offset",
        rule_code="invalid_offset",
        message="must be nonnegative",
        conflicting_object=None,
    )

    expected = """ConfigurationDiagnostic(
    object=ConfigurationObjectIdentity(
        object_type='node',
        id='J1',
        index=0,
    ),
    property_path='inlet_offset',
    rule_code='invalid_offset',
    message='must be nonnegative',
    conflicting_object=None,
)"""
    assert repr(diagnostic) == expected
    assert str(diagnostic) == expected
    assert _prettier.pformat(diagnostic) == expected


def test_configuration_error_string_surfaces_ordered_diagnostic_details() -> None:
    diagnostics = (
        ConfigurationDiagnostic(
            object=ConfigurationObjectIdentity("aquifer", "A1", 0),
            property_path="parameters",
            rule_code="aquifer.parameters.range",
            message="aquifer parameters do not satisfy groundwater constraints",
            conflicting_object=None,
        ),
        ConfigurationDiagnostic(
            object=ConfigurationObjectIdentity("subcatchment", "S0001", 2),
            property_path="groundwater",
            rule_code="subcatchment.groundwater.elevation",
            message="surface elevation must not be below water-table elevation",
            conflicting_object=ConfigurationObjectIdentity("aquifer", "A1", 0),
        ),
    )
    error = ConfigurationError(diagnostics)

    expected_args = (
        "start: post-open preparation rejected 2 configuration violation(s)",
    )
    assert error.args == expected_args
    assert str(error) == (
        "start: post-open preparation rejected 2 configuration violation(s):\n"
        "- aquifer 'A1' (index 0), property 'parameters', rule "
        "'aquifer.parameters.range': aquifer parameters do not satisfy groundwater constraints\n"
        "- subcatchment 'S0001' (index 2), property 'groundwater', rule "
        "'subcatchment.groundwater.elevation': surface elevation must not be below water-table "
        "elevation; conflicts with aquifer 'A1' (index 0)"
    )


def test_pretty_dataclass_creates_a_slotted_frozen_dataclass() -> None:
    @_prettier.pretty_dataclass(frozen=True, slots=True)
    class Value:
        amount: int

    value = Value(3)
    assert is_dataclass(value)
    assert repr(value) == "Value(\n    amount=3,\n)"


def test_formatter_stops_at_recursive_dataclass_references() -> None:
    @dataclass
    class RecursiveValue:
        child: object | None = None

    value = RecursiveValue()
    value.child = value

    assert (
        _prettier.pformat(value) == "RecursiveValue(\n    child=<Recursion on RecursiveValue>,\n)"
    )


def test_every_non_custom_dataclass_uses_the_vendored_pretty_repr() -> None:
    for module in _DATACLASS_MODULES:
        for value in vars(module).values():
            if (
                not isinstance(value, type)
                or value.__module__ != module.__name__
                or not is_dataclass(value)
            ):
                continue
            if value.__name__ in _CUSTOM_REPR_CLASSES:
                assert value.__repr__.__module__ == module.__name__
                continue
            assert value.__repr__.__module__ == "swmmrs._prettier"
            assert value.__str__ is value.__repr__


def test_custom_dataclass_representations_remain_compact() -> None:
    assert repr(OutputName(b"J1")) == "OutputName('J1')"
    assert repr(ReportTiming(2.0, 300, 288)) == ("ReportTiming(origin=2.0, step=300, periods=288)")
    assert repr(RunStatus(0)) == "RunStatus(\n    code=0,\n)"
