from __future__ import annotations

import struct
import threading
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

from swmmrs.output import (
    BulkSeriesResult,
    ConcentrationUnits,
    FlowUnits,
    LinkKind,
    LinkResultAttribute,
    NodeKind,
    NodeResultAttribute,
    OutputError,
    OutputName,
    OutputReader,
    OutputTimeSeries,
    OutputValueSeries,
    PollutantAttribute,
    ReportTiming,
    ResultAttributeCode,
    ResultElementType,
    SeriesSelection,
    SubcatchmentResultAttribute,
    SystemResultAttribute,
    UnitSystem,
    UnknownCode,
)

_FIXTURE = (
    Path(__file__).parents[2] / "crates" / "output" / "tests" / "fixtures" / "legacy" / "Model.out"
)
_EXPANDED_FIXTURE = (
    Path(__file__).parents[2]
    / "crates"
    / "output"
    / "tests"
    / "fixtures"
    / "expanded"
    / "minimal.out"
)
_INCOMPLETE_FIXTURE = (
    Path(__file__).parents[2]
    / "crates"
    / "output"
    / "tests"
    / "fixtures"
    / "incomplete"
    / "minimal-partial-period.out"
)


def _f32_bits(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", value))[0]


def _f64_bits(value: float) -> int:
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def _sample_selections(reader: OutputReader) -> list[SeriesSelection]:
    schema = reader.metadata.result_schema
    return [
        SeriesSelection("subcatchment", 0, schema.subcatchment[0]),
        SeriesSelection("node", 0, schema.node[0]),
        SeriesSelection("link", 0, schema.link[0]),
        SeriesSelection("system", None, schema.system[0]),
    ]


def _edge_reader(tmp_path: Path, *, duplicate_attribute: bool = False) -> OutputReader:
    nodes = [b"", b"Case", b"case", b"dup", b"dup", b"\xff"]
    pollutants = [b"invert_depth", b"dup", b"dup", b"\xff"]
    node_schema = [0, 6, 9, -7]
    if duplicate_attribute:
        node_schema.append(-7)
    schemas = ([0, 8, -7], node_schema, [0, 5, -7], [0, -7])

    identifiers = bytearray()
    for name in [b"sub", *nodes, b"link", *pollutants]:
        identifiers.extend(struct.pack("<i", len(name)))
        identifiers.extend(name)
    for _ in pollutants:
        identifiers.extend(struct.pack("<i", 0))

    properties = bytearray()

    def put_i32(value: int) -> None:
        properties.extend(struct.pack("<i", value))

    def put_u32(value: int) -> None:
        properties.extend(struct.pack("<I", value))

    def property_block(codes: list[int], values: list[int]) -> None:
        put_i32(len(codes))
        for code in codes:
            put_i32(code)
        for value in values:
            put_u32(value)

    property_block([1], [0])
    property_block([0, 2, 3], [0] * (len(nodes) * 3))
    property_block([0, 4, 4, 3, 5], [0] * 5)
    for codes in schemas:
        put_i32(len(codes))
        for code in codes:
            put_i32(code)
    properties.extend(struct.pack("<d", 0.0))
    properties.extend(struct.pack("<i", 300))

    period_count = 4
    payload_words = (
        len(schemas[0]) + len(nodes) * len(schemas[1]) + len(schemas[2]) + len(schemas[3])
    )
    results = bytearray()
    for period in range(period_count):
        results.extend(struct.pack("<d", (period + 1) * 300.0 / 86_400.0))
        for word in range(payload_words):
            results.extend(struct.pack("<f", float(period * payload_words + word + 1)))

    magic = 516_114_522
    input_start = 28 + len(identifiers)
    result_start = input_start + len(properties)
    output = bytearray(
        struct.pack(
            "<7i",
            magic,
            1,
            0,
            1,
            len(nodes),
            1,
            len(pollutants),
        )
    )
    output.extend(identifiers)
    output.extend(properties)
    output.extend(results)
    output.extend(struct.pack("<6i", 28, input_start, result_start, period_count, 0, magic))

    path = tmp_path / ("edge-duplicate.out" if duplicate_attribute else "edge.out")
    path.write_bytes(output)
    return OutputReader(path)


def test_output_reader_exposes_metadata_and_bit_exact_bulk_reads() -> None:
    reader = OutputReader(_FIXTURE)
    metadata = reader.metadata

    assert reader.source_path == _FIXTURE
    assert reader.is_finalized
    assert metadata.source_path == _FIXTURE
    assert metadata.solver_release == 51014
    assert metadata.run_status.code == 0
    assert metadata.run_status.is_success
    assert metadata.flow_units is FlowUnits.CFS
    assert metadata.unit_system is UnitSystem.US
    assert metadata.report_timing.period_count == 288
    assert metadata.report_timing.report_step_seconds == 300
    assert metadata.report_timing.nominal_date(0) is not None
    assert metadata.report_timing.nominal_date(288) is None
    assert all(isinstance(item.name, OutputName) for item in metadata.subcatchments)
    assert all(isinstance(item.name, OutputName) for item in metadata.nodes)
    assert all(isinstance(item.name, OutputName) for item in metadata.links)
    assert all(isinstance(item.name, OutputName) for item in metadata.pollutants)

    selections = _sample_selections(reader)
    default = reader.read_bulk_series(selections, 0, 1)
    selective = reader.read_bulk_series(selections, 0, 1, low_memory=True)
    by_period = reader.read_bulk_series(selections, 0, 1, low_memory=False)

    assert isinstance(selective, BulkSeriesResult)
    assert default == by_period
    assert selective == by_period
    assert not hasattr(reader, "read_bulk_series_by_period")
    assert len(selective.times) == 1
    assert len(selective.series) == len(selections)
    assert [item.selection for item in selective.series] == [
        selections[0],
        selections[1],
        selections[2],
        selections[3],
    ]
    assert [_f32_bits(item.values[0]) for item in selective.series] == [
        0x3CF5C28F,
        0x3B0C7643,
        0x3800E1DE,
        0x428C0000,
    ]
    assert [_f32_bits(selective.value(0, index)) for index in range(4)] == [
        0x3CF5C28F,
        0x3B0C7643,
        0x3800E1DE,
        0x428C0000,
    ]
    assert reader.times is reader.times
    assert reader.times == tuple(
        metadata.report_timing.nominal_date(period)
        for period in range(metadata.report_timing.period_count)
    )
    assert len(reader.read_stored_dates(1, 3)) == 2


def test_output_reader_automatically_recovers_incomplete_output() -> None:
    reader = OutputReader(_INCOMPLETE_FIXTURE)
    finalized = OutputReader(_EXPANDED_FIXTURE)
    assert not reader.is_finalized
    assert finalized.is_finalized
    assert reader.metadata.run_status.code is None
    assert not reader.metadata.run_status.is_finalized
    assert not reader.metadata.run_status.is_success
    assert reader.metadata.report_timing.period_count == 5
    assert reader.times == finalized.times[:5]

    selections = _sample_selections(reader)
    for low_memory in (False, True):
        assert reader.read_bulk_series(
            selections, low_memory=low_memory
        ) == finalized.read_bulk_series(selections, end=5, low_memory=low_memory)
    assert reader.read_stored_dates() == finalized.read_stored_dates(end=5)

    schema = reader.metadata.result_schema
    assert reader.subcatchment_series(0, schema.subcatchment[0]) == finalized.subcatchment_series(
        0, schema.subcatchment[0], end=5
    )
    assert reader.node_series(0, schema.node[0]) == finalized.node_series(0, schema.node[0], end=5)
    assert reader.link_series(0, schema.link[0]) == finalized.link_series(0, schema.link[0], end=5)
    assert reader.system_series(schema.system[0]) == finalized.system_series(
        schema.system[0], end=5
    )


def test_structured_bulk_preserves_order_duplicates_and_empty_dimensions() -> None:
    reader = OutputReader(_FIXTURE)
    selection = _sample_selections(reader)[0]
    requested = [selection, selection]

    selective = reader.read_bulk_series(requested, 0, 2, low_memory=True)
    by_period = reader.read_bulk_series(requested, 0, 2, low_memory=False)
    assert selective == by_period
    assert selective.times == reader.times[:2]
    assert [item.selection for item in selective.series] == requested
    assert selective.series[0].values == selective.series[1].values

    no_selections = reader.read_bulk_series([], 0, 2)
    assert no_selections.times == reader.times[:2]
    assert no_selections.series == ()

    no_periods = reader.read_bulk_series(requested, 2, 2)
    assert no_periods.times == ()
    assert [item.values for item in no_periods.series] == [(), ()]

    both_empty = reader.read_bulk_series([], 2, 2)
    assert both_empty.times == ()
    assert both_empty.series == ()


def test_output_reader_translates_structured_errors() -> None:
    reader = OutputReader(_FIXTURE)
    selection = SeriesSelection("system", None, reader.metadata.result_schema.system[0])

    with pytest.raises(OutputError) as raised:
        reader.read_bulk_series([selection], 3, 2)

    assert raised.value.category == "invalid_period_range"
    recovered = reader.read_bulk_series([selection], 0, 1)
    assert len(recovered.series[0].values) == 1
    with pytest.raises(TypeError, match="low_memory must be bool"):
        reader.read_bulk_series([selection], low_memory=1)  # type: ignore[arg-type]

    with pytest.raises(ValueError, match="do not carry"):
        SeriesSelection("system", 0, reader.metadata.result_schema.system[0])


def test_family_queries_match_one_selection_bulk_results() -> None:
    reader = OutputReader(_FIXTURE)
    cases = (
        (
            ResultElementType.SUBCATCHMENT,
            reader.metadata.subcatchments[0],
            SubcatchmentResultAttribute.RAINFALL.value,
            reader.subcatchment_series,
        ),
        (
            ResultElementType.NODE,
            reader.metadata.nodes[0],
            NodeResultAttribute.INVERT_DEPTH,
            reader.node_series,
        ),
        (
            ResultElementType.LINK,
            reader.metadata.links[0],
            LinkResultAttribute.FLOW_RATE.value,
            reader.link_series,
        ),
    )
    for element_type, metadata, attribute, family_query in cases:
        assert metadata.name.text is not None
        for element in (0, metadata.name.text, metadata.name.raw, metadata.name):
            selection = SeriesSelection(element_type, element, attribute)
            for low_memory in (False, True):
                expected = reader.read_bulk_series(
                    (selection,),
                    1,
                    3,
                    low_memory=low_memory,
                )
                actual = family_query(
                    element,
                    attribute,
                    1,
                    3,
                    low_memory=low_memory,
                )
                assert isinstance(actual, OutputTimeSeries)
                assert actual.selection == expected.series[0].selection
                assert actual.times == expected.times
                assert actual.values == expected.series[0].values

    system_selection = SeriesSelection(
        ResultElementType.SYSTEM,
        None,
        SystemResultAttribute.RAINFALL,
    )
    for low_memory in (False, True):
        expected_system = reader.read_bulk_series(
            (system_selection,),
            1,
            3,
            low_memory=low_memory,
        )
        actual_system = reader.system_series(
            "rainfall",
            1,
            3,
            low_memory=low_memory,
        )
        assert actual_system == OutputTimeSeries(
            expected_system.series[0].selection,
            expected_system.times,
            expected_system.series[0].values,
        )


def test_edge_names_pollutants_and_duplicate_codes(tmp_path: Path) -> None:
    reader = _edge_reader(tmp_path)
    assert reader.node_series("", "invert_depth").selection.element == 0
    assert reader.node_series("Case", "invert_depth").selection.element == 1
    assert reader.node_series(b"case", "invert_depth").selection.element == 2
    assert reader.node_series(OutputName(b"\xff"), "invert_depth").selection.element == 5
    with pytest.raises(OutputError) as duplicate_element:
        reader.node_series("dup", "invert_depth")
    assert duplicate_element.value.category == "ambiguous_element"
    with pytest.raises(OutputError) as case_mismatch:
        reader.node_series(b"CASE", "invert_depth")
    assert case_mismatch.value.category == "element_not_found"

    static = reader.node_series(1, "invert_depth")
    for selector in (0, "invert_depth", b"invert_depth", OutputName(b"\xff")):
        pollutant = reader.node_series(1, PollutantAttribute(selector))
        assert isinstance(pollutant.selection.attribute, PollutantAttribute)
    assert static.selection.attribute is NodeResultAttribute.INVERT_DEPTH
    with pytest.raises(OutputError) as duplicate_pollutant:
        reader.node_series(1, PollutantAttribute("dup"))
    assert duplicate_pollutant.value.category == "ambiguous_pollutant"
    with pytest.raises(OutputError) as missing_pollutant:
        reader.node_series(1, PollutantAttribute("missing"))
    assert missing_pollutant.value.category == "pollutant_not_found"

    unknown = reader.system_series(ResultAttributeCode(-7), 0, 0)
    assert unknown.selection.attribute == ResultAttributeCode(-7)
    empty = reader.node_series(1, NodeResultAttribute.INVERT_DEPTH, 2, 2)
    assert empty.times == () and empty.values == ()

    duplicate_reader = _edge_reader(tmp_path, duplicate_attribute=True)
    with pytest.raises(OutputError) as duplicate_attribute:
        duplicate_reader.node_series(1, ResultAttributeCode(-7))
    assert duplicate_attribute.value.category == "ambiguous_attribute"


def test_series_selection_normalizes_and_rejects_invalid_inputs() -> None:
    selection = SeriesSelection("node", 0, "invert_depth")
    assert selection.element_type is ResultElementType.NODE
    assert selection.attribute is NodeResultAttribute.INVERT_DEPTH
    assert selection.element == 0
    with pytest.raises(TypeError):
        SeriesSelection("node", True, NodeResultAttribute.INVERT_DEPTH)
    with pytest.raises(ValueError):
        SeriesSelection("node", -1, NodeResultAttribute.INVERT_DEPTH)
    with pytest.raises(TypeError):
        SeriesSelection("node", 0, 0)
    with pytest.raises(ValueError):
        SeriesSelection("node", 0, "flow")
    with pytest.raises(TypeError):
        SeriesSelection("system", None, True)
    with pytest.raises(ValueError):
        SeriesSelection("system", None, PollutantAttribute(0))
    with pytest.raises(TypeError):
        PollutantAttribute(True)
    with pytest.raises(ValueError):
        PollutantAttribute(-1)
    with pytest.raises(ValueError):
        ResultAttributeCode(2**31)


def test_name_and_schema_failures_keep_stable_categories() -> None:
    reader = OutputReader(_FIXTURE)
    with pytest.raises(OutputError) as missing_element:
        reader.node_series(b"\xff\x00missing", NodeResultAttribute.INVERT_DEPTH)
    assert missing_element.value.category == "element_not_found"
    with pytest.raises(OutputError) as missing_pollutant:
        reader.node_series(0, PollutantAttribute("missing"))
    assert missing_pollutant.value.category == "pollutant_not_found"
    with pytest.raises(OutputError) as missing_attribute:
        reader.system_series(ResultAttributeCode(-(2**31)))
    assert missing_attribute.value.category == "attribute_not_found"


def test_structured_results_and_nominal_calendar_are_checked() -> None:
    half_even_down = ReportTiming(-0.5 / 86_400.0, 1, 1)
    half_even_up = ReportTiming(0.5 / 86_400.0, 1, 1)
    assert half_even_down.nominal_date(0) == datetime(1899, 12, 30)
    assert half_even_up.nominal_date(0) == datetime(1899, 12, 30) + timedelta(seconds=2)

    selection = SeriesSelection("system", None, ResultAttributeCode(0))
    result = BulkSeriesResult(
        (datetime(1899, 12, 30),),
        (OutputValueSeries(selection, (1.0,)),),
    )
    assert result.value(0, 0) == 1.0
    with pytest.raises(IndexError):
        result.value(1, 0)
    with pytest.raises(IndexError):
        result.value(0, 1)


def test_structured_result_repr_summarizes_dimensions() -> None:
    selection = SeriesSelection("system", None, ResultAttributeCode(0))
    times = (datetime(1899, 12, 30), datetime(1899, 12, 31))
    values = OutputValueSeries(selection, (1.0, 2.0))
    time_series = OutputTimeSeries(selection, times, values.values)
    result = BulkSeriesResult(times, (values,))

    assert repr(values) == f"OutputValueSeries(selection={selection!r}, periods=2)"
    assert repr(time_series) == f"OutputTimeSeries(selection={selection!r}, periods=2)"
    assert repr(result) == "BulkSeriesResult(periods=2, series=1)"


def test_nominal_calendar_epoch_rounding_and_datetime_overflow() -> None:
    epoch = ReportTiming(-300.0 / 86_400.0, 300, 1)
    assert epoch.nominal_date(0) == datetime(1899, 12, 30)

    rollover = ReportTiming(1.0 - 300.0 / 86_400.0, 300, 1)
    assert rollover.nominal_date(0) == datetime(1899, 12, 31)

    leap_serial = (datetime(2020, 2, 29) - datetime(1899, 12, 30)).total_seconds() / 86_400.0
    leap_day = ReportTiming(leap_serial - 300.0 / 86_400.0, 300, 1)
    assert leap_day.nominal_date(0) == datetime(2020, 2, 29)

    pre_epoch = ReportTiming(-1.0 - 300.0 / 86_400.0, 300, 1)
    assert pre_epoch.nominal_date(0) == datetime(1899, 12, 29)

    below = ReportTiming((0.499999999 - 1.0) / 86_400.0, 1, 1)
    above = ReportTiming((0.500000001 - 1.0) / 86_400.0, 1, 1)
    assert below.nominal_date(0) == datetime(1899, 12, 30)
    assert above.nominal_date(0) == datetime(1899, 12, 30) + timedelta(seconds=1)

    assert epoch.nominal_date(0) == epoch.nominal_date(0)
    assert epoch.nominal_date(-1) is None
    assert epoch.nominal_date(1) is None

    for origin in (-700_000.0, 3_000_000.0):
        with pytest.raises(OutputError) as raised:
            ReportTiming(origin, 1, 1).nominal_date(0)
        assert raised.value.category == "datetime_out_of_range"


def test_output_bounds_clipping_inversion_and_stored_dates() -> None:
    reader = OutputReader(_FIXTURE)
    selection = _sample_selections(reader)[0]
    period_count = reader.metadata.report_timing.period_count
    times = reader.times

    full = reader.read_bulk_series([selection], 0, period_count)
    assert full.times == times
    assert reader.read_bulk_series([selection], start=0, end=period_count) == full
    assert reader.read_bulk_series([selection], 0, 0).times == ()

    before = times[0] - timedelta(seconds=1)
    after = times[-1] + timedelta(seconds=1)
    assert reader.read_bulk_series([selection], before, after).times == times
    assert reader.read_bulk_series([selection], after, None).times == ()

    off_grid_start = times[1] - timedelta(seconds=1)
    off_grid_end = times[3] - timedelta(seconds=1)
    assert reader.read_bulk_series([selection], off_grid_start, off_grid_end).times == times[1:3]
    assert reader.read_bulk_series([selection], 1, times[3]).times == times[1:3]

    with pytest.raises(OutputError) as raised:
        reader.read_bulk_series([selection], times[2], times[1])
    assert raised.value.category == "invalid_period_range"
    with pytest.raises(OutputError) as raised:
        reader.read_bulk_series([selection], 2, times[1])
    assert raised.value.category == "invalid_period_range"

    with pytest.raises(ValueError):
        reader.read_bulk_series([selection], -1, None)
    with pytest.raises(ValueError):
        reader.read_bulk_series([selection], None, period_count + 1)
    with pytest.raises(TypeError):
        reader.read_bulk_series([selection], True, None)
    with pytest.raises(ValueError):
        reader.read_bulk_series([selection], datetime.now(timezone.utc), None)
    assert reader.read_stored_dates(2, 2) == []

    integer_reader = OutputReader(_FIXTURE)
    stored = integer_reader.read_stored_dates(1, 3)
    assert all(type(value) is float for value in stored)
    assert [_f64_bits(value) for value in stored] == [
        _f64_bits(value) for value in integer_reader.read_stored_dates(1, 3)
    ]
    assert integer_reader._times is None

    calendar_stored = reader.read_stored_dates(times[1], times[3])
    assert [_f64_bits(value) for value in calendar_stored] == [_f64_bits(value) for value in stored]


@pytest.mark.parametrize("shared", [True, False], ids=["shared-reader", "independent-readers"])
def test_output_concurrent_queries_match_sequential_reads(shared: bool) -> None:
    reference = OutputReader(_FIXTURE)
    selections = _sample_selections(reference)
    expected_times = reference.times
    requests = [
        (
            start,
            reference.read_bulk_series(selections, start, start + 17),
            reference.read_stored_dates(start, start + 17),
        )
        for start in range(0, 88, 11)
    ]
    readers = [OutputReader(_FIXTURE) for _ in range(1 if shared else len(requests))]
    ready = threading.Barrier(len(requests))
    errors: list[BaseException] = []

    def query(index: int) -> None:
        try:
            reader = readers[0] if shared else readers[index]
            start, expected, dates = requests[index]
            ready.wait(timeout=30)
            # First access races the nominal-date cache initialization.
            assert reader.times == expected_times
            for iteration in range(16):
                result = reader.read_bulk_series(
                    selections, start, start + 17, low_memory=bool(iteration % 2)
                )
                assert result == expected
                assert reader.read_stored_dates(start, start + 17) == dates
                assert reader.metadata == reference.metadata
        except BaseException as error:
            errors.append(error)

    threads = [
        threading.Thread(target=query, args=(index,), daemon=True)
        for index in range(len(requests))
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join(timeout=30)
    assert not any(thread.is_alive() for thread in threads), "output query deadlocked"
    if errors:
        raise errors[0]


def test_output_native_read_releases_gil_to_an_unrelated_worker() -> None:
    reader = OutputReader(_FIXTURE)
    metadata = reader.metadata
    selections: list[SeriesSelection] = []
    for index in range(len(metadata.subcatchments)):
        selections.extend(
            SeriesSelection("subcatchment", index, attribute)
            for attribute in metadata.result_schema.subcatchment
        )
    for index in range(len(metadata.nodes)):
        selections.extend(
            SeriesSelection("node", index, attribute) for attribute in metadata.result_schema.node
        )
    for index in range(len(metadata.links)):
        selections.extend(
            SeriesSelection("link", index, attribute) for attribute in metadata.result_schema.link
        )
    selections.extend(
        SeriesSelection("system", None, attribute) for attribute in metadata.result_schema.system
    )
    selections *= 32

    read_started = threading.Event()
    read_finished = threading.Event()
    worker_progress = threading.Event()
    errors: list[BaseException] = []

    def read() -> None:
        read_started.set()
        try:
            reader.read_bulk_series(
                selections,
                0,
                metadata.report_timing.period_count,
            )
        except BaseException as error:
            errors.append(error)
        finally:
            read_finished.set()

    def worker() -> None:
        read_started.wait()
        while not read_finished.is_set():
            worker_progress.set()

    read_thread = threading.Thread(target=read)
    worker_thread = threading.Thread(target=worker)
    worker_thread.start()
    read_thread.start()
    assert read_started.wait(timeout=30)
    assert worker_progress.wait(timeout=30)
    assert not read_finished.is_set()
    read_thread.join(timeout=180)
    worker_thread.join(timeout=30)
    assert not read_thread.is_alive()
    assert not worker_thread.is_alive()
    assert errors == []


def test_output_metadata_reprs_are_concise() -> None:
    metadata = OutputReader(_FIXTURE).metadata

    assert repr(OutputName(b"J1")) == "OutputName('J1')"
    assert repr(OutputName(b"\xff")) == "OutputName(b'\\xff')"
    assert repr(metadata.report_timing) == "ReportTiming(origin=2.0, step=300, periods=288)"
    assert (
        repr(metadata.subcatchments[0])
        == "SubcatchmentMetadata(index=0, name=OutputName('SUB1'), area=5.0)"
    )
    assert (
        repr(metadata.nodes[0])
        == "NodeMetadata(index=0, name=OutputName('JUNC1'), kind=<NodeKind.JUNCTION: 'junction'>)"
    )
    assert (
        repr(metadata.links[0])
        == "LinkMetadata(index=0, name=OutputName('COND1'), kind=<LinkKind.CONDUIT: 'conduit'>)"
    )
    assert (
        repr(metadata.pollutants[0])
        == "PollutantMetadata(index=0, name=OutputName('Groundwater'), "
        "concentration_units=<ConcentrationUnits.MILLIGRAMS_PER_LITER: "
        "'milligrams_per_liter'>)"
    )
    assert (
        repr(metadata.result_schema) == "ResultSchema(subcatchment=11, node=9, link=8, system=15)"
    )
    assert (
        repr(metadata) == "OutputMetadata(source='Model.out', solver_release=51014, status=0, "
        "flow_units='cfs', unit_system='us', periods=288, subcatchments=3, "
        "nodes=9, links=8, pollutants=3)"
    )


def test_public_output_metadata_vocabulary_is_typed_and_lossless() -> None:
    import swmmrs.output as output_module

    assert [member.value for member in output_module.ResultElementType] == [
        "subcatchment",
        "node",
        "link",
        "system",
    ]
    assert [member.value for member in NodeKind] == [
        "junction",
        "outfall",
        "storage",
        "divider",
    ]
    assert [member.value for member in ConcentrationUnits] == [
        "milligrams_per_liter",
        "micrograms_per_liter",
        "counts_per_liter",
    ]
    assert [member.value for member in SubcatchmentResultAttribute] == [
        "rainfall",
        "snow_depth",
        "evap_loss",
        "infil_loss",
        "runoff_rate",
        "gw_outflow_rate",
        "gw_table_elev",
        "soil_moisture",
    ]
    assert [member.value for member in NodeResultAttribute] == [
        "invert_depth",
        "hydraulic_head",
        "ponded_volume",
        "lateral_inflow",
        "total_inflow",
        "flooding_losses",
    ]
    assert [member.value for member in LinkResultAttribute] == [
        "flow_rate",
        "flow_depth",
        "flow_velocity",
        "flow_volume",
        "capacity",
    ]
    assert [member.value for member in SystemResultAttribute] == [
        "air_temp",
        "rainfall",
        "snow_depth",
        "evap_infil_loss",
        "runoff_flow",
        "dry_weather_inflow",
        "gw_inflow",
        "rdii_inflow",
        "direct_inflow",
        "total_lateral_inflow",
        "flood_losses",
        "outfall_flows",
        "volume_stored",
        "evap_rate",
        "ptnl_evap_rate",
    ]
    assert output_module.OutputName(b"").text == ""
    assert output_module.OutputName(b"\xff").text is None
    assert UnknownCode(-(2**31)).code == -(2**31)
    assert ResultAttributeCode(2**31 - 1).code == 2**31 - 1
    assert output_module.RunStatus(0).is_success
    assert not output_module.RunStatus(-7).is_success

    schema = output_module.ResultSchema(
        subcatchment=(
            output_module.SubcatchmentResultAttribute.RAINFALL,
            output_module.PollutantAttribute(0),
            output_module.ResultAttributeCode(-7),
        ),
        node=(output_module.NodeResultAttribute.INVERT_DEPTH,),
        link=(output_module.LinkResultAttribute.FLOW_RATE,),
        system=(output_module.SystemResultAttribute.RAINFALL,),
    )
    assert isinstance(schema.subcatchment, tuple)
    assert schema.subcatchment[1] == output_module.PollutantAttribute(0)
    with pytest.raises(TypeError):
        output_module.ResultSchema(
            subcatchment=(output_module.NodeResultAttribute.INVERT_DEPTH,),
            node=(),
            link=(),
            system=(),
        )
    with pytest.raises(TypeError):
        UnknownCode(True)
    with pytest.raises(ValueError):
        UnknownCode(2**31)
    with pytest.raises(TypeError):
        output_module.PollutantAttribute(True)
    with pytest.raises(ValueError):
        output_module.PollutantAttribute(-1)
