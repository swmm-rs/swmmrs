use super::*;

#[pymethods]
impl NativeSimulation {
    /// Returns the number of locally indexed LID units owned by one subcatchment.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `subcatchment_index` - Captured subcatchment index.
    /// * `subcatchment_id` - Captured canonical subcatchment ID.
    /// * `subcatchment_subtype` - Captured ordinary-object subtype.
    ///
    /// # Returns
    /// Number of subcatchment-local LID units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, index, mutex, or panic failure.
    fn lid_unit_count(
        &self,
        py: Python<'_>,
        generation: u64,
        subcatchment_index: usize,
        subcatchment_id: &str,
        subcatchment_subtype: &str,
    ) -> PyResult<usize> {
        self.checked_view(
            py,
            "lid_unit_count",
            generation,
            ObjectType::Subcatch,
            subcatchment_index,
            subcatchment_id,
            subcatchment_subtype,
            |simulation| simulation.lid_unit_count_read(subcatchment_index),
        )
    }

    /// Copies one selected locally indexed LID-unit configuration value.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `subcatchment_index` - Captured subcatchment index.
    /// * `subcatchment_id` - Captured canonical subcatchment ID.
    /// * `subcatchment_subtype` - Captured ordinary-object subtype.
    /// * `unit_index` - Subcatchment-local LID-unit position.
    /// * `field` - Private configuration field name.
    ///
    /// # Returns
    /// Selected Python-owned LID-unit configuration value.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, index, invalid-field, conversion, mutex, or panic failure.
    #[allow(clippy::too_many_arguments)]
    fn lid_unit_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        subcatchment_index: usize,
        subcatchment_id: &str,
        subcatchment_subtype: &str,
        unit_index: usize,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = lid_unit_field(field)?;
        let value = self.checked_view(
            py,
            "lid_unit_value",
            generation,
            ObjectType::Subcatch,
            subcatchment_index,
            subcatchment_id,
            subcatchment_subtype,
            |simulation| {
                simulation.lid_unit_configuration_value_read(subcatchment_index, unit_index, field)
            },
        )?;
        lid_unit_value(py, facade, generation, value)
    }

    /// Copies one LID-unit runtime snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `subcatchment_index` - Captured subcatchment index.
    /// * `subcatchment_id` - Captured canonical subcatchment ID.
    /// * `subcatchment_subtype` - Captured ordinary-object subtype.
    /// * `unit_index` - Subcatchment-local LID-unit position.
    ///
    /// # Returns
    /// Immutable native LID-unit snapshot in configured project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, index, mutex, conversion, or panic failure.
    fn lid_unit_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        subcatchment_index: usize,
        subcatchment_id: String,
        subcatchment_subtype: String,
        unit_index: usize,
    ) -> PyResult<Py<LidUnitSnapshot>> {
        let values = self.detached_checked_view(
            py,
            "lid_unit_snapshot",
            generation,
            ObjectType::Subcatch,
            subcatchment_index,
            subcatchment_id,
            subcatchment_subtype,
            move |simulation| simulation.lid_unit_snapshot_read(subcatchment_index, unit_index),
        )?;
        Py::new(py, LidUnitSnapshot::new(values))
    }

    /// Applies one atomic mapping-based update to a subcatchment-owned LID unit.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `subcatchment_index` - Captured subcatchment index.
    /// * `subcatchment_id` - Captured canonical subcatchment ID.
    /// * `subcatchment_subtype` - Captured ordinary-object subtype.
    /// * `unit_index` - Subcatchment-local LID-unit position.
    /// * `changes` - Supplied values keyed by public property name.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, extraction, relationship, validation, mutex, or panic
    /// failure without partial mutation.
    #[allow(clippy::too_many_arguments)]
    fn update_lid_unit(
        &self,
        py: Python<'_>,
        generation: u64,
        subcatchment_index: usize,
        subcatchment_id: String,
        subcatchment_subtype: String,
        unit_index: usize,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_lid_unit_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_lid_unit",
            generation,
            ObjectType::Subcatch,
            subcatchment_index,
            subcatchment_id,
            subcatchment_subtype,
            move |simulation| {
                simulation.patch_lid_unit(
                    subcatchment_index,
                    unit_index,
                    update.into_patch(simulation)?,
                )
            },
        )
    }

    /// Reports whether one optional configuration layer exists for a validated LID control.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured LID-control index.
    /// * `id` - Captured canonical LID-control ID.
    /// * `subtype` - Captured LID-control subtype.
    /// * `layer` - Private layer selector.
    ///
    /// # Returns
    /// `true` when the selected layer is configured.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, index, invalid-layer, mutex, or panic failure.
    fn lid_control_layer_exists(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        layer: &str,
    ) -> PyResult<bool> {
        let field = lid_layer_presence_field(layer)?;
        let value = self.checked_view(
            py,
            "lid_control_layer_exists",
            generation,
            ObjectType::Lid,
            index,
            id,
            subtype,
            |simulation| simulation.lid_control_configuration_value_read(index, field),
        )?;
        Ok(value.is_some())
    }

    /// Copies one selected property from a validated LID-control layer.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured LID-control index.
    /// * `id` - Captured canonical LID-control ID.
    /// * `subtype` - Captured LID-control subtype.
    /// * `layer` - Private layer selector.
    /// * `field` - Private layer-property selector.
    ///
    /// # Returns
    /// Selected Python-owned layer value.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, index, invalid-field, conversion, mutex, or panic failure.
    #[allow(clippy::too_many_arguments)]
    fn lid_control_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        layer: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = lid_control_field(layer, field)?;
        let value = self.checked_view(
            py,
            "lid_control_value",
            generation,
            ObjectType::Lid,
            index,
            id,
            subtype,
            |simulation| simulation.lid_control_configuration_value_read(index, field),
        )?;
        let Some(value) = value else {
            return Err(stale_error("lid_control_value"));
        };
        lid_control_value(py, facade, generation, value)
    }

    /// Applies one atomic mapping-based update to a configured LID-control layer.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured LID-control index.
    /// * `id` - Captured canonical LID-control ID.
    /// * `subtype` - Captured LID-control subtype.
    /// * `layer` - Private stable layer selector.
    /// * `changes` - Supplied values keyed by public layer-property name.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, extraction, validation, mutex, or panic failure without
    /// partial mutation.
    #[allow(clippy::too_many_arguments)]
    fn update_lid_control_layer(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        layer: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let patch = extract_lid_layer_patch(&layer, changes)?;
        self.detached_checked_view_result(
            py,
            "update_lid_control_layer",
            generation,
            ObjectType::Lid,
            index,
            id,
            subtype,
            move |simulation| {
                let configured = simulation
                    .lid_control_configuration_read(index)
                    .map_err(|error| DetachedFailure::Solver("update_lid_control_layer", error))?;
                if !lid_patch_layer_present(patch, &configured) {
                    return Err(DetachedFailure::Stale);
                }
                simulation
                    .patch_lid_control_layer(index, patch)
                    .map_err(|error| DetachedFailure::Solver("update_lid_control_layer", error))
            },
        )
    }
}

/// Maps one private LID-unit field name to its targeted solver selector.
///
/// # Arguments
/// * `field` - Private Python field name.
///
/// # Returns
/// Typed targeted LID-unit selector.
///
/// # Errors
/// Returns an internal error for an invalid private field.
fn lid_unit_field(field: &str) -> PyResult<LidUnitField> {
    match field {
        "area" => Ok(LidUnitField::Area),
        "full_width" => Ok(LidUnitField::FullWidth),
        "bottom_width" => Ok(LidUnitField::BottomWidth),
        "initial_saturation" => Ok(LidUnitField::InitialSaturation),
        "impervious_runoff_treated" => Ok(LidUnitField::ImperviousRunoffTreated),
        "pervious_runoff_treated" => Ok(LidUnitField::PerviousRunoffTreated),
        "control_identity" => Ok(LidUnitField::Control),
        "count" => Ok(LidUnitField::Count),
        "routes_to_pervious" => Ok(LidUnitField::RoutesToPervious),
        "drain_destination" => Ok(LidUnitField::DrainDestination),
        _ => Err(internal_error(
            "lid_unit_value",
            "invalid private LID-unit field",
        )),
    }
}

/// Maps one private LID-control layer field to its targeted solver selector.
///
/// # Arguments
/// * `layer` - Private layer name.
/// * `field` - Private layer-field name.
///
/// # Returns
/// Typed targeted LID-control selector.
///
/// # Errors
/// Returns an internal error for an invalid private selector.
fn lid_control_field(layer: &str, field: &str) -> PyResult<LidControlField> {
    let selector = match (layer, field) {
        ("surface", "thickness") => LidControlField::SurfaceThickness,
        ("surface", "vegetation_volume_fraction") => {
            LidControlField::SurfaceVegetationVolumeFraction
        }
        ("surface", "roughness") => LidControlField::SurfaceRoughness,
        ("surface", "slope") => LidControlField::SurfaceSlope,
        ("surface", "side_slope") => LidControlField::SurfaceSideSlope,
        ("surface", "alpha") => LidControlField::SurfaceAlpha,
        ("surface", "immediate_overflow") => LidControlField::SurfaceImmediateOverflow,
        ("soil", "thickness") => LidControlField::SoilThickness,
        ("soil", "porosity") => LidControlField::SoilPorosity,
        ("soil", "field_capacity") => LidControlField::SoilFieldCapacity,
        ("soil", "wilting_point") => LidControlField::SoilWiltingPoint,
        ("soil", "saturated_conductivity") => LidControlField::SoilSaturatedConductivity,
        ("soil", "conductivity_slope") => LidControlField::SoilConductivitySlope,
        ("soil", "suction_head") => LidControlField::SoilSuctionHead,
        ("storage", "thickness") => LidControlField::StorageThickness,
        ("storage", "void_ratio") => LidControlField::StorageVoidRatio,
        ("storage", "saturated_conductivity") => LidControlField::StorageSaturatedConductivity,
        ("storage", "clogging_factor") => LidControlField::StorageCloggingFactor,
        ("pavement", "thickness") => LidControlField::PavementThickness,
        ("pavement", "void_ratio") => LidControlField::PavementVoidRatio,
        ("pavement", "impervious_fraction") => LidControlField::PavementImperviousFraction,
        ("pavement", "saturated_conductivity") => LidControlField::PavementSaturatedConductivity,
        ("pavement", "clogging_factor") => LidControlField::PavementCloggingFactor,
        ("pavement", "regeneration_interval") => LidControlField::PavementRegenerationInterval,
        ("pavement", "regeneration_fraction") => LidControlField::PavementRegenerationFraction,
        ("drain", "coefficient") => LidControlField::DrainCoefficient,
        ("drain", "exponent") => LidControlField::DrainExponent,
        ("drain", "offset") => LidControlField::DrainOffset,
        ("drain", "delay") => LidControlField::DrainDelay,
        ("drain", "open_head") => LidControlField::DrainOpenHead,
        ("drain", "close_head") => LidControlField::DrainCloseHead,
        ("drain", "control_curve_identity") => LidControlField::DrainControlCurve,
        ("drainage_mat", "thickness") => LidControlField::DrainageMatThickness,
        ("drainage_mat", "void_fraction") => LidControlField::DrainageMatVoidFraction,
        ("drainage_mat", "roughness") => LidControlField::DrainageMatRoughness,
        ("drainage_mat", "alpha") => LidControlField::DrainageMatAlpha,
        _ => {
            return Err(internal_error(
                "lid_control_value",
                "invalid private LID-control field",
            ));
        }
    };
    Ok(selector)
}

/// Returns one representative targeted field for an optional LID layer.
///
/// # Arguments
/// * `layer` - Private layer name.
///
/// # Returns
/// Field whose presence matches the selected optional layer.
///
/// # Errors
/// Returns an internal error for an invalid private layer.
fn lid_layer_presence_field(layer: &str) -> PyResult<LidControlField> {
    match layer {
        "surface" => Ok(LidControlField::SurfaceThickness),
        "soil" => Ok(LidControlField::SoilThickness),
        "storage" => Ok(LidControlField::StorageThickness),
        "pavement" => Ok(LidControlField::PavementThickness),
        "drain" => Ok(LidControlField::DrainCoefficient),
        "drainage_mat" => Ok(LidControlField::DrainageMatThickness),
        _ => Err(internal_error(
            "lid_control_layer_exists",
            "invalid private LID layer",
        )),
    }
}

/// Reports whether the selected patch layer is present in a control read.
///
/// # Arguments
/// * `patch` - Typed layer-scoped patch.
/// * `configured` - Current configured optional layers.
///
/// # Returns
/// `true` when the patch's selected layer currently exists.
fn lid_patch_layer_present(patch: LidLayerPatch, configured: &LidControlRead) -> bool {
    match patch {
        LidLayerPatch::Surface(_) => configured.surface.is_some(),
        LidLayerPatch::Soil(_) => configured.soil.is_some(),
        LidLayerPatch::Storage(_) => configured.storage.is_some(),
        LidLayerPatch::Pavement(_) => configured.pavement.is_some(),
        LidLayerPatch::Drain(_) => configured.drain.is_some(),
        LidLayerPatch::DrainageMat(_) => configured.drainage_mat.is_some(),
    }
}

#[derive(Default)]
struct NativeLidUnitUpdate {
    patch: LidUnitPatch,
    control: Option<RelatedObject>,
    drain_destination: Option<Option<RelatedObject>>,
}

impl NativeLidUnitUpdate {
    /// Resolves relationships and builds one typed LID-unit patch.
    ///
    /// # Arguments
    /// * `simulation` - Locked authoritative Simulation Owner.
    ///
    /// # Returns
    /// Complete partial patch ready for atomic solver validation.
    ///
    /// # Errors
    /// Returns a family, subtype, stale-identity, or configured-storage error.
    fn into_patch(mut self, simulation: &SwmmSimulation) -> Result<LidUnitPatch, SwmmError> {
        if let Some(control) = self.control {
            self.patch.control_index = Some(validate_related_object(
                simulation,
                &control,
                &[ObjectType::Lid],
                "control",
            )?);
        }
        if let Some(destination) = self.drain_destination {
            self.patch.drain_destination = Some(match destination {
                None => LidDrainDestination::None,
                Some(relation) if relation.object_type == ObjectType::Node => {
                    LidDrainDestination::Node(validate_related_object(
                        simulation,
                        &relation,
                        &[ObjectType::Node],
                        "drain_destination",
                    )?)
                }
                Some(relation) if relation.object_type == ObjectType::Subcatch => {
                    LidDrainDestination::Subcatchment(validate_related_object(
                        simulation,
                        &relation,
                        &[ObjectType::Subcatch],
                        "drain_destination",
                    )?)
                }
                Some(_) => {
                    return Err(SwmmError::with_detail(
                        ErrorCode::ApiPropertyValue,
                        "drain_destination has the wrong object family",
                    ));
                }
            });
        }
        Ok(self.patch)
    }
}

/// Extracts one mapping-based LID-unit update while Python is attached.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target native Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `changes` - Public update mapping.
///
/// # Returns
/// Owned update with unresolved relationship identities.
///
/// # Errors
/// Returns a validation error for unknown fields, malformed values, or foreign relationships.
fn extract_lid_unit_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<NativeLidUnitUpdate> {
    let operation = "update_lid_unit";
    let mut update = NativeLidUnitUpdate::default();
    for (key, value) in changes.iter() {
        let name = key
            .extract::<String>()
            .map_err(|_| lid_py_error(operation, "LID-unit update keys must be strings"))?;
        match name.as_str() {
            "area" => update.patch.area = Some(lid_float(&value, &name, operation)?),
            "full_width" => update.patch.full_width = Some(lid_float(&value, &name, operation)?),
            "initial_saturation" => {
                update.patch.initial_saturation = Some(lid_float(&value, &name, operation)?)
            }
            "impervious_runoff_treated" => {
                update.patch.impervious_runoff_treated = Some(lid_float(&value, &name, operation)?)
            }
            "pervious_runoff_treated" => {
                update.patch.pervious_runoff_treated = Some(lid_float(&value, &name, operation)?)
            }
            "control" => {
                update.control =
                    related_live_view(py, owner, generation, &value, &name, operation, false)?;
            }
            "count" => update.patch.count = Some(lid_i32(&value, &name, operation)?),
            "routes_to_pervious" => {
                update.patch.routes_to_pervious = Some(lid_bool(&value, &name, operation)?)
            }
            "drain_destination" => {
                update.drain_destination = Some(related_live_view(
                    py, owner, generation, &value, &name, operation, true,
                )?);
            }
            _ => return Err(invalid_property(operation, &name)),
        }
    }
    Ok(update)
}

/// Extracts one layer-scoped update while Python is attached.
///
/// # Arguments
/// * `layer` - Stable private layer selector.
/// * `changes` - Public update mapping.
///
/// # Returns
/// Typed layer patch containing every supplied value.
///
/// # Errors
/// Returns a validation error for an unknown layer, field, or malformed value.
fn extract_lid_layer_patch(layer: &str, changes: &Bound<'_, PyDict>) -> PyResult<LidLayerPatch> {
    let operation = "update_lid_control_layer";
    let mut patch = match layer {
        "surface" => LidLayerPatch::Surface(LidSurfacePatch::default()),
        "soil" => LidLayerPatch::Soil(LidSoilPatch::default()),
        "storage" => LidLayerPatch::Storage(LidStoragePatch::default()),
        "pavement" => LidLayerPatch::Pavement(LidPavementPatch::default()),
        "drain" => LidLayerPatch::Drain(LidDrainPatch::default()),
        "drainage_mat" => LidLayerPatch::DrainageMat(LidDrainageMatPatch::default()),
        _ => return Err(invalid_property(operation, layer)),
    };
    for (key, value) in changes.iter() {
        let name = key
            .extract::<String>()
            .map_err(|_| lid_py_error(operation, "LID-layer update keys must be strings"))?;
        match (&mut patch, name.as_str()) {
            (LidLayerPatch::Surface(patch), "thickness") => {
                patch.thickness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Surface(patch), "vegetation_volume_fraction") => {
                patch.vegetation_volume_fraction = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Surface(patch), "roughness") => {
                patch.roughness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Surface(patch), "slope") => {
                patch.slope = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Surface(patch), "side_slope") => {
                patch.side_slope = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "thickness") => {
                patch.thickness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "porosity") => {
                patch.porosity = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "field_capacity") => {
                patch.field_capacity = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "wilting_point") => {
                patch.wilting_point = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "saturated_conductivity") => {
                patch.saturated_conductivity = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "conductivity_slope") => {
                patch.conductivity_slope = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Soil(patch), "suction_head") => {
                patch.suction_head = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Storage(patch), "thickness") => {
                patch.thickness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Storage(patch), "void_ratio") => {
                patch.void_ratio = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Storage(patch), "saturated_conductivity") => {
                patch.saturated_conductivity = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Storage(patch), "clogging_factor") => {
                patch.clogging_factor = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "thickness") => {
                patch.thickness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "void_ratio") => {
                patch.void_ratio = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "impervious_fraction") => {
                patch.impervious_fraction = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "saturated_conductivity") => {
                patch.saturated_conductivity = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "clogging_factor") => {
                patch.clogging_factor = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "regeneration_interval") => {
                patch.regeneration_interval_seconds =
                    Some(lid_duration_seconds(&value, &name, operation)?)
            }
            (LidLayerPatch::Pavement(patch), "regeneration_fraction") => {
                patch.regeneration_fraction = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "coefficient") => {
                patch.coefficient = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "exponent") => {
                patch.exponent = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "offset") => {
                patch.offset = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "delay") => {
                patch.delay_seconds = Some(lid_duration_seconds(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "open_head") => {
                patch.open_head = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::Drain(patch), "close_head") => {
                patch.close_head = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::DrainageMat(patch), "thickness") => {
                patch.thickness = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::DrainageMat(patch), "void_fraction") => {
                patch.void_fraction = Some(lid_float(&value, &name, operation)?)
            }
            (LidLayerPatch::DrainageMat(patch), "roughness") => {
                patch.roughness = Some(lid_float(&value, &name, operation)?)
            }
            _ => return Err(invalid_property(operation, &format!("{layer}.{name}"))),
        }
    }
    Ok(patch)
}

/// Extracts the total seconds from one exact Python `timedelta` value.
///
/// # Arguments
/// * `value` - Python candidate duration.
/// * `name` - Public field name for diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Duration in seconds without applying its domain range.
///
/// # Errors
/// Returns a validation error for values other than `datetime.timedelta`.
fn lid_duration_seconds(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<f64> {
    let datetime = value
        .py()
        .import("datetime")
        .map_err(|_| internal_error(operation, "datetime module is unavailable"))?;
    let timedelta = datetime
        .getattr("timedelta")
        .map_err(|_| internal_error(operation, "datetime.timedelta is unavailable"))?;
    if !value.is_instance(&timedelta)? {
        return Err(lid_py_error(
            operation,
            format!("{name} must be datetime.timedelta"),
        ));
    }
    value
        .call_method0("total_seconds")?
        .extract::<f64>()
        .map_err(|_| lid_py_error(operation, format!("{name} must be a duration")))
}

/// Creates one native LID validation exception.
///
/// # Arguments
/// * `operation` - Stable native operation name.
/// * `detail` - Human-readable validation detail.
///
/// # Returns
/// Private native validation exception.
fn lid_py_error(operation: &str, detail: impl Into<String>) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

/// Extracts one non-boolean numeric LID value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public field name for diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn lid_float(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(lid_py_error(operation, format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| lid_py_error(operation, format!("{name} must be numeric")))
}

/// Extracts one strict boolean LID value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public field name for diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Extracted boolean value.
///
/// # Errors
/// Returns a validation error for non-boolean values.
fn lid_bool(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<bool> {
    if !value.is_instance_of::<PyBool>() {
        return Err(lid_py_error(operation, format!("{name} must be bool")));
    }
    value.extract::<bool>()
}

/// Extracts one non-boolean native integer LID value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public field name for diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Extracted signed 32-bit integer.
///
/// # Errors
/// Returns a validation error for booleans, overflow, or non-integers.
fn lid_i32(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<i32> {
    if value.is_instance_of::<PyBool>() {
        return Err(lid_py_error(operation, format!("{name} must not be bool")));
    }
    value
        .extract::<i32>()
        .map_err(|_| lid_py_error(operation, format!("{name} must be an integer")))
}
