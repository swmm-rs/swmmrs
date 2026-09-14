//! Browser-facing AMM model and assignment adapters.

use serde::{Deserialize, Serialize};
use swmmrs::engine::enums::ObjectType;
use swmmrs::simulation::{AmmAssignmentConfiguration, AmmComponentConfiguration, AmmModelPatch};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, decode_value, identity_id, non_null, serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum AmmComponentRecord {
    /// The exact public tag is `standard`.
    #[serde(rename = "standard")]
    Standard {
        id: String,
        dry_capture: f64,
        precipitation_window_hours: f64,
        hydrograph_half_life_hours: f64,
        initial_wet_capture: f64,
        moisture_half_life_hours: f64,
        temperature_window_hours: f64,
        spring_cold_shcf: f64,
        hot_shcf: f64,
        fall_cold_shcf: f64,
    },
    /// The exact public tag is `baseflow`.
    #[serde(rename = "baseflow")]
    Baseflow {
        id: String,
        precipitation_window_hours: f64,
        hydrograph_half_life_hours: f64,
        temperature_window_hours: f64,
        spring_cold_capture: f64,
        hot_capture: f64,
        fall_cold_capture: f64,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AmmModelRecord {
    rain_gage: String,
    cold_temperature: f64,
    hot_temperature: f64,
    components: Vec<AmmComponentRecord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AmmModelPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    rain_gage: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    cold_temperature: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    hot_temperature: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    components: Option<Vec<AmmComponentRecord>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AmmAssignmentRecord {
    node: String,
    model: String,
    area: f64,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = ammModelConfiguration)]
    pub fn amm_model_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::AmmModel, id)?;
        let configuration = self
            .inner
            .amm_model_configuration_read(index)
            .map_err(|error| self.error("ammModelConfiguration", error))?;
        let rain_gage = identity_id(
            self,
            ObjectType::Gage,
            configuration.rain_gage_index,
            "ammModelConfiguration",
        )?;
        let components = configuration
            .components
            .into_iter()
            .map(component_record)
            .collect();
        serialize(&AmmModelRecord {
            rain_gage,
            cold_temperature: configuration.cold_temperature,
            hot_temperature: configuration.hot_temperature,
            components,
        })
    }

    #[wasm_bindgen(js_name = configureAmmModel)]
    pub fn configure_amm_model(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let value: AmmModelPatchDto = decode(patch, "configureAmmModel")?;
        let index = self.index(ObjectType::AmmModel, id)?;
        let rain_gage_index = value
            .rain_gage
            .map(|rain_gage| self.index(ObjectType::Gage, &rain_gage))
            .transpose()?;
        let components = value.components.map(|components| {
            components
                .into_iter()
                .map(component_configuration)
                .collect::<Vec<_>>()
        });
        let patch = AmmModelPatch {
            rain_gage_index,
            cold_temperature: value.cold_temperature,
            hot_temperature: value.hot_temperature,
            components,
        };
        self.inner
            .patch_amm_model(index, patch)
            .map_err(|error| self.error("configureAmmModel", error))
    }

    #[wasm_bindgen(js_name = ammAssignments)]
    pub fn amm_assignments(&self) -> Result<JsValue, JsValue> {
        let assignments = self
            .inner
            .amm_assignments_read()
            .map_err(|error| self.error("ammAssignments", error))?;
        let records = assignments
            .into_iter()
            .map(|assignment| {
                Ok(AmmAssignmentRecord {
                    node: identity_id(
                        self,
                        ObjectType::Node,
                        assignment.node_index,
                        "ammAssignments",
                    )?,
                    model: identity_id(
                        self,
                        ObjectType::AmmModel,
                        assignment.model_index,
                        "ammAssignments",
                    )?,
                    area: assignment.area,
                })
            })
            .collect::<Result<Vec<_>, JsValue>>()?;
        serialize(&records)
    }

    #[wasm_bindgen(js_name = replaceAmmAssignments)]
    pub fn replace_amm_assignments(&mut self, assignments: JsValue) -> Result<(), JsValue> {
        let records: Vec<AmmAssignmentRecord> = decode_value(assignments, "replaceAmmAssignments")?;
        let assignments = records
            .into_iter()
            .map(|assignment| {
                Ok(AmmAssignmentConfiguration {
                    node_index: self.index(ObjectType::Node, &assignment.node)?,
                    model_index: self.index(ObjectType::AmmModel, &assignment.model)?,
                    area: assignment.area,
                })
            })
            .collect::<Result<Vec<_>, JsValue>>()?;
        self.inner
            .replace_amm_assignments(assignments)
            .map_err(|error| self.error("replaceAmmAssignments", error))
    }
}

fn component_record(component: AmmComponentConfiguration) -> AmmComponentRecord {
    match component {
        AmmComponentConfiguration::Standard {
            id,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        } => AmmComponentRecord::Standard {
            id,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        },
        AmmComponentConfiguration::Baseflow {
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        } => AmmComponentRecord::Baseflow {
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        },
    }
}

fn component_configuration(component: AmmComponentRecord) -> AmmComponentConfiguration {
    match component {
        AmmComponentRecord::Standard {
            id,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        } => AmmComponentConfiguration::Standard {
            id,
            dry_capture,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            initial_wet_capture,
            moisture_half_life_hours,
            temperature_window_hours,
            spring_cold_shcf,
            hot_shcf,
            fall_cold_shcf,
        },
        AmmComponentRecord::Baseflow {
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        } => AmmComponentConfiguration::Baseflow {
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        },
    }
}
