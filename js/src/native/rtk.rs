//! Browser-facing RTK unit-hydrograph and RDII assignment adapters.

use serde::{Deserialize, Serialize};
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::simulation::{
    RdiiAssignmentConfiguration, UnitHydrographPatch, UnitHydrographResponseConfiguration,
};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, decode_value, identity_id, non_null, serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitHydrographResponseRecord {
    rainfall_fraction: f64,
    time_to_peak_hours: f64,
    recession_ratio: f64,
    maximum_initial_abstraction: f64,
    initial_abstraction_recovery_rate: f64,
    initial_abstraction_at_start: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitHydrographMonthRecord {
    short: UnitHydrographResponseRecord,
    medium: UnitHydrographResponseRecord,
    long: UnitHydrographResponseRecord,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnitHydrographRecord {
    rain_gage: String,
    monthly_responses: Vec<UnitHydrographMonthRecord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnitHydrographPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    rain_gage: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    monthly_responses: Option<Vec<UnitHydrographMonthRecord>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RdiiAssignmentRecord {
    node: String,
    unit_hydrograph: String,
    area: f64,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = unitHydrographConfiguration)]
    pub fn unit_hydrograph_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Unithyd, id)?;
        let configuration = self
            .inner
            .unit_hydrograph_configuration_read(index)
            .map_err(|error| self.error("unitHydrographConfiguration", error))?;
        let rain_gage = identity_id(
            self,
            ObjectType::Gage,
            configuration.rain_gage_index,
            "unitHydrographConfiguration",
        )?;
        let monthly_responses = configuration
            .monthly_responses
            .into_iter()
            .map(month_record)
            .collect();
        serialize(&UnitHydrographRecord {
            rain_gage,
            monthly_responses,
        })
    }

    #[wasm_bindgen(js_name = configureUnitHydrograph)]
    pub fn configure_unit_hydrograph(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let value: UnitHydrographPatchDto = decode(patch, "configureUnitHydrograph")?;
        let index = self.index(ObjectType::Unithyd, id)?;
        let rain_gage_index = value
            .rain_gage
            .map(|rain_gage| self.index(ObjectType::Gage, &rain_gage))
            .transpose()?;
        let monthly_responses = value
            .monthly_responses
            .map(monthly_configuration)
            .transpose()
            .map_err(|error| self.error("configureUnitHydrograph", error))?;
        let patch = UnitHydrographPatch {
            rain_gage_index,
            monthly_responses,
        };
        self.inner
            .patch_unit_hydrograph(index, patch)
            .map_err(|error| self.error("configureUnitHydrograph", error))
    }

    #[wasm_bindgen(js_name = rdiiAssignments)]
    pub fn rdii_assignments(&self) -> Result<JsValue, JsValue> {
        let assignments = self
            .inner
            .rdii_assignments_read()
            .map_err(|error| self.error("rdiiAssignments", error))?;
        let records = assignments
            .into_iter()
            .map(|assignment| {
                Ok(RdiiAssignmentRecord {
                    node: identity_id(
                        self,
                        ObjectType::Node,
                        assignment.node_index,
                        "rdiiAssignments",
                    )?,
                    unit_hydrograph: identity_id(
                        self,
                        ObjectType::Unithyd,
                        assignment.unit_hydrograph_index,
                        "rdiiAssignments",
                    )?,
                    area: assignment.area,
                })
            })
            .collect::<Result<Vec<_>, JsValue>>()?;
        serialize(&records)
    }

    #[wasm_bindgen(js_name = replaceRdiiAssignments)]
    pub fn replace_rdii_assignments(&mut self, assignments: JsValue) -> Result<(), JsValue> {
        let records: Vec<RdiiAssignmentRecord> =
            decode_value(assignments, "replaceRdiiAssignments")?;
        let assignments = records
            .into_iter()
            .map(|assignment| {
                Ok(RdiiAssignmentConfiguration {
                    node_index: self.index(ObjectType::Node, &assignment.node)?,
                    unit_hydrograph_index: self
                        .index(ObjectType::Unithyd, &assignment.unit_hydrograph)?,
                    area: assignment.area,
                })
            })
            .collect::<Result<Vec<_>, JsValue>>()?;
        self.inner
            .replace_rdii_assignments(assignments)
            .map_err(|error| self.error("replaceRdiiAssignments", error))
    }
}

fn response_record(response: UnitHydrographResponseConfiguration) -> UnitHydrographResponseRecord {
    UnitHydrographResponseRecord {
        rainfall_fraction: response.rainfall_fraction,
        time_to_peak_hours: response.time_to_peak_hours,
        recession_ratio: response.recession_ratio,
        maximum_initial_abstraction: response.maximum_initial_abstraction,
        initial_abstraction_recovery_rate: response.initial_abstraction_recovery_rate,
        initial_abstraction_at_start: response.initial_abstraction_at_start,
    }
}

fn month_record(month: [UnitHydrographResponseConfiguration; 3]) -> UnitHydrographMonthRecord {
    let [short, medium, long] = month;
    UnitHydrographMonthRecord {
        short: response_record(short),
        medium: response_record(medium),
        long: response_record(long),
    }
}

fn response_configuration(
    response: UnitHydrographResponseRecord,
) -> UnitHydrographResponseConfiguration {
    UnitHydrographResponseConfiguration {
        rainfall_fraction: response.rainfall_fraction,
        time_to_peak_hours: response.time_to_peak_hours,
        recession_ratio: response.recession_ratio,
        maximum_initial_abstraction: response.maximum_initial_abstraction,
        initial_abstraction_recovery_rate: response.initial_abstraction_recovery_rate,
        initial_abstraction_at_start: response.initial_abstraction_at_start,
    }
}

fn monthly_configuration(
    months: Vec<UnitHydrographMonthRecord>,
) -> Result<[[UnitHydrographResponseConfiguration; 3]; 12], SwmmError> {
    let months: [UnitHydrographMonthRecord; 12] = months
        .try_into()
        .map_err(|_| invalid_rtk("monthlyResponses must contain exactly 12 months"))?;
    Ok(months.map(|month| {
        [
            response_configuration(month.short),
            response_configuration(month.medium),
            response_configuration(month.long),
        ]
    }))
}

fn invalid_rtk(detail: impl Into<String>) -> SwmmError {
    SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail)
}
