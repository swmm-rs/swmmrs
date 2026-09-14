//! Browser-facing standalone SWMM binary-output reader.

use js_sys::{Error, Reflect};
use serde::Serialize;
use swmm_output::{
    BulkSeriesResult, ConcentrationUnits, FlowUnits, LinkKind, LinkResultAttribute, NodeKind,
    NodeResultAttribute, OutputElementType, OutputError, OutputMetadata, OutputRange,
    OutputSeriesSelection, RunStatus, SubcatchmentResultAttribute, SystemResultAttribute,
    UnitSystem,
};
use wasm_bindgen::prelude::*;

use super::common::serialize;

use swmm_output::OutputReader as StandaloneOutputReader;

type RawSelection = (String, usize, i32);

/// Standalone reader owning one validated SWMM output byte buffer.
#[wasm_bindgen]
pub struct OutputReader {
    reader: StandaloneOutputReader,
}

#[wasm_bindgen]
impl OutputReader {
    /// Opens and validates one finalized or recoverable output byte buffer.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<OutputReader, JsValue> {
        StandaloneOutputReader::open_bytes(bytes)
            .map(|reader| Self { reader })
            .map_err(|error| output_error("openOutput", error))
    }

    /// Returns immutable output metadata.
    #[wasm_bindgen(js_name = outputMetadata)]
    pub fn output_metadata(&self) -> Result<JsValue, JsValue> {
        serialize(&metadata_payload(self.reader.metadata()))
    }

    /// Reads selected cells through the deduplicated adjacent-run strategy.
    #[wasm_bindgen(js_name = outputReadBulkSeries)]
    pub fn output_read_bulk_series(
        &mut self,
        selections: JsValue,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_series(selections, start, end, false, "outputReadBulkSeries")
    }

    /// Reads selected cells through one complete payload per period.
    #[wasm_bindgen(js_name = outputReadBulkSeriesByPeriod)]
    pub fn output_read_bulk_series_by_period(
        &mut self,
        selections: JsValue,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_series(selections, start, end, true, "outputReadBulkSeriesByPeriod")
    }

    /// Reads exact stored report-date serial values over a period range.
    #[wasm_bindgen(js_name = outputReadStoredDates)]
    pub fn output_read_stored_dates(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        let dates = self
            .reader
            .read_stored_dates(OutputRange::Periods { start, end })
            .map_err(|error| output_error("outputReadStoredDates", error))?
            .into_iter()
            .map(|date| date.serial_days())
            .collect::<Vec<_>>();
        serialize(&dates)
    }

    /// Reads one subcatchment series identified by an already-resolved index/code pair.
    #[wasm_bindgen(js_name = outputSubcatchmentSeries)]
    pub fn output_subcatchment_series(
        &mut self,
        index: usize,
        code: i32,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_one_series(
            "subcatchment",
            index,
            code,
            start,
            end,
            "outputSubcatchmentSeries",
        )
    }

    /// Reads one node series identified by an already-resolved index/code pair.
    #[wasm_bindgen(js_name = outputNodeSeries)]
    pub fn output_node_series(
        &mut self,
        index: usize,
        code: i32,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_one_series("node", index, code, start, end, "outputNodeSeries")
    }

    /// Reads one link series identified by an already-resolved index/code pair.
    #[wasm_bindgen(js_name = outputLinkSeries)]
    pub fn output_link_series(
        &mut self,
        index: usize,
        code: i32,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_one_series("link", index, code, start, end, "outputLinkSeries")
    }

    /// Reads one system series identified by its stored result code.
    #[wasm_bindgen(js_name = outputSystemSeries)]
    pub fn output_system_series(
        &mut self,
        code: i32,
        start: usize,
        end: usize,
    ) -> Result<JsValue, JsValue> {
        self.read_one_series("system", 0, code, start, end, "outputSystemSeries")
    }
}

impl OutputReader {
    fn read_series(
        &mut self,
        raw: JsValue,
        start: usize,
        end: usize,
        by_period: bool,
        operation: &str,
    ) -> Result<JsValue, JsValue> {
        let raw = serde_wasm_bindgen::from_value::<Vec<RawSelection>>(raw)
            .map_err(|error| invalid_selection(operation, error))?;
        let selections =
            compile_selections(self.reader.metadata(), raw).map_err(|error| match error {
                BindingFailure::Output(error) => output_error(operation, error),
                BindingFailure::Invalid(message) => invalid_selection(operation, message),
            })?;
        self.read_compiled_series(selections, start, end, by_period, operation)
    }

    fn read_one_series(
        &mut self,
        family: &str,
        index: usize,
        code: i32,
        start: usize,
        end: usize,
        operation: &str,
    ) -> Result<JsValue, JsValue> {
        let selections = compile_selections(
            self.reader.metadata(),
            vec![(family.to_string(), index, code)],
        )
        .map_err(|error| match error {
            BindingFailure::Output(error) => output_error(operation, error),
            BindingFailure::Invalid(message) => invalid_selection(operation, message),
        })?;
        self.read_compiled_series(selections, start, end, false, operation)
    }

    fn read_compiled_series(
        &mut self,
        selections: Vec<OutputSeriesSelection>,
        start: usize,
        end: usize,
        by_period: bool,
        operation: &str,
    ) -> Result<JsValue, JsValue> {
        let range = OutputRange::Periods { start, end };
        let result = if by_period {
            self.reader
                .read_bulk_series_by_period(&selections, range)
                .map_err(|error| output_error(operation, error))?
        } else {
            self.reader
                .read_bulk_series(&selections, range)
                .map_err(|error| output_error(operation, error))?
        };
        serialize(&bulk_payload(result))
    }
}

#[derive(Debug)]
enum BindingFailure {
    Output(OutputError),
    Invalid(String),
}

fn compile_selections(
    metadata: &OutputMetadata,
    raw: Vec<RawSelection>,
) -> Result<Vec<OutputSeriesSelection>, BindingFailure> {
    raw.into_iter()
        .map(|(family, index, code)| match family.as_str() {
            "subcatchment" => {
                let id = metadata
                    .subcatchments()
                    .get(index)
                    .ok_or_else(|| {
                        BindingFailure::Output(OutputError::InvalidElementId {
                            element_type: OutputElementType::Subcatchment,
                            index,
                            count: metadata.subcatchments().len(),
                        })
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
                        BindingFailure::Output(OutputError::InvalidElementId {
                            element_type: OutputElementType::Node,
                            index,
                            count: metadata.nodes().len(),
                        })
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
                        BindingFailure::Output(OutputError::InvalidElementId {
                            element_type: OutputElementType::Link,
                            index,
                            count: metadata.links().len(),
                        })
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
            "system" => Err(BindingFailure::Invalid(
                "system selections must use element index 0".to_string(),
            )),
            _ => Err(BindingFailure::Invalid(format!(
                "unknown output element type {family:?}"
            ))),
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BulkPayload {
    times: Vec<f64>,
    series: Vec<SeriesPayload>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeriesPayload {
    selection: SelectionPayload,
    values: Vec<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionPayload {
    element_type: &'static str,
    element: Option<usize>,
    attribute: AttributePayload,
}

fn bulk_payload(result: BulkSeriesResult) -> BulkPayload {
    BulkPayload {
        times: result
            .times()
            .iter()
            .map(|date| date.serial_days())
            .collect(),
        series: result
            .series()
            .iter()
            .map(|series| SeriesPayload {
                selection: selection_payload(series.selection()),
                values: series.values().to_vec(),
            })
            .collect(),
    }
}

fn selection_payload(selection: OutputSeriesSelection) -> SelectionPayload {
    match selection {
        OutputSeriesSelection::Subcatchment { id, attribute } => SelectionPayload {
            element_type: "subcatchment",
            element: Some(id.index()),
            attribute: subcatchment_attribute_payload(attribute),
        },
        OutputSeriesSelection::Node { id, attribute } => SelectionPayload {
            element_type: "node",
            element: Some(id.index()),
            attribute: node_attribute_payload(attribute),
        },
        OutputSeriesSelection::Link { id, attribute } => SelectionPayload {
            element_type: "link",
            element: Some(id.index()),
            attribute: link_attribute_payload(attribute),
        },
        OutputSeriesSelection::System { attribute } => SelectionPayload {
            element_type: "system",
            element: None,
            attribute: system_attribute_payload(attribute),
        },
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum AttributePayload {
    Known(&'static str),
    Pollutant(PollutantPayload),
    Unknown(UnknownCodePayload),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PollutantPayload {
    pollutant: usize,
}

#[derive(Serialize)]
struct UnknownCodePayload {
    code: i32,
}

fn subcatchment_attribute_payload(attribute: SubcatchmentResultAttribute) -> AttributePayload {
    match attribute {
        SubcatchmentResultAttribute::Rainfall => AttributePayload::Known("rainfall"),
        SubcatchmentResultAttribute::SnowDepth => AttributePayload::Known("snow_depth"),
        SubcatchmentResultAttribute::EvaporationLoss => AttributePayload::Known("evap_loss"),
        SubcatchmentResultAttribute::InfiltrationLoss => AttributePayload::Known("infil_loss"),
        SubcatchmentResultAttribute::RunoffFlow => AttributePayload::Known("runoff_rate"),
        SubcatchmentResultAttribute::GroundwaterFlow => AttributePayload::Known("gw_outflow_rate"),
        SubcatchmentResultAttribute::GroundwaterElevation => {
            AttributePayload::Known("gw_table_elev")
        }
        SubcatchmentResultAttribute::SoilMoisture => AttributePayload::Known("soil_moisture"),
        SubcatchmentResultAttribute::Pollutant(id) => {
            AttributePayload::Pollutant(PollutantPayload {
                pollutant: id.index(),
            })
        }
        SubcatchmentResultAttribute::Unknown(code) => {
            AttributePayload::Unknown(UnknownCodePayload { code })
        }
    }
}

fn node_attribute_payload(attribute: NodeResultAttribute) -> AttributePayload {
    match attribute {
        NodeResultAttribute::Depth => AttributePayload::Known("invert_depth"),
        NodeResultAttribute::HydraulicHead => AttributePayload::Known("hydraulic_head"),
        NodeResultAttribute::StoredVolume => AttributePayload::Known("ponded_volume"),
        NodeResultAttribute::LateralInflow => AttributePayload::Known("lateral_inflow"),
        NodeResultAttribute::TotalInflow => AttributePayload::Known("total_inflow"),
        NodeResultAttribute::Overflow => AttributePayload::Known("flooding_losses"),
        NodeResultAttribute::Pollutant(id) => AttributePayload::Pollutant(PollutantPayload {
            pollutant: id.index(),
        }),
        NodeResultAttribute::Unknown(code) => {
            AttributePayload::Unknown(UnknownCodePayload { code })
        }
    }
}

fn link_attribute_payload(attribute: LinkResultAttribute) -> AttributePayload {
    match attribute {
        LinkResultAttribute::Flow => AttributePayload::Known("flow_rate"),
        LinkResultAttribute::Depth => AttributePayload::Known("flow_depth"),
        LinkResultAttribute::Velocity => AttributePayload::Known("flow_velocity"),
        LinkResultAttribute::Volume => AttributePayload::Known("flow_volume"),
        LinkResultAttribute::Capacity => AttributePayload::Known("capacity"),
        LinkResultAttribute::Pollutant(id) => AttributePayload::Pollutant(PollutantPayload {
            pollutant: id.index(),
        }),
        LinkResultAttribute::Unknown(code) => {
            AttributePayload::Unknown(UnknownCodePayload { code })
        }
    }
}

fn system_attribute_payload(attribute: SystemResultAttribute) -> AttributePayload {
    match attribute {
        SystemResultAttribute::AirTemperature => AttributePayload::Known("air_temp"),
        SystemResultAttribute::Rainfall => AttributePayload::Known("rainfall"),
        SystemResultAttribute::SnowDepth => AttributePayload::Known("snow_depth"),
        SystemResultAttribute::InfiltrationLoss => AttributePayload::Known("evap_infil_loss"),
        SystemResultAttribute::RunoffFlow => AttributePayload::Known("runoff_flow"),
        SystemResultAttribute::DryWeatherInflow => AttributePayload::Known("dry_weather_inflow"),
        SystemResultAttribute::GroundwaterInflow => AttributePayload::Known("gw_inflow"),
        SystemResultAttribute::RdiiInflow => AttributePayload::Known("rdii_inflow"),
        SystemResultAttribute::ExternalInflow => AttributePayload::Known("direct_inflow"),
        SystemResultAttribute::TotalLateralInflow => {
            AttributePayload::Known("total_lateral_inflow")
        }
        SystemResultAttribute::FloodingOutflow => AttributePayload::Known("flood_losses"),
        SystemResultAttribute::OutfallFlow => AttributePayload::Known("outfall_flows"),
        SystemResultAttribute::StorageVolume => AttributePayload::Known("volume_stored"),
        SystemResultAttribute::Evaporation => AttributePayload::Known("evap_rate"),
        SystemResultAttribute::PotentialEvapotranspiration => {
            AttributePayload::Known("ptnl_evap_rate")
        }
        SystemResultAttribute::Unknown(code) => {
            AttributePayload::Unknown(UnknownCodePayload { code })
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataPayload {
    solver_release: i32,
    run_status: RunStatusPayload,
    flow_units: FlowUnitsPayload,
    unit_system: Option<&'static str>,
    report_timing: ReportTimingPayload,
    subcatchments: Vec<SubcatchmentPayload>,
    nodes: Vec<NodePayload>,
    links: Vec<LinkPayload>,
    pollutants: Vec<PollutantMetadataPayload>,
    result_schema: ResultSchemaPayload,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunStatusPayload {
    code: Option<i32>,
    is_finalized: bool,
    is_success: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportTimingPayload {
    report_schedule_origin: f64,
    report_step_seconds: u64,
    period_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NamePayload {
    raw: Vec<u8>,
    text: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentPayload {
    index: usize,
    name: NamePayload,
    area: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodePayload {
    index: usize,
    name: NamePayload,
    kind: CategoryPayload,
    invert_elevation: f32,
    maximum_depth: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkPayload {
    index: usize,
    name: NamePayload,
    kind: CategoryPayload,
    inlet_offset: f32,
    outlet_offset: f32,
    maximum_depth: f32,
    length: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PollutantMetadataPayload {
    index: usize,
    name: NamePayload,
    concentration_units: CategoryPayload,
}

#[derive(Serialize)]
#[serde(untagged)]
enum CategoryPayload {
    Known(&'static str),
    Unknown(UnknownCodePayload),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultSchemaPayload {
    subcatchment: Vec<AttributePayload>,
    node: Vec<AttributePayload>,
    link: Vec<AttributePayload>,
    system: Vec<AttributePayload>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum FlowUnitsPayload {
    Known(&'static str),
    Unknown(UnknownCodePayload),
}

fn metadata_payload(metadata: &OutputMetadata) -> MetadataPayload {
    let timing = metadata.report_timing();
    let status = run_status_payload(metadata.run_status());
    MetadataPayload {
        solver_release: metadata.solver_release(),
        run_status: status,
        flow_units: flow_units_payload(metadata.flow_units()),
        unit_system: metadata.unit_system().map(unit_system_name),
        report_timing: ReportTimingPayload {
            report_schedule_origin: timing.report_schedule_origin().serial_days(),
            report_step_seconds: timing.report_step().as_secs(),
            period_count: timing.period_count(),
        },
        subcatchments: metadata
            .subcatchments()
            .iter()
            .map(|item| SubcatchmentPayload {
                index: item.id().index(),
                name: name_payload(item.name().as_bytes()),
                area: item.area(),
            })
            .collect(),
        nodes: metadata
            .nodes()
            .iter()
            .map(|item| NodePayload {
                index: item.id().index(),
                name: name_payload(item.name().as_bytes()),
                kind: node_kind_payload(item.kind()),
                invert_elevation: item.invert_elevation(),
                maximum_depth: item.maximum_depth(),
            })
            .collect(),
        links: metadata
            .links()
            .iter()
            .map(|item| LinkPayload {
                index: item.id().index(),
                name: name_payload(item.name().as_bytes()),
                kind: link_kind_payload(item.kind()),
                inlet_offset: item.inlet_offset(),
                outlet_offset: item.outlet_offset(),
                maximum_depth: item.maximum_depth(),
                length: item.length(),
            })
            .collect(),
        pollutants: metadata
            .pollutants()
            .iter()
            .map(|item| PollutantMetadataPayload {
                index: item.id().index(),
                name: name_payload(item.name().as_bytes()),
                concentration_units: concentration_units_payload(item.concentration_units()),
            })
            .collect(),
        result_schema: ResultSchemaPayload {
            subcatchment: metadata
                .result_schema()
                .subcatchment()
                .iter()
                .copied()
                .map(subcatchment_attribute_payload)
                .collect(),
            node: metadata
                .result_schema()
                .node()
                .iter()
                .copied()
                .map(node_attribute_payload)
                .collect(),
            link: metadata
                .result_schema()
                .link()
                .iter()
                .copied()
                .map(link_attribute_payload)
                .collect(),
            system: metadata
                .result_schema()
                .system()
                .iter()
                .copied()
                .map(system_attribute_payload)
                .collect(),
        },
    }
}

fn name_payload(bytes: &[u8]) -> NamePayload {
    NamePayload {
        raw: bytes.to_vec(),
        text: std::str::from_utf8(bytes).ok().map(str::to_owned),
    }
}

fn run_status_payload(status: RunStatus) -> RunStatusPayload {
    let code = match status {
        RunStatus::Unfinalized => None,
        RunStatus::Success => Some(0),
        RunStatus::Warning(code) => Some(code),
    };
    RunStatusPayload {
        code,
        is_finalized: code.is_some(),
        is_success: code == Some(0),
    }
}

fn flow_units_payload(units: FlowUnits) -> FlowUnitsPayload {
    match units {
        FlowUnits::Cfs => FlowUnitsPayload::Known("cfs"),
        FlowUnits::Gpm => FlowUnitsPayload::Known("gpm"),
        FlowUnits::Mgd => FlowUnitsPayload::Known("mgd"),
        FlowUnits::Cms => FlowUnitsPayload::Known("cms"),
        FlowUnits::Lps => FlowUnitsPayload::Known("lps"),
        FlowUnits::Mld => FlowUnitsPayload::Known("mld"),
        FlowUnits::Unknown(code) => FlowUnitsPayload::Unknown(UnknownCodePayload { code }),
    }
}

fn unit_system_name(units: UnitSystem) -> &'static str {
    match units {
        UnitSystem::Us => "us",
        UnitSystem::Si => "si",
    }
}

fn node_kind_payload(kind: NodeKind) -> CategoryPayload {
    match kind {
        NodeKind::Junction => CategoryPayload::Known("Junction"),
        NodeKind::Outfall => CategoryPayload::Known("Outfall"),
        NodeKind::Storage => CategoryPayload::Known("Storage"),
        NodeKind::Divider => CategoryPayload::Known("Divider"),
        NodeKind::Unknown(code) => CategoryPayload::Unknown(UnknownCodePayload { code }),
    }
}

fn link_kind_payload(kind: LinkKind) -> CategoryPayload {
    match kind {
        LinkKind::Conduit => CategoryPayload::Known("Conduit"),
        LinkKind::Pump => CategoryPayload::Known("Pump"),
        LinkKind::Orifice => CategoryPayload::Known("Orifice"),
        LinkKind::Weir => CategoryPayload::Known("Weir"),
        LinkKind::Outlet => CategoryPayload::Known("Outlet"),
        LinkKind::Unknown(code) => CategoryPayload::Unknown(UnknownCodePayload { code }),
    }
}

fn concentration_units_payload(units: ConcentrationUnits) -> CategoryPayload {
    match units {
        ConcentrationUnits::MilligramsPerLiter => CategoryPayload::Known("milligrams_per_liter"),
        ConcentrationUnits::MicrogramsPerLiter => CategoryPayload::Known("micrograms_per_liter"),
        ConcentrationUnits::CountsPerLiter => CategoryPayload::Known("counts_per_liter"),
        ConcentrationUnits::Unknown(code) => CategoryPayload::Unknown(UnknownCodePayload { code }),
    }
}

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

fn invalid_selection(operation: &str, error: impl std::fmt::Display) -> JsValue {
    let value = Error::new(&format!("{operation}: invalid selection: {error}"));
    let _ = Reflect::set(&value, &"category".into(), &"invalid_selection".into());
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    value.into()
}

fn output_error(operation: &str, error: OutputError) -> JsValue {
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
    let value = Error::new(&error.to_string());
    let _ = Reflect::set(&value, &"category".into(), &category.into());
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    value.into()
}
