mod common;

use std::fs;
use std::path::PathBuf;

use common::{FixtureKind, RawOracle, copy_fixture, fixture_layout, write_i32, write_u64};
use swmm_output::{
    BulkSeriesResult, IoOperation, LinkResultAttribute, NodeResultAttribute, OutputElementSelector,
    OutputElementType, OutputError, OutputRange, OutputReader, OutputSeriesSelection,
    OutputTimeSeries, ResultElementType, SubcatchmentResultAttribute, SwmmDate,
    SystemResultAttribute,
};

fn patch_names(kind: FixtureKind, desired: &[&[u8]]) -> (tempfile::TempDir, PathBuf) {
    let (directory, path, mut bytes) = copy_fixture(kind);
    let layout = fixture_layout(&bytes);
    assert_eq!(desired.len(), layout.name_length_offsets.len());
    let names_end = layout
        .pollutant_unit_offsets
        .first()
        .copied()
        .unwrap_or(layout.sections.properties);
    let payload_bytes = names_end - layout.sections.identifiers - desired.len() * 4;
    let mut names = desired.iter().map(|name| name.to_vec()).collect::<Vec<_>>();
    let used = names.iter().map(Vec::len).sum::<usize>();
    assert!(used <= payload_bytes);
    names
        .last_mut()
        .unwrap()
        .extend(std::iter::repeat_n(b'x', payload_bytes - used));
    let mut cursor = layout.sections.identifiers;
    for name in names {
        write_i32(&mut bytes, cursor, name.len() as i32);
        cursor += 4;
        bytes[cursor..cursor + name.len()].copy_from_slice(&name);
        cursor += name.len();
    }
    assert_eq!(cursor, names_end);
    fs::write(&path, bytes).unwrap();
    (directory, path)
}

fn family_by_index(
    reader: &mut OutputReader,
    selection: OutputSeriesSelection,
    range: OutputRange,
) -> OutputTimeSeries {
    match selection {
        OutputSeriesSelection::Subcatchment { id, attribute } => reader
            .subcatchment_series(OutputElementSelector::Index(id.index()), attribute, range)
            .unwrap(),
        OutputSeriesSelection::Node { id, attribute } => reader
            .node_series(OutputElementSelector::Index(id.index()), attribute, range)
            .unwrap(),
        OutputSeriesSelection::Link { id, attribute } => reader
            .link_series(OutputElementSelector::Index(id.index()), attribute, range)
            .unwrap(),
        OutputSeriesSelection::System { attribute } => {
            reader.system_series(attribute, range).unwrap()
        }
    }
}

fn family_by_name(
    reader: &mut OutputReader,
    selection: OutputSeriesSelection,
    name: &[u8],
    range: OutputRange,
) -> OutputTimeSeries {
    match selection {
        OutputSeriesSelection::Subcatchment { attribute, .. } => reader
            .subcatchment_series(OutputElementSelector::Name(name), attribute, range)
            .unwrap(),
        OutputSeriesSelection::Node { attribute, .. } => reader
            .node_series(OutputElementSelector::Name(name), attribute, range)
            .unwrap(),
        OutputSeriesSelection::Link { attribute, .. } => reader
            .link_series(OutputElementSelector::Name(name), attribute, range)
            .unwrap(),
        OutputSeriesSelection::System { .. } => {
            panic!("system selections do not have element names")
        }
    }
}

fn name_for_selection(reader: &OutputReader, selection: OutputSeriesSelection) -> Vec<u8> {
    match selection {
        OutputSeriesSelection::Subcatchment { id, .. } => reader.metadata().subcatchments()
            [id.index()]
        .name()
        .as_bytes()
        .to_vec(),
        OutputSeriesSelection::Node { id, .. } => reader.metadata().nodes()[id.index()]
            .name()
            .as_bytes()
            .to_vec(),
        OutputSeriesSelection::Link { id, .. } => reader.metadata().links()[id.index()]
            .name()
            .as_bytes()
            .to_vec(),
        OutputSeriesSelection::System { .. } => Vec::new(),
    }
}

fn assert_series_equal(actual: &OutputTimeSeries, expected: &BulkSeriesResult) {
    assert_eq!(expected.series().len(), 1);
    assert_eq!(actual.selection(), expected.series()[0].selection());
    assert_eq!(actual.times().len(), expected.times().len());
    assert_eq!(actual.values().len(), expected.times().len());
    for (actual, expected) in actual.times().iter().zip(expected.times()) {
        assert_eq!(
            actual.serial_days().to_bits(),
            expected.serial_days().to_bits()
        );
    }
    for (actual, expected) in actual.values().iter().zip(expected.series()[0].values()) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

#[test]
fn every_family_matches_both_one_selection_bulk_strategies_by_index_and_name() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let oracle = RawOracle::load(kind);
        let mut reader = OutputReader::open(common::fixture_path(kind)).unwrap();
        let selections = oracle.all_selections(&reader);
        let end = oracle.period_count.min(4);
        let range = OutputRange::Periods { start: 0, end };
        for selection in selections {
            let expected = reader
                .read_bulk_series(std::slice::from_ref(&selection), range)
                .unwrap();
            let by_period = reader
                .read_bulk_series_by_period(std::slice::from_ref(&selection), range)
                .unwrap();
            assert_series_equal(&family_by_index(&mut reader, selection, range), &expected);
            assert_series_equal(&family_by_index(&mut reader, selection, range), &by_period);
            if !matches!(selection, OutputSeriesSelection::System { .. }) {
                let name = name_for_selection(&reader, selection);
                let by_name = family_by_name(&mut reader, selection, &name, range);
                assert_series_equal(&by_name, &expected);
            }
        }
    }
}

#[test]
fn exact_names_preserve_empty_non_utf8_case_and_distinct_error_categories() {
    let (_directory, path) = patch_names(
        FixtureKind::Expanded,
        &[b"", b"Case", b"case", &[0xff, 0x00], b"p"],
    );
    let mut reader = OutputReader::open(&path).unwrap();
    let range = OutputRange::Periods { start: 0, end: 1 };
    let empty = reader
        .subcatchment_series(
            OutputElementSelector::Name(b""),
            SubcatchmentResultAttribute::Rainfall,
            range,
        )
        .unwrap();
    assert_eq!(
        empty.selection(),
        OutputSeriesSelection::Subcatchment {
            id: reader.metadata().subcatchments()[0].id(),
            attribute: SubcatchmentResultAttribute::Rainfall,
        }
    );
    assert_eq!(reader.metadata().subcatchments()[0].name().as_bytes(), b"");

    let upper = reader
        .node_series(
            OutputElementSelector::Name(b"Case"),
            NodeResultAttribute::Depth,
            range,
        )
        .unwrap();
    let lower = reader
        .node_series(
            OutputElementSelector::Name(b"case"),
            NodeResultAttribute::Depth,
            range,
        )
        .unwrap();
    assert_eq!(
        upper.selection(),
        OutputSeriesSelection::Node {
            id: reader.metadata().nodes()[0].id(),
            attribute: NodeResultAttribute::Depth,
        }
    );
    assert_eq!(
        lower.selection(),
        OutputSeriesSelection::Node {
            id: reader.metadata().nodes()[1].id(),
            attribute: NodeResultAttribute::Depth,
        }
    );
    let non_utf8 = reader
        .link_series(
            OutputElementSelector::Name(&[0xff, 0x00]),
            LinkResultAttribute::Flow,
            range,
        )
        .unwrap();
    assert_eq!(
        non_utf8.selection(),
        OutputSeriesSelection::Link {
            id: reader.metadata().links()[0].id(),
            attribute: LinkResultAttribute::Flow,
        }
    );

    let absent = reader.node_series(
        OutputElementSelector::Name(b"CASE"),
        NodeResultAttribute::Depth,
        range,
    );
    assert!(matches!(
        absent,
        Err(OutputError::ElementNameNotFound {
            element_type: ResultElementType::Node,
            name,
        }) if &*name == b"CASE"
    ));
    let invalid_index = reader.subcatchment_series(
        OutputElementSelector::Index(1),
        SubcatchmentResultAttribute::Rainfall,
        range,
    );
    assert!(matches!(
        invalid_index,
        Err(OutputError::InvalidElementId {
            element_type: OutputElementType::Subcatchment,
            index: 1,
            count: 1,
        })
    ));
    assert!(
        reader
            .subcatchment_series(
                OutputElementSelector::Index(0),
                SubcatchmentResultAttribute::Rainfall,
                range
            )
            .is_ok()
    );

    let (_directory, duplicate_path) = patch_names(
        FixtureKind::Expanded,
        &[b"", b"dup", b"dup", &[0xff, 0x00], b"p"],
    );
    let mut duplicate_reader = OutputReader::open(&duplicate_path).unwrap();
    let ambiguous = duplicate_reader.node_series(
        OutputElementSelector::Name(b"dup"),
        NodeResultAttribute::Depth,
        range,
    );
    assert!(matches!(
        ambiguous,
        Err(OutputError::AmbiguousElementName {
            element_type: ResultElementType::Node,
            name,
            occurrences: 2,
        }) if &*name == b"dup"
    ));
    assert!(
        duplicate_reader
            .node_series(
                OutputElementSelector::Index(0),
                NodeResultAttribute::Depth,
                range
            )
            .is_ok()
    );
}

#[test]
fn family_ranges_are_nominal_and_empty_without_replacing_stored_dates() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    write_u64(
        &mut bytes,
        layout.sections.results,
        12_345.678_901_f64.to_bits(),
    );
    fs::write(&path, bytes).unwrap();
    let mut reader = OutputReader::open(&path).unwrap();
    let timing = reader.metadata().report_timing();
    let first = timing.nominal_date(0).unwrap();
    let second = timing.nominal_date(1).unwrap();
    let stored = reader
        .read_stored_dates(OutputRange::Periods { start: 0, end: 1 })
        .unwrap();
    assert_eq!(
        stored[0].serial_days().to_bits(),
        12_345.678_901_f64.to_bits()
    );

    let one = reader
        .system_series(
            SystemResultAttribute::Rainfall,
            OutputRange::Dates {
                start: Some(first),
                end: Some(second),
            },
        )
        .unwrap();
    assert_eq!(
        one.times()[0].serial_days().to_bits(),
        first.serial_days().to_bits()
    );
    assert_eq!(one.values().len(), 1);
    let empty = reader
        .system_series(
            SystemResultAttribute::Rainfall,
            OutputRange::Dates {
                start: Some(SwmmDate::from_serial_days(second.serial_days() + 1.0).unwrap()),
                end: None,
            },
        )
        .unwrap();
    assert!(empty.times().is_empty());
    assert!(empty.values().is_empty());
    let inverted = reader.system_series(
        SystemResultAttribute::Rainfall,
        OutputRange::Dates {
            start: Some(second),
            end: Some(first),
        },
    );
    assert!(matches!(
        inverted,
        Err(OutputError::InvalidDateRange { .. })
    ));
    assert!(
        reader
            .system_series(
                SystemResultAttribute::Rainfall,
                OutputRange::Periods { start: 0, end: 1 }
            )
            .is_ok()
    );
    drop(directory);
}

#[test]
fn family_preserves_pollutant_unknown_and_duplicate_schema_errors() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    let unknown_code = 99_123;
    write_i32(
        &mut bytes,
        layout.result_schemas[0].code_offsets[0],
        unknown_code,
    );
    fs::write(&path, bytes).unwrap();
    let mut reader = OutputReader::open(&path).unwrap();
    let unknown = SubcatchmentResultAttribute::Unknown(unknown_code);
    let selection = OutputSeriesSelection::Subcatchment {
        id: reader.metadata().subcatchments()[0].id(),
        attribute: unknown,
    };
    let expected = reader
        .read_bulk_series(std::slice::from_ref(&selection), OutputRange::All)
        .unwrap();
    assert_series_equal(
        &reader
            .subcatchment_series(OutputElementSelector::Index(0), unknown, OutputRange::All)
            .unwrap(),
        &expected,
    );
    let absent = reader.subcatchment_series(
        OutputElementSelector::Index(0),
        SubcatchmentResultAttribute::Unknown(unknown_code + 1),
        OutputRange::All,
    );
    assert!(matches!(
        absent,
        Err(OutputError::AttributeNotFound {
            element_type: ResultElementType::Subcatchment,
            code,
        }) if code == unknown_code + 1
    ));
    drop(directory);

    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    let duplicate_code = 99_124;
    write_i32(
        &mut bytes,
        layout.result_schemas[0].code_offsets[0],
        duplicate_code,
    );
    write_i32(
        &mut bytes,
        layout.result_schemas[0].code_offsets[1],
        duplicate_code,
    );
    fs::write(&path, bytes).unwrap();
    let mut reader = OutputReader::open(&path).unwrap();
    let ambiguous = reader.subcatchment_series(
        OutputElementSelector::Index(0),
        SubcatchmentResultAttribute::Unknown(duplicate_code),
        OutputRange::All,
    );
    assert!(matches!(
        ambiguous,
        Err(OutputError::AmbiguousAttribute {
            element_type: ResultElementType::Subcatchment,
            code,
            occurrences: 2,
        }) if code == duplicate_code
    ));
    assert!(
        reader
            .subcatchment_series(
                OutputElementSelector::Index(0),
                SubcatchmentResultAttribute::Rainfall,
                OutputRange::All,
            )
            .is_err()
    );
    drop(directory);
}

#[test]
fn family_read_failure_is_atomic_and_retryable() {
    let (directory, path, original) = copy_fixture(FixtureKind::Expanded);
    let mut reader = OutputReader::open(&path).unwrap();
    let layout = fixture_layout(&original);
    let system_rainfall_word = {
        let metadata = reader.metadata();
        let schema = metadata.result_schema();
        metadata.subcatchments().len() * schema.subcatchment().len()
            + metadata.nodes().len() * schema.node().len()
            + metadata.links().len() * schema.link().len()
            + schema
                .system()
                .iter()
                .position(|attribute| *attribute == SystemResultAttribute::Rainfall)
                .unwrap()
    };
    let bytes_per_period =
        (layout.trailer_offset - layout.sections.results) / layout.sections.periods;
    let rainfall_end = layout.sections.results
        + (layout.sections.periods - 1) * bytes_per_period
        + 8
        + (system_rainfall_word + 1) * 4;
    fs::write(&path, &original[..rainfall_end - 1]).unwrap();
    let failed = reader.system_series(SystemResultAttribute::Rainfall, OutputRange::All);
    assert!(matches!(
        failed,
        Err(OutputError::Io {
            operation: IoOperation::Read,
            source,
            ..
        }) if source.kind() == std::io::ErrorKind::UnexpectedEof
    ));
    fs::write(&path, original).unwrap();
    assert!(
        reader
            .system_series(SystemResultAttribute::Rainfall, OutputRange::All)
            .is_ok()
    );
    drop(directory);
}

#[test]
fn invalid_positions_retain_each_typed_family_identity() {
    let mut reader = OutputReader::open(common::fixture_path(FixtureKind::Expanded)).unwrap();
    let range = OutputRange::Periods { start: 0, end: 1 };
    let sub_count = reader.metadata().subcatchments().len();
    let node_count = reader.metadata().nodes().len();
    let link_count = reader.metadata().links().len();

    let error = reader.subcatchment_series(
        OutputElementSelector::Index(sub_count),
        SubcatchmentResultAttribute::Rainfall,
        range,
    );
    assert!(matches!(
        error,
        Err(OutputError::InvalidElementId {
            element_type: OutputElementType::Subcatchment,
            ..
        })
    ));
    let error = reader.node_series(
        OutputElementSelector::Index(node_count),
        NodeResultAttribute::Depth,
        range,
    );
    assert!(matches!(
        error,
        Err(OutputError::InvalidElementId {
            element_type: OutputElementType::Node,
            ..
        })
    ));
    let error = reader.link_series(
        OutputElementSelector::Index(link_count),
        LinkResultAttribute::Flow,
        range,
    );
    assert!(matches!(
        error,
        Err(OutputError::InvalidElementId {
            element_type: OutputElementType::Link,
            ..
        })
    ));
}

#[test]
fn unsupported_schema_still_fails_during_open() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    write_i32(
        &mut bytes,
        layout.property_blocks[0].code_offsets[0],
        77_777,
    );
    fs::write(&path, bytes).unwrap();
    assert!(matches!(
        OutputReader::open(&path),
        Err(OutputError::UnsupportedPropertySchema { .. })
    ));
    drop(directory);
}
