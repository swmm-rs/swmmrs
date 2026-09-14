//! Browser-facing node configuration, quality, forcing, and statistics adapters.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use swmmrs::engine::datetime::{DateTime, datetime_decodeDate, datetime_decodeTime};
use swmmrs::engine::enums::{ObjectType, StorageType};
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::simulation::nodes::{
    CanonicalStorageShape, DividerMode, NodeQualityRead, NodeQualitySnapshotRead,
    NodeStatisticsRead, NodeStatisticsSnapshotRead, NodeSubtypeConfigurationRead, OutfallBoundary,
    OutfallStatisticsRead, StorageStatisticsRead,
};
use swmmrs::simulation::quality::{PersistentPollutantValuesPatch, PollutantValuesPatch};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, decode_value, identity_id, non_null, serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageShapeValue {
    kind: String,
    coefficients: [f64; 3],
    curve: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageExfiltrationValue {
    conductivity: f64,
    suction_head: f64,
    moisture_deficit: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutfallBoundaryValue {
    kind: String,
    stage: f64,
    reference: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DividerRuleValue {
    kind: String,
    values: [f64; 3],
    curve: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeConfiguration {
    kind: String,
    tag: String,
    invert_elevation: f64,
    full_depth: f64,
    surcharge_depth: f64,
    ponded_area: f64,
    initial_depth: f64,
    included_in_report: bool,
    external_inflow: f64,
    shape: Option<StorageShapeValue>,
    evaporation_fraction: Option<f64>,
    exfiltration: Option<StorageExfiltrationValue>,
    boundary: Option<OutfallBoundaryValue>,
    has_flap_gate: Option<bool>,
    route_to_subcatchment: Option<String>,
    rule: Option<DividerRuleValue>,
    diverted_link: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeQuality {
    pollutant_ids: Vec<String>,
    concentrations: Vec<f64>,
    inflow_concentrations: Vec<f64>,
    reactor_concentrations: Vec<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeQualitySnapshot {
    object_ids: Vec<String>,
    pollutant_ids: Vec<String>,
    concentrations: Vec<Vec<f64>>,
    inflow_concentrations: Vec<Vec<f64>>,
    reactor_concentrations: Vec<Vec<f64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeStatistics {
    average_depth: f64,
    maximum_depth: f64,
    maximum_depth_time: String,
    maximum_reported_depth: f64,
    flooded_volume: f64,
    time_flooded_seconds: f64,
    time_surcharged_seconds: f64,
    time_courant_critical_seconds: f64,
    total_lateral_inflow: f64,
    maximum_lateral_inflow: f64,
    maximum_inflow: f64,
    maximum_overflow: f64,
    maximum_ponded_volume: f64,
    nonconverged_count: i32,
    maximum_inflow_time: String,
    maximum_overflow_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageStatistics {
    initial_volume: f64,
    average_volume: f64,
    maximum_volume: f64,
    maximum_inflow: f64,
    evaporation_losses: f64,
    exfiltration_losses: f64,
    maximum_volume_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutfallStatistics {
    average_flow: f64,
    maximum_flow: f64,
    pollutant_loads: BTreeMap<String, f64>,
    period_count: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeStatisticsSnapshot {
    object_ids: Vec<String>,
    average_depth: Vec<f64>,
    maximum_depth: Vec<f64>,
    maximum_depth_time: Vec<String>,
    maximum_reported_depth: Vec<f64>,
    flooded_volume: Vec<f64>,
    time_flooded_seconds: Vec<f64>,
    time_surcharged_seconds: Vec<f64>,
    time_courant_critical_seconds: Vec<f64>,
    total_lateral_inflow: Vec<f64>,
    maximum_lateral_inflow: Vec<f64>,
    maximum_inflow: Vec<f64>,
    maximum_overflow: Vec<f64>,
    maximum_ponded_volume: Vec<f64>,
    nonconverged_count: Vec<i32>,
    maximum_inflow_time: Vec<String>,
    maximum_overflow_time: Vec<String>,
}

/// Retains the distinction between an omitted key and an explicit null.
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum StorageShapeDto {
    Functional {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
    },
    Tabular {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
        #[serde(default, deserialize_with = "non_null")]
        curve: Option<String>,
    },
    Cylindrical {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
    },
    Conical {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
    },
    Paraboloid {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
    },
    Pyramidal {
        #[serde(default, deserialize_with = "non_null")]
        coefficients: Option<[f64; 3]>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StorageExfiltrationDto {
    conductivity: f64,
    #[serde(default, deserialize_with = "non_null")]
    suction_head: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    moisture_deficit: Option<f64>,
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum OutfallBoundaryDto {
    Free {},
    Normal {},
    Fixed {
        #[serde(default, deserialize_with = "non_null")]
        stage: Option<f64>,
    },
    Tidal {
        #[serde(default, deserialize_with = "non_null")]
        reference: Option<String>,
    },
    #[serde(rename = "timeseries")]
    TimeSeries {
        #[serde(default, deserialize_with = "non_null")]
        reference: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum DividerRuleDto {
    Overflow {},
    Cutoff {
        #[serde(default, deserialize_with = "non_null")]
        values: Option<[f64; 3]>,
    },
    Tabular {
        #[serde(default, deserialize_with = "non_null")]
        values: Option<[f64; 3]>,
        #[serde(default, deserialize_with = "non_null")]
        curve: Option<String>,
    },
    Weir {
        #[serde(default, deserialize_with = "non_null")]
        values: Option<[f64; 3]>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NodePatchDto {
    #[serde(default, deserialize_with = "non_null")]
    tag: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    invert_elevation: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    full_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    surcharge_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    ponded_area: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    initial_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    included_in_report: Option<bool>,
    #[serde(default, deserialize_with = "non_null")]
    shape: Option<StorageShapeDto>,
    #[serde(default, deserialize_with = "non_null")]
    evaporation_fraction: Option<f64>,
    #[serde(default, deserialize_with = "double_option")]
    exfiltration: Option<Option<StorageExfiltrationDto>>,
    #[serde(default, deserialize_with = "non_null")]
    boundary: Option<OutfallBoundaryDto>,
    #[serde(default, deserialize_with = "non_null")]
    has_flap_gate: Option<bool>,
    #[serde(default, deserialize_with = "double_option")]
    route_to_subcatchment: Option<Option<String>>,
    #[serde(default, deserialize_with = "non_null")]
    rule: Option<DividerRuleDto>,
    #[serde(default, deserialize_with = "double_option")]
    diverted_link: Option<Option<String>>,
}

fn patch_error(simulation: &Simulation, detail: impl Into<String>) -> JsValue {
    simulation.error(
        "configureNode",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

fn relation_index(
    simulation: &Simulation,
    object_type: ObjectType,
    id: &str,
) -> Result<usize, JsValue> {
    simulation
        .inner
        .object_identity_read(object_type, id)
        .map_err(|error| simulation.error("configureNode", error))?
        .map(|identity| identity.index)
        .ok_or_else(|| patch_error(simulation, format!("unknown related object: {id}")))
}

impl StorageShapeDto {
    fn into_patch(self, simulation: &Simulation) -> Result<CanonicalStorageShape, JsValue> {
        match self {
            Self::Tabular {
                coefficients,
                curve,
            } => {
                let coefficients = coefficients.unwrap_or_default();
                if coefficients != [0.0; 3] {
                    return Err(patch_error(
                        simulation,
                        "tabular shape coefficients must all be zero",
                    ));
                }
                let curve = curve.ok_or_else(|| {
                    patch_error(simulation, "tabular storage shape requires a curve")
                })?;
                Ok(CanonicalStorageShape::Tabular {
                    curve_index: relation_index(simulation, ObjectType::Curve, &curve)?,
                })
            }
            Self::Functional { coefficients } => Ok(CanonicalStorageShape::Coefficients {
                kind: StorageType::Functional,
                coefficients: coefficients.unwrap_or_default(),
            }),
            Self::Cylindrical { coefficients } => Ok(CanonicalStorageShape::Coefficients {
                kind: StorageType::Cylindrical,
                coefficients: coefficients.unwrap_or_default(),
            }),
            Self::Conical { coefficients } => Ok(CanonicalStorageShape::Coefficients {
                kind: StorageType::Conical,
                coefficients: coefficients.unwrap_or_default(),
            }),
            Self::Paraboloid { coefficients } => Ok(CanonicalStorageShape::Coefficients {
                kind: StorageType::Paraboloid,
                coefficients: coefficients.unwrap_or_default(),
            }),
            Self::Pyramidal { coefficients } => Ok(CanonicalStorageShape::Coefficients {
                kind: StorageType::Pyramidal,
                coefficients: coefficients.unwrap_or_default(),
            }),
        }
    }
}
impl StorageExfiltrationDto {
    fn into_patch(self) -> swmmrs::simulation::nodes::StorageExfiltration {
        let suction_head = self.suction_head.unwrap_or_default();
        let moisture_deficit = self.moisture_deficit.unwrap_or_default();
        if suction_head == 0.0 && moisture_deficit == 0.0 {
            swmmrs::simulation::nodes::StorageExfiltration::Seepage {
                conductivity: self.conductivity,
            }
        } else {
            swmmrs::simulation::nodes::StorageExfiltration::GreenAmpt {
                suction_head,
                conductivity: self.conductivity,
                moisture_deficit,
            }
        }
    }
}
impl OutfallBoundaryDto {
    fn into_patch(self, simulation: &Simulation) -> Result<OutfallBoundary, JsValue> {
        match self {
            Self::Free {} => Ok(OutfallBoundary::Free),
            Self::Normal {} => Ok(OutfallBoundary::Normal),
            Self::Fixed { stage } => Ok(OutfallBoundary::Fixed {
                stage: stage.unwrap_or_default(),
            }),
            Self::Tidal { reference } => {
                let reference = reference.ok_or_else(|| {
                    patch_error(simulation, "table-driven boundary requires a reference")
                })?;
                let index = relation_index(simulation, ObjectType::Curve, &reference)?;
                Ok(OutfallBoundary::Tidal { curve_index: index })
            }
            Self::TimeSeries { reference } => {
                let reference = reference.ok_or_else(|| {
                    patch_error(simulation, "table-driven boundary requires a reference")
                })?;
                let index = relation_index(simulation, ObjectType::Tseries, &reference)?;
                Ok(OutfallBoundary::TimeSeries {
                    series_index: index,
                })
            }
        }
    }
}
impl DividerRuleDto {
    fn into_patch(self, simulation: &Simulation) -> Result<DividerMode, JsValue> {
        match self {
            Self::Overflow {} => Ok(DividerMode::Overflow),
            Self::Cutoff { values } => {
                let values = values.unwrap_or_default();
                if values[1] != 0.0 || values[2] != 0.0 {
                    return Err(patch_error(
                        simulation,
                        "cutoff rule accepts only minimum flow",
                    ));
                }
                Ok(DividerMode::Cutoff {
                    minimum_flow: values[0],
                })
            }
            Self::Tabular { values, curve } => {
                let values = values.unwrap_or_default();
                if values != [0.0; 3] {
                    return Err(patch_error(simulation, "tabular rule accepts only a curve"));
                }
                let curve = curve.ok_or_else(|| {
                    patch_error(simulation, "tabular divider rule requires a curve")
                })?;
                Ok(DividerMode::Tabular {
                    curve_index: relation_index(simulation, ObjectType::Curve, &curve)?,
                })
            }
            Self::Weir { values } => {
                let values = values.unwrap_or_default();
                Ok(DividerMode::Weir {
                    minimum_flow: values[0],
                    maximum_head: values[1],
                    discharge_coefficient: values[2],
                })
            }
        }
    }
}

impl NodePatchDto {
    fn into_patch(
        self,
        simulation: &Simulation,
    ) -> Result<swmmrs::simulation::nodes::NodePatch, JsValue> {
        let storage = self.shape.is_some()
            || self.evaporation_fraction.is_some()
            || self.exfiltration.is_some();
        let outfall = self.boundary.is_some()
            || self.has_flap_gate.is_some()
            || self.route_to_subcatchment.is_some();
        let divider = self.rule.is_some() || self.diverted_link.is_some();
        if [storage, outfall, divider]
            .into_iter()
            .filter(|value| *value)
            .count()
            > 1
        {
            return Err(patch_error(
                simulation,
                "node patch accepts fields for only one concrete subtype",
            ));
        }

        let subtype = if storage {
            let shape = self
                .shape
                .map(|shape| shape.into_patch(simulation))
                .transpose()?;
            let exfiltration = match self.exfiltration {
                None => None,
                Some(None) => Some(None),
                Some(Some(value)) => Some(Some(value.into_patch())),
            };
            Some(swmmrs::simulation::nodes::NodeSubtypePatch::StorageUpdate(
                swmmrs::simulation::nodes::StorageUpdate {
                    shape,
                    evaporation_fraction: self.evaporation_fraction,
                    exfiltration,
                },
            ))
        } else if outfall {
            let boundary = self
                .boundary
                .map(|boundary| boundary.into_patch(simulation))
                .transpose()?;
            let route_to_subcatchment = match self.route_to_subcatchment {
                None => None,
                Some(None) => Some(None),
                Some(Some(value)) => Some(Some(relation_index(
                    simulation,
                    ObjectType::Subcatch,
                    &value,
                )?)),
            };
            Some(swmmrs::simulation::nodes::NodeSubtypePatch::OutfallUpdate(
                swmmrs::simulation::nodes::OutfallUpdate {
                    boundary,
                    has_flap_gate: self.has_flap_gate,
                    route_to_subcatchment,
                },
            ))
        } else if divider {
            let mode = self
                .rule
                .map(|rule| rule.into_patch(simulation))
                .transpose()?;
            let diverted_link_index = match self.diverted_link {
                None => None,
                Some(None) => Some(None),
                Some(Some(value)) => {
                    Some(Some(relation_index(simulation, ObjectType::Link, &value)?))
                }
            };
            Some(swmmrs::simulation::nodes::NodeSubtypePatch::DividerUpdate(
                swmmrs::simulation::nodes::DividerUpdate {
                    mode,
                    diverted_link_index,
                },
            ))
        } else {
            None
        };

        Ok(swmmrs::simulation::nodes::NodePatch {
            tag: self.tag,
            invert_elevation: self.invert_elevation,
            full_depth: self.full_depth,
            surcharge_depth: self.surcharge_depth,
            ponded_area: self.ponded_area,
            initial_depth: self.initial_depth,
            included_in_report: self.included_in_report,
            subtype,
        })
    }
}

fn model_time(time: DateTime) -> String {
    let (mut year, mut month, mut day, mut hour, mut minute, mut second) = (0, 0, 0, 0, 0, 0);
    datetime_decodeDate(time, &mut year, &mut month, &mut day);
    datetime_decodeTime(time, &mut hour, &mut minute, &mut second);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}")
}

fn storage_shape_value(
    simulation: &Simulation,
    shape: StorageType,
    coefficients: [f64; 3],
    curve_index: Option<usize>,
) -> Result<StorageShapeValue, JsValue> {
    let curve = if shape == StorageType::Tabular {
        Some(identity_id(
            simulation,
            ObjectType::Curve,
            curve_index
                .ok_or_else(|| patch_error(simulation, "tabular storage curve index is invalid"))?,
            "nodeConfiguration",
        )?)
    } else {
        None
    };
    Ok(StorageShapeValue {
        kind: storage_type_name(shape).into(),
        coefficients,
        curve,
    })
}

fn storage_type_name(kind: StorageType) -> &'static str {
    match kind {
        StorageType::Functional => "functional",
        StorageType::Tabular => "tabular",
        StorageType::Cylindrical => "cylindrical",
        StorageType::Conical => "conical",
        StorageType::Paraboloid => "paraboloid",
        StorageType::Pyramidal => "pyramidal",
    }
}

fn storage_exfiltration_value(value: Option<(f64, f64, f64)>) -> Option<StorageExfiltrationValue> {
    value.map(
        |(suction_head, conductivity, moisture_deficit)| StorageExfiltrationValue {
            conductivity,
            suction_head,
            moisture_deficit,
        },
    )
}

fn outfall_boundary_value(
    simulation: &Simulation,
    boundary: OutfallBoundary,
) -> Result<OutfallBoundaryValue, JsValue> {
    Ok(match boundary {
        OutfallBoundary::Free => OutfallBoundaryValue {
            kind: "free".into(),
            stage: 0.0,
            reference: None,
        },
        OutfallBoundary::Normal => OutfallBoundaryValue {
            kind: "normal".into(),
            stage: 0.0,
            reference: None,
        },
        OutfallBoundary::Fixed { stage } => OutfallBoundaryValue {
            kind: "fixed".into(),
            stage,
            reference: None,
        },
        OutfallBoundary::Tidal { curve_index } => OutfallBoundaryValue {
            kind: "tidal".into(),
            stage: 0.0,
            reference: Some(identity_id(
                simulation,
                ObjectType::Curve,
                curve_index,
                "nodeConfiguration",
            )?),
        },
        OutfallBoundary::TimeSeries { series_index } => OutfallBoundaryValue {
            kind: "timeseries".into(),
            stage: 0.0,
            reference: Some(identity_id(
                simulation,
                ObjectType::Tseries,
                series_index,
                "nodeConfiguration",
            )?),
        },
    })
}

fn divider_rule_value(
    simulation: &Simulation,
    mode: DividerMode,
) -> Result<DividerRuleValue, JsValue> {
    Ok(match mode {
        DividerMode::Overflow => DividerRuleValue {
            kind: "overflow".into(),
            values: [0.0; 3],
            curve: None,
        },
        DividerMode::Cutoff { minimum_flow } => DividerRuleValue {
            kind: "cutoff".into(),
            values: [minimum_flow, 0.0, 0.0],
            curve: None,
        },
        DividerMode::Tabular { curve_index } => DividerRuleValue {
            kind: "tabular".into(),
            values: [0.0; 3],
            curve: Some(identity_id(
                simulation,
                ObjectType::Curve,
                curve_index,
                "nodeConfiguration",
            )?),
        },
        DividerMode::Weir {
            minimum_flow,
            maximum_head,
            discharge_coefficient,
        } => DividerRuleValue {
            kind: "weir".into(),
            values: [minimum_flow, maximum_head, discharge_coefficient],
            curve: None,
        },
    })
}

fn node_configuration_value(
    simulation: &Simulation,
    values: swmmrs::simulation::nodes::NodeConfigurationRead,
) -> Result<NodeConfiguration, JsValue> {
    let (
        shape,
        evaporation_fraction,
        exfiltration,
        boundary,
        has_flap_gate,
        route_to_subcatchment,
        rule,
        diverted_link,
    ) = match values.subtype {
        NodeSubtypeConfigurationRead::Junction => (None, None, None, None, None, None, None, None),
        NodeSubtypeConfigurationRead::Storage(storage) => (
            Some(storage_shape_value(
                simulation,
                storage.shape,
                storage.coefficients,
                storage.curve_index,
            )?),
            Some(storage.evaporation_fraction),
            storage_exfiltration_value(storage.exfiltration),
            None,
            None,
            None,
            None,
            None,
        ),
        NodeSubtypeConfigurationRead::Outfall(outfall) => (
            None,
            None,
            None,
            Some(outfall_boundary_value(simulation, outfall.boundary)?),
            Some(outfall.has_flap_gate),
            outfall
                .route_to_subcatchment
                .map(|index| {
                    identity_id(simulation, ObjectType::Subcatch, index, "nodeConfiguration")
                })
                .transpose()?,
            None,
            None,
        ),
        NodeSubtypeConfigurationRead::Divider(divider) => (
            None,
            None,
            None,
            None,
            None,
            None,
            Some(divider_rule_value(simulation, divider.mode)?),
            divider
                .diverted_link_index
                .map(|index| identity_id(simulation, ObjectType::Link, index, "nodeConfiguration"))
                .transpose()?,
        ),
    };
    Ok(NodeConfiguration {
        kind: format!("{:?}", values.kind),
        tag: values.tag,
        invert_elevation: values.invert_elevation,
        full_depth: values.full_depth,
        surcharge_depth: values.surcharge_depth,
        ponded_area: values.ponded_area,
        initial_depth: values.initial_depth,
        included_in_report: values.included_in_report,
        external_inflow: values.external_inflow,
        shape,
        evaporation_fraction,
        exfiltration,
        boundary,
        has_flap_gate,
        route_to_subcatchment,
        rule,
        diverted_link,
    })
}

fn quality_record(values: NodeQualityRead) -> NodeQuality {
    NodeQuality {
        pollutant_ids: values.pollutant_ids,
        concentrations: values.concentrations,
        inflow_concentrations: values.inflow_concentrations,
        reactor_concentrations: values.reactor_concentrations,
    }
}

fn quality_snapshot_record(values: NodeQualitySnapshotRead) -> NodeQualitySnapshot {
    NodeQualitySnapshot {
        object_ids: values.object_ids,
        pollutant_ids: values.pollutant_ids,
        concentrations: values.concentrations,
        inflow_concentrations: values.inflow_concentrations,
        reactor_concentrations: values.reactor_concentrations,
    }
}

fn node_statistics_record(values: NodeStatisticsRead) -> NodeStatistics {
    NodeStatistics {
        average_depth: values.average_depth,
        maximum_depth: values.maximum_depth,
        maximum_depth_time: model_time(values.maximum_depth_time),
        maximum_reported_depth: values.maximum_reported_depth,
        flooded_volume: values.flooded_volume,
        time_flooded_seconds: values.time_flooded_seconds,
        time_surcharged_seconds: values.time_surcharged_seconds,
        time_courant_critical_seconds: values.time_courant_critical_seconds,
        total_lateral_inflow: values.total_lateral_inflow,
        maximum_lateral_inflow: values.maximum_lateral_inflow,
        maximum_inflow: values.maximum_inflow,
        maximum_overflow: values.maximum_overflow,
        maximum_ponded_volume: values.maximum_ponded_volume,
        nonconverged_count: values.nonconverged_count,
        maximum_inflow_time: model_time(values.maximum_inflow_time),
        maximum_overflow_time: model_time(values.maximum_overflow_time),
    }
}

fn storage_statistics_record(values: StorageStatisticsRead) -> StorageStatistics {
    StorageStatistics {
        initial_volume: values.initial_volume,
        average_volume: values.average_volume,
        maximum_volume: values.maximum_volume,
        maximum_inflow: values.maximum_inflow,
        evaporation_losses: values.evaporation_losses,
        exfiltration_losses: values.exfiltration_losses,
        maximum_volume_time: model_time(values.maximum_volume_time),
    }
}

fn pollutant_map(ids: Vec<String>, values: Vec<f64>) -> BTreeMap<String, f64> {
    ids.into_iter().zip(values).collect()
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = nodeConfiguration)]
    pub fn node_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .node_configuration_read(index)
            .map_err(|error| self.error("nodeConfiguration", error))?;
        serialize(&node_configuration_value(self, values)?)
    }

    #[wasm_bindgen(js_name = configureNode)]
    pub fn configure_node(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let patch: NodePatchDto = decode(patch, "configureNode")?;
        let index = self.index(ObjectType::Node, id)?;
        let patch = patch.into_patch(self)?;
        self.inner
            .patch_node(index, patch)
            .map_err(|error| self.error("configureNode", error))
    }

    #[wasm_bindgen(js_name = nodeExternalInflow)]
    pub fn node_external_inflow(&self, id: &str) -> Result<f64, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        self.inner
            .node_external_inflow_read(index)
            .map_err(|error| self.error("nodeExternalInflow", error))
    }

    #[wasm_bindgen(js_name = setOutfallStage)]
    pub fn set_outfall_stage(&mut self, id: &str, value: JsValue) -> Result<(), JsValue> {
        let operation = "setOutfallStage";
        let index = self.index(ObjectType::Node, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_outfall_fixed_stage(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = outfallFixedStage)]
    pub fn outfall_fixed_stage(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let value = self
            .inner
            .outfall_fixed_stage_read(index)
            .map_err(|error| self.error("outfallFixedStage", error))?;
        serialize(&value)
    }

    #[wasm_bindgen(js_name = nodeQuality)]
    pub fn node_quality(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .node_quality_read(index)
            .map_err(|error| self.error("nodeQuality", error))?;
        serialize(&quality_record(values))
    }

    #[wasm_bindgen(js_name = nodeQualitySnapshot)]
    pub fn node_quality_snapshot(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .node_quality_snapshot_read(ids.as_deref())
            .map_err(|error| self.error("nodeQualitySnapshot", error))?;
        serialize(&quality_snapshot_record(values))
    }

    #[wasm_bindgen(js_name = overrideNodePollutantConcentrations)]
    pub fn override_node_pollutant_concentrations(
        &mut self,
        id: &str,
        values: JsValue,
    ) -> Result<(), JsValue> {
        let values: BTreeMap<String, f64> = decode(values, "overrideNodePollutantConcentrations")?;
        let index = self.index(ObjectType::Node, id)?;
        self.inner
            .override_node_pollutant_concentrations(
                index,
                PollutantValuesPatch {
                    values: values.into_iter().collect(),
                },
            )
            .map_err(|error| self.error("overrideNodePollutantConcentrations", error))
    }

    #[wasm_bindgen(js_name = nodeExternalPollutantMassFlux)]
    pub fn node_external_pollutant_mass_flux(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .node_external_pollutant_mass_flux_read(index)
            .map_err(|error| self.error("nodeExternalPollutantMassFlux", error))?;
        serialize(&pollutant_map(values.pollutant_ids, values.values))
    }

    #[wasm_bindgen(js_name = nodeExternalPollutantMassFluxValue)]
    pub fn node_external_pollutant_mass_flux_value(
        &self,
        id: &str,
        pollutant_id: &str,
    ) -> Result<f64, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        self.inner
            .node_external_pollutant_mass_flux_value_read(index, pollutant_id)
            .map_err(|error| self.error("nodeExternalPollutantMassFluxValue", error))
    }

    #[wasm_bindgen(js_name = updateNodeExternalPollutantMassFlux)]
    pub fn update_node_external_pollutant_mass_flux(
        &mut self,
        id: &str,
        values: JsValue,
        replace: bool,
    ) -> Result<(), JsValue> {
        let values: BTreeMap<String, f64> = decode(values, "updateNodeExternalPollutantMassFlux")?;
        let index = self.index(ObjectType::Node, id)?;
        self.inner
            .patch_node_external_pollutant_mass_flux(
                index,
                PersistentPollutantValuesPatch {
                    values: values.into_iter().collect(),
                    replace,
                },
            )
            .map_err(|error| self.error("updateNodeExternalPollutantMassFlux", error))
    }

    #[wasm_bindgen(js_name = nodeStatistics)]
    pub fn node_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .node_statistics_read(index)
            .map_err(|error| self.error("nodeStatistics", error))?;
        serialize(&node_statistics_record(values))
    }

    #[wasm_bindgen(js_name = storageStatistics)]
    pub fn storage_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .storage_statistics_read(index)
            .map_err(|error| self.error("storageStatistics", error))?;
        serialize(&storage_statistics_record(values))
    }

    #[wasm_bindgen(js_name = outfallStatistics)]
    pub fn outfall_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values: OutfallStatisticsRead = self
            .inner
            .outfall_statistics_read(index)
            .map_err(|error| self.error("outfallStatistics", error))?;
        serialize(&OutfallStatistics {
            average_flow: values.average_flow,
            maximum_flow: values.maximum_flow,
            pollutant_loads: pollutant_map(
                values
                    .pollutant_loads
                    .iter()
                    .map(|(id, _)| id.clone())
                    .collect(),
                values
                    .pollutant_loads
                    .iter()
                    .map(|(_, value)| *value)
                    .collect(),
            ),
            period_count: values.period_count,
        })
    }

    #[wasm_bindgen(js_name = nodeStatisticsSnapshot)]
    pub fn node_statistics_snapshot(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values: NodeStatisticsSnapshotRead = self
            .inner
            .node_statistics_snapshot_read(ids.as_deref())
            .map_err(|error| self.error("nodeStatisticsSnapshot", error))?;
        serialize(&NodeStatisticsSnapshot {
            object_ids: values.object_ids,
            average_depth: values.average_depth,
            maximum_depth: values.maximum_depth,
            maximum_depth_time: values
                .maximum_depth_time
                .into_iter()
                .map(model_time)
                .collect(),
            maximum_reported_depth: values.maximum_reported_depth,
            flooded_volume: values.flooded_volume,
            time_flooded_seconds: values.time_flooded_seconds,
            time_surcharged_seconds: values.time_surcharged_seconds,
            time_courant_critical_seconds: values.time_courant_critical_seconds,
            total_lateral_inflow: values.total_lateral_inflow,
            maximum_lateral_inflow: values.maximum_lateral_inflow,
            maximum_inflow: values.maximum_inflow,
            maximum_overflow: values.maximum_overflow,
            maximum_ponded_volume: values.maximum_ponded_volume,
            nonconverged_count: values.nonconverged_count,
            maximum_inflow_time: values
                .maximum_inflow_time
                .into_iter()
                .map(model_time)
                .collect(),
            maximum_overflow_time: values
                .maximum_overflow_time
                .into_iter()
                .map(model_time)
                .collect(),
        })
    }

    #[wasm_bindgen(js_name = nodeTotalInflowVolume)]
    pub fn node_total_inflow_volume(&self, id: &str) -> Result<f64, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        self.inner
            .node_total_inflow_volume_read(index)
            .map_err(|error| self.error("nodeTotalInflowVolume", error))
    }
}
