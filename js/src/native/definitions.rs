//! Browser-facing adapters for shared SWMM definition families.

use serde::{Deserialize, Deserializer, Serialize};
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::simulation::definitions::{
    AquiferConfigurationField, AquiferConfigurationValue, AquiferPatch, SnowmeltConfigurationField,
    SnowmeltConfigurationValue, SnowmeltPatch, SnowmeltSurface, SnowmeltSurfaceConfiguration,
    SnowmeltSurfaceField,
};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, identity_id, non_null, serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AquiferConfigurationRecord {
    porosity: f64,
    wilting_point: f64,
    field_capacity: f64,
    hydraulic_conductivity: f64,
    conductivity_slope: f64,
    tension_slope: f64,
    upper_evaporation_fraction: f64,
    lower_evaporation_depth: f64,
    lower_loss_coefficient: f64,
    bottom_elevation: f64,
    water_table_elevation: f64,
    upper_moisture: f64,
    upper_evaporation_pattern: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AquiferPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    porosity: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    wilting_point: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    field_capacity: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    hydraulic_conductivity: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    conductivity_slope: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    tension_slope: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    upper_evaporation_fraction: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    lower_evaporation_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    lower_loss_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    bottom_elevation: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    water_table_elevation: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    upper_moisture: Option<f64>,
    #[serde(default, deserialize_with = "optional_field")]
    upper_evaporation_pattern: Option<Option<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnowmeltSurfaceConfigurationRecord {
    minimum_melt_coefficient: f64,
    maximum_melt_coefficient: f64,
    base_temperature: f64,
    free_water_fraction: f64,
    initial_snow_depth: f64,
    initial_free_water: f64,
    snow_depth_for_full_coverage: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnowmeltSurfacePatchDto {
    #[serde(default, deserialize_with = "non_null")]
    minimum_melt_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    maximum_melt_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    base_temperature: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    free_water_fraction: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    initial_snow_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    initial_free_water: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    snow_depth_for_full_coverage: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnowmeltParameterSetConfigurationRecord {
    plowable_fraction: f64,
    plowable: SnowmeltSurfaceConfigurationRecord,
    impervious: SnowmeltSurfaceConfigurationRecord,
    pervious: SnowmeltSurfaceConfigurationRecord,
    plow_depth: f64,
    removal_fractions: [f64; 5],
    removal_subcatchment: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnowmeltPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    plowable_fraction: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    plow_depth: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    removal_fractions: Option<[f64; 5]>,
    #[serde(default, deserialize_with = "optional_field")]
    removal_subcatchment: Option<Option<String>>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = aquiferConfiguration)]
    pub fn aquifer_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "aquiferConfiguration";
        let index = self.index(ObjectType::Aquifer, id)?;
        let porosity = aquifer_number(self, index, AquiferConfigurationField::Porosity, operation)?;
        let wilting_point = aquifer_number(
            self,
            index,
            AquiferConfigurationField::WiltingPoint,
            operation,
        )?;
        let field_capacity = aquifer_number(
            self,
            index,
            AquiferConfigurationField::FieldCapacity,
            operation,
        )?;
        let hydraulic_conductivity = aquifer_number(
            self,
            index,
            AquiferConfigurationField::HydraulicConductivity,
            operation,
        )?;
        let conductivity_slope = aquifer_number(
            self,
            index,
            AquiferConfigurationField::ConductivitySlope,
            operation,
        )?;
        let tension_slope = aquifer_number(
            self,
            index,
            AquiferConfigurationField::TensionSlope,
            operation,
        )?;
        let upper_evaporation_fraction = aquifer_number(
            self,
            index,
            AquiferConfigurationField::UpperEvaporationFraction,
            operation,
        )?;
        let lower_evaporation_depth = aquifer_number(
            self,
            index,
            AquiferConfigurationField::LowerEvaporationDepth,
            operation,
        )?;
        let lower_loss_coefficient = aquifer_number(
            self,
            index,
            AquiferConfigurationField::LowerLossCoefficient,
            operation,
        )?;
        let bottom_elevation = aquifer_number(
            self,
            index,
            AquiferConfigurationField::BottomElevation,
            operation,
        )?;
        let water_table_elevation = aquifer_number(
            self,
            index,
            AquiferConfigurationField::WaterTableElevation,
            operation,
        )?;
        let upper_moisture = aquifer_number(
            self,
            index,
            AquiferConfigurationField::UpperMoisture,
            operation,
        )?;
        let upper_evaporation_pattern = match self
            .inner
            .aquifer_configuration_value_read(
                index,
                AquiferConfigurationField::UpperEvaporationPattern,
            )
            .map_err(|error| self.error(operation, error))?
        {
            AquiferConfigurationValue::UpperEvaporationPattern(index) => index
                .map(|index| identity_id(self, ObjectType::Timepattern, index, operation))
                .transpose()?,
            AquiferConfigurationValue::Number(_) => {
                return Err(invalid_value(
                    self,
                    operation,
                    "aquifer pattern read returned a number",
                ));
            }
        };
        serialize(&AquiferConfigurationRecord {
            porosity,
            wilting_point,
            field_capacity,
            hydraulic_conductivity,
            conductivity_slope,
            tension_slope,
            upper_evaporation_fraction,
            lower_evaporation_depth,
            lower_loss_coefficient,
            bottom_elevation,
            water_table_elevation,
            upper_moisture,
            upper_evaporation_pattern,
        })
    }

    #[wasm_bindgen(js_name = configureAquifer)]
    pub fn configure_aquifer(&mut self, id: &str, value: JsValue) -> Result<(), JsValue> {
        let operation = "configureAquifer";
        let patch: AquiferPatchDto = decode(value, operation)?;
        let index = self.index(ObjectType::Aquifer, id)?;
        let upper_evaporation_pattern_index = match patch.upper_evaporation_pattern {
            None => None,
            Some(None) => Some(None),
            Some(Some(pattern)) => Some(Some(self.index(ObjectType::Timepattern, &pattern)?)),
        };
        self.inner
            .patch_aquifer(
                index,
                AquiferPatch {
                    porosity: patch.porosity,
                    wilting_point: patch.wilting_point,
                    field_capacity: patch.field_capacity,
                    hydraulic_conductivity: patch.hydraulic_conductivity,
                    conductivity_slope: patch.conductivity_slope,
                    tension_slope: patch.tension_slope,
                    upper_evaporation_fraction: patch.upper_evaporation_fraction,
                    lower_evaporation_depth: patch.lower_evaporation_depth,
                    lower_loss_coefficient: patch.lower_loss_coefficient,
                    bottom_elevation: patch.bottom_elevation,
                    water_table_elevation: patch.water_table_elevation,
                    upper_moisture: patch.upper_moisture,
                    upper_evaporation_pattern_index,
                },
            )
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = snowmeltConfiguration)]
    pub fn snowmelt_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "snowmeltConfiguration";
        let index = self.index(ObjectType::Snowmelt, id)?;
        let plowable_fraction = snowmelt_number(
            self,
            index,
            SnowmeltConfigurationField::PlowableFraction,
            operation,
        )?;
        let plow_depth = snowmelt_number(
            self,
            index,
            SnowmeltConfigurationField::PlowDepth,
            operation,
        )?;
        let removal_fractions = match self
            .inner
            .snowmelt_configuration_value_read(index, SnowmeltConfigurationField::RemovalFractions)
            .map_err(|error| self.error(operation, error))?
        {
            SnowmeltConfigurationValue::RemovalFractions(values) => values,
            SnowmeltConfigurationValue::Number(_)
            | SnowmeltConfigurationValue::RemovalSubcatchment(_) => {
                return Err(invalid_value(
                    self,
                    operation,
                    "snowmelt removal fractions read returned the wrong value",
                ));
            }
        };
        let removal_subcatchment = match self
            .inner
            .snowmelt_configuration_value_read(
                index,
                SnowmeltConfigurationField::RemovalSubcatchment,
            )
            .map_err(|error| self.error(operation, error))?
        {
            SnowmeltConfigurationValue::RemovalSubcatchment(index) => index
                .map(|index| identity_id(self, ObjectType::Subcatch, index, operation))
                .transpose()?,
            SnowmeltConfigurationValue::Number(_)
            | SnowmeltConfigurationValue::RemovalFractions(_) => {
                return Err(invalid_value(
                    self,
                    operation,
                    "snowmelt removal subcatchment read returned the wrong value",
                ));
            }
        };
        serialize(&SnowmeltParameterSetConfigurationRecord {
            plowable_fraction,
            plowable: snowmelt_surface_configuration_record(
                self,
                index,
                SnowmeltSurface::Plowable,
                operation,
            )?,
            impervious: snowmelt_surface_configuration_record(
                self,
                index,
                SnowmeltSurface::Impervious,
                operation,
            )?,
            pervious: snowmelt_surface_configuration_record(
                self,
                index,
                SnowmeltSurface::Pervious,
                operation,
            )?,
            plow_depth,
            removal_fractions,
            removal_subcatchment,
        })
    }

    #[wasm_bindgen(js_name = configureSnowmelt)]
    pub fn configure_snowmelt(&mut self, id: &str, value: JsValue) -> Result<(), JsValue> {
        let operation = "configureSnowmelt";
        let patch: SnowmeltPatchDto = decode(value, operation)?;
        let index = self.index(ObjectType::Snowmelt, id)?;
        let removal_subcatchment_index = match patch.removal_subcatchment {
            None => None,
            Some(None) => Some(None),
            Some(Some(subcatchment)) => {
                Some(Some(self.index(ObjectType::Subcatch, &subcatchment)?))
            }
        };
        self.inner
            .patch_snowmelt(
                index,
                SnowmeltPatch {
                    plowable_fraction: patch.plowable_fraction,
                    surface: None,
                    plow_depth: patch.plow_depth,
                    removal_fractions: patch.removal_fractions,
                    removal_subcatchment_index,
                },
            )
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = snowmeltSurfaceConfiguration)]
    pub fn snowmelt_surface_configuration(
        &self,
        id: &str,
        surface: &str,
    ) -> Result<JsValue, JsValue> {
        let operation = "snowmeltSurfaceConfiguration";
        let index = self.index(ObjectType::Snowmelt, id)?;
        let surface = parse_surface(self, surface, operation)?;
        serialize(&snowmelt_surface_configuration_record(
            self, index, surface, operation,
        )?)
    }

    #[wasm_bindgen(js_name = configureSnowmeltSurface)]
    pub fn configure_snowmelt_surface(
        &mut self,
        id: &str,
        surface: &str,
        value: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "configureSnowmeltSurface";
        let patch: SnowmeltSurfacePatchDto = decode(value, operation)?;
        let index = self.index(ObjectType::Snowmelt, id)?;
        let surface = parse_surface(self, surface, operation)?;
        if patch.is_empty() {
            return Ok(());
        }
        let current = snowmelt_surface_configuration(self, index, surface, operation)?;
        self.inner
            .patch_snowmelt(
                index,
                SnowmeltPatch {
                    surface: Some((
                        surface,
                        SnowmeltSurfaceConfiguration {
                            minimum_melt_coefficient: patch
                                .minimum_melt_coefficient
                                .unwrap_or(current.minimum_melt_coefficient),
                            maximum_melt_coefficient: patch
                                .maximum_melt_coefficient
                                .unwrap_or(current.maximum_melt_coefficient),
                            base_temperature: patch
                                .base_temperature
                                .unwrap_or(current.base_temperature),
                            free_water_fraction: patch
                                .free_water_fraction
                                .unwrap_or(current.free_water_fraction),
                            initial_snow_depth: patch
                                .initial_snow_depth
                                .unwrap_or(current.initial_snow_depth),
                            initial_free_water: patch
                                .initial_free_water
                                .unwrap_or(current.initial_free_water),
                            snow_depth_for_full_coverage: patch
                                .snow_depth_for_full_coverage
                                .unwrap_or(current.snow_depth_for_full_coverage),
                        },
                    )),
                    ..SnowmeltPatch::default()
                },
            )
            .map_err(|error| self.error(operation, error))
    }
}

fn optional_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

impl SnowmeltSurfacePatchDto {
    fn is_empty(&self) -> bool {
        self.minimum_melt_coefficient.is_none()
            && self.maximum_melt_coefficient.is_none()
            && self.base_temperature.is_none()
            && self.free_water_fraction.is_none()
            && self.initial_snow_depth.is_none()
            && self.initial_free_water.is_none()
            && self.snow_depth_for_full_coverage.is_none()
    }
}

fn aquifer_number(
    simulation: &Simulation,
    index: usize,
    field: AquiferConfigurationField,
    operation: &str,
) -> Result<f64, JsValue> {
    match simulation
        .inner
        .aquifer_configuration_value_read(index, field)
        .map_err(|error| simulation.error(operation, error))?
    {
        AquiferConfigurationValue::Number(value) => Ok(value),
        AquiferConfigurationValue::UpperEvaporationPattern(_) => Err(invalid_value(
            simulation,
            operation,
            "aquifer numeric read returned a pattern",
        )),
    }
}

fn snowmelt_number(
    simulation: &Simulation,
    index: usize,
    field: SnowmeltConfigurationField,
    operation: &str,
) -> Result<f64, JsValue> {
    match simulation
        .inner
        .snowmelt_configuration_value_read(index, field)
        .map_err(|error| simulation.error(operation, error))?
    {
        SnowmeltConfigurationValue::Number(value) => Ok(value),
        SnowmeltConfigurationValue::RemovalFractions(_)
        | SnowmeltConfigurationValue::RemovalSubcatchment(_) => Err(invalid_value(
            simulation,
            operation,
            "snowmelt numeric read returned a non-numeric value",
        )),
    }
}

fn snowmelt_surface_configuration_record(
    simulation: &Simulation,
    index: usize,
    surface: SnowmeltSurface,
    operation: &str,
) -> Result<SnowmeltSurfaceConfigurationRecord, JsValue> {
    let values = snowmelt_surface_configuration(simulation, index, surface, operation)?;
    Ok(SnowmeltSurfaceConfigurationRecord {
        minimum_melt_coefficient: values.minimum_melt_coefficient,
        maximum_melt_coefficient: values.maximum_melt_coefficient,
        base_temperature: values.base_temperature,
        free_water_fraction: values.free_water_fraction,
        initial_snow_depth: values.initial_snow_depth,
        initial_free_water: values.initial_free_water,
        snow_depth_for_full_coverage: values.snow_depth_for_full_coverage,
    })
}

fn snowmelt_surface_configuration(
    simulation: &Simulation,
    index: usize,
    surface: SnowmeltSurface,
    operation: &str,
) -> Result<SnowmeltSurfaceConfiguration, JsValue> {
    let value = |field| {
        simulation
            .inner
            .snowmelt_surface_configuration_value_read(index, surface, field)
            .map_err(|error| simulation.error(operation, error))
    };
    Ok(SnowmeltSurfaceConfiguration {
        minimum_melt_coefficient: value(SnowmeltSurfaceField::MinimumMeltCoefficient)?,
        maximum_melt_coefficient: value(SnowmeltSurfaceField::MaximumMeltCoefficient)?,
        base_temperature: value(SnowmeltSurfaceField::BaseTemperature)?,
        free_water_fraction: value(SnowmeltSurfaceField::FreeWaterFraction)?,
        initial_snow_depth: value(SnowmeltSurfaceField::InitialSnowDepth)?,
        initial_free_water: value(SnowmeltSurfaceField::InitialFreeWater)?,
        snow_depth_for_full_coverage: value(SnowmeltSurfaceField::SnowDepthForFullCoverage)?,
    })
}

fn parse_surface(
    simulation: &Simulation,
    value: &str,
    operation: &str,
) -> Result<SnowmeltSurface, JsValue> {
    match value {
        "plowable" => Ok(SnowmeltSurface::Plowable),
        "impervious" => Ok(SnowmeltSurface::Impervious),
        "pervious" => Ok(SnowmeltSurface::Pervious),
        _ => Err(invalid_value(
            simulation,
            operation,
            "surface must be one of: plowable, impervious, pervious",
        )),
    }
}

fn invalid_value(simulation: &Simulation, operation: &str, detail: &str) -> JsValue {
    simulation.error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}
