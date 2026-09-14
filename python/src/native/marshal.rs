use super::*;

pub(super) type DateTimeParts = (i32, u8, u8, u8, u8, u8);
pub(super) type PollutantMappingParts = (Vec<String>, Vec<f64>);
/// Converts one aligned pollutant mapping to the private tuple.
///
/// # Arguments
/// * `values` - Aligned pollutant IDs and values in configured public units.
///
/// # Returns
/// Owned pollutant ID and value vectors in configured public units.
pub(super) fn pollutant_mapping_tuple(values: PollutantMappingRead) -> PollutantMappingParts {
    (values.pollutant_ids, values.values)
}

/// Selects one aligned current link-quality mapping.
///
/// # Arguments
/// * `values` - Current link quality in configured reporting units.
/// * `field` - Private link-quality field name.
///
/// # Returns
/// Owned pollutant IDs and the selected aligned value vector.
///
/// # Errors
/// Returns an internal failure for an invalid private field.
pub(super) fn link_quality_mapping(
    values: LinkQualityRead,
    field: &str,
) -> PyResult<PollutantMappingParts> {
    let selected = match field {
        "concentrations" => values.concentrations,
        "reactor_concentrations" => values.reactor_concentrations,
        "total_loads" => values.total_loads,
        _ => {
            return Err(internal_error(
                "link_quality_mapping",
                "invalid private link quality field",
            ));
        }
    };
    Ok((values.pollutant_ids, selected))
}

/// Selects one aligned current node-quality mapping.
///
/// # Arguments
/// * `values` - Current node quality in configured concentration units.
/// * `field` - Private node-quality field name.
///
/// # Returns
/// Owned pollutant IDs and the selected aligned value vector.
///
/// # Errors
/// Returns an internal failure for an invalid private field.
pub(super) fn node_quality_mapping(
    values: NodeQualityRead,
    field: &str,
) -> PyResult<PollutantMappingParts> {
    let selected = match field {
        "concentrations" => values.concentrations,
        "inflow_concentrations" => values.inflow_concentrations,
        "reactor_concentrations" => values.reactor_concentrations,
        _ => {
            return Err(internal_error(
                "node_quality_mapping",
                "invalid private node quality field",
            ));
        }
    };
    Ok((values.pollutant_ids, selected))
}

/// Selects one aligned current subcatchment-quality mapping.
///
/// # Arguments
/// * `values` - Current subcatchment quality in configured reporting units.
/// * `field` - Private subcatchment-quality field name.
///
/// # Returns
/// Owned pollutant IDs and the selected aligned value vector.
///
/// # Errors
/// Returns an internal failure for an invalid private field.
pub(super) fn subcatchment_quality_mapping(
    values: SubcatchmentQualityRead,
    field: &str,
) -> PyResult<PollutantMappingParts> {
    let selected = match field {
        "runoff_concentrations" => values.runoff_concentrations,
        "ponded_concentrations" => values.ponded_concentrations,
        "buildup_loads" => values.buildup_loads,
        "total_washoff_loads" => values.total_washoff_loads,
        _ => {
            return Err(internal_error(
                "subcatchment_quality_mapping",
                "invalid private subcatchment quality field",
            ));
        }
    };
    Ok((values.pollutant_ids, selected))
}

/// Converts one selected LID-unit configuration value to a Python object.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Owned targeted LID-unit value in public units.
///
/// # Returns
/// Selected Python-owned LID-unit value.
///
/// # Errors
/// Returns a Python conversion error.
pub(super) fn lid_unit_value(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: LidUnitValue,
) -> PyResult<Py<PyAny>> {
    match value {
        LidUnitValue::Float(value) => value.into_py_any(py),
        LidUnitValue::Integer(value) => value.into_py_any(py),
        LidUnitValue::Bool(value) => value.into_py_any(py),
        LidUnitValue::Relationship(identity) => {
            relationship_view(py, facade, generation, ObjectType::Lid, identity)
        }
        LidUnitValue::DrainDestination(LidDrainDestinationRead::None) => {
            None::<Py<PyAny>>.into_py_any(py)
        }
        LidUnitValue::DrainDestination(LidDrainDestinationRead::Node(identity)) => {
            relationship_view(py, facade, generation, ObjectType::Node, identity)
        }
        LidUnitValue::DrainDestination(LidDrainDestinationRead::Subcatchment(identity)) => {
            relationship_view(py, facade, generation, ObjectType::Subcatch, identity)
        }
    }
}

/// Converts one selected LID-control layer value to a Python object.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Owned targeted LID-layer value in public units.
///
/// # Returns
/// Selected Python-owned layer value.
///
/// # Errors
/// Returns a Python conversion error.
pub(super) fn lid_control_value(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: LidControlValue,
) -> PyResult<Py<PyAny>> {
    match value {
        LidControlValue::Float(value) => value.into_py_any(py),
        LidControlValue::Bool(value) => value.into_py_any(py),
        LidControlValue::Relationship(identity) => identity
            .map(|identity| relationship_view(py, facade, generation, ObjectType::Curve, identity))
            .transpose()?
            .into_py_any(py),
    }
}

/// Converts one selected simulation option to a Python object.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Targeted native option value in its public unit.
///
/// # Returns
/// Selected Python-owned option value.
///
/// # Errors
/// Returns a Python conversion error.
pub(super) fn option_value(py: Python<'_>, value: SimulationOptionValue) -> PyResult<Py<PyAny>> {
    match value {
        SimulationOptionValue::Number(value) => value.into_py_any(py),
        SimulationOptionValue::Integer(value) => value.into_py_any(py),
        SimulationOptionValue::Flag(value) => value.into_py_any(py),
        SimulationOptionValue::CustomEllipseModel(value) => match value {
            CustomEllipseModel::EpaLegacy => "epa_legacy",
            CustomEllipseModel::TrueEllipse => "true_ellipse",
        }
        .into_py_any(py),
        SimulationOptionValue::SurchargeMethod(value) => match value {
            SurchargeMethodType::Extran => "extran",
            SurchargeMethodType::Slot => "slot",
        }
        .into_py_any(py),
        SimulationOptionValue::InertiaDamping(value) => match value {
            InertialDampingType::NoDamping => "none",
            InertialDampingType::PartialDamping => "partial",
            InertialDampingType::FullDamping => "full",
        }
        .into_py_any(py),
        SimulationOptionValue::NormalFlowLimit(value) => match value {
            NormalFlowType::Slope => "slope",
            NormalFlowType::Froude => "froude",
            NormalFlowType::Both => "both",
            NormalFlowType::Neither => "neither",
        }
        .into_py_any(py),
    }
}

/// Builds Python-owned option values for aggregate inspection.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Owned native option snapshot in public units.
///
/// # Returns
/// Python-owned dictionary of every supported option.
///
/// # Errors
/// Returns a Python exception when any dictionary item cannot be installed.
pub(super) fn options_dict(py: Python<'_>, values: SimulationOptionsRead) -> PyResult<Py<PyDict>> {
    let result = PyDict::new(py);
    result.set_item("routing_step", values.routing_step_seconds)?;
    result.set_item("report_step", values.report_step_seconds)?;
    result.set_item(
        "detailed_reporting_enabled",
        values.detailed_reporting_enabled,
    )?;
    result.set_item(
        "custom_ellipse_model",
        match values.custom_ellipse_model {
            CustomEllipseModel::EpaLegacy => "epa_legacy",
            CustomEllipseModel::TrueEllipse => "true_ellipse",
        },
    )?;
    result.set_item(
        "surcharge_method",
        match values.surcharge_method {
            SurchargeMethodType::Extran => "extran",
            SurchargeMethodType::Slot => "slot",
        },
    )?;
    result.set_item("allow_ponding", values.allow_ponding)?;
    result.set_item(
        "inertia_damping",
        match values.inertia_damping {
            InertialDampingType::NoDamping => "none",
            InertialDampingType::PartialDamping => "partial",
            InertialDampingType::FullDamping => "full",
        },
    )?;
    result.set_item(
        "normal_flow_limit",
        match values.normal_flow_limit {
            NormalFlowType::Slope => "slope",
            NormalFlowType::Froude => "froude",
            NormalFlowType::Both => "both",
            NormalFlowType::Neither => "neither",
        },
    )?;
    result.set_item("skip_steady_state", values.skip_steady_state)?;
    result.set_item("rainfall_enabled", values.rainfall_enabled)?;
    result.set_item("rdii_enabled", values.rdii_enabled)?;
    result.set_item("snowmelt_enabled", values.snowmelt_enabled)?;
    result.set_item("groundwater_enabled", values.groundwater_enabled)?;
    result.set_item("routing_enabled", values.routing_enabled)?;
    result.set_item("quality_enabled", values.quality_enabled)?;
    result.set_item("rule_step", values.rule_step_seconds)?;
    result.set_item("sweep_start", values.sweep_start)?;
    result.set_item("sweep_end", values.sweep_end)?;
    result.set_item("maximum_trials", values.maximum_trials)?;
    result.set_item("requested_threads", values.requested_threads)?;
    result.set_item("minimum_routing_step", values.minimum_routing_step_seconds)?;
    result.set_item("lengthening_step", values.lengthening_step_seconds)?;
    result.set_item("antecedent_dry_duration", values.antecedent_dry_seconds)?;
    result.set_item("courant_factor", values.courant_factor)?;
    result.set_item("minimum_surface_area", values.minimum_surface_area)?;
    result.set_item("minimum_conduit_slope", values.minimum_conduit_slope)?;
    result.set_item("head_tolerance", values.head_tolerance)?;
    result.set_item("system_flow_tolerance", values.system_flow_tolerance)?;
    result.set_item("lateral_flow_tolerance", values.lateral_flow_tolerance)?;
    Ok(result.unbind())
}
