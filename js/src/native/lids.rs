//! Browser-facing LID controls, units, and runtime snapshots.

use js_sys::Reflect;
use serde::{Deserialize, Serialize};
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::simulation::lids::{
    LidControlRead, LidDrainDestination, LidDrainDestinationRead, LidDrainPatch,
    LidDrainageMatPatch, LidLayerPatch, LidPavementPatch, LidSoilPatch, LidStoragePatch,
    LidSurfacePatch, LidUnitPatch, LidUnitRead, LidUnitSnapshotRead, SubcatchmentLidSnapshotRead,
};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidRelationship {
    kind: &'static str,
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidUnitConfiguration {
    area: f64,
    full_width: f64,
    bottom_width: f64,
    initial_saturation: f64,
    impervious_runoff_treated: f64,
    pervious_runoff_treated: f64,
    control: LidRelationship,
    count: i32,
    routes_to_pervious: bool,
    drain_destination: Option<LidRelationship>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidSurfaceConfiguration {
    thickness: f64,
    vegetation_volume_fraction: f64,
    roughness: f64,
    slope: f64,
    side_slope: f64,
    alpha: f64,
    immediate_overflow: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidSoilConfiguration {
    thickness: f64,
    porosity: f64,
    field_capacity: f64,
    wilting_point: f64,
    saturated_conductivity: f64,
    conductivity_slope: f64,
    suction_head: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidStorageConfiguration {
    thickness: f64,
    void_ratio: f64,
    saturated_conductivity: f64,
    clogging_factor: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidPavementConfiguration {
    thickness: f64,
    void_ratio: f64,
    impervious_fraction: f64,
    saturated_conductivity: f64,
    clogging_factor: f64,
    regeneration_interval_seconds: f64,
    regeneration_fraction: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidDrainConfiguration {
    coefficient: f64,
    exponent: f64,
    offset: f64,
    delay_seconds: f64,
    open_head: f64,
    close_head: f64,
    control_curve: Option<LidRelationship>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidDrainageMatConfiguration {
    thickness: f64,
    void_fraction: f64,
    roughness: f64,
    alpha: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidControlConfiguration {
    surface: Option<LidSurfaceConfiguration>,
    soil: Option<LidSoilConfiguration>,
    storage: Option<LidStorageConfiguration>,
    pavement: Option<LidPavementConfiguration>,
    drain: Option<LidDrainConfiguration>,
    drainage_mat: Option<LidDrainageMatConfiguration>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LidUnitSnapshot {
    inflow: f64,
    evaporation: f64,
    infiltration: f64,
    surface_outflow: f64,
    drain_outflow: f64,
    initial_volume: f64,
    final_volume: f64,
    surface_depth: f64,
    pavement_depth: f64,
    storage_depth: f64,
    soil_moisture: f64,
    dry_time_seconds: f64,
    old_drain_flow: f64,
    new_drain_flow: f64,
    evaporation_rate: f64,
    maximum_native_infiltration_rate: f64,
    surface_inflow_rate: f64,
    surface_infiltration_rate: f64,
    surface_evaporation_rate: f64,
    surface_outflow_rate: f64,
    pavement_evaporation_rate: f64,
    pavement_percolation_rate: f64,
    soil_evaporation_rate: f64,
    soil_percolation_rate: f64,
    storage_inflow_rate: f64,
    storage_exfiltration_rate: f64,
    storage_evaporation_rate: f64,
    storage_drain_rate: f64,
    surface_flux_rate: f64,
    soil_flux_rate: f64,
    storage_flux_rate: f64,
    pavement_flux_rate: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentLidSnapshot {
    pervious_area: f64,
    flow_to_pervious_area: f64,
    old_drain_flow: f64,
    new_drain_flow: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidRelationshipPatch {
    kind: String,
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidUnitPatchDto {
    area: Option<f64>,
    full_width: Option<f64>,
    initial_saturation: Option<f64>,
    impervious_runoff_treated: Option<f64>,
    pervious_runoff_treated: Option<f64>,
    control: Option<String>,
    count: Option<i32>,
    routes_to_pervious: Option<bool>,
    drain_destination: Option<LidRelationshipPatch>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidSurfacePatchDto {
    thickness: Option<f64>,
    vegetation_volume_fraction: Option<f64>,
    roughness: Option<f64>,
    slope: Option<f64>,
    side_slope: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidSoilPatchDto {
    thickness: Option<f64>,
    porosity: Option<f64>,
    field_capacity: Option<f64>,
    wilting_point: Option<f64>,
    saturated_conductivity: Option<f64>,
    conductivity_slope: Option<f64>,
    suction_head: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidStoragePatchDto {
    thickness: Option<f64>,
    void_ratio: Option<f64>,
    saturated_conductivity: Option<f64>,
    clogging_factor: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidPavementPatchDto {
    thickness: Option<f64>,
    void_ratio: Option<f64>,
    impervious_fraction: Option<f64>,
    saturated_conductivity: Option<f64>,
    clogging_factor: Option<f64>,
    regeneration_interval_seconds: Option<f64>,
    regeneration_fraction: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidDrainPatchDto {
    coefficient: Option<f64>,
    exponent: Option<f64>,
    offset: Option<f64>,
    delay_seconds: Option<f64>,
    open_head: Option<f64>,
    close_head: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LidDrainageMatPatchDto {
    thickness: Option<f64>,
    void_fraction: Option<f64>,
    roughness: Option<f64>,
}

fn invalid(operation: &str, detail: impl Into<String>) -> JsValue {
    let error = js_sys::Error::new(&format!("{operation}: {}", detail.into()));
    let _ = Reflect::set(
        &error,
        &JsValue::from_str("code"),
        &ErrorCode::ApiPropertyValue.as_i32().into(),
    );
    let _ = Reflect::set(&error, &JsValue::from_str("operation"), &operation.into());
    error.into()
}

fn has_property(value: &JsValue, name: &str) -> Result<bool, JsValue> {
    Reflect::has(value, &JsValue::from_str(name))
        .map_err(|_| invalid("decode", "invalid patch object"))
}

/// Rejects explicit null/undefined for scalar sparse-patch fields. `null` is
/// meaningful only for the optional drain destination, where it clears the
/// retained explicit relationship.
fn reject_null_fields(value: &JsValue, fields: &[&str], operation: &str) -> Result<(), JsValue> {
    for field in fields {
        if !has_property(value, field)? {
            continue;
        }
        let item = Reflect::get(value, &JsValue::from_str(field))
            .map_err(|_| invalid(operation, "invalid patch object"))?;
        if item.is_null() || item.is_undefined() {
            return Err(invalid(operation, format!("{field} must not be null")));
        }
    }
    Ok(())
}

fn relation(kind: &'static str, id: String) -> LidRelationship {
    LidRelationship { kind, id }
}

fn destination_read(value: LidDrainDestinationRead) -> Option<LidRelationship> {
    match value {
        LidDrainDestinationRead::None => None,
        LidDrainDestinationRead::Node(identity) => Some(relation("node", identity.id)),
        LidDrainDestinationRead::Subcatchment(identity) => {
            Some(relation("subcatchment", identity.id))
        }
    }
}

fn unit_configuration(value: LidUnitRead) -> LidUnitConfiguration {
    LidUnitConfiguration {
        area: value.area,
        full_width: value.full_width,
        bottom_width: value.bottom_width,
        initial_saturation: value.initial_saturation,
        impervious_runoff_treated: value.impervious_runoff_treated,
        pervious_runoff_treated: value.pervious_runoff_treated,
        control: relation("lid", value.control.id),
        count: value.count,
        routes_to_pervious: value.routes_to_pervious,
        drain_destination: destination_read(value.drain_destination),
    }
}

fn control_configuration(value: LidControlRead) -> LidControlConfiguration {
    LidControlConfiguration {
        surface: value.surface.map(|layer| LidSurfaceConfiguration {
            thickness: layer.thickness,
            vegetation_volume_fraction: layer.vegetation_volume_fraction,
            roughness: layer.roughness,
            slope: layer.slope,
            side_slope: layer.side_slope,
            alpha: layer.alpha,
            immediate_overflow: layer.immediate_overflow,
        }),
        soil: value.soil.map(|layer| LidSoilConfiguration {
            thickness: layer.thickness,
            porosity: layer.porosity,
            field_capacity: layer.field_capacity,
            wilting_point: layer.wilting_point,
            saturated_conductivity: layer.saturated_conductivity,
            conductivity_slope: layer.conductivity_slope,
            suction_head: layer.suction_head,
        }),
        storage: value.storage.map(|layer| LidStorageConfiguration {
            thickness: layer.thickness,
            void_ratio: layer.void_ratio,
            saturated_conductivity: layer.saturated_conductivity,
            clogging_factor: layer.clogging_factor,
        }),
        pavement: value.pavement.map(|layer| LidPavementConfiguration {
            thickness: layer.thickness,
            void_ratio: layer.void_ratio,
            impervious_fraction: layer.impervious_fraction,
            saturated_conductivity: layer.saturated_conductivity,
            clogging_factor: layer.clogging_factor,
            regeneration_interval_seconds: layer.regeneration_seconds,
            regeneration_fraction: layer.regeneration_fraction,
        }),
        drain: value.drain.map(|layer| LidDrainConfiguration {
            coefficient: layer.coefficient,
            exponent: layer.exponent,
            offset: layer.offset,
            delay_seconds: layer.delay_seconds,
            open_head: layer.open_head,
            close_head: layer.close_head,
            control_curve: layer
                .control_curve
                .map(|identity| relation("curve", identity.id)),
        }),
        drainage_mat: value.drainage_mat.map(|layer| LidDrainageMatConfiguration {
            thickness: layer.thickness,
            void_fraction: layer.void_fraction,
            roughness: layer.roughness,
            alpha: layer.alpha,
        }),
    }
}

fn unit_snapshot(value: LidUnitSnapshotRead) -> LidUnitSnapshot {
    LidUnitSnapshot {
        inflow: value.inflow,
        evaporation: value.evaporation,
        infiltration: value.infiltration,
        surface_outflow: value.surface_outflow,
        drain_outflow: value.drain_outflow,
        initial_volume: value.initial_volume,
        final_volume: value.final_volume,
        surface_depth: value.surface_depth,
        pavement_depth: value.pavement_depth,
        storage_depth: value.storage_depth,
        soil_moisture: value.soil_moisture,
        dry_time_seconds: value.dry_time_seconds,
        old_drain_flow: value.old_drain_flow,
        new_drain_flow: value.new_drain_flow,
        evaporation_rate: value.evaporation_rate,
        maximum_native_infiltration_rate: value.maximum_native_infiltration_rate,
        surface_inflow_rate: value.surface_inflow_rate,
        surface_infiltration_rate: value.surface_infiltration_rate,
        surface_evaporation_rate: value.surface_evaporation_rate,
        surface_outflow_rate: value.surface_outflow_rate,
        pavement_evaporation_rate: value.pavement_evaporation_rate,
        pavement_percolation_rate: value.pavement_percolation_rate,
        soil_evaporation_rate: value.soil_evaporation_rate,
        soil_percolation_rate: value.soil_percolation_rate,
        storage_inflow_rate: value.storage_inflow_rate,
        storage_exfiltration_rate: value.storage_exfiltration_rate,
        storage_evaporation_rate: value.storage_evaporation_rate,
        storage_drain_rate: value.storage_drain_rate,
        surface_flux_rate: value.surface_flux_rate,
        soil_flux_rate: value.soil_flux_rate,
        storage_flux_rate: value.storage_flux_rate,
        pavement_flux_rate: value.pavement_flux_rate,
    }
}

fn group_snapshot(value: SubcatchmentLidSnapshotRead) -> SubcatchmentLidSnapshot {
    SubcatchmentLidSnapshot {
        pervious_area: value.pervious_area,
        flow_to_pervious_area: value.flow_to_pervious_area,
        old_drain_flow: value.old_drain_flow,
        new_drain_flow: value.new_drain_flow,
    }
}

fn resolve_id(
    simulation: &Simulation,
    object_type: ObjectType,
    id: &str,
    operation: &str,
) -> Result<usize, JsValue> {
    simulation
        .inner
        .object_identity_read(object_type, id)
        .map_err(|error| simulation.error(operation, error))?
        .map(|identity| identity.index)
        .ok_or_else(|| {
            simulation.error(
                operation,
                SwmmError::with_detail(ErrorCode::ApiObjectIndex, format!("unknown object: {id}")),
            )
        })
}

fn resolve_destination(
    simulation: &Simulation,
    value: LidRelationshipPatch,
    operation: &str,
) -> Result<LidDrainDestination, JsValue> {
    match value.kind.as_str() {
        "node" => Ok(LidDrainDestination::Node(resolve_id(
            simulation,
            ObjectType::Node,
            &value.id,
            operation,
        )?)),
        "subcatchment" => Ok(LidDrainDestination::Subcatchment(resolve_id(
            simulation,
            ObjectType::Subcatch,
            &value.id,
            operation,
        )?)),
        _ => Err(invalid(
            operation,
            "drainDestination.kind must be node or subcatchment",
        )),
    }
}

fn unit_patch(
    simulation: &Simulation,
    value: LidUnitPatchDto,
    raw: &JsValue,
    operation: &str,
) -> Result<LidUnitPatch, JsValue> {
    reject_null_fields(
        raw,
        &[
            "area",
            "fullWidth",
            "initialSaturation",
            "imperviousRunoffTreated",
            "perviousRunoffTreated",
            "control",
            "count",
            "routesToPervious",
        ],
        operation,
    )?;
    let control_index = value
        .control
        .as_deref()
        .map(|id| resolve_id(simulation, ObjectType::Lid, id, operation))
        .transpose()?;
    let drain_destination = if has_property(raw, "drainDestination")? {
        let raw_destination = Reflect::get(raw, &JsValue::from_str("drainDestination"))
            .map_err(|_| invalid(operation, "invalid patch object"))?;
        if raw_destination.is_undefined() {
            return Err(invalid(operation, "drainDestination must not be undefined"));
        }
        Some(match value.drain_destination {
            Some(destination) => resolve_destination(simulation, destination, operation)?,
            None => LidDrainDestination::None,
        })
    } else {
        None
    };
    Ok(LidUnitPatch {
        area: value.area,
        full_width: value.full_width,
        initial_saturation: value.initial_saturation,
        impervious_runoff_treated: value.impervious_runoff_treated,
        pervious_runoff_treated: value.pervious_runoff_treated,
        control_index,
        count: value.count,
        routes_to_pervious: value.routes_to_pervious,
        drain_destination,
    })
}

fn layer_patch(layer: &str, raw: JsValue, operation: &str) -> Result<LidLayerPatch, JsValue> {
    match layer {
        "surface" => {
            reject_null_fields(
                &raw,
                &[
                    "thickness",
                    "vegetationVolumeFraction",
                    "roughness",
                    "slope",
                    "sideSlope",
                ],
                operation,
            )?;
            let value: LidSurfacePatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::Surface(LidSurfacePatch {
                thickness: value.thickness,
                vegetation_volume_fraction: value.vegetation_volume_fraction,
                roughness: value.roughness,
                slope: value.slope,
                side_slope: value.side_slope,
            }))
        }
        "soil" => {
            reject_null_fields(
                &raw,
                &[
                    "thickness",
                    "porosity",
                    "fieldCapacity",
                    "wiltingPoint",
                    "saturatedConductivity",
                    "conductivitySlope",
                    "suctionHead",
                ],
                operation,
            )?;
            let value: LidSoilPatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::Soil(LidSoilPatch {
                thickness: value.thickness,
                porosity: value.porosity,
                field_capacity: value.field_capacity,
                wilting_point: value.wilting_point,
                saturated_conductivity: value.saturated_conductivity,
                conductivity_slope: value.conductivity_slope,
                suction_head: value.suction_head,
            }))
        }
        "storage" => {
            reject_null_fields(
                &raw,
                &[
                    "thickness",
                    "voidRatio",
                    "saturatedConductivity",
                    "cloggingFactor",
                ],
                operation,
            )?;
            let value: LidStoragePatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::Storage(LidStoragePatch {
                thickness: value.thickness,
                void_ratio: value.void_ratio,
                saturated_conductivity: value.saturated_conductivity,
                clogging_factor: value.clogging_factor,
            }))
        }
        "pavement" => {
            reject_null_fields(
                &raw,
                &[
                    "thickness",
                    "voidRatio",
                    "imperviousFraction",
                    "saturatedConductivity",
                    "cloggingFactor",
                    "regenerationIntervalSeconds",
                    "regenerationFraction",
                ],
                operation,
            )?;
            let value: LidPavementPatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::Pavement(LidPavementPatch {
                thickness: value.thickness,
                void_ratio: value.void_ratio,
                impervious_fraction: value.impervious_fraction,
                saturated_conductivity: value.saturated_conductivity,
                clogging_factor: value.clogging_factor,
                regeneration_interval_seconds: value.regeneration_interval_seconds,
                regeneration_fraction: value.regeneration_fraction,
            }))
        }
        "drain" => {
            reject_null_fields(
                &raw,
                &[
                    "coefficient",
                    "exponent",
                    "offset",
                    "delaySeconds",
                    "openHead",
                    "closeHead",
                ],
                operation,
            )?;
            let value: LidDrainPatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::Drain(LidDrainPatch {
                coefficient: value.coefficient,
                exponent: value.exponent,
                offset: value.offset,
                delay_seconds: value.delay_seconds,
                open_head: value.open_head,
                close_head: value.close_head,
            }))
        }
        "drainage_mat" => {
            reject_null_fields(&raw, &["thickness", "voidFraction", "roughness"], operation)?;
            let value: LidDrainageMatPatchDto = decode(raw, operation)?;
            Ok(LidLayerPatch::DrainageMat(LidDrainageMatPatch {
                thickness: value.thickness,
                void_fraction: value.void_fraction,
                roughness: value.roughness,
            }))
        }
        _ => Err(invalid(operation, format!("unknown LID layer: {layer}"))),
    }
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = lidControlConfiguration)]
    pub fn lid_control_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = resolve_id(self, ObjectType::Lid, id, "lidControlConfiguration")?;
        let values = self
            .inner
            .lid_control_configuration_read(index)
            .map_err(|error| self.error("lidControlConfiguration", error))?;
        serialize(&control_configuration(values))
    }

    #[wasm_bindgen(js_name = configureLidControl)]
    pub fn configure_lid_control(
        &mut self,
        id: &str,
        layer: &str,
        patch: JsValue,
    ) -> Result<(), JsValue> {
        let index = resolve_id(self, ObjectType::Lid, id, "configureLidControl")?;
        let patch = layer_patch(layer, patch, "configureLidControl")?;
        self.inner
            .patch_lid_control_layer(index, patch)
            .map_err(|error| self.error("configureLidControl", error))
    }

    #[wasm_bindgen(js_name = lidUnitCount)]
    pub fn lid_unit_count(&self, subcatchment_id: &str) -> Result<usize, JsValue> {
        let index = resolve_id(self, ObjectType::Subcatch, subcatchment_id, "lidUnitCount")?;
        self.inner
            .lid_unit_count_read(index)
            .map_err(|error| self.error("lidUnitCount", error))
    }

    #[wasm_bindgen(js_name = lidUnitConfiguration)]
    pub fn lid_unit_configuration(
        &self,
        subcatchment_id: &str,
        unit_index: usize,
    ) -> Result<JsValue, JsValue> {
        let index = resolve_id(
            self,
            ObjectType::Subcatch,
            subcatchment_id,
            "lidUnitConfiguration",
        )?;
        let values = self
            .inner
            .lid_unit_configuration_read(index, unit_index)
            .map_err(|error| self.error("lidUnitConfiguration", error))?;
        serialize(&unit_configuration(values))
    }

    #[wasm_bindgen(js_name = configureLidUnit)]
    pub fn configure_lid_unit(
        &mut self,
        subcatchment_id: &str,
        unit_index: usize,
        patch: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "configureLidUnit";
        let index = resolve_id(self, ObjectType::Subcatch, subcatchment_id, operation)?;
        let value: LidUnitPatchDto = decode(patch.clone(), operation)?;
        let value = unit_patch(self, value, &patch, operation)?;
        self.inner
            .patch_lid_unit(index, unit_index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = lidUnitSnapshot)]
    pub fn lid_unit_snapshot(
        &self,
        subcatchment_id: &str,
        unit_index: usize,
    ) -> Result<JsValue, JsValue> {
        let operation = "lidUnitSnapshot";
        let index = resolve_id(self, ObjectType::Subcatch, subcatchment_id, operation)?;
        let values = self
            .inner
            .lid_unit_snapshot_read(index, unit_index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&unit_snapshot(values))
    }

    #[wasm_bindgen(js_name = subcatchmentLidSnapshot)]
    pub fn subcatchment_lid_snapshot(&self, subcatchment_id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentLidSnapshot";
        let index = resolve_id(self, ObjectType::Subcatch, subcatchment_id, operation)?;
        let values = self
            .inner
            .subcatchment_lid_snapshot_read(index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&group_snapshot(values))
    }
}
