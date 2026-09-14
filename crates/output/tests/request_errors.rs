mod common;

use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use common::{FixtureKind, RawOracle, copy_fixture, fixture_layout, write_i32, write_u64};
use swmm_output::{
    FormatProblem, NodeResultAttribute, OutputDateKind, OutputElementType, OutputError,
    OutputRange, OutputReader, OutputSeriesSelection, ResultElementType,
    SubcatchmentResultAttribute,
};

fn open_reader(kind: FixtureKind) -> (tempfile::TempDir, PathBuf, OutputReader) {
    let (directory, path, _bytes) = copy_fixture(kind);
    let reader = OutputReader::open(&path).unwrap();
    (directory, path, reader)
}

fn assert_period_error(error: OutputError, start: usize, end: usize, period_count: usize) {
    assert!(matches!(
        error,
        OutputError::InvalidPeriodRange {
            start: actual_start,
            end: actual_end,
            period_count: actual_count,
        } if actual_start == start && actual_end == end && actual_count == period_count
    ));
}

fn assert_element_error(
    error: OutputError,
    element_type: OutputElementType,
    index: usize,
    count: usize,
) {
    assert!(matches!(
        error,
        OutputError::InvalidElementId {
            element_type: actual_type,
            index: actual_index,
            count: actual_count,
        } if actual_type == element_type && actual_index == index && actual_count == count
    ));
}

fn assert_attribute_error(error: OutputError, element_type: ResultElementType, code: i32) {
    assert!(matches!(
        error,
        OutputError::AttributeNotFound {
            element_type: actual_type,
            code: actual_code,
        } if actual_type == element_type && actual_code == code
    ));
}

fn assert_ambiguous_error(error: OutputError, element_type: ResultElementType, code: i32) {
    assert!(matches!(
        error,
        OutputError::AmbiguousAttribute {
            element_type: actual_type,
            code: actual_code,
            occurrences: 2,
        } if actual_type == element_type && actual_code == code
    ));
}

#[test]
fn reversed_and_out_of_range_periods_are_rejected_for_every_read_surface() {
    let (_directory, _path, mut reader) = open_reader(FixtureKind::Expanded);
    let period_count = reader.metadata().report_timing().period_count();
    let selections: [OutputSeriesSelection; 0] = [];
    assert_period_error(
        reader
            .read_bulk_series(&selections, OutputRange::Periods { start: 3, end: 2 })
            .unwrap_err(),
        3,
        2,
        period_count,
    );
    assert_period_error(
        reader
            .read_bulk_series_by_period(
                &selections,
                OutputRange::Periods {
                    start: 0,
                    end: period_count + 1,
                },
            )
            .unwrap_err(),
        0,
        period_count + 1,
        period_count,
    );
    assert_period_error(
        reader
            .read_stored_dates(OutputRange::Periods { start: 2, end: 1 })
            .unwrap_err(),
        2,
        1,
        period_count,
    );
}

#[test]
fn cross_file_typed_ids_are_rejected_for_each_element_family() {
    let (_legacy_directory, _legacy_path, legacy) = open_reader(FixtureKind::Legacy);
    let (_expanded_directory, _expanded_path, mut expanded) = open_reader(FixtureKind::Expanded);
    let selections = [
        (
            OutputSeriesSelection::Subcatchment {
                id: legacy.metadata().subcatchments()[2].id(),
                attribute: SubcatchmentResultAttribute::Rainfall,
            },
            OutputElementType::Subcatchment,
            2,
            expanded.metadata().subcatchments().len(),
        ),
        (
            OutputSeriesSelection::Node {
                id: legacy.metadata().nodes()[8].id(),
                attribute: NodeResultAttribute::Depth,
            },
            OutputElementType::Node,
            8,
            expanded.metadata().nodes().len(),
        ),
        (
            OutputSeriesSelection::Link {
                id: legacy.metadata().links()[7].id(),
                attribute: swmm_output::LinkResultAttribute::Flow,
            },
            OutputElementType::Link,
            7,
            expanded.metadata().links().len(),
        ),
    ];
    for (selection, element_type, index, count) in selections {
        let error = expanded
            .read_bulk_series(&[selection], OutputRange::Periods { start: 0, end: 0 })
            .unwrap_err();
        assert_element_error(error, element_type, index, count);
    }

    let pollutant = legacy.metadata().pollutants()[2].id();
    let expanded_node = expanded.metadata().nodes()[0].id();
    let error = expanded
        .read_bulk_series(
            &[OutputSeriesSelection::Node {
                id: expanded_node,
                attribute: NodeResultAttribute::Pollutant(pollutant),
            }],
            OutputRange::Periods { start: 0, end: 0 },
        )
        .unwrap_err();
    assert_element_error(
        error,
        OutputElementType::Pollutant,
        2,
        expanded.metadata().pollutants().len(),
    );
}

#[test]
fn absent_attributes_are_distinct_from_io_and_element_errors() {
    let (_directory, _path, mut reader) = open_reader(FixtureKind::Expanded);
    let subcatchment = reader.metadata().subcatchments()[0].id();
    let node = reader.metadata().nodes()[0].id();
    let link = reader.metadata().links()[0].id();
    assert_attribute_error(
        reader
            .read_bulk_series(
                &[OutputSeriesSelection::Subcatchment {
                    id: subcatchment,
                    attribute: SubcatchmentResultAttribute::Unknown(30_001),
                }],
                OutputRange::Periods { start: 0, end: 0 },
            )
            .unwrap_err(),
        ResultElementType::Subcatchment,
        30_001,
    );
    assert_attribute_error(
        reader
            .read_bulk_series(
                &[OutputSeriesSelection::Node {
                    id: node,
                    attribute: NodeResultAttribute::Unknown(30_002),
                }],
                OutputRange::Periods { start: 0, end: 0 },
            )
            .unwrap_err(),
        ResultElementType::Node,
        30_002,
    );
    assert_attribute_error(
        reader
            .read_bulk_series(
                &[OutputSeriesSelection::Link {
                    id: link,
                    attribute: swmm_output::LinkResultAttribute::Unknown(30_003),
                }],
                OutputRange::Periods { start: 0, end: 0 },
            )
            .unwrap_err(),
        ResultElementType::Link,
        30_003,
    );
    assert_attribute_error(
        reader
            .read_bulk_series(
                &[OutputSeriesSelection::System {
                    attribute: swmm_output::SystemResultAttribute::Unknown(30_004),
                }],
                OutputRange::Periods { start: 0, end: 0 },
            )
            .unwrap_err(),
        ResultElementType::System,
        30_004,
    );
}

#[test]
fn request_validation_precedence_is_period_then_selection_left_to_right() {
    let (_directory, _path, mut reader) = open_reader(FixtureKind::Expanded);
    let invalid_node = OutputSeriesSelection::Node {
        id: reader.metadata().nodes()[0].id(),
        attribute: NodeResultAttribute::Unknown(40_001),
    };
    let period_count = reader.metadata().report_timing().period_count();
    assert_period_error(
        reader
            .read_bulk_series(&[invalid_node], OutputRange::Periods { start: 4, end: 3 })
            .unwrap_err(),
        4,
        3,
        period_count,
    );

    let invalid_subcatchment = OutputSeriesSelection::Subcatchment {
        id: reader.metadata().subcatchments()[0].id(),
        attribute: SubcatchmentResultAttribute::Unknown(40_002),
    };
    let invalid_system = OutputSeriesSelection::System {
        attribute: swmm_output::SystemResultAttribute::Unknown(40_003),
    };
    assert_attribute_error(
        reader
            .read_bulk_series(
                &[invalid_subcatchment, invalid_system],
                OutputRange::Periods { start: 0, end: 0 },
            )
            .unwrap_err(),
        ResultElementType::Subcatchment,
        40_002,
    );
}

#[test]
fn invalid_selection_precedes_bulk_result_allocation_for_each_method() {
    let (_directory, _path, mut reader) = open_reader(FixtureKind::Expanded);
    let selection = OutputSeriesSelection::Node {
        id: reader.metadata().nodes()[0].id(),
        attribute: NodeResultAttribute::Unknown(40_004),
    };
    let period_count = reader.metadata().report_timing().period_count();

    assert_attribute_error(
        reader
            .read_bulk_series(
                &[selection],
                OutputRange::Periods {
                    start: 0,
                    end: period_count,
                },
            )
            .unwrap_err(),
        ResultElementType::Node,
        40_004,
    );
    assert_attribute_error(
        reader
            .read_bulk_series_by_period(
                &[selection],
                OutputRange::Periods {
                    start: 0,
                    end: period_count,
                },
            )
            .unwrap_err(),
        ResultElementType::Node,
        40_004,
    );
}

#[test]
fn duplicate_codes_are_accepted_on_open_but_rejected_at_selection_time() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    let code = 99_004;
    write_i32(&mut bytes, layout.result_schemas[0].code_offsets[0], code);
    write_i32(&mut bytes, layout.result_schemas[0].code_offsets[1], code);
    fs::write(&path, &bytes).unwrap();
    let mut reader = OutputReader::open(&path).unwrap();
    let selection = OutputSeriesSelection::Subcatchment {
        id: reader.metadata().subcatchments()[0].id(),
        attribute: SubcatchmentResultAttribute::Unknown(code),
    };
    assert_ambiguous_error(
        reader
            .read_bulk_series(&[selection], OutputRange::Periods { start: 0, end: 0 })
            .unwrap_err(),
        ResultElementType::Subcatchment,
        code,
    );
    drop(directory);
}

#[test]
fn stored_date_failure_is_atomic_and_reader_can_retry_absolutely() {
    let oracle = RawOracle::load(FixtureKind::Expanded);
    let (directory, path, original) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&original);
    let mut reader = OutputReader::open(&path).unwrap();
    let mut corrupted = original.clone();
    write_u64(&mut corrupted, layout.sections.results, f64::NAN.to_bits());
    fs::write(&path, &corrupted).unwrap();
    let error = reader
        .read_stored_dates(OutputRange::Periods { start: 0, end: 1 })
        .unwrap_err();
    assert!(matches!(
        error,
        OutputError::InvalidFormat(FormatProblem::NonFiniteDate {
            kind: OutputDateKind::StoredReportDate,
            period: Some(0),
            bits,
        }) if bits == f64::NAN.to_bits()
    ));
    fs::write(&path, &original).unwrap();
    let dates = reader
        .read_stored_dates(OutputRange::Periods { start: 1, end: 3 })
        .unwrap();
    assert_eq!(dates[0].serial_days().to_bits(), oracle.date_bits(1));
    assert_eq!(dates[1].serial_days().to_bits(), oracle.date_bits(2));
    drop(directory);
}

#[test]
fn post_open_truncation_preserves_original_io_source_and_allows_retry() {
    let oracle = RawOracle::load(FixtureKind::Expanded);
    let (directory, path, original) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&original);
    let mut reader = OutputReader::open(&path).unwrap();
    let original_selection = oracle.all_selections(&reader)[0];
    let payload_offset = layout.sections.results as u64 + 8;
    let truncated_len = payload_offset as usize;
    let mut truncated = original.clone();
    truncated.truncate(truncated_len);
    fs::write(&path, &truncated).unwrap();
    let error = reader
        .read_bulk_series(
            &[original_selection],
            OutputRange::Periods { start: 0, end: 1 },
        )
        .unwrap_err();
    match error {
        OutputError::Io {
            path: actual_path,
            operation: swmm_output::IoOperation::Read,
            offset: Some(actual_offset),
            source,
        } => {
            assert_eq!(actual_path, path);
            assert_eq!(actual_offset, payload_offset);
            assert_eq!(source.kind(), io::ErrorKind::UnexpectedEof);
            let io_error = OutputError::Io {
                path: actual_path,
                operation: swmm_output::IoOperation::Read,
                offset: Some(actual_offset),
                source,
            };
            assert!(io_error.source().is_some());
        }
        other => panic!("expected read I/O error, got {other:?}"),
    }
    fs::write(&path, &original).unwrap();
    let values = reader
        .read_bulk_series(
            &[original_selection],
            OutputRange::Periods { start: 1, end: 2 },
        )
        .unwrap();
    assert_eq!(
        values.series()[0].values()[0].to_bits(),
        oracle.cell_bits(&original_selection, 1)
    );
    drop(directory);
}

#[test]
fn date_and_result_errors_do_not_change_the_requested_result_shape() {
    let (_directory, _path, mut reader) = open_reader(FixtureKind::Expanded);
    let period_count = reader.metadata().report_timing().period_count();
    let error = reader.read_bulk_series(
        &[],
        OutputRange::Periods {
            start: 0,
            end: period_count + 1,
        },
    );
    assert!(matches!(error, Err(OutputError::InvalidPeriodRange { .. })));
    let dates = reader
        .read_stored_dates(OutputRange::Periods { start: 0, end: 0 })
        .unwrap();
    assert!(dates.is_empty());
}

#[test]
fn source_path_is_exactly_the_supplied_path_for_opened_files() {
    let (directory, path, reader) = open_reader(FixtureKind::Expanded);
    assert_eq!(reader.source_path(), path.as_path());
    drop(directory);
}
