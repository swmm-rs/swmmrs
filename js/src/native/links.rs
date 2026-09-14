//! Browser-facing link, quality, statistics, and link-local inlet adapters.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use swmmrs::engine::enums::{LinkType, ObjectType, OrificeType, RoadSurface, WeirType, XsectType};
use swmmrs::simulation::links::{
    ConduitPatch, CrossSectionConfiguration, InletPersistentPatch, InletResultsRead,
    LinkConfigurationField, LinkConfigurationValue, LinkCrossSectionValue, LinkPatch,
    LinkQualityRead, LinkQualitySnapshotRead, LinkStatisticsRead, LinkStatisticsSnapshotRead,
    LinkSubtypeConfigurationRead, LinkSubtypePatch, OrificeUpdate, OutletHeadBasis, OutletRating,
    OutletRatingConfigurationRead, OutletUpdate, PumpConfiguration, PumpOperatingConfigurationRead,
    PumpStatisticsRead, PumpUpdate, WeirUpdate,
};
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::{decode, decode_value, identity_id, non_null, serialize};

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(js_name = linkConfiguration)]
    pub fn link_configuration(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .link_configuration_read(index)
            .map_err(|error| self.error("linkConfiguration", error))?;
        let cross_section = if values.kind == LinkType::Conduit {
            match self
                .inner
                .link_configuration_value_read(index, LinkConfigurationField::ConduitCrossSection)
                .map_err(|error| self.error("linkConfiguration", error))?
            {
                LinkConfigurationValue::CrossSection(value) => value.map(cross_section_record),
                _ => None,
            }
        } else {
            None
        };
        let subtype = self.subtype_record(values.subtype, cross_section.clone())?;
        let inlet = self
            .inlet_record(index)
            .map_err(|error| self.error("linkConfiguration", error))?;
        serialize(&LinkConfigurationRecord {
            kind: format!("{:?}", values.kind),
            tag: values.tag,
            inlet_node: values.inlet_node.id,
            outlet_node: values.outlet_node.id,
            flow_direction: values.flow_direction,
            length: values.length,
            slope: values.slope,
            full_depth: values.full_depth,
            full_flow: values.full_flow,
            included_in_report: values.included_in_report,
            inlet_offset: values.inlet_offset,
            outlet_offset: values.outlet_offset,
            initial_flow: values.initial_flow,
            flow_limit: values.flow_limit,
            inlet_loss_coefficient: values.inlet_loss_coefficient,
            outlet_loss_coefficient: values.outlet_loss_coefficient,
            average_loss_coefficient: values.average_loss_coefficient,
            seepage_rate: values.seepage_rate,
            has_flap_gate: values.has_flap_gate,
            subtype,
            cross_section,
            inlet,
        })
    }

    #[wasm_bindgen(js_name = configureLink)]
    pub fn configure_link(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let patch: LinkPatchDto = decode(patch, "configureLink")?;
        let index = self.index(ObjectType::Link, id)?;
        let patch = patch.into_patch(self)?;
        self.inner
            .patch_link(index, patch)
            .map_err(|error| self.error("configureLink", error))
    }

    #[wasm_bindgen(js_name = setLinkFlowLimit)]
    pub fn set_link_flow_limit(&mut self, id: &str, value: JsValue) -> Result<(), JsValue> {
        let operation = "setLinkFlowLimit";
        let index = self.index(ObjectType::Link, id)?;
        let value = decode_value(value, operation)?;
        self.inner
            .set_link_flow_limit(index, value)
            .map_err(|error| self.error(operation, error))
    }

    #[wasm_bindgen(js_name = linkQuality)]
    pub fn link_quality(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .link_quality_read(index)
            .map_err(|error| self.error("linkQuality", error))?;
        serialize(&LinkQualityRecord::from(values))
    }

    #[wasm_bindgen(js_name = linkQualitySnapshot)]
    pub fn link_quality_snapshot(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .link_quality_snapshot_read(ids.as_deref())
            .map_err(|error| self.error("linkQualitySnapshot", error))?;
        serialize(&LinkQualitySnapshotRecord::from(values))
    }

    #[wasm_bindgen(js_name = overrideLinkPollutantConcentrations)]
    pub fn override_link_pollutant_concentrations(
        &mut self,
        id: &str,
        values: JsValue,
    ) -> Result<(), JsValue> {
        let values: BTreeMap<String, f64> = decode(values, "overrideLinkPollutantConcentrations")?;
        let index = self.index(ObjectType::Link, id)?;
        self.inner
            .override_link_pollutant_concentrations(
                index,
                swmmrs::simulation::quality::PollutantValuesPatch {
                    values: values.into_iter().collect(),
                },
            )
            .map_err(|error| self.error("overrideLinkPollutantConcentrations", error))
    }

    #[wasm_bindgen(js_name = linkExternalPollutantMassFlux)]
    pub fn link_external_pollutant_mass_flux(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .link_external_pollutant_mass_flux_read(index)
            .map_err(|error| self.error("linkExternalPollutantMassFlux", error))?;
        serialize(&mapping_record(values.pollutant_ids, values.values))
    }

    #[wasm_bindgen(js_name = linkExternalPollutantMassFluxValue)]
    pub fn link_external_pollutant_mass_flux_value(
        &self,
        id: &str,
        pollutant_id: &str,
    ) -> Result<f64, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        self.inner
            .link_external_pollutant_mass_flux_value_read(index, pollutant_id)
            .map_err(|error| self.error("linkExternalPollutantMassFluxValue", error))
    }

    #[wasm_bindgen(js_name = updateLinkExternalPollutantMassFlux)]
    pub fn update_link_external_pollutant_mass_flux(
        &mut self,
        id: &str,
        values: JsValue,
        replace: bool,
    ) -> Result<(), JsValue> {
        let values: BTreeMap<String, f64> = decode(values, "updateLinkExternalPollutantMassFlux")?;
        let index = self.index(ObjectType::Link, id)?;
        self.inner
            .patch_link_external_pollutant_mass_flux(
                index,
                swmmrs::simulation::quality::PersistentPollutantValuesPatch {
                    values: values.into_iter().collect(),
                    replace,
                },
            )
            .map_err(|error| self.error("updateLinkExternalPollutantMassFlux", error))
    }

    #[wasm_bindgen(js_name = linkStatistics)]
    pub fn link_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .link_statistics_read(index)
            .map_err(|error| self.error("linkStatistics", error))?;
        serialize(&LinkStatisticsRecord::from(values))
    }

    #[wasm_bindgen(js_name = pumpStatistics)]
    pub fn pump_statistics(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .pump_statistics_read(index)
            .map_err(|error| self.error("pumpStatistics", error))?;
        serialize(&PumpStatisticsRecord::from(values))
    }

    #[wasm_bindgen(js_name = linkStatisticsSnapshot)]
    pub fn link_statistics_snapshot(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .link_statistics_snapshot_read(ids.as_deref())
            .map_err(|error| self.error("linkStatisticsSnapshot", error))?;
        serialize(&LinkStatisticsSnapshotRecord::from(values))
    }

    /// Returns the complete optional inlet placement owned by a link.
    #[wasm_bindgen(js_name = linkInlet)]
    pub fn link_inlet(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let inlet = self
            .inlet_record(index)
            .map_err(|error| self.error("linkInlet", error))?;
        serialize(&inlet)
    }

    #[wasm_bindgen(js_name = inletResults)]
    pub fn inlet_results(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let Some(identity) = self
            .inner
            .link_inlet_identity_read(index)
            .map_err(|error| self.error("inletResults", error))?
        else {
            return serialize(&Option::<InletResultsRecord>::None);
        };
        let values = self
            .inner
            .inlet_results_read(index, identity.index, &identity.id)
            .map_err(|error| self.error("inletResults", error))?;
        serialize(&Some(InletResultsRecord::from(values)))
    }

    #[wasm_bindgen(js_name = updateInlet)]
    pub fn update_inlet(&mut self, id: &str, patch: JsValue) -> Result<(), JsValue> {
        let patch: InletPatchDto = decode(patch, "updateInlet")?;
        let index = self.index(ObjectType::Link, id)?;
        let Some(identity) = self
            .inner
            .link_inlet_identity_read(index)
            .map_err(|error| self.error("updateInlet", error))?
        else {
            return Err(self.error(
                "updateInlet",
                swmmrs::engine::error::SwmmError::with_detail(
                    swmmrs::engine::error::ErrorCode::ApiPropertyValue,
                    "link has no configured inlet",
                ),
            ));
        };
        self.inner
            .patch_inlet_persistent(
                index,
                identity.index,
                &identity.id,
                InletPersistentPatch {
                    percent_clogged: patch.percent_clogged,
                    flow_limit: patch.flow_limit,
                },
            )
            .map_err(|error| self.error("updateInlet", error))
    }
}

impl Simulation {
    fn inlet_record(
        &self,
        index: usize,
    ) -> Result<Option<InletConfigurationRecord>, swmmrs::engine::error::SwmmError> {
        let Some(identity) = self.inner.link_inlet_identity_read(index)? else {
            return Ok(None);
        };
        let values = self
            .inner
            .inlet_configuration_read(index, identity.index, &identity.id)?;
        Ok(Some(InletConfigurationRecord {
            count: values.count,
            percent_clogged: values.percent_clogged,
            flow_limit: values.flow_limit,
            local_depression: values.local_depression,
            local_width: values.local_width,
            design: values.design.id,
        }))
    }

    fn subtype_record(
        &self,
        subtype: LinkSubtypeConfigurationRead,
        cross_section: Option<CrossSectionRecord>,
    ) -> Result<LinkSubtypeRecord, JsValue> {
        Ok(match subtype {
            LinkSubtypeConfigurationRead::Conduit(value) => LinkSubtypeRecord::Conduit {
                length: value.length,
                roughness: value.roughness,
                barrels: value.barrels,
                cross_section,
            },
            LinkSubtypeConfigurationRead::Pump(value) => {
                let curve = match value.configuration {
                    PumpOperatingConfigurationRead::Ideal => None,
                    PumpOperatingConfigurationRead::Curve { curve_index } => Some(identity_id(
                        self,
                        ObjectType::Curve,
                        curve_index,
                        "linkConfiguration",
                    )?),
                };
                LinkSubtypeRecord::Pump {
                    curve,
                    initial_setting: value.initial_setting,
                    startup_depth: value.startup_depth,
                    shutoff_depth: value.shutoff_depth,
                }
            }
            LinkSubtypeConfigurationRead::Orifice(value) => LinkSubtypeRecord::Orifice {
                orifice_kind: orifice_tag(value.kind),
                discharge_coefficient: value.discharge_coefficient,
                opening_time_hours: value.opening_time_hours,
            },
            LinkSubtypeConfigurationRead::Weir(value) => LinkSubtypeRecord::Weir {
                weir_kind: weir_tag(value.kind),
                discharge_coefficient: value.discharge_coefficient,
                end_discharge_coefficient: value.end_discharge_coefficient,
                end_contractions: value.end_contractions,
                can_surcharge: value.can_surcharge,
                roadway_width: value.roadway_width,
                roadway_surface: road_surface_tag(value.roadway_surface),
                coefficient_curve: value
                    .coefficient_curve_index
                    .map(|curve_index| {
                        identity_id(self, ObjectType::Curve, curve_index, "linkConfiguration")
                    })
                    .transpose()?,
            },
            LinkSubtypeConfigurationRead::Outlet(value) => LinkSubtypeRecord::Outlet {
                rating: match value.rating {
                    OutletRatingConfigurationRead::Functional {
                        coefficient,
                        exponent,
                        head_basis,
                    } => OutletRatingRecord::Functional {
                        coefficient,
                        exponent,
                        head_basis: outlet_head_basis_tag(head_basis),
                    },
                    OutletRatingConfigurationRead::Tabular {
                        curve_index,
                        head_basis,
                    } => OutletRatingRecord::Tabular {
                        curve: identity_id(
                            self,
                            ObjectType::Curve,
                            curve_index,
                            "linkConfiguration",
                        )?,
                        head_basis: outlet_head_basis_tag(head_basis),
                    },
                },
            },
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkConfigurationRecord {
    kind: String,
    tag: String,
    inlet_node: String,
    outlet_node: String,
    flow_direction: i8,
    length: Option<f64>,
    slope: Option<f64>,
    full_depth: f64,
    full_flow: f64,
    included_in_report: bool,
    inlet_offset: f64,
    outlet_offset: f64,
    initial_flow: f64,
    flow_limit: f64,
    inlet_loss_coefficient: f64,
    outlet_loss_coefficient: f64,
    average_loss_coefficient: f64,
    seepage_rate: f64,
    has_flap_gate: bool,
    subtype: LinkSubtypeRecord,
    cross_section: Option<CrossSectionRecord>,
    inlet: Option<InletConfigurationRecord>,
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum LinkSubtypeRecord {
    Conduit {
        length: f64,
        roughness: f64,
        barrels: u8,
        cross_section: Option<CrossSectionRecord>,
    },
    Pump {
        curve: Option<String>,
        initial_setting: f64,
        startup_depth: f64,
        shutoff_depth: f64,
    },
    Orifice {
        orifice_kind: String,
        discharge_coefficient: f64,
        opening_time_hours: f64,
    },
    Weir {
        weir_kind: String,
        discharge_coefficient: f64,
        end_discharge_coefficient: f64,
        end_contractions: f64,
        can_surcharge: bool,
        roadway_width: f64,
        roadway_surface: String,
        coefficient_curve: Option<String>,
    },
    Outlet {
        rating: OutletRatingRecord,
    },
}

#[derive(Serialize, Clone)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum CrossSectionRecord {
    Circular {
        diameter: f64,
    },
    Standard {
        shape: String,
        geometry: [f64; 4],
        culvert_code: usize,
    },
    Custom {
        shape: String,
        full_depth: f64,
        culvert_code: usize,
    },
    Irregular {
        transect: String,
        culvert_code: usize,
    },
    Street {
        street: String,
        culvert_code: usize,
    },
}

#[derive(Serialize, Clone)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum OutletRatingRecord {
    Functional {
        coefficient: f64,
        exponent: f64,
        head_basis: String,
    },
    Tabular {
        curve: String,
        head_basis: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InletConfigurationRecord {
    count: i32,
    percent_clogged: f64,
    flow_limit: f64,
    local_depression: f64,
    local_width: f64,
    design: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InletResultsRecord {
    flow_factor: f64,
    captured_flow: f64,
    backflow: f64,
    backflow_ratio: f64,
}

impl From<InletResultsRead> for InletResultsRecord {
    fn from(value: InletResultsRead) -> Self {
        Self {
            flow_factor: value.flow_factor,
            captured_flow: value.captured_flow,
            backflow: value.backflow,
            backflow_ratio: value.backflow_ratio,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkQualityRecord {
    pollutant_ids: Vec<String>,
    concentrations: Vec<f64>,
    reactor_concentrations: Vec<f64>,
    total_loads: Vec<f64>,
}

impl From<LinkQualityRead> for LinkQualityRecord {
    fn from(value: LinkQualityRead) -> Self {
        Self {
            pollutant_ids: value.pollutant_ids,
            concentrations: value.concentrations,
            reactor_concentrations: value.reactor_concentrations,
            total_loads: value.total_loads,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkQualitySnapshotRecord {
    object_ids: Vec<String>,
    pollutant_ids: Vec<String>,
    concentrations: Vec<Vec<f64>>,
    reactor_concentrations: Vec<Vec<f64>>,
    total_loads: Vec<Vec<f64>>,
}

impl From<LinkQualitySnapshotRead> for LinkQualitySnapshotRecord {
    fn from(value: LinkQualitySnapshotRead) -> Self {
        Self {
            object_ids: value.object_ids,
            pollutant_ids: value.pollutant_ids,
            concentrations: value.concentrations,
            reactor_concentrations: value.reactor_concentrations,
            total_loads: value.total_loads,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkStatisticsRecord {
    maximum_flow: f64,
    maximum_flow_time: String,
    maximum_velocity: f64,
    maximum_depth: f64,
    maximum_street_fill_fraction: Option<f64>,
    time_normal_flow_seconds: f64,
    time_inlet_control_seconds: f64,
    time_surcharged_seconds: f64,
    time_full_upstream_seconds: f64,
    time_full_downstream_seconds: f64,
    time_full_flow_seconds: f64,
    time_capacity_limited_seconds: f64,
    time_in_flow_class_seconds: Vec<f64>,
    time_courant_critical_seconds: f64,
    flow_turns: i64,
    flow_turn_sign: i32,
}

impl From<LinkStatisticsRead> for LinkStatisticsRecord {
    fn from(value: LinkStatisticsRead) -> Self {
        Self {
            maximum_flow: value.maximum_flow,
            maximum_flow_time: super::super::timestamp(value.maximum_flow_time),
            maximum_velocity: value.maximum_velocity,
            maximum_depth: value.maximum_depth,
            maximum_street_fill_fraction: value.maximum_street_fill_fraction,
            time_normal_flow_seconds: value.time_normal_flow_seconds,
            time_inlet_control_seconds: value.time_inlet_control_seconds,
            time_surcharged_seconds: value.time_surcharged_seconds,
            time_full_upstream_seconds: value.time_full_upstream_seconds,
            time_full_downstream_seconds: value.time_full_downstream_seconds,
            time_full_flow_seconds: value.time_full_flow_seconds,
            time_capacity_limited_seconds: value.time_capacity_limited_seconds,
            time_in_flow_class_seconds: value.time_in_flow_class_seconds.to_vec(),
            time_courant_critical_seconds: value.time_courant_critical_seconds,
            flow_turns: value.flow_turns,
            flow_turn_sign: value.flow_turn_sign,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PumpStatisticsRecord {
    time_utilized_seconds: f64,
    minimum_flow: f64,
    average_flow: f64,
    maximum_flow: f64,
    pumped_volume: f64,
    energy_consumed: f64,
    off_curve_low_seconds: f64,
    off_curve_high_seconds: f64,
    startup_count: i32,
    period_count: i32,
}

impl From<PumpStatisticsRead> for PumpStatisticsRecord {
    fn from(value: PumpStatisticsRead) -> Self {
        Self {
            time_utilized_seconds: value.time_utilized_seconds,
            minimum_flow: value.minimum_flow,
            average_flow: value.average_flow,
            maximum_flow: value.maximum_flow,
            pumped_volume: value.pumped_volume,
            energy_consumed: value.energy_consumed,
            off_curve_low_seconds: value.off_curve_low_seconds,
            off_curve_high_seconds: value.off_curve_high_seconds,
            startup_count: value.startup_count,
            period_count: value.period_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkStatisticsSnapshotRecord {
    object_ids: Vec<String>,
    maximum_flow: Vec<f64>,
    maximum_flow_time: Vec<String>,
    maximum_velocity: Vec<f64>,
    maximum_depth: Vec<f64>,
    maximum_street_fill_fraction: Vec<Option<f64>>,
    time_normal_flow_seconds: Vec<f64>,
    time_inlet_control_seconds: Vec<f64>,
    time_surcharged_seconds: Vec<f64>,
    time_full_upstream_seconds: Vec<f64>,
    time_full_downstream_seconds: Vec<f64>,
    time_full_flow_seconds: Vec<f64>,
    time_capacity_limited_seconds: Vec<f64>,
    time_in_flow_class_seconds: Vec<Vec<f64>>,
    time_courant_critical_seconds: Vec<f64>,
    flow_turns: Vec<i64>,
    flow_turn_sign: Vec<i32>,
}

impl From<LinkStatisticsSnapshotRead> for LinkStatisticsSnapshotRecord {
    fn from(value: LinkStatisticsSnapshotRead) -> Self {
        Self {
            object_ids: value.object_ids,
            maximum_flow: value.maximum_flow,
            maximum_flow_time: value
                .maximum_flow_time
                .into_iter()
                .map(super::super::timestamp)
                .collect(),
            maximum_velocity: value.maximum_velocity,
            maximum_depth: value.maximum_depth,
            maximum_street_fill_fraction: value.maximum_street_fill_fraction,
            time_normal_flow_seconds: value.time_normal_flow_seconds,
            time_inlet_control_seconds: value.time_inlet_control_seconds,
            time_surcharged_seconds: value.time_surcharged_seconds,
            time_full_upstream_seconds: value.time_full_upstream_seconds,
            time_full_downstream_seconds: value.time_full_downstream_seconds,
            time_full_flow_seconds: value.time_full_flow_seconds,
            time_capacity_limited_seconds: value.time_capacity_limited_seconds,
            time_in_flow_class_seconds: value
                .time_in_flow_class_seconds
                .into_iter()
                .map(|values| values.to_vec())
                .collect(),
            time_courant_critical_seconds: value.time_courant_critical_seconds,
            flow_turns: value.flow_turns,
            flow_turn_sign: value.flow_turn_sign,
        }
    }
}

fn present_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

#[derive(Deserialize, Clone)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum RatingPatchDto {
    Functional {
        coefficient: f64,
        exponent: f64,
        #[serde(default, deserialize_with = "non_null")]
        head_basis: Option<String>,
    },
    Tabular {
        curve: String,
        #[serde(default, deserialize_with = "non_null")]
        head_basis: Option<String>,
    },
}

#[derive(Deserialize, Clone)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum CrossSectionPatchDto {
    Circular {
        diameter: f64,
    },
    Standard {
        shape: String,
        geometry: [f64; 4],
        #[serde(default, deserialize_with = "non_null")]
        culvert_code: Option<usize>,
    },
    Custom {
        shape: String,
        full_depth: f64,
        #[serde(default, deserialize_with = "non_null")]
        culvert_code: Option<usize>,
    },
    Irregular {
        transect: String,
        #[serde(default, deserialize_with = "non_null")]
        culvert_code: Option<usize>,
    },
    Street {
        street: String,
        #[serde(default, deserialize_with = "non_null")]
        culvert_code: Option<usize>,
    },
}

#[derive(Deserialize, Clone)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum TaggedSubtypePatchDto {
    Conduit {
        #[serde(default, deserialize_with = "non_null")]
        length: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        roughness: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        barrels: Option<u8>,
    },
    Pump {
        #[serde(default, deserialize_with = "present_nullable")]
        curve: Option<Option<String>>,
        #[serde(default, deserialize_with = "non_null")]
        initial_setting: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        startup_depth: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        shutoff_depth: Option<f64>,
    },
    Orifice {
        #[serde(default, deserialize_with = "non_null")]
        orifice_kind: Option<String>,
        #[serde(default, deserialize_with = "non_null")]
        discharge_coefficient: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        opening_time_hours: Option<f64>,
    },
    Weir {
        #[serde(default, deserialize_with = "non_null")]
        weir_kind: Option<String>,
        #[serde(default, deserialize_with = "non_null")]
        discharge_coefficient: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        end_discharge_coefficient: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        end_contractions: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        can_surcharge: Option<bool>,
        #[serde(default, deserialize_with = "non_null")]
        roadway_width: Option<f64>,
        #[serde(default, deserialize_with = "non_null")]
        roadway_surface: Option<String>,
        #[serde(default, deserialize_with = "present_nullable")]
        coefficient_curve: Option<Option<String>>,
    },
    Outlet {
        #[serde(default, deserialize_with = "non_null")]
        rating: Option<RatingPatchDto>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InletPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    percent_clogged: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    flow_limit: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LinkPatchDto {
    #[serde(default, deserialize_with = "non_null")]
    tag: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    inlet_node: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    outlet_node: Option<String>,
    #[serde(default, deserialize_with = "non_null")]
    inlet_offset: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    outlet_offset: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    included_in_report: Option<bool>,
    #[serde(default, deserialize_with = "non_null")]
    initial_flow: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    inlet_loss_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    outlet_loss_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    average_loss_coefficient: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    seepage_rate: Option<f64>,
    #[serde(default, deserialize_with = "non_null")]
    has_flap_gate: Option<bool>,
    #[serde(default, deserialize_with = "non_null")]
    cross_section: Option<CrossSectionPatchDto>,
    #[serde(default, deserialize_with = "non_null")]
    subtype: Option<TaggedSubtypePatchDto>,
}

impl LinkPatchDto {
    fn into_patch(self, simulation: &Simulation) -> Result<LinkPatch, JsValue> {
        let operation = "configureLink";
        Ok(LinkPatch {
            tag: self.tag,
            inlet_node_index: self
                .inlet_node
                .as_deref()
                .map(|id| simulation.index(ObjectType::Node, id))
                .transpose()?,
            outlet_node_index: self
                .outlet_node
                .as_deref()
                .map(|id| simulation.index(ObjectType::Node, id))
                .transpose()?,
            inlet_offset: self.inlet_offset,
            outlet_offset: self.outlet_offset,
            included_in_report: self.included_in_report,
            initial_flow: self.initial_flow,
            inlet_loss_coefficient: self.inlet_loss_coefficient,
            outlet_loss_coefficient: self.outlet_loss_coefficient,
            average_loss_coefficient: self.average_loss_coefficient,
            seepage_rate: self.seepage_rate,
            has_flap_gate: self.has_flap_gate,
            cross_section: self
                .cross_section
                .map(|value| {
                    cross_section_patch(simulation, value)
                        .map_err(|error| simulation.error(operation, error))
                })
                .transpose()?,
            subtype: self
                .subtype
                .map(|value| subtype_patch(simulation, value))
                .transpose()
                .map_err(|error| simulation.error(operation, error))?,
        })
    }
}

fn subtype_patch(
    simulation: &Simulation,
    value: TaggedSubtypePatchDto,
) -> Result<LinkSubtypePatch, swmmrs::engine::error::SwmmError> {
    match value {
        TaggedSubtypePatchDto::Conduit {
            length,
            roughness,
            barrels,
        } => Ok(LinkSubtypePatch::Conduit(ConduitPatch {
            length,
            roughness,
            barrels,
        })),
        TaggedSubtypePatchDto::Pump {
            curve,
            initial_setting,
            startup_depth,
            shutoff_depth,
        } => {
            let configuration = match curve {
                None => None,
                Some(None) => Some(PumpConfiguration::Ideal),
                Some(Some(id)) => Some(PumpConfiguration::Curve {
                    curve_index: configured_index(simulation, ObjectType::Curve, &id)?,
                }),
            };
            Ok(LinkSubtypePatch::Pump(PumpUpdate {
                configuration,
                initial_setting,
                startup_depth,
                shutoff_depth,
            }))
        }
        TaggedSubtypePatchDto::Orifice {
            orifice_kind,
            discharge_coefficient,
            opening_time_hours,
        } => Ok(LinkSubtypePatch::Orifice(OrificeUpdate {
            kind: orifice_kind
                .map(|kind| parse_orifice_kind(&kind))
                .transpose()?,
            discharge_coefficient,
            opening_time_hours,
        })),
        TaggedSubtypePatchDto::Weir {
            weir_kind,
            discharge_coefficient,
            end_discharge_coefficient,
            end_contractions,
            can_surcharge,
            roadway_width,
            roadway_surface,
            coefficient_curve,
        } => Ok(LinkSubtypePatch::Weir(WeirUpdate {
            kind: weir_kind.map(|kind| parse_weir_kind(&kind)).transpose()?,
            discharge_coefficient,
            end_discharge_coefficient,
            end_contractions,
            can_surcharge,
            roadway_width,
            roadway_surface: roadway_surface
                .map(|surface| parse_road_surface(&surface))
                .transpose()?,
            coefficient_curve_index: match coefficient_curve {
                None => None,
                Some(None) => Some(None),
                Some(Some(id)) => Some(Some(configured_index(simulation, ObjectType::Curve, &id)?)),
            },
        })),
        TaggedSubtypePatchDto::Outlet { rating } => Ok(LinkSubtypePatch::Outlet(OutletUpdate {
            rating: rating
                .map(|rating| outlet_rating(simulation, rating))
                .transpose()?,
        })),
    }
}

fn outlet_rating(
    simulation: &Simulation,
    value: RatingPatchDto,
) -> Result<OutletRating, swmmrs::engine::error::SwmmError> {
    match value {
        RatingPatchDto::Functional {
            coefficient,
            exponent,
            head_basis,
        } => Ok(OutletRating::Functional {
            coefficient,
            exponent,
            head_basis: parse_head_basis(head_basis.as_deref().unwrap_or("depth"))?,
        }),
        RatingPatchDto::Tabular { curve, head_basis } => Ok(OutletRating::Tabular {
            curve_index: configured_index(simulation, ObjectType::Curve, &curve)?,
            head_basis: parse_head_basis(head_basis.as_deref().unwrap_or("depth"))?,
        }),
    }
}

fn cross_section_patch(
    simulation: &Simulation,
    value: CrossSectionPatchDto,
) -> Result<CrossSectionConfiguration, swmmrs::engine::error::SwmmError> {
    match value {
        CrossSectionPatchDto::Circular { diameter } => {
            Ok(CrossSectionConfiguration::Circular { diameter })
        }
        CrossSectionPatchDto::Standard {
            shape,
            geometry,
            culvert_code,
        } => Ok(CrossSectionConfiguration::Standard {
            shape: parse_xsect_shape(&shape)?,
            geometry,
            culvert_code: culvert_code.unwrap_or(0),
        }),
        CrossSectionPatchDto::Custom {
            shape,
            full_depth,
            culvert_code,
        } => {
            Ok(CrossSectionConfiguration::Custom {
                shape_curve_index: simulation.inner.custom_shape_curve_index_read(
                    configured_index(simulation, ObjectType::Shape, &shape)?,
                )?,
                full_depth,
                culvert_code: culvert_code.unwrap_or(0),
            })
        }
        CrossSectionPatchDto::Irregular {
            transect,
            culvert_code,
        } => Ok(CrossSectionConfiguration::Irregular {
            transect_index: configured_index(simulation, ObjectType::Transect, &transect)?,
            culvert_code: culvert_code.unwrap_or(0),
        }),
        CrossSectionPatchDto::Street {
            street,
            culvert_code,
        } => Ok(CrossSectionConfiguration::Street {
            street_index: configured_index(simulation, ObjectType::Street, &street)?,
            culvert_code: culvert_code.unwrap_or(0),
        }),
    }
}

fn cross_section_record(value: LinkCrossSectionValue) -> CrossSectionRecord {
    match value {
        LinkCrossSectionValue::Circular { diameter } => CrossSectionRecord::Circular { diameter },
        LinkCrossSectionValue::Standard {
            shape,
            geometry,
            culvert_code,
        } => CrossSectionRecord::Standard {
            shape: xsect_tag(shape),
            geometry,
            culvert_code,
        },
        LinkCrossSectionValue::Custom {
            shape,
            full_depth,
            culvert_code,
        } => CrossSectionRecord::Custom {
            shape: shape.id,
            full_depth,
            culvert_code,
        },
        LinkCrossSectionValue::Irregular {
            transect,
            culvert_code,
        } => CrossSectionRecord::Irregular {
            transect: transect.id,
            culvert_code,
        },
        LinkCrossSectionValue::Street {
            street,
            culvert_code,
        } => CrossSectionRecord::Street {
            street: street.id,
            culvert_code,
        },
    }
}

fn mapping_record(ids: Vec<String>, values: Vec<f64>) -> BTreeMap<String, f64> {
    ids.into_iter().zip(values).collect()
}

fn configured_index(
    simulation: &Simulation,
    object_type: ObjectType,
    id: &str,
) -> Result<usize, swmmrs::engine::error::SwmmError> {
    simulation
        .inner
        .object_identity_read(object_type, id)?
        .map(|identity| identity.index)
        .ok_or_else(|| {
            swmmrs::engine::error::SwmmError::with_detail(
                swmmrs::engine::error::ErrorCode::ApiObjectName,
                format!("unknown object: {id}"),
            )
        })
}

fn property_error(detail: &str) -> swmmrs::engine::error::SwmmError {
    swmmrs::engine::error::SwmmError::with_detail(
        swmmrs::engine::error::ErrorCode::ApiPropertyValue,
        detail,
    )
}

fn parse_orifice_kind(value: &str) -> Result<OrificeType, swmmrs::engine::error::SwmmError> {
    match value {
        "side" => Ok(OrificeType::SideOrifice),
        "bottom" => Ok(OrificeType::BottomOrifice),
        _ => Err(property_error("unknown orifice kind")),
    }
}

fn parse_weir_kind(value: &str) -> Result<WeirType, swmmrs::engine::error::SwmmError> {
    match value {
        "transverse" => Ok(WeirType::TransverseWeir),
        "sideflow" => Ok(WeirType::SideflowWeir),
        "v_notch" => Ok(WeirType::VnotchWeir),
        "trapezoidal" => Ok(WeirType::TrapezoidalWeir),
        "roadway" => Ok(WeirType::RoadwayWeir),
        _ => Err(property_error("unknown weir kind")),
    }
}

fn parse_road_surface(value: &str) -> Result<RoadSurface, swmmrs::engine::error::SwmmError> {
    match value {
        "unspecified" => Ok(RoadSurface::Unspecified),
        "paved" => Ok(RoadSurface::PAVED),
        "gravel" => Ok(RoadSurface::GRAVEL),
        _ => Err(property_error("unknown roadway surface")),
    }
}

fn parse_head_basis(value: &str) -> Result<OutletHeadBasis, swmmrs::engine::error::SwmmError> {
    match value {
        "depth" => Ok(OutletHeadBasis::Depth),
        "head" => Ok(OutletHeadBasis::Head),
        _ => Err(property_error("unknown outlet head basis")),
    }
}

fn parse_xsect_shape(value: &str) -> Result<XsectType, swmmrs::engine::error::SwmmError> {
    Ok(match value {
        "dummy" => XsectType::Dummy,
        "circular" => XsectType::Circular,
        "filled_circular" => XsectType::FilledCircular,
        "rect_closed" => XsectType::RectClosed,
        "rect_open" => XsectType::RectOpen,
        "trapezoidal" => XsectType::Trapezoidal,
        "triangular" => XsectType::Triangular,
        "parabolic" => XsectType::Parabolic,
        "power_function" => XsectType::Powerfunc,
        "rect_triangular" => XsectType::RectTriang,
        "rect_round" => XsectType::RectRound,
        "modified_basket" => XsectType::ModBasket,
        "horizontal_ellipse" => XsectType::HorizEllipse,
        "vertical_ellipse" => XsectType::VertEllipse,
        "arch" => XsectType::Arch,
        "egg_shaped" => XsectType::Eggshaped,
        "horseshoe" => XsectType::Horseshoe,
        "gothic" => XsectType::Gothic,
        "catenary" => XsectType::Catenary,
        "semi_elliptical" => XsectType::Semielliptical,
        "basket_handle" => XsectType::Baskethandle,
        "semi_circular" => XsectType::Semicircular,
        "force_main" => XsectType::ForceMain,
        _ => return Err(property_error("unknown standard cross-section shape")),
    })
}

fn xsect_tag(value: XsectType) -> String {
    match value {
        XsectType::Dummy => "dummy",
        XsectType::Circular => "circular",
        XsectType::FilledCircular => "filled_circular",
        XsectType::RectClosed => "rect_closed",
        XsectType::RectOpen => "rect_open",
        XsectType::Trapezoidal => "trapezoidal",
        XsectType::Triangular => "triangular",
        XsectType::Parabolic => "parabolic",
        XsectType::Powerfunc => "power_function",
        XsectType::RectTriang => "rect_triangular",
        XsectType::RectRound => "rect_round",
        XsectType::ModBasket => "modified_basket",
        XsectType::HorizEllipse => "horizontal_ellipse",
        XsectType::VertEllipse => "vertical_ellipse",
        XsectType::Arch => "arch",
        XsectType::Eggshaped => "egg_shaped",
        XsectType::Horseshoe => "horseshoe",
        XsectType::Gothic => "gothic",
        XsectType::Catenary => "catenary",
        XsectType::Semielliptical => "semi_elliptical",
        XsectType::Baskethandle => "basket_handle",
        XsectType::Semicircular => "semi_circular",
        XsectType::Irregular => "irregular",
        XsectType::Custom => "custom",
        XsectType::ForceMain => "force_main",
        XsectType::StreetXsect => "street",
    }
    .into()
}

fn orifice_tag(value: OrificeType) -> String {
    match value {
        OrificeType::SideOrifice => "side",
        OrificeType::BottomOrifice => "bottom",
    }
    .into()
}

fn weir_tag(value: WeirType) -> String {
    match value {
        WeirType::TransverseWeir => "transverse",
        WeirType::SideflowWeir => "sideflow",
        WeirType::VnotchWeir => "v_notch",
        WeirType::TrapezoidalWeir => "trapezoidal",
        WeirType::RoadwayWeir => "roadway",
    }
    .into()
}

fn road_surface_tag(value: RoadSurface) -> String {
    match value {
        RoadSurface::Unspecified => "unspecified",
        RoadSurface::PAVED => "paved",
        RoadSurface::GRAVEL => "gravel",
    }
    .into()
}

fn outlet_head_basis_tag(value: OutletHeadBasis) -> String {
    match value {
        OutletHeadBasis::Depth => "depth",
        OutletHeadBasis::Head => "head",
    }
    .into()
}
