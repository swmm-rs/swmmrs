use super::*;
use pyo3::exceptions::PyTypeError;

#[pymethods]
impl NativeSimulation {
    /// Copies one selected precipitation value for a validated rain-gage Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured rain-gage index.
    /// * `id` - Captured canonical rain-gage ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private precipitation field name.
    ///
    /// # Returns
    /// Selected precipitation value in configured project rainfall units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, invalid-field, mutex, or panic failure.
    fn rain_gage_precipitation_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<f64> {
        let values = self.checked_view(
            py,
            "rain_gage_precipitation_value",
            generation,
            ObjectType::Gage,
            index,
            id,
            subtype,
            |simulation| simulation.rain_gage_precipitation_read(index),
        )?;
        match field {
            "total_precip" => Ok(values.total_precip),
            "rainfall" => Ok(values.rainfall),
            "snowfall" => Ok(values.snowfall),
            _ => Err(internal_error(
                "rain_gage_precipitation_value",
                "invalid private rain-gage precipitation field",
            )),
        }
    }

    /// Copies the persistent rainfall override for one validated rain-gage Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured rain-gage index.
    /// * `id` - Captured canonical rain-gage ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    ///
    /// # Returns
    /// Override in configured project rainfall units, or `None` when cleared.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, mutex, or panic failure.
    fn rain_gage_rainfall_override(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Option<f64>> {
        self.checked_view(
            py,
            "rain_gage_rainfall_override",
            generation,
            ObjectType::Gage,
            index,
            id,
            subtype,
            |simulation| simulation.rain_gage_rainfall_override_read(index),
        )
    }

    /// Copies the persistent external precipitation rate for one validated rain-gage Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured rain-gage index.
    /// * `id` - Captured canonical rain-gage ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    ///
    /// # Returns
    /// External precipitation rate in project rainfall units, or `None` when not selected.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, mutex, or panic failure.
    fn rain_gage_external_precipitation_rate(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Option<f64>> {
        self.checked_view(
            py,
            "rain_gage_external_precipitation_rate",
            generation,
            ObjectType::Gage,
            index,
            id,
            subtype,
            |simulation| simulation.rain_gage_external_precipitation_rate_read(index),
        )
    }

    /// Sets persistent external precipitation and makes the API this rain gage's source.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured rain-gage index.
    /// * `id` - Captured canonical rain-gage ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `value` - Candidate precipitation rate in project rainfall units.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, mutex, or panic failure.
    ///
    /// # Behavior
    /// Selects `RainApi`, unlinks any co-gage, and persists until replaced or closed.
    fn use_rain_gage_external_precipitation(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "use_rain_gage_external_precipitation";
        let value = extract_strict_number(value, "external_precipitation_rate", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Gage,
            index,
            id,
            subtype,
            move |simulation| simulation.set_rain_gage_external_precipitation_rate(index, value),
        )
    }

    /// Sets or clears the persistent highest-priority rainfall override.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured rain-gage index.
    /// * `id` - Captured canonical rain-gage ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `value` - Candidate rainfall rate in project units, or `None` to clear.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, mutex, or panic failure.
    ///
    /// # Behavior
    /// Leaves the configured source unchanged; `None` resumes it after clearing the override.
    fn set_rain_gage_rainfall_override(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_rain_gage_rainfall_override";
        let value = if value.is_none() {
            None
        } else {
            Some(extract_strict_number(
                value,
                "rainfall_override",
                operation,
            )?)
        };
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Gage,
            index,
            id,
            subtype,
            move |simulation| simulation.set_rain_gage_rainfall_override(index, value),
        )
    }

    /// Copies one selected precipitation scale factor for a validated subcatchment Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private precipitation scale-factor name.
    ///
    /// # Returns
    /// Selected precipitation scale factor.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, invalid-field, mutex, or panic failure.
    fn subcatchment_precipitation_scale_factor(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<f64> {
        let field = match field {
            "rain_scale_factor" => SubcatchmentConfigurationField::RainScaleFactor,
            "snow_scale_factor" => SubcatchmentConfigurationField::SnowScaleFactor,
            _ => {
                return Err(internal_error(
                    "subcatchment_precipitation_scale_factor",
                    "invalid private subcatchment forcing field",
                ));
            }
        };
        match self.checked_view(
            py,
            "subcatchment_precipitation_scale_factor",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_configuration_value_read(index, field),
        )? {
            SubcatchmentConfigurationValue::Number(value) => Ok(value),
            _ => Err(internal_error(
                "subcatchment_precipitation_scale_factor",
                "private forcing selector returned wrong value kind",
            )),
        }
    }

    /// Copies one selected live root subcatchment setting.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private stable root field name.
    ///
    /// # Returns
    /// Selected scalar, text, flag, relationship discriminator, or final public Live View.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, conversion, mutex, or panic failure.
    #[allow(clippy::too_many_arguments)]
    fn subcatchment_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let outlet_kind = field == "outlet_kind";
        let field = match field {
            "tag" => SubcatchmentConfigurationField::Tag,
            "area" => SubcatchmentConfigurationField::Area,
            "rain_gage" => SubcatchmentConfigurationField::RainGage,
            "included_in_report" => SubcatchmentConfigurationField::IncludedInReport,
            "width" => SubcatchmentConfigurationField::Width,
            "slope" => SubcatchmentConfigurationField::Slope,
            "curb_length" => SubcatchmentConfigurationField::CurbLength,
            "impervious_fraction" => SubcatchmentConfigurationField::ImperviousFraction,
            "zero_impervious_fraction" => SubcatchmentConfigurationField::ZeroImperviousFraction,
            "impervious_roughness" => SubcatchmentConfigurationField::ImperviousRoughness,
            "pervious_roughness" => SubcatchmentConfigurationField::PerviousRoughness,
            "impervious_depression_storage" => {
                SubcatchmentConfigurationField::ImperviousDepressionStorage
            }
            "pervious_depression_storage" => {
                SubcatchmentConfigurationField::PerviousDepressionStorage
            }
            "outlet" | "outlet_kind" => SubcatchmentConfigurationField::Outlet,
            _ => {
                return Err(internal_error(
                    "subcatchment_configuration_value",
                    "invalid private subcatchment configuration field",
                ));
            }
        };
        if outlet_kind {
            let value = self.checked_view(
                py,
                "subcatchment_configuration_value",
                generation,
                ObjectType::Subcatch,
                index,
                id,
                subtype,
                |simulation| simulation.subcatchment_configuration_value_read(index, field),
            )?;
            return match value {
                SubcatchmentConfigurationValue::Outlet(SubcatchmentOutletRead::Node(_)) => {
                    "node".into_py_any(py)
                }
                SubcatchmentConfigurationValue::Outlet(SubcatchmentOutletRead::Subcatchment(_)) => {
                    "subcatchment".into_py_any(py)
                }
                _ => Err(internal_error(
                    "subcatchment_configuration_value",
                    "outlet discriminator selector returned wrong value kind",
                )),
            };
        }
        if matches!(
            field,
            SubcatchmentConfigurationField::RainGage | SubcatchmentConfigurationField::Outlet
        ) {
            let (object_type, identity) = self.checked_view(
                py,
                "subcatchment_configuration_value",
                generation,
                ObjectType::Subcatch,
                index,
                id,
                subtype,
                |simulation| match simulation.subcatchment_configuration_value_read(index, field)? {
                    SubcatchmentConfigurationValue::RainGage(index) => Ok((
                        ObjectType::Gage,
                        simulation
                            .object_identity_at_read(ObjectType::Gage, index)?
                            .ok_or_else(|| {
                                SwmmError::with_detail(
                                    ErrorCode::System,
                                    "rain gage identity is unavailable",
                                )
                            })?,
                    )),
                    SubcatchmentConfigurationValue::Outlet(SubcatchmentOutletRead::Node(index)) => {
                        Ok((
                            ObjectType::Node,
                            simulation
                                .object_identity_at_read(ObjectType::Node, index)?
                                .ok_or_else(|| {
                                    SwmmError::with_detail(
                                        ErrorCode::System,
                                        "outlet node identity is unavailable",
                                    )
                                })?,
                        ))
                    }
                    SubcatchmentConfigurationValue::Outlet(
                        SubcatchmentOutletRead::Subcatchment(index),
                    ) => Ok((
                        ObjectType::Subcatch,
                        simulation
                            .object_identity_at_read(ObjectType::Subcatch, index)?
                            .ok_or_else(|| {
                                SwmmError::with_detail(
                                    ErrorCode::System,
                                    "outlet subcatchment identity is unavailable",
                                )
                            })?,
                    )),
                    _ => Err(SwmmError::with_detail(
                        ErrorCode::System,
                        "subcatchment relationship selector returned wrong value kind",
                    )),
                },
            )?;
            return relationship_view(py, facade, generation, object_type, identity);
        }
        let value = self.checked_view(
            py,
            "subcatchment_configuration_value",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_configuration_value_read(index, field),
        )?;
        match value {
            SubcatchmentConfigurationValue::Text(value) => value.into_py_any(py),
            SubcatchmentConfigurationValue::Number(value) => value.into_py_any(py),
            SubcatchmentConfigurationValue::Flag(value) => value.into_py_any(py),
            SubcatchmentConfigurationValue::RainGage(_)
            | SubcatchmentConfigurationValue::Outlet(_) => Err(internal_error(
                "subcatchment_configuration_value",
                "relationship escaped native relationship marshalling",
            )),
        }
    }

    fn subcatchment_infiltration_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        expected_kind: Option<&str>,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let value = self.checked_view(
            py,
            "subcatchment_infiltration_value",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_infiltration_configuration_read(index),
        )?;
        let Some(value) = value else {
            if expected_kind.is_none() && field == "kind" {
                return Ok(py.None());
            }
            return Err(stale_error("subcatchment_infiltration_value"));
        };
        let kind = infiltration_kind(value);
        if expected_kind.is_some_and(|expected| expected != kind) {
            return Err(stale_error("subcatchment_infiltration_value"));
        }
        infiltration_configuration_value(py, value, field)
    }

    /// Copies one selected groundwater configuration value or relationship.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private groundwater field selector.
    ///
    /// # Returns
    /// Selected Python-owned scalar, flag, or final public relationship view.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, conversion, mutex, panic, or invalid-selector failure.
    #[allow(clippy::too_many_arguments)]
    fn subcatchment_groundwater_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        if matches!(field, "aquifer" | "node") {
            let (object_type, identity) = self.checked_view(
                py,
                "subcatchment_groundwater_value",
                generation,
                ObjectType::Subcatch,
                index,
                id,
                subtype,
                |simulation| {
                    let SubcatchmentGroundwaterConfigurationRead::Present {
                        aquifer_index,
                        node_index,
                        ..
                    } = simulation.subcatchment_groundwater_configuration_read(index)?
                    else {
                        return Err(SwmmError::with_detail(
                            ErrorCode::ApiPropertyValue,
                            "groundwater configuration is absent",
                        ));
                    };
                    let (object_type, related_index) = if field == "aquifer" {
                        (ObjectType::Aquifer, aquifer_index)
                    } else {
                        (ObjectType::Node, node_index)
                    };
                    let identity = simulation
                        .object_identity_at_read(object_type, related_index)?
                        .ok_or_else(|| {
                            SwmmError::with_detail(
                                ErrorCode::System,
                                "groundwater relationship identity is unavailable",
                            )
                        })?;
                    Ok((object_type, identity))
                },
            )?;
            return relationship_view(py, facade, generation, object_type, identity);
        }
        let value = self.checked_view(
            py,
            "subcatchment_groundwater_value",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_groundwater_configuration_read(index),
        )?;
        groundwater_configuration_value(py, value, field)
    }

    /// Copies the optional snowmelt-parameter-set relationship for one subcatchment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for the relationship result.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    ///
    /// # Returns
    /// Fresh final public snowmelt Live View, or `None` when no snowpack is configured.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, inconsistent-storage, mutex, panic, or constructor failure.
    fn subcatchment_snowpack(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Option<Py<PyAny>>> {
        let identity = self.checked_view(
            py,
            "subcatchment_snowpack",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| {
                let Some(value) = simulation.subcatchment_snowpack_component_read(index)? else {
                    return Ok(None);
                };
                simulation
                    .object_identity_at_read(ObjectType::Snowmelt, value.snowmelt_index)?
                    .ok_or_else(|| {
                        SwmmError::with_detail(
                            ErrorCode::Exception,
                            "configured snowpack identity is unavailable",
                        )
                    })
                    .map(Some)
            },
        )?;
        identity
            .map(|identity| {
                relationship_view(py, facade, generation, ObjectType::Snowmelt, identity)
            })
            .transpose()
    }

    fn subcatchment_named_settings_mapping(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        mapping: &str,
    ) -> PyResult<Vec<(String, f64)>> {
        self.checked_view(
            py,
            "subcatchment_named_settings_mapping",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| {
                let (object_type, mapping) = match mapping {
                    "loading" => (ObjectType::Pollut, SubcatchmentNamedSettings::Loading),
                    "coverage" => (ObjectType::Landuse, SubcatchmentNamedSettings::Coverage),
                    _ => {
                        return Err(SwmmError::with_detail(
                            ErrorCode::ApiPropertyValue,
                            "invalid private named settings mapping",
                        ));
                    }
                };
                let values = simulation.subcatchment_named_settings_read(index, mapping)?;
                let identities = simulation.object_identity_indexes_read(object_type)?;
                Ok(identities
                    .into_iter()
                    .zip(values)
                    .map(|(identity, value)| (identity.id, value))
                    .collect())
            },
        )
    }

    /// Reads one named subcatchment setting through native canonical lookup.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `mapping` - Private named settings family.
    /// * `key` - Pollutant or land-use identity supplied by Python.
    ///
    /// # Returns
    /// Current configured value for the canonical identity.
    ///
    /// # Errors
    /// Returns type, unknown-name, stale, lifecycle, mutex, or owner-read errors.
    fn subcatchment_named_setting_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        mapping: &str,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<f64> {
        let key = key
            .extract::<String>()
            .map_err(|_| PyTypeError::new_err("subcatchment settings mapping keys must be str"))?;
        self.checked_view(
            py,
            "subcatchment_named_setting_value",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                let (object_type, mapping) = match mapping {
                    "loading" => (ObjectType::Pollut, SubcatchmentNamedSettings::Loading),
                    "coverage" => (ObjectType::Landuse, SubcatchmentNamedSettings::Coverage),
                    _ => {
                        return Err(SwmmError::with_detail(
                            ErrorCode::ApiPropertyValue,
                            "invalid private named settings mapping",
                        ));
                    }
                };
                let identities = simulation.object_identity_indexes_read(object_type)?;
                let values = simulation.subcatchment_named_settings_read(index, mapping)?;
                identities
                    .into_iter()
                    .zip(values)
                    .find_map(|(identity, value)| {
                        identity.id.eq_ignore_ascii_case(&key).then_some(value)
                    })
                    .ok_or_else(|| SwmmError::with_detail(ErrorCode::ApiObjectName, key))
            },
        )
    }

    /// Copies one selected external precipitation value for a validated subcatchment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private external-forcing field name.
    ///
    /// # Returns
    /// Selected external precipitation value in configured project rainfall units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, invalid-field, mutex, or panic failure.
    fn subcatchment_external_forcing_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<f64> {
        let values = self.checked_view(
            py,
            "subcatchment_external_forcing_value",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_external_forcing_read(index),
        )?;
        match field {
            "rainfall" => Ok(values.rainfall),
            "snowfall" => Ok(values.snowfall),
            _ => Err(internal_error(
                "subcatchment_external_forcing_value",
                "invalid private subcatchment forcing field",
            )),
        }
    }

    /// Copies one selected current runoff result for a validated subcatchment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private result field name.
    ///
    /// # Returns
    /// Selected current subcatchment result in project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, invalid-field, mutex, or panic failure.
    fn subcatchment_result(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<f64> {
        self.checked_view(
            py,
            "subcatchment_result",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| {
                let field = match field {
                    "rainfall" => SubcatchmentResultField::Rainfall,
                    "evaporation" => SubcatchmentResultField::Evaporation,
                    "infiltration" => SubcatchmentResultField::Infiltration,
                    "runon" => SubcatchmentResultField::Runon,
                    "runoff" => SubcatchmentResultField::Runoff,
                    "snow_depth" => SubcatchmentResultField::SnowDepth,
                    _ => {
                        // Preserve lifecycle-error precedence for malformed private calls.
                        simulation.subcatchment_results_read(index)?;
                        return Err(SwmmError::with_detail(
                            ErrorCode::System,
                            "invalid private subcatchment result field",
                        ));
                    }
                };
                simulation.subcatchment_result_value_read(index, field)
            },
        )
    }

    /// Copies one fixed-shape subcatchment hydraulic snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native subcatchment snapshot in configured project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn subcatchment_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<SubcatchmentSnapshot>> {
        let ids = snapshot_ids(ids, "subcatchment_snapshot")?;
        let values = self.detached_checked_generation(
            py,
            "subcatchment_snapshot",
            generation,
            move |handle| handle.simulation.subcatchment_snapshot_read(ids.as_deref()),
        )?;
        Py::new(py, SubcatchmentSnapshot::new(values))
    }
    /// Copies one selected pollutant mapping for a validated subcatchment view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured subcatchment subtype label.
    /// * `field` - Private subcatchment-quality field name.
    ///
    /// # Returns
    /// Owned pollutant IDs and the selected quality values in configured units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, configured-storage, invalid-field, mutex, or panic failure.
    fn subcatchment_quality_mapping(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<PollutantMappingParts> {
        let values = self.checked_view(
            py,
            "subcatchment_quality_mapping",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_quality_read(index),
        )?;
        subcatchment_quality_mapping(values, field)
    }

    /// Copies persistent external buildup increments for one validated subcatchment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured subcatchment subtype label.
    ///
    /// # Returns
    /// Owned pollutant IDs and persistent buildup increments in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale view, invalid private subtype, poisoned owner mutex,
    /// lifecycle, index, forcing-storage, or caught-panic failure.
    fn subcatchment_external_pollutant_buildup_increment(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<PollutantMappingParts> {
        self.checked_view(
            py,
            "subcatchment_external_pollutant_buildup_increment",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| simulation.subcatchment_external_pollutant_buildup_increment_read(index),
        )
        .map(pollutant_mapping_tuple)
    }

    /// Copies one pollutant-major subcatchment snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native subcatchment-quality snapshot in configured reporting units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale generation, poisoned owner mutex, lifecycle,
    /// duplicate-ID, unknown-ID, configured-storage, or caught-panic failure.
    fn subcatchment_quality_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<SubcatchmentQualitySnapshot>> {
        let ids = snapshot_ids(ids, "subcatchment_quality_snapshot")?;
        let values = self.detached_checked_generation(
            py,
            "subcatchment_quality_snapshot",
            generation,
            move |handle| {
                handle
                    .simulation
                    .subcatchment_quality_snapshot_read(ids.as_deref())
            },
        )?;
        Py::new(py, SubcatchmentQualitySnapshot::new(values))
    }

    /// Reads one persistent subcatchment buildup increment by pollutant identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured subcatchment subtype label.
    /// * `pollutant_id` - Canonical or case-insensitive pollutant identity.
    ///
    /// # Returns
    /// Persistent buildup increment in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, unknown pollutant,
    /// invalid private subtype, inconsistent storage, poisoned owner mutex, or caught panic.
    fn subcatchment_external_pollutant_buildup_increment_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        pollutant_id: &Bound<'_, PyAny>,
    ) -> PyResult<f64> {
        let operation = "subcatchment_external_pollutant_buildup_increment_value";
        let pollutant_id = extract_pollutant_id(pollutant_id, operation)?;
        self.checked_view(
            py,
            operation,
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            |simulation| {
                simulation.subcatchment_external_pollutant_buildup_increment_value_read(
                    index,
                    &pollutant_id,
                )
            },
        )
    }

    /// Atomically mutates persistent subcatchment pollutant buildup increments.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured subcatchment subtype label.
    /// * `replace` - Whether omitted configured pollutants reset to zero.
    /// * `args` - Positional mapping-update arguments.
    /// * `kwargs` - Keyword mapping-update values.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, duplicate or unknown
    /// pollutants, invalid values, disallowed lifecycle, inconsistent storage, poisoned owner
    /// mutex, or caught panic.
    #[allow(clippy::too_many_arguments)]
    fn update_subcatchment_external_pollutant_buildup_increment(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        replace: bool,
        args: &Bound<'_, PyTuple>,
        kwargs: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let operation = "update_subcatchment_external_pollutant_buildup_increment";
        let patch = PersistentPollutantValuesPatch {
            values: extract_pollutant_update(
                args,
                kwargs,
                "external_pollutant_buildup_increment",
                operation,
            )?,
            replace,
        };
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                simulation.patch_subcatchment_external_pollutant_buildup_increment(index, patch)
            },
        )
    }

    /// Copies one subcatchment LID-group snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    ///
    /// # Returns
    /// Immutable native LID-group snapshot in configured project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, absent-group, mutex, or panic failure.
    fn subcatchment_lid_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
    ) -> PyResult<Py<SubcatchmentLidSnapshot>> {
        let values = self.detached_checked_view(
            py,
            "subcatchment_lid_snapshot",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| simulation.subcatchment_lid_snapshot_read(index),
        )?;
        Py::new(py, SubcatchmentLidSnapshot::new(values))
    }

    /// Applies one live subcatchment settings mutation under the owner lock.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary subtype label.
    /// * `changes` - Supplied stable root, component-installation, relationship, and mapping values.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, extraction, mutex, or panic failure.
    fn update_subcatchment(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_subcatchment_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_subcatchment",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                let patch = update.into_patch(simulation)?;
                simulation.patch_subcatchment(index, patch)
            },
        )
    }

    fn update_subcatchment_named_settings(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        mapping: &str,
        replace: bool,
        values: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let values = named_mapping_values(values, mapping)?;
        let object_type = match mapping {
            "loading" => ObjectType::Pollut,
            "coverage" => ObjectType::Landuse,
            _ => {
                return Err(subcatchment_py_error(
                    "invalid private named settings mapping",
                ));
            }
        };
        self.detached_checked_view_result(
            py,
            "update_subcatchment_named_settings",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                if values.is_empty() && !replace {
                    return simulation
                        .patch_subcatchment(index, SubcatchmentPatch::default())
                        .map_err(|error| {
                            DetachedFailure::Solver("update_subcatchment_named_settings", error)
                        });
                }
                let values = resolve_named_values(simulation, object_type, values, replace)
                    .map_err(|error| {
                        DetachedFailure::Solver("update_subcatchment_named_settings", error)
                    })?;
                let patch = if object_type == ObjectType::Pollut {
                    SubcatchmentPatch {
                        loading: Some(SubcatchmentLoadingPatch {
                            initial_buildup: values,
                        }),
                        ..SubcatchmentPatch::default()
                    }
                } else {
                    SubcatchmentPatch {
                        coverage: Some(SubcatchmentCoveragePatch { fractions: values }),
                        ..SubcatchmentPatch::default()
                    }
                };
                simulation
                    .patch_subcatchment(index, patch)
                    .map_err(|error| {
                        DetachedFailure::Solver("update_subcatchment_named_settings", error)
                    })
            },
        )
    }

    fn update_subcatchment_infiltration(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        expected_kind: &str,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_infiltration_update(expected_kind, changes)?;
        self.detached_checked_view_result(
            py,
            "update_subcatchment_infiltration",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                let patch =
                    update
                        .into_patch(simulation, index)
                        .map_err(|failure| match failure {
                            SettingsMutationFailure::Stale => DetachedFailure::Stale,
                            SettingsMutationFailure::Solver(error) => {
                                DetachedFailure::Solver("update_subcatchment_infiltration", error)
                            }
                        })?;
                simulation
                    .patch_subcatchment(index, patch)
                    .map_err(|error| {
                        DetachedFailure::Solver("update_subcatchment_infiltration", error)
                    })
            },
        )
    }

    fn update_subcatchment_groundwater(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_groundwater_update(py, self, generation, changes)?;
        self.detached_checked_view_result(
            py,
            "update_subcatchment_groundwater",
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                let patch =
                    update
                        .into_patch(simulation, index)
                        .map_err(|failure| match failure {
                            SettingsMutationFailure::Stale => DetachedFailure::Stale,
                            SettingsMutationFailure::Solver(error) => {
                                DetachedFailure::Solver("update_subcatchment_groundwater", error)
                            }
                        })?;
                simulation
                    .patch_subcatchment(index, patch)
                    .map_err(|error| {
                        DetachedFailure::Solver("update_subcatchment_groundwater", error)
                    })
            },
        )
    }

    /// Sets persistent subcatchment rainfall and snowfall scale factors.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `rainfall` - Positive gage-rainfall multiplier.
    /// * `snowfall` - Positive gage-snowfall multiplier.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, mutex, or panic failure.
    fn set_subcatchment_precipitation_scale_factors(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        rainfall: &Bound<'_, PyAny>,
        snowfall: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_subcatchment_precipitation_scale_factors";
        let rainfall = extract_strict_number(rainfall, "rain_scale_factor", operation)?;
        let snowfall = extract_strict_number(snowfall, "snow_scale_factor", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| {
                simulation.set_subcatchment_precipitation_scale_factors(
                    index,
                    SubcatchmentPrecipitationScaleFactors { rainfall, snowfall },
                )
            },
        )
    }

    /// Sets persistent external rainfall through the typed owner operation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `value` - Candidate rainfall rate in project units.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, mutex, or panic failure.
    fn set_subcatchment_external_rainfall(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_subcatchment_external_rainfall";
        let value = extract_strict_number(value, "external_rainfall", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| simulation.set_subcatchment_external_rainfall(index, value),
        )
    }

    /// Sets persistent external snowfall through the typed owner operation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured subcatchment index.
    /// * `id` - Captured canonical subcatchment ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `value` - Candidate snowfall rate in project units.
    ///
    /// # Errors
    /// Returns a stale, validation, lifecycle, mutex, or panic failure.
    fn set_subcatchment_external_snowfall(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_subcatchment_external_snowfall";
        let value = extract_strict_number(value, "external_snowfall", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Subcatch,
            index,
            id,
            subtype,
            move |simulation| simulation.set_subcatchment_external_snowfall(index, value),
        )
    }

    /// Reads one current aquifer configuration value after validating its definition view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured aquifer index.
    /// * `id` - Captured canonical aquifer identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private aquifer field selector.
    ///
    /// # Returns
    /// Python-owned scalar aquifer configuration value in project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, conversion, mutex, panic, or invalid-selector failure.
    #[allow(clippy::too_many_arguments)]
    fn aquifer_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = match field {
            "porosity" => AquiferConfigurationField::Porosity,
            "wilting_point" => AquiferConfigurationField::WiltingPoint,
            "field_capacity" => AquiferConfigurationField::FieldCapacity,
            "hydraulic_conductivity" => AquiferConfigurationField::HydraulicConductivity,
            "conductivity_slope" => AquiferConfigurationField::ConductivitySlope,
            "tension_slope" => AquiferConfigurationField::TensionSlope,
            "upper_evaporation_fraction" => AquiferConfigurationField::UpperEvaporationFraction,
            "lower_evaporation_depth" => AquiferConfigurationField::LowerEvaporationDepth,
            "lower_loss_coefficient" => AquiferConfigurationField::LowerLossCoefficient,
            "bottom_elevation" => AquiferConfigurationField::BottomElevation,
            "water_table_elevation" => AquiferConfigurationField::WaterTableElevation,
            "upper_moisture" => AquiferConfigurationField::UpperMoisture,
            "upper_evaporation_pattern" => AquiferConfigurationField::UpperEvaporationPattern,
            _ => {
                return Err(internal_error(
                    "aquifer_configuration_value",
                    "invalid private aquifer configuration field",
                ));
            }
        };
        if field == AquiferConfigurationField::UpperEvaporationPattern {
            let identity = self.checked_view(
                py,
                "aquifer_configuration_value",
                generation,
                ObjectType::Aquifer,
                index,
                id,
                subtype,
                move |simulation| {
                    let AquiferConfigurationValue::UpperEvaporationPattern(index) =
                        simulation.aquifer_configuration_value_read(index, field)?
                    else {
                        return Err(SwmmError::with_detail(
                            ErrorCode::System,
                            "aquifer relationship selector returned wrong value kind",
                        ));
                    };
                    let Some(index) = index else {
                        return Ok(None);
                    };
                    simulation
                        .object_identity_at_read(ObjectType::Timepattern, index)?
                        .ok_or_else(|| {
                            SwmmError::with_detail(
                                ErrorCode::Exception,
                                "configured evaporation-pattern identity is unavailable",
                            )
                        })
                        .map(Some)
                },
            )?;
            return identity
                .map(|identity| {
                    relationship_view(py, facade, generation, ObjectType::Timepattern, identity)
                })
                .transpose()?
                .into_py_any(py);
        }
        let value = self.checked_view(
            py,
            "aquifer_configuration_value",
            generation,
            ObjectType::Aquifer,
            index,
            id,
            subtype,
            move |simulation| simulation.aquifer_configuration_value_read(index, field),
        )?;
        match value {
            AquiferConfigurationValue::Number(value) => value.into_py_any(py),
            AquiferConfigurationValue::UpperEvaporationPattern(_) => Err(internal_error(
                "aquifer_configuration_value",
                "relationship escaped native relationship marshalling",
            )),
        }
    }

    /// Copies one selected root snowmelt value after validating its definition view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured snowmelt parameter-set index.
    /// * `id` - Captured canonical snowmelt identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `field` - Private root snowmelt field name.
    ///
    /// # Returns
    /// Selected scalar, fraction tuple, or optional relationship index.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, conversion, mutex, or panic failure.
    #[allow(clippy::too_many_arguments)]
    fn snowmelt_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = match field {
            "plowable_fraction" => SnowmeltConfigurationField::PlowableFraction,
            "plow_depth" => SnowmeltConfigurationField::PlowDepth,
            "removal_fractions" => SnowmeltConfigurationField::RemovalFractions,
            "removal_subcatchment" => SnowmeltConfigurationField::RemovalSubcatchment,
            _ => {
                return Err(internal_error(
                    "snowmelt_configuration_value",
                    "invalid private snowmelt configuration field",
                ));
            }
        };
        if field == SnowmeltConfigurationField::RemovalSubcatchment {
            let identity = self.checked_view(
                py,
                "snowmelt_configuration_value",
                generation,
                ObjectType::Snowmelt,
                index,
                id,
                subtype,
                |simulation| {
                    let SnowmeltConfigurationValue::RemovalSubcatchment(index) =
                        simulation.snowmelt_configuration_value_read(index, field)?
                    else {
                        return Err(SwmmError::with_detail(
                            ErrorCode::System,
                            "snowmelt relationship selector returned wrong value kind",
                        ));
                    };
                    let Some(index) = index else {
                        return Ok(None);
                    };
                    simulation
                        .object_identity_at_read(ObjectType::Subcatch, index)?
                        .ok_or_else(|| {
                            SwmmError::with_detail(
                                ErrorCode::Exception,
                                "configured snowmelt removal identity is unavailable",
                            )
                        })
                        .map(Some)
                },
            )?;
            return identity
                .map(|identity| {
                    relationship_view(py, facade, generation, ObjectType::Subcatch, identity)
                })
                .transpose()?
                .into_py_any(py);
        }
        let value = self.checked_view(
            py,
            "snowmelt_configuration_value",
            generation,
            ObjectType::Snowmelt,
            index,
            id,
            subtype,
            |simulation| simulation.snowmelt_configuration_value_read(index, field),
        )?;
        match value {
            SnowmeltConfigurationValue::Number(value) => value.into_py_any(py),
            SnowmeltConfigurationValue::RemovalFractions(values) => {
                (values[0], values[1], values[2], values[3], values[4]).into_py_any(py)
            }
            SnowmeltConfigurationValue::RemovalSubcatchment(_) => Err(internal_error(
                "snowmelt_configuration_value",
                "relationship escaped native relationship marshalling",
            )),
        }
    }

    /// Copies one selected fixed-surface snowmelt value.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured snowmelt parameter-set index.
    /// * `id` - Captured canonical snowmelt identifier.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `surface` - Private fixed surface name.
    /// * `field` - Private surface field name.
    ///
    /// # Returns
    /// Selected surface value in project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, field, conversion, mutex, or panic failure.
    fn snowmelt_surface_configuration_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        surface: &str,
        field: &str,
    ) -> PyResult<f64> {
        let surface = snowmelt_surface(surface)?;
        let field = match field {
            "minimum_melt_coefficient" => SnowmeltSurfaceField::MinimumMeltCoefficient,
            "maximum_melt_coefficient" => SnowmeltSurfaceField::MaximumMeltCoefficient,
            "base_temperature" => SnowmeltSurfaceField::BaseTemperature,
            "free_water_fraction" => SnowmeltSurfaceField::FreeWaterFraction,
            "initial_snow_depth" => SnowmeltSurfaceField::InitialSnowDepth,
            "initial_free_water" => SnowmeltSurfaceField::InitialFreeWater,
            "snow_depth_for_full_coverage" => SnowmeltSurfaceField::SnowDepthForFullCoverage,
            _ => {
                return Err(internal_error(
                    "snowmelt_surface_configuration_value",
                    "invalid private snowmelt surface field",
                ));
            }
        };
        self.checked_view(
            py,
            "snowmelt_surface_configuration_value",
            generation,
            ObjectType::Snowmelt,
            index,
            id,
            subtype,
            |simulation| {
                simulation.snowmelt_surface_configuration_value_read(index, surface, field)
            },
        )
    }

    /// Applies one atomic aquifer update through a validated definition view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured aquifer index.
    /// * `id` - Captured canonical aquifer ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `changes` - Supplied aquifer field values keyed by public property name.
    ///
    /// # Errors
    /// Returns a transport, stale, lifecycle, owner-validation, mutex, or panic failure without partial mutation.
    fn update_aquifer(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_aquifer_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_aquifer",
            generation,
            ObjectType::Aquifer,
            index,
            id,
            subtype,
            move |simulation| {
                let patch = update.into_patch(simulation)?;
                simulation.patch_aquifer(index, patch)
            },
        )
    }

    /// Applies one atomic snowmelt update through a validated definition view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured snowmelt parameter-set index.
    /// * `id` - Captured canonical snowmelt parameter-set ID.
    /// * `subtype` - Captured ordinary-object subtype label.
    /// * `surface` - Optional private fixed-surface selector.
    /// * `changes` - Supplied root or selected-surface values by public property name.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, extraction, owner-validation, mutex, or panic failure.
    fn update_snowmelt(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        surface: Option<&str>,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_snowmelt_update(py, self, generation, surface, changes)?;
        self.detached_checked_view(
            py,
            "update_snowmelt",
            generation,
            ObjectType::Snowmelt,
            index,
            id,
            subtype,
            move |simulation| {
                let patch = update.into_patch(simulation, index)?;
                simulation.patch_snowmelt(index, patch)
            },
        )
    }
}

/// Creates one Python validation error for aquifer update transport.
///
/// # Arguments
/// * `detail` - Stable validation detail.
///
/// # Returns
/// Private native validation exception.
fn aquifer_py_error(detail: &str) -> PyErr {
    native_error(
        "update_aquifer",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}

/// Extracts one non-boolean aquifer floating-point value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Property name used in diagnostics.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn aquifer_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(aquifer_py_error(&format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| aquifer_py_error(&format!("{name} must be numeric")))
}

struct AquiferUpdate {
    patch: AquiferPatch,
    upper_evaporation_pattern: Option<Option<RelatedObject>>,
}

impl AquiferUpdate {
    /// Resolves relationships and builds the typed solver patch under the owner lock.
    ///
    /// # Arguments
    /// * `simulation` - Current validated Simulation Owner.
    ///
    /// # Returns
    /// Typed partial aquifer patch in project units.
    ///
    /// # Errors
    /// Returns a relationship identity or index conversion error without mutation.
    fn into_patch(mut self, simulation: &SwmmSimulation) -> Result<AquiferPatch, SwmmError> {
        if let Some(pattern) = self.upper_evaporation_pattern {
            self.patch.upper_evaporation_pattern_index = Some(if let Some(pattern) = pattern {
                validate_relationship(
                    simulation,
                    &pattern,
                    &[ObjectType::Timepattern],
                    "upper_evaporation_pattern",
                )?;
                Some(pattern.index)
            } else {
                None
            });
        }
        Ok(self.patch)
    }
}

/// Extracts one optional upper-evaporation pattern Live View.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Python `TimePattern` Live View or `None`.
///
/// # Returns
/// `None` to clear the relationship or an owned identity for owner-locked validation.
///
/// # Errors
/// Returns a validation error for malformed, foreign-owner, or foreign-generation values.
fn aquifer_pattern(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<RelatedObject>> {
    related_live_view(
        py,
        owner,
        generation,
        value,
        "upper_evaporation_pattern",
        "update_aquifer",
        true,
    )
}

/// Extracts one complete owned aquifer update from supplied keyword changes.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `changes` - Supplied aquifer values keyed by public property name.
///
/// # Returns
/// Owned aquifer update in project units with unresolved relationship identity.
///
/// # Errors
/// Returns a validation error for unknown fields, malformed values, or foreign relationships.
fn extract_aquifer_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<AquiferUpdate> {
    let mut patch = AquiferPatch::default();
    let mut upper_evaporation_pattern = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| aquifer_py_error("aquifer update fields must be strings"))?;
        match name.as_str() {
            "porosity" => patch.porosity = Some(aquifer_float(&value, &name)?),
            "wilting_point" => patch.wilting_point = Some(aquifer_float(&value, &name)?),
            "field_capacity" => patch.field_capacity = Some(aquifer_float(&value, &name)?),
            "hydraulic_conductivity" => {
                patch.hydraulic_conductivity = Some(aquifer_float(&value, &name)?)
            }
            "conductivity_slope" => patch.conductivity_slope = Some(aquifer_float(&value, &name)?),
            "tension_slope" => patch.tension_slope = Some(aquifer_float(&value, &name)?),
            "upper_evaporation_fraction" => {
                patch.upper_evaporation_fraction = Some(aquifer_float(&value, &name)?)
            }
            "lower_evaporation_depth" => {
                patch.lower_evaporation_depth = Some(aquifer_float(&value, &name)?)
            }
            "lower_loss_coefficient" => {
                patch.lower_loss_coefficient = Some(aquifer_float(&value, &name)?)
            }
            "bottom_elevation" => patch.bottom_elevation = Some(aquifer_float(&value, &name)?),
            "water_table_elevation" => {
                patch.water_table_elevation = Some(aquifer_float(&value, &name)?)
            }
            "upper_moisture" => patch.upper_moisture = Some(aquifer_float(&value, &name)?),
            "upper_evaporation_pattern" => {
                upper_evaporation_pattern = Some(aquifer_pattern(py, owner, generation, &value)?)
            }
            _ => return Err(aquifer_py_error(&format!("unknown aquifer field: {name}"))),
        }
    }
    Ok(AquiferUpdate {
        patch,
        upper_evaporation_pattern,
    })
}

/// Creates one Python validation error for snowmelt update transport.
///
/// # Arguments
/// * `detail` - Stable validation detail.
///
/// # Returns
/// Private native validation exception.
fn snowmelt_py_error(detail: &str) -> PyErr {
    native_error(
        "update_snowmelt",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}

/// Extracts one non-boolean snowmelt floating-point value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Property name used in diagnostics.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn snowmelt_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(snowmelt_py_error(&format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| snowmelt_py_error(&format!("{name} must be numeric")))
}

/// Resolves one private snowmelt surface name to its typed selector.
///
/// # Arguments
/// * `surface` - Private fixed-surface name.
///
/// # Returns
/// Typed fixed-surface selector.
///
/// # Errors
/// Returns an internal binding error for an invalid private name.
fn snowmelt_surface(surface: &str) -> PyResult<SnowmeltSurface> {
    match surface {
        "plowable" => Ok(SnowmeltSurface::Plowable),
        "impervious" => Ok(SnowmeltSurface::Impervious),
        "pervious" => Ok(SnowmeltSurface::Pervious),
        _ => Err(internal_error(
            "snowmelt_surface",
            "invalid private snowmelt surface",
        )),
    }
}

#[derive(Default)]
struct SnowmeltSurfaceUpdate {
    minimum_melt_coefficient: Option<f64>,
    maximum_melt_coefficient: Option<f64>,
    base_temperature: Option<f64>,
    free_water_fraction: Option<f64>,
    initial_snow_depth: Option<f64>,
    initial_free_water: Option<f64>,
    snow_depth_for_full_coverage: Option<f64>,
}

impl SnowmeltSurfaceUpdate {
    /// Reports whether no surface fields were supplied.
    ///
    /// # Returns
    /// `true` when applying this surface update is a no-op.
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

enum SnowmeltUpdate {
    Root {
        patch: SnowmeltPatch,
        removal_subcatchment: Option<Option<RelatedObject>>,
    },
    Surface {
        surface: SnowmeltSurface,
        changes: SnowmeltSurfaceUpdate,
    },
}

impl SnowmeltUpdate {
    /// Resolves relationships and builds the typed solver patch under the owner lock.
    ///
    /// # Arguments
    /// * `simulation` - Current validated Simulation Owner.
    /// * `snowmelt_index` - Zero-based configured snowmelt parameter-set index.
    ///
    /// # Returns
    /// Typed partial snowmelt patch in project units.
    ///
    /// # Errors
    /// Returns a targeted-read or relationship validation error without mutation.
    fn into_patch(
        self,
        simulation: &SwmmSimulation,
        snowmelt_index: usize,
    ) -> Result<SnowmeltPatch, SwmmError> {
        match self {
            Self::Root {
                mut patch,
                removal_subcatchment,
            } => {
                if let Some(relation) = removal_subcatchment {
                    patch.removal_subcatchment_index = Some(if let Some(relation) = relation {
                        validate_relationship(
                            simulation,
                            &relation,
                            &[ObjectType::Subcatch],
                            "removal_subcatchment",
                        )?;
                        Some(relation.index)
                    } else {
                        None
                    });
                }
                Ok(patch)
            }
            Self::Surface { surface, changes } => {
                if changes.is_empty() {
                    return Ok(SnowmeltPatch::default());
                }
                let read = |field| {
                    simulation.snowmelt_surface_configuration_value_read(
                        snowmelt_index,
                        surface,
                        field,
                    )
                };
                Ok(SnowmeltPatch {
                    surface: Some((
                        surface,
                        SnowmeltSurfaceConfiguration {
                            minimum_melt_coefficient: changes
                                .minimum_melt_coefficient
                                .map_or_else(
                                    || read(SnowmeltSurfaceField::MinimumMeltCoefficient),
                                    Ok,
                                )?,
                            maximum_melt_coefficient: changes
                                .maximum_melt_coefficient
                                .map_or_else(
                                    || read(SnowmeltSurfaceField::MaximumMeltCoefficient),
                                    Ok,
                                )?,
                            base_temperature: changes
                                .base_temperature
                                .map_or_else(|| read(SnowmeltSurfaceField::BaseTemperature), Ok)?,
                            free_water_fraction: changes.free_water_fraction.map_or_else(
                                || read(SnowmeltSurfaceField::FreeWaterFraction),
                                Ok,
                            )?,
                            initial_snow_depth: changes
                                .initial_snow_depth
                                .map_or_else(|| read(SnowmeltSurfaceField::InitialSnowDepth), Ok)?,
                            initial_free_water: changes
                                .initial_free_water
                                .map_or_else(|| read(SnowmeltSurfaceField::InitialFreeWater), Ok)?,
                            snow_depth_for_full_coverage: changes
                                .snow_depth_for_full_coverage
                                .map_or_else(
                                    || read(SnowmeltSurfaceField::SnowDepthForFullCoverage),
                                    Ok,
                                )?,
                        },
                    )),
                    ..SnowmeltPatch::default()
                })
            }
        }
    }
}

/// Extracts one optional snowmelt removal-subcatchment Live View.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Python `Subcatchment` Live View or `None`.
///
/// # Returns
/// `None` to clear the relationship or an owned identity for owner-locked validation.
///
/// # Errors
/// Returns a validation error for malformed, foreign-owner, or foreign-generation values.
fn snowmelt_removal_subcatchment(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<RelatedObject>> {
    related_live_view(
        py,
        owner,
        generation,
        value,
        "removal_subcatchment",
        "update_snowmelt",
        true,
    )
}

/// Extracts exactly five non-boolean snow-removal fractions.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Python sequence candidate.
///
/// # Returns
/// Fixed removal-fraction array in caller project units.
///
/// # Errors
/// Returns a validation error for the wrong shape or non-numeric values.
fn snowmelt_removal_fractions(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<[f64; 5]> {
    let values = value
        .extract::<Vec<Py<PyAny>>>()
        .map_err(|_| snowmelt_py_error("removal_fractions must contain five numeric values"))?;
    if values.len() != 5 {
        return Err(snowmelt_py_error(
            "removal_fractions must contain five numeric values",
        ));
    }
    let mut fractions = [0.0; 5];
    for (index, value) in values.iter().enumerate() {
        fractions[index] = snowmelt_float(value.bind(py), "removal_fractions")?;
    }
    Ok(fractions)
}

/// Extracts one complete owned snowmelt update from supplied keyword changes.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `surface` - Optional private fixed-surface name.
/// * `changes` - Supplied values keyed by public property name.
///
/// # Returns
/// Owned root or fixed-surface update in project units.
///
/// # Errors
/// Returns a validation error for unknown fields, malformed values, or foreign relationships.
fn extract_snowmelt_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    surface: Option<&str>,
    changes: &Bound<'_, PyDict>,
) -> PyResult<SnowmeltUpdate> {
    if let Some(surface) = surface {
        let mut update = SnowmeltSurfaceUpdate::default();
        for (name, value) in changes.iter() {
            let name = name
                .extract::<String>()
                .map_err(|_| snowmelt_py_error("snowmelt update fields must be strings"))?;
            match name.as_str() {
                "minimum_melt_coefficient" => {
                    update.minimum_melt_coefficient = Some(snowmelt_float(&value, &name)?)
                }
                "maximum_melt_coefficient" => {
                    update.maximum_melt_coefficient = Some(snowmelt_float(&value, &name)?)
                }
                "base_temperature" => {
                    update.base_temperature = Some(snowmelt_float(&value, &name)?)
                }
                "free_water_fraction" => {
                    update.free_water_fraction = Some(snowmelt_float(&value, &name)?)
                }
                "initial_snow_depth" => {
                    update.initial_snow_depth = Some(snowmelt_float(&value, &name)?)
                }
                "initial_free_water" => {
                    update.initial_free_water = Some(snowmelt_float(&value, &name)?)
                }
                "snow_depth_for_full_coverage" => {
                    update.snow_depth_for_full_coverage = Some(snowmelt_float(&value, &name)?)
                }
                _ => {
                    return Err(snowmelt_py_error(&format!(
                        "unknown snowmelt surface field: {name}"
                    )));
                }
            }
        }
        return Ok(SnowmeltUpdate::Surface {
            surface: snowmelt_surface(surface)?,
            changes: update,
        });
    }

    let mut patch = SnowmeltPatch::default();
    let mut removal_subcatchment = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| snowmelt_py_error("snowmelt update fields must be strings"))?;
        match name.as_str() {
            "plowable_fraction" => patch.plowable_fraction = Some(snowmelt_float(&value, &name)?),
            "plow_depth" => patch.plow_depth = Some(snowmelt_float(&value, &name)?),
            "removal_fractions" => {
                patch.removal_fractions = Some(snowmelt_removal_fractions(py, &value)?)
            }
            "removal_subcatchment" => {
                removal_subcatchment = Some(snowmelt_removal_subcatchment(
                    py, owner, generation, &value,
                )?)
            }
            _ => {
                return Err(snowmelt_py_error(&format!(
                    "unknown snowmelt field: {name}"
                )));
            }
        }
    }
    Ok(SnowmeltUpdate::Root {
        patch,
        removal_subcatchment,
    })
}

struct SubcatchmentUpdate {
    patch: SubcatchmentPatch,
    rain_gage: Option<RelatedObject>,
    outlet: Option<RelatedObject>,
    infiltration: Option<SubcatchmentInfiltrationConfiguration>,
    groundwater: Option<Option<GroundwaterSettings>>,
    snowpack: Option<Option<RelatedObject>>,
    loading: Option<Vec<(String, f64)>>,
    coverage: Option<Vec<(String, f64)>>,
}

impl SubcatchmentUpdate {
    /// Resolves relationships and builds the typed solver patch under the owner lock.
    ///
    /// # Arguments
    /// * `simulation` - Current validated Simulation Owner.
    ///
    /// # Returns
    /// Typed partial subcatchment patch in project units.
    ///
    /// # Errors
    /// Returns a relationship validation error without mutation.
    fn into_patch(mut self, simulation: &SwmmSimulation) -> Result<SubcatchmentPatch, SwmmError> {
        if let Some(rain_gage) = self.rain_gage {
            self.patch.rain_gage_index = Some(validate_relationship(
                simulation,
                &rain_gage,
                &[ObjectType::Gage],
                "rain_gage",
            )?);
        }
        if let Some(outlet) = self.outlet {
            self.patch.outlet = Some(match outlet.object_type {
                ObjectType::Node => SubcatchmentOutletRead::Node(validate_relationship(
                    simulation,
                    &outlet,
                    &[ObjectType::Node],
                    "outlet",
                )?),
                ObjectType::Subcatch => SubcatchmentOutletRead::Subcatchment(
                    validate_relationship(simulation, &outlet, &[ObjectType::Subcatch], "outlet")?,
                ),
                _ => {
                    return Err(SwmmError::with_detail(
                        ErrorCode::ApiPropertyValue,
                        "outlet has the wrong object family",
                    ));
                }
            });
        }
        if let Some(infiltration) = self.infiltration {
            self.patch.infiltration = Some(infiltration);
        }
        if let Some(groundwater) = self.groundwater {
            self.patch.groundwater = Some(match groundwater {
                Some(values) => groundwater_configuration(simulation, values)?,
                None => SubcatchmentGroundwaterConfiguration::Removed,
            });
        }
        if let Some(snowpack) = self.snowpack {
            self.patch.snowpack = Some(match snowpack {
                Some(relation) => SubcatchmentSnowpackConfiguration::Present {
                    snowmelt_index: validate_relationship(
                        simulation,
                        &relation,
                        &[ObjectType::Snowmelt],
                        "snowpack",
                    )?,
                },
                None => SubcatchmentSnowpackConfiguration::Removed,
            });
        }
        if let Some(values) = self.loading {
            self.patch.loading = Some(SubcatchmentLoadingPatch {
                initial_buildup: resolve_named_values(
                    simulation,
                    ObjectType::Pollut,
                    values,
                    true,
                )?,
            });
        }
        if let Some(values) = self.coverage {
            self.patch.coverage = Some(SubcatchmentCoveragePatch {
                fractions: resolve_named_values(simulation, ObjectType::Landuse, values, true)?,
            });
        }
        Ok(self.patch)
    }
}

/// Creates one Python validation error for root subcatchment updates.
///
/// # Arguments
/// * `detail` - Stable validation detail.
///
/// # Returns
/// Private native validation exception.
fn subcatchment_py_error(detail: &str) -> PyErr {
    native_error(
        "update_subcatchment",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}

/// Extracts one non-boolean root subcatchment floating-point value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Property name used in diagnostics.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn subcatchment_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(subcatchment_py_error(&format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| subcatchment_py_error(&format!("{name} must be numeric")))
}

/// Extracts one required boolean root subcatchment value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Property name used in diagnostics.
///
/// # Returns
/// Extracted boolean value.
///
/// # Errors
/// Returns a validation error for non-boolean values.
fn subcatchment_bool(value: &Bound<'_, PyAny>, name: &str) -> PyResult<bool> {
    if !value.is_instance_of::<PyBool>() {
        return Err(subcatchment_py_error(&format!("{name} must be bool")));
    }
    value.extract::<bool>()
}

/// Extracts one related Live View identity directly from Python.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Related Live View.
/// * `name` - Relationship property name used in diagnostics.
///
/// # Returns
/// Owned identity for owner-locked family and canonical identity validation.
///
/// # Errors
/// Returns a validation error for malformed, foreign-owner, or foreign-generation values.
fn subcatchment_related_view(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
    name: &str,
) -> PyResult<RelatedObject> {
    related_live_view(
        py,
        owner,
        generation,
        value,
        name,
        "update_subcatchment",
        false,
    )?
    .ok_or_else(|| subcatchment_py_error(&format!("{name} must be a Live View")))
}

/// Extracts one complete owned root subcatchment update.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `changes` - Supplied values keyed by public property name.
///
/// # Returns
/// Owned root update in project units with unresolved relationships.
///
/// # Errors
/// Returns a validation error for unknown fields, malformed values, or foreign relationships.
fn extract_subcatchment_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<SubcatchmentUpdate> {
    let mut patch = SubcatchmentPatch::default();
    let mut rain_gage = None;
    let mut outlet = None;
    let mut infiltration = None;
    let mut groundwater = None;
    let mut snowpack = None;
    let mut loading = None;
    let mut coverage = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| subcatchment_py_error("subcatchment update fields must be strings"))?;
        match name.as_str() {
            "tag" => {
                patch.tag = Some(
                    value
                        .extract::<String>()
                        .map_err(|_| subcatchment_py_error("tag must be str"))?,
                )
            }
            "area" => patch.area = Some(subcatchment_float(&value, &name)?),
            "rain_gage" => {
                rain_gage = Some(subcatchment_related_view(
                    py, owner, generation, &value, &name,
                )?)
            }
            "included_in_report" => {
                patch.included_in_report = Some(subcatchment_bool(&value, &name)?)
            }
            "width" => patch.width = Some(subcatchment_float(&value, &name)?),
            "slope" => patch.slope = Some(subcatchment_float(&value, &name)?),
            "curb_length" => patch.curb_length = Some(subcatchment_float(&value, &name)?),
            "impervious_fraction" => {
                patch.impervious_fraction = Some(subcatchment_float(&value, &name)?)
            }
            "zero_impervious_fraction" => {
                patch.zero_impervious_fraction = Some(subcatchment_float(&value, &name)?)
            }
            "impervious_roughness" => {
                patch.impervious_roughness = Some(subcatchment_float(&value, &name)?)
            }
            "pervious_roughness" => {
                patch.pervious_roughness = Some(subcatchment_float(&value, &name)?)
            }
            "impervious_depression_storage" => {
                patch.impervious_depression_storage = Some(subcatchment_float(&value, &name)?)
            }
            "pervious_depression_storage" => {
                patch.pervious_depression_storage = Some(subcatchment_float(&value, &name)?)
            }
            "outlet" => {
                outlet = Some(subcatchment_related_view(
                    py, owner, generation, &value, &name,
                )?)
            }
            "infiltration" => infiltration = Some(infiltration_settings(&value)?),
            "groundwater" => {
                groundwater = Some(groundwater_settings(py, owner, generation, &value)?)
            }
            "snowpack" => {
                snowpack = Some(if value.is_none() {
                    None
                } else {
                    Some(subcatchment_related_view(
                        py, owner, generation, &value, &name,
                    )?)
                })
            }
            "initial_buildup" => loading = Some(named_mapping_values(&value, "initial_buildup")?),
            "coverage_fractions" => {
                coverage = Some(named_mapping_values(&value, "coverage_fractions")?)
            }
            _ => {
                return Err(subcatchment_py_error(&format!(
                    "unknown subcatchment field: {name}"
                )));
            }
        }
    }
    Ok(SubcatchmentUpdate {
        patch,
        rain_gage,
        outlet,
        infiltration,
        groundwater,
        snowpack,
        loading,
        coverage,
    })
}

#[derive(Clone, Copy)]
enum InfiltrationField {
    InitialRate,
    MinimumRate,
    DecayCoefficient,
    DryingTimeDays,
    MaximumInfiltration,
    SuctionHead,
    HydraulicConductivity,
    InitialMoistureDeficit,
    CurveNumber,
}

#[derive(Clone, Copy)]
enum GroundwaterFloatField {
    SurfaceElevation,
    GroundwaterCoefficient,
    GroundwaterExponent,
    SurfaceCoefficient,
    SurfaceExponent,
    InteractionCoefficient,
    FixedSurfaceDepth,
    BottomElevation,
    WaterTableElevation,
    UpperMoisture,
}

struct InfiltrationUpdate {
    expected_kind: String,
    changes: Vec<(InfiltrationField, Option<f64>)>,
}

impl InfiltrationUpdate {
    /// Builds one owner-locked partial infiltration patch.
    ///
    /// # Arguments
    /// * `simulation` - Current validated Simulation Owner.
    /// * `subcatchment_index` - Zero-based configured subcatchment index.
    ///
    /// # Returns
    /// Typed partial subcatchment patch in project units.
    ///
    /// # Errors
    /// Returns stale when the slot is absent or changed kind, or a solver validation error.
    fn into_patch(
        self,
        simulation: &SwmmSimulation,
        subcatchment_index: usize,
    ) -> Result<SubcatchmentPatch, SettingsMutationFailure> {
        let mut current = simulation
            .subcatchment_infiltration_configuration_read(subcatchment_index)?
            .ok_or(SettingsMutationFailure::Stale)?;
        if infiltration_kind(current) != self.expected_kind {
            return Err(SettingsMutationFailure::Stale);
        }
        if self.changes.is_empty() {
            return Ok(SubcatchmentPatch::default());
        }
        for (field, value) in self.changes {
            current = replace_infiltration_field(current, field, value)?;
        }
        Ok(SubcatchmentPatch {
            infiltration: Some(current),
            ..SubcatchmentPatch::default()
        })
    }
}

struct GroundwaterUpdate {
    floats: Vec<(GroundwaterFloatField, f64)>,
    aquifer: Option<RelatedObject>,
    node: Option<RelatedObject>,
}

impl GroundwaterUpdate {
    /// Builds one owner-locked partial groundwater patch.
    ///
    /// # Arguments
    /// * `simulation` - Current validated Simulation Owner.
    /// * `subcatchment_index` - Zero-based configured subcatchment index.
    ///
    /// # Returns
    /// Typed partial subcatchment patch in project units.
    ///
    /// # Errors
    /// Returns stale when the slot is absent, or a relationship/solver validation error.
    fn into_patch(
        self,
        simulation: &SwmmSimulation,
        subcatchment_index: usize,
    ) -> Result<SubcatchmentPatch, SettingsMutationFailure> {
        let mut current = current_groundwater(simulation, subcatchment_index)?;
        if self.floats.is_empty() && self.aquifer.is_none() && self.node.is_none() {
            return Ok(SubcatchmentPatch::default());
        }
        for (field, value) in self.floats {
            current = replace_groundwater_float(current, field, value)?;
        }
        let SubcatchmentGroundwaterConfiguration::Present {
            aquifer_index,
            node_index,
            ..
        } = &mut current
        else {
            return Err(SettingsMutationFailure::Stale);
        };
        if let Some(aquifer) = self.aquifer {
            *aquifer_index =
                validate_relationship(simulation, &aquifer, &[ObjectType::Aquifer], "aquifer")?;
        }
        if let Some(node) = self.node {
            *node_index = validate_relationship(simulation, &node, &[ObjectType::Node], "node")?;
        }
        Ok(SubcatchmentPatch {
            groundwater: Some(current),
            ..SubcatchmentPatch::default()
        })
    }
}

/// Extracts one atomic infiltration slot update.
///
/// # Arguments
/// * `expected_kind` - Infiltration kind captured by the configuration slot view.
/// * `changes` - Supplied values keyed by public property name.
///
/// # Returns
/// Owned infiltration update in project units.
///
/// # Errors
/// Returns a validation error for unknown, boolean, or malformed values.
fn extract_infiltration_update(
    expected_kind: &str,
    changes: &Bound<'_, PyDict>,
) -> PyResult<InfiltrationUpdate> {
    let mut values = Vec::with_capacity(changes.len());
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| subcatchment_py_error("infiltration update fields must be strings"))?;
        let (field, value) = match name.as_str() {
            "initial_rate" => (
                InfiltrationField::InitialRate,
                Some(subcatchment_float(&value, &name)?),
            ),
            "minimum_rate" => (
                InfiltrationField::MinimumRate,
                Some(subcatchment_float(&value, &name)?),
            ),
            "decay_coefficient" => (
                InfiltrationField::DecayCoefficient,
                Some(subcatchment_float(&value, &name)?),
            ),
            "drying_time_days" => (
                InfiltrationField::DryingTimeDays,
                Some(subcatchment_float(&value, &name)?),
            ),
            "maximum_infiltration" => (
                InfiltrationField::MaximumInfiltration,
                if value.is_none() {
                    None
                } else {
                    Some(subcatchment_float(&value, &name)?)
                },
            ),
            "suction_head" => (
                InfiltrationField::SuctionHead,
                Some(subcatchment_float(&value, &name)?),
            ),
            "hydraulic_conductivity" => (
                InfiltrationField::HydraulicConductivity,
                Some(subcatchment_float(&value, &name)?),
            ),
            "initial_moisture_deficit" => (
                InfiltrationField::InitialMoistureDeficit,
                Some(subcatchment_float(&value, &name)?),
            ),
            "curve_number" => (
                InfiltrationField::CurveNumber,
                Some(subcatchment_float(&value, &name)?),
            ),
            _ => {
                return Err(subcatchment_py_error(&format!(
                    "unknown infiltration field: {name}"
                )));
            }
        };
        values.push((field, value));
    }
    Ok(InfiltrationUpdate {
        expected_kind: expected_kind.to_owned(),
        changes: values,
    })
}

/// Extracts one atomic groundwater slot update.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `changes` - Supplied values keyed by public property name.
///
/// # Returns
/// Owned groundwater update in project units.
///
/// # Errors
/// Returns a validation error for unknown fields, malformed values, or foreign relationships.
fn extract_groundwater_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<GroundwaterUpdate> {
    let mut floats = Vec::with_capacity(changes.len());
    let mut aquifer = None;
    let mut node = None;
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| subcatchment_py_error("groundwater update fields must be strings"))?;
        match name.as_str() {
            "aquifer" => {
                aquifer = Some(subcatchment_related_view(
                    py, owner, generation, &value, &name,
                )?)
            }
            "node" => {
                node = Some(subcatchment_related_view(
                    py, owner, generation, &value, &name,
                )?)
            }
            "surface_elevation" => floats.push((
                GroundwaterFloatField::SurfaceElevation,
                subcatchment_float(&value, &name)?,
            )),
            "groundwater_coefficient" => floats.push((
                GroundwaterFloatField::GroundwaterCoefficient,
                subcatchment_float(&value, &name)?,
            )),
            "groundwater_exponent" => floats.push((
                GroundwaterFloatField::GroundwaterExponent,
                subcatchment_float(&value, &name)?,
            )),
            "surface_coefficient" => floats.push((
                GroundwaterFloatField::SurfaceCoefficient,
                subcatchment_float(&value, &name)?,
            )),
            "surface_exponent" => floats.push((
                GroundwaterFloatField::SurfaceExponent,
                subcatchment_float(&value, &name)?,
            )),
            "interaction_coefficient" => floats.push((
                GroundwaterFloatField::InteractionCoefficient,
                subcatchment_float(&value, &name)?,
            )),
            "fixed_surface_depth" => floats.push((
                GroundwaterFloatField::FixedSurfaceDepth,
                subcatchment_float(&value, &name)?,
            )),
            "bottom_elevation" => floats.push((
                GroundwaterFloatField::BottomElevation,
                subcatchment_float(&value, &name)?,
            )),
            "water_table_elevation" => floats.push((
                GroundwaterFloatField::WaterTableElevation,
                subcatchment_float(&value, &name)?,
            )),
            "upper_moisture" => floats.push((
                GroundwaterFloatField::UpperMoisture,
                subcatchment_float(&value, &name)?,
            )),
            _ => {
                return Err(subcatchment_py_error(&format!(
                    "unknown groundwater field: {name}"
                )));
            }
        }
    }
    Ok(GroundwaterUpdate {
        floats,
        aquifer,
        node,
    })
}

enum SettingsMutationFailure {
    Stale,
    Solver(SwmmError),
}

impl From<SwmmError> for SettingsMutationFailure {
    fn from(error: SwmmError) -> Self {
        Self::Solver(error)
    }
}

#[derive(Clone)]
struct GroundwaterSettings {
    aquifer: RelatedObject,
    node: RelatedObject,
    values: [f64; 10],
}

/// Creates one solver validation error for a prepared settings mutation.
///
/// # Arguments
/// * `detail` - Stable validation detail.
///
/// # Returns
/// Typed solver property-value error.
fn settings_error(detail: impl Into<String>) -> SwmmError {
    SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into())
}

/// Extracts one required field from a settings transport mapping.
///
/// # Arguments
/// * `value` - Python settings transport mapping.
/// * `name` - Required field name.
///
/// # Returns
/// Borrowed Python field value.
///
/// # Errors
/// Returns a validation error when the field is absent.
fn settings_field<'py>(value: &Bound<'py, PyAny>, name: &str) -> PyResult<Bound<'py, PyAny>> {
    value
        .getattr(name)
        .map_err(|_| subcatchment_py_error(&format!("missing settings field: {name}")))
}

/// Extracts one optional non-boolean floating-point transport value.
///
/// # Arguments
/// * `value` - Python candidate value or `None`.
/// * `name` - Property name used in diagnostics.
///
/// # Returns
/// Optional extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn settings_optional_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<Option<f64>> {
    if value.is_none() {
        Ok(None)
    } else {
        subcatchment_float(value, name).map(Some)
    }
}

/// Extracts ordered string-keyed settings mapping entries.
///
/// # Arguments
/// * `value` - Iterable Python key/value pairs.
/// * `name` - Value name used in diagnostics.
///
/// # Returns
/// Owned ordered name/value pairs.
///
/// # Errors
/// Returns a Python extraction or validation error for malformed entries.
fn named_values(value: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<(String, f64)>> {
    value
        .try_iter()?
        .map(|entry| {
            let entry = entry?;
            let key = entry.get_item(0)?.extract::<String>().map_err(|_| {
                PyTypeError::new_err("subcatchment settings mapping keys must be str")
            })?;
            let number = subcatchment_float(&entry.get_item(1)?, name)?;
            Ok((key, number))
        })
        .collect()
}

/// Extracts ordered entries from one Python mapping-like candidate.
///
/// # Arguments
/// * `value` - Python mapping or iterable pair candidate.
/// * `name` - Value name used in diagnostics.
///
/// # Returns
/// Owned ordered name/value pairs.
///
/// # Errors
/// Returns a Python extraction or validation error for malformed entries.
fn named_mapping_values(value: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<(String, f64)>> {
    if let Ok(items) = value.call_method0("items") {
        named_values(&items, name)
    } else {
        named_values(value, name)
    }
}

/// Extracts one complete infiltration declaration.
///
/// # Arguments
/// * `value` - Discriminated Python infiltration mapping.
///
/// # Returns
/// Typed complete infiltration configuration in project units.
///
/// # Errors
/// Returns a validation error for an unknown kind or malformed field.
fn infiltration_settings(
    value: &Bound<'_, PyAny>,
) -> PyResult<SubcatchmentInfiltrationConfiguration> {
    let kind = settings_field(value, "kind")?.extract::<String>()?;
    let number = |name| subcatchment_float(&settings_field(value, name)?, name);
    match kind.as_str() {
        "horton" => Ok(SubcatchmentInfiltrationConfiguration::Horton {
            initial_rate: number("initial_rate")?,
            minimum_rate: number("minimum_rate")?,
            decay_coefficient: number("decay_coefficient")?,
            drying_time_days: number("drying_time_days")?,
            maximum_infiltration: settings_optional_float(
                &settings_field(value, "maximum_infiltration")?,
                "maximum_infiltration",
            )?,
        }),
        "modified_horton" => Ok(SubcatchmentInfiltrationConfiguration::ModifiedHorton {
            initial_rate: number("initial_rate")?,
            minimum_rate: number("minimum_rate")?,
            decay_coefficient: number("decay_coefficient")?,
            drying_time_days: number("drying_time_days")?,
            maximum_infiltration: settings_optional_float(
                &settings_field(value, "maximum_infiltration")?,
                "maximum_infiltration",
            )?,
        }),
        "green_ampt" => Ok(SubcatchmentInfiltrationConfiguration::GreenAmpt {
            suction_head: number("suction_head")?,
            hydraulic_conductivity: number("hydraulic_conductivity")?,
            initial_moisture_deficit: number("initial_moisture_deficit")?,
        }),
        "modified_green_ampt" => Ok(SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
            suction_head: number("suction_head")?,
            hydraulic_conductivity: number("hydraulic_conductivity")?,
            initial_moisture_deficit: number("initial_moisture_deficit")?,
        }),
        "curve_number" => Ok(SubcatchmentInfiltrationConfiguration::CurveNumber {
            curve_number: number("curve_number")?,
            drying_time_days: number("drying_time_days")?,
        }),
        _ => Err(subcatchment_py_error("invalid infiltration model")),
    }
}

/// Extracts one optional complete groundwater declaration.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Groundwater mapping or `None`.
///
/// # Returns
/// Owned optional groundwater transport.
///
/// # Errors
/// Returns a validation error for malformed values or relationships.
fn groundwater_settings(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<GroundwaterSettings>> {
    if value.is_none() {
        return Ok(None);
    }
    let aquifer = subcatchment_related_view(
        py,
        owner,
        generation,
        &settings_field(value, "aquifer")?,
        "aquifer",
    )?;
    let node = subcatchment_related_view(
        py,
        owner,
        generation,
        &settings_field(value, "node")?,
        "node",
    )?;
    let names = [
        "surface_elevation",
        "groundwater_coefficient",
        "groundwater_exponent",
        "surface_coefficient",
        "surface_exponent",
        "interaction_coefficient",
        "fixed_surface_depth",
        "bottom_elevation",
        "water_table_elevation",
        "upper_moisture",
    ];
    let mut values = [0.0; 10];
    for (slot, name) in values.iter_mut().zip(names) {
        *slot = subcatchment_float(&settings_field(value, name)?, name)?;
    }
    Ok(Some(GroundwaterSettings {
        aquifer,
        node,
        values,
    }))
}

/// Validates one captured relationship against current solver identity.
///
/// # Arguments
/// * `simulation` - Current Simulation Owner state.
/// * `relation` - Captured relationship identity.
/// * `expected` - Allowed object families.
/// * `name` - Relationship name used in diagnostics.
///
/// # Returns
/// Validated configured object index.
///
/// # Errors
/// Returns a validation, lifecycle, or identity-storage error.
fn validate_relationship(
    simulation: &SwmmSimulation,
    relation: &RelatedObject,
    expected: &[ObjectType],
    name: &str,
) -> Result<usize, SwmmError> {
    validate_related_object(simulation, relation, expected, name)
}

/// Resolves named mapping entries in configured identity order.
///
/// # Arguments
/// * `simulation` - Current Simulation Owner state.
/// * `object_type` - Pollutant or land-use family.
/// * `values` - Ordered case-insensitive name/value entries.
/// * `replace` - Whether omitted configured identities become zero.
///
/// # Returns
/// Configured-index entries in configured order.
///
/// # Errors
/// Returns an unknown-name, duplicate-name, lifecycle, or storage error.
fn resolve_named_values(
    simulation: &SwmmSimulation,
    object_type: ObjectType,
    values: Vec<(String, f64)>,
    replace: bool,
) -> Result<Vec<(usize, f64)>, SwmmError> {
    let identities = simulation.object_identity_indexes_read(object_type)?;
    let lookup = identities
        .iter()
        .map(|identity| (identity.id.to_lowercase(), identity.index))
        .collect::<HashMap<_, _>>();
    let mut resolved = HashMap::with_capacity(values.len());
    for (name, value) in values {
        let Some(index) = lookup.get(&name.to_lowercase()).copied() else {
            return Err(SwmmError::with_detail(ErrorCode::ApiObjectName, name));
        };
        if resolved.insert(index, value).is_some() {
            return Err(settings_error("duplicate configured settings name"));
        }
    }
    Ok(identities
        .into_iter()
        .filter_map(|identity| {
            resolved
                .remove(&identity.index)
                .or(replace.then_some(0.0))
                .map(|value| (identity.index, value))
        })
        .collect())
}

/// Validates relationships and builds one complete groundwater configuration.
///
/// # Arguments
/// * `simulation` - Current Simulation Owner state.
/// * `values` - Owned complete groundwater transport.
///
/// # Returns
/// Complete typed groundwater configuration.
///
/// # Errors
/// Returns a relationship validation or identity-storage error.
fn groundwater_configuration(
    simulation: &SwmmSimulation,
    values: GroundwaterSettings,
) -> Result<SubcatchmentGroundwaterConfiguration, SwmmError> {
    let [
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
    ] = values.values;
    Ok(SubcatchmentGroundwaterConfiguration::Present {
        aquifer_index: validate_relationship(
            simulation,
            &values.aquifer,
            &[ObjectType::Aquifer],
            "aquifer",
        )?,
        node_index: validate_relationship(simulation, &values.node, &[ObjectType::Node], "node")?,
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
    })
}

/// Returns the private name of one typed infiltration configuration.
///
/// # Arguments
/// * `value` - Typed infiltration configuration.
///
/// # Returns
/// Stable private infiltration kind name.
fn infiltration_kind(value: SubcatchmentInfiltrationConfiguration) -> &'static str {
    match value {
        SubcatchmentInfiltrationConfiguration::Horton { .. } => "horton",
        SubcatchmentInfiltrationConfiguration::ModifiedHorton { .. } => "modified_horton",
        SubcatchmentInfiltrationConfiguration::GreenAmpt { .. } => "green_ampt",
        SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt { .. } => "modified_green_ampt",
        SubcatchmentInfiltrationConfiguration::CurveNumber { .. } => "curve_number",
    }
}

/// Converts one selected infiltration declaration field to Python.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Current canonical infiltration declaration in project units.
/// * `field` - Private targeted field name.
///
/// # Returns
/// Selected kind, numeric value, or optional maximum infiltration.
///
/// # Errors
/// Returns a validation error when the field does not apply to the current kind.
fn infiltration_configuration_value(
    py: Python<'_>,
    value: SubcatchmentInfiltrationConfiguration,
    field: &str,
) -> PyResult<Py<PyAny>> {
    if field == "kind" {
        return infiltration_kind(value).into_py_any(py);
    }
    let selected = match (value, field) {
        (
            SubcatchmentInfiltrationConfiguration::Horton { initial_rate, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton { initial_rate, .. },
            "initial_rate",
        ) => initial_rate.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::Horton { minimum_rate, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton { minimum_rate, .. },
            "minimum_rate",
        ) => minimum_rate.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                decay_coefficient, ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                decay_coefficient, ..
            },
            "decay_coefficient",
        ) => decay_coefficient.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                drying_time_days, ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                drying_time_days, ..
            }
            | SubcatchmentInfiltrationConfiguration::CurveNumber {
                drying_time_days, ..
            },
            "drying_time_days",
        ) => drying_time_days.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                maximum_infiltration,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                maximum_infiltration,
                ..
            },
            "maximum_infiltration",
        ) => maximum_infiltration.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt { suction_head, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt { suction_head, .. },
            "suction_head",
        ) => suction_head.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt {
                hydraulic_conductivity,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                hydraulic_conductivity,
                ..
            },
            "hydraulic_conductivity",
        ) => hydraulic_conductivity.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt {
                initial_moisture_deficit,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                initial_moisture_deficit,
                ..
            },
            "initial_moisture_deficit",
        ) => initial_moisture_deficit.into_py_any(py),
        (
            SubcatchmentInfiltrationConfiguration::CurveNumber { curve_number, .. },
            "curve_number",
        ) => curve_number.into_py_any(py),
        _ => Err(native_error(
            "subcatchment_infiltration_value",
            SwmmError::with_detail(
                ErrorCode::ApiPropertyValue,
                format!("{field} does not apply to the current infiltration kind"),
            ),
        )),
    }?;
    Ok(selected)
}

/// Converts one selected groundwater declaration field to Python.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Current complete groundwater declaration in project units.
/// * `field` - Private targeted field name.
///
/// # Returns
/// Presence flag, related-object index, or selected numeric value.
///
/// # Errors
/// Returns stale when the optional slot is absent or a validation error for an invalid field.
fn groundwater_configuration_value(
    py: Python<'_>,
    value: SubcatchmentGroundwaterConfigurationRead,
    field: &str,
) -> PyResult<Py<PyAny>> {
    let SubcatchmentGroundwaterConfigurationRead::Present {
        aquifer_index: _,
        node_index: _,
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
    } = value
    else {
        if field == "present" {
            return false.into_py_any(py);
        }
        return Err(stale_error("subcatchment_groundwater_value"));
    };
    match field {
        "present" => true.into_py_any(py),
        "aquifer" | "node" => Err(internal_error(
            "subcatchment_groundwater_value",
            "relationship escaped native relationship marshalling",
        )),
        "surface_elevation" => surface_elevation.into_py_any(py),
        "groundwater_coefficient" => groundwater_coefficient.into_py_any(py),
        "groundwater_exponent" => groundwater_exponent.into_py_any(py),
        "surface_coefficient" => surface_coefficient.into_py_any(py),
        "surface_exponent" => surface_exponent.into_py_any(py),
        "interaction_coefficient" => interaction_coefficient.into_py_any(py),
        "fixed_surface_depth" => fixed_surface_depth.into_py_any(py),
        "bottom_elevation" => bottom_elevation.into_py_any(py),
        "water_table_elevation" => water_table_elevation.into_py_any(py),
        "upper_moisture" => upper_moisture.into_py_any(py),
        _ => Err(internal_error(
            "subcatchment_groundwater_value",
            "invalid private groundwater field",
        )),
    }
}

/// Replaces one field of the current infiltration configuration.
///
/// # Arguments
/// * `current` - Current complete infiltration configuration.
/// * `field` - Field selector.
/// * `value` - Candidate optional numeric value.
///
/// # Returns
/// Complete replacement infiltration configuration.
///
/// # Errors
/// Returns a validation error for an absent required value or invalid field.
fn replace_infiltration_field(
    mut current: SubcatchmentInfiltrationConfiguration,
    field: InfiltrationField,
    value: Option<f64>,
) -> Result<SubcatchmentInfiltrationConfiguration, SwmmError> {
    let required = || value.ok_or_else(|| settings_error("infiltration field cannot be None"));
    match (&mut current, field) {
        (
            SubcatchmentInfiltrationConfiguration::Horton { initial_rate, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton { initial_rate, .. },
            InfiltrationField::InitialRate,
        ) => *initial_rate = required()?,
        (
            SubcatchmentInfiltrationConfiguration::Horton { minimum_rate, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton { minimum_rate, .. },
            InfiltrationField::MinimumRate,
        ) => *minimum_rate = required()?,
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                decay_coefficient, ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                decay_coefficient, ..
            },
            InfiltrationField::DecayCoefficient,
        ) => *decay_coefficient = required()?,
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                drying_time_days, ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                drying_time_days, ..
            }
            | SubcatchmentInfiltrationConfiguration::CurveNumber {
                drying_time_days, ..
            },
            InfiltrationField::DryingTimeDays,
        ) => *drying_time_days = required()?,
        (
            SubcatchmentInfiltrationConfiguration::Horton {
                maximum_infiltration,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedHorton {
                maximum_infiltration,
                ..
            },
            InfiltrationField::MaximumInfiltration,
        ) => *maximum_infiltration = value,
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt { suction_head, .. }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt { suction_head, .. },
            InfiltrationField::SuctionHead,
        ) => *suction_head = required()?,
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt {
                hydraulic_conductivity,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                hydraulic_conductivity,
                ..
            },
            InfiltrationField::HydraulicConductivity,
        ) => *hydraulic_conductivity = required()?,
        (
            SubcatchmentInfiltrationConfiguration::GreenAmpt {
                initial_moisture_deficit,
                ..
            }
            | SubcatchmentInfiltrationConfiguration::ModifiedGreenAmpt {
                initial_moisture_deficit,
                ..
            },
            InfiltrationField::InitialMoistureDeficit,
        ) => *initial_moisture_deficit = required()?,
        (
            SubcatchmentInfiltrationConfiguration::CurveNumber { curve_number, .. },
            InfiltrationField::CurveNumber,
        ) => *curve_number = required()?,
        _ => return Err(settings_error("field does not belong to infiltration kind")),
    }
    Ok(current)
}

/// Reads the current complete groundwater configuration.
///
/// # Arguments
/// * `simulation` - Current Simulation Owner state.
/// * `index` - Configured subcatchment index.
///
/// # Returns
/// Complete present groundwater configuration.
///
/// # Errors
/// Returns a stale failure when absent, or a solver failure for lifecycle/storage errors.
fn current_groundwater(
    simulation: &SwmmSimulation,
    index: usize,
) -> Result<SubcatchmentGroundwaterConfiguration, SettingsMutationFailure> {
    match simulation.subcatchment_groundwater_configuration_read(index)? {
        SubcatchmentGroundwaterConfigurationRead::Removed => Err(SettingsMutationFailure::Stale),
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
        } => Ok(SubcatchmentGroundwaterConfiguration::Present {
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

/// Replaces one scalar in a complete groundwater configuration.
///
/// # Arguments
/// * `current` - Current complete groundwater configuration.
/// * `field` - Scalar field selector.
/// * `value` - Candidate project-unit value.
///
/// # Returns
/// Complete replacement groundwater configuration.
///
/// # Errors
/// Returns a validation error when groundwater is absent.
fn replace_groundwater_float(
    mut current: SubcatchmentGroundwaterConfiguration,
    field: GroundwaterFloatField,
    value: f64,
) -> Result<SubcatchmentGroundwaterConfiguration, SwmmError> {
    let SubcatchmentGroundwaterConfiguration::Present {
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
        ..
    } = &mut current
    else {
        return Err(settings_error("groundwater settings are absent"));
    };
    match field {
        GroundwaterFloatField::SurfaceElevation => *surface_elevation = value,
        GroundwaterFloatField::GroundwaterCoefficient => *groundwater_coefficient = value,
        GroundwaterFloatField::GroundwaterExponent => *groundwater_exponent = value,
        GroundwaterFloatField::SurfaceCoefficient => *surface_coefficient = value,
        GroundwaterFloatField::SurfaceExponent => *surface_exponent = value,
        GroundwaterFloatField::InteractionCoefficient => *interaction_coefficient = value,
        GroundwaterFloatField::FixedSurfaceDepth => *fixed_surface_depth = value,
        GroundwaterFloatField::BottomElevation => *bottom_elevation = value,
        GroundwaterFloatField::WaterTableElevation => *water_table_elevation = value,
        GroundwaterFloatField::UpperMoisture => *upper_moisture = value,
    }
    Ok(current)
}
