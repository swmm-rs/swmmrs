use super::*;

#[pymethods]
impl NativeSimulation {
    /// Reads one complete AMM model through a validated Live View.
    fn amm_model_configuration(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
    ) -> PyResult<Py<PyAny>> {
        let (configuration, rain_gage) = self.checked_view(
            py,
            "amm_model_configuration",
            generation,
            ObjectType::AmmModel,
            index,
            &id,
            &subtype,
            |simulation| {
                let configuration = simulation.amm_model_configuration_read(index)?;
                let rain_gage = simulation
                    .object_identity_at_read(ObjectType::Gage, configuration.rain_gage_index)?
                    .ok_or_else(|| {
                        SwmmError::with_detail(
                            ErrorCode::ApiPropertyValue,
                            "configured AMM rain gage is unavailable",
                        )
                    })?;
                Ok((configuration, rain_gage))
            },
        )?;
        let rain_gage = relationship_view(py, facade, generation, ObjectType::Gage, rain_gage)?;
        let components = PyTuple::new(
            py,
            configuration
                .components
                .into_iter()
                .map(|component| amm_component_to_python(py, component))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        (
            rain_gage,
            configuration.cold_temperature,
            configuration.hot_temperature,
            components,
        )
            .into_py_any(py)
    }

    /// Applies one atomic AMM model update through a validated Live View.
    fn update_amm_model(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_amm_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_amm_model",
            generation,
            ObjectType::AmmModel,
            index,
            id,
            subtype,
            move |simulation| {
                let mut patch = update.patch;
                if let Some(rain_gage) = update.rain_gage {
                    validate_related_object(
                        simulation,
                        &rain_gage,
                        &[ObjectType::Gage],
                        "rain_gage",
                    )?;
                    patch.rain_gage_index = Some(rain_gage.index);
                }
                simulation.patch_amm_model(index, patch)
            },
        )
    }

    /// Reads all AMM assignments with related Live Views and project-unit areas.
    fn amm_assignments(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
    ) -> PyResult<Py<PyAny>> {
        let assignments: Vec<(ObjectIdentityRead, ObjectIdentityRead, f64)> = self
            .checked_generation(py, "amm_assignments", generation, |handle| {
                let read = || -> Result<_, SwmmError> {
                    handle
                        .simulation
                        .amm_assignments_read()?
                        .into_iter()
                        .map(|assignment| {
                            let node = handle
                                .simulation
                                .object_identity_at_read(ObjectType::Node, assignment.node_index)?
                                .ok_or_else(|| {
                                    assignment_error("configured AMM node is unavailable")
                                })?;
                            let model = handle
                                .simulation
                                .object_identity_at_read(
                                    ObjectType::AmmModel,
                                    assignment.model_index,
                                )?
                                .ok_or_else(|| {
                                    assignment_error("configured AMM model is unavailable")
                                })?;
                            Ok((node, model, assignment.area))
                        })
                        .collect()
                };
                read().map_err(|error| DetachedFailure::Solver("amm_assignments", error))
            })?;
        let values = assignments
            .into_iter()
            .map(|(node, model, area)| {
                let node = relationship_view(py, facade, generation, ObjectType::Node, node)?;
                let model = relationship_view(py, facade, generation, ObjectType::AmmModel, model)?;
                (node, model, area).into_py_any(py)
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, values)?.unbind().into_any())
    }

    /// Atomically replaces all AMM assignments.
    fn replace_amm_assignments(
        &self,
        py: Python<'_>,
        generation: u64,
        assignments: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let assignments = extract_assignments(py, self, generation, assignments)?;
        self.detached_checked_generation(py, "replace_amm_assignments", generation, move |handle| {
            let assignments = assignments
                .into_iter()
                .map(|assignment| {
                    validate_related_object(
                        &handle.simulation,
                        &assignment.node,
                        &[ObjectType::Node],
                        "node",
                    )?;
                    validate_related_object(
                        &handle.simulation,
                        &assignment.model,
                        &[ObjectType::AmmModel],
                        "model",
                    )?;
                    Ok(AmmAssignmentConfiguration {
                        node_index: assignment.node.index,
                        model_index: assignment.model.index,
                        area: assignment.area,
                    })
                })
                .collect::<Result<Vec<_>, SwmmError>>()?;
            handle.simulation.replace_amm_assignments(assignments)
        })
    }
}

struct AmmUpdate {
    patch: AmmModelPatch,
    rain_gage: Option<RelatedObject>,
}

struct AmmAssignmentUpdate {
    node: RelatedObject,
    model: RelatedObject,
    area: f64,
}

fn extract_assignments(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<AmmAssignmentUpdate>> {
    value
        .try_iter()
        .map_err(|_| assignment_py_error("assignments must be iterable"))?
        .map(|item| {
            let item = item?;
            let tuple = item
                .cast::<PyTuple>()
                .map_err(|_| assignment_py_error("each assignment must be a tuple"))?;
            if tuple.len() != 3 {
                return Err(assignment_py_error(
                    "each assignment must contain node, model, and area",
                ));
            }
            let node = related_live_view(
                py,
                owner,
                generation,
                &tuple.get_item(0)?,
                "node",
                "replace_amm_assignments",
                false,
            )?
            .ok_or_else(|| assignment_py_error("node must be a Live View"))?;
            let model = related_live_view(
                py,
                owner,
                generation,
                &tuple.get_item(1)?,
                "model",
                "replace_amm_assignments",
                false,
            )?
            .ok_or_else(|| assignment_py_error("model must be a Live View"))?;
            let area = assignment_float(&tuple.get_item(2)?)?;
            Ok(AmmAssignmentUpdate { node, model, area })
        })
        .collect()
}

fn assignment_float(value: &Bound<'_, PyAny>) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(assignment_py_error("area must not be bool"));
    }
    value
        .extract::<f64>()
        .map_err(|_| assignment_py_error("area must be numeric"))
}

fn assignment_error(detail: impl Into<String>) -> SwmmError {
    SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail)
}

fn assignment_py_error(detail: &str) -> PyErr {
    native_error(
        "replace_amm_assignments",
        assignment_error(detail.to_owned()),
    )
}

fn extract_amm_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<AmmUpdate> {
    let mut patch = AmmModelPatch::default();
    let mut rain_gage = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| amm_py_error("AMM update fields must be strings"))?;
        match name.as_str() {
            "rain_gage" => {
                rain_gage = related_live_view(
                    py,
                    owner,
                    generation,
                    &value,
                    "rain_gage",
                    "update_amm_model",
                    false,
                )?;
            }
            "cold_temperature" => patch.cold_temperature = Some(amm_float(&value, &name)?),
            "hot_temperature" => patch.hot_temperature = Some(amm_float(&value, &name)?),
            "components" => patch.components = Some(extract_components(&value)?),
            _ => return Err(amm_py_error(&format!("unknown AMM model field: {name}"))),
        }
    }
    Ok(AmmUpdate { patch, rain_gage })
}

fn extract_components(value: &Bound<'_, PyAny>) -> PyResult<Vec<AmmComponentConfiguration>> {
    let iterator = value
        .try_iter()
        .map_err(|_| amm_py_error("components must be iterable"))?;
    iterator
        .map(|item| extract_component(&item?))
        .collect::<PyResult<Vec<_>>>()
}

fn extract_component(value: &Bound<'_, PyAny>) -> PyResult<AmmComponentConfiguration> {
    let tuple = value
        .cast::<PyTuple>()
        .map_err(|_| amm_py_error("each component must use the native tuple format"))?;
    let kind = tuple
        .get_item(0)
        .and_then(|value| value.extract::<String>())
        .map_err(|_| amm_py_error("component kind must be a string"))?;
    let id = tuple
        .get_item(1)
        .and_then(|value| value.extract::<String>())
        .map_err(|_| amm_py_error("component id must be a string"))?;
    let number = |index: usize, name: &str| -> PyResult<f64> {
        tuple
            .get_item(index)
            .and_then(|value| amm_float(&value, name))
    };
    match kind.as_str() {
        "standard" if tuple.len() == 11 => Ok(AmmComponentConfiguration::Standard {
            id,
            dry_capture: number(2, "dry_capture")?,
            precipitation_window_hours: number(3, "precipitation_window_hours")?,
            hydrograph_half_life_hours: number(4, "hydrograph_half_life_hours")?,
            initial_wet_capture: number(5, "initial_wet_capture")?,
            moisture_half_life_hours: number(6, "moisture_half_life_hours")?,
            temperature_window_hours: number(7, "temperature_window_hours")?,
            spring_cold_shcf: number(8, "spring_cold_shcf")?,
            hot_shcf: number(9, "hot_shcf")?,
            fall_cold_shcf: number(10, "fall_cold_shcf")?,
        }),
        "baseflow" if tuple.len() == 8 => Ok(AmmComponentConfiguration::Baseflow {
            id,
            precipitation_window_hours: number(2, "precipitation_window_hours")?,
            hydrograph_half_life_hours: number(3, "hydrograph_half_life_hours")?,
            temperature_window_hours: number(4, "temperature_window_hours")?,
            spring_cold_capture: number(5, "spring_cold_capture")?,
            hot_capture: number(6, "hot_capture")?,
            fall_cold_capture: number(7, "fall_cold_capture")?,
        }),
        "standard" | "baseflow" => Err(amm_py_error("component tuple has the wrong length")),
        _ => Err(amm_py_error(
            "component kind must be 'standard' or 'baseflow'",
        )),
    }
}

fn amm_component_to_python(
    py: Python<'_>,
    component: AmmComponentConfiguration,
) -> PyResult<Py<PyAny>> {
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
        } => (
            "standard",
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
        )
            .into_py_any(py),
        AmmComponentConfiguration::Baseflow {
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        } => (
            "baseflow",
            id,
            precipitation_window_hours,
            hydrograph_half_life_hours,
            temperature_window_hours,
            spring_cold_capture,
            hot_capture,
            fall_cold_capture,
        )
            .into_py_any(py),
    }
}

fn amm_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(amm_py_error(&format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| amm_py_error(&format!("{name} must be numeric")))
}

fn amm_py_error(detail: &str) -> PyErr {
    native_error(
        "update_amm_model",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}
