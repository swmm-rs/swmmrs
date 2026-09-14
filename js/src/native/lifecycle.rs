//! Lifecycle progress and project metadata adapters.

use swmmrs::engine::consts::{SOLVER_BUILD_ID, SOLVER_VERSION};
use swmmrs::engine::enums::{ObjectType, UnitsType};
use swmmrs::simulation::SimulationLifecycle;
use wasm_bindgen::prelude::*;

use super::super::{Simulation, timestamp};
use super::common::{identity_ids, serialize};
use super::records::{Info, Status};

fn state_name(state: SimulationLifecycle) -> &'static str {
    match state {
        SimulationLifecycle::Closed => "closed",
        SimulationLifecycle::Open => "open",
        SimulationLifecycle::Running => "running",
        SimulationLifecycle::Complete => "complete",
        SimulationLifecycle::Ended => "ended",
        SimulationLifecycle::Failed => "failed",
    }
}

#[wasm_bindgen]
impl Simulation {
    /// Lifecycle, model-time, progress, and warning state in one owner read.
    #[wasm_bindgen(js_name = status)]
    pub fn status(&self) -> Result<JsValue, JsValue> {
        let lifecycle = self.inner.lifecycle_read();
        let current_time = match lifecycle.state {
            SimulationLifecycle::Running
            | SimulationLifecycle::Complete
            | SimulationLifecycle::Ended => self
                .inner
                .current_time_read()
                .map_err(|error| self.error("status", error))
                .map(|current| Some(timestamp(current)))?,
            _ => None,
        };
        let elapsed_seconds = if matches!(
            lifecycle.state,
            SimulationLifecycle::Running
                | SimulationLifecycle::Complete
                | SimulationLifecycle::Ended
        ) {
            (lifecycle.routing_time / 1000.0).max(0.0)
        } else {
            0.0
        };
        let duration_seconds = (lifecycle.total_duration / 1000.0).max(0.0);
        let percent_complete = if duration_seconds > 0.0 {
            (100.0 * elapsed_seconds / duration_seconds).clamp(0.0, 100.0)
        } else if matches!(
            lifecycle.state,
            SimulationLifecycle::Complete | SimulationLifecycle::Ended
        ) {
            100.0
        } else {
            0.0
        };
        serialize(&Status {
            state: state_name(lifecycle.state),
            is_open: lifecycle.is_open,
            is_started: lifecycle.is_started,
            configuration_dirty: lifecycle.configuration_dirty,
            save_results: lifecycle.save_results,
            current_time,
            elapsed_seconds,
            duration_seconds,
            percent_complete,
            step_count: lifecycle.total_step_count,
            report_period_count: lifecycle.report_period_count,
            warning_count: lifecycle.warning_count,
        })
    }
}

impl Simulation {
    /// Extends the existing metadata object with browser-facing identity lists and schedule.
    pub(crate) fn info_metadata(&self) -> Result<JsValue, JsValue> {
        let project = self
            .inner
            .project_read()
            .map_err(|error| self.error("info", error))?;
        let times = self
            .inner
            .simulation_time_read()
            .map_err(|error| self.error("info", error))?;
        let (_, units) = self
            .inner
            .units_read()
            .map_err(|error| self.error("info", error))?;
        serialize(&Info {
            flow_units: format!("{:?}", project.flow_units),
            unit_system: match units {
                UnitsType::Us => "us",
                UnitsType::Si => "si",
            },
            solver_version: SOLVER_VERSION,
            solver_build_id: SOLVER_BUILD_ID,
            effective_threads: project.effective_thread_count,
            node_ids: project.node_ids,
            link_ids: project.link_ids,
            subcatchment_ids: identity_ids(self, ObjectType::Subcatch, "info")?,
            rain_gage_ids: identity_ids(self, ObjectType::Gage, "info")?,
            pollutant_ids: identity_ids(self, ObjectType::Pollut, "info")?,
            land_use_ids: identity_ids(self, ObjectType::Landuse, "info")?,
            time_pattern_ids: identity_ids(self, ObjectType::Timepattern, "info")?,
            curve_ids: identity_ids(self, ObjectType::Curve, "info")?,
            time_series_ids: identity_ids(self, ObjectType::Tseries, "info")?,
            control_ids: identity_ids(self, ObjectType::Control, "info")?,
            transect_ids: identity_ids(self, ObjectType::Transect, "info")?,
            aquifer_ids: identity_ids(self, ObjectType::Aquifer, "info")?,
            snowmelt_ids: identity_ids(self, ObjectType::Snowmelt, "info")?,
            shape_ids: identity_ids(self, ObjectType::Shape, "info")?,
            street_ids: identity_ids(self, ObjectType::Street, "info")?,
            inlet_design_ids: identity_ids(self, ObjectType::Inlet, "info")?,
            amm_model_ids: identity_ids(self, ObjectType::AmmModel, "info")?,
            unit_hydrograph_ids: identity_ids(self, ObjectType::Unithyd, "info")?,
            lid_control_ids: identity_ids(self, ObjectType::Lid, "info")?,
            route_model: format!("{:?}", project.route_model),
            start_time: timestamp(times.start_time),
            report_start: timestamp(times.report_start),
            end_time: timestamp(times.end_time),
        })
    }
}
