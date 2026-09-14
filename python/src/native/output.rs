use std::path::PathBuf;
use std::sync::Mutex;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use swmm_output::{
    BulkSeriesResult, ConcentrationUnits, FlowUnits, LinkKind, LinkResultAttribute, NodeKind,
    NodeResultAttribute, OutputElementType, OutputError, OutputMetadata, OutputRange, OutputReader,
    OutputSeriesSelection, RunStatus, SubcatchmentResultAttribute, SystemResultAttribute,
    UnitSystem,
};

use crate::NativeOutputError;

type RawSelection = (String, usize, i32);

/// Private native owner of one SWMM output reader.
#[pyclass(module = "swmmrs._swmmrs", frozen)]
pub(crate) struct NativeOutputReader {
    reader: Mutex<OutputReader>,
}

#[pymethods]
impl NativeOutputReader {
    /// Opens and validates one finalized or recoverable SWMM binary output file.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `path` - Local output file path.
    ///
    /// # Returns
    /// Native reader owning the opened file.
    ///
    /// # Errors
    /// Returns `NativeOutputError` when opening or validation fails.
    #[new]
    fn new(py: Python<'_>, path: PathBuf) -> PyResult<Self> {
        let reader = py
            .detach(move || OutputReader::open(path))
            .map_err(output_error)?;
        Ok(Self {
            reader: Mutex::new(reader),
        })
    }

    /// Returns all immutable output metadata as private primitive payloads.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dictionary consumed by the typed Python facade.
    ///
    /// # Errors
    /// Returns `NativeOutputError` if the native reader lock is unavailable, or a Python error if dictionary conversion fails.
    fn metadata(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let reader = self.reader.lock().map_err(|_| poisoned_error())?;
        metadata_dict(py, reader.metadata())
    }

    /// Reads selected result series with the selective adjacent-run strategy.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `selections` - Ordered private element-family, element-index, and attribute-code tuples.
    /// * `start` - Inclusive zero-based report period.
    /// * `end` - Exclusive zero-based report period.
    ///
    /// # Returns
    /// Private structured nominal-date and selection-labelled value payload.
    ///
    /// # Errors
    /// Returns `NativeOutputError` for invalid selections, ranges, allocation failure, or file I/O failure.
    fn read_bulk_series(
        &self,
        py: Python<'_>,
        selections: Vec<RawSelection>,
        start: usize,
        end: usize,
    ) -> PyResult<Py<PyDict>> {
        self.read_series(py, selections, start, end, false)
    }

    /// Reads selected result series with one complete payload read per period.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `selections` - Ordered private element-family, element-index, and attribute-code tuples.
    /// * `start` - Inclusive zero-based report period.
    /// * `end` - Exclusive zero-based report period.
    ///
    /// # Returns
    /// Private structured nominal-date and selection-labelled value payload.
    ///
    /// # Errors
    /// Returns `NativeOutputError` for invalid selections, ranges, allocation failure, or file I/O failure.
    fn read_bulk_series_by_period(
        &self,
        py: Python<'_>,
        selections: Vec<RawSelection>,
        start: usize,
        end: usize,
    ) -> PyResult<Py<PyDict>> {
        self.read_series(py, selections, start, end, true)
    }
    /// Reads exact stored report dates over one half-open query range.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `start` - Inclusive zero-based report period.
    /// * `end` - Exclusive zero-based report period.
    ///
    /// # Returns
    /// Exact SWMM serial-day values in ascending period order.
    ///
    /// # Errors
    /// Returns `NativeOutputError` for an invalid range, non-finite date, allocation failure, or file I/O failure.
    fn read_stored_dates(&self, py: Python<'_>, start: usize, end: usize) -> PyResult<Vec<f64>> {
        let result = py.detach(|| {
            let mut reader = self.reader.lock().map_err(|_| BindingFailure::Poisoned)?;
            reader
                .read_stored_dates(OutputRange::Periods { start, end })
                .map(|dates| dates.into_iter().map(|date| date.serial_days()).collect())
                .map_err(BindingFailure::Output)
        });
        result.map_err(binding_error)
    }
}

impl NativeOutputReader {
    /// Executes one bulk strategy after compiling private Python selections.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `raw_selections` - Ordered private selection tuples.
    /// * `start` - Inclusive zero-based report period.
    /// * `end` - Exclusive zero-based report period.
    /// * `by_period` - Whether to use whole-period physical reads.
    ///
    /// # Returns
    /// Detached structured nominal-date and selection-labelled value result.
    ///
    /// # Errors
    /// Returns `NativeOutputError` for invalid selections, ranges, allocation failure, lock failure, or file I/O failure.
    fn read_series(
        &self,
        py: Python<'_>,
        raw_selections: Vec<RawSelection>,
        start: usize,
        end: usize,
        by_period: bool,
    ) -> PyResult<Py<PyDict>> {
        let structured = py.detach(|| {
            let mut reader = self.reader.lock().map_err(|_| BindingFailure::Poisoned)?;
            let selections = compile_selections(reader.metadata(), raw_selections)?;
            let range = OutputRange::Periods { start, end };
            let result = if by_period {
                reader
                    .read_bulk_series_by_period(&selections, range)
                    .map_err(BindingFailure::Output)?
            } else {
                reader
                    .read_bulk_series(&selections, range)
                    .map_err(BindingFailure::Output)?
            };
            Ok::<_, BindingFailure>(result)
        });
        let structured = structured.map_err(binding_error)?;
        bulk_series_dict(py, structured)
    }
}

/// Converts one structured Rust result to a private Python payload.
fn bulk_series_dict(py: Python<'_>, result: BulkSeriesResult) -> PyResult<Py<PyDict>> {
    let payload = PyDict::new(py);
    let times = result
        .times()
        .iter()
        .map(|date| date.serial_days())
        .collect::<Vec<_>>();
    let series = result
        .series()
        .iter()
        .map(|series| {
            let (family, index, code) = selection_payload(series.selection());
            (family, index, code, series.values().to_vec())
        })
        .collect::<Vec<_>>();
    payload.set_item("times", times)?;
    payload.set_item("series", series)?;
    Ok(payload.unbind())
}

/// Encodes one canonical Rust selection for the private Python payload.
fn selection_payload(selection: OutputSeriesSelection) -> (String, usize, i32) {
    match selection {
        OutputSeriesSelection::Subcatchment { id, attribute } => (
            "subcatchment".to_string(),
            id.index(),
            subcatchment_result_code(attribute),
        ),
        OutputSeriesSelection::Node { id, attribute } => {
            ("node".to_string(), id.index(), node_result_code(attribute))
        }
        OutputSeriesSelection::Link { id, attribute } => {
            ("link".to_string(), id.index(), link_result_code(attribute))
        }
        OutputSeriesSelection::System { attribute } => {
            ("system".to_string(), 0, system_result_code(attribute))
        }
    }
}

#[derive(Debug)]
enum BindingFailure {
    Output(OutputError),
    InvalidSelection(String),
    Poisoned,
}

/// Converts one detached binding failure to the private Python exception.
///
/// # Arguments
/// * `failure` - Owned failure produced without the GIL.
///
/// # Returns
/// Private native output exception consumed by the Python facade.
fn binding_error(failure: BindingFailure) -> PyErr {
    match failure {
        BindingFailure::Output(error) => output_error(error),
        BindingFailure::InvalidSelection(message) => {
            NativeOutputError::new_err(("invalid_selection", message))
        }
        BindingFailure::Poisoned => poisoned_error(),
    }
}

/// Creates the private lock-poisoning output exception.
///
/// # Returns
/// Internal native output exception.
fn poisoned_error() -> PyErr {
    NativeOutputError::new_err(("internal", "output reader is unavailable"))
}

/// Converts one structured Rust output failure to the private Python exception.
///
/// # Arguments
/// * `error` - Structured output-reader failure.
///
/// # Returns
/// Private native output exception with a stable category and descriptive message.
fn output_error(error: OutputError) -> PyErr {
    let category = match &error {
        OutputError::Io { .. } => "io",
        OutputError::InvalidFormat(_) => "invalid_format",
        OutputError::UnsupportedPropertySchema { .. } => "unsupported_property_schema",
        OutputError::InvalidPeriodRange { .. } => "invalid_period_range",
        OutputError::InvalidDateRange { .. } => "invalid_date_range",
        OutputError::InvalidElementId { .. } => "invalid_element_id",
        OutputError::AttributeNotFound { .. } => "attribute_not_found",
        OutputError::AmbiguousAttribute { .. } => "ambiguous_attribute",
        OutputError::ElementNameNotFound { .. } => "element_not_found",
        OutputError::AmbiguousElementName { .. } => "ambiguous_element",
        OutputError::ResultSizeOverflow { .. } => "result_size_overflow",
        OutputError::AllocationFailed { .. } => "allocation_failed",
        _ => "output",
    };
    NativeOutputError::new_err((category, error.to_string()))
}

/// Builds the private primitive metadata payload consumed by `swmmrs.output`.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `metadata` - Immutable validated output metadata.
///
/// # Returns
/// Python dictionary containing exact names, codes, properties, timing, and schemas.
/// Pollutant schema entries retain their raw code and pollutant index marker.
///
/// # Errors
/// Returns a Python exception when any dictionary item cannot be converted or installed.
fn metadata_dict(py: Python<'_>, metadata: &OutputMetadata) -> PyResult<Py<PyDict>> {
    let result = PyDict::new(py);
    let timing = metadata.report_timing();
    result.set_item("solver_release", metadata.solver_release())?;
    result.set_item("run_status", run_status_code(metadata.run_status()))?;
    result.set_item("flow_units", flow_units_code(metadata.flow_units()))?;
    result.set_item(
        "unit_system",
        metadata.unit_system().map(|units| match units {
            UnitSystem::Us => "US",
            UnitSystem::Si => "SI",
        }),
    )?;
    result.set_item(
        "report_schedule_origin",
        timing.report_schedule_origin().serial_days(),
    )?;
    result.set_item("report_step_seconds", timing.report_step().as_secs())?;
    result.set_item("period_count", timing.period_count())?;
    result.set_item(
        "subcatchments",
        metadata
            .subcatchments()
            .iter()
            .map(|item| {
                (
                    item.id().index(),
                    pyo3::types::PyBytes::new(py, item.name().as_bytes()).unbind(),
                    item.area(),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "nodes",
        metadata
            .nodes()
            .iter()
            .map(|item| {
                (
                    item.id().index(),
                    pyo3::types::PyBytes::new(py, item.name().as_bytes()).unbind(),
                    node_kind_code(item.kind()),
                    item.invert_elevation(),
                    item.maximum_depth(),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "links",
        metadata
            .links()
            .iter()
            .map(|item| {
                (
                    item.id().index(),
                    pyo3::types::PyBytes::new(py, item.name().as_bytes()).unbind(),
                    link_kind_code(item.kind()),
                    item.inlet_offset(),
                    item.outlet_offset(),
                    item.maximum_depth(),
                    item.length(),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "pollutants",
        metadata
            .pollutants()
            .iter()
            .map(|item| {
                (
                    item.id().index(),
                    pyo3::types::PyBytes::new(py, item.name().as_bytes()).unbind(),
                    concentration_units_code(item.concentration_units()),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    let schema = metadata.result_schema();
    result.set_item(
        "subcatchment_schema",
        schema
            .subcatchment()
            .iter()
            .copied()
            .map(|attribute| {
                let pollutant = match attribute {
                    SubcatchmentResultAttribute::Pollutant(id) => Some(id.index()),
                    _ => None,
                };
                (subcatchment_result_code(attribute), pollutant)
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "node_schema",
        schema
            .node()
            .iter()
            .copied()
            .map(|attribute| {
                let pollutant = match attribute {
                    NodeResultAttribute::Pollutant(id) => Some(id.index()),
                    _ => None,
                };
                (node_result_code(attribute), pollutant)
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "link_schema",
        schema
            .link()
            .iter()
            .copied()
            .map(|attribute| {
                let pollutant = match attribute {
                    LinkResultAttribute::Pollutant(id) => Some(id.index()),
                    _ => None,
                };
                (link_result_code(attribute), pollutant)
            })
            .collect::<Vec<_>>(),
    )?;
    result.set_item(
        "system_schema",
        schema
            .system()
            .iter()
            .copied()
            .map(|attribute| (system_result_code(attribute), Option::<usize>::None))
            .collect::<Vec<_>>(),
    )?;
    Ok(result.unbind())
}

/// Compiles private primitive selections through the validated stored schemas.
///
/// # Arguments
/// * `metadata` - Immutable output metadata owning typed identities and schemas.
/// * `raw_selections` - Ordered private element-family, index, and code tuples.
///
/// # Returns
/// Ordered typed selections accepted by `swmm-output`.
///
/// # Errors
/// Returns a binding failure for an unknown family, invalid system index, or invalid element index.
fn compile_selections(
    metadata: &OutputMetadata,
    raw_selections: Vec<RawSelection>,
) -> Result<Vec<OutputSeriesSelection>, BindingFailure> {
    raw_selections
        .into_iter()
        .map(|(element_type, index, code)| match element_type.as_str() {
            "subcatchment" => {
                let id = metadata
                    .subcatchments()
                    .get(index)
                    .ok_or_else(|| {
                        invalid_element(
                            OutputElementType::Subcatchment,
                            index,
                            metadata.subcatchments().len(),
                        )
                    })?
                    .id();
                let attribute = metadata
                    .result_schema()
                    .subcatchment()
                    .iter()
                    .copied()
                    .find(|attribute| subcatchment_result_code(*attribute) == code)
                    .unwrap_or(SubcatchmentResultAttribute::Unknown(code));
                Ok(OutputSeriesSelection::Subcatchment { id, attribute })
            }
            "node" => {
                let id = metadata
                    .nodes()
                    .get(index)
                    .ok_or_else(|| {
                        invalid_element(OutputElementType::Node, index, metadata.nodes().len())
                    })?
                    .id();
                let attribute = metadata
                    .result_schema()
                    .node()
                    .iter()
                    .copied()
                    .find(|attribute| node_result_code(*attribute) == code)
                    .unwrap_or(NodeResultAttribute::Unknown(code));
                Ok(OutputSeriesSelection::Node { id, attribute })
            }
            "link" => {
                let id = metadata
                    .links()
                    .get(index)
                    .ok_or_else(|| {
                        invalid_element(OutputElementType::Link, index, metadata.links().len())
                    })?
                    .id();
                let attribute = metadata
                    .result_schema()
                    .link()
                    .iter()
                    .copied()
                    .find(|attribute| link_result_code(*attribute) == code)
                    .unwrap_or(LinkResultAttribute::Unknown(code));
                Ok(OutputSeriesSelection::Link { id, attribute })
            }
            "system" if index == 0 => {
                let attribute = metadata
                    .result_schema()
                    .system()
                    .iter()
                    .copied()
                    .find(|attribute| system_result_code(*attribute) == code)
                    .unwrap_or(SystemResultAttribute::Unknown(code));
                Ok(OutputSeriesSelection::System { attribute })
            }
            "system" => Err(BindingFailure::InvalidSelection(
                "system selections must use element index 0".to_string(),
            )),
            _ => Err(BindingFailure::InvalidSelection(format!(
                "unknown output element type {element_type:?}"
            ))),
        })
        .collect()
}

/// Creates one structured invalid-element failure for the native facade.
///
/// # Arguments
/// * `element_type` - Requested output element family.
/// * `index` - Requested zero-based element index.
/// * `count` - Available element count.
///
/// # Returns
/// Binding failure preserving the structured Rust error.
fn invalid_element(element_type: OutputElementType, index: usize, count: usize) -> BindingFailure {
    BindingFailure::Output(OutputError::InvalidElementId {
        element_type,
        index,
        count,
    })
}

/// Returns the stored run-status code when the file was finalized.
fn run_status_code(status: RunStatus) -> Option<i32> {
    match status {
        RunStatus::Unfinalized => None,
        RunStatus::Success => Some(0),
        RunStatus::Warning(code) => Some(code),
    }
}

/// Returns the exact stored flow-unit code.
fn flow_units_code(units: FlowUnits) -> i32 {
    match units {
        FlowUnits::Cfs => 0,
        FlowUnits::Gpm => 1,
        FlowUnits::Mgd => 2,
        FlowUnits::Cms => 3,
        FlowUnits::Lps => 4,
        FlowUnits::Mld => 5,
        FlowUnits::Unknown(code) => code,
    }
}

/// Returns the exact stored concentration-unit code.
fn concentration_units_code(units: ConcentrationUnits) -> i32 {
    match units {
        ConcentrationUnits::MilligramsPerLiter => 0,
        ConcentrationUnits::MicrogramsPerLiter => 1,
        ConcentrationUnits::CountsPerLiter => 2,
        ConcentrationUnits::Unknown(code) => code,
    }
}

/// Returns the exact stored node-kind code.
fn node_kind_code(kind: NodeKind) -> i32 {
    match kind {
        NodeKind::Junction => 0,
        NodeKind::Outfall => 1,
        NodeKind::Storage => 2,
        NodeKind::Divider => 3,
        NodeKind::Unknown(code) => code,
    }
}

/// Returns the exact stored link-kind code.
fn link_kind_code(kind: LinkKind) -> i32 {
    match kind {
        LinkKind::Conduit => 0,
        LinkKind::Pump => 1,
        LinkKind::Orifice => 2,
        LinkKind::Weir => 3,
        LinkKind::Outlet => 4,
        LinkKind::Unknown(code) => code,
    }
}

/// Returns the exact stored subcatchment result code.
fn subcatchment_result_code(attribute: SubcatchmentResultAttribute) -> i32 {
    match attribute {
        SubcatchmentResultAttribute::Rainfall => 0,
        SubcatchmentResultAttribute::SnowDepth => 1,
        SubcatchmentResultAttribute::EvaporationLoss => 2,
        SubcatchmentResultAttribute::InfiltrationLoss => 3,
        SubcatchmentResultAttribute::RunoffFlow => 4,
        SubcatchmentResultAttribute::GroundwaterFlow => 5,
        SubcatchmentResultAttribute::GroundwaterElevation => 6,
        SubcatchmentResultAttribute::SoilMoisture => 7,
        SubcatchmentResultAttribute::Pollutant(id) => 8 + id.index() as i32,
        SubcatchmentResultAttribute::Unknown(code) => code,
    }
}

/// Returns the exact stored node result code.
fn node_result_code(attribute: NodeResultAttribute) -> i32 {
    match attribute {
        NodeResultAttribute::Depth => 0,
        NodeResultAttribute::HydraulicHead => 1,
        NodeResultAttribute::StoredVolume => 2,
        NodeResultAttribute::LateralInflow => 3,
        NodeResultAttribute::TotalInflow => 4,
        NodeResultAttribute::Overflow => 5,
        NodeResultAttribute::Pollutant(id) => 6 + id.index() as i32,
        NodeResultAttribute::Unknown(code) => code,
    }
}

/// Returns the exact stored link result code.
fn link_result_code(attribute: LinkResultAttribute) -> i32 {
    match attribute {
        LinkResultAttribute::Flow => 0,
        LinkResultAttribute::Depth => 1,
        LinkResultAttribute::Velocity => 2,
        LinkResultAttribute::Volume => 3,
        LinkResultAttribute::Capacity => 4,
        LinkResultAttribute::Pollutant(id) => 5 + id.index() as i32,
        LinkResultAttribute::Unknown(code) => code,
    }
}

/// Returns the exact stored system result code.
fn system_result_code(attribute: SystemResultAttribute) -> i32 {
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
