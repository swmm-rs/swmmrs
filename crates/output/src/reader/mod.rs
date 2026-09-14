mod parse;
mod read;

use std::fs::File;
#[cfg(not(unix))]
use std::io::{Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use crate::{
    AllocationContext, BulkSeriesResult, IoOperation, LinkResultAttribute, NodeResultAttribute,
    OutputElementSelector, OutputError, OutputMetadata, OutputRange, OutputSeriesSelection,
    OutputTimeSeries, SubcatchmentResultAttribute, SwmmDate, SystemResultAttribute,
};

const MAGIC: i32 = 516_114_522;
const HEADER_BYTES: u64 = 28;
const TRAILER_BYTES: u64 = 24;
const DATE_BYTES: u64 = 8;
const FLOAT_BYTES: u64 = 4;

/// Source storage retained by one validated output reader.
pub(crate) enum OutputSource {
    /// Native file handle for the existing path-backed API.
    File(File),
    /// Caller-owned bytes for the browser/WASM API.
    Bytes(Vec<u8>),
}

/// Sole owner of one validated local SWMM binary output file or byte buffer.
pub struct OutputReader {
    pub(crate) source: OutputSource,
    pub(crate) source_path: PathBuf,
    pub(crate) metadata: OutputMetadata,
    pub(crate) result_start: u64,
    pub(crate) payload_bytes: usize,
    pub(crate) bytes_per_period: u64,
    pub(crate) sub_result_offset: u64,
    pub(crate) node_result_offset: u64,
    pub(crate) link_result_offset: u64,
    pub(crate) system_result_offset: u64,
}

impl OutputReader {
    /// Opens a local SWMM binary output file and detects a valid final trailer.
    ///
    /// Finalized files receive strict trailer and extent validation. Otherwise, the reader
    /// exposes every complete result record visible at open time and ignores a partial tail.
    ///
    /// # Arguments
    /// * `path` - Exact source path to retain without canonicalization.
    ///
    /// # Returns
    /// Sole reader owning the open file and immutable metadata.
    ///
    /// # Errors
    /// Returns `OutputError` for I/O, invalid format, unsupported schema, overflow, or allocation failure.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, OutputError> {
        parse::open(path)
    }
    /// Opens an owned SWMM binary output byte buffer.
    ///
    /// This is the byte-native counterpart to [`Self::open`]. The supplied
    /// allocation is moved into the reader and retained for all later queries.
    ///
    /// # Errors
    /// Returns `OutputError` for invalid format, unsupported schema, overflow, or allocation failure.
    pub fn open_bytes(bytes: Vec<u8>) -> Result<Self, OutputError> {
        parse::open_bytes(bytes)
    }

    /// Opens an owned SWMM binary output byte buffer.
    ///
    /// This spelling is retained as a convenience for callers that model the
    /// reader as a byte-backed value.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, OutputError> {
        Self::open_bytes(bytes)
    }

    /// Returns the exact supplied source path.
    ///
    /// # Returns
    /// Informational path copied during open.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Returns eagerly parsed immutable output metadata.
    ///
    /// # Returns
    /// Metadata owned by this reader.
    pub const fn metadata(&self) -> &OutputMetadata {
        &self.metadata
    }

    /// Returns whether the file contained a valid final trailer when opened.
    pub const fn is_finalized(&self) -> bool {
        !matches!(self.metadata.run_status(), crate::RunStatus::Unfinalized)
    }

    /// Reads selected result cells while coalescing only exact adjacency.
    ///
    /// # Arguments
    /// * `selections` - Ordered heterogeneous Output Series Selections.
    /// * `range` - Exact period or Nominal Report Date query range.
    ///
    /// # Returns
    /// Structured values aligned to one ascending Nominal Report Date axis.
    ///
    /// # Errors
    /// Returns `OutputError` on request validation, allocation, or absolute file I/O failure.
    pub fn read_bulk_series(
        &mut self,
        selections: &[OutputSeriesSelection],
        range: OutputRange,
    ) -> Result<BulkSeriesResult, OutputError> {
        read::selective(self, selections, range)
    }

    /// Reads one complete result payload for each selected period.
    ///
    /// # Arguments
    /// * `selections` - Ordered heterogeneous Output Series Selections.
    /// * `range` - Exact period or Nominal Report Date query range.
    ///
    /// # Returns
    /// Structured values aligned to one ascending Nominal Report Date axis.
    ///
    /// # Errors
    /// Returns `OutputError` on request validation, allocation, or absolute file I/O failure.
    pub fn read_bulk_series_by_period(
        &mut self,
        selections: &[OutputSeriesSelection],
        range: OutputRange,
    ) -> Result<BulkSeriesResult, OutputError> {
        read::by_period(self, selections, range)
    }

    /// Reads one subcatchment result series by exact index or stored name.
    ///
    /// # Arguments
    /// * `selector` - Zero-based subcatchment index, or exact stored subcatchment name bytes. Name matching is case-sensitive and does not decode the bytes.
    /// * `attribute` - Subcatchment result attribute to read.
    /// * `range` - Exact half-open period or Nominal Report Date range to resolve.
    ///
    /// # Returns
    /// `OutputTimeSeries` containing the selected attribute values aligned with the resolved Nominal Report Dates.
    ///
    /// # Errors
    /// Returns `OutputError` when the index is outside the stored subcatchment set, the exact name is absent or ambiguous, the attribute is absent or ambiguous in the stored schema, the range is invalid, request or offset arithmetic overflows, allocation fails, or source-file I/O fails.
    pub fn subcatchment_series(
        &mut self,
        selector: OutputElementSelector<'_>,
        attribute: SubcatchmentResultAttribute,
        range: OutputRange,
    ) -> Result<OutputTimeSeries, OutputError> {
        let id = read::resolve_subcatchment_selector(self, selector)?;
        self.read_one_series(OutputSeriesSelection::Subcatchment { id, attribute }, range)
    }

    /// Reads one node result series by exact index or stored name.
    ///
    /// # Arguments
    /// * `selector` - Zero-based node index, or exact stored node name bytes. Name matching is case-sensitive and does not decode the bytes.
    /// * `attribute` - Node result attribute to read.
    /// * `range` - Exact half-open period or Nominal Report Date range to resolve.
    ///
    /// # Returns
    /// `OutputTimeSeries` containing the selected attribute values aligned with the resolved Nominal Report Dates.
    ///
    /// # Errors
    /// Returns `OutputError` when the index is outside the stored node set, the exact name is absent or ambiguous, the attribute is absent or ambiguous in the stored schema, the range is invalid, request or offset arithmetic overflows, allocation fails, or source-file I/O fails.
    pub fn node_series(
        &mut self,
        selector: OutputElementSelector<'_>,
        attribute: NodeResultAttribute,
        range: OutputRange,
    ) -> Result<OutputTimeSeries, OutputError> {
        let id = read::resolve_node_selector(self, selector)?;
        self.read_one_series(OutputSeriesSelection::Node { id, attribute }, range)
    }

    /// Reads one link result series by exact index or stored name.
    ///
    /// # Arguments
    /// * `selector` - Zero-based link index, or exact stored link name bytes. Name matching is case-sensitive and does not decode the bytes.
    /// * `attribute` - Link result attribute to read.
    /// * `range` - Exact half-open period or Nominal Report Date range to resolve.
    ///
    /// # Returns
    /// `OutputTimeSeries` containing the selected attribute values aligned with the resolved Nominal Report Dates.
    ///
    /// # Errors
    /// Returns `OutputError` when the index is outside the stored link set, the exact name is absent or ambiguous, the attribute is absent or ambiguous in the stored schema, the range is invalid, request or offset arithmetic overflows, allocation fails, or source-file I/O fails.
    pub fn link_series(
        &mut self,
        selector: OutputElementSelector<'_>,
        attribute: LinkResultAttribute,
        range: OutputRange,
    ) -> Result<OutputTimeSeries, OutputError> {
        let id = read::resolve_link_selector(self, selector)?;
        self.read_one_series(OutputSeriesSelection::Link { id, attribute }, range)
    }

    /// Reads one system result series without a synthetic element identity.
    ///
    /// # Arguments
    /// * `attribute` - System result attribute to read.
    /// * `range` - Exact half-open period or Nominal Report Date range to resolve.
    ///
    /// # Returns
    /// `OutputTimeSeries` containing the selected system values aligned with the resolved Nominal Report Dates.
    ///
    /// # Errors
    /// Returns `OutputError` when the attribute is absent or ambiguous in the stored schema, the range is invalid, request or offset arithmetic overflows, allocation fails, or source-file I/O fails.
    pub fn system_series(
        &mut self,
        attribute: SystemResultAttribute,
        range: OutputRange,
    ) -> Result<OutputTimeSeries, OutputError> {
        self.read_one_series(OutputSeriesSelection::System { attribute }, range)
    }

    /// Converts one typed result selection into one time series.
    ///
    /// # Arguments
    /// * `selection` - One canonical typed result selection identifying an element and result attribute, or a system result attribute.
    /// * `range` - Exact half-open period or Nominal Report Date range to resolve.
    ///
    /// # Returns
    /// `OutputTimeSeries` containing the selection label, resolved Nominal Report Dates, and selected values.
    ///
    /// # Errors
    /// Returns `OutputError` when the selection or range is invalid, request or offset arithmetic overflows, allocation fails, source-file I/O fails, or the single-series result invariant is violated.
    fn read_one_series(
        &mut self,
        selection: OutputSeriesSelection,
        range: OutputRange,
    ) -> Result<OutputTimeSeries, OutputError> {
        let mut bulk = self.read_bulk_series(std::slice::from_ref(&selection), range)?;
        let series = match bulk.series.pop() {
            Some(series) => series,
            None => return Err(invalid_arithmetic("single series")),
        };
        Ok(OutputTimeSeries {
            selection: series.selection,
            times: bulk.times,
            values: series.values,
        })
    }

    /// Reads exact Stored Report Dates selected through the Nominal Report Date axis.
    ///
    /// # Arguments
    /// * `range` - Exact period or Nominal Report Date query range.
    ///
    /// # Returns
    /// Stored serial dates in ascending resolved period order.
    ///
    /// # Errors
    /// Returns `OutputError` on range validation, allocation, non-finite date, or absolute file I/O failure.
    pub fn read_stored_dates(&mut self, range: OutputRange) -> Result<Vec<SwmmDate>, OutputError> {
        read::dates(self, range)
    }
}

pub(crate) fn read_at(
    source: &mut OutputSource,
    offset: u64,
    bytes: &mut [u8],
    path: &Path,
) -> Result<(), OutputError> {
    if bytes.is_empty() {
        return Ok(());
    }
    match source {
        OutputSource::File(file) => {
            #[cfg(not(unix))]
            {
                file.seek(SeekFrom::Start(offset))
                    .map_err(|source| OutputError::Io {
                        path: path.to_path_buf(),
                        operation: IoOperation::Seek,
                        offset: Some(offset),
                        source,
                    })?;
                file.read_exact(bytes).map_err(|source| OutputError::Io {
                    path: path.to_path_buf(),
                    operation: IoOperation::Read,
                    offset: Some(offset),
                    source,
                })?;
            }
            #[cfg(unix)]
            file.read_exact_at(bytes, offset)
                .map_err(|source| OutputError::Io {
                    path: path.to_path_buf(),
                    operation: IoOperation::Read,
                    offset: Some(offset),
                    source,
                })?;
        }
        OutputSource::Bytes(source_bytes) => {
            let start = usize::try_from(offset).map_err(|_| OutputError::Io {
                path: path.to_path_buf(),
                operation: IoOperation::Read,
                offset: Some(offset),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "output byte offset does not fit the host index type",
                ),
            })?;
            let end = start
                .checked_add(bytes.len())
                .ok_or_else(|| OutputError::Io {
                    path: path.to_path_buf(),
                    operation: IoOperation::Read,
                    offset: Some(offset),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "output byte range overflows the host index type",
                    ),
                })?;
            let available = source_bytes
                .get(start..end)
                .ok_or_else(|| OutputError::Io {
                    path: path.to_path_buf(),
                    operation: IoOperation::Read,
                    offset: Some(offset),
                    source: std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "output byte buffer is shorter than the requested read",
                    ),
                })?;
            bytes.copy_from_slice(available);
        }
    }
    record_physical_read(offset, bytes.len());
    Ok(())
}

pub(crate) fn reserve_vec<T>(
    vec: &mut Vec<T>,
    requested: usize,
    context: AllocationContext,
) -> Result<(), OutputError> {
    vec.try_reserve_exact(requested)
        .map_err(|_| OutputError::AllocationFailed { context, requested })
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct TrackedBytes(Vec<u8>);

#[cfg(test)]
impl std::ops::Deref for TrackedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
impl std::ops::DerefMut for TrackedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
impl From<Vec<u8>> for TrackedBytes {
    fn from(bytes: Vec<u8>) -> Self {
        record_scratch_allocation(bytes.len());
        Self(bytes)
    }
}

#[cfg(test)]
impl Drop for TrackedBytes {
    fn drop(&mut self) {
        record_scratch_release(self.0.len());
    }
}

#[cfg(test)]
pub(crate) type AllocatedBytes = TrackedBytes;

#[cfg(not(test))]
pub(crate) type AllocatedBytes = Vec<u8>;

pub(crate) fn allocate_bytes(
    len: usize,
    context: AllocationContext,
) -> Result<AllocatedBytes, OutputError> {
    let mut bytes = Vec::new();
    reserve_vec(&mut bytes, len, context)?;
    bytes.resize(len, 0);
    #[cfg(test)]
    return Ok(bytes.into());
    #[cfg(not(test))]
    Ok(bytes)
}

pub(crate) fn checked_add(
    left: u64,
    right: u64,
    context: &'static str,
) -> Result<u64, OutputError> {
    left.checked_add(right)
        .ok_or_else(|| invalid_arithmetic(context))
}

pub(crate) fn checked_mul(
    left: u64,
    right: u64,
    context: &'static str,
) -> Result<u64, OutputError> {
    left.checked_mul(right)
        .ok_or_else(|| invalid_arithmetic(context))
}

pub(crate) fn invalid_arithmetic(context: &'static str) -> OutputError {
    OutputError::InvalidFormat(crate::FormatProblem::ArithmeticOverflow { context })
}

pub(crate) fn request_size_overflow(periods: usize, selections: usize) -> OutputError {
    OutputError::ResultSizeOverflow {
        periods,
        selections,
    }
}

pub(crate) fn checked_result_len(periods: usize, selections: usize) -> Result<usize, OutputError> {
    periods
        .checked_mul(selections)
        .ok_or_else(|| request_size_overflow(periods, selections))
}

pub(crate) fn record_allocations(result_len: usize, scratch_len: usize) {
    #[cfg(test)]
    READ_METRICS.with(|metrics| {
        if let Some(metrics) = metrics.borrow_mut().as_mut() {
            metrics.result_len = result_len;
            metrics.scratch_len = scratch_len;
        }
    });
    #[cfg(not(test))]
    let _ = (result_len, scratch_len);
}

pub(crate) fn record_physical_read(offset: u64, bytes: usize) {
    #[cfg(test)]
    READ_METRICS.with(|metrics| {
        if let Some(metrics) = metrics.borrow_mut().as_mut() {
            metrics.calls += 1;
            metrics.bytes += bytes;
            metrics.reads.push((offset, bytes));
        }
    });
    #[cfg(not(test))]
    let _ = (offset, bytes);
}
/// Records one selective-scatter destination inspection for test metrics.
pub(crate) fn record_scatter_inspection() {
    #[cfg(test)]
    READ_METRICS.with(|metrics| {
        if let Some(metrics) = metrics.borrow_mut().as_mut() {
            metrics.scatter_inspections += 1;
        }
    });
}

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReadMetrics {
    pub(crate) calls: usize,
    pub(crate) bytes: usize,
    pub(crate) reads: Vec<(u64, usize)>,
    pub(crate) result_len: usize,
    pub(crate) scratch_len: usize,
    pub(crate) scratch_live: usize,
    pub(crate) scatter_inspections: usize,
}

#[cfg(test)]
thread_local! {
    pub(crate) static READ_METRICS: RefCell<Option<ReadMetrics>> = const { RefCell::new(None) };
}
#[cfg(test)]
fn record_scratch_allocation(bytes: usize) {
    READ_METRICS.with(|metrics| {
        if let Some(metrics) = metrics.borrow_mut().as_mut() {
            metrics.scratch_live = metrics.scratch_live.saturating_add(bytes);
        }
    });
}

#[cfg(test)]
fn record_scratch_release(bytes: usize) {
    READ_METRICS.with(|metrics| {
        if let Some(metrics) = metrics.borrow_mut().as_mut() {
            metrics.scratch_live = metrics.scratch_live.saturating_sub(bytes);
        }
    });
}

#[cfg(test)]
pub(crate) fn begin_metrics() {
    READ_METRICS.with(|metrics| {
        let mut slot = metrics.borrow_mut();
        let scratch_live = slot.as_ref().map_or(0, |metrics| metrics.scratch_live);
        *slot = Some(ReadMetrics {
            scratch_live,
            ..ReadMetrics::default()
        });
    });
}

#[cfg(test)]
pub(crate) fn take_metrics() -> ReadMetrics {
    READ_METRICS.with(|metrics| {
        let mut slot = metrics.borrow_mut();
        let current = slot.take().unwrap_or_default();
        if current.scratch_live != 0 {
            *slot = Some(current.clone());
        }
        current
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_result_size_reports_dimensions() {
        assert!(matches!(
            checked_result_len(usize::MAX, 2),
            Err(OutputError::ResultSizeOverflow {
                periods: usize::MAX,
                selections: 2
            })
        ));
    }

    #[test]
    fn checked_layout_arithmetic_reports_overflow() {
        assert!(matches!(
            checked_add(u64::MAX, 1, "test"),
            Err(OutputError::InvalidFormat(
                crate::FormatProblem::ArithmeticOverflow { context: "test" }
            ))
        ));
        assert!(matches!(
            checked_mul(u64::MAX, 2, "test"),
            Err(OutputError::InvalidFormat(
                crate::FormatProblem::ArithmeticOverflow { context: "test" }
            ))
        ));
    }

    #[test]
    fn allocation_context_is_retained() {
        let error = allocate_bytes(usize::MAX, AllocationContext::SelectiveScratch)
            .expect_err("the deliberately impossible reservation must fail");
        assert!(matches!(
            error,
            OutputError::AllocationFailed {
                context: AllocationContext::SelectiveScratch,
                requested: usize::MAX
            }
        ));
    }
    fn legacy_fixture_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/Model.out")
    }

    fn trailer_start(reader: &OutputReader) -> u64 {
        std::fs::metadata(reader.source_path())
            .expect("legacy fixture metadata must be readable")
            .len()
            - TRAILER_BYTES
    }

    fn selection_word(reader: &OutputReader, selection: OutputSeriesSelection) -> u64 {
        let schema = reader.metadata.result_schema();
        match selection {
            OutputSeriesSelection::Subcatchment { id, attribute } => {
                reader.sub_result_offset
                    + id.index() as u64 * schema.subcatchment().len() as u64
                    + schema
                        .subcatchment()
                        .iter()
                        .position(|candidate| *candidate == attribute)
                        .expect("fixture selection must be in its schema")
                        as u64
            }
            OutputSeriesSelection::Node { id, attribute } => {
                reader.node_result_offset
                    + id.index() as u64 * schema.node().len() as u64
                    + schema
                        .node()
                        .iter()
                        .position(|candidate| *candidate == attribute)
                        .expect("fixture selection must be in its schema")
                        as u64
            }
            OutputSeriesSelection::Link { id, attribute } => {
                reader.link_result_offset
                    + id.index() as u64 * schema.link().len() as u64
                    + schema
                        .link()
                        .iter()
                        .position(|candidate| *candidate == attribute)
                        .expect("fixture selection must be in its schema")
                        as u64
            }
            OutputSeriesSelection::System { attribute } => {
                reader.system_result_offset
                    + schema
                        .system()
                        .iter()
                        .position(|candidate| *candidate == attribute)
                        .expect("fixture selection must be in its schema")
                        as u64
            }
        }
    }

    fn selection_shape(
        reader: &OutputReader,
        selections: &[OutputSeriesSelection],
    ) -> (usize, usize, usize) {
        let mut cells = selections
            .iter()
            .copied()
            .map(|selection| selection_word(reader, selection))
            .collect::<Vec<_>>();
        cells.sort_unstable();
        cells.dedup();
        let unique = cells.len();
        let mut runs = 0;
        let mut largest_run = 0;
        let mut start = 0;
        while start < cells.len() {
            let mut end = start + 1;
            while end < cells.len() && cells[end - 1].checked_add(1) == Some(cells[end]) {
                end += 1;
            }
            runs += 1;
            largest_run = largest_run.max(end - start);
            start = end;
        }
        (unique, runs, largest_run)
    }

    fn selective_selections(reader: &OutputReader) -> [OutputSeriesSelection; 3] {
        let metadata = reader.metadata();
        let sub_schema = metadata.result_schema().subcatchment();
        let node_schema = metadata.result_schema().node();
        assert!(sub_schema.len() >= 2);
        assert!(!node_schema.is_empty());
        [
            OutputSeriesSelection::Subcatchment {
                id: metadata.subcatchments()[0].id(),
                attribute: sub_schema[0],
            },
            OutputSeriesSelection::Subcatchment {
                id: metadata.subcatchments()[0].id(),
                attribute: sub_schema[1],
            },
            OutputSeriesSelection::Node {
                id: metadata.nodes()[0].id(),
                attribute: node_schema[0],
            },
        ]
    }

    fn period_start_for(
        reader: &OutputReader,
        periods: &std::ops::Range<usize>,
        offset: u64,
        end: u64,
    ) -> u64 {
        periods
            .clone()
            .find_map(|period| {
                let start = reader.result_start + period as u64 * reader.bytes_per_period;
                let period_end = start + reader.bytes_per_period;
                (offset >= start && end <= period_end).then_some(start)
            })
            .expect("physical read must fit entirely within one requested period")
    }

    fn assert_period_ranges(
        reader: &OutputReader,
        metrics: &ReadMetrics,
        periods: std::ops::Range<usize>,
    ) {
        let trailer_start = trailer_start(reader);
        for &(offset, bytes) in &metrics.reads {
            let end = offset + bytes as u64;
            let period_start = period_start_for(reader, &periods, offset, end);
            assert!(end <= period_start + reader.bytes_per_period);
            assert!(end <= trailer_start);
        }
    }

    fn assert_payload_ranges(
        reader: &OutputReader,
        metrics: &ReadMetrics,
        periods: std::ops::Range<usize>,
    ) {
        assert_period_ranges(reader, metrics, periods.clone());
        for &(offset, bytes) in &metrics.reads {
            let end = offset + bytes as u64;
            let period_start = period_start_for(reader, &periods, offset, end);
            assert!(offset >= period_start + DATE_BYTES);
        }
    }

    fn assert_date_ranges(
        reader: &OutputReader,
        metrics: &ReadMetrics,
        periods: std::ops::Range<usize>,
    ) {
        assert_period_ranges(reader, metrics, periods.clone());
        for &(offset, bytes) in &metrics.reads {
            let end = offset + bytes as u64;
            let period_start = period_start_for(reader, &periods, offset, end);
            assert!(end <= period_start + DATE_BYTES);
        }
    }

    #[test]
    fn legacy_open_and_selective_metrics_match_physical_plan() {
        let path = legacy_fixture_path();
        begin_metrics();
        let mut reader = OutputReader::open(&path).expect("legacy fixture must open");
        let open_metrics = take_metrics();
        let trailer_start = trailer_start(&reader);
        for &(offset, bytes) in &open_metrics.reads {
            let end = offset + bytes as u64;
            assert!(
                end <= reader.result_start || offset >= trailer_start,
                "open read must not overlap the result payload"
            );
        }

        let periods = 1..4;
        let selections = selective_selections(&reader);
        let (unique, runs, largest_run) = selection_shape(&reader, &selections);
        begin_metrics();
        let values = reader
            .read_bulk_series(
                &selections,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("selective fixture read must succeed");
        let metrics = take_metrics();
        let period_count = periods.len();
        assert_eq!(values.times().len(), period_count);
        assert_eq!(values.series().len(), selections.len());
        assert_eq!(metrics.result_len, period_count * selections.len());
        assert_eq!(metrics.scatter_inspections, period_count * selections.len());
        assert_eq!(metrics.bytes, period_count * unique * FLOAT_BYTES as usize);
        assert_eq!(metrics.calls, period_count * runs);
        assert_eq!(metrics.scratch_len, FLOAT_BYTES as usize * largest_run);
        assert_eq!(metrics.scratch_live, 0);
        assert_payload_ranges(&reader, &metrics, periods.clone());

        let base = [selections[0], selections[2]];
        begin_metrics();
        let base_values = reader
            .read_bulk_series(
                &base,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("base fixture read must succeed");
        let base_metrics = take_metrics();
        let duplicated = [selections[0], selections[0], selections[2], selections[2]];
        begin_metrics();
        let duplicated_values = reader
            .read_bulk_series(
                &duplicated,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("duplicate fixture read must succeed");
        let duplicated_metrics = take_metrics();
        assert_eq!(duplicated_values.times().len(), period_count);
        assert_eq!(duplicated_values.series().len(), duplicated.len());
        assert_eq!(duplicated_metrics.reads, base_metrics.reads);
        assert_eq!(duplicated_metrics.calls, base_metrics.calls);
        assert_eq!(duplicated_metrics.bytes, base_metrics.bytes);
        assert_eq!(duplicated_metrics.scratch_len, base_metrics.scratch_len);
        assert_eq!(base_metrics.scratch_live, 0);
        assert_eq!(duplicated_metrics.scratch_live, 0);
        for period in 0..period_count {
            assert_eq!(
                duplicated_values.series()[0].values()[period].to_bits(),
                base_values.series()[0].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[1].values()[period].to_bits(),
                base_values.series()[0].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[2].values()[period].to_bits(),
                base_values.series()[1].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[3].values()[period].to_bits(),
                base_values.series()[1].values()[period].to_bits()
            );
        }
        assert_payload_ranges(&reader, &duplicated_metrics, periods.clone());

        let small = [selections[0]];
        let (_, _, small_largest_run) = selection_shape(&reader, &small);
        begin_metrics();
        let small_values = reader
            .read_bulk_series(&small, OutputRange::Periods { start: 0, end: 1 })
            .expect("small fixture read must succeed");
        let small_metrics = take_metrics();
        assert_eq!(small_values.times().len(), 1);
        assert_eq!(small_values.series().len(), small.len());
        assert_eq!(small_metrics.result_len, small.len());
        assert_eq!(small_metrics.calls, 1);
        assert_eq!(small_metrics.bytes, FLOAT_BYTES as usize);
        assert_eq!(
            small_metrics.scratch_len,
            FLOAT_BYTES as usize * small_largest_run
        );
        assert!(small_metrics.scratch_len < metrics.scratch_len);
        assert_eq!(small_metrics.scratch_live, 0);
        assert_payload_ranges(&reader, &small_metrics, 0..1);
    }

    #[test]
    fn legacy_by_period_and_dates_metrics_match_physical_plan() {
        let mut reader =
            OutputReader::open(legacy_fixture_path()).expect("legacy fixture must open");
        let periods = 2..5;
        let selections = selective_selections(&reader);
        let period_count = periods.len();

        begin_metrics();
        let values = reader
            .read_bulk_series_by_period(
                &selections,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("by-period fixture read must succeed");
        let metrics = take_metrics();
        assert_eq!(values.times().len(), period_count);
        assert_eq!(values.series().len(), selections.len());
        assert_eq!(metrics.result_len, period_count * selections.len());
        assert_eq!(metrics.bytes, period_count * reader.payload_bytes);
        assert_eq!(metrics.calls, period_count);
        assert_eq!(metrics.scratch_len, reader.payload_bytes);
        assert_eq!(metrics.scratch_live, 0);
        assert_payload_ranges(&reader, &metrics, periods.clone());
        let base = [selections[0], selections[2]];
        begin_metrics();
        let base_values = reader
            .read_bulk_series_by_period(
                &base,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("base by-period fixture read must succeed");
        let base_metrics = take_metrics();
        let duplicated = [selections[0], selections[0], selections[2], selections[2]];
        begin_metrics();
        let duplicated_values = reader
            .read_bulk_series_by_period(
                &duplicated,
                OutputRange::Periods {
                    start: periods.start,
                    end: periods.end,
                },
            )
            .expect("duplicate by-period fixture read must succeed");
        let duplicated_metrics = take_metrics();
        assert_eq!(duplicated_values.times().len(), period_count);
        assert_eq!(duplicated_values.series().len(), duplicated.len());
        assert_eq!(duplicated_metrics.reads, base_metrics.reads);
        assert_eq!(duplicated_metrics.calls, base_metrics.calls);
        assert_eq!(duplicated_metrics.bytes, base_metrics.bytes);
        assert_eq!(duplicated_metrics.scratch_len, base_metrics.scratch_len);
        assert_eq!(base_metrics.scratch_live, 0);
        assert_eq!(duplicated_metrics.scratch_live, 0);
        for period in 0..period_count {
            assert_eq!(
                duplicated_values.series()[0].values()[period].to_bits(),
                base_values.series()[0].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[1].values()[period].to_bits(),
                base_values.series()[0].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[2].values()[period].to_bits(),
                base_values.series()[1].values()[period].to_bits()
            );
            assert_eq!(
                duplicated_values.series()[3].values()[period].to_bits(),
                base_values.series()[1].values()[period].to_bits()
            );
        }
        assert_payload_ranges(&reader, &duplicated_metrics, periods.clone());

        begin_metrics();
        let dates = reader
            .read_stored_dates(OutputRange::Periods {
                start: periods.start,
                end: periods.end,
            })
            .expect("date fixture read must succeed");
        let date_metrics = take_metrics();
        assert_eq!(dates.len(), period_count);
        assert_eq!(date_metrics.result_len, period_count);
        assert_eq!(date_metrics.bytes, period_count * DATE_BYTES as usize);
        assert_eq!(date_metrics.calls, period_count);
        assert_eq!(date_metrics.scratch_len, 0);
        assert_eq!(date_metrics.scratch_live, 0);
        assert_date_ranges(&reader, &date_metrics, periods);
    }
}
