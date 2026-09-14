//! JavaScript-facing lifecycle, hydraulic, and statistics records.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Status {
    pub(super) state: &'static str,
    pub(super) is_open: bool,
    pub(super) is_started: bool,
    pub(super) configuration_dirty: bool,
    pub(super) save_results: bool,
    pub(super) current_time: Option<String>,
    pub(super) elapsed_seconds: f64,
    pub(super) duration_seconds: f64,
    pub(super) percent_complete: f64,
    pub(super) step_count: i64,
    pub(super) report_period_count: i64,
    pub(super) warning_count: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Info {
    pub(super) flow_units: String,
    pub(super) unit_system: &'static str,
    pub(super) solver_version: &'static str,
    pub(super) solver_build_id: &'static str,
    pub(super) effective_threads: i32,
    pub(super) node_ids: Vec<String>,
    pub(super) link_ids: Vec<String>,
    pub(super) subcatchment_ids: Vec<String>,
    pub(super) rain_gage_ids: Vec<String>,
    pub(super) pollutant_ids: Vec<String>,
    pub(super) land_use_ids: Vec<String>,
    pub(super) time_pattern_ids: Vec<String>,
    pub(super) curve_ids: Vec<String>,
    pub(super) time_series_ids: Vec<String>,
    pub(super) control_ids: Vec<String>,
    pub(super) transect_ids: Vec<String>,
    pub(super) aquifer_ids: Vec<String>,
    pub(super) snowmelt_ids: Vec<String>,
    pub(super) shape_ids: Vec<String>,
    pub(super) street_ids: Vec<String>,
    pub(super) inlet_design_ids: Vec<String>,
    pub(super) amm_model_ids: Vec<String>,
    pub(super) unit_hydrograph_ids: Vec<String>,
    pub(super) lid_control_ids: Vec<String>,
    pub(super) route_model: String,
    pub(super) start_time: String,
    pub(super) report_start: String,
    pub(super) end_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubcatchmentResults {
    pub(super) rainfall: f64,
    pub(super) evaporation: f64,
    pub(super) infiltration: f64,
    pub(super) runon: f64,
    pub(super) runoff: f64,
    pub(super) snow_depth: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubcatchmentSnapshot {
    pub(super) object_ids: Vec<String>,
    pub(super) rainfall: Vec<f64>,
    pub(super) evaporation: Vec<f64>,
    pub(super) infiltration: Vec<f64>,
    pub(super) runon: Vec<f64>,
    pub(super) runoff: Vec<f64>,
    pub(super) snow_depth: Vec<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RainGage {
    pub(super) total_precip: f64,
    pub(super) rainfall: f64,
    pub(super) snowfall: f64,
    pub(super) external_precipitation_rate: Option<f64>,
    pub(super) rainfall_override: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RoutingTotals {
    pub(super) dry_weather_inflow: f64,
    pub(super) wet_weather_inflow: f64,
    pub(super) groundwater_inflow: f64,
    pub(super) rdii_inflow: f64,
    pub(super) external_inflow: f64,
    pub(super) flooding: f64,
    pub(super) outflow: f64,
    pub(super) evaporation_loss: f64,
    pub(super) seepage_loss: f64,
    pub(super) reaction_loss: f64,
    pub(super) initial_storage: f64,
    pub(super) final_storage: f64,
    pub(super) continuity_error: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RunoffTotals {
    pub(super) rainfall: f64,
    pub(super) evaporation_loss: f64,
    pub(super) infiltration_loss: f64,
    pub(super) runoff: f64,
    pub(super) lid_drainage: f64,
    pub(super) outfall_runon: f64,
    pub(super) initial_surface_storage: f64,
    pub(super) final_surface_storage: f64,
    pub(super) initial_snow_cover: f64,
    pub(super) final_snow_cover: f64,
    pub(super) snow_removed: f64,
    pub(super) continuity_error: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RoutingDiagnostics {
    pub(super) average_time_step_seconds: f64,
    pub(super) minimum_time_step_seconds: f64,
    pub(super) maximum_time_step_seconds: f64,
    pub(super) step_count: i32,
    pub(super) nonconverged_step_count: i64,
    pub(super) nonconverged_step_percentage: f64,
    pub(super) average_iterations: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QualityBalance {
    pub(super) continuity_error: f64,
    pub(super) seepage_loss: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Statistics {
    pub(super) routing_totals: RoutingTotals,
    pub(super) runoff_totals: RunoffTotals,
    pub(super) groundwater_continuity_error: f64,
    pub(super) quality_continuity_error: f64,
    pub(super) routing_diagnostics: RoutingDiagnostics,
    #[serde(serialize_with = "serialize_quality_balances")]
    pub(super) quality_balances: Vec<(String, QualityBalance)>,
}

fn serialize_quality_balances<S: serde::Serializer>(
    values: &[(String, QualityBalance)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for (id, balance) in values {
        map.serialize_entry(id, balance)?;
    }
    map.end()
}
