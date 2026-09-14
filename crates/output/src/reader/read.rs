use std::ops::Range;

use crate::{
    AllocationContext, BulkSeriesResult, FormatProblem, LinkId, LinkResultAttribute, NodeId,
    NodeResultAttribute, OutputElementSelector, OutputElementType, OutputError, OutputMetadata,
    OutputName, OutputRange, OutputResultSchema, OutputSeriesSelection, OutputValueSeries,
    ResultElementType, SubcatchmentId, SubcatchmentResultAttribute, SwmmDate,
    SystemResultAttribute,
};

use super::{
    DATE_BYTES, FLOAT_BYTES, OutputReader, allocate_bytes, checked_add, checked_mul,
    checked_result_len, invalid_arithmetic, read_at, record_allocations, record_scatter_inspection,
    request_size_overflow, reserve_vec,
};

#[derive(Debug)]
struct CompiledRequest {
    cells: Vec<u64>,
    requested_cells: Vec<usize>,
    destinations_by_cell: Vec<usize>,
    runs: Vec<Range<usize>>,
}

/// Reads requested result series through deduplicated adjacent cell runs.
///
/// # Arguments
/// * `reader` - Mutable reader that owns the output file.
/// * `selections` - Ordered result selections; duplicates remain observable.
/// * `range` - Exact period or Nominal Report Date range.
///
/// # Returns
/// Structured series aligned to one ascending nominal date axis.
///
/// # Errors
/// Returns `OutputError` when validation, sizing, allocation, offset arithmetic, or exact file I/O fails.
///
/// # Panics
/// Panics only if an internal compiled-request destination or scratch-buffer invariant is violated.
pub(crate) fn selective(
    reader: &mut OutputReader,
    selections: &[OutputSeriesSelection],
    range: OutputRange,
) -> Result<BulkSeriesResult, OutputError> {
    let (periods, request, mut result, result_len) = prepare(reader, selections, range)?;
    let largest_run = request
        .runs
        .iter()
        .map(|run| run.end - run.start)
        .max()
        .unwrap_or(0);
    let scratch_len = largest_run
        .checked_mul(FLOAT_BYTES as usize)
        .ok_or_else(|| request_size_overflow(periods.len(), selections.len()))?;
    let mut scratch = allocate_bytes(scratch_len, AllocationContext::SelectiveScratch)?;
    record_allocations(result_len, scratch_len);
    if result_len == 0 {
        return Ok(result);
    }

    let period_count = periods.len();
    for (period_index, period) in periods.enumerate() {
        let mut destination_cursor = 0;
        for run in &request.runs {
            let run_words = run.end - run.start;
            let offset = result_payload_offset(reader, period, request.cells[run.start])?;
            let bytes = run_words
                .checked_mul(FLOAT_BYTES as usize)
                .ok_or_else(|| request_size_overflow(period_count, selections.len()))?;
            read_at(
                &mut reader.source,
                offset,
                &mut scratch[..bytes],
                &reader.source_path,
            )?;
            while destination_cursor < request.destinations_by_cell.len() {
                let destination = request.destinations_by_cell[destination_cursor];
                let cell = request.requested_cells[destination];
                if cell >= run.end {
                    break;
                }
                debug_assert!(cell >= run.start);
                record_scatter_inspection();
                let within = (cell - run.start) * FLOAT_BYTES as usize;
                result.series[destination].values[period_index] =
                    f32::from_bits(u32::from_le_bytes([
                        scratch[within],
                        scratch[within + 1],
                        scratch[within + 2],
                        scratch[within + 3],
                    ]));
                destination_cursor += 1;
            }
        }
        debug_assert_eq!(destination_cursor, request.destinations_by_cell.len());
    }
    Ok(result)
}

/// Reads requested result series from one complete payload per selected period.
///
/// # Arguments
/// * `reader` - Mutable reader that owns the output file.
/// * `selections` - Ordered result selections; duplicates remain observable.
/// * `range` - Exact period or Nominal Report Date range.
///
/// # Returns
/// Structured series aligned to one ascending nominal date axis.
///
/// # Errors
/// Returns `OutputError` when validation, sizing, allocation, offset arithmetic, or exact file I/O fails.
///
/// # Panics
/// Panics only if an internal compiled-request cell or payload-buffer invariant is violated.
pub(crate) fn by_period(
    reader: &mut OutputReader,
    selections: &[OutputSeriesSelection],
    range: OutputRange,
) -> Result<BulkSeriesResult, OutputError> {
    let (periods, request, mut result, result_len) = prepare(reader, selections, range)?;
    let scratch_len = if result_len == 0 {
        0
    } else {
        reader.payload_bytes
    };
    let mut scratch = allocate_bytes(scratch_len, AllocationContext::PeriodScratch)?;
    record_allocations(result_len, scratch_len);
    if result_len == 0 {
        return Ok(result);
    }

    let period_count = periods.len();
    for (period_index, period) in periods.enumerate() {
        let offset = period_payload_offset(reader, period)?;
        read_at(
            &mut reader.source,
            offset,
            &mut scratch,
            &reader.source_path,
        )?;
        for destination in 0..selections.len() {
            let word = request.cells[request.requested_cells[destination]];
            let byte_u64 = word
                .checked_mul(FLOAT_BYTES)
                .ok_or_else(|| request_size_overflow(period_count, selections.len()))?;
            let byte = usize::try_from(byte_u64)
                .map_err(|_| request_size_overflow(period_count, selections.len()))?;
            result.series[destination].values[period_index] = f32::from_bits(u32::from_le_bytes([
                scratch[byte],
                scratch[byte + 1],
                scratch[byte + 2],
                scratch[byte + 3],
            ]));
        }
    }
    Ok(result)
}

/// Reads exact Stored Report Dates for a resolved range.
///
/// # Arguments
/// * `reader` - Mutable reader that owns the output file.
/// * `range` - Exact period or Nominal Report Date range.
///
/// # Returns
/// Exact finite stored serial dates in ascending period order.
///
/// # Errors
/// Returns `OutputError` when range resolution, allocation, offset arithmetic, file I/O, or stored-date validation fails.
pub(crate) fn dates(
    reader: &mut OutputReader,
    range: OutputRange,
) -> Result<Vec<SwmmDate>, OutputError> {
    let periods = resolve_range(&reader.metadata, range)?;
    let mut result = allocate_dates(periods.len())?;
    record_allocations(periods.len(), 0);
    let mut bytes = [0_u8; DATE_BYTES as usize];
    for (index, period) in periods.enumerate() {
        let offset = period_record_offset(reader, period)?;
        read_at(&mut reader.source, offset, &mut bytes, &reader.source_path)?;
        let bits = u64::from_le_bytes(bytes);
        let date = f64::from_bits(bits);
        if !date.is_finite() {
            return Err(OutputError::InvalidFormat(FormatProblem::NonFiniteDate {
                kind: crate::OutputDateKind::StoredReportDate,
                period: Some(period),
                bits,
            }));
        }
        result[index] = SwmmDate(date);
    }
    Ok(result)
}
/// Resolves one subcatchment selector against the reader's stored metadata.
///
/// # Arguments
/// * `reader` - Open reader whose subcatchment count and exact stored names are searched.
/// * `selector` - Zero-based index or exact stored subcatchment name bytes. Name matching is case-sensitive and does not decode the bytes.
///
/// # Returns
/// Typed `SubcatchmentId` for the resolved zero-based subcatchment position.
///
/// # Errors
/// Returns `OutputError::InvalidElementId` for an out-of-range index, `OutputError::ElementNameNotFound` for an absent exact name, or `OutputError::AmbiguousElementName` for duplicate exact matches.
pub(crate) fn resolve_subcatchment_selector(
    reader: &OutputReader,
    selector: OutputElementSelector<'_>,
) -> Result<SubcatchmentId, OutputError> {
    let index = resolve_element_selector(
        selector,
        ResultElementType::Subcatchment,
        OutputElementType::Subcatchment,
        reader.metadata.subcatchments().len(),
        reader
            .metadata
            .subcatchments()
            .iter()
            .map(|metadata| metadata.name()),
    )?;
    Ok(SubcatchmentId(index))
}

/// Resolves one node selector against the reader's stored metadata.
///
/// # Arguments
/// * `reader` - Open reader whose node count and exact stored names are searched.
/// * `selector` - Zero-based index or exact stored node name bytes. Name matching is case-sensitive and does not decode the bytes.
///
/// # Returns
/// Typed `NodeId` for the resolved zero-based node position.
///
/// # Errors
/// Returns `OutputError::InvalidElementId` for an out-of-range index, `OutputError::ElementNameNotFound` for an absent exact name, or `OutputError::AmbiguousElementName` for duplicate exact matches.
pub(crate) fn resolve_node_selector(
    reader: &OutputReader,
    selector: OutputElementSelector<'_>,
) -> Result<NodeId, OutputError> {
    let index = resolve_element_selector(
        selector,
        ResultElementType::Node,
        OutputElementType::Node,
        reader.metadata.nodes().len(),
        reader
            .metadata
            .nodes()
            .iter()
            .map(|metadata| metadata.name()),
    )?;
    Ok(NodeId(index))
}

/// Resolves one link selector against the reader's stored metadata.
///
/// # Arguments
/// * `reader` - Open reader whose link count and exact stored names are searched.
/// * `selector` - Zero-based index or exact stored link name bytes. Name matching is case-sensitive and does not decode the bytes.
///
/// # Returns
/// Typed `LinkId` for the resolved zero-based link position.
///
/// # Errors
/// Returns `OutputError::InvalidElementId` for an out-of-range index, `OutputError::ElementNameNotFound` for an absent exact name, or `OutputError::AmbiguousElementName` for duplicate exact matches.
pub(crate) fn resolve_link_selector(
    reader: &OutputReader,
    selector: OutputElementSelector<'_>,
) -> Result<LinkId, OutputError> {
    let index = resolve_element_selector(
        selector,
        ResultElementType::Link,
        OutputElementType::Link,
        reader.metadata.links().len(),
        reader
            .metadata
            .links()
            .iter()
            .map(|metadata| metadata.name()),
    )?;
    Ok(LinkId(index))
}

/// Resolves an element selector to a zero-based position.
///
/// # Arguments
/// * `selector` - Zero-based index or exact stored name bytes. Name matching is case-sensitive and does not decode the bytes.
/// * `result_element` - Result schema family reported in name-resolution errors.
/// * `output_element` - Physical identity family used to validate an index selector.
/// * `count` - Number of valid zero-based identities in the selected family.
/// * `names` - Iterator of exact stored names in identity order; comparisons use each name's raw bytes.
///
/// # Returns
/// Resolved zero-based element position as `usize`.
///
/// # Errors
/// Returns `OutputError::InvalidElementId` when an index is not below `count`, `OutputError::ElementNameNotFound` when no stored name exactly matches, or `OutputError::AmbiguousElementName` when multiple stored names exactly match. Returns an invalid-format arithmetic error if a unique name match cannot produce its index.
fn resolve_element_selector<'selector, 'names, I>(
    selector: OutputElementSelector<'selector>,
    result_element: ResultElementType,
    output_element: OutputElementType,
    count: usize,
    names: I,
) -> Result<usize, OutputError>
where
    I: Iterator<Item = &'names OutputName>,
{
    match selector {
        OutputElementSelector::Index(index) => {
            validate_id(output_element, index, count)?;
            Ok(index)
        }
        OutputElementSelector::Name(name) => {
            let mut found = None;
            let mut occurrences = 0;
            for (index, stored) in names.enumerate() {
                if stored.as_bytes() == name {
                    occurrences += 1;
                    found = Some(index);
                }
            }
            if occurrences == 0 {
                return Err(OutputError::ElementNameNotFound {
                    element_type: result_element,
                    name: name.to_vec().into_boxed_slice(),
                });
            }
            if occurrences == 1 {
                return match found {
                    Some(index) => Ok(index),
                    None => Err(invalid_arithmetic("element name index")),
                };
            }
            Err(OutputError::AmbiguousElementName {
                element_type: result_element,
                name: name.to_vec().into_boxed_slice(),
                occurrences,
            })
        }
    }
}

/// Validates and allocates every component of a bulk request before payload I/O.
///
/// # Arguments
/// * `reader` - Reader providing immutable metadata and layout.
/// * `selections` - Ordered result selections to validate and compile.
/// * `range` - Exact period or Nominal Report Date range.
///
/// # Returns
/// The resolved periods, compiled physical request, initialized result, and total result-cell count.
///
/// # Errors
/// Returns `OutputError` when validation, sizing, allocation, or offset arithmetic fails.
fn prepare(
    reader: &OutputReader,
    selections: &[OutputSeriesSelection],
    range: OutputRange,
) -> Result<(Range<usize>, CompiledRequest, BulkSeriesResult, usize), OutputError> {
    let periods = resolve_range(&reader.metadata, range)?;
    validate_selections(reader, selections)?;
    let result_len = checked_result_len(periods.len(), selections.len())?;
    let request = compile_request(reader, selections)?;
    let result = allocate_result(reader, selections, &periods)?;
    Ok((periods, request, result, result_len))
}

/// Allocates a structured result with nominal dates and zeroed series values.
///
/// # Arguments
/// * `reader` - Reader providing report timing.
/// * `selections` - Ordered canonical labels copied into the result.
/// * `periods` - Resolved half-open report-period span.
///
/// # Returns
/// An initialized structured result preserving both dimensions.
///
/// # Errors
/// Returns `OutputError` when allocation or nominal-date arithmetic fails.
fn allocate_result(
    reader: &OutputReader,
    selections: &[OutputSeriesSelection],
    periods: &Range<usize>,
) -> Result<BulkSeriesResult, OutputError> {
    let mut times = allocate_dates(periods.len())?;
    for (index, period) in periods.clone().enumerate() {
        times[index] = nominal_date(&reader.metadata, period)?;
    }
    let mut series = Vec::new();
    reserve_vec(&mut series, selections.len(), AllocationContext::Result)?;
    for &selection in selections {
        series.push(OutputValueSeries {
            selection,
            values: allocate_values(periods.len())?,
        });
    }
    Ok(BulkSeriesResult { times, series })
}

/// Validates every requested selection against stored identities and schemas.
///
/// # Arguments
/// * `reader` - Reader providing physical identities and result schemas.
/// * `selections` - Ordered selections to validate.
///
/// # Errors
/// Returns `OutputError` when an identity or attribute is invalid, absent, ambiguous, or cannot be represented.
fn validate_selections(
    reader: &OutputReader,
    selections: &[OutputSeriesSelection],
) -> Result<(), OutputError> {
    let schema = reader.metadata.result_schema();
    for &selection in selections {
        selection_offset(reader, selection, schema)?;
    }
    Ok(())
}

/// Compiles selections into unique cells, adjacent runs, and destination order.
///
/// # Arguments
/// * `reader` - Reader providing physical result layout.
/// * `selections` - Ordered selections to compile.
///
/// # Returns
/// A deduplicated physical read plan that preserves every requested destination.
///
/// # Errors
/// Returns `OutputError` when selection validation, allocation, or offset arithmetic fails.
fn compile_request(
    reader: &OutputReader,
    selections: &[OutputSeriesSelection],
) -> Result<CompiledRequest, OutputError> {
    let schema = reader.metadata.result_schema();
    let mut offsets = Vec::new();
    reserve_vec(&mut offsets, selections.len(), AllocationContext::Selection)?;
    for selection in selections {
        offsets.push(selection_offset(reader, *selection, schema)?);
    }
    let mut cells = Vec::new();
    reserve_vec(&mut cells, offsets.len(), AllocationContext::Selection)?;
    cells.extend_from_slice(&offsets);
    cells.sort_unstable();
    cells.dedup();
    let mut requested_cells = Vec::new();
    reserve_vec(
        &mut requested_cells,
        offsets.len(),
        AllocationContext::Selection,
    )?;
    for offset in offsets {
        requested_cells.push(
            cells
                .binary_search(&offset)
                .map_err(|_| invalid_arithmetic("selection cell"))?,
        );
    }
    let mut destinations_by_cell = Vec::new();
    reserve_vec(
        &mut destinations_by_cell,
        requested_cells.len(),
        AllocationContext::Selection,
    )?;
    destinations_by_cell.extend(0..requested_cells.len());
    destinations_by_cell.sort_unstable_by_key(|&destination| requested_cells[destination]);
    let mut runs = Vec::new();
    reserve_vec(&mut runs, cells.len(), AllocationContext::Selection)?;
    if !cells.is_empty() {
        let mut start = 0;
        for index in 1..=cells.len() {
            let adjacent =
                index < cells.len() && cells[index - 1].checked_add(1) == Some(cells[index]);
            if index == cells.len() || !adjacent {
                runs.push(start..index);
                start = index;
            }
        }
    }
    Ok(CompiledRequest {
        cells,
        requested_cells,
        destinations_by_cell,
        runs,
    })
}

/// Resolves one typed selection to its payload word offset.
///
/// # Arguments
/// * `reader` - Reader providing element counts and family offsets.
/// * `selection` - Typed element and result attribute selection.
/// * `schema` - Stored result schemas in physical order.
///
/// # Returns
/// Zero-based 32-bit word offset within one period payload.
///
/// # Errors
/// Returns `OutputError` when the element or attribute is invalid, absent, ambiguous, or offset arithmetic overflows.
fn selection_offset(
    reader: &OutputReader,
    selection: OutputSeriesSelection,
    schema: &OutputResultSchema,
) -> Result<u64, OutputError> {
    match selection {
        OutputSeriesSelection::Subcatchment { id, attribute } => {
            validate_id(
                OutputElementType::Subcatchment,
                id.index(),
                reader.metadata.subcatchments().len(),
            )?;
            let index = resolve_sub_attribute(
                attribute,
                schema.subcatchment(),
                reader.metadata.pollutants().len(),
            )?;
            checked_add(
                reader.sub_result_offset,
                checked_add(
                    checked_mul(
                        id.index() as u64,
                        schema.subcatchment().len() as u64,
                        "subcatchment selection",
                    )?,
                    index as u64,
                    "subcatchment selection",
                )?,
                "subcatchment selection",
            )
        }
        OutputSeriesSelection::Node { id, attribute } => {
            validate_id(
                OutputElementType::Node,
                id.index(),
                reader.metadata.nodes().len(),
            )?;
            let index = resolve_node_attribute(
                attribute,
                schema.node(),
                reader.metadata.pollutants().len(),
            )?;
            checked_add(
                reader.node_result_offset,
                checked_add(
                    checked_mul(
                        id.index() as u64,
                        schema.node().len() as u64,
                        "node selection",
                    )?,
                    index as u64,
                    "node selection",
                )?,
                "node selection",
            )
        }
        OutputSeriesSelection::Link { id, attribute } => {
            validate_id(
                OutputElementType::Link,
                id.index(),
                reader.metadata.links().len(),
            )?;
            let index = resolve_link_attribute(
                attribute,
                schema.link(),
                reader.metadata.pollutants().len(),
            )?;
            checked_add(
                reader.link_result_offset,
                checked_add(
                    checked_mul(
                        id.index() as u64,
                        schema.link().len() as u64,
                        "link selection",
                    )?,
                    index as u64,
                    "link selection",
                )?,
                "link selection",
            )
        }
        OutputSeriesSelection::System { attribute } => {
            let index = resolve_system_attribute(attribute, schema.system())?;
            checked_add(
                reader.system_result_offset,
                index as u64,
                "system selection",
            )
        }
    }
}

/// Resolves a public range to an exact half-open report-period span.
///
/// # Arguments
/// * `metadata` - Output metadata containing the nominal report schedule.
/// * `range` - All periods, exact period bounds, or optional nominal date bounds.
///
/// # Returns
/// Resolved zero-based half-open period span.
///
/// # Errors
/// Returns `OutputError` for invalid period bounds, explicitly inverted date bounds, or nominal-date arithmetic failure.
fn resolve_range(
    metadata: &OutputMetadata,
    range: OutputRange,
) -> Result<Range<usize>, OutputError> {
    let period_count = metadata.report_timing().period_count();
    match range {
        OutputRange::All => Ok(0..period_count),
        OutputRange::Periods { start, end } => {
            validate_period_span(start, end, period_count)?;
            Ok(start..end)
        }
        OutputRange::Dates { start, end } => {
            if let (Some(start), Some(end)) = (start, end)
                && start.serial_days() > end.serial_days()
            {
                return Err(OutputError::InvalidDateRange {
                    start: Some(start),
                    end: Some(end),
                });
            }
            let start_index = start
                .map(|bound| lower_bound(metadata, bound))
                .transpose()?
                .unwrap_or(0);
            let end_index = end
                .map(|bound| lower_bound(metadata, bound))
                .transpose()?
                .unwrap_or(period_count);
            Ok(start_index..end_index)
        }
    }
}

/// Validates an exact half-open report-period span.
///
/// # Arguments
/// * `start` - Inclusive zero-based period offset.
/// * `end` - Exclusive zero-based period offset.
/// * `period_count` - Number of available complete report periods.
///
/// # Errors
/// Returns `OutputError` when `start > end` or `end > period_count`.
fn validate_period_span(start: usize, end: usize, period_count: usize) -> Result<(), OutputError> {
    if start > end || end > period_count {
        return Err(OutputError::InvalidPeriodRange {
            start,
            end,
            period_count,
        });
    }
    Ok(())
}

/// Finds the first nominal report date not less than a date bound.
///
/// # Arguments
/// * `metadata` - Output metadata containing the nominal report schedule.
/// * `bound` - Finite serial-day lower bound.
///
/// # Returns
/// Zero-based insertion position in the nominal date axis.
///
/// # Errors
/// Returns `OutputError` when nominal-date arithmetic fails.
fn lower_bound(metadata: &OutputMetadata, bound: SwmmDate) -> Result<usize, OutputError> {
    let period_count = metadata.report_timing().period_count();
    let mut low = 0;
    let mut high = period_count;
    while low < high {
        let middle = low + (high - low) / 2;
        if nominal_date(metadata, middle)?.serial_days() < bound.serial_days() {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low)
}

/// Computes one exact Nominal Report Date.
///
/// # Arguments
/// * `metadata` - Output metadata containing the report schedule.
/// * `period` - Zero-based report-period offset.
///
/// # Returns
/// The period's finite nominal serial date.
///
/// # Errors
/// Returns `OutputError` when the period is outside the schedule or date arithmetic overflows.
fn nominal_date(metadata: &OutputMetadata, period: usize) -> Result<SwmmDate, OutputError> {
    metadata
        .report_timing()
        .nominal_date(period)
        .ok_or_else(|| invalid_arithmetic("nominal report date"))
}

/// Validates a zero-based physical element identity.
///
/// # Arguments
/// * `element_type` - Physical family reported on failure.
/// * `index` - Requested zero-based identity.
/// * `count` - Number of stored identities in the family.
///
/// # Errors
/// Returns `OutputError` when `index` is not below `count`.
fn validate_id(
    element_type: OutputElementType,
    index: usize,
    count: usize,
) -> Result<(), OutputError> {
    if index >= count {
        return Err(OutputError::InvalidElementId {
            element_type,
            index,
            count,
        });
    }
    Ok(())
}

/// Computes the absolute offset of one period record.
///
/// # Arguments
/// * `reader` - Reader providing the results origin and period width.
/// * `period` - Zero-based report-period offset.
///
/// # Returns
/// Absolute byte offset of the period's Stored Report Date.
///
/// # Errors
/// Returns `OutputError` when offset arithmetic overflows.
fn period_record_offset(reader: &OutputReader, period: usize) -> Result<u64, OutputError> {
    checked_add(
        reader.result_start,
        checked_mul(period as u64, reader.bytes_per_period, "period offset")?,
        "period offset",
    )
}

/// Computes the absolute offset of one period payload.
///
/// # Arguments
/// * `reader` - Reader providing the results origin and period width.
/// * `period` - Zero-based report-period offset.
///
/// # Returns
/// Absolute byte offset immediately after the period's Stored Report Date.
///
/// # Errors
/// Returns `OutputError` when offset arithmetic overflows.
fn period_payload_offset(reader: &OutputReader, period: usize) -> Result<u64, OutputError> {
    checked_add(
        period_record_offset(reader, period)?,
        DATE_BYTES,
        "payload offset",
    )
}

/// Computes the absolute offset of one result word.
///
/// # Arguments
/// * `reader` - Reader providing the results layout.
/// * `period` - Zero-based report-period offset.
/// * `word` - Zero-based word offset within the period payload.
///
/// # Returns
/// Absolute byte offset of the selected 32-bit result value.
///
/// # Errors
/// Returns `OutputError` when offset arithmetic overflows.
fn result_payload_offset(
    reader: &OutputReader,
    period: usize,
    word: u64,
) -> Result<u64, OutputError> {
    checked_add(
        period_payload_offset(reader, period)?,
        checked_mul(word, FLOAT_BYTES, "cell offset")?,
        "cell offset",
    )
}

/// Resolves one subcatchment attribute to its stored schema position.
///
/// # Arguments
/// * `attribute` - Requested typed subcatchment attribute.
/// * `schema` - Stored subcatchment schema in physical order.
/// * `pollutants` - Number of valid pollutant identities.
///
/// # Returns
/// Zero-based attribute position in `schema`.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity is invalid, its code cannot be represented, or the attribute is absent or ambiguous.
fn resolve_sub_attribute(
    attribute: SubcatchmentResultAttribute,
    schema: &[SubcatchmentResultAttribute],
    pollutants: usize,
) -> Result<usize, OutputError> {
    if let SubcatchmentResultAttribute::Pollutant(id) = attribute {
        validate_id(OutputElementType::Pollutant, id.index(), pollutants)?;
    }
    resolve_attribute(
        ResultElementType::Subcatchment,
        attribute_code_sub(attribute)?,
        attribute,
        schema,
    )
}

/// Resolves one node attribute to its stored schema position.
///
/// # Arguments
/// * `attribute` - Requested typed node attribute.
/// * `schema` - Stored node schema in physical order.
/// * `pollutants` - Number of valid pollutant identities.
///
/// # Returns
/// Zero-based attribute position in `schema`.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity is invalid, its code cannot be represented, or the attribute is absent or ambiguous.
fn resolve_node_attribute(
    attribute: NodeResultAttribute,
    schema: &[NodeResultAttribute],
    pollutants: usize,
) -> Result<usize, OutputError> {
    if let NodeResultAttribute::Pollutant(id) = attribute {
        validate_id(OutputElementType::Pollutant, id.index(), pollutants)?;
    }
    resolve_attribute(
        ResultElementType::Node,
        attribute_code_node(attribute)?,
        attribute,
        schema,
    )
}

/// Resolves one link attribute to its stored schema position.
///
/// # Arguments
/// * `attribute` - Requested typed link attribute.
/// * `schema` - Stored link schema in physical order.
/// * `pollutants` - Number of valid pollutant identities.
///
/// # Returns
/// Zero-based attribute position in `schema`.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity is invalid, its code cannot be represented, or the attribute is absent or ambiguous.
fn resolve_link_attribute(
    attribute: LinkResultAttribute,
    schema: &[LinkResultAttribute],
    pollutants: usize,
) -> Result<usize, OutputError> {
    if let LinkResultAttribute::Pollutant(id) = attribute {
        validate_id(OutputElementType::Pollutant, id.index(), pollutants)?;
    }
    resolve_attribute(
        ResultElementType::Link,
        attribute_code_link(attribute)?,
        attribute,
        schema,
    )
}

/// Resolves one system attribute to its stored schema position.
///
/// # Arguments
/// * `attribute` - Requested typed system attribute.
/// * `schema` - Stored system schema in physical order.
///
/// # Returns
/// Zero-based attribute position in `schema`.
///
/// # Errors
/// Returns `OutputError` when the attribute is absent or ambiguous.
fn resolve_system_attribute(
    attribute: SystemResultAttribute,
    schema: &[SystemResultAttribute],
) -> Result<usize, OutputError> {
    resolve_attribute(
        ResultElementType::System,
        attribute_code_system(attribute),
        attribute,
        schema,
    )
}

/// Finds one exact typed attribute in a physical result schema.
///
/// # Arguments
/// * `element_type` - Result family reported on failure.
/// * `code` - Signed result code reported on failure.
/// * `attribute` - Exact typed attribute to match.
/// * `schema` - Stored typed attributes in physical order.
///
/// # Returns
/// Unique zero-based schema position.
///
/// # Errors
/// Returns `OutputError` when no entry matches, multiple entries match, or an internal unique-match invariant is violated.
fn resolve_attribute<T: PartialEq>(
    element_type: ResultElementType,
    code: i32,
    attribute: T,
    schema: &[T],
) -> Result<usize, OutputError> {
    let mut found = None;
    let mut occurrences = 0;
    for (index, stored) in schema.iter().enumerate() {
        if *stored == attribute {
            occurrences += 1;
            found = Some(index);
        }
    }
    if occurrences == 0 {
        return Err(OutputError::AttributeNotFound { element_type, code });
    }
    if occurrences == 1 {
        return match found {
            Some(index) => Ok(index),
            None => Err(invalid_arithmetic("attribute index")),
        };
    }
    Err(OutputError::AmbiguousAttribute {
        element_type,
        code,
        occurrences,
    })
}

/// Allocates a zeroed result-value vector.
///
/// # Arguments
/// * `len` - Number of `f32` values to allocate.
///
/// # Returns
/// A vector containing `len` zero values.
///
/// # Errors
/// Returns `OutputError` when allocation fails.
fn allocate_values(len: usize) -> Result<Vec<f32>, OutputError> {
    let mut values = Vec::new();
    reserve_vec(&mut values, len, AllocationContext::Result)?;
    values.resize(len, 0.0);
    Ok(values)
}

/// Allocates a zeroed serial-date vector.
///
/// # Arguments
/// * `len` - Number of `SwmmDate` values to allocate.
///
/// # Returns
/// A vector containing `len` zero serial dates.
///
/// # Errors
/// Returns `OutputError` when allocation fails.
fn allocate_dates(len: usize) -> Result<Vec<SwmmDate>, OutputError> {
    let mut values = Vec::new();
    reserve_vec(&mut values, len, AllocationContext::Result)?;
    values.resize(len, SwmmDate(0.0));
    Ok(values)
}

/// Converts a subcatchment attribute to its signed stored result code.
///
/// # Arguments
/// * `attribute` - Typed subcatchment attribute.
///
/// # Returns
/// Signed result code used by the binary schema.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity cannot be represented as an `i32` result code.
fn attribute_code_sub(attribute: SubcatchmentResultAttribute) -> Result<i32, OutputError> {
    match attribute {
        SubcatchmentResultAttribute::Rainfall => Ok(0),
        SubcatchmentResultAttribute::SnowDepth => Ok(1),
        SubcatchmentResultAttribute::EvaporationLoss => Ok(2),
        SubcatchmentResultAttribute::InfiltrationLoss => Ok(3),
        SubcatchmentResultAttribute::RunoffFlow => Ok(4),
        SubcatchmentResultAttribute::GroundwaterFlow => Ok(5),
        SubcatchmentResultAttribute::GroundwaterElevation => Ok(6),
        SubcatchmentResultAttribute::SoilMoisture => Ok(7),
        SubcatchmentResultAttribute::Pollutant(id) => i32::try_from(id.index())
            .ok()
            .and_then(|value| value.checked_add(8))
            .ok_or_else(|| request_size_overflow(0, 0)),
        SubcatchmentResultAttribute::Unknown(code) => Ok(code),
    }
}

/// Converts a node attribute to its signed stored result code.
///
/// # Arguments
/// * `attribute` - Typed node attribute.
///
/// # Returns
/// Signed result code used by the binary schema.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity cannot be represented as an `i32` result code.
fn attribute_code_node(attribute: NodeResultAttribute) -> Result<i32, OutputError> {
    match attribute {
        NodeResultAttribute::Depth => Ok(0),
        NodeResultAttribute::HydraulicHead => Ok(1),
        NodeResultAttribute::StoredVolume => Ok(2),
        NodeResultAttribute::LateralInflow => Ok(3),
        NodeResultAttribute::TotalInflow => Ok(4),
        NodeResultAttribute::Overflow => Ok(5),
        NodeResultAttribute::Pollutant(id) => i32::try_from(id.index())
            .ok()
            .and_then(|value| value.checked_add(6))
            .ok_or_else(|| request_size_overflow(0, 0)),
        NodeResultAttribute::Unknown(code) => Ok(code),
    }
}

/// Converts a link attribute to its signed stored result code.
///
/// # Arguments
/// * `attribute` - Typed link attribute.
///
/// # Returns
/// Signed result code used by the binary schema.
///
/// # Errors
/// Returns `OutputError` when a pollutant identity cannot be represented as an `i32` result code.
fn attribute_code_link(attribute: LinkResultAttribute) -> Result<i32, OutputError> {
    match attribute {
        LinkResultAttribute::Flow => Ok(0),
        LinkResultAttribute::Depth => Ok(1),
        LinkResultAttribute::Velocity => Ok(2),
        LinkResultAttribute::Volume => Ok(3),
        LinkResultAttribute::Capacity => Ok(4),
        LinkResultAttribute::Pollutant(id) => i32::try_from(id.index())
            .ok()
            .and_then(|value| value.checked_add(5))
            .ok_or_else(|| request_size_overflow(0, 0)),
        LinkResultAttribute::Unknown(code) => Ok(code),
    }
}

/// Converts a system attribute to its signed stored result code.
///
/// # Arguments
/// * `attribute` - Typed system attribute.
///
/// # Returns
/// Signed result code used by the binary schema.
fn attribute_code_system(attribute: SystemResultAttribute) -> i32 {
    match attribute {
        SystemResultAttribute::AirTemperature => 0,
        SystemResultAttribute::Rainfall => 1,
        SystemResultAttribute::SnowDepth => 2,
        SystemResultAttribute::InfiltrationLoss => 3,
        SystemResultAttribute::RunoffFlow => 4,
        SystemResultAttribute::DryWeatherInflow => 5,
        SystemResultAttribute::GroundwaterInflow => 6,
        SystemResultAttribute::RdiiInflow => 7,
        SystemResultAttribute::ExternalInflow => 8,
        SystemResultAttribute::TotalLateralInflow => 9,
        SystemResultAttribute::FloodingOutflow => 10,
        SystemResultAttribute::OutfallFlow => 11,
        SystemResultAttribute::StorageVolume => 12,
        SystemResultAttribute::Evaporation => 13,
        SystemResultAttribute::PotentialEvapotranspiration => 14,
        SystemResultAttribute::Unknown(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_size_overflow_keeps_dimensions() {
        assert!(matches!(
            checked_result_len(usize::MAX, 2),
            Err(OutputError::ResultSizeOverflow {
                periods: usize::MAX,
                selections: 2
            })
        ));
    }
}
