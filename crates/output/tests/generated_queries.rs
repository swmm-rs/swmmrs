mod common;

use std::ops::Range;

use common::{FixtureKind, RawOracle, fixture_path};
use swmm_output::{
    BulkSeriesResult, OutputError, OutputRange, OutputReader, OutputSeriesSelection, SwmmDate,
};

#[test]
fn named_query_shapes_cover_boundaries_order_duplicates_and_ranges() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let mut reader = OutputReader::open(fixture_path(kind)).unwrap();
        let oracle = RawOracle::load(kind);
        let all = oracle.all_selections(&reader);
        assert!(!all.is_empty());
        let last = all.len() - 1;
        let period_count = oracle.period_count;
        let stored_slice = reader
            .read_stored_dates(OutputRange::Periods {
                start: 1,
                end: period_count.min(3),
            })
            .unwrap();
        for (index, actual) in stored_slice.iter().enumerate() {
            assert_eq!(actual.serial_days().to_bits(), oracle.date_bits(index + 1));
        }
        let interior = if period_count > 2 {
            1..period_count - 1
        } else {
            0..period_count
        };
        let ranges = [
            0..0,
            0..1,
            interior,
            period_count - 1..period_count,
            0..period_count,
        ];

        assert_query(&mut reader, &oracle, &[], 0..0);
        assert_query(&mut reader, &oracle, &[], 1..period_count.min(2));
        assert_query(&mut reader, &oracle, &all, 0..0);
        for range in ranges {
            assert_query(&mut reader, &oracle, &all[0..1], range.clone());
        }
        assert_query(&mut reader, &oracle, &all[0..1], 0..period_count);
        assert_query(&mut reader, &oracle, &[all[0]], 0..1);
        assert_query(
            &mut reader,
            &oracle,
            &[
                all[0],
                all[33.min(last)],
                all[114.min(last)],
                all[178.min(last)],
            ],
            0..1,
        );
        assert_query(
            &mut reader,
            &oracle,
            &[all[last], all[0], all[33.min(last)]],
            0..period_count,
        );
        assert_query(
            &mut reader,
            &oracle,
            &[all[0], all[0], all[last], all[0]],
            0..period_count,
        );
        assert_query(
            &mut reader,
            &oracle,
            &all[0..all.len().min(8)],
            0..period_count,
        );
        let sparse = [0, all.len() / 5, all.len() / 2, all.len() - 1]
            .into_iter()
            .map(|index| all[index])
            .collect::<Vec<_>>();
        assert_query(&mut reader, &oracle, &sparse, 0..period_count);
        let pollutant = all
            .iter()
            .copied()
            .find(|selection| match selection {
                OutputSeriesSelection::Subcatchment { attribute, .. } => {
                    matches!(
                        attribute,
                        swmm_output::SubcatchmentResultAttribute::Pollutant(_)
                    )
                }
                OutputSeriesSelection::Node { attribute, .. } => {
                    matches!(attribute, swmm_output::NodeResultAttribute::Pollutant(_))
                }
                OutputSeriesSelection::Link { attribute, .. } => {
                    matches!(attribute, swmm_output::LinkResultAttribute::Pollutant(_))
                }
                OutputSeriesSelection::System { .. } => false,
            })
            .expect("fixture should contain pollutant result columns");
        assert_query(&mut reader, &oracle, &[pollutant], 0..period_count);
        assert_query(&mut reader, &oracle, &all, 0..1);
    }
}

#[test]
fn date_ranges_clip_nominal_axis_and_reject_inversion() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let mut reader = OutputReader::open(fixture_path(kind)).unwrap();
        let period_count = reader.metadata().report_timing().period_count();
        let timing = reader.metadata().report_timing();
        let first = timing.nominal_date(0).unwrap();
        let second = timing.nominal_date(1).unwrap();
        let last = timing.nominal_date(period_count - 1).unwrap();
        let before = SwmmDate::from_serial_days(first.serial_days() - 1.0).unwrap();
        let after = SwmmDate::from_serial_days(last.serial_days() + 1.0).unwrap();

        let all_result = reader.read_bulk_series(&[], OutputRange::All).unwrap();
        assert_eq!(all_result.times().len(), period_count);
        let clipped = reader
            .read_bulk_series(
                &[],
                OutputRange::Dates {
                    start: Some(before),
                    end: Some(after),
                },
            )
            .unwrap();
        assert_eq!(clipped.times().len(), period_count);
        assert!(clipped.series().is_empty());

        let exact = reader
            .read_bulk_series(
                &[],
                OutputRange::Dates {
                    start: Some(first),
                    end: Some(second),
                },
            )
            .unwrap();
        assert_eq!(exact.times().len(), 1);
        assert_eq!(
            exact.times()[0].serial_days().to_bits(),
            first.serial_days().to_bits()
        );

        let inverted = reader.read_stored_dates(OutputRange::Dates {
            start: Some(second),
            end: Some(first),
        });
        assert!(matches!(
            inverted,
            Err(OutputError::InvalidDateRange { .. })
        ));
    }
}

#[test]
fn fixed_seed_generator_exercises_heterogeneous_duplicates_reordering_and_ranges() {
    for kind in [FixtureKind::Legacy, FixtureKind::Expanded] {
        let mut reader = OutputReader::open(fixture_path(kind)).unwrap();
        let oracle = RawOracle::load(kind);
        let available = oracle.all_selections(&reader);
        let mut random = Rng::new(0x5eed_5eed_0123_4567);
        for case in 0..256 {
            let selection_count = random.usize(24);
            let mut selections = Vec::with_capacity(selection_count + 4);
            for _ in 0..selection_count {
                selections.push(available[random.usize(available.len())]);
            }
            if case % 3 == 0 && !selections.is_empty() {
                selections.push(selections[0]);
            }
            if case % 5 == 0 {
                selections.reverse();
            }
            if case % 7 == 0 {
                let duplicate_len = selections.len().min(3);
                selections.extend_from_within(..duplicate_len);
            }
            let start = random.usize(oracle.period_count + 1);
            let end = start + random.usize(oracle.period_count - start + 1);
            assert_query(&mut reader, &oracle, &selections, start..end);
        }
    }
}

fn assert_query(
    reader: &mut OutputReader,
    oracle: &RawOracle,
    selections: &[OutputSeriesSelection],
    periods: Range<usize>,
) {
    let range = OutputRange::Periods {
        start: periods.start,
        end: periods.end,
    };
    let selective = reader.read_bulk_series(selections, range).unwrap();
    let by_period = reader
        .read_bulk_series_by_period(selections, range)
        .unwrap();
    assert_structured(&selective, oracle, selections, periods.clone());
    assert_structured(&by_period, oracle, selections, periods);
    assert_eq!(selective.times().len(), by_period.times().len());
    for (selective_time, by_period_time) in selective.times().iter().zip(by_period.times()) {
        assert_eq!(
            selective_time.serial_days().to_bits(),
            by_period_time.serial_days().to_bits()
        );
    }
    assert_eq!(selective.series().len(), by_period.series().len());
    for (selective_series, by_period_series) in selective.series().iter().zip(by_period.series()) {
        assert_eq!(selective_series.selection(), by_period_series.selection());
        assert_eq!(
            selective_series.values().len(),
            by_period_series.values().len()
        );
        for (selective_value, by_period_value) in selective_series
            .values()
            .iter()
            .zip(by_period_series.values())
        {
            assert_eq!(selective_value.to_bits(), by_period_value.to_bits());
        }
    }
}

fn assert_structured(
    result: &BulkSeriesResult,
    oracle: &RawOracle,
    selections: &[OutputSeriesSelection],
    periods: Range<usize>,
) {
    assert_eq!(result.times().len(), periods.len());
    assert_eq!(result.series().len(), selections.len());
    for (series, selection) in result.series().iter().zip(selections) {
        assert_eq!(series.selection(), *selection);
        assert_eq!(series.values().len(), periods.len());
        for (offset, period) in periods.clone().enumerate() {
            assert_eq!(
                series.values()[offset].to_bits(),
                oracle.cell_bits(selection, period)
            );
        }
    }
}

struct Rng(u64);

impl Rng {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0
    }

    fn usize(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        (self.next() as usize) % upper
    }
}
