use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    AllocationContext, ConcentrationUnits, FlowUnits, FormatProblem, IoOperation, LinkId, LinkKind,
    LinkMetadata, LinkResultAttribute, MagicLocation, NodeId, NodeKind, NodeMetadata,
    NodeResultAttribute, OutputDateKind, OutputError, OutputMetadata, OutputName,
    OutputResultSchema, OutputSection, PollutantId, PollutantMetadata, ReportTiming, RunStatus,
    SignedField, SubcatchmentId, SubcatchmentMetadata, SubcatchmentResultAttribute, SwmmDate,
    SystemResultAttribute,
};

use super::{HEADER_BYTES, TRAILER_BYTES};
use super::{
    OutputReader, OutputSource, checked_add, checked_mul, invalid_arithmetic, read_at, reserve_vec,
};

/// Opens a SWMM binary output file and detects whether it was finalized.
pub(crate) fn open(path: impl AsRef<Path>) -> Result<OutputReader, OutputError> {
    let supplied_path = path.as_ref().to_path_buf();
    let file = File::open(&supplied_path).map_err(|source| OutputError::Io {
        path: supplied_path.clone(),
        operation: IoOperation::Open,
        offset: None,
        source,
    })?;
    let file_length = file
        .metadata()
        .map_err(|source| OutputError::Io {
            path: supplied_path.clone(),
            operation: IoOperation::FileLength,
            offset: None,
            source,
        })?
        .len();
    open_source(OutputSource::File(file), supplied_path, file_length)
}

/// Opens an owned SWMM binary output byte buffer.
pub(crate) fn open_bytes(bytes: Vec<u8>) -> Result<OutputReader, OutputError> {
    let file_length = bytes.len() as u64;
    open_source(
        OutputSource::Bytes(bytes),
        PathBuf::from("<memory>"),
        file_length,
    )
}

fn open_source(
    mut source: OutputSource,
    supplied_path: PathBuf,
    file_length: u64,
) -> Result<OutputReader, OutputError> {
    if file_length < HEADER_BYTES {
        return Err(OutputError::InvalidFormat(FormatProblem::ShortFile {
            actual: file_length,
            minimum: HEADER_BYTES,
        }));
    }

    let mut header = [0_u8; 28];
    read_at(&mut source, 0, &mut header, &supplied_path)?;
    let magic = i32_at(&header, 0);
    if magic != super::MAGIC {
        return Err(OutputError::InvalidFormat(FormatProblem::BadMagic {
            location: MagicLocation::Header,
            value: magic,
        }));
    }
    let solver_release = i32_at(&header, 4);
    let flow_units = FlowUnits::from_code(i32_at(&header, 8));
    let subcatchment_count_raw = i32_at(&header, 12);
    let node_count_raw = i32_at(&header, 16);
    let link_count_raw = i32_at(&header, 20);
    let pollutant_count_raw = i32_at(&header, 24);

    let trailer = if file_length >= HEADER_BYTES + TRAILER_BYTES {
        let trailer_start = file_length - TRAILER_BYTES;
        let mut trailer = [0_u8; 24];
        read_at(&mut source, trailer_start, &mut trailer, &supplied_path)?;
        (i32_at(&trailer, 20) == super::MAGIC).then_some((trailer_start, trailer))
    } else {
        None
    };
    let is_finalized = trailer.is_some();

    let (
        id_start,
        input_start,
        result_start,
        mut period_count,
        run_status,
        trailer_start,
        subcatchment_count,
        node_count,
        link_count,
        pollutant_count,
    ) = if trailer.is_none() {
        let subcatchment_count =
            checked_count(subcatchment_count_raw, SignedField::SubcatchmentCount)?;
        let node_count = checked_count(node_count_raw, SignedField::NodeCount)?;
        let link_count = checked_count(link_count_raw, SignedField::LinkCount)?;
        let pollutant_count = checked_count(pollutant_count_raw, SignedField::PollutantCount)?;
        let (input_start, result_start) = scan_metadata_layout(
            &mut source,
            file_length,
            subcatchment_count,
            node_count,
            link_count,
            pollutant_count,
            &supplied_path,
        )?;
        (
            HEADER_BYTES,
            input_start,
            result_start,
            0,
            RunStatus::Unfinalized,
            file_length,
            subcatchment_count,
            node_count,
            link_count,
            pollutant_count,
        )
    } else {
        let (trailer_start, trailer) = trailer.expect("checked finalized trailer");
        let id_start_raw = i32_at(&trailer, 0);
        let input_start_raw = i32_at(&trailer, 4);
        let result_start_raw = i32_at(&trailer, 8);
        let period_count_raw = i32_at(&trailer, 12);
        let run_status_code = i32_at(&trailer, 16);
        let id_start = checked_offset(id_start_raw, SignedField::IdentifierOffset)?;
        let input_start = checked_offset(input_start_raw, SignedField::PropertyOffset)?;
        let result_start = checked_offset(result_start_raw, SignedField::ResultOffset)?;
        if period_count_raw < 0 {
            return Err(OutputError::InvalidFormat(
                FormatProblem::InvalidSignedField {
                    field: SignedField::PeriodCount,
                    value: i64::from(period_count_raw),
                },
            ));
        }
        let period_count = usize::try_from(period_count_raw as u32).map_err(|_| {
            OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
                context: "period count",
                value: period_count_raw as u64,
            })
        })?;
        let subcatchment_count =
            checked_count(subcatchment_count_raw, SignedField::SubcatchmentCount)?;
        let node_count = checked_count(node_count_raw, SignedField::NodeCount)?;
        let link_count = checked_count(link_count_raw, SignedField::LinkCount)?;
        let pollutant_count = checked_count(pollutant_count_raw, SignedField::PollutantCount)?;
        let run_status = if run_status_code == 0 {
            RunStatus::Success
        } else {
            RunStatus::Warning(run_status_code)
        };
        (
            id_start,
            input_start,
            result_start,
            period_count,
            run_status,
            trailer_start,
            subcatchment_count,
            node_count,
            link_count,
            pollutant_count,
        )
    };

    if id_start != HEADER_BYTES {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionBoundary {
            section: OutputSection::Identifiers,
            expected: HEADER_BYTES,
            actual: id_start,
        }));
    }
    if input_start < id_start {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionOrder {
            earlier: OutputSection::Identifiers,
            earlier_offset: id_start,
            later: OutputSection::Properties,
            later_offset: input_start,
        }));
    }
    if result_start < input_start {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionOrder {
            earlier: OutputSection::Properties,
            earlier_offset: input_start,
            later: OutputSection::Results,
            later_offset: result_start,
        }));
    }
    if result_start > trailer_start {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionOrder {
            earlier: OutputSection::Results,
            earlier_offset: result_start,
            later: OutputSection::Trailer,
            later_offset: trailer_start,
        }));
    }

    let metadata_len_u64 = result_start
        .checked_sub(id_start)
        .ok_or_else(|| invalid_arithmetic("metadata extent"))?;
    let metadata_len = usize::try_from(metadata_len_u64).map_err(|_| {
        OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
            context: "metadata extent",
            value: metadata_len_u64,
        })
    })?;
    let metadata_bytes = read_region(&mut source, id_start, metadata_len, &supplied_path)?;
    let mut cursor = Cursor::new(&metadata_bytes, id_start);

    let subcatchment_names = read_names(&mut cursor, subcatchment_count, input_start)?;
    let node_names = read_names(&mut cursor, node_count, input_start)?;
    let link_names = read_names(&mut cursor, link_count, input_start)?;
    let pollutant_names = read_names(&mut cursor, pollutant_count, input_start)?;
    let pollutant_units = read_words_i32(
        &mut cursor,
        pollutant_count,
        input_start,
        OutputSection::Identifiers,
    )?;
    if cursor.absolute() != input_start {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionBoundary {
            section: OutputSection::Identifiers,
            expected: input_start,
            actual: cursor.absolute(),
        }));
    }

    let sub_property_codes = read_property_codes(&mut cursor, result_start)?;
    let sub_property_values = read_property_values(
        &mut cursor,
        subcatchment_count,
        sub_property_codes.len(),
        result_start,
    )?;
    let node_property_codes = read_property_codes(&mut cursor, result_start)?;
    let node_property_values = read_property_values(
        &mut cursor,
        node_count,
        node_property_codes.len(),
        result_start,
    )?;
    let link_property_codes = read_property_codes(&mut cursor, result_start)?;
    let link_property_values = read_property_values(
        &mut cursor,
        link_count,
        link_property_codes.len(),
        result_start,
    )?;
    if !property_schema(
        &sub_property_codes,
        &node_property_codes,
        &link_property_codes,
    ) {
        return Err(OutputError::UnsupportedPropertySchema {
            subcatchment: sub_property_codes.into_boxed_slice(),
            node: node_property_codes.into_boxed_slice(),
            link: link_property_codes.into_boxed_slice(),
        });
    }

    let sub_result_codes = read_result_codes(&mut cursor, result_start)?;
    let node_result_codes = read_result_codes(&mut cursor, result_start)?;
    let link_result_codes = read_result_codes(&mut cursor, result_start)?;
    let system_result_codes = read_result_codes(&mut cursor, result_start)?;
    let schedule_origin_bits = cursor.read_u64(result_start, OutputSection::Timing)?;
    let schedule_origin = f64::from_bits(schedule_origin_bits);
    if !schedule_origin.is_finite() {
        return Err(OutputError::InvalidFormat(FormatProblem::NonFiniteDate {
            kind: OutputDateKind::ReportScheduleOrigin,
            period: None,
            bits: schedule_origin_bits,
        }));
    }
    let report_step = cursor.read_i32(result_start, OutputSection::Timing)?;
    if report_step <= 0 {
        return Err(OutputError::InvalidFormat(
            FormatProblem::InvalidReportStep { value: report_step },
        ));
    }
    if cursor.absolute() != result_start {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionBoundary {
            section: OutputSection::Timing,
            expected: result_start,
            actual: cursor.absolute(),
        }));
    }

    let subcatchments = decode_subcatchments(subcatchment_names, &sub_property_values)?;
    let nodes = decode_nodes(node_names, &node_property_values)?;
    let links = decode_links(link_names, &link_property_values)?;
    let pollutants = decode_pollutants(pollutant_names, pollutant_units)?;
    let result_schema = OutputResultSchema {
        subcatchment: decode_subcatchment_attributes(sub_result_codes, pollutants.len())?
            .into_boxed_slice(),
        node: decode_node_attributes(node_result_codes, pollutants.len())?.into_boxed_slice(),
        link: decode_link_attributes(link_result_codes, pollutants.len())?.into_boxed_slice(),
        system: decode_system_attributes(system_result_codes)?.into_boxed_slice(),
    };

    let sub_width = result_schema.subcatchment.len() as u64;
    let node_width = result_schema.node.len() as u64;
    let link_width = result_schema.link.len() as u64;
    let system_width = result_schema.system.len() as u64;
    let sub_result_offset = 0_u64;
    let node_result_offset = checked_mul(
        subcatchment_count as u64,
        sub_width,
        "subcatchment result extent",
    )?;
    let node_extent = checked_mul(node_count as u64, node_width, "node result extent")?;
    let link_result_offset = checked_add(node_result_offset, node_extent, "link result offset")?;
    let link_extent = checked_mul(link_count as u64, link_width, "link result extent")?;
    let system_result_offset =
        checked_add(link_result_offset, link_extent, "system result offset")?;
    let payload_words = checked_add(system_result_offset, system_width, "result payload width")?;
    let payload_bytes_u64 = checked_mul(payload_words, super::FLOAT_BYTES, "result payload bytes")?;
    let payload_bytes = usize::try_from(payload_bytes_u64).map_err(|_| {
        OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
            context: "result payload bytes",
            value: payload_bytes_u64,
        })
    })?;
    let bytes_per_period = checked_add(payload_bytes_u64, super::DATE_BYTES, "period width")?;
    if !is_finalized {
        let result_bytes = file_length
            .checked_sub(result_start)
            .ok_or_else(|| invalid_arithmetic("result extent"))?;
        let count = result_bytes / bytes_per_period;
        period_count = usize::try_from(count).map_err(|_| {
            OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
                context: "period count",
                value: count,
            })
        })?;
    } else {
        let result_bytes = checked_mul(period_count as u64, bytes_per_period, "result extent")?;
        let result_end = checked_add(result_start, result_bytes, "result end")?;
        let expected_file_length = checked_add(result_end, TRAILER_BYTES, "file extent")?;
        if expected_file_length != file_length {
            return Err(OutputError::InvalidFormat(
                FormatProblem::UnexpectedFileLength {
                    expected: expected_file_length,
                    actual: file_length,
                },
            ));
        }
    }

    let metadata = OutputMetadata {
        solver_release,
        run_status,
        flow_units,
        report_timing: ReportTiming {
            report_schedule_origin: SwmmDate(schedule_origin),
            report_step: Duration::from_secs(report_step as u64),
            period_count,
        },
        subcatchments: subcatchments.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        links: links.into_boxed_slice(),
        pollutants: pollutants.into_boxed_slice(),
        result_schema,
    };
    Ok(OutputReader {
        source,
        source_path: supplied_path,
        metadata,
        result_start,
        payload_bytes,
        bytes_per_period,
        sub_result_offset,
        node_result_offset,
        link_result_offset,
        system_result_offset,
    })
}

fn scan_metadata_layout(
    source: &mut OutputSource,
    file_length: u64,
    subcatchment_count: usize,
    node_count: usize,
    link_count: usize,
    pollutant_count: usize,
    path: &Path,
) -> Result<(u64, u64), OutputError> {
    let name_count = subcatchment_count
        .checked_add(node_count)
        .and_then(|count| count.checked_add(link_count))
        .and_then(|count| count.checked_add(pollutant_count))
        .ok_or_else(|| invalid_arithmetic("identifier count"))?;
    let mut cursor = HEADER_BYTES;
    for _ in 0..name_count {
        let length = scan_i32(
            source,
            &mut cursor,
            file_length,
            OutputSection::Identifiers,
            path,
        )?;
        if length < 0 {
            return Err(OutputError::InvalidFormat(
                FormatProblem::InvalidSignedField {
                    field: SignedField::NameLength,
                    value: i64::from(length),
                },
            ));
        }
        cursor = scan_advance(
            cursor,
            length as u64,
            file_length,
            OutputSection::Identifiers,
        )?;
    }
    let pollutant_unit_bytes = checked_mul(
        pollutant_count as u64,
        super::FLOAT_BYTES,
        "pollutant unit extent",
    )?;
    cursor = scan_advance(
        cursor,
        pollutant_unit_bytes,
        file_length,
        OutputSection::Identifiers,
    )?;
    let input_start = cursor;

    for element_count in [subcatchment_count, node_count, link_count] {
        let property_count = scan_count(
            source,
            &mut cursor,
            file_length,
            SignedField::PropertyCount,
            OutputSection::Properties,
            path,
        )?;
        let code_bytes = checked_mul(
            property_count as u64,
            super::FLOAT_BYTES,
            "property code extent",
        )?;
        let value_count = checked_mul(
            element_count as u64,
            property_count as u64,
            "property value count",
        )?;
        let value_bytes = checked_mul(value_count, super::FLOAT_BYTES, "property value extent")?;
        cursor = scan_advance(
            cursor,
            checked_add(code_bytes, value_bytes, "property block extent")?,
            file_length,
            OutputSection::Properties,
        )?;
    }

    for _ in 0..4 {
        let result_count = scan_count(
            source,
            &mut cursor,
            file_length,
            SignedField::ResultVariableCount,
            OutputSection::ResultDescription,
            path,
        )?;
        let result_bytes = checked_mul(
            result_count as u64,
            super::FLOAT_BYTES,
            "result description extent",
        )?;
        cursor = scan_advance(
            cursor,
            result_bytes,
            file_length,
            OutputSection::ResultDescription,
        )?;
    }

    let result_start = scan_advance(cursor, 12, file_length, OutputSection::Timing)?;
    Ok((input_start, result_start))
}

fn scan_count(
    source: &mut OutputSource,
    cursor: &mut u64,
    file_length: u64,
    field: SignedField,
    section: OutputSection,
    path: &Path,
) -> Result<usize, OutputError> {
    checked_count(scan_i32(source, cursor, file_length, section, path)?, field)
}

fn scan_i32(
    source: &mut OutputSource,
    cursor: &mut u64,
    file_length: u64,
    section: OutputSection,
    path: &Path,
) -> Result<i32, OutputError> {
    let next = scan_advance(*cursor, 4, file_length, section)?;
    let mut bytes = [0_u8; 4];
    read_at(source, *cursor, &mut bytes, path)?;
    *cursor = next;
    Ok(i32::from_le_bytes(bytes))
}

fn scan_advance(
    cursor: u64,
    bytes: u64,
    file_length: u64,
    section: OutputSection,
) -> Result<u64, OutputError> {
    let end = checked_add(cursor, bytes, "section extent")?;
    if end > file_length {
        return Err(OutputError::InvalidFormat(FormatProblem::SectionBoundary {
            section,
            expected: end,
            actual: file_length,
        }));
    }
    Ok(end)
}
/// Reads one exact file region into an owned byte buffer.
///
/// # Arguments
/// * `file` - Mutable output file handle.
/// * `offset` - Absolute byte offset at which to begin reading.
/// * `len` - Number of bytes to read.
/// * `path` - Source path reported in I/O errors.
///
/// # Returns
/// The exact bytes stored in the requested region.
///
/// # Errors
/// Returns `OutputError` when allocation or file I/O fails.
fn read_region(
    source: &mut OutputSource,
    offset: u64,
    len: usize,
    path: &Path,
) -> Result<Vec<u8>, OutputError> {
    let mut bytes = Vec::new();
    reserve_vec(&mut bytes, len, AllocationContext::Metadata)?;
    bytes.resize(len, 0);
    read_at(source, offset, &mut bytes, path).map(|()| bytes)
}

/// Converts a nonnegative signed count to the host index type.
///
/// # Arguments
/// * `value` - Signed count read from the output file.
/// * `field` - Metadata field reported when `value` is invalid.
///
/// # Returns
/// The count as `usize`.
///
/// # Errors
/// Returns `OutputError` when `value` is negative or cannot fit in `usize`.
fn checked_count(value: i32, field: SignedField) -> Result<usize, OutputError> {
    if value < 0 {
        return Err(OutputError::InvalidFormat(
            FormatProblem::InvalidSignedField {
                field,
                value: i64::from(value),
            },
        ));
    }
    usize::try_from(value as u32).map_err(|_| {
        OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
            context: "count",
            value: value as u64,
        })
    })
}

/// Converts a nonnegative signed file offset to `u64`.
///
/// # Arguments
/// * `value` - Signed offset read from the output file.
/// * `field` - Metadata field reported when `value` is invalid.
///
/// # Returns
/// The absolute offset as `u64`.
///
/// # Errors
/// Returns `OutputError` when `value` is negative.
fn checked_offset(value: i32, field: SignedField) -> Result<u64, OutputError> {
    if value < 0 {
        return Err(OutputError::InvalidFormat(
            FormatProblem::InvalidSignedField {
                field,
                value: i64::from(value),
            },
        ));
    }
    Ok(value as u64)
}

/// Reads length-prefixed element names without decoding their stored bytes.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `count` - Number of names to read.
/// * `limit` - Absolute end offset of the identifier section.
///
/// # Returns
/// Names in physical file order.
///
/// # Errors
/// Returns `OutputError` when sizing or allocation fails, a name length is invalid, or the identifier section is truncated.
fn read_names(
    cursor: &mut Cursor<'_>,
    count: usize,
    limit: u64,
) -> Result<Vec<OutputName>, OutputError> {
    let minimum_words = count
        .checked_mul(4)
        .ok_or_else(|| invalid_arithmetic("identifier count"))?;
    cursor.ensure_available(minimum_words, limit, OutputSection::Identifiers)?;
    let mut names = Vec::new();
    reserve_vec(&mut names, count, AllocationContext::Metadata)?;
    for _ in 0..count {
        let length = cursor.read_i32(limit, OutputSection::Identifiers)?;
        if length < 0 {
            return Err(OutputError::InvalidFormat(
                FormatProblem::InvalidSignedField {
                    field: SignedField::NameLength,
                    value: i64::from(length),
                },
            ));
        }
        let length = usize::try_from(length as u32).map_err(|_| {
            OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
                context: "name length",
                value: length as u64,
            })
        })?;
        let bytes = cursor.read_bytes(length, limit, OutputSection::Identifiers)?;
        let mut name = Vec::new();
        reserve_vec(&mut name, bytes.len(), AllocationContext::Metadata)?;
        name.extend_from_slice(bytes);
        names.push(OutputName(name.into_boxed_slice()));
    }
    Ok(names)
}

/// Reads little-endian signed 32-bit words from a metadata section.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `count` - Number of words to read.
/// * `limit` - Absolute end offset of the section.
/// * `section` - Section reported when the requested bytes cross its boundary.
///
/// # Returns
/// Signed words in physical file order.
///
/// # Errors
/// Returns `OutputError` when sizing or allocation fails or the section is truncated.
fn read_words_i32(
    cursor: &mut Cursor<'_>,
    count: usize,
    limit: u64,
    section: OutputSection,
) -> Result<Vec<i32>, OutputError> {
    let bytes_len = count
        .checked_mul(4)
        .ok_or_else(|| invalid_arithmetic("word count"))?;
    let bytes = cursor.read_bytes(bytes_len, limit, section)?;
    let mut words = Vec::new();
    reserve_vec(&mut words, count, AllocationContext::Metadata)?;
    for chunk in bytes.chunks_exact(4) {
        words.push(i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(words)
}

/// Reads little-endian 32-bit words while preserving their raw bits.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `count` - Number of words to read.
/// * `limit` - Absolute end offset of the section.
/// * `section` - Section reported when the requested bytes cross its boundary.
///
/// # Returns
/// Raw words in physical file order.
///
/// # Errors
/// Returns `OutputError` when sizing or allocation fails or the section is truncated.
fn read_raw_words(
    cursor: &mut Cursor<'_>,
    count: usize,
    limit: u64,
    section: OutputSection,
) -> Result<Vec<u32>, OutputError> {
    let bytes_len = count
        .checked_mul(4)
        .ok_or_else(|| invalid_arithmetic("property value count"))?;
    let bytes = cursor.read_bytes(bytes_len, limit, section)?;
    let mut words = Vec::new();
    reserve_vec(&mut words, count, AllocationContext::Metadata)?;
    for chunk in bytes.chunks_exact(4) {
        words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(words)
}

/// Reads one result-family property-code vector.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `limit` - Absolute end offset of the properties section.
///
/// # Returns
/// Signed property codes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when the stored count is invalid, allocation fails, or the section is truncated.
fn read_property_codes(cursor: &mut Cursor<'_>, limit: u64) -> Result<Vec<i32>, OutputError> {
    let count = cursor.read_i32(limit, OutputSection::Properties)?;
    if count < 0 {
        return Err(OutputError::InvalidFormat(
            FormatProblem::InvalidSignedField {
                field: SignedField::PropertyCount,
                value: i64::from(count),
            },
        ));
    }
    let count = usize::try_from(count as u32).map_err(|_| {
        OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
            context: "property count",
            value: count as u64,
        })
    })?;
    read_words_i32(cursor, count, limit, OutputSection::Properties)
}

/// Reads the rectangular raw property values for one result family.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `elements` - Number of physical elements in the family.
/// * `properties` - Number of stored properties per element.
/// * `limit` - Absolute end offset of the properties section.
///
/// # Returns
/// Raw property words in element-major physical order.
///
/// # Errors
/// Returns `OutputError` when the value count overflows, allocation fails, or the section is truncated.
fn read_property_values(
    cursor: &mut Cursor<'_>,
    elements: usize,
    properties: usize,
    limit: u64,
) -> Result<Vec<u32>, OutputError> {
    let count = elements
        .checked_mul(properties)
        .ok_or_else(|| invalid_arithmetic("property value count"))?;
    read_raw_words(cursor, count, limit, OutputSection::Properties)
}

/// Reads one result-family attribute-code vector.
///
/// # Arguments
/// * `cursor` - Mutable metadata cursor.
/// * `limit` - Absolute end offset of the result-description section.
///
/// # Returns
/// Signed result codes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when the stored count is invalid, allocation fails, or the section is truncated.
fn read_result_codes(cursor: &mut Cursor<'_>, limit: u64) -> Result<Vec<i32>, OutputError> {
    let count = cursor.read_i32(limit, OutputSection::ResultDescription)?;
    if count < 0 {
        return Err(OutputError::InvalidFormat(
            FormatProblem::InvalidSignedField {
                field: SignedField::ResultVariableCount,
                value: i64::from(count),
            },
        ));
    }
    let count = usize::try_from(count as u32).map_err(|_| {
        OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
            context: "result variable count",
            value: count as u64,
        })
    })?;
    read_words_i32(cursor, count, limit, OutputSection::ResultDescription)
}

/// Reports whether the three physical property schemas are supported.
///
/// # Arguments
/// * `sub` - Subcatchment property codes.
/// * `node` - Node property codes.
/// * `link` - Link property codes.
///
/// # Returns
/// `true` when all three schemas match one supported SWMM layout.
fn property_schema(sub: &[i32], node: &[i32], link: &[i32]) -> bool {
    (sub == [1] && node == [0, 2, 3] && link == [0, 4, 4, 3, 5])
        || (sub == [1] && node == [0, 2, 4] && link == [0, 6, 7, 4, 8])
}
/// Decodes subcatchment names and raw area values into typed metadata.
///
/// # Arguments
/// * `names` - Exact stored subcatchment names in identity order.
/// * `values` - Raw area words in the same order.
///
/// # Returns
/// Typed subcatchment metadata in physical identity order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
///
/// # Panics
/// Panics if `values` contains fewer entries than `names`.
fn decode_subcatchments(
    names: Vec<OutputName>,
    values: &[u32],
) -> Result<Vec<SubcatchmentMetadata>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, names.len(), AllocationContext::Metadata)?;
    for (index, name) in names.into_iter().enumerate() {
        result.push(SubcatchmentMetadata {
            id: SubcatchmentId(index),
            name,
            area: f32::from_bits(values[index]),
        });
    }
    Ok(result)
}

/// Decodes node names and raw property values into typed metadata.
///
/// # Arguments
/// * `names` - Exact stored node names in identity order.
/// * `values` - Raw node kind, invert elevation, and maximum-depth words.
///
/// # Returns
/// Typed node metadata in physical identity order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
///
/// # Panics
/// Panics if `values` does not contain three words per node.
fn decode_nodes(names: Vec<OutputName>, values: &[u32]) -> Result<Vec<NodeMetadata>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, names.len(), AllocationContext::Metadata)?;
    for (index, name) in names.into_iter().enumerate() {
        let start = index * 3;
        result.push(NodeMetadata {
            id: NodeId(index),
            name,
            kind: NodeKind::from_code(values[start] as i32),
            invert_elevation: f32::from_bits(values[start + 1]),
            maximum_depth: f32::from_bits(values[start + 2]),
        });
    }
    Ok(result)
}

/// Decodes link names and raw property values into typed metadata.
///
/// # Arguments
/// * `names` - Exact stored link names in identity order.
/// * `values` - Raw link kind, offsets, maximum-depth, and length words.
///
/// # Returns
/// Typed link metadata in physical identity order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
///
/// # Panics
/// Panics if `values` does not contain five words per link.
fn decode_links(names: Vec<OutputName>, values: &[u32]) -> Result<Vec<LinkMetadata>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, names.len(), AllocationContext::Metadata)?;
    for (index, name) in names.into_iter().enumerate() {
        let start = index * 5;
        result.push(LinkMetadata {
            id: LinkId(index),
            name,
            kind: LinkKind::from_code(values[start] as i32),
            inlet_offset: f32::from_bits(values[start + 1]),
            outlet_offset: f32::from_bits(values[start + 2]),
            maximum_depth: f32::from_bits(values[start + 3]),
            length: f32::from_bits(values[start + 4]),
        });
    }
    Ok(result)
}

/// Decodes pollutant names and concentration-unit codes into typed metadata.
///
/// # Arguments
/// * `names` - Exact stored pollutant names in identity order.
/// * `units` - Signed concentration-unit codes in the same order.
///
/// # Returns
/// Typed pollutant metadata in physical identity order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
///
/// # Panics
/// Panics if `units` contains fewer entries than `names`.
fn decode_pollutants(
    names: Vec<OutputName>,
    units: Vec<i32>,
) -> Result<Vec<PollutantMetadata>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, names.len(), AllocationContext::Metadata)?;
    for (index, name) in names.into_iter().enumerate() {
        result.push(PollutantMetadata {
            id: PollutantId(index),
            name,
            concentration_units: ConcentrationUnits::from_code(units[index]),
        });
    }
    Ok(result)
}

/// Decodes subcatchment result codes without discarding unknown codes.
///
/// # Arguments
/// * `codes` - Signed result codes in physical schema order.
/// * `pollutants` - Number of stored pollutants available to pollutant attributes.
///
/// # Returns
/// Typed subcatchment attributes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
fn decode_subcatchment_attributes(
    codes: Vec<i32>,
    pollutants: usize,
) -> Result<Vec<SubcatchmentResultAttribute>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, codes.len(), AllocationContext::Metadata)?;
    for code in codes {
        result.push(match code {
            0 => SubcatchmentResultAttribute::Rainfall,
            1 => SubcatchmentResultAttribute::SnowDepth,
            2 => SubcatchmentResultAttribute::EvaporationLoss,
            3 => SubcatchmentResultAttribute::InfiltrationLoss,
            4 => SubcatchmentResultAttribute::RunoffFlow,
            5 => SubcatchmentResultAttribute::GroundwaterFlow,
            6 => SubcatchmentResultAttribute::GroundwaterElevation,
            7 => SubcatchmentResultAttribute::SoilMoisture,
            code if code >= 8
                && usize::try_from(code - 8)
                    .ok()
                    .is_some_and(|i| i < pollutants) =>
            {
                SubcatchmentResultAttribute::Pollutant(PollutantId((code - 8) as usize))
            }
            code => SubcatchmentResultAttribute::Unknown(code),
        });
    }
    Ok(result)
}

/// Decodes node result codes without discarding unknown codes.
///
/// # Arguments
/// * `codes` - Signed result codes in physical schema order.
/// * `pollutants` - Number of stored pollutants available to pollutant attributes.
///
/// # Returns
/// Typed node attributes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
fn decode_node_attributes(
    codes: Vec<i32>,
    pollutants: usize,
) -> Result<Vec<NodeResultAttribute>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, codes.len(), AllocationContext::Metadata)?;
    for code in codes {
        result.push(match code {
            0 => NodeResultAttribute::Depth,
            1 => NodeResultAttribute::HydraulicHead,
            2 => NodeResultAttribute::StoredVolume,
            3 => NodeResultAttribute::LateralInflow,
            4 => NodeResultAttribute::TotalInflow,
            5 => NodeResultAttribute::Overflow,
            code if code >= 6
                && usize::try_from(code - 6)
                    .ok()
                    .is_some_and(|i| i < pollutants) =>
            {
                NodeResultAttribute::Pollutant(PollutantId((code - 6) as usize))
            }
            code => NodeResultAttribute::Unknown(code),
        });
    }
    Ok(result)
}

/// Decodes link result codes without discarding unknown codes.
///
/// # Arguments
/// * `codes` - Signed result codes in physical schema order.
/// * `pollutants` - Number of stored pollutants available to pollutant attributes.
///
/// # Returns
/// Typed link attributes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
fn decode_link_attributes(
    codes: Vec<i32>,
    pollutants: usize,
) -> Result<Vec<LinkResultAttribute>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, codes.len(), AllocationContext::Metadata)?;
    for code in codes {
        result.push(match code {
            0 => LinkResultAttribute::Flow,
            1 => LinkResultAttribute::Depth,
            2 => LinkResultAttribute::Velocity,
            3 => LinkResultAttribute::Volume,
            4 => LinkResultAttribute::Capacity,
            code if code >= 5
                && usize::try_from(code - 5)
                    .ok()
                    .is_some_and(|i| i < pollutants) =>
            {
                LinkResultAttribute::Pollutant(PollutantId((code - 5) as usize))
            }
            code => LinkResultAttribute::Unknown(code),
        });
    }
    Ok(result)
}

/// Decodes system result codes without discarding unknown codes.
///
/// # Arguments
/// * `codes` - Signed result codes in physical schema order.
///
/// # Returns
/// Typed system attributes in physical schema order.
///
/// # Errors
/// Returns `OutputError` when result allocation fails.
fn decode_system_attributes(codes: Vec<i32>) -> Result<Vec<SystemResultAttribute>, OutputError> {
    let mut result = Vec::new();
    reserve_vec(&mut result, codes.len(), AllocationContext::Metadata)?;
    for code in codes {
        result.push(match code {
            0 => SystemResultAttribute::AirTemperature,
            1 => SystemResultAttribute::Rainfall,
            2 => SystemResultAttribute::SnowDepth,
            3 => SystemResultAttribute::InfiltrationLoss,
            4 => SystemResultAttribute::RunoffFlow,
            5 => SystemResultAttribute::DryWeatherInflow,
            6 => SystemResultAttribute::GroundwaterInflow,
            7 => SystemResultAttribute::RdiiInflow,
            8 => SystemResultAttribute::ExternalInflow,
            9 => SystemResultAttribute::TotalLateralInflow,
            10 => SystemResultAttribute::FloodingOutflow,
            11 => SystemResultAttribute::OutfallFlow,
            12 => SystemResultAttribute::StorageVolume,
            13 => SystemResultAttribute::Evaporation,
            14 => SystemResultAttribute::PotentialEvapotranspiration,
            code => SystemResultAttribute::Unknown(code),
        });
    }
    Ok(result)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    base: u64,
    position: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a cursor over one in-memory metadata region.
    ///
    /// # Arguments
    /// * `bytes` - Metadata bytes addressed by the cursor.
    /// * `base` - Absolute file offset represented by `bytes[0]`.
    ///
    /// # Returns
    /// A cursor positioned at the start of `bytes`.
    fn new(bytes: &'a [u8], base: u64) -> Self {
        Self {
            bytes,
            base,
            position: 0,
        }
    }

    /// Returns the cursor's absolute file offset.
    ///
    /// # Returns
    /// Absolute byte offset of the current cursor position.
    ///
    /// # Panics
    /// Panics on debug builds if `base + position` exceeds `u64`.
    fn absolute(&self) -> u64 {
        self.base + self.position as u64
    }

    /// Checks that a byte span fits within both the section and buffered metadata.
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes required at the current position.
    /// * `limit` - Absolute end offset of the logical section.
    /// * `section` - Section reported when the span crosses a boundary.
    ///
    /// # Errors
    /// Returns `OutputError` when arithmetic overflows, the limit precedes the buffer, the host cannot represent the boundary, or the span is truncated.
    fn ensure_available(
        &self,
        bytes: usize,
        limit: u64,
        section: OutputSection,
    ) -> Result<(), OutputError> {
        let end = self
            .position
            .checked_add(bytes)
            .ok_or_else(|| invalid_arithmetic("section extent"))?;
        let limit_position_u64 = limit
            .checked_sub(self.base)
            .ok_or_else(|| invalid_arithmetic("section boundary"))?;
        let limit_position = usize::try_from(limit_position_u64).map_err(|_| {
            OutputError::InvalidFormat(FormatProblem::HostSizeOverflow {
                context: "section boundary",
                value: limit_position_u64,
            })
        })?;
        if end > limit_position || end > self.bytes.len() {
            let expected = self
                .base
                .checked_add(end as u64)
                .ok_or_else(|| invalid_arithmetic("section extent"))?;
            let actual_position = limit_position.min(self.bytes.len());
            let actual = self
                .base
                .checked_add(actual_position as u64)
                .ok_or_else(|| invalid_arithmetic("section boundary"))?;
            return Err(OutputError::InvalidFormat(FormatProblem::SectionBoundary {
                section,
                expected,
                actual,
            }));
        }
        Ok(())
    }

    /// Returns bytes at the current position and advances the cursor.
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes to consume.
    /// * `limit` - Absolute end offset of the logical section.
    /// * `section` - Section reported when the requested bytes cross its boundary.
    ///
    /// # Returns
    /// Borrowed bytes from the buffered metadata.
    ///
    /// # Errors
    /// Returns `OutputError` when the requested span is outside the section or buffer.
    fn read_bytes(
        &mut self,
        bytes: usize,
        limit: u64,
        section: OutputSection,
    ) -> Result<&'a [u8], OutputError> {
        self.ensure_available(bytes, limit, section)?;
        let end = self.position + bytes;
        let result = &self.bytes[self.position..end];
        self.position = end;
        Ok(result)
    }

    /// Reads one little-endian signed 32-bit value and advances the cursor.
    ///
    /// # Arguments
    /// * `limit` - Absolute end offset of the logical section.
    /// * `section` - Section reported when four bytes are unavailable.
    ///
    /// # Returns
    /// The decoded signed value.
    ///
    /// # Errors
    /// Returns `OutputError` when four bytes are unavailable.
    fn read_i32(&mut self, limit: u64, section: OutputSection) -> Result<i32, OutputError> {
        let bytes = self.read_bytes(4, limit, section)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Reads one little-endian unsigned 64-bit value and advances the cursor.
    ///
    /// # Arguments
    /// * `limit` - Absolute end offset of the logical section.
    /// * `section` - Section reported when eight bytes are unavailable.
    ///
    /// # Returns
    /// The decoded unsigned value.
    ///
    /// # Errors
    /// Returns `OutputError` when eight bytes are unavailable.
    fn read_u64(&mut self, limit: u64, section: OutputSection) -> Result<u64, OutputError> {
        let bytes = self.read_bytes(8, limit, section)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

/// Decodes a little-endian signed 32-bit value at a byte offset.
///
/// # Arguments
/// * `bytes` - Byte slice containing the encoded value.
/// * `offset` - Zero-based byte offset of the first encoded byte.
///
/// # Returns
/// The decoded signed value.
///
/// # Panics
/// Panics if four bytes are not available at `offset`.
fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_families_require_complete_sequences() {
        assert!(property_schema(&[1], &[0, 2, 3], &[0, 4, 4, 3, 5]));
        assert!(property_schema(&[1], &[0, 2, 4], &[0, 6, 7, 4, 8]));
        assert!(!property_schema(&[1], &[0, 2, 3], &[0, 6, 7, 4, 8]));
    }
}
