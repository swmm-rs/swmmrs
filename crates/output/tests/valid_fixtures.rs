mod common;

use std::time::Duration;

use common::{
    FixtureKind, RawOracle, assert_fixture_identity, copy_fixture, fixture_layout, fixture_path,
    link_attribute, node_attribute, subcatchment_attribute, system_attribute,
};
use swmm_output::{
    BulkSeriesResult, ConcentrationUnits, FlowUnits, LinkKind, NodeKind, OutputRange, OutputReader,
    OutputSeriesSelection, RunStatus,
};

#[test]
fn fixture_identity_and_provenance_are_pinned() {
    assert_fixture_identity(FixtureKind::Legacy);
    assert_fixture_identity(FixtureKind::Expanded);
    let provenance = std::fs::read_to_string(common::provenance_path()).unwrap();
    assert!(provenance.contains(common::LEGACY_SHA256));
    assert!(provenance.contains(common::EXPANDED_SHA256));
    assert!(provenance.contains("writer_revision = \"410f19013149f6c995251d7df888803b1c70fdd8\""));
    assert!(provenance.contains("writer_revision = \"02192473c7f222faf4277a264e2ccc107ab68f0b\""));
    assert!(provenance.contains("expanded/minimal.inp"));
    let input = std::fs::read(common::fixture_input_path()).unwrap();
    let expected_input_hash = provenance
        .lines()
        .find_map(|line| {
            line.strip_prefix("input_sha256 = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .expect("expanded input hash must be recorded in provenance");
    let expected_input_length = provenance
        .lines()
        .find_map(|line| {
            line.strip_prefix("input_byte_length = ")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .expect("expanded input length must be recorded in provenance");
    assert_eq!(input.len(), expected_input_length);
    assert_eq!(common::sha256_hex(&input), expected_input_hash);
}

#[test]
fn valid_fixtures_expose_exact_metadata_dates_and_values() {
    assert_valid_fixture(FixtureKind::Legacy);
    assert_valid_fixture(FixtureKind::Expanded);
}

#[test]
fn output_open_detects_unfinalized_files_and_exposes_only_complete_records() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let (directory, path, bytes) = copy_fixture(kind);
        let layout = fixture_layout(&bytes);
        let oracle = RawOracle::load(kind);
        let bytes_per_period =
            (layout.trailer_offset - layout.sections.results) / oracle.period_count;
        for (period_count, trailing_bytes) in [
            (oracle.period_count, 0),
            (oracle.period_count - 1, 1),
            (oracle.period_count - 1, bytes_per_period - 1),
            (0, 0),
        ] {
            let end = layout.sections.results + period_count * bytes_per_period + trailing_bytes;
            std::fs::write(&path, &bytes[..end]).unwrap();

            let mut reader = OutputReader::open(&path).unwrap();
            assert!(!reader.is_finalized());
            assert_eq!(
                reader.metadata().report_timing().period_count(),
                period_count
            );
            let selection = oracle.all_selections(&reader)[0];
            let result = reader
                .read_bulk_series(&[selection], OutputRange::All)
                .unwrap();
            assert_eq!(result.times().len(), period_count);
            assert_eq!(result.series()[0].values().len(), period_count);
            let dates = reader.read_stored_dates(OutputRange::All).unwrap();
            assert_eq!(dates.len(), period_count);
            if period_count > 0 {
                for period in [0, period_count - 1] {
                    assert_eq!(
                        result.series()[0].values()[period].to_bits(),
                        oracle.cell_bits(&selection, period)
                    );
                    assert_eq!(
                        dates[period].serial_days().to_bits(),
                        oracle.date_bits(period)
                    );
                }
            }
        }
        drop(directory);
    }
}

#[test]
fn incomplete_fixture_matches_the_finalized_prefix() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/incomplete/minimal-partial-period.out");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 1347);
    assert_eq!(
        common::sha256_hex(&bytes),
        "0aba87e26c949f4e79ed4a96e8388dfb3dd6fb9fd61eca2273fdc5c2f3e9c335"
    );
    let mut reader = OutputReader::open(&path).unwrap();
    let mut finalized = OutputReader::open(fixture_path(FixtureKind::Expanded)).unwrap();
    assert!(!reader.is_finalized());
    assert!(finalized.is_finalized());
    assert_eq!(reader.metadata().run_status(), RunStatus::Unfinalized);
    assert_eq!(reader.metadata().report_timing().period_count(), 5);

    let oracle = RawOracle::load(FixtureKind::Expanded);
    let selections = oracle.all_selections(&reader);
    let recovered = reader
        .read_bulk_series(&selections, OutputRange::All)
        .unwrap();
    let expected = finalized
        .read_bulk_series(&selections, OutputRange::Periods { start: 0, end: 5 })
        .unwrap();
    assert_structured_equal(&recovered, &expected);

    let recovered = reader
        .read_bulk_series_by_period(&selections, OutputRange::All)
        .unwrap();
    assert_structured_equal(&recovered, &expected);
    let recovered_dates = reader.read_stored_dates(OutputRange::All).unwrap();
    let expected_dates = finalized
        .read_stored_dates(OutputRange::Periods { start: 0, end: 5 })
        .unwrap();
    assert_eq!(recovered_dates, expected_dates);
}

fn assert_valid_fixture(kind: FixtureKind) {
    let path = fixture_path(kind);
    let expected_path = path.as_path();
    let mut reader = OutputReader::open(expected_path).unwrap();
    assert_eq!(reader.source_path(), expected_path);
    assert!(reader.is_finalized());
    let oracle = RawOracle::load(kind);
    let metadata = reader.metadata();

    assert_eq!(metadata.solver_release(), oracle.solver_release);
    assert_eq!(metadata.run_status(), RunStatus::Success);
    assert_eq!(metadata.flow_units(), FlowUnits::Cfs);
    assert_eq!(metadata.unit_system(), FlowUnits::Cfs.unit_system());
    assert_eq!(
        metadata
            .report_timing()
            .report_schedule_origin()
            .serial_days()
            .to_bits(),
        oracle.schedule_origin_bits
    );
    assert_eq!(
        metadata.report_timing().report_step(),
        Duration::from_secs(oracle.report_step_seconds as u64)
    );
    assert_eq!(metadata.report_timing().period_count(), oracle.period_count);

    assert_eq!(metadata.subcatchments().len(), oracle.counts.subcatchments);
    assert_eq!(metadata.nodes().len(), oracle.counts.nodes);
    assert_eq!(metadata.links().len(), oracle.counts.links);
    assert_eq!(metadata.pollutants().len(), oracle.counts.pollutants);

    for (index, (actual, expected)) in metadata
        .subcatchments()
        .iter()
        .zip(&oracle.subcatchments)
        .enumerate()
    {
        assert_eq!(actual.id().index(), index);
        assert_eq!(actual.name().as_bytes(), expected.name.bytes.as_slice());
        assert_eq!(actual.area().to_bits(), expected.area.bits);
    }
    for (index, (actual, expected)) in metadata.nodes().iter().zip(&oracle.nodes).enumerate() {
        assert_eq!(actual.id().index(), index);
        assert_eq!(actual.name().as_bytes(), expected.name.bytes.as_slice());
        assert_eq!(actual.kind(), node_kind(expected.kind));
        assert_eq!(
            actual.invert_elevation().to_bits(),
            expected.invert_elevation.bits
        );
        assert_eq!(
            actual.maximum_depth().to_bits(),
            expected.maximum_depth.bits
        );
    }
    for (index, (actual, expected)) in metadata.links().iter().zip(&oracle.links).enumerate() {
        assert_eq!(actual.id().index(), index);
        assert_eq!(actual.name().as_bytes(), expected.name.bytes.as_slice());
        assert_eq!(actual.kind(), link_kind(expected.kind));
        assert_eq!(actual.inlet_offset().to_bits(), expected.inlet_offset.bits);
        assert_eq!(
            actual.outlet_offset().to_bits(),
            expected.outlet_offset.bits
        );
        assert_eq!(
            actual.maximum_depth().to_bits(),
            expected.maximum_depth.bits
        );
        assert_eq!(actual.length().to_bits(), expected.length.bits);
    }
    for (index, (actual, expected)) in metadata
        .pollutants()
        .iter()
        .zip(&oracle.pollutants)
        .enumerate()
    {
        assert_eq!(actual.id().index(), index);
        assert_eq!(actual.name().as_bytes(), expected.name.bytes.as_slice());
        assert_eq!(
            actual.concentration_units(),
            concentration_units(expected.concentration_units)
        );
    }

    let schema = metadata.result_schema();
    let expected_subcatchment = oracle
        .schemas
        .subcatchment
        .iter()
        .map(|&code| subcatchment_attribute(code, metadata.pollutants()))
        .collect::<Vec<_>>();
    let expected_node = oracle
        .schemas
        .node
        .iter()
        .map(|&code| node_attribute(code, metadata.pollutants()))
        .collect::<Vec<_>>();
    let expected_link = oracle
        .schemas
        .link
        .iter()
        .map(|&code| link_attribute(code, metadata.pollutants()))
        .collect::<Vec<_>>();
    let expected_system = oracle
        .schemas
        .system
        .iter()
        .map(|&code| system_attribute(code))
        .collect::<Vec<_>>();
    assert_eq!(schema.subcatchment(), expected_subcatchment.as_slice());
    assert_eq!(schema.node(), expected_node.as_slice());
    assert_eq!(schema.link(), expected_link.as_slice());
    assert_eq!(schema.system(), expected_system.as_slice());

    let timing = metadata.report_timing();
    let nominal_periods = [
        0,
        oracle.period_count / 2,
        oracle.period_count.saturating_sub(1),
    ];
    for period in nominal_periods {
        let expected = f64::from_bits(oracle.schedule_origin_bits)
            + (period as f64 + 1.0) * oracle.report_step_seconds as f64 / 86_400.0;
        assert_eq!(
            timing.nominal_date(period).unwrap().serial_days().to_bits(),
            expected.to_bits()
        );
    }
    assert!(timing.nominal_date(oracle.period_count).is_none());

    let stored_dates = reader
        .read_stored_dates(OutputRange::Periods {
            start: 0,
            end: oracle.period_count,
        })
        .unwrap();
    assert_eq!(stored_dates.len(), oracle.period_count);
    for (period, actual) in stored_dates.iter().enumerate() {
        assert_eq!(actual.serial_days().to_bits(), oracle.date_bits(period));
    }

    let selections = oracle.all_selections(&reader);
    let range = OutputRange::Periods {
        start: 0,
        end: oracle.period_count,
    };
    let selective = reader.read_bulk_series(&selections, range).unwrap();
    let by_period = reader
        .read_bulk_series_by_period(&selections, range)
        .unwrap();
    assert_structured_bits(&selective, &oracle, &selections);
    assert_structured_bits(&by_period, &oracle, &selections);
    assert_structured_equal(&selective, &by_period);

    if kind == FixtureKind::Legacy {
        assert_golden_bits(&mut reader, &selections);
    }
}

fn assert_golden_bits(reader: &mut OutputReader, selections: &[OutputSeriesSelection]) {
    for (selection_index, expected_bits) in [
        (0, 0x3cf5_c28f_u32),
        (33, 0x3b0c_7643_u32),
        (114, 0x3800_e1de_u32),
        (178, 0x428c_0000_u32),
    ] {
        let actual = reader
            .read_bulk_series(
                &selections[selection_index..=selection_index],
                OutputRange::Periods { start: 0, end: 1 },
            )
            .unwrap();
        assert_eq!(actual.series()[0].values()[0].to_bits(), expected_bits);
    }
}

fn assert_structured_bits(
    result: &BulkSeriesResult,
    oracle: &RawOracle,
    selections: &[OutputSeriesSelection],
) {
    assert_eq!(result.times().len(), oracle.period_count);
    assert_eq!(result.series().len(), selections.len());
    for (series, selection) in result.series().iter().zip(selections) {
        assert_eq!(series.selection(), *selection);
        assert_eq!(series.values().len(), oracle.period_count);
        for (period, value) in series.values().iter().enumerate() {
            assert_eq!(value.to_bits(), oracle.cell_bits(selection, period));
        }
    }
}

fn assert_structured_equal(left: &BulkSeriesResult, right: &BulkSeriesResult) {
    assert_eq!(left.times().len(), right.times().len());
    for (left, right) in left.times().iter().zip(right.times()) {
        assert_eq!(left.serial_days().to_bits(), right.serial_days().to_bits());
    }
    assert_eq!(left.series().len(), right.series().len());
    for (left, right) in left.series().iter().zip(right.series()) {
        assert_eq!(left.selection(), right.selection());
        assert_eq!(left.values().len(), right.values().len());
        for (left, right) in left.values().iter().zip(right.values()) {
            assert_eq!(left.to_bits(), right.to_bits());
        }
    }
}

fn node_kind(code: i32) -> NodeKind {
    match code {
        0 => NodeKind::Junction,
        1 => NodeKind::Outfall,
        2 => NodeKind::Storage,
        3 => NodeKind::Divider,
        value => NodeKind::Unknown(value),
    }
}

fn link_kind(code: i32) -> LinkKind {
    match code {
        0 => LinkKind::Conduit,
        1 => LinkKind::Pump,
        2 => LinkKind::Orifice,
        3 => LinkKind::Weir,
        4 => LinkKind::Outlet,
        value => LinkKind::Unknown(value),
    }
}

fn concentration_units(code: i32) -> ConcentrationUnits {
    match code {
        0 => ConcentrationUnits::MilligramsPerLiter,
        1 => ConcentrationUnits::MicrogramsPerLiter,
        2 => ConcentrationUnits::CountsPerLiter,
        value => ConcentrationUnits::Unknown(value),
    }
}

#[test]
fn finalized_output_without_report_periods_remains_readable() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let (_directory, path, bytes) = copy_fixture(kind);
        let layout = fixture_layout(&bytes);
        let mut trailer = bytes[layout.trailer_offset..].to_vec();
        trailer[12..16].copy_from_slice(&0_i32.to_le_bytes());
        let mut empty = bytes[..layout.sections.results].to_vec();
        empty.extend_from_slice(&trailer);
        std::fs::write(&path, empty).unwrap();
        let mut reader = OutputReader::open(&path).unwrap();
        assert!(reader.is_finalized());
        assert_eq!(reader.metadata().report_timing().period_count(), 0);
        let selection = RawOracle::load(kind).all_selections(&reader)[0];
        let result = reader
            .read_bulk_series(&[selection], OutputRange::All)
            .unwrap();
        assert!(result.times().is_empty());
        assert!(result.series()[0].values().is_empty());
        assert!(
            reader
                .read_stored_dates(OutputRange::All)
                .unwrap()
                .is_empty()
        );
    }
}
