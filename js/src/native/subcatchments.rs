//! Subcatchment and rain-gage adapters.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::simulation::quality::PersistentPollutantValuesPatch;
use swmmrs::simulation::subcatchments::{
    RainGagePrecipitationRead, SubcatchmentCoveragePatch, SubcatchmentGroundwaterConfiguration,
    SubcatchmentGroundwaterConfigurationRead, SubcatchmentInfiltrationConfiguration,
    SubcatchmentLoadingPatch, SubcatchmentNamedSettings, SubcatchmentOutletRead, SubcatchmentPatch,
    SubcatchmentPrecipitationScaleFactors, SubcatchmentSnowpackConfiguration,
};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, decode_value, identity_id, serialize};
use super::records::{RainGage, SubcatchmentResults, SubcatchmentSnapshot};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentOutletRecord {
    kind: &'static str,
    id: String,
}

// Presence distinguishes an omitted field from explicit null. The latter is
// rejected for required fields instead of silently becoming an accidental no-op.
enum Presence<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for Presence<T> {
    fn default() -> Self {
        Self::Missing
    }
}

fn deserialize_presence<'de, D, T>(deserializer: D) -> Result<Presence<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(|value| match value {
        Some(value) => Presence::Value(value),
        None => Presence::Null,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OutletPatchDto {
    kind: String,
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InfiltrationPatchDto {
    #[serde(default, deserialize_with = "deserialize_presence")]
    kind: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    initial_rate: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    minimum_rate: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    decay_coefficient: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    drying_time_days: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    maximum_infiltration: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    suction_head: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    hydraulic_conductivity: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    initial_moisture_deficit: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    curve_number: Presence<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GroundwaterPatchDto {
    #[serde(default, deserialize_with = "deserialize_presence")]
    aquifer: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    node: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    surface_elevation: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    groundwater_coefficient: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    groundwater_exponent: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    surface_coefficient: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    surface_exponent: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    interaction_coefficient: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    fixed_surface_depth: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    bottom_elevation: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    water_table_elevation: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    upper_moisture: Presence<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SubcatchmentPatchDto {
    #[serde(default, deserialize_with = "deserialize_presence")]
    tag: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    area: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    impervious_fraction: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    zero_impervious_fraction: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    width: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    slope: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    impervious_roughness: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    pervious_roughness: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    impervious_depression_storage: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    pervious_depression_storage: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    curb_length: Presence<f64>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    included_in_report: Presence<bool>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    rain_gage: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    outlet: Presence<OutletPatchDto>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    infiltration: Presence<InfiltrationPatchDto>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    groundwater: Presence<GroundwaterPatchDto>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    snowpack: Presence<String>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    initial_buildup: Presence<BTreeMap<String, f64>>,
    #[serde(default, deserialize_with = "deserialize_presence")]
    coverage_fractions: Presence<BTreeMap<String, f64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfiltrationHortonRecord {
    kind: &'static str,
    initial_rate: f64,
    minimum_rate: f64,
    decay_coefficient: f64,
    drying_time_days: f64,
    maximum_infiltration: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfiltrationGreenAmptRecord {
    kind: &'static str,
    suction_head: f64,
    hydraulic_conductivity: f64,
    initial_moisture_deficit: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InfiltrationCurveNumberRecord {
    kind: &'static str,
    curve_number: f64,
    drying_time_days: f64,
}

#[derive(Serialize)]
#[serde(untagged)]
enum InfiltrationRecord {
    Horton(InfiltrationHortonRecord),
    GreenAmpt(InfiltrationGreenAmptRecord),
    CurveNumber(InfiltrationCurveNumberRecord),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroundwaterRecord {
    aquifer: String,
    node: String,
    surface_elevation: f64,
    groundwater_coefficient: f64,
    groundwater_exponent: f64,
    surface_coefficient: f64,
    surface_exponent: f64,
    interaction_coefficient: f64,
    fixed_surface_depth: f64,
    bottom_elevation: f64,
    water_table_elevation: f64,
    upper_moisture: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentConfigurationRecord {
    tag: String,
    rain_scale_factor: f64,
    snow_scale_factor: f64,
    area: f64,
    rain_gage: String,
    included_in_report: bool,
    width: f64,
    slope: f64,
    curb_length: f64,
    impervious_fraction: f64,
    zero_impervious_fraction: f64,
    impervious_roughness: f64,
    pervious_roughness: f64,
    impervious_depression_storage: f64,
    pervious_depression_storage: f64,
    outlet: SubcatchmentOutletRecord,
    infiltration: Option<InfiltrationRecord>,
    groundwater: Option<GroundwaterRecord>,
    snowpack: Option<String>,
    initial_buildup: BTreeMap<String, f64>,
    coverage_fractions: BTreeMap<String, f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentQualityRecord {
    pollutant_ids: Vec<String>,
    runoff_concentrations: Vec<f64>,
    ponded_concentrations: Vec<f64>,
    buildup_loads: Vec<f64>,
    total_washoff_loads: Vec<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentQualitySnapshotRecord {
    object_ids: Vec<String>,
    pollutant_ids: Vec<String>,
    runoff_concentrations: Vec<Vec<f64>>,
    ponded_concentrations: Vec<Vec<f64>>,
    buildup_loads: Vec<Vec<f64>>,
    total_washoff_loads: Vec<Vec<f64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentStatisticsRecord {
    precipitation: f64,
    runon_volume: f64,
    evaporation_volume: f64,
    infiltration_volume: f64,
    runoff_volume: f64,
    maximum_runoff: f64,
    impervious_runoff_volume: f64,
    pervious_runoff_volume: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubcatchmentStatisticsSnapshotRecord {
    object_ids: Vec<String>,
    precipitation: Vec<f64>,
    runon_volume: Vec<f64>,
    evaporation_volume: Vec<f64>,
    infiltration_volume: Vec<f64>,
    runoff_volume: Vec<f64>,
    maximum_runoff: Vec<f64>,
    impervious_runoff_volume: Vec<f64>,
    pervious_runoff_volume: Vec<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScaleFactorsRecord {
    rainfall: f64,
    snowfall: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalForcingRecord {
    rainfall: f64,
    snowfall: f64,
}

fn invalid(detail: impl Into<String>) -> SwmmError {
    SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail)
}

fn supplied<T>(value: Presence<T>, name: &str) -> Result<Option<T>, SwmmError> {
    match value {
        Presence::Missing => Ok(None),
        Presence::Value(value) => Ok(Some(value)),
        Presence::Null => Err(invalid(format!("{name} must not be null"))),
    }
}

fn present<T>(value: &Presence<T>) -> bool {
    !matches!(value, Presence::Missing)
}

fn object_index(
    simulation: &swmmrs::simulation::SwmmSimulation,
    object_type: ObjectType,
    id: &str,
    name: &str,
) -> Result<usize, SwmmError> {
    simulation
        .object_identity_read(object_type, id)?
        .map(|identity| identity.index)
        .ok_or_else(|| {
            SwmmError::with_detail(ErrorCode::ApiObjectName, format!("unknown {name}: {id}"))
        })
}

fn infiltration_kind(value: &SubcatchmentInfiltrationConfiguration) -> &'static str {
    match value {
        SubcatchmentInfiltrationConfiguration::Horton { .. } => "horton",
        SubcatchmentInfiltrationConfiguration::ModifiedHorton { .. } => "modified_horton",
        SubcatchmentInfiltrationConfiguration::GreenAmpt { .. } => "green_ampt",
        SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt { .. } => "modified_green_ampt",
        SubcatchmentInfiltrationConfiguration::CurveNumber { .. } => "curve_number",
    }
}

fn required_number(
    current: Option<f64>,
    value: Presence<f64>,
    name: &str,
) -> Result<f64, SwmmError> {
    match value {
        Presence::Value(value) => Ok(value),
        Presence::Null => Err(invalid(format!("{name} must not be null"))),
        Presence::Missing => {
            current.ok_or_else(|| invalid(format!("missing infiltration field: {name}")))
        }
    }
}

fn optional_number(
    current: Option<f64>,
    value: Presence<f64>,
    _name: &str,
) -> Result<Option<f64>, SwmmError> {
    match value {
        Presence::Value(value) => Ok(Some(value)),
        Presence::Null => Ok(None),
        Presence::Missing => Ok(current),
    }
}

fn reject_fields(fields: &[(&str, bool)]) -> Result<(), SwmmError> {
    for (name, present) in fields {
        if *present {
            return Err(invalid(format!(
                "{name} does not apply to the selected infiltration kind"
            )));
        }
    }
    Ok(())
}

fn infiltration_from_patch(
    simulation: &swmmrs::simulation::SwmmSimulation,
    index: usize,
    dto: InfiltrationPatchDto,
) -> Result<SubcatchmentInfiltrationConfiguration, SwmmError> {
    let current = simulation.subcatchment_infiltration_configuration_read(index)?;
    let current_kind = current.as_ref().map(infiltration_kind);
    let kind = match dto.kind {
        Presence::Missing => current_kind
            .map(str::to_owned)
            .ok_or_else(|| invalid("infiltration kind is required"))?,
        Presence::Value(kind) => match kind.as_str() {
            "horton"
            | "modified_horton"
            | "green_ampt"
            | "modified_green_ampt"
            | "curve_number" => kind,
            _ => return Err(invalid("invalid infiltration model")),
        },
        Presence::Null => return Err(invalid("infiltration kind must not be null")),
    };
    let switching = current_kind != Some(kind.as_str());
    let current_horton = if !switching {
        match current.as_ref() {
            Some(
                SubcatchmentInfiltrationConfiguration::Horton {
                    initial_rate,
                    minimum_rate,
                    decay_coefficient,
                    drying_time_days,
                    maximum_infiltration,
                }
                | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                    initial_rate,
                    minimum_rate,
                    decay_coefficient,
                    drying_time_days,
                    maximum_infiltration,
                },
            ) => Some((
                *initial_rate,
                *minimum_rate,
                *decay_coefficient,
                *drying_time_days,
                *maximum_infiltration,
            )),
            _ => None,
        }
    } else {
        None
    };
    let current_green_ampt = if !switching {
        match current.as_ref() {
            Some(
                SubcatchmentInfiltrationConfiguration::GreenAmpt {
                    suction_head,
                    hydraulic_conductivity,
                    initial_moisture_deficit,
                }
                | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                    suction_head,
                    hydraulic_conductivity,
                    initial_moisture_deficit,
                },
            ) => Some((
                *suction_head,
                *hydraulic_conductivity,
                *initial_moisture_deficit,
            )),
            _ => None,
        }
    } else {
        None
    };
    let current_curve_number = if !switching {
        match current.as_ref() {
            Some(SubcatchmentInfiltrationConfiguration::CurveNumber {
                curve_number,
                drying_time_days,
            }) => Some((*curve_number, *drying_time_days)),
            _ => None,
        }
    } else {
        None
    };
    match kind.as_str() {
        "horton" | "modified_horton" => {
            reject_fields(&[
                ("suction_head", present(&dto.suction_head)),
                (
                    "hydraulic_conductivity",
                    present(&dto.hydraulic_conductivity),
                ),
                (
                    "initial_moisture_deficit",
                    present(&dto.initial_moisture_deficit),
                ),
                ("curve_number", present(&dto.curve_number)),
            ])?;
            let current = current_horton;
            let values = (
                required_number(
                    current.map(|values| values.0),
                    dto.initial_rate,
                    "initial_rate",
                )?,
                required_number(
                    current.map(|values| values.1),
                    dto.minimum_rate,
                    "minimum_rate",
                )?,
                required_number(
                    current.map(|values| values.2),
                    dto.decay_coefficient,
                    "decay_coefficient",
                )?,
                required_number(
                    current.map(|values| values.3),
                    dto.drying_time_days,
                    "drying_time_days",
                )?,
                optional_number(
                    current.and_then(|values| values.4),
                    dto.maximum_infiltration,
                    "maximum_infiltration",
                )?,
            );
            if kind == "horton" {
                Ok(SubcatchmentInfiltrationConfiguration::Horton {
                    initial_rate: values.0,
                    minimum_rate: values.1,
                    decay_coefficient: values.2,
                    drying_time_days: values.3,
                    maximum_infiltration: values.4,
                })
            } else {
                Ok(SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                    initial_rate: values.0,
                    minimum_rate: values.1,
                    decay_coefficient: values.2,
                    drying_time_days: values.3,
                    maximum_infiltration: values.4,
                })
            }
        }
        "green_ampt" | "modified_green_ampt" => {
            reject_fields(&[
                ("initial_rate", present(&dto.initial_rate)),
                ("minimum_rate", present(&dto.minimum_rate)),
                ("decay_coefficient", present(&dto.decay_coefficient)),
                ("drying_time_days", present(&dto.drying_time_days)),
                ("maximum_infiltration", present(&dto.maximum_infiltration)),
                ("curve_number", present(&dto.curve_number)),
            ])?;
            let current = current_green_ampt;
            let values = (
                required_number(
                    current.map(|values| values.0),
                    dto.suction_head,
                    "suction_head",
                )?,
                required_number(
                    current.map(|values| values.1),
                    dto.hydraulic_conductivity,
                    "hydraulic_conductivity",
                )?,
                required_number(
                    current.map(|values| values.2),
                    dto.initial_moisture_deficit,
                    "initial_moisture_deficit",
                )?,
            );
            if kind == "green_ampt" {
                Ok(SubcatchmentInfiltrationConfiguration::GreenAmpt {
                    suction_head: values.0,
                    hydraulic_conductivity: values.1,
                    initial_moisture_deficit: values.2,
                })
            } else {
                Ok(SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                    suction_head: values.0,
                    hydraulic_conductivity: values.1,
                    initial_moisture_deficit: values.2,
                })
            }
        }
        "curve_number" => {
            reject_fields(&[
                ("initial_rate", present(&dto.initial_rate)),
                ("minimum_rate", present(&dto.minimum_rate)),
                ("decay_coefficient", present(&dto.decay_coefficient)),
                ("maximum_infiltration", present(&dto.maximum_infiltration)),
                ("suction_head", present(&dto.suction_head)),
                (
                    "hydraulic_conductivity",
                    present(&dto.hydraulic_conductivity),
                ),
                (
                    "initial_moisture_deficit",
                    present(&dto.initial_moisture_deficit),
                ),
            ])?;
            let current = current_curve_number;
            Ok(SubcatchmentInfiltrationConfiguration::CurveNumber {
                curve_number: required_number(
                    current.map(|values| values.0),
                    dto.curve_number,
                    "curve_number",
                )?,
                drying_time_days: required_number(
                    current.map(|values| values.1),
                    dto.drying_time_days,
                    "drying_time_days",
                )?,
            })
        }
        _ => unreachable!(),
    }
}

fn groundwater_from_read(
    value: SubcatchmentGroundwaterConfigurationRead,
) -> Option<SubcatchmentGroundwaterConfiguration> {
    match value {
        SubcatchmentGroundwaterConfigurationRead::Removed => None,
        SubcatchmentGroundwaterConfigurationRead::Present {
            aquifer_index,
            node_index,
            surface_elevation,
            groundwater_coefficient,
            groundwater_exponent,
            surface_coefficient,
            surface_exponent,
            interaction_coefficient,
            fixed_surface_depth,
            bottom_elevation,
            water_table_elevation,
            upper_moisture,
        } => Some(SubcatchmentGroundwaterConfiguration::Present {
            aquifer_index,
            node_index,
            surface_elevation,
            groundwater_coefficient,
            groundwater_exponent,
            surface_coefficient,
            surface_exponent,
            interaction_coefficient,
            fixed_surface_depth,
            bottom_elevation,
            water_table_elevation,
            upper_moisture,
        }),
    }
}

fn groundwater_required_number(
    current: Option<f64>,
    value: Presence<f64>,
    name: &str,
) -> Result<f64, SwmmError> {
    required_number(current, value, name)
}

fn groundwater_from_patch(
    simulation: &swmmrs::simulation::SwmmSimulation,
    index: usize,
    dto: GroundwaterPatchDto,
) -> Result<SubcatchmentGroundwaterConfiguration, SwmmError> {
    let current =
        groundwater_from_read(simulation.subcatchment_groundwater_configuration_read(index)?);
    let current_values = match current.as_ref() {
        Some(SubcatchmentGroundwaterConfiguration::Present {
            aquifer_index,
            node_index,
            surface_elevation,
            groundwater_coefficient,
            groundwater_exponent,
            surface_coefficient,
            surface_exponent,
            interaction_coefficient,
            fixed_surface_depth,
            bottom_elevation,
            water_table_elevation,
            upper_moisture,
        }) => Some((
            *aquifer_index,
            *node_index,
            *surface_elevation,
            *groundwater_coefficient,
            *groundwater_exponent,
            *surface_coefficient,
            *surface_exponent,
            *interaction_coefficient,
            *fixed_surface_depth,
            *bottom_elevation,
            *water_table_elevation,
            *upper_moisture,
        )),
        None | Some(SubcatchmentGroundwaterConfiguration::Removed) => None,
    };
    let aquifer = match dto.aquifer {
        Presence::Missing => current_values
            .map(|values| values.0)
            .ok_or_else(|| invalid("missing groundwater field: aquifer"))?,
        Presence::Value(id) => object_index(simulation, ObjectType::Aquifer, &id, "aquifer")?,
        Presence::Null => return Err(invalid("aquifer must not be null")),
    };
    let node = match dto.node {
        Presence::Missing => current_values
            .map(|values| values.1)
            .ok_or_else(|| invalid("missing groundwater field: node"))?,
        Presence::Value(id) => object_index(simulation, ObjectType::Node, &id, "node")?,
        Presence::Null => return Err(invalid("node must not be null")),
    };
    Ok(SubcatchmentGroundwaterConfiguration::Present {
        aquifer_index: aquifer,
        node_index: node,
        surface_elevation: groundwater_required_number(
            current_values.map(|values| values.2),
            dto.surface_elevation,
            "surface_elevation",
        )?,
        groundwater_coefficient: groundwater_required_number(
            current_values.map(|values| values.3),
            dto.groundwater_coefficient,
            "groundwater_coefficient",
        )?,
        groundwater_exponent: groundwater_required_number(
            current_values.map(|values| values.4),
            dto.groundwater_exponent,
            "groundwater_exponent",
        )?,
        surface_coefficient: groundwater_required_number(
            current_values.map(|values| values.5),
            dto.surface_coefficient,
            "surface_coefficient",
        )?,
        surface_exponent: groundwater_required_number(
            current_values.map(|values| values.6),
            dto.surface_exponent,
            "surface_exponent",
        )?,
        interaction_coefficient: groundwater_required_number(
            current_values.map(|values| values.7),
            dto.interaction_coefficient,
            "interaction_coefficient",
        )?,
        fixed_surface_depth: groundwater_required_number(
            current_values.map(|values| values.8),
            dto.fixed_surface_depth,
            "fixed_surface_depth",
        )?,
        bottom_elevation: groundwater_required_number(
            current_values.map(|values| values.9),
            dto.bottom_elevation,
            "bottom_elevation",
        )?,
        water_table_elevation: groundwater_required_number(
            current_values.map(|values| values.10),
            dto.water_table_elevation,
            "water_table_elevation",
        )?,
        upper_moisture: groundwater_required_number(
            current_values.map(|values| values.11),
            dto.upper_moisture,
            "upper_moisture",
        )?,
    })
}

fn named_selector(kind: &str) -> Result<(ObjectType, SubcatchmentNamedSettings), SwmmError> {
    match kind {
        "loading" | "initialBuildup" => {
            Ok((ObjectType::Pollut, SubcatchmentNamedSettings::Loading))
        }
        "coverage" | "coverageFractions" => {
            Ok((ObjectType::Landuse, SubcatchmentNamedSettings::Coverage))
        }
        _ => Err(invalid("named settings kind must be loading or coverage")),
    }
}

fn resolve_named_values(
    simulation: &swmmrs::simulation::SwmmSimulation,
    index: usize,
    kind: &str,
    values: BTreeMap<String, f64>,
    replace: bool,
) -> Result<Vec<(usize, f64)>, SwmmError> {
    let (object_type, selector) = named_selector(kind)?;
    let identities = simulation.object_identities_read(object_type)?;
    let current = simulation.subcatchment_named_settings_read(index, selector)?;
    if current.len() != identities.len() {
        return Err(SwmmError::with_detail(
            ErrorCode::System,
            "named settings identity/value storage mismatch",
        ));
    }
    let supplied_any = !values.is_empty();
    let mut resolved = HashMap::with_capacity(values.len());
    for (id, value) in values {
        let identity = identities
            .iter()
            .find(|identity| identity.id.eq_ignore_ascii_case(&id))
            .ok_or_else(|| SwmmError::with_detail(ErrorCode::ApiObjectName, id.clone()))?;
        if resolved.insert(identity.index, value).is_some() {
            return Err(invalid("duplicate configured settings name"));
        }
    }
    let mut result = Vec::with_capacity(identities.len());
    for (position, identity) in identities.iter().enumerate() {
        if let Some(value) = resolved.remove(&identity.index) {
            result.push((identity.index, value));
        } else if replace {
            result.push((identity.index, 0.0));
        } else if supplied_any {
            result.push((identity.index, current[position]));
        }
    }
    Ok(result)
}

fn mapping_record(
    simulation: &swmmrs::simulation::SwmmSimulation,
    index: usize,
    kind: &str,
) -> Result<BTreeMap<String, f64>, SwmmError> {
    let (object_type, selector) = named_selector(kind)?;
    let identities = simulation.object_identities_read(object_type)?;
    let values = simulation.subcatchment_named_settings_read(index, selector)?;
    if values.len() != identities.len() {
        return Err(SwmmError::with_detail(
            ErrorCode::System,
            "named settings identity/value storage mismatch",
        ));
    }
    Ok(identities
        .into_iter()
        .zip(values)
        .map(|(identity, value)| (identity.id, value))
        .collect())
}

fn infiltration_record(value: SubcatchmentInfiltrationConfiguration) -> InfiltrationRecord {
    match value {
        SubcatchmentInfiltrationConfiguration::Horton {
            initial_rate,
            minimum_rate,
            decay_coefficient,
            drying_time_days,
            maximum_infiltration,
        } => InfiltrationRecord::Horton(InfiltrationHortonRecord {
            kind: "horton",
            initial_rate,
            minimum_rate,
            decay_coefficient,
            drying_time_days,
            maximum_infiltration,
        }),
        SubcatchmentInfiltrationConfiguration::ModifiedHorton {
            initial_rate,
            minimum_rate,
            decay_coefficient,
            drying_time_days,
            maximum_infiltration,
        } => InfiltrationRecord::Horton(InfiltrationHortonRecord {
            kind: "modified_horton",
            initial_rate,
            minimum_rate,
            decay_coefficient,
            drying_time_days,
            maximum_infiltration,
        }),
        SubcatchmentInfiltrationConfiguration::GreenAmpt {
            suction_head,
            hydraulic_conductivity,
            initial_moisture_deficit,
        } => InfiltrationRecord::GreenAmpt(InfiltrationGreenAmptRecord {
            kind: "green_ampt",
            suction_head,
            hydraulic_conductivity,
            initial_moisture_deficit,
        }),
        SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
            suction_head,
            hydraulic_conductivity,
            initial_moisture_deficit,
        } => InfiltrationRecord::GreenAmpt(InfiltrationGreenAmptRecord {
            kind: "modified_green_ampt",
            suction_head,
            hydraulic_conductivity,
            initial_moisture_deficit,
        }),
        SubcatchmentInfiltrationConfiguration::CurveNumber {
            curve_number,
            drying_time_days,
        } => InfiltrationRecord::CurveNumber(InfiltrationCurveNumberRecord {
            kind: "curve_number",
            curve_number,
            drying_time_days,
        }),
    }
}
fn into_patch(
    simulation: &swmmrs::simulation::SwmmSimulation,
    index: usize,
    dto: SubcatchmentPatchDto,
) -> Result<SubcatchmentPatch, SwmmError> {
    let mut patch = SubcatchmentPatch {
        tag: supplied(dto.tag, "tag")?,
        area: supplied(dto.area, "area")?,
        impervious_fraction: supplied(dto.impervious_fraction, "impervious_fraction")?,
        zero_impervious_fraction: supplied(
            dto.zero_impervious_fraction,
            "zero_impervious_fraction",
        )?,
        width: supplied(dto.width, "width")?,
        slope: supplied(dto.slope, "slope")?,
        impervious_roughness: supplied(dto.impervious_roughness, "impervious_roughness")?,
        pervious_roughness: supplied(dto.pervious_roughness, "pervious_roughness")?,
        impervious_depression_storage: supplied(
            dto.impervious_depression_storage,
            "impervious_depression_storage",
        )?,
        pervious_depression_storage: supplied(
            dto.pervious_depression_storage,
            "pervious_depression_storage",
        )?,
        curb_length: supplied(dto.curb_length, "curb_length")?,
        included_in_report: supplied(dto.included_in_report, "included_in_report")?,
        ..SubcatchmentPatch::default()
    };
    match dto.rain_gage {
        Presence::Missing => {}
        Presence::Null => return Err(invalid("rain_gage must not be null")),
        Presence::Value(value) => {
            patch.rain_gage_index = Some(object_index(
                simulation,
                ObjectType::Gage,
                &value,
                "rain gage",
            )?);
        }
    }
    match dto.outlet {
        Presence::Missing => {}
        Presence::Null => return Err(invalid("outlet must not be null")),
        Presence::Value(value) => {
            patch.outlet = Some(match value.kind.as_str() {
                "node" => SubcatchmentOutletRead::Node(object_index(
                    simulation,
                    ObjectType::Node,
                    &value.id,
                    "node outlet",
                )?),
                "subcatchment" => SubcatchmentOutletRead::Subcatchment(object_index(
                    simulation,
                    ObjectType::Subcatch,
                    &value.id,
                    "subcatchment outlet",
                )?),
                _ => return Err(invalid("outlet kind must be node or subcatchment")),
            });
        }
    }
    match dto.infiltration {
        Presence::Missing => {}
        Presence::Null => return Err(invalid("infiltration must not be null")),
        Presence::Value(value) => {
            patch.infiltration = Some(infiltration_from_patch(simulation, index, value)?);
        }
    }
    match dto.groundwater {
        Presence::Missing => {}
        Presence::Null => {
            patch.groundwater = Some(SubcatchmentGroundwaterConfiguration::Removed);
        }
        Presence::Value(value) => {
            patch.groundwater = Some(groundwater_from_patch(simulation, index, value)?);
        }
    }
    match dto.snowpack {
        Presence::Missing => {}
        Presence::Null => {
            patch.snowpack = Some(SubcatchmentSnowpackConfiguration::Removed);
        }
        Presence::Value(value) => {
            patch.snowpack = Some(SubcatchmentSnowpackConfiguration::Present {
                snowmelt_index: object_index(simulation, ObjectType::Snowmelt, &value, "snowpack")?,
            });
        }
    }
    match dto.initial_buildup {
        Presence::Missing => {}
        Presence::Null => return Err(invalid("initial_buildup must not be null")),
        Presence::Value(value) => {
            patch.loading = Some(SubcatchmentLoadingPatch {
                initial_buildup: resolve_named_values(simulation, index, "loading", value, true)?,
            });
        }
    }
    match dto.coverage_fractions {
        Presence::Missing => {}
        Presence::Null => return Err(invalid("coverage_fractions must not be null")),
        Presence::Value(value) => {
            patch.coverage = Some(SubcatchmentCoveragePatch {
                fractions: resolve_named_values(simulation, index, "coverage", value, true)?,
            });
        }
    }
    Ok(patch)
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = subcatchment)]
    pub fn subcatchment(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_results_read(index)
            .map_err(|error| self.error("subcatchment", error))?;
        serialize(&SubcatchmentResults {
            rainfall: values.rainfall,
            evaporation: values.evaporation,
            infiltration: values.infiltration,
            runon: values.runon,
            runoff: values.runoff,
            snow_depth: values.snow_depth,
        })
    }

    #[wasm_bindgen(js_name = subcatchments)]
    pub fn subcatchments(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .subcatchment_snapshot_read(ids.as_deref())
            .map_err(|error| self.error("subcatchments", error))?;
        serialize(&SubcatchmentSnapshot {
            object_ids: values.object_ids,
            rainfall: values.rainfall,
            evaporation: values.evaporation,
            infiltration: values.infiltration,
            runon: values.runon,
            runoff: values.runoff,
            snow_depth: values.snow_depth,
        })
    }

    #[wasm_bindgen(js_name = subcatchmentConfiguration)]
    pub fn subcatchment_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentConfiguration";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_configuration_read(index)
            .map_err(|error| self.error(operation, error))?;
        let rain_gage = identity_id(self, ObjectType::Gage, values.rain_gage_index, operation)?;
        let outlet = match values.outlet {
            SubcatchmentOutletRead::Node(index) => SubcatchmentOutletRecord {
                kind: "node",
                id: identity_id(self, ObjectType::Node, index, operation)?,
            },
            SubcatchmentOutletRead::Subcatchment(index) => SubcatchmentOutletRecord {
                kind: "subcatchment",
                id: identity_id(self, ObjectType::Subcatch, index, operation)?,
            },
        };
        let groundwater = match values.groundwater {
            SubcatchmentGroundwaterConfigurationRead::Removed => None,
            SubcatchmentGroundwaterConfigurationRead::Present {
                aquifer_index,
                node_index,
                surface_elevation,
                groundwater_coefficient,
                groundwater_exponent,
                surface_coefficient,
                surface_exponent,
                interaction_coefficient,
                fixed_surface_depth,
                bottom_elevation,
                water_table_elevation,
                upper_moisture,
            } => Some(GroundwaterRecord {
                aquifer: identity_id(self, ObjectType::Aquifer, aquifer_index, operation)?,
                node: identity_id(self, ObjectType::Node, node_index, operation)?,
                surface_elevation,
                groundwater_coefficient,
                groundwater_exponent,
                surface_coefficient,
                surface_exponent,
                interaction_coefficient,
                fixed_surface_depth,
                bottom_elevation,
                water_table_elevation,
                upper_moisture,
            }),
        };
        let snowpack = self
            .inner
            .subcatchment_snowpack_component_read(index)
            .map_err(|error| self.error(operation, error))?
            .map(|value| identity_id(self, ObjectType::Snowmelt, value.snowmelt_index, operation))
            .transpose()?;
        let initial_buildup = mapping_record(&self.inner, index, "loading")
            .map_err(|error| self.error(operation, error))?;
        let coverage_fractions = mapping_record(&self.inner, index, "coverage")
            .map_err(|error| self.error(operation, error))?;
        let infiltration = values.infiltration.map(infiltration_record);
        serialize(&SubcatchmentConfigurationRecord {
            tag: values.tag,
            rain_scale_factor: values.rain_scale_factor,
            snow_scale_factor: values.snow_scale_factor,
            area: values.area,
            rain_gage,
            included_in_report: values.included_in_report,
            width: values.width,
            slope: values.slope,
            curb_length: values.curb_length,
            impervious_fraction: values.impervious_fraction,
            zero_impervious_fraction: values.zero_impervious_fraction,
            impervious_roughness: values.impervious_roughness,
            pervious_roughness: values.pervious_roughness,
            impervious_depression_storage: values.impervious_depression_storage,
            pervious_depression_storage: values.pervious_depression_storage,
            outlet,
            infiltration,
            groundwater,
            snowpack,
            initial_buildup,
            coverage_fractions,
        })
    }

    #[wasm_bindgen(js_name = configureSubcatchment)]
    pub fn configure_subcatchment(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let operation = "configureSubcatchment";
        let patch: SubcatchmentPatchDto = decode(patch, operation)?;
        let index = self.index(ObjectType::Subcatch, id)?;
        let patch =
            into_patch(&self.inner, index, patch).map_err(|error| self.error(operation, error))?;
        self.inner
            .patch_subcatchment(index, patch)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = rainGage)]
    pub fn rain_gage(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Gage, id)?;
        let values: RainGagePrecipitationRead = self
            .inner
            .rain_gage_precipitation_read(index)
            .map_err(|error| self.error("rainGage", error))?;
        let external_precipitation_rate = self
            .inner
            .rain_gage_external_precipitation_rate_read(index)
            .map_err(|error| self.error("rainGage", error))?;
        let rainfall_override = self
            .inner
            .rain_gage_rainfall_override_read(index)
            .map_err(|error| self.error("rainGage", error))?;
        serialize(&RainGage {
            total_precip: values.total_precip,
            rainfall: values.rainfall,
            snowfall: values.snowfall,
            external_precipitation_rate,
            rainfall_override,
        })
    }

    #[wasm_bindgen(js_name = useRainGageExternalPrecipitation)]
    pub fn use_rain_gage_external_precipitation(
        &mut self,
        id: &str,
        value: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "useRainGageExternalPrecipitation";
        let index = self.index(ObjectType::Gage, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_rain_gage_external_precipitation_rate(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = rainGageExternalPrecipitationRate)]
    pub fn rain_gage_external_precipitation_rate(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Gage, id)?;
        let value = self
            .inner
            .rain_gage_external_precipitation_rate_read(index)
            .map_err(|error| self.error("rainGageExternalPrecipitationRate", error))?;
        serialize(&value)
    }

    #[wasm_bindgen(js_name = rainGageRainfallOverride)]
    pub fn rain_gage_rainfall_override(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Gage, id)?;
        let value = self
            .inner
            .rain_gage_rainfall_override_read(index)
            .map_err(|error| self.error("rainGageRainfallOverride", error))?;
        serialize(&value)
    }

    #[wasm_bindgen(js_name = setRainGagePrecipitation)]
    pub fn set_rain_gage_precipitation(&mut self, id: &str, value: JsValue) -> Result<(), JsValue> {
        let operation = "setRainGagePrecipitation";
        let index = self.index(ObjectType::Gage, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_rain_gage_external_precipitation_rate(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = setRainGageRainfallOverride)]
    pub fn set_rain_gage_rainfall_override(
        &mut self,
        id: &str,
        value: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "setRainGageRainfallOverride";
        let index = self.index(ObjectType::Gage, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_rain_gage_rainfall_override(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = subcatchmentPrecipitationScaleFactors)]
    pub fn subcatchment_precipitation_scale_factors(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentPrecipitationScaleFactors";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_configuration_read(index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&ScaleFactorsRecord {
            rainfall: values.rain_scale_factor,
            snowfall: values.snow_scale_factor,
        })
    }

    #[wasm_bindgen(js_name = setSubcatchmentPrecipitationScaleFactors)]
    pub fn set_subcatchment_precipitation_scale_factors(
        &mut self,
        id: &str,
        rainfall: JsValue,
        snowfall: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "setSubcatchmentPrecipitationScaleFactors";
        let index = self.index(ObjectType::Subcatch, id)?;
        let rainfall = decode_value(rainfall, operation)?;
        let snowfall = decode_value(snowfall, operation)?;
        self.inner
            .set_subcatchment_precipitation_scale_factors(
                index,
                SubcatchmentPrecipitationScaleFactors { rainfall, snowfall },
            )
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = subcatchmentExternalForcing)]
    pub fn subcatchment_external_forcing(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentExternalForcing";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_external_forcing_read(index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&ExternalForcingRecord {
            rainfall: values.rainfall,
            snowfall: values.snowfall,
        })
    }

    #[wasm_bindgen(js_name = setSubcatchmentExternalRainfall)]
    pub fn set_subcatchment_external_rainfall(
        &mut self,
        id: &str,
        value: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "setSubcatchmentExternalRainfall";
        let index = self.index(ObjectType::Subcatch, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_subcatchment_external_rainfall(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = setSubcatchmentExternalSnowfall)]
    pub fn set_subcatchment_external_snowfall(
        &mut self,
        id: &str,
        value: JsValue,
    ) -> Result<(), JsValue> {
        let operation = "setSubcatchmentExternalSnowfall";
        let index = self.index(ObjectType::Subcatch, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_subcatchment_external_snowfall(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = subcatchmentQuality)]
    pub fn subcatchment_quality(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentQuality";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_quality_read(index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&SubcatchmentQualityRecord {
            pollutant_ids: values.pollutant_ids,
            runoff_concentrations: values.runoff_concentrations,
            ponded_concentrations: values.ponded_concentrations,
            buildup_loads: values.buildup_loads,
            total_washoff_loads: values.total_washoff_loads,
        })
    }

    #[wasm_bindgen(js_name = subcatchmentQualitySnapshot)]
    pub fn subcatchment_quality_snapshot(
        &self,
        ids: Option<Vec<String>>,
    ) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentQualitySnapshot";
        let values = self
            .inner
            .subcatchment_quality_snapshot_read(ids.as_deref())
            .map_err(|error| self.error(operation, error))?;
        serialize(&SubcatchmentQualitySnapshotRecord {
            object_ids: values.object_ids,
            pollutant_ids: values.pollutant_ids,
            runoff_concentrations: values.runoff_concentrations,
            ponded_concentrations: values.ponded_concentrations,
            buildup_loads: values.buildup_loads,
            total_washoff_loads: values.total_washoff_loads,
        })
    }

    #[wasm_bindgen(js_name = subcatchmentNamedSettings)]
    pub fn subcatchment_named_settings(&self, id: &str, kind: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentNamedSettings";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = mapping_record(&self.inner, index, kind)
            .map_err(|error| self.error(operation, error))?;
        serialize(&values)
    }

    #[wasm_bindgen(js_name = subcatchmentNamedSetting)]
    pub fn subcatchment_named_setting(
        &self,
        id: &str,
        kind: &str,
        name: &str,
    ) -> Result<f64, JsValue> {
        let operation = "subcatchmentNamedSetting";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = mapping_record(&self.inner, index, kind)
            .map_err(|error| self.error(operation, error))?;
        values
            .into_iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
            .ok_or_else(|| {
                self.error(
                    operation,
                    SwmmError::with_detail(ErrorCode::ApiObjectName, name),
                )
            })
    }

    #[wasm_bindgen(js_name = updateSubcatchmentNamedSettings)]
    pub fn update_subcatchment_named_settings(
        &mut self,
        id: &str,
        kind: &str,
        values: JsValue,
        replace: bool,
    ) -> Result<(), JsValue> {
        let operation = "updateSubcatchmentNamedSettings";
        let values: BTreeMap<String, f64> = decode(values, operation)?;
        let index = self.index(ObjectType::Subcatch, id)?;
        let resolved = resolve_named_values(&self.inner, index, kind, values, replace)
            .map_err(|error| self.error(operation, error))?;
        let patch = match named_selector(kind)
            .map_err(|error| self.error(operation, error))?
            .1
        {
            SubcatchmentNamedSettings::Loading => SubcatchmentPatch {
                loading: Some(SubcatchmentLoadingPatch {
                    initial_buildup: resolved,
                }),
                ..SubcatchmentPatch::default()
            },
            SubcatchmentNamedSettings::Coverage => SubcatchmentPatch {
                coverage: Some(SubcatchmentCoveragePatch {
                    fractions: resolved,
                }),
                ..SubcatchmentPatch::default()
            },
        };
        self.inner
            .patch_subcatchment(index, patch)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = subcatchmentExternalPollutantBuildupIncrement)]
    pub fn subcatchment_external_pollutant_buildup_increment(
        &self,
        id: &str,
    ) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentExternalPollutantBuildupIncrement";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_external_pollutant_buildup_increment_read(index)
            .map_err(|error| self.error(operation, error))?;
        let values: BTreeMap<_, _> = values
            .pollutant_ids
            .into_iter()
            .zip(values.values)
            .collect();
        serialize(&values)
    }

    #[wasm_bindgen(js_name = subcatchmentExternalPollutantBuildupIncrementValue)]
    pub fn subcatchment_external_pollutant_buildup_increment_value(
        &self,
        id: &str,
        pollutant_id: &str,
    ) -> Result<f64, JsValue> {
        let operation = "subcatchmentExternalPollutantBuildupIncrementValue";
        let index = self.index(ObjectType::Subcatch, id)?;
        self.inner
            .subcatchment_external_pollutant_buildup_increment_value_read(index, pollutant_id)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = updateSubcatchmentExternalPollutantBuildupIncrement)]
    pub fn update_subcatchment_external_pollutant_buildup_increment(
        &mut self,
        id: &str,
        values: JsValue,
        replace: bool,
    ) -> Result<(), JsValue> {
        let operation = "updateSubcatchmentExternalPollutantBuildupIncrement";
        let values: BTreeMap<String, f64> = decode(values, operation)?;
        let index = self.index(ObjectType::Subcatch, id)?;
        self.inner
            .patch_subcatchment_external_pollutant_buildup_increment(
                index,
                PersistentPollutantValuesPatch {
                    values: values.into_iter().collect(),
                    replace,
                },
            )
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = subcatchmentStatistics)]
    pub fn subcatchment_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentStatistics";
        let index = self.index(ObjectType::Subcatch, id)?;
        let values = self
            .inner
            .subcatchment_statistics_read(index)
            .map_err(|error| self.error(operation, error))?;
        serialize(&SubcatchmentStatisticsRecord {
            precipitation: values.precipitation,
            runon_volume: values.runon_volume,
            evaporation_volume: values.evaporation_volume,
            infiltration_volume: values.infiltration_volume,
            runoff_volume: values.runoff_volume,
            maximum_runoff: values.maximum_runoff,
            impervious_runoff_volume: values.impervious_runoff_volume,
            pervious_runoff_volume: values.pervious_runoff_volume,
        })
    }

    #[wasm_bindgen(js_name = subcatchmentStatisticsSnapshot)]
    pub fn subcatchment_statistics_snapshot(
        &self,
        ids: Option<Vec<String>>,
    ) -> Result<JsValue, JsValue> {
        let operation = "subcatchmentStatisticsSnapshot";
        let values = self
            .inner
            .subcatchment_statistics_snapshot_read(ids.as_deref())
            .map_err(|error| self.error(operation, error))?;
        serialize(&SubcatchmentStatisticsSnapshotRecord {
            object_ids: values.object_ids,
            precipitation: values.precipitation,
            runon_volume: values.runon_volume,
            evaporation_volume: values.evaporation_volume,
            infiltration_volume: values.infiltration_volume,
            runoff_volume: values.runoff_volume,
            maximum_runoff: values.maximum_runoff,
            impervious_runoff_volume: values.impervious_runoff_volume,
            pervious_runoff_volume: values.pervious_runoff_volume,
        })
    }
}
