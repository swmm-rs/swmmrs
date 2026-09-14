use super::*;

#[pymethods]
impl NativeSimulation {
    /// Reads one complete RTK unit hydrograph through a validated Live View.
    fn unit_hydrograph_configuration(
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
            "unit_hydrograph_configuration",
            generation,
            ObjectType::Unithyd,
            index,
            &id,
            &subtype,
            |simulation| {
                let configuration = simulation.unit_hydrograph_configuration_read(index)?;
                let rain_gage = simulation
                    .object_identity_at_read(ObjectType::Gage, configuration.rain_gage_index)?
                    .ok_or_else(|| {
                        SwmmError::with_detail(
                            ErrorCode::ApiPropertyValue,
                            "configured unit-hydrograph rain gage is unavailable",
                        )
                    })?;
                Ok((configuration, rain_gage))
            },
        )?;
        let rain_gage = relationship_view(py, facade, generation, ObjectType::Gage, rain_gage)?;
        let monthly_responses = responses_to_python(py, configuration.monthly_responses)?;
        (rain_gage, monthly_responses).into_py_any(py)
    }

    /// Applies one atomic RTK unit-hydrograph update through a validated Live View.
    fn update_unit_hydrograph(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_unit_hydrograph_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_unit_hydrograph",
            generation,
            ObjectType::Unithyd,
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
                simulation.patch_unit_hydrograph(index, patch)
            },
        )
    }

    /// Reads all RTK RDII assignments with related Live Views and project-unit areas.
    fn rdii_assignments(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
    ) -> PyResult<Py<PyAny>> {
        let assignments: Vec<(ObjectIdentityRead, ObjectIdentityRead, f64)> = self
            .checked_generation(py, "rdii_assignments", generation, |handle| {
                let read = || -> Result<_, SwmmError> {
                    handle
                        .simulation
                        .rdii_assignments_read()?
                        .into_iter()
                        .map(|assignment| {
                            let node = handle
                                .simulation
                                .object_identity_at_read(ObjectType::Node, assignment.node_index)?
                                .ok_or_else(|| {
                                    assignment_error("configured RDII node is unavailable")
                                })?;
                            let unit_hydrograph = handle
                                .simulation
                                .object_identity_at_read(
                                    ObjectType::Unithyd,
                                    assignment.unit_hydrograph_index,
                                )?
                                .ok_or_else(|| {
                                    assignment_error(
                                        "configured RDII unit hydrograph is unavailable",
                                    )
                                })?;
                            Ok((node, unit_hydrograph, assignment.area))
                        })
                        .collect()
                };
                read().map_err(|error| DetachedFailure::Solver("rdii_assignments", error))
            })?;
        let values = assignments
            .into_iter()
            .map(|(node, unit_hydrograph, area)| {
                let node = relationship_view(py, facade, generation, ObjectType::Node, node)?;
                let unit_hydrograph = relationship_view(
                    py,
                    facade,
                    generation,
                    ObjectType::Unithyd,
                    unit_hydrograph,
                )?;
                (node, unit_hydrograph, area).into_py_any(py)
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, values)?.unbind().into_any())
    }

    /// Atomically replaces all RTK RDII assignments.
    fn replace_rdii_assignments(
        &self,
        py: Python<'_>,
        generation: u64,
        assignments: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let assignments = extract_rdii_assignments(py, self, generation, assignments)?;
        self.detached_checked_generation(
            py,
            "replace_rdii_assignments",
            generation,
            move |handle| {
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
                            &assignment.unit_hydrograph,
                            &[ObjectType::Unithyd],
                            "unit_hydrograph",
                        )?;
                        Ok(RdiiAssignmentConfiguration {
                            node_index: assignment.node.index,
                            unit_hydrograph_index: assignment.unit_hydrograph.index,
                            area: assignment.area,
                        })
                    })
                    .collect::<Result<Vec<_>, SwmmError>>()?;
                handle.simulation.replace_rdii_assignments(assignments)
            },
        )
    }
}

struct UnitHydrographUpdate {
    patch: UnitHydrographPatch,
    rain_gage: Option<RelatedObject>,
}

struct RdiiAssignmentUpdate {
    node: RelatedObject,
    unit_hydrograph: RelatedObject,
    area: f64,
}

fn responses_to_python(
    py: Python<'_>,
    monthly_responses: [[UnitHydrographResponseConfiguration; 3]; 12],
) -> PyResult<Py<PyAny>> {
    let months = monthly_responses
        .into_iter()
        .map(|month| {
            let responses = month
                .into_iter()
                .map(|response| {
                    (
                        response.rainfall_fraction,
                        response.time_to_peak_hours,
                        response.recession_ratio,
                        response.maximum_initial_abstraction,
                        response.initial_abstraction_recovery_rate,
                        response.initial_abstraction_at_start,
                    )
                        .into_py_any(py)
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyTuple::new(py, responses)?.unbind().into_any())
        })
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, months)?.unbind().into_any())
}

fn extract_unit_hydrograph_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<UnitHydrographUpdate> {
    let mut patch = UnitHydrographPatch::default();
    let mut rain_gage = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| rtk_py_error("unit-hydrograph update fields must be strings"))?;
        match name.as_str() {
            "rain_gage" => {
                rain_gage = related_live_view(
                    py,
                    owner,
                    generation,
                    &value,
                    "rain_gage",
                    "update_unit_hydrograph",
                    false,
                )?;
            }
            "monthly_responses" => {
                patch.monthly_responses = Some(extract_monthly_responses(&value)?);
            }
            _ => {
                return Err(rtk_py_error(&format!(
                    "unknown unit-hydrograph field: {name}"
                )));
            }
        }
    }
    Ok(UnitHydrographUpdate { patch, rain_gage })
}

fn extract_monthly_responses(
    value: &Bound<'_, PyAny>,
) -> PyResult<[[UnitHydrographResponseConfiguration; 3]; 12]> {
    let months = value
        .try_iter()
        .map_err(|_| rtk_py_error("monthly_responses must be iterable"))?
        .map(|month| extract_response_month(&month?))
        .collect::<PyResult<Vec<_>>>()?;
    months
        .try_into()
        .map_err(|_| rtk_py_error("monthly_responses must contain 12 months"))
}

fn extract_response_month(
    value: &Bound<'_, PyAny>,
) -> PyResult<[UnitHydrographResponseConfiguration; 3]> {
    let responses = value
        .try_iter()
        .map_err(|_| rtk_py_error("each month must be iterable"))?
        .map(|response| extract_response(&response?))
        .collect::<PyResult<Vec<_>>>()?;
    responses
        .try_into()
        .map_err(|_| rtk_py_error("each month must contain short, medium, and long responses"))
}

fn extract_response(value: &Bound<'_, PyAny>) -> PyResult<UnitHydrographResponseConfiguration> {
    let tuple = value
        .cast::<PyTuple>()
        .map_err(|_| rtk_py_error("each response must use the native tuple format"))?;
    if tuple.len() != 6 {
        return Err(rtk_py_error("each response must contain six parameters"));
    }
    let number = |index: usize, name: &str| -> PyResult<f64> {
        tuple
            .get_item(index)
            .and_then(|value| rtk_float(&value, name))
    };
    Ok(UnitHydrographResponseConfiguration {
        rainfall_fraction: number(0, "rainfall_fraction")?,
        time_to_peak_hours: number(1, "time_to_peak_hours")?,
        recession_ratio: number(2, "recession_ratio")?,
        maximum_initial_abstraction: number(3, "maximum_initial_abstraction")?,
        initial_abstraction_recovery_rate: number(4, "initial_abstraction_recovery_rate")?,
        initial_abstraction_at_start: number(5, "initial_abstraction_at_start")?,
    })
}

fn extract_rdii_assignments(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<RdiiAssignmentUpdate>> {
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
                    "each assignment must contain node, unit_hydrograph, and area",
                ));
            }
            let node = related_live_view(
                py,
                owner,
                generation,
                &tuple.get_item(0)?,
                "node",
                "replace_rdii_assignments",
                false,
            )?
            .ok_or_else(|| assignment_py_error("node must be a Live View"))?;
            let unit_hydrograph = related_live_view(
                py,
                owner,
                generation,
                &tuple.get_item(1)?,
                "unit_hydrograph",
                "replace_rdii_assignments",
                false,
            )?
            .ok_or_else(|| assignment_py_error("unit_hydrograph must be a Live View"))?;
            let area = assignment_float(&tuple.get_item(2)?)?;
            Ok(RdiiAssignmentUpdate {
                node,
                unit_hydrograph,
                area,
            })
        })
        .collect()
}

fn rtk_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(rtk_py_error(&format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| rtk_py_error(&format!("{name} must be numeric")))
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
        "replace_rdii_assignments",
        assignment_error(detail.to_owned()),
    )
}

fn rtk_py_error(detail: &str) -> PyErr {
    native_error(
        "update_unit_hydrograph",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}
