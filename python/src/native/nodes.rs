use super::*;

#[pymethods]
impl NativeSimulation {
    /// Copies one selected node configuration value after revalidating its Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    /// * `field` - Private configuration field name.
    ///
    /// # Returns
    /// Selected node configuration value in project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, invalid-field, conversion, or internal failure.
    #[allow(clippy::too_many_arguments)]
    fn node_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = node_configuration_field(field)?;
        let value = self.checked_view(
            py,
            "node_configuration_value",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_configuration_value_read(index, field),
        )?;
        match value {
            NodeConfigurationValue::Text(value) => value.into_py_any(py),
            NodeConfigurationValue::Number(value) => value.into_py_any(py),
            NodeConfigurationValue::Flag(value) => value.into_py_any(py),
            NodeConfigurationValue::StorageShape {
                kind,
                coefficients,
                curve,
            } => {
                let curve = curve
                    .map(|identity| {
                        relationship_view(py, facade, generation, ObjectType::Curve, identity)
                    })
                    .transpose()?;
                (storage_type_name(kind), coefficients, curve).into_py_any(py)
            }
            NodeConfigurationValue::StorageExfiltration(value) => value.into_py_any(py),
            NodeConfigurationValue::OutfallBoundary(boundary) => {
                outfall_boundary_parts(py, facade, generation, boundary)?.into_py_any(py)
            }
            NodeConfigurationValue::DividerRule(mode) => {
                divider_rule_parts(py, facade, generation, mode)?.into_py_any(py)
            }
            NodeConfigurationValue::Relationship(value) => {
                let object_type = match field {
                    NodeConfigurationField::OutfallRouteToSubcatchment => ObjectType::Subcatch,
                    NodeConfigurationField::DividerDivertedLink => ObjectType::Link,
                    _ => {
                        return Err(internal_error(
                            "node_configuration_value",
                            "invalid private node relationship field",
                        ));
                    }
                };
                value
                    .map(|identity| {
                        relationship_view(py, facade, generation, object_type, identity)
                    })
                    .transpose()?
                    .into_py_any(py)
            }
        }
    }

    /// Copies persistent external inflow after revalidating the node Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Persistent external inflow in configured project flow units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, conversion, or internal failure.
    fn node_external_inflow(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<f64> {
        self.checked_view(
            py,
            "node_external_inflow",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_external_inflow_read(index),
        )
    }

    /// Copies one selected current node result after revalidating its complete Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    /// * `field` - Private scalar result field name.
    ///
    /// # Returns
    /// Selected current node result in project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, invalid-field, or internal native failure.
    fn node_result(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<f64> {
        let field = node_result_field(field)?;
        self.checked_view(
            py,
            "node_result",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_result_value_read(index, field),
        )
    }

    /// Copies one fixed-shape node hydraulic snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native node snapshot in configured project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn node_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<NodeSnapshot>> {
        let ids = snapshot_ids(ids, "node_snapshot")?;
        let values =
            self.detached_checked_generation(py, "node_snapshot", generation, move |handle| {
                handle.simulation.node_snapshot_read(ids.as_deref())
            })?;
        Py::new(py, NodeSnapshot::new(values))
    }
    /// Copies one selected pollutant mapping for a validated node view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    /// * `field` - Private node-quality field name.
    ///
    /// # Returns
    /// Owned pollutant IDs and the selected concentrations in configured units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, configured-storage, invalid-field, mutex, or panic failure.
    fn node_quality_mapping(
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
            "node_quality_mapping",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_quality_read(index),
        )?;
        node_quality_mapping(values, field)
    }

    /// Copies persistent external mass fluxes for one validated node view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Owned pollutant IDs and persistent external mass fluxes in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale view, invalid private subtype, poisoned owner mutex,
    /// lifecycle, index, forcing-storage, or caught-panic failure.
    fn node_external_pollutant_mass_flux(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<PollutantMappingParts> {
        self.checked_view(
            py,
            "node_external_pollutant_mass_flux",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_external_pollutant_mass_flux_read(index),
        )
        .map(pollutant_mapping_tuple)
    }

    /// Copies one pollutant-major node snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native node-quality snapshot in configured concentration units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale generation, poisoned owner mutex, lifecycle,
    /// duplicate-ID, unknown-ID, configured-storage, or caught-panic failure.
    fn node_quality_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<NodeQualitySnapshot>> {
        let ids = snapshot_ids(ids, "node_quality_snapshot")?;
        let values = self.detached_checked_generation(
            py,
            "node_quality_snapshot",
            generation,
            move |handle| handle.simulation.node_quality_snapshot_read(ids.as_deref()),
        )?;
        Py::new(py, NodeQualitySnapshot::new(values))
    }

    /// Overrides multiple node pollutant concentrations for the next quality step atomically.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    /// * `values` - Mapping or pair iterable of pollutant IDs and concentrations.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, invalid private subtype,
    /// invalid values or pollutants, non-running lifecycle, inconsistent storage, poisoned owner
    /// mutex, or caught panic.
    fn override_node_pollutant_concentrations(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        values: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "override_node_pollutant_concentrations";
        let values = PollutantValuesPatch {
            values: extract_pollutant_values(values, "pollutant concentrations", operation)?,
        };
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            move |simulation| simulation.override_node_pollutant_concentrations(index, values),
        )
    }

    /// Reads one persistent node pollutant mass flux by configured pollutant identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    /// * `pollutant_id` - Canonical or case-insensitive pollutant identity.
    ///
    /// # Returns
    /// Persistent external mass flux in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, unknown pollutant,
    /// invalid private subtype, inconsistent storage, poisoned owner mutex, or caught panic.
    fn node_external_pollutant_mass_flux_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        pollutant_id: &Bound<'_, PyAny>,
    ) -> PyResult<f64> {
        let operation = "node_external_pollutant_mass_flux_value";
        let pollutant_id = extract_pollutant_id(pollutant_id, operation)?;
        self.checked_view(
            py,
            operation,
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| {
                simulation.node_external_pollutant_mass_flux_value_read(index, &pollutant_id)
            },
        )
    }

    /// Atomically mutates persistent node pollutant mass fluxes.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node ID.
    /// * `subtype` - Captured concrete node subtype.
    /// * `replace` - Whether omitted configured pollutants reset to zero.
    /// * `args` - Positional mapping-update arguments.
    /// * `kwargs` - Keyword mapping-update values.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, duplicate or unknown
    /// pollutants, invalid values, disallowed lifecycle, inconsistent storage, poisoned owner
    /// mutex, or caught panic.
    #[allow(clippy::too_many_arguments)]
    fn update_node_external_pollutant_mass_flux(
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
        let operation = "update_node_external_pollutant_mass_flux";
        let patch = PersistentPollutantValuesPatch {
            values: extract_pollutant_update(
                args,
                kwargs,
                "external_pollutant_mass_flux",
                operation,
            )?,
            replace,
        };
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            move |simulation| simulation.patch_node_external_pollutant_mass_flux(index, patch),
        )
    }

    /// Reads an outfall fixed stage after revalidating its complete Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Fixed stage in project length units, or `None` for non-fixed mode.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, or internal native failure.
    fn outfall_fixed_stage(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<Option<f64>> {
        self.checked_view(
            py,
            "outfall_fixed_stage",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.outfall_fixed_stage_read(index),
        )
    }

    /// Reads cumulative node inflow volume during its native lifetime.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    ///
    /// # Returns
    /// Cumulative inflow in project volume units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, or internal native failure.
    fn node_total_inflow_volume(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<f64> {
        self.checked_view(
            py,
            "node_total_inflow_volume",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            |simulation| simulation.node_total_inflow_volume_read(index),
        )
    }

    /// Applies one atomic mapping-based stable node update.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    /// * `changes` - Supplied values keyed by public property name.
    ///
    /// # Errors
    /// Returns a stale, subtype, lifecycle, extraction, relationship, validation, conversion, or internal failure without partial mutation.
    fn update_node_settings(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_node_update(py, self, generation, changes)?;
        self.detached_checked_view(
            py,
            "update_node_settings",
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            move |simulation| simulation.patch_node(index, update.into_patch(simulation)?),
        )
    }

    /// Sets persistent additive node inflow after complete Live View revalidation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    /// * `value` - Validated finite flow in project flow units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, or internal native failure.
    fn set_node_external_inflow(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_node_external_inflow";
        let value = extract_strict_number(value, "external_inflow", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            move |simulation| simulation.set_node_external_inflow(index, value),
        )
    }

    /// Sets persistent fixed outfall stage after complete Live View revalidation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured node index.
    /// * `id` - Captured canonical node identifier.
    /// * `subtype` - Captured concrete node subtype.
    /// * `value` - Validated finite stage in project length units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, or internal native failure.
    fn set_outfall_fixed_stage(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "set_outfall_fixed_stage";
        let value = extract_strict_number(value, "fixed_stage", operation)?;
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Node,
            index,
            id,
            subtype,
            move |simulation| simulation.set_outfall_fixed_stage(index, value),
        )
    }
}

/// Maps one private stable-field name to its targeted solver selector.
fn node_configuration_field(name: &str) -> PyResult<NodeConfigurationField> {
    match name {
        "tag" => Ok(NodeConfigurationField::Tag),
        "invert_elevation" => Ok(NodeConfigurationField::InvertElevation),
        "full_depth" => Ok(NodeConfigurationField::FullDepth),
        "surcharge_depth" => Ok(NodeConfigurationField::SurchargeDepth),
        "ponded_area" => Ok(NodeConfigurationField::PondedArea),
        "initial_depth" => Ok(NodeConfigurationField::InitialDepth),
        "included_in_report" => Ok(NodeConfigurationField::IncludedInReport),
        "shape" => Ok(NodeConfigurationField::StorageShape),
        "evaporation_fraction" => Ok(NodeConfigurationField::StorageEvaporationFraction),
        "exfiltration" => Ok(NodeConfigurationField::StorageExfiltration),
        "boundary" => Ok(NodeConfigurationField::OutfallBoundary),
        "has_flap_gate" => Ok(NodeConfigurationField::OutfallHasFlapGate),
        "route_to_subcatchment" => Ok(NodeConfigurationField::OutfallRouteToSubcatchment),
        "rule" => Ok(NodeConfigurationField::DividerRule),
        "diverted_link" => Ok(NodeConfigurationField::DividerDivertedLink),
        _ => Err(internal_error(
            "node_configuration_value",
            "invalid private node configuration field",
        )),
    }
}

/// Maps one private current-result name to its targeted solver selector.
fn node_result_field(name: &str) -> PyResult<NodeResultField> {
    match name {
        "depth" => Ok(NodeResultField::Depth),
        "head" => Ok(NodeResultField::Head),
        "volume" => Ok(NodeResultField::Volume),
        "lateral_inflow" => Ok(NodeResultField::LateralInflow),
        "total_inflow" => Ok(NodeResultField::TotalInflow),
        "total_outflow" => Ok(NodeResultField::TotalOutflow),
        "losses" => Ok(NodeResultField::Losses),
        "flooding" => Ok(NodeResultField::Flooding),
        "hydraulic_retention_seconds" => Ok(NodeResultField::HydraulicRetentionSeconds),
        _ => Err(internal_error(
            "node_result",
            "invalid private node result field",
        )),
    }
}

/// Returns the stable private name for one storage shape discriminator.
fn storage_type_name(kind: StorageType) -> &'static str {
    match kind {
        StorageType::Functional => "functional",
        StorageType::Tabular => "tabular",
        StorageType::Cylindrical => "cylindrical",
        StorageType::Conical => "conical",
        StorageType::Paraboloid => "paraboloid",
        StorageType::Pyramidal => "pyramidal",
    }
}

/// Converts one targeted outfall boundary to private Python parts.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `boundary` - Owned targeted outfall boundary.
///
/// # Returns
/// Python-ready outfall-boundary discriminator, stage, and optional final relationship view.
///
/// # Errors
/// Returns a relationship-construction failure for referenced curves or time series.
fn outfall_boundary_parts(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    boundary: OutfallBoundaryValue,
) -> PyResult<(&'static str, f64, Option<Py<PyAny>>)> {
    Ok(match boundary {
        OutfallBoundaryValue::Free => ("free", 0.0, None),
        OutfallBoundaryValue::Normal => ("normal", 0.0, None),
        OutfallBoundaryValue::Fixed { stage } => ("fixed", stage, None),
        OutfallBoundaryValue::Tidal { curve } => (
            "tidal",
            0.0,
            Some(relationship_view(
                py,
                facade,
                generation,
                ObjectType::Curve,
                curve,
            )?),
        ),
        OutfallBoundaryValue::TimeSeries { series } => (
            "timeseries",
            0.0,
            Some(relationship_view(
                py,
                facade,
                generation,
                ObjectType::Tseries,
                series,
            )?),
        ),
    })
}

type DividerRuleParts = (&'static str, [f64; 3], Option<Py<PyAny>>);

/// Converts one targeted divider rule to private Python parts.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `mode` - Owned targeted divider rule.
///
/// # Returns
/// Python-ready divider-rule discriminator, values, and optional final relationship view.
///
/// # Errors
/// Returns a relationship-construction failure for a referenced curve.
fn divider_rule_parts(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    mode: DividerRuleValue,
) -> PyResult<DividerRuleParts> {
    Ok(match mode {
        DividerRuleValue::Overflow => ("overflow", [0.0; 3], None),
        DividerRuleValue::Cutoff { minimum_flow } => ("cutoff", [minimum_flow, 0.0, 0.0], None),
        DividerRuleValue::Tabular { curve } => (
            "tabular",
            [0.0; 3],
            Some(relationship_view(
                py,
                facade,
                generation,
                ObjectType::Curve,
                curve,
            )?),
        ),
        DividerRuleValue::Weir {
            minimum_flow,
            maximum_head,
            discharge_coefficient,
        } => (
            "weir",
            [minimum_flow, maximum_head, discharge_coefficient],
            None,
        ),
    })
}

#[derive(Default)]
struct NodeUpdate {
    patch: NodePatch,
    storage: Option<NativeStorageUpdate>,
    outfall: Option<NativeOutfallUpdate>,
    divider: Option<NativeDividerUpdate>,
}

#[derive(Default)]
struct NativeStorageUpdate {
    shape: Option<NativeStorageShape>,
    evaporation_fraction: Option<f64>,
    exfiltration: Option<Option<StorageExfiltration>>,
}

enum NativeStorageShape {
    Tabular(RelatedObject),
    Coefficients(StorageType, [f64; 3]),
}

#[derive(Default)]
struct NativeOutfallUpdate {
    boundary: Option<NativeOutfallBoundary>,
    has_flap_gate: Option<bool>,
    route_to_subcatchment: Option<Option<RelatedObject>>,
}

enum NativeOutfallBoundary {
    Free,
    Normal,
    Fixed(f64),
    Tidal(RelatedObject),
    TimeSeries(RelatedObject),
}

#[derive(Default)]
struct NativeDividerUpdate {
    rule: Option<NativeDividerRule>,
    diverted_link: Option<Option<RelatedObject>>,
}

enum NativeDividerRule {
    Overflow,
    Cutoff(f64),
    Tabular(RelatedObject),
    Weir(f64, f64, f64),
}

impl NodeUpdate {
    /// Resolves related identities and builds one typed solver patch.
    ///
    /// # Arguments
    /// * `simulation` - Locked authoritative simulation owner.
    ///
    /// # Returns
    /// Complete partial node patch ready for atomic preparation.
    ///
    /// # Errors
    /// Returns a family, subtype, stale-identity, or configured-storage error.
    fn into_patch(mut self, simulation: &SwmmSimulation) -> Result<NodePatch, SwmmError> {
        self.patch.subtype = if let Some(storage) = self.storage {
            let shape = storage
                .shape
                .map(|shape| -> Result<CanonicalStorageShape, SwmmError> {
                    match shape {
                        NativeStorageShape::Tabular(relation) => {
                            Ok(CanonicalStorageShape::Tabular {
                                curve_index: validate_node_relationship(
                                    simulation,
                                    &relation,
                                    ObjectType::Curve,
                                    "shape.curve",
                                )?,
                            })
                        }
                        NativeStorageShape::Coefficients(kind, coefficients) => {
                            Ok(CanonicalStorageShape::Coefficients { kind, coefficients })
                        }
                    }
                })
                .transpose()?;
            Some(NodeSubtypePatch::StorageUpdate(StorageUpdate {
                shape,
                evaporation_fraction: storage.evaporation_fraction,
                exfiltration: storage.exfiltration,
            }))
        } else if let Some(outfall) = self.outfall {
            let boundary = outfall
                .boundary
                .map(|boundary| -> Result<OutfallBoundary, SwmmError> {
                    match boundary {
                        NativeOutfallBoundary::Free => Ok(OutfallBoundary::Free),
                        NativeOutfallBoundary::Normal => Ok(OutfallBoundary::Normal),
                        NativeOutfallBoundary::Fixed(stage) => Ok(OutfallBoundary::Fixed { stage }),
                        NativeOutfallBoundary::Tidal(relation) => Ok(OutfallBoundary::Tidal {
                            curve_index: validate_node_relationship(
                                simulation,
                                &relation,
                                ObjectType::Curve,
                                "boundary.reference",
                            )?,
                        }),
                        NativeOutfallBoundary::TimeSeries(relation) => {
                            Ok(OutfallBoundary::TimeSeries {
                                series_index: validate_node_relationship(
                                    simulation,
                                    &relation,
                                    ObjectType::Tseries,
                                    "boundary.reference",
                                )?,
                            })
                        }
                    }
                })
                .transpose()?;
            let route_to_subcatchment = outfall
                .route_to_subcatchment
                .map(|relation| {
                    relation
                        .map(|relation| {
                            validate_node_relationship(
                                simulation,
                                &relation,
                                ObjectType::Subcatch,
                                "route_to_subcatchment",
                            )
                        })
                        .transpose()
                })
                .transpose()?;
            Some(NodeSubtypePatch::OutfallUpdate(OutfallUpdate {
                boundary,
                has_flap_gate: outfall.has_flap_gate,
                route_to_subcatchment,
            }))
        } else if let Some(divider) = self.divider {
            let mode = divider
                .rule
                .map(|rule| -> Result<DividerMode, SwmmError> {
                    match rule {
                        NativeDividerRule::Overflow => Ok(DividerMode::Overflow),
                        NativeDividerRule::Cutoff(minimum_flow) => {
                            Ok(DividerMode::Cutoff { minimum_flow })
                        }
                        NativeDividerRule::Tabular(relation) => Ok(DividerMode::Tabular {
                            curve_index: validate_node_relationship(
                                simulation,
                                &relation,
                                ObjectType::Curve,
                                "rule.curve",
                            )?,
                        }),
                        NativeDividerRule::Weir(
                            minimum_flow,
                            maximum_head,
                            discharge_coefficient,
                        ) => Ok(DividerMode::Weir {
                            minimum_flow,
                            maximum_head,
                            discharge_coefficient,
                        }),
                    }
                })
                .transpose()?;
            let diverted_link_index = divider
                .diverted_link
                .map(|relation| {
                    relation
                        .map(|relation| {
                            validate_node_relationship(
                                simulation,
                                &relation,
                                ObjectType::Link,
                                "diverted_link",
                            )
                        })
                        .transpose()
                })
                .transpose()?;
            Some(NodeSubtypePatch::DividerUpdate(DividerUpdate {
                mode,
                diverted_link_index,
            }))
        } else {
            None
        };
        Ok(self.patch)
    }
}

/// Creates one native validation error for `update_node_settings` extraction.
///
/// # Arguments
/// * `detail` - Stable diagnostic detail.
///
/// # Returns
/// Private native validation exception.
fn node_py_error(detail: impl Into<String>) -> PyErr {
    native_error(
        "update_node_settings",
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

/// Extracts one non-boolean numeric node value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public property name for diagnostics.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error for booleans or non-numeric values.
fn node_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(node_py_error(format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| node_py_error(format!("{name} must be numeric")))
}

/// Extracts one strict boolean node value.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public property name for diagnostics.
///
/// # Returns
/// Extracted boolean value.
///
/// # Errors
/// Returns a validation error for non-boolean values.
fn node_bool(value: &Bound<'_, PyAny>, name: &str) -> PyResult<bool> {
    if !value.is_instance_of::<PyBool>() {
        return Err(node_py_error(format!("{name} must be bool")));
    }
    value.extract::<bool>()
}

/// Extracts one strict three-number tuple.
///
/// # Arguments
/// * `value` - Python tuple candidate.
/// * `name` - Public property path for diagnostics.
///
/// # Returns
/// Fixed three-value array.
///
/// # Errors
/// Returns a validation error for the wrong container, length, or element type.
fn node_triple(value: &Bound<'_, PyAny>, name: &str) -> PyResult<[f64; 3]> {
    let values = value
        .cast::<PyTuple>()
        .map_err(|_| node_py_error(format!("{name} must be a three-value tuple")))?;
    if values.len() != 3 {
        return Err(node_py_error(format!("{name} must contain three values")));
    }
    Ok([
        node_float(&values.get_item(0)?, name)?,
        node_float(&values.get_item(1)?, name)?,
        node_float(&values.get_item(2)?, name)?,
    ])
}

/// Extracts all supplied node update fields without touching solver state.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `changes` - Supplied values keyed by public property name.
///
/// # Returns
/// Owned unresolved update ready for owner-locked relationship validation.
///
/// # Errors
/// Returns a field, type, detached-value, or relationship-owner validation error.
fn extract_node_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    changes: &Bound<'_, PyDict>,
) -> PyResult<NodeUpdate> {
    let mut update = NodeUpdate::default();
    for (key, value) in changes.iter() {
        let name = key
            .extract::<String>()
            .map_err(|_| node_py_error("node update fields must be strings"))?;
        match name.as_str() {
            "tag" => {
                update.patch.tag = Some(
                    value
                        .extract::<String>()
                        .map_err(|_| node_py_error("tag must be str"))?,
                );
            }
            "invert_elevation" => {
                update.patch.invert_elevation = Some(node_float(&value, &name)?);
            }
            "full_depth" => update.patch.full_depth = Some(node_float(&value, &name)?),
            "surcharge_depth" => {
                update.patch.surcharge_depth = Some(node_float(&value, &name)?);
            }
            "ponded_area" => update.patch.ponded_area = Some(node_float(&value, &name)?),
            "initial_depth" => update.patch.initial_depth = Some(node_float(&value, &name)?),
            "included_in_report" => {
                update.patch.included_in_report = Some(node_bool(&value, &name)?);
            }
            "shape" => {
                reject_other_subtypes(&update, "storage")?;
                update.storage.get_or_insert_default().shape =
                    Some(extract_storage_shape(py, owner, generation, &value)?);
            }
            "evaporation_fraction" => {
                reject_other_subtypes(&update, "storage")?;
                update.storage.get_or_insert_default().evaporation_fraction =
                    Some(node_float(&value, &name)?);
            }
            "exfiltration" => {
                reject_other_subtypes(&update, "storage")?;
                update.storage.get_or_insert_default().exfiltration = Some(if value.is_none() {
                    None
                } else {
                    Some(extract_storage_exfiltration(&value)?)
                });
            }
            "boundary" => {
                reject_other_subtypes(&update, "outfall")?;
                update.outfall.get_or_insert_default().boundary =
                    Some(extract_outfall_boundary(py, owner, generation, &value)?);
            }
            "has_flap_gate" => {
                reject_other_subtypes(&update, "outfall")?;
                update.outfall.get_or_insert_default().has_flap_gate =
                    Some(node_bool(&value, &name)?);
            }
            "route_to_subcatchment" => {
                reject_other_subtypes(&update, "outfall")?;
                update.outfall.get_or_insert_default().route_to_subcatchment =
                    Some(related_live_view(
                        py,
                        owner,
                        generation,
                        &value,
                        &name,
                        "update_node_settings",
                        true,
                    )?);
            }
            "rule" => {
                reject_other_subtypes(&update, "divider")?;
                update.divider.get_or_insert_default().rule =
                    Some(extract_divider_rule(py, owner, generation, &value)?);
            }
            "diverted_link" => {
                reject_other_subtypes(&update, "divider")?;
                update.divider.get_or_insert_default().diverted_link = Some(related_live_view(
                    py,
                    owner,
                    generation,
                    &value,
                    &name,
                    "update_node_settings",
                    true,
                )?);
            }
            _ => return Err(node_py_error(format!("unknown node field: {name}"))),
        }
    }
    Ok(update)
}

/// Rejects one update containing fields from multiple concrete node subtypes.
///
/// # Arguments
/// * `update` - Partially extracted update.
/// * `requested` - Concrete subtype requested by the next field.
///
/// # Errors
/// Returns a validation error when another subtype already has fields.
fn reject_other_subtypes(update: &NodeUpdate, requested: &str) -> PyResult<()> {
    let conflict = match requested {
        "storage" => update.outfall.is_some() || update.divider.is_some(),
        "outfall" => update.storage.is_some() || update.divider.is_some(),
        "divider" => update.storage.is_some() || update.outfall.is_some(),
        _ => true,
    };
    if conflict {
        Err(node_py_error(
            "node update accepts fields for only one concrete subtype",
        ))
    } else {
        Ok(())
    }
}

/// Extracts one canonical storage shape and its optional related curve.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Detached `StorageShape` candidate.
///
/// # Returns
/// Owned unresolved canonical shape.
///
/// # Errors
/// Returns a declaration, type, or relationship-owner validation error.
fn extract_storage_shape(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<NativeStorageShape> {
    let malformed = || node_py_error("shape must be StorageShape");
    if !is_exact_public_python_type(value, PublicType::StorageShape, "update_node_settings")? {
        return Err(malformed());
    }
    let kind = value
        .getattr("kind")
        .map_err(|_| malformed())?
        .extract::<String>()
        .map_err(|_| malformed())?;
    let coefficients = node_triple(
        &value.getattr("coefficients").map_err(|_| malformed())?,
        "shape.coefficients",
    )?;
    let curve = value.getattr("curve").map_err(|_| malformed())?;
    if kind == "tabular" {
        if coefficients != [0.0; 3] {
            return Err(node_py_error("tabular shape coefficients must all be zero"));
        }
        let relation = related_live_view(
            py,
            owner,
            generation,
            &curve,
            "shape.curve",
            "update_node_settings",
            false,
        )?
        .ok_or_else(malformed)?;
        return Ok(NativeStorageShape::Tabular(relation));
    }
    if !curve.is_none() {
        return Err(node_py_error(
            "non-tabular storage shape cannot reference a curve",
        ));
    }
    let kind = match kind.as_str() {
        "functional" => StorageType::Functional,
        "cylindrical" => StorageType::Cylindrical,
        "conical" => StorageType::Conical,
        "paraboloid" => StorageType::Paraboloid,
        "pyramidal" => StorageType::Pyramidal,
        _ => return Err(node_py_error("unknown storage shape kind")),
    };
    Ok(NativeStorageShape::Coefficients(kind, coefficients))
}

/// Extracts one storage exfiltration declaration.
///
/// # Arguments
/// * `value` - Detached `StorageExfiltration` candidate.
///
/// # Returns
/// Typed solver declaration in project units.
///
/// # Errors
/// Returns a declaration or numeric-type validation error.
fn extract_storage_exfiltration(value: &Bound<'_, PyAny>) -> PyResult<StorageExfiltration> {
    let malformed = || node_py_error("exfiltration must be StorageExfiltration or None");
    if !is_exact_public_python_type(
        value,
        PublicType::StorageExfiltration,
        "update_node_settings",
    )? {
        return Err(malformed());
    }
    let conductivity = node_float(
        &value.getattr("conductivity").map_err(|_| malformed())?,
        "exfiltration.conductivity",
    )?;
    let suction_head = node_float(
        &value.getattr("suction_head").map_err(|_| malformed())?,
        "exfiltration.suction_head",
    )?;
    let moisture_deficit = node_float(
        &value.getattr("moisture_deficit").map_err(|_| malformed())?,
        "exfiltration.moisture_deficit",
    )?;
    if suction_head == 0.0 && moisture_deficit == 0.0 {
        Ok(StorageExfiltration::Seepage { conductivity })
    } else {
        Ok(StorageExfiltration::GreenAmpt {
            suction_head,
            conductivity,
            moisture_deficit,
        })
    }
}

/// Extracts one mutually exclusive outfall boundary declaration.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Detached `OutfallBoundary` candidate.
///
/// # Returns
/// Owned unresolved boundary declaration.
///
/// # Errors
/// Returns a declaration, type, or relationship-owner validation error.
fn extract_outfall_boundary(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<NativeOutfallBoundary> {
    let malformed = || node_py_error("boundary must be OutfallBoundary");
    if !is_exact_public_python_type(value, PublicType::OutfallBoundary, "update_node_settings")? {
        return Err(malformed());
    }
    let kind = value
        .getattr("kind")
        .map_err(|_| malformed())?
        .extract::<String>()
        .map_err(|_| malformed())?;
    let stage = node_float(
        &value.getattr("stage").map_err(|_| malformed())?,
        "boundary.stage",
    )?;
    let reference = value.getattr("reference").map_err(|_| malformed())?;
    match kind.as_str() {
        "free" | "normal" => {
            if stage != 0.0 || !reference.is_none() {
                return Err(node_py_error(
                    "free and normal boundaries accept no stage or reference",
                ));
            }
            Ok(if kind == "free" {
                NativeOutfallBoundary::Free
            } else {
                NativeOutfallBoundary::Normal
            })
        }
        "fixed" => {
            if !reference.is_none() {
                return Err(node_py_error("fixed boundary accepts no reference"));
            }
            Ok(NativeOutfallBoundary::Fixed(stage))
        }
        "tidal" | "timeseries" => {
            if stage != 0.0 {
                return Err(node_py_error(
                    "table-driven boundary accepts no fixed stage",
                ));
            }
            let relation = related_live_view(
                py,
                owner,
                generation,
                &reference,
                "boundary.reference",
                "update_node_settings",
                false,
            )?
            .ok_or_else(malformed)?;
            Ok(if kind == "tidal" {
                NativeOutfallBoundary::Tidal(relation)
            } else {
                NativeOutfallBoundary::TimeSeries(relation)
            })
        }
        _ => Err(node_py_error("unknown outfall boundary kind")),
    }
}

/// Extracts one mutually exclusive divider rule declaration.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Detached `DividerRule` candidate.
///
/// # Returns
/// Owned unresolved divider rule.
///
/// # Errors
/// Returns a declaration, type, or relationship-owner validation error.
fn extract_divider_rule(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<NativeDividerRule> {
    let malformed = || node_py_error("rule must be DividerRule");
    if !is_exact_public_python_type(value, PublicType::DividerRule, "update_node_settings")? {
        return Err(malformed());
    }
    let kind = value
        .getattr("kind")
        .map_err(|_| malformed())?
        .extract::<String>()
        .map_err(|_| malformed())?;
    let values = node_triple(
        &value.getattr("values").map_err(|_| malformed())?,
        "rule.values",
    )?;
    let curve = value.getattr("curve").map_err(|_| malformed())?;
    match kind.as_str() {
        "overflow" => {
            if values != [0.0; 3] || !curve.is_none() {
                return Err(node_py_error("overflow rule accepts no values or curve"));
            }
            Ok(NativeDividerRule::Overflow)
        }
        "cutoff" => {
            if values[1] != 0.0 || values[2] != 0.0 || !curve.is_none() {
                return Err(node_py_error("cutoff rule accepts only minimum flow"));
            }
            Ok(NativeDividerRule::Cutoff(values[0]))
        }
        "tabular" => {
            if values != [0.0; 3] {
                return Err(node_py_error("tabular rule accepts only a curve"));
            }
            let relation = related_live_view(
                py,
                owner,
                generation,
                &curve,
                "rule.curve",
                "update_node_settings",
                false,
            )?
            .ok_or_else(malformed)?;
            Ok(NativeDividerRule::Tabular(relation))
        }
        "weir" => {
            if !curve.is_none() {
                return Err(node_py_error("weir rule accepts no curve"));
            }
            Ok(NativeDividerRule::Weir(values[0], values[1], values[2]))
        }
        _ => Err(node_py_error("unknown divider rule kind")),
    }
}

/// Validates one related Live View against authoritative configured identity.
///
/// # Arguments
/// * `simulation` - Locked authoritative simulation owner.
/// * `relation` - Extracted related Live View identity.
/// * `expected` - Required object family.
/// * `name` - Public relationship property path.
///
/// # Returns
/// Validated configured related-object index.
///
/// # Errors
/// Returns a family, subtype, stale-identity, or configured-storage error.
fn validate_node_relationship(
    simulation: &SwmmSimulation,
    relation: &RelatedObject,
    expected: ObjectType,
    name: &str,
) -> Result<usize, SwmmError> {
    validate_related_object(simulation, relation, &[expected], name)
}
