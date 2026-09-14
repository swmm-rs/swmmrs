//! Standalone reader for finalized SWMM output and complete records in unfinalized files.

mod error;
mod metadata;
mod reader;

pub use error::{
    AllocationContext, FormatProblem, IoOperation, MagicLocation, OutputDateKind,
    OutputElementType, OutputError, OutputSection, ResultElementType, SignedField,
};
pub use metadata::{
    BulkSeriesResult, ConcentrationUnits, FlowUnits, LinkId, LinkKind, LinkMetadata,
    LinkResultAttribute, NodeId, NodeKind, NodeMetadata, NodeResultAttribute,
    OutputElementSelector, OutputMetadata, OutputName, OutputRange, OutputResultSchema,
    OutputSeriesSelection, OutputTimeSeries, OutputValueSeries, PollutantId, PollutantMetadata,
    ReportTiming, RunStatus, SubcatchmentId, SubcatchmentMetadata, SubcatchmentResultAttribute,
    SwmmDate, SystemResultAttribute, UnitSystem,
};
pub use reader::OutputReader;
