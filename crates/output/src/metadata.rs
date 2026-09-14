use std::str::Utf8Error;
use std::time::Duration;

macro_rules! output_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub(crate) usize);

        impl $name {
            /// Returns the zero-based position in the opened output file.
            ///
            /// # Returns
            /// Zero-based element position.
            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

output_id!(SubcatchmentId);
output_id!(NodeId);
output_id!(LinkId);
output_id!(PollutantId);

/// Exact byte-preserving output element name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OutputName(pub(crate) Box<[u8]>);

impl OutputName {
    /// Returns the exact declared name bytes.
    ///
    /// # Returns
    /// Borrowed name bytes without decoding.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Decodes the declared name as UTF-8 without loss.
    ///
    /// # Returns
    /// Borrowed UTF-8 text when the stored bytes are valid UTF-8.
    ///
    /// # Errors
    /// Returns `Utf8Error` when the stored bytes are not valid UTF-8.
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}

/// Flow units stored in the output header.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlowUnits {
    /// Cubic feet per second.
    Cfs,
    /// Gallons per minute.
    Gpm,
    /// Million gallons per day.
    Mgd,
    /// Cubic metres per second.
    Cms,
    /// Litres per second.
    Lps,
    /// Million litres per day.
    Mld,
    /// Unrecognized stored unit code.
    Unknown(i32),
}

impl FlowUnits {
    pub(crate) const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Cfs,
            1 => Self::Gpm,
            2 => Self::Mgd,
            3 => Self::Cms,
            4 => Self::Lps,
            5 => Self::Mld,
            value => Self::Unknown(value),
        }
    }

    /// Infers the unit system only for known flow units.
    ///
    /// # Returns
    /// US or SI units for recognized flow units, otherwise `None`.
    pub const fn unit_system(self) -> Option<UnitSystem> {
        match self {
            Self::Cfs | Self::Gpm | Self::Mgd => Some(UnitSystem::Us),
            Self::Cms | Self::Lps | Self::Mld => Some(UnitSystem::Si),
            Self::Unknown(_) => None,
        }
    }
}

/// Unit system inferred from recognized flow units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitSystem {
    /// United States customary units.
    Us,
    /// International System of Units.
    Si,
}

/// Pollutant concentration units stored in the identifier section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConcentrationUnits {
    /// Milligrams per litre.
    MilligramsPerLiter,
    /// Micrograms per litre.
    MicrogramsPerLiter,
    /// Counts per litre.
    CountsPerLiter,
    /// Unrecognized stored unit code.
    Unknown(i32),
}

impl ConcentrationUnits {
    pub(crate) const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::MilligramsPerLiter,
            1 => Self::MicrogramsPerLiter,
            2 => Self::CountsPerLiter,
            value => Self::Unknown(value),
        }
    }
}

/// Node kind stored as a physical integer property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// Junction node.
    Junction,
    /// Outfall node.
    Outfall,
    /// Storage node.
    Storage,
    /// Divider node.
    Divider,
    /// Unrecognized stored node-kind code.
    Unknown(i32),
}

impl NodeKind {
    pub(crate) const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Junction,
            1 => Self::Outfall,
            2 => Self::Storage,
            3 => Self::Divider,
            value => Self::Unknown(value),
        }
    }
}

/// Link kind stored as a physical integer property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkKind {
    /// Conduit link.
    Conduit,
    /// Pump link.
    Pump,
    /// Orifice link.
    Orifice,
    /// Weir link.
    Weir,
    /// Outlet link.
    Outlet,
    /// Unrecognized stored link-kind code.
    Unknown(i32),
}

impl LinkKind {
    pub(crate) const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Conduit,
            1 => Self::Pump,
            2 => Self::Orifice,
            3 => Self::Weir,
            4 => Self::Outlet,
            value => Self::Unknown(value),
        }
    }
}

/// Simulation conclusion, or absence of one before output finalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunStatus {
    /// The output file has no finalized trailer.
    Unfinalized,
    /// Simulation completed without a nonzero status code.
    Success,
    /// Simulation completed with the exact nonzero status code.
    Warning(i32),
}

/// Exact timezone-free SWMM serial day.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwmmDate(pub(crate) f64);

impl SwmmDate {
    /// Creates a serial date when the supplied value is finite.
    ///
    /// # Arguments
    /// * `serial_days` - SWMM serial days since December 30, 1899.
    ///
    /// # Returns
    /// `Some` for every finite value, otherwise `None`.
    pub fn from_serial_days(serial_days: f64) -> Option<Self> {
        serial_days.is_finite().then_some(Self(serial_days))
    }

    /// Returns the exact stored or computed serial-day value.
    ///
    /// # Returns
    /// SWMM serial days since December 30, 1899.
    pub const fn serial_days(self) -> f64 {
        self.0
    }
}
/// One checked half-open report-period or Nominal Report Date range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputRange {
    /// Select every available complete report period.
    All,
    /// Select the exact zero-based half-open period span.
    Periods {
        /// Inclusive period offset.
        start: usize,
        /// Exclusive period offset.
        end: usize,
    },
    /// Select the half-open Nominal Report Date interval.
    Dates {
        /// Optional inclusive serial-day bound.
        start: Option<SwmmDate>,
        /// Optional exclusive serial-day bound.
        end: Option<SwmmDate>,
    },
}
/// Selects one stored element by zero-based position or exact raw name bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputElementSelector<'a> {
    /// Zero-based element position.
    Index(usize),
    /// Exact stored name bytes, matched case-sensitively without decoding.
    Name(&'a [u8]),
}

/// One immutable typed series aligned to its Nominal Report Date axis.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputTimeSeries {
    pub(crate) selection: OutputSeriesSelection,
    pub(crate) times: Vec<SwmmDate>,
    pub(crate) values: Vec<f32>,
}

impl OutputTimeSeries {
    /// Returns the canonical resolved selection requested for this series.
    pub const fn selection(&self) -> OutputSeriesSelection {
        self.selection
    }

    /// Returns Nominal Report Dates aligned one-for-one with the values.
    pub fn times(&self) -> &[SwmmDate] {
        &self.times
    }

    /// Returns stored values aligned one-for-one with the Nominal Report Dates.
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

/// One immutable selection-labelled sequence aligned to a bulk result axis.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputValueSeries {
    pub(crate) selection: OutputSeriesSelection,
    pub(crate) values: Vec<f32>,
}

impl OutputValueSeries {
    /// Returns the canonical selection requested for this series.
    pub const fn selection(&self) -> OutputSeriesSelection {
        self.selection
    }

    /// Returns stored values aligned one-for-one with the bulk result axis.
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

/// Immutable column-oriented result for an ordered bulk query.
#[derive(Clone, Debug, PartialEq)]
pub struct BulkSeriesResult {
    pub(crate) times: Vec<SwmmDate>,
    pub(crate) series: Vec<OutputValueSeries>,
}

impl BulkSeriesResult {
    /// Returns the ascending Nominal Report Date axis.
    pub fn times(&self) -> &[SwmmDate] {
        &self.times
    }

    /// Returns ordered selection-labelled value series.
    pub fn series(&self) -> &[OutputValueSeries] {
        &self.series
    }

    /// Returns one value using offsets local to this result.
    pub fn value(&self, period_offset: usize, selection_offset: usize) -> Option<f32> {
        self.series
            .get(selection_offset)?
            .values
            .get(period_offset)
            .copied()
    }
}

/// Fixed report schedule and available complete-period count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReportTiming {
    pub(crate) report_schedule_origin: SwmmDate,
    pub(crate) report_step: Duration,
    pub(crate) period_count: usize,
}

impl ReportTiming {
    /// Returns the Report Schedule Origin.
    ///
    /// # Returns
    /// Header date one report step before nominal period zero.
    pub const fn report_schedule_origin(&self) -> SwmmDate {
        self.report_schedule_origin
    }

    /// Returns the positive whole-second report step.
    ///
    /// # Returns
    /// Fixed report interval.
    pub const fn report_step(&self) -> Duration {
        self.report_step
    }

    /// Returns the available number of complete report periods.
    ///
    /// # Returns
    /// Number of complete stored result records.
    pub const fn period_count(&self) -> usize {
        self.period_count
    }

    /// Computes the Nominal Report Date for a zero-based period.
    ///
    /// # Arguments
    /// * `period` - Zero-based report period.
    ///
    /// # Returns
    /// Nominal scheduled date, or `None` when `period` is out of range or arithmetic is non-finite.
    pub fn nominal_date(&self, period: usize) -> Option<SwmmDate> {
        if period >= self.period_count {
            return None;
        }
        let seconds = (period as f64 + 1.0) * self.report_step.as_secs_f64();
        let serial_days = self.report_schedule_origin.0 + seconds / 86_400.0;
        serial_days.is_finite().then_some(SwmmDate(serial_days))
    }
}

/// Stored subcatchment identity and static properties.
#[derive(Clone, Debug, PartialEq)]
pub struct SubcatchmentMetadata {
    pub(crate) id: SubcatchmentId,
    pub(crate) name: OutputName,
    pub(crate) area: f32,
}

impl SubcatchmentMetadata {
    /// Returns the typed subcatchment identity.
    pub const fn id(&self) -> SubcatchmentId {
        self.id
    }
    /// Returns the exact stored name.
    pub const fn name(&self) -> &OutputName {
        &self.name
    }
    /// Returns the exact stored area in configured output units.
    pub const fn area(&self) -> f32 {
        self.area
    }
}

/// Stored node identity and static properties.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeMetadata {
    pub(crate) id: NodeId,
    pub(crate) name: OutputName,
    pub(crate) kind: NodeKind,
    pub(crate) invert_elevation: f32,
    pub(crate) maximum_depth: f32,
}

impl NodeMetadata {
    /// Returns the typed node identity.
    pub const fn id(&self) -> NodeId {
        self.id
    }
    /// Returns the exact stored name.
    pub const fn name(&self) -> &OutputName {
        &self.name
    }
    /// Returns the exact stored node kind.
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }
    /// Returns the exact stored invert elevation in configured output units.
    pub const fn invert_elevation(&self) -> f32 {
        self.invert_elevation
    }
    /// Returns the exact stored maximum depth in configured output units.
    pub const fn maximum_depth(&self) -> f32 {
        self.maximum_depth
    }
}

/// Stored link identity and uniform static properties.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkMetadata {
    pub(crate) id: LinkId,
    pub(crate) name: OutputName,
    pub(crate) kind: LinkKind,
    pub(crate) inlet_offset: f32,
    pub(crate) outlet_offset: f32,
    pub(crate) maximum_depth: f32,
    pub(crate) length: f32,
}

impl LinkMetadata {
    /// Returns the typed link identity.
    pub const fn id(&self) -> LinkId {
        self.id
    }
    /// Returns the exact stored name.
    pub const fn name(&self) -> &OutputName {
        &self.name
    }
    /// Returns the exact stored link kind.
    pub const fn kind(&self) -> LinkKind {
        self.kind
    }
    /// Returns the exact stored inlet offset in configured output units.
    pub const fn inlet_offset(&self) -> f32 {
        self.inlet_offset
    }
    /// Returns the exact stored outlet offset in configured output units.
    pub const fn outlet_offset(&self) -> f32 {
        self.outlet_offset
    }
    /// Returns the exact stored maximum depth in configured output units.
    pub const fn maximum_depth(&self) -> f32 {
        self.maximum_depth
    }
    /// Returns the exact stored length in configured output units.
    pub const fn length(&self) -> f32 {
        self.length
    }
}

/// Stored pollutant identity, name, and concentration units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PollutantMetadata {
    pub(crate) id: PollutantId,
    pub(crate) name: OutputName,
    pub(crate) concentration_units: ConcentrationUnits,
}

impl PollutantMetadata {
    /// Returns the typed pollutant identity.
    pub const fn id(&self) -> PollutantId {
        self.id
    }
    /// Returns the exact stored name.
    pub const fn name(&self) -> &OutputName {
        &self.name
    }
    /// Returns the exact stored concentration units.
    pub const fn concentration_units(&self) -> ConcentrationUnits {
        self.concentration_units
    }
}

/// Subcatchment Output Result Attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubcatchmentResultAttribute {
    /// Rainfall.
    Rainfall,
    /// Snow depth.
    SnowDepth,
    /// Evaporation loss.
    EvaporationLoss,
    /// Infiltration loss.
    InfiltrationLoss,
    /// Runoff flow.
    RunoffFlow,
    /// Groundwater flow.
    GroundwaterFlow,
    /// Groundwater elevation.
    GroundwaterElevation,
    /// Soil moisture.
    SoilMoisture,
    /// Pollutant concentration.
    Pollutant(PollutantId),
    /// Unrecognized stored result code.
    Unknown(i32),
}

/// Node Output Result Attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeResultAttribute {
    /// Depth.
    Depth,
    /// Hydraulic head.
    HydraulicHead,
    /// Stored volume.
    StoredVolume,
    /// Lateral inflow.
    LateralInflow,
    /// Total inflow.
    TotalInflow,
    /// Overflow.
    Overflow,
    /// Pollutant concentration.
    Pollutant(PollutantId),
    /// Unrecognized stored result code.
    Unknown(i32),
}

/// Link Output Result Attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkResultAttribute {
    /// Flow.
    Flow,
    /// Depth.
    Depth,
    /// Velocity.
    Velocity,
    /// Volume.
    Volume,
    /// Capacity.
    Capacity,
    /// Pollutant concentration.
    Pollutant(PollutantId),
    /// Unrecognized stored result code.
    Unknown(i32),
}

/// System Output Result Attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemResultAttribute {
    /// Air temperature.
    AirTemperature,
    /// Rainfall.
    Rainfall,
    /// Snow depth.
    SnowDepth,
    /// Infiltration loss.
    InfiltrationLoss,
    /// Runoff flow.
    RunoffFlow,
    /// Dry-weather inflow.
    DryWeatherInflow,
    /// Groundwater inflow.
    GroundwaterInflow,
    /// RDII inflow.
    RdiiInflow,
    /// External inflow.
    ExternalInflow,
    /// Total lateral inflow.
    TotalLateralInflow,
    /// Flooding outflow.
    FloodingOutflow,
    /// Outfall flow.
    OutfallFlow,
    /// Storage volume.
    StorageVolume,
    /// Evaporation.
    Evaporation,
    /// Potential evapotranspiration.
    PotentialEvapotranspiration,
    /// Unrecognized stored result code.
    Unknown(i32),
}

/// Ordered physical result schemas for every output element family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputResultSchema {
    pub(crate) subcatchment: Box<[SubcatchmentResultAttribute]>,
    pub(crate) node: Box<[NodeResultAttribute]>,
    pub(crate) link: Box<[LinkResultAttribute]>,
    pub(crate) system: Box<[SystemResultAttribute]>,
}

impl OutputResultSchema {
    /// Returns subcatchment attributes in physical column order.
    pub fn subcatchment(&self) -> &[SubcatchmentResultAttribute] {
        &self.subcatchment
    }
    /// Returns node attributes in physical column order.
    pub fn node(&self) -> &[NodeResultAttribute] {
        &self.node
    }
    /// Returns link attributes in physical column order.
    pub fn link(&self) -> &[LinkResultAttribute] {
        &self.link
    }
    /// Returns system attributes in physical column order.
    pub fn system(&self) -> &[SystemResultAttribute] {
        &self.system
    }
}

/// Immutable metadata eagerly parsed from a SWMM output file.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputMetadata {
    pub(crate) solver_release: i32,
    pub(crate) run_status: RunStatus,
    pub(crate) flow_units: FlowUnits,
    pub(crate) report_timing: ReportTiming,
    pub(crate) subcatchments: Box<[SubcatchmentMetadata]>,
    pub(crate) nodes: Box<[NodeMetadata]>,
    pub(crate) links: Box<[LinkMetadata]>,
    pub(crate) pollutants: Box<[PollutantMetadata]>,
    pub(crate) result_schema: OutputResultSchema,
}

impl OutputMetadata {
    /// Returns the exact packed producing solver release.
    pub const fn solver_release(&self) -> i32 {
        self.solver_release
    }
    /// Returns the finalized run status, or `Unfinalized` when no trailer was read.
    pub const fn run_status(&self) -> RunStatus {
        self.run_status
    }
    /// Returns the stored flow units.
    pub const fn flow_units(&self) -> FlowUnits {
        self.flow_units
    }
    /// Returns the inferred unit system for known flow units.
    pub const fn unit_system(&self) -> Option<UnitSystem> {
        self.flow_units.unit_system()
    }
    /// Returns the report timing metadata.
    pub const fn report_timing(&self) -> &ReportTiming {
        &self.report_timing
    }
    /// Returns subcatchments in declared order.
    pub fn subcatchments(&self) -> &[SubcatchmentMetadata] {
        &self.subcatchments
    }
    /// Returns nodes in declared order.
    pub fn nodes(&self) -> &[NodeMetadata] {
        &self.nodes
    }
    /// Returns links in declared order.
    pub fn links(&self) -> &[LinkMetadata] {
        &self.links
    }
    /// Returns pollutants in declared order.
    pub fn pollutants(&self) -> &[PollutantMetadata] {
        &self.pollutants
    }
    /// Returns all ordered result schemas.
    pub const fn result_schema(&self) -> &OutputResultSchema {
        &self.result_schema
    }
}

/// Ordered heterogeneous Output Series Selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutputSeriesSelection {
    /// Select one subcatchment result attribute.
    Subcatchment {
        /// Subcatchment identity.
        id: SubcatchmentId,
        /// Stored result attribute.
        attribute: SubcatchmentResultAttribute,
    },
    /// Select one node result attribute.
    Node {
        /// Node identity.
        id: NodeId,
        /// Stored result attribute.
        attribute: NodeResultAttribute,
    },
    /// Select one link result attribute.
    Link {
        /// Link identity.
        id: LinkId,
        /// Stored result attribute.
        attribute: LinkResultAttribute,
    },
    /// Select one system result attribute.
    System {
        /// Stored result attribute.
        attribute: SystemResultAttribute,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_date_has_explicit_bounds() {
        let timing = ReportTiming {
            report_schedule_origin: SwmmDate(10.0),
            report_step: Duration::from_secs(86_400),
            period_count: 1,
        };
        assert_eq!(timing.nominal_date(0).unwrap().serial_days(), 11.0);
        assert!(timing.nominal_date(1).is_none());
    }

    #[test]
    fn checked_serial_days_accept_only_finite_values() {
        assert_eq!(
            SwmmDate::from_serial_days(-1.25)
                .expect("finite serial days must be accepted")
                .serial_days()
                .to_bits(),
            (-1.25_f64).to_bits()
        );
        assert!(SwmmDate::from_serial_days(f64::NAN).is_none());
        assert!(SwmmDate::from_serial_days(f64::INFINITY).is_none());
        assert!(SwmmDate::from_serial_days(f64::NEG_INFINITY).is_none());
    }
}
