mod common;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use common::{
    FixtureKind, FixtureLayout, RawOracle, copy_fixture, fixture_layout, fixture_path, write_i32,
    write_u32, write_u64,
};
use swmm_output::{
    ConcentrationUnits, FlowUnits, FormatProblem, LinkKind, LinkResultAttribute, MagicLocation,
    NodeKind, NodeResultAttribute, OutputDateKind, OutputError, OutputRange, OutputReader,
    OutputResultSchema, OutputSection, RunStatus, SignedField, SubcatchmentResultAttribute,
    SystemResultAttribute,
};

fn open_error(path: &Path) -> OutputError {
    match OutputReader::open(path) {
        Ok(_) => panic!("expected open to fail for {}", path.display()),
        Err(error) => error,
    }
}

fn patched_fixture(
    kind: FixtureKind,
    patch: impl FnOnce(&mut Vec<u8>, &FixtureLayout),
) -> (tempfile::TempDir, PathBuf, Vec<u8>, FixtureLayout) {
    let (directory, path, mut bytes) = copy_fixture(kind);
    let layout = fixture_layout(&bytes);
    patch(&mut bytes, &layout);
    fs::write(&path, &bytes).unwrap();
    (directory, path, bytes, layout)
}

fn invalid_format(path: &Path) -> FormatProblem {
    match open_error(path) {
        OutputError::InvalidFormat(problem) => problem,
        error => panic!("expected invalid format, got {error:?}"),
    }
}

fn assert_unsupported(error: OutputError, subcatchment: &[i32], node: &[i32], link: &[i32]) {
    match error {
        OutputError::UnsupportedPropertySchema {
            subcatchment: actual_subcatchment,
            node: actual_node,
            link: actual_link,
        } => {
            assert_eq!(&*actual_subcatchment, subcatchment);
            assert_eq!(&*actual_node, node);
            assert_eq!(&*actual_link, link);
        }
        other => panic!("expected unsupported schema, got {other:?}"),
    }
}

fn assert_ambiguous(error: OutputError, code: i32) {
    match error {
        OutputError::AmbiguousAttribute {
            element_type: swmm_output::ResultElementType::Subcatchment,
            code: actual_code,
            occurrences: actual_occurrences,
        } => {
            assert_eq!(actual_code, code);
            assert_eq!(actual_occurrences, 2);
        }
        other => panic!("expected ambiguous attribute, got {other:?}"),
    }
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn replace_names(bytes: &mut [u8], layout: &FixtureLayout) {
    let count = layout.name_length_offsets.len();
    assert!(count >= 4);
    let names_end = layout
        .pollutant_unit_offsets
        .first()
        .copied()
        .unwrap_or(layout.sections.properties);
    let payload_bytes =
        names_end - layout.sections.identifiers - count * std::mem::size_of::<i32>();
    let mut names = vec![Vec::new(); count];
    names[1] = b"dup".to_vec();
    names[2] = b"dup".to_vec();
    names[3] = vec![0xff];
    let used = names.iter().map(Vec::len).sum::<usize>();
    assert!(payload_bytes >= used);
    names[count - 1].extend(std::iter::repeat_n(b'x', payload_bytes - used));
    let mut cursor = layout.sections.identifiers;
    for name in names {
        write_i32(bytes, cursor, name.len() as i32);
        cursor += 4;
        bytes[cursor..cursor + name.len()].copy_from_slice(&name);
        cursor += name.len();
    }
    assert_eq!(cursor, names_end);
}

#[test]
fn short_file_is_rejected_before_fixed_sections() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    bytes.truncate(27);
    fs::write(&path, &bytes).unwrap();
    let problem = invalid_format(&path);
    assert_eq!(
        problem,
        FormatProblem::ShortFile {
            actual: 27,
            minimum: 28
        }
    );
    drop(directory);
}

#[test]
fn automatic_recovery_preserves_metadata_validation() {
    let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
    let layout = fixture_layout(&bytes);
    write_i32(bytes.as_mut_slice(), layout.name_length_offsets[0], -1);
    bytes.truncate(layout.trailer_offset);
    fs::write(&path, bytes).unwrap();
    let problem = match OutputReader::open(&path) {
        Err(OutputError::InvalidFormat(problem)) => problem,
        Err(error) => panic!("expected invalid format, got {error:?}"),
        Ok(_) => panic!("expected invalid unfinalized metadata"),
    };
    assert_eq!(
        problem,
        FormatProblem::InvalidSignedField {
            field: SignedField::NameLength,
            value: -1,
        }
    );
    drop(directory);
}

#[test]
fn header_magic_is_required_and_trailer_magic_marks_finalization() {
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.trailer_offset + 20, 0x0506_0708);
        });
    let reader = OutputReader::open(path).unwrap();
    assert!(!reader.is_finalized());
    assert_eq!(reader.metadata().run_status(), RunStatus::Unfinalized);

    for trailer_bad in [false, true] {
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, layout| {
                write_i32(bytes, 0, 0x0102_0304);
                if trailer_bad {
                    write_i32(bytes, layout.trailer_offset + 20, 0x0506_0708);
                }
            });
        assert_eq!(
            invalid_format(&path),
            FormatProblem::BadMagic {
                location: MagicLocation::Header,
                value: 0x0102_0304,
            }
        );
    }
}

#[test]
fn every_negative_signed_field_is_rejected_with_its_identity() {
    let header_fields = [
        (12, SignedField::SubcatchmentCount),
        (16, SignedField::NodeCount),
        (20, SignedField::LinkCount),
        (24, SignedField::PollutantCount),
    ];
    for (offset, field) in header_fields {
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, _| {
                write_i32(bytes, offset, -1);
            });
        assert_eq!(
            invalid_format(&path),
            FormatProblem::InvalidSignedField { field, value: -1 }
        );
    }

    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.name_length_offsets[0], -1);
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::InvalidSignedField {
            field: SignedField::NameLength,
            value: -1,
        }
    );

    let original = fs::read(fixture_path(FixtureKind::Expanded)).unwrap();
    let original_layout = fixture_layout(&original);
    for block in &original_layout.property_blocks {
        let offset = block.count_offset;
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, _| {
                write_i32(bytes, offset, -1);
            });
        assert_eq!(
            invalid_format(&path),
            FormatProblem::InvalidSignedField {
                field: SignedField::PropertyCount,
                value: -1,
            }
        );
    }

    for schema_index in 0..4 {
        let original = fs::read(fixture_path(FixtureKind::Expanded)).unwrap();
        let original_layout = fixture_layout(&original);
        let offset = original_layout.result_schemas[schema_index].count_offset;
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, _| {
                write_i32(bytes, offset, -1);
            });
        assert_eq!(
            invalid_format(&path),
            FormatProblem::InvalidSignedField {
                field: SignedField::ResultVariableCount,
                value: -1,
            }
        );
    }

    let trailer_fields = [
        (0, SignedField::IdentifierOffset),
        (4, SignedField::PropertyOffset),
        (8, SignedField::ResultOffset),
        (12, SignedField::PeriodCount),
    ];
    for (relative, field) in trailer_fields {
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, layout| {
                write_i32(bytes, layout.trailer_offset + relative, -1);
            });
        assert_eq!(
            invalid_format(&path),
            FormatProblem::InvalidSignedField { field, value: -1 }
        );
    }

    let (_directory, path, bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.trailer_offset + 12, 0);
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::UnexpectedFileLength {
            expected: (layout.sections.results + 24) as u64,
            actual: bytes.len() as u64,
        }
    );
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.report_step_offset, -1);
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::InvalidReportStep { value: -1 }
    );
}

#[test]
fn impossible_name_extent_is_a_identifier_boundary_error() {
    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.name_length_offsets[0], i32::MAX);
        });
    let expected_end = (layout.name_data_offsets[0] as u64) + i32::MAX as u64;
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionBoundary {
            section: OutputSection::Identifiers,
            expected: expected_end,
            actual: layout.sections.properties as u64,
        }
    );
}

#[test]
fn impossible_property_and_result_dimensions_are_rejected_before_reads() {
    let original = fs::read(fixture_path(FixtureKind::Expanded)).unwrap();
    let original_layout = fixture_layout(&original);
    let property_offset = original_layout.property_blocks[1].count_offset;
    let (_directory, path, _bytes, layout) = patched_fixture(FixtureKind::Expanded, |bytes, _| {
        write_i32(bytes, property_offset, i32::MAX);
    });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionBoundary {
            section: OutputSection::Properties,
            expected: (property_offset as u64) + 4 + i32::MAX as u64 * 4,
            actual: layout.sections.results as u64,
        }
    );
    let schema_offset = original_layout.result_schemas[0].count_offset;
    let (_directory, path, _bytes, layout) = patched_fixture(FixtureKind::Expanded, |bytes, _| {
        write_i32(bytes, schema_offset, i32::MAX);
    });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionBoundary {
            section: OutputSection::ResultDescription,
            expected: (schema_offset as u64) + 4 + i32::MAX as u64 * 4,
            actual: layout.sections.results as u64,
        }
    );
}

#[cfg(target_pointer_width = "32")]
#[test]
fn thirty_two_bit_count_arithmetic_is_checked_before_region_reads() {
    let (_directory, path, _bytes, _layout) = patched_fixture(FixtureKind::Expanded, |bytes, _| {
        write_i32(bytes, 12, 1_073_741_824);
    });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::ArithmeticOverflow {
            context: "identifier count",
        }
    );
}

#[cfg(target_pointer_width = "32")]
#[test]
fn thirty_two_bit_counts_report_host_size_overflow() {
    let (_directory, path, _bytes, _layout) = patched_fixture(FixtureKind::Expanded, |bytes, _| {
        write_i32(bytes, 12, i32::MAX);
    });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::HostSizeOverflow {
            context: "count",
            value: i32::MAX as u64,
        }
    );
}

#[test]
fn section_order_and_boundaries_are_checked_in_declared_order() {
    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(
                bytes,
                layout.trailer_offset + 4,
                (layout.sections.identifiers - 1) as i32,
            );
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionOrder {
            earlier: OutputSection::Identifiers,
            earlier_offset: layout.sections.identifiers as u64,
            later: OutputSection::Properties,
            later_offset: (layout.sections.identifiers - 1) as u64,
        }
    );

    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(
                bytes,
                layout.trailer_offset + 4,
                (layout.sections.properties + 1) as i32,
            );
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionBoundary {
            section: OutputSection::Identifiers,
            expected: (layout.sections.properties + 1) as u64,
            actual: layout.sections.properties as u64,
        }
    );

    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(
                bytes,
                layout.trailer_offset + 8,
                (layout.sections.results + 4) as i32,
            );
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionBoundary {
            section: OutputSection::Timing,
            expected: (layout.sections.results + 4) as u64,
            actual: layout.sections.results as u64,
        }
    );
}

#[test]
fn impossible_section_offsets_are_rejected_before_result_extent_math() {
    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(
                bytes,
                layout.trailer_offset + 8,
                (layout.sections.properties - 1) as i32,
            );
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionOrder {
            earlier: OutputSection::Properties,
            earlier_offset: layout.sections.properties as u64,
            later: OutputSection::Results,
            later_offset: (layout.sections.properties - 1) as u64,
        }
    );

    let (_directory, path, _bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.trailer_offset + 8, i32::MAX);
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::SectionOrder {
            earlier: OutputSection::Results,
            earlier_offset: i32::MAX as u64,
            later: OutputSection::Trailer,
            later_offset: layout.trailer_offset as u64,
        }
    );
}

#[test]
fn result_extent_rejects_truncation_and_trailing_bytes() {
    for extra in [-1_i32, 1_i32] {
        let (directory, path, mut bytes) = copy_fixture(FixtureKind::Expanded);
        let original_len = bytes.len();
        let trailer = original_len - 24;
        if extra < 0 {
            bytes.drain(trailer - 1..trailer);
        } else {
            bytes.splice(trailer..trailer, [0xa5]);
        }
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            invalid_format(&path),
            FormatProblem::UnexpectedFileLength {
                expected: original_len as u64,
                actual: bytes.len() as u64,
            }
        );
        drop(directory);
    }
}

#[test]
fn report_step_and_schedule_origin_must_be_valid() {
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.report_step_offset, 0);
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::InvalidReportStep { value: 0 }
    );

    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_u64(bytes, layout.schedule_origin_offset, f64::NAN.to_bits());
        });
    assert_eq!(
        invalid_format(&path),
        FormatProblem::NonFiniteDate {
            kind: OutputDateKind::ReportScheduleOrigin,
            period: None,
            bits: f64::NAN.to_bits(),
        }
    );
}

#[test]
fn legacy_and_expanded_property_schemas_are_both_accepted() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let (_directory, path, _bytes) = copy_fixture(kind);
        let reader = OutputReader::open(&path).unwrap();
        assert_eq!(reader.metadata().solver_release(), kind.expected_release());
    }
}

#[test]
fn mixed_and_unknown_property_schemas_are_not_guessed() {
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, layout.property_blocks[1].code_offsets[2], 3);
        });
    assert_unsupported(open_error(&path), &[1], &[0, 2, 3], &[0, 6, 7, 4, 8]);

    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Legacy, |bytes, layout| {
            write_i32(bytes, layout.property_blocks[2].code_offsets[1], 99);
        });
    assert_unsupported(open_error(&path), &[1], &[0, 2, 3], &[0, 99, 4, 3, 5]);
}

#[test]
fn warning_codes_open_and_preserve_result_bits() {
    for warning in [1, -7, 65_535] {
        let oracle = RawOracle::load(FixtureKind::Expanded);
        let (_directory, path, _bytes, _layout) =
            patched_fixture(FixtureKind::Expanded, |bytes, layout| {
                write_i32(bytes, layout.simulation_code_offset, warning);
            });
        let mut reader = OutputReader::open(&path).unwrap();
        assert_eq!(reader.metadata().run_status(), RunStatus::Warning(warning));
        let selections = oracle.all_selections(&reader);
        let expected = oracle.cell_bits(&selections[0], 0);
        let actual = reader
            .read_bulk_series(&selections[0..1], OutputRange::Periods { start: 0, end: 1 })
            .unwrap()
            .series()[0]
            .values()[0];
        assert_eq!(actual.to_bits(), expected);
    }
}

#[test]
fn unknown_categories_names_and_release_are_preserved() {
    const UNKNOWN_SUBCATCHMENT: i32 = 99_003;
    const UNKNOWN_NODE: i32 = 99_004;
    const UNKNOWN_LINK: i32 = 99_005;
    const UNKNOWN_SYSTEM: i32 = 99_006;

    let oracle = RawOracle::load(FixtureKind::Expanded);
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_i32(bytes, 4, 9_999_999);
            write_i32(bytes, 8, 99_001);
            write_i32(bytes, layout.pollutant_unit_offsets[0], 99_002);
            write_i32(bytes, layout.property_blocks[1].value_offsets[0], 123_456);
            write_i32(bytes, layout.property_blocks[2].value_offsets[0], 654_321);
            write_i32(
                bytes,
                layout.result_schemas[0].code_offsets[0],
                UNKNOWN_SUBCATCHMENT,
            );
            write_i32(
                bytes,
                layout.result_schemas[1].code_offsets[0],
                UNKNOWN_NODE,
            );
            write_i32(
                bytes,
                layout.result_schemas[2].code_offsets[0],
                UNKNOWN_LINK,
            );
            write_i32(
                bytes,
                layout.result_schemas[3].code_offsets[0],
                UNKNOWN_SYSTEM,
            );
            replace_names(bytes, layout);
        });
    let mut reader = OutputReader::open(&path).unwrap();
    let selections = {
        let metadata = reader.metadata();
        assert_eq!(metadata.solver_release(), 9_999_999);
        assert_eq!(metadata.flow_units(), FlowUnits::Unknown(99_001));
        assert_eq!(metadata.unit_system(), None);
        assert_eq!(
            metadata.pollutants()[0].concentration_units(),
            ConcentrationUnits::Unknown(99_002)
        );
        assert_eq!(metadata.nodes()[0].kind(), NodeKind::Unknown(123_456));
        assert_eq!(metadata.links()[0].kind(), LinkKind::Unknown(654_321));
        assert_eq!(metadata.subcatchments()[0].name().as_bytes(), b"");
        assert_eq!(metadata.nodes()[0].name().as_bytes(), b"dup");
        assert_eq!(metadata.nodes()[1].name().as_bytes(), b"dup");
        assert_eq!(metadata.links()[0].name().as_bytes(), &[0xff]);
        assert_eq!(
            metadata.result_schema().subcatchment()[0],
            SubcatchmentResultAttribute::Unknown(UNKNOWN_SUBCATCHMENT)
        );
        assert_eq!(
            metadata.result_schema().node()[0],
            NodeResultAttribute::Unknown(UNKNOWN_NODE)
        );
        assert_eq!(
            metadata.result_schema().link()[0],
            LinkResultAttribute::Unknown(UNKNOWN_LINK)
        );
        assert_eq!(
            metadata.result_schema().system()[0],
            SystemResultAttribute::Unknown(UNKNOWN_SYSTEM)
        );
        [
            swmm_output::OutputSeriesSelection::Subcatchment {
                id: metadata.subcatchments()[0].id(),
                attribute: SubcatchmentResultAttribute::Unknown(UNKNOWN_SUBCATCHMENT),
            },
            swmm_output::OutputSeriesSelection::Node {
                id: metadata.nodes()[0].id(),
                attribute: NodeResultAttribute::Unknown(UNKNOWN_NODE),
            },
            swmm_output::OutputSeriesSelection::Link {
                id: metadata.links()[0].id(),
                attribute: LinkResultAttribute::Unknown(UNKNOWN_LINK),
            },
            swmm_output::OutputSeriesSelection::System {
                attribute: SystemResultAttribute::Unknown(UNKNOWN_SYSTEM),
            },
        ]
    };
    let original_reader = OutputReader::open(fixture_path(FixtureKind::Expanded)).unwrap();
    let original_selections = oracle.all_selections(&original_reader);
    let node_index = oracle.counts.subcatchments * oracle.schemas.subcatchment.len();
    let link_index = node_index + oracle.counts.nodes * oracle.schemas.node.len();
    let system_index = link_index + oracle.counts.links * oracle.schemas.link.len();
    let expected = [
        oracle.cell_bits(&original_selections[0], 0),
        oracle.cell_bits(&original_selections[node_index], 0),
        oracle.cell_bits(&original_selections[link_index], 0),
        oracle.cell_bits(&original_selections[system_index], 0),
    ];
    let range = OutputRange::Periods { start: 0, end: 1 };
    let selective = reader.read_bulk_series(&selections, range).unwrap();
    let by_period = reader
        .read_bulk_series_by_period(&selections, range)
        .unwrap();
    assert_eq!(selective.series().len(), expected.len());
    assert_eq!(by_period.series().len(), expected.len());
    for (index, expected_bits) in expected.iter().copied().enumerate() {
        assert_eq!(
            selective.series()[index].values()[0].to_bits(),
            expected_bits
        );
        assert_eq!(
            by_period.series()[index].values()[0].to_bits(),
            expected_bits
        );
    }
}

#[test]
fn duplicate_result_codes_open_but_are_ambiguous_when_selected() {
    let (_directory, path, bytes, layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            let duplicate = 99_004;
            write_i32(bytes, layout.result_schemas[0].code_offsets[0], duplicate);
            write_i32(bytes, layout.result_schemas[0].code_offsets[1], duplicate);
        });
    let duplicate_code = read_i32(&bytes, layout.result_schemas[0].code_offsets[1]);
    let mut reader = OutputReader::open(&path).unwrap();
    let id = reader.metadata().subcatchments()[0].id();
    let error = reader
        .read_bulk_series(
            &[swmm_output::OutputSeriesSelection::Subcatchment {
                id,
                attribute: SubcatchmentResultAttribute::Unknown(duplicate_code),
            }],
            OutputRange::Periods { start: 0, end: 1 },
        )
        .unwrap_err();
    assert_ambiguous(error, duplicate_code);
}

#[test]
fn nonfinite_property_and_result_bits_are_preserved() {
    const RESULT_NAN_BITS: u32 = 0x7fc0_1234;

    let oracle = RawOracle::load(FixtureKind::Expanded);
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_u32(
                bytes,
                layout.property_blocks[0].value_offsets[0],
                f32::NAN.to_bits(),
            );
            write_u32(
                bytes,
                layout.property_blocks[1].value_offsets[1],
                f32::INFINITY.to_bits(),
            );
            write_u32(bytes, layout.sections.results + 8, RESULT_NAN_BITS);
            write_u32(
                bytes,
                layout.sections.results + 12,
                f32::NEG_INFINITY.to_bits(),
            );
        });
    let mut reader = OutputReader::open(&path).unwrap();
    assert_eq!(
        reader.metadata().subcatchments()[0].area().to_bits(),
        f32::NAN.to_bits()
    );
    assert_eq!(
        reader.metadata().nodes()[0].invert_elevation().to_bits(),
        f32::INFINITY.to_bits()
    );
    let selections = oracle.all_selections(&reader);
    let range = OutputRange::Periods { start: 0, end: 1 };
    let selective = reader.read_bulk_series(&selections[..2], range).unwrap();
    let by_period = reader
        .read_bulk_series_by_period(&selections[..2], range)
        .unwrap();
    assert_eq!(selective.series()[0].values()[0].to_bits(), RESULT_NAN_BITS);
    assert_eq!(by_period.series()[0].values()[0].to_bits(), RESULT_NAN_BITS);
    assert_eq!(
        selective.series()[1].values()[0].to_bits(),
        f32::NEG_INFINITY.to_bits()
    );
    assert_eq!(
        by_period.series()[1].values()[0].to_bits(),
        f32::NEG_INFINITY.to_bits()
    );
}

#[test]
fn nonfinite_stored_date_is_rejected_only_when_that_date_is_read() {
    let (_directory, path, _bytes, _layout) =
        patched_fixture(FixtureKind::Expanded, |bytes, layout| {
            write_u64(bytes, layout.sections.results, f64::INFINITY.to_bits());
        });
    let mut reader = OutputReader::open(&path).unwrap();
    let error = reader
        .read_stored_dates(OutputRange::Periods { start: 0, end: 1 })
        .unwrap_err();
    assert!(matches!(
        error,
        OutputError::InvalidFormat(FormatProblem::NonFiniteDate {
            kind: OutputDateKind::StoredReportDate,
            period: Some(0),
            bits,
        }) if bits == f64::INFINITY.to_bits()
    ));
}

#[test]
fn opening_a_missing_source_retains_path_operation_and_io_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.out");
    let error = open_error(&path);
    match error {
        OutputError::Io {
            path: actual,
            operation: swmm_output::IoOperation::Open,
            offset: None,
            source,
        } => {
            assert_eq!(actual, path);
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
            let error = OutputError::Io {
                path: actual,
                operation: swmm_output::IoOperation::Open,
                offset: None,
                source,
            };
            assert!(std::error::Error::source(&error).is_some());
        }
        other => panic!("expected open I/O error, got {other:?}"),
    }
}

#[test]
fn output_result_schema_remains_a_public_immutable_value() {
    let (_directory, path, _bytes) = copy_fixture(FixtureKind::Expanded);
    let reader = OutputReader::open(&path).unwrap();
    let schema: &OutputResultSchema = reader.metadata().result_schema();
    assert!(!schema.subcatchment().is_empty());
}
