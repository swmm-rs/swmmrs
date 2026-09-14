//! Simulation options and schedule adapters.

use js_sys::{Error, Reflect};
use serde::{Deserialize, Deserializer, Serialize};
use swmmrs::engine::datetime::{
    DateTime, datetime_dayOfYear, datetime_decodeDate, datetime_decodeTime, datetime_encodeDate,
    datetime_encodeTime,
};
use swmmrs::engine::enums::{
    CustomEllipseModel, InertialDampingType, NormalFlowType, SurchargeMethodType,
};
use swmmrs::engine::error::ErrorCode;
use swmmrs::simulation::options::{SimulationOptionsPatch, SimulationSchedulePatch};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, serialize};

/// Full browser-facing simulation option snapshot.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    routing_step_seconds: f64,
    report_step_seconds: i32,
    detailed_reporting_enabled: bool,
    custom_ellipse_model: &'static str,
    surcharge_method: &'static str,
    allow_ponding: bool,
    inertia_damping: &'static str,
    normal_flow_limit: &'static str,
    skip_steady_state: bool,
    rainfall_enabled: bool,
    rdii_enabled: bool,
    snowmelt_enabled: bool,
    groundwater_enabled: bool,
    routing_enabled: bool,
    quality_enabled: bool,
    rule_step_seconds: i32,
    sweep_start: [i32; 2],
    sweep_end: [i32; 2],
    maximum_trials: i32,
    requested_threads: i32,
    minimum_routing_step_seconds: f64,
    lengthening_step_seconds: f64,
    antecedent_dry_seconds: f64,
    courant_factor: f64,
    minimum_surface_area: f64,
    minimum_conduit_slope: f64,
    head_tolerance: f64,
    system_flow_tolerance: f64,
    lateral_flow_tolerance: f64,
}

/// Sparse options accepted by `configureOptions`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OptionsPatchDto {
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    routing_step_seconds: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    report_step_seconds: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    detailed_reporting_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    custom_ellipse_model: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    surcharge_method: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    allow_ponding: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    inertia_damping: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    normal_flow_limit: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    skip_steady_state: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    rainfall_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    rdii_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    snowmelt_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    groundwater_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    routing_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    quality_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    rule_step_seconds: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    sweep_start: Option<[i32; 2]>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    sweep_end: Option<[i32; 2]>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    maximum_trials: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    requested_threads: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    minimum_routing_step_seconds: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    lengthening_step_seconds: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    antecedent_dry_seconds: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    courant_factor: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    minimum_surface_area: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    minimum_conduit_slope: Option<f64>,
}

/// Sparse timezone-free model schedule accepted by `updateSchedule`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SchedulePatchDto {
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    start_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    report_start: Option<String>,
    #[serde(default, deserialize_with = "deserialize_non_null_option")]
    end_time: Option<String>,
}

/// Rejects explicit null while preserving omitted sparse fields via `serde(default)`.
fn deserialize_non_null_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)?
        .map(Some)
        .ok_or_else(|| serde::de::Error::custom("value must not be null"))
}
fn invalid_patch(operation: &str, detail: impl std::fmt::Display) -> JsValue {
    let value = Error::new(&format!("{operation}: invalid patch: {detail}"));
    let _ = Reflect::set(
        &value,
        &"code".into(),
        &ErrorCode::ApiPropertyValue.as_i32().into(),
    );
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    value.into()
}

fn parse_enum<T>(
    value: String,
    operation: &str,
    field: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, JsValue> {
    let normalized = value.to_ascii_lowercase();
    parse(&normalized).ok_or_else(|| invalid_patch(operation, format!("{field} is not supported")))
}

fn parse_sweep_day(value: [i32; 2], field: &str, operation: &str) -> Result<i32, JsValue> {
    let [month, day] = value;
    let encoded = datetime_encodeDate(2001, month, day);
    let mut year = 0;
    let mut decoded_month = 0;
    let mut decoded_day = 0;
    datetime_decodeDate(encoded, &mut year, &mut decoded_month, &mut decoded_day);
    if year != 2001 || decoded_month != month || decoded_day != day {
        return Err(invalid_patch(
            operation,
            format!("{field} must be a non-leap calendar date"),
        ));
    }
    Ok(datetime_dayOfYear(encoded))
}

impl OptionsPatchDto {
    fn into_patch(self, operation: &str) -> Result<SimulationOptionsPatch, JsValue> {
        Ok(SimulationOptionsPatch {
            routing_step_seconds: self.routing_step_seconds,
            report_step_seconds: self.report_step_seconds,
            detailed_reporting_enabled: self.detailed_reporting_enabled,
            custom_ellipse_model: self
                .custom_ellipse_model
                .map(|value| {
                    parse_enum(
                        value,
                        operation,
                        "customEllipseModel",
                        |value| match value {
                            "epa_legacy" => Some(CustomEllipseModel::EpaLegacy),
                            "true_ellipse" => Some(CustomEllipseModel::TrueEllipse),
                            _ => None,
                        },
                    )
                })
                .transpose()?,
            surcharge_method: self
                .surcharge_method
                .map(|value| {
                    parse_enum(value, operation, "surchargeMethod", |value| match value {
                        "extran" => Some(SurchargeMethodType::Extran),
                        "slot" => Some(SurchargeMethodType::Slot),
                        _ => None,
                    })
                })
                .transpose()?,
            allow_ponding: self.allow_ponding,
            inertia_damping: self
                .inertia_damping
                .map(|value| {
                    parse_enum(value, operation, "inertiaDamping", |value| match value {
                        "none" => Some(InertialDampingType::NoDamping),
                        "partial" => Some(InertialDampingType::PartialDamping),
                        "full" => Some(InertialDampingType::FullDamping),
                        _ => None,
                    })
                })
                .transpose()?,
            normal_flow_limit: self
                .normal_flow_limit
                .map(|value| {
                    parse_enum(value, operation, "normalFlowLimit", |value| match value {
                        "slope" => Some(NormalFlowType::Slope),
                        "froude" => Some(NormalFlowType::Froude),
                        "both" => Some(NormalFlowType::Both),
                        "neither" => Some(NormalFlowType::Neither),
                        _ => None,
                    })
                })
                .transpose()?,
            skip_steady_state: self.skip_steady_state,
            rainfall_enabled: self.rainfall_enabled,
            rdii_enabled: self.rdii_enabled,
            snowmelt_enabled: self.snowmelt_enabled,
            groundwater_enabled: self.groundwater_enabled,
            routing_enabled: self.routing_enabled,
            quality_enabled: self.quality_enabled,
            rule_step_seconds: self.rule_step_seconds,
            sweep_start: self
                .sweep_start
                .map(|value| parse_sweep_day(value, "sweepStart", operation))
                .transpose()?,
            sweep_end: self
                .sweep_end
                .map(|value| parse_sweep_day(value, "sweepEnd", operation))
                .transpose()?,
            maximum_trials: self.maximum_trials,
            requested_threads: self.requested_threads,
            minimum_routing_step_seconds: self.minimum_routing_step_seconds,
            lengthening_step_seconds: self.lengthening_step_seconds,
            antecedent_dry_seconds: self.antecedent_dry_seconds,
            courant_factor: self.courant_factor,
            minimum_surface_area: self.minimum_surface_area,
            minimum_conduit_slope: self.minimum_conduit_slope,
        })
    }
}

fn parse_fixed_digits(value: &[u8], start: usize, end: usize) -> Option<i32> {
    value
        .get(start..end)?
        .iter()
        .try_fold(0_i32, |number, digit| {
            (*digit)
                .is_ascii_digit()
                .then_some(number * 10 + i32::from(*digit - b'0'))
        })
}

fn parse_model_time(value: &str, operation: &str, field: &str) -> Result<DateTime, JsValue> {
    let bytes = value.as_bytes();
    let valid_separators = bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'T')
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':');
    let Some((year, month, day, hour, minute, second)) = valid_separators
        .then(|| {
            Some((
                parse_fixed_digits(bytes, 0, 4)?,
                parse_fixed_digits(bytes, 5, 7)?,
                parse_fixed_digits(bytes, 8, 10)?,
                parse_fixed_digits(bytes, 11, 13)?,
                parse_fixed_digits(bytes, 14, 16)?,
                parse_fixed_digits(bytes, 17, 19)?,
            ))
        })
        .flatten()
    else {
        return Err(invalid_patch(
            operation,
            format!("{field} must be YYYY-MM-DDTHH:mm:ss with no timezone"),
        ));
    };
    if bytes.len() != 19 || !(1..=9999).contains(&year) || hour > 23 || minute > 59 || second > 59 {
        return Err(invalid_patch(
            operation,
            format!("{field} must be a whole-second timezone-free timestamp"),
        ));
    }
    let date = datetime_encodeDate(year, month, day);
    let time = datetime_encodeTime(hour, minute, second);
    let value = DateTime(date.0 + time.0);
    let (mut actual_year, mut actual_month, mut actual_day) = (0, 0, 0);
    let (mut actual_hour, mut actual_minute, mut actual_second) = (0, 0, 0);
    datetime_decodeDate(value, &mut actual_year, &mut actual_month, &mut actual_day);
    datetime_decodeTime(
        value,
        &mut actual_hour,
        &mut actual_minute,
        &mut actual_second,
    );
    if (
        actual_year,
        actual_month,
        actual_day,
        actual_hour,
        actual_minute,
        actual_second,
    ) != (year, month, day, hour, minute, second)
    {
        return Err(invalid_patch(
            operation,
            format!("{field} is not a valid calendar time"),
        ));
    }
    Ok(value)
}

impl SchedulePatchDto {
    fn into_patch(self, operation: &str) -> Result<SimulationSchedulePatch, JsValue> {
        Ok(SimulationSchedulePatch {
            start_time: self
                .start_time
                .map(|value| parse_model_time(&value, operation, "startTime"))
                .transpose()?,
            report_start: self
                .report_start
                .map(|value| parse_model_time(&value, operation, "reportStart"))
                .transpose()?,
            end_time: self
                .end_time
                .map(|value| parse_model_time(&value, operation, "endTime"))
                .transpose()?,
        })
    }
}

fn custom_ellipse_model(value: CustomEllipseModel) -> &'static str {
    match value {
        CustomEllipseModel::EpaLegacy => "epa_legacy",
        CustomEllipseModel::TrueEllipse => "true_ellipse",
    }
}

fn surcharge_method(value: SurchargeMethodType) -> &'static str {
    match value {
        SurchargeMethodType::Extran => "extran",
        SurchargeMethodType::Slot => "slot",
    }
}

fn inertia_damping(value: InertialDampingType) -> &'static str {
    match value {
        InertialDampingType::NoDamping => "none",
        InertialDampingType::PartialDamping => "partial",
        InertialDampingType::FullDamping => "full",
    }
}

fn normal_flow_limit(value: NormalFlowType) -> &'static str {
    match value {
        NormalFlowType::Slope => "slope",
        NormalFlowType::Froude => "froude",
        NormalFlowType::Both => "both",
        NormalFlowType::Neither => "neither",
    }
}

fn sweep_pair(day_of_year: i32) -> [i32; 2] {
    let date = datetime_encodeDate(2001, 1, 1) + DateTime((day_of_year - 1) as f64);
    let (mut year, mut month, mut day) = (0, 0, 0);
    datetime_decodeDate(date, &mut year, &mut month, &mut day);
    [month, day]
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = options)]
    pub fn options(&self) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .simulation_options_read()
            .map_err(|error| self.error("options", error))?;
        serialize(&Options {
            routing_step_seconds: values.routing_step_seconds,
            report_step_seconds: values.report_step_seconds,
            detailed_reporting_enabled: values.detailed_reporting_enabled,
            custom_ellipse_model: custom_ellipse_model(values.custom_ellipse_model),
            surcharge_method: surcharge_method(values.surcharge_method),
            allow_ponding: values.allow_ponding,
            inertia_damping: inertia_damping(values.inertia_damping),
            normal_flow_limit: normal_flow_limit(values.normal_flow_limit),
            skip_steady_state: values.skip_steady_state,
            rainfall_enabled: values.rainfall_enabled,
            rdii_enabled: values.rdii_enabled,
            snowmelt_enabled: values.snowmelt_enabled,
            groundwater_enabled: values.groundwater_enabled,
            routing_enabled: values.routing_enabled,
            quality_enabled: values.quality_enabled,
            rule_step_seconds: values.rule_step_seconds,
            sweep_start: sweep_pair(values.sweep_start),
            sweep_end: sweep_pair(values.sweep_end),
            maximum_trials: values.maximum_trials,
            requested_threads: values.requested_threads,
            minimum_routing_step_seconds: values.minimum_routing_step_seconds,
            lengthening_step_seconds: values.lengthening_step_seconds,
            antecedent_dry_seconds: values.antecedent_dry_seconds,
            courant_factor: values.courant_factor,
            minimum_surface_area: values.minimum_surface_area,
            minimum_conduit_slope: values.minimum_conduit_slope,
            head_tolerance: values.head_tolerance,
            system_flow_tolerance: values.system_flow_tolerance,
            lateral_flow_tolerance: values.lateral_flow_tolerance,
        })
    }

    #[wasm_bindgen(js_name = configureOptions)]
    pub fn configure_options(&mut self, patch: JsValue) -> Result<(), JsValue> {
        let patch: OptionsPatchDto = decode(patch, "configureOptions")?;
        let patch = patch.into_patch("configureOptions")?;
        self.inner
            .patch_simulation_options(patch)
            .map_err(|error| self.error("configureOptions", error))
    }

    #[wasm_bindgen(js_name = updateSchedule)]
    pub fn update_schedule(&mut self, patch: JsValue) -> Result<(), JsValue> {
        let patch: SchedulePatchDto = decode(patch, "updateSchedule")?;
        let patch = patch.into_patch("updateSchedule")?;
        self.inner
            .patch_simulation_schedule(patch)
            .map_err(|error| self.error("updateSchedule", error))
    }

    #[wasm_bindgen(js_name = maximumRoutingStepSeconds)]
    pub fn maximum_routing_step_seconds(&self) -> Result<f64, JsValue> {
        self.inner
            .maximum_routing_step_read()
            .map_err(|error| self.error("maximumRoutingStepSeconds", error))
    }
}
