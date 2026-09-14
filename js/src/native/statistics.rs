//! System continuity and routing diagnostics adapters.

use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::serialize;
use super::records::{QualityBalance, RoutingDiagnostics, RoutingTotals, RunoffTotals, Statistics};

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = statistics)]
    pub fn statistics(&self) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .simulation_statistics_read()
            .map_err(|error| self.error("statistics", error))?;
        let quality_balances: Vec<_> = values
            .quality_balances
            .into_iter()
            .map(|(id, balance)| {
                (
                    id,
                    QualityBalance {
                        continuity_error: balance.continuity_error,
                        seepage_loss: balance.seepage_loss,
                    },
                )
            })
            .collect();
        serialize(&Statistics {
            routing_totals: RoutingTotals {
                dry_weather_inflow: values.routing_totals.dry_weather_inflow,
                wet_weather_inflow: values.routing_totals.wet_weather_inflow,
                groundwater_inflow: values.routing_totals.groundwater_inflow,
                rdii_inflow: values.routing_totals.rdii_inflow,
                external_inflow: values.routing_totals.external_inflow,
                flooding: values.routing_totals.flooding,
                outflow: values.routing_totals.outflow,
                evaporation_loss: values.routing_totals.evaporation_loss,
                seepage_loss: values.routing_totals.seepage_loss,
                reaction_loss: values.routing_totals.reaction_loss,
                initial_storage: values.routing_totals.initial_storage,
                final_storage: values.routing_totals.final_storage,
                continuity_error: values.routing_totals.continuity_error,
            },
            runoff_totals: RunoffTotals {
                rainfall: values.runoff_totals.rainfall,
                evaporation_loss: values.runoff_totals.evaporation_loss,
                infiltration_loss: values.runoff_totals.infiltration_loss,
                runoff: values.runoff_totals.runoff,
                lid_drainage: values.runoff_totals.lid_drainage,
                outfall_runon: values.runoff_totals.outfall_runon,
                initial_surface_storage: values.runoff_totals.initial_surface_storage,
                final_surface_storage: values.runoff_totals.final_surface_storage,
                initial_snow_cover: values.runoff_totals.initial_snow_cover,
                final_snow_cover: values.runoff_totals.final_snow_cover,
                snow_removed: values.runoff_totals.snow_removed,
                continuity_error: values.runoff_totals.continuity_error,
            },
            groundwater_continuity_error: values.groundwater_continuity_error,
            quality_continuity_error: values.quality_continuity_error,
            routing_diagnostics: RoutingDiagnostics {
                average_time_step_seconds: values.routing_diagnostics.average_time_step_seconds,
                minimum_time_step_seconds: values.routing_diagnostics.minimum_time_step_seconds,
                maximum_time_step_seconds: values.routing_diagnostics.maximum_time_step_seconds,
                step_count: values.routing_diagnostics.step_count,
                nonconverged_step_count: values.routing_diagnostics.nonconverged_step_count,
                nonconverged_step_percentage: values
                    .routing_diagnostics
                    .nonconverged_step_percentage,
                average_iterations: values.routing_diagnostics.average_iterations,
            },
            quality_balances,
        })
    }
}
