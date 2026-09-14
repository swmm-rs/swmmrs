use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Physical output element family used in identity validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputElementType {
    /// Subcatchment element.
    Subcatchment,
    /// Node element.
    Node,
    /// Link element.
    Link,
    /// Pollutant identity.
    Pollutant,
}

/// Result element family used in result-attribute errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResultElementType {
    /// Subcatchment result schema.
    Subcatchment,
    /// Node result schema.
    Node,
    /// Link result schema.
    Link,
    /// System result schema.
    System,
}

/// File operation retained with an I/O failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IoOperation {
    /// Opening the supplied source path.
    Open,
    /// Querying source file length.
    FileLength,
    /// Seeking to an absolute source offset.
    Seek,
    /// Reading source bytes.
    Read,
}

/// Location of a required SWMM magic word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MagicLocation {
    /// Opening fixed-header magic.
    Header,
    /// Closing fixed-trailer magic.
    Trailer,
}

/// Serialized signed field rejected during validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignedField {
    /// Subcatchment count.
    SubcatchmentCount,
    /// Node count.
    NodeCount,
    /// Link count.
    LinkCount,
    /// Pollutant count.
    PollutantCount,
    /// Length-prefixed identifier length.
    NameLength,
    /// Stored-property count.
    PropertyCount,
    /// Result-variable count.
    ResultVariableCount,
    /// Identifier section offset.
    IdentifierOffset,
    /// Property section offset.
    PropertyOffset,
    /// Result section offset.
    ResultOffset,
    /// Finalized report-period count.
    PeriodCount,
    /// Whole-second report interval.
    ReportStep,
}

/// Parsed section used in boundary and ordering errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputSection {
    /// Fixed header.
    FixedHeader,
    /// Identifier and pollutant-unit section.
    Identifiers,
    /// Stored-property section.
    Properties,
    /// Result-description section.
    ResultDescription,
    /// Report timing section.
    Timing,
    /// Period result payload section.
    Results,
    /// Fixed trailer.
    Trailer,
}

/// Date category rejected for a non-finite serialized value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputDateKind {
    /// Header report schedule origin.
    ReportScheduleOrigin,
    /// Period record stored report date.
    StoredReportDate,
}

/// Context of a fallible allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AllocationContext {
    /// Eager metadata storage.
    Metadata,
    /// Final returned result vector.
    Result,
    /// Selective exact-run scratch storage.
    SelectiveScratch,
    /// One-period payload scratch storage.
    PeriodScratch,
    /// Selection compilation bookkeeping.
    Selection,
}

/// Detailed invalid binary format reason.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatProblem {
    /// File is shorter than the fixed header.
    ShortFile { actual: u64, minimum: u64 },
    /// Required magic word differs from the SWMM constant.
    BadMagic { location: MagicLocation, value: i32 },
    /// Signed serialized field is negative or otherwise invalid.
    InvalidSignedField { field: SignedField, value: i64 },
    /// Checked layout arithmetic overflowed.
    ArithmeticOverflow { context: &'static str },
    /// A checked conversion to a host-sized value overflowed.
    HostSizeOverflow { context: &'static str, value: u64 },
    /// Declared sections are out of order.
    SectionOrder {
        earlier: OutputSection,
        earlier_offset: u64,
        later: OutputSection,
        later_offset: u64,
    },
    /// Parsed bytes do not end at the declared section boundary.
    SectionBoundary {
        section: OutputSection,
        expected: u64,
        actual: u64,
    },
    /// Computed result extent differs from the actual file length.
    UnexpectedFileLength { expected: u64, actual: u64 },
    /// Report interval is not a positive whole-second value.
    InvalidReportStep { value: i32 },
    /// Date bits decode to a non-finite value.
    NonFiniteDate {
        kind: OutputDateKind,
        period: Option<usize>,
        bits: u64,
    },
}

/// One structured error model shared by opening and result reads.
#[non_exhaustive]
#[derive(Debug)]
pub enum OutputError {
    /// Source I/O failed while opening or reading.
    Io {
        /// Exact supplied source path.
        path: PathBuf,
        /// Physical operation that failed.
        operation: IoOperation,
        /// Known absolute byte offset, when applicable.
        offset: Option<u64>,
        /// Original operating-system error.
        source: io::Error,
    },
    /// Source bytes violate the computable binary layout.
    InvalidFormat(FormatProblem),
    /// Stored properties do not match one coherent known schema family.
    UnsupportedPropertySchema {
        /// Exact subcatchment property code sequence.
        subcatchment: Box<[i32]>,
        /// Exact node property code sequence.
        node: Box<[i32]>,
        /// Exact link property code sequence.
        link: Box<[i32]>,
    },
    /// Requested half-open period range is outside available records.
    InvalidPeriodRange {
        /// Inclusive range start.
        start: usize,
        /// Exclusive range end.
        end: usize,
        /// Available complete period count.
        period_count: usize,
    },
    /// Requested date bounds are explicitly inverted.
    InvalidDateRange {
        /// Optional inclusive Nominal Report Date bound.
        start: Option<crate::SwmmDate>,
        /// Optional exclusive Nominal Report Date bound.
        end: Option<crate::SwmmDate>,
    },
    /// Requested typed identity is outside the opened element set.
    InvalidElementId {
        /// Identity family.
        element_type: OutputElementType,
        /// Requested zero-based index.
        index: usize,
        /// Available identity count.
        count: usize,
    },
    /// Requested result code is absent from its stored schema.
    AttributeNotFound {
        /// Matching result schema family.
        element_type: ResultElementType,
        /// Exact requested code.
        code: i32,
    },
    /// Requested result code occurs more than once in its stored schema.
    AmbiguousAttribute {
        /// Matching result schema family.
        element_type: ResultElementType,
        /// Exact requested code.
        code: i32,
        /// Number of stored occurrences.
        occurrences: usize,
    },
    /// Requested exact element name is absent from its stored family.
    ElementNameNotFound {
        /// Matching result schema family.
        element_type: ResultElementType,
        /// Exact requested raw name bytes.
        name: Box<[u8]>,
    },
    /// Requested exact element name occurs more than once in its stored family.
    AmbiguousElementName {
        /// Matching result schema family.
        element_type: ResultElementType,
        /// Exact requested raw name bytes.
        name: Box<[u8]>,
        /// Number of stored exact matches.
        occurrences: usize,
    },

    /// Result vector length multiplication overflowed.
    ResultSizeOverflow {
        /// Requested period count.
        periods: usize,
        /// Requested selection count.
        selections: usize,
    },
    /// A checked allocation could not be reserved.
    AllocationFailed {
        /// Allocation being attempted.
        context: AllocationContext,
        /// Requested element or byte count.
        requested: usize,
    },
}

impl fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                path,
                operation,
                offset,
                source,
            } => {
                write!(formatter, "{operation:?} failed for {}", path.display())?;
                if let Some(offset) = offset {
                    write!(formatter, " at byte {offset}")?;
                }
                write!(formatter, ": {source}")
            }
            Self::InvalidFormat(problem) => {
                write!(formatter, "invalid SWMM output format: {problem:?}")
            }
            Self::UnsupportedPropertySchema {
                subcatchment,
                node,
                link,
            } => write!(
                formatter,
                "unsupported SWMM property schema (subcatchment={subcatchment:?}, node={node:?}, link={link:?})"
            ),
            Self::InvalidPeriodRange {
                start,
                end,
                period_count,
            } => write!(
                formatter,
                "invalid period range {start}..{end} for {period_count} periods"
            ),
            Self::InvalidDateRange { start, end } => write!(
                formatter,
                "invalid Nominal Report Date range {:?}..{:?}",
                start.map(crate::SwmmDate::serial_days),
                end.map(crate::SwmmDate::serial_days)
            ),
            Self::InvalidElementId {
                element_type,
                index,
                count,
            } => write!(
                formatter,
                "invalid {element_type:?} identity {index} for {count} elements"
            ),
            Self::AttributeNotFound { element_type, code } => {
                write!(
                    formatter,
                    "attribute code {code} is absent from {element_type:?} schema"
                )
            }
            Self::AmbiguousAttribute {
                element_type,
                code,
                occurrences,
            } => write!(
                formatter,
                "attribute code {code} occurs {occurrences} times in {element_type:?} schema"
            ),
            Self::ElementNameNotFound { element_type, name } => write!(
                formatter,
                "element name {name:?} is absent from {element_type:?} family"
            ),
            Self::AmbiguousElementName {
                element_type,
                name,
                occurrences,
            } => write!(
                formatter,
                "element name {name:?} occurs {occurrences} times in {element_type:?} family"
            ),

            Self::ResultSizeOverflow {
                periods,
                selections,
            } => write!(
                formatter,
                "bulk result size overflow for {periods} periods and {selections} selections"
            ),
            Self::AllocationFailed { context, requested } => {
                write!(
                    formatter,
                    "allocation of {requested} {context:?} units failed"
                )
            }
        }
    }
}

impl Error for OutputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_exposes_original_source() {
        let source = io::Error::new(io::ErrorKind::UnexpectedEof, "short");
        let error = OutputError::Io {
            path: PathBuf::from("x.out"),
            operation: IoOperation::Read,
            offset: Some(12),
            source,
        };
        assert!(error.source().is_some());
    }
}
