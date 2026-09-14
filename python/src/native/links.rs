use super::*;

#[pymethods]
impl NativeSimulation {
    /// Copies one selected common link configuration value.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `field` - Private configuration field name.
    ///
    /// # Returns
    /// Selected Python-owned common link value.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, solver, invalid-field, conversion, or internal failure.
    #[allow(clippy::too_many_arguments)]
    fn link_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = link_configuration_field(field)?;
        let value = self.checked_view(
            py,
            "link_configuration_value",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_configuration_value_read(index, field),
        )?;
        link_configuration_value_to_py(py, facade, generation, value)
    }

    /// Copies one selected current link result after revalidating its complete Live View identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `field` - Private scalar result field name.
    ///
    /// # Returns
    /// Selected current link result in project units, or `None` when unavailable for the subtype.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, solver, invalid-field, or internal native failure.
    fn link_result(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        field: &str,
    ) -> PyResult<Option<f64>> {
        let field = link_result_field(field)?;
        self.checked_view(
            py,
            "link_result",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_result_value_read(index, field),
        )
    }

    /// Applies one atomic mapping-based stable link-settings update.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `changes` - Supplied settings keyed by public property name.
    ///
    /// # Errors
    /// Returns a stale, subtype, lifecycle, extraction, relationship, validation, conversion, or
    /// internal failure without partial mutation.
    fn update_link_settings(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let update = extract_link_update(py, self, generation, &subtype, changes)?;
        self.detached_checked_view(
            py,
            "update_link_settings",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            move |simulation| simulation.patch_link(index, update.into_patch(simulation)?),
        )
    }

    /// Switches one pump to curve-driven operation while preserving scalar settings.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured pump link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `curve` - Related `Curve` Live View.
    ///
    /// # Errors
    /// Returns a stale, subtype, lifecycle, relationship, validation, or internal failure.
    fn use_pump_curve(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        curve: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let relation = related_live_view(
            py,
            self,
            generation,
            curve,
            "curve",
            "use_pump_curve",
            false,
        )?
        .ok_or_else(|| link_py_error("use_pump_curve", "curve must be Curve"))?;
        self.detached_checked_view(
            py,
            "use_pump_curve",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            move |simulation| {
                let curve_index =
                    validate_link_relationship(simulation, &relation, ObjectType::Curve, "curve")?;
                simulation.patch_link(
                    index,
                    LinkPatch {
                        subtype: Some(LinkSubtypePatch::Pump(PumpUpdate {
                            configuration: Some(PumpConfiguration::Curve { curve_index }),
                            ..PumpUpdate::default()
                        })),
                        ..LinkPatch::default()
                    },
                )
            },
        )
    }

    /// Switches one pump to ideal operation while preserving scalar settings.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured pump link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    ///
    /// # Errors
    /// Returns a stale, subtype, lifecycle, validation, or internal failure.
    fn use_ideal_pump(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
    ) -> PyResult<()> {
        self.detached_checked_view(
            py,
            "use_ideal_pump",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            move |simulation| {
                simulation.patch_link(
                    index,
                    LinkPatch {
                        subtype: Some(LinkSubtypePatch::Pump(PumpUpdate {
                            configuration: Some(PumpConfiguration::Ideal),
                            ..PumpUpdate::default()
                        })),
                        ..LinkPatch::default()
                    },
                )
            },
        )
    }

    /// Copies one typed link's optional inlet-design identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for the final inlet view.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured parent-link index.
    /// * `id` - Captured canonical parent-link identifier.
    /// * `subtype` - Captured concrete parent-link subtype.
    ///
    /// # Returns
    /// Fresh final public inlet placement view, or `None` when the link has no inlet.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, solver, or internal native failure.
    fn link_inlet(
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
            "link_inlet",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_inlet_identity_read(index),
        )?;
        identity
            .map(|design| inlet_view(py, facade, generation, index, id, subtype, design))
            .transpose()
    }

    /// Copies one selected stable inlet configuration value.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `link_index` - Captured parent-link index.
    /// * `link_id` - Captured canonical parent-link identifier.
    /// * `link_subtype` - Captured concrete parent-link subtype.
    /// * `design_index` - Captured inlet-design discriminator.
    /// * `design_id` - Captured canonical inlet-design identifier.
    /// * `field` - Private configuration field name.
    ///
    /// # Returns
    /// Selected Python-owned stable inlet value.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, invalid-field, conversion, or internal failure.
    #[allow(clippy::too_many_arguments)]
    fn inlet_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        link_index: usize,
        link_id: &str,
        link_subtype: &str,
        design_index: usize,
        design_id: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = inlet_configuration_field(field)?;
        let value = self.checked_view(
            py,
            "inlet_configuration_value",
            generation,
            ObjectType::Link,
            link_index,
            link_id,
            link_subtype,
            |simulation| {
                simulation.inlet_configuration_value_read(
                    link_index,
                    design_index,
                    design_id,
                    field,
                )
            },
        )?;
        inlet_configuration_value_to_py(py, facade, generation, value)
    }

    /// Copies one selected persistent inlet configuration value.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade used for relationship results.
    /// * `generation` - Captured Project Generation.
    /// * `link_index` - Captured parent-link index.
    /// * `link_id` - Captured canonical parent-link identifier.
    /// * `link_subtype` - Captured concrete parent-link subtype.
    /// * `design_index` - Captured inlet-design discriminator.
    /// * `design_id` - Captured canonical inlet-design identifier.
    /// * `field` - Private configuration field name.
    ///
    /// # Returns
    /// Selected Python-owned persistent inlet value.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, invalid-field, conversion, or internal failure.
    #[allow(clippy::too_many_arguments)]
    fn inlet_persistent_configuration_value(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        link_index: usize,
        link_id: &str,
        link_subtype: &str,
        design_index: usize,
        design_id: &str,
        field: &str,
    ) -> PyResult<Py<PyAny>> {
        let field = inlet_configuration_field(field)?;
        let value = self.checked_view(
            py,
            "inlet_persistent_configuration_value",
            generation,
            ObjectType::Link,
            link_index,
            link_id,
            link_subtype,
            |simulation| {
                simulation.inlet_persistent_value_read(link_index, design_index, design_id, field)
            },
        )?;
        inlet_configuration_value_to_py(py, facade, generation, value)
    }

    /// Copies one selected current inlet result.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `link_index` - Captured parent-link index.
    /// * `link_id` - Captured canonical parent-link identifier.
    /// * `link_subtype` - Captured concrete parent-link subtype.
    /// * `design_index` - Captured inlet-design discriminator.
    /// * `design_id` - Captured canonical inlet-design identifier.
    /// * `field` - Private result field name.
    ///
    /// # Returns
    /// Selected current inlet result in public units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, invalid-field, solver, or internal failure.
    fn inlet_result(
        &self,
        py: Python<'_>,
        generation: u64,
        link_index: usize,
        link_id: &str,
        link_subtype: &str,
        design_index: usize,
        design_id: &str,
        field: &str,
    ) -> PyResult<f64> {
        let field = inlet_result_field(field)?;
        self.checked_view(
            py,
            "inlet_result",
            generation,
            ObjectType::Link,
            link_index,
            link_id,
            link_subtype,
            |simulation| {
                simulation.inlet_result_value_read(link_index, design_index, design_id, field)
            },
        )
    }

    /// Atomically updates persistent controls for one link-local inlet placement.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `link_index` - Captured parent-link index.
    /// * `link_id` - Captured canonical parent-link identifier.
    /// * `link_subtype` - Captured concrete parent-link subtype.
    /// * `design_index` - Captured inlet-design discriminator.
    /// * `design_id` - Captured canonical inlet-design identifier.
    /// * `changes` - Supplied persistent controls keyed by public property name.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, applicability, extraction, validation, or internal failure
    /// without partial mutation.
    #[allow(clippy::too_many_arguments)]
    fn update_inlet(
        &self,
        py: Python<'_>,
        generation: u64,
        link_index: usize,
        link_id: String,
        link_subtype: String,
        design_index: usize,
        design_id: String,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let mut patch = InletPersistentPatch::default();
        for (key, value) in changes.iter() {
            let name = key
                .extract::<String>()
                .map_err(|_| link_py_error("update_inlet", "inlet fields must be strings"))?;
            match name.as_str() {
                "percent_clogged" => {
                    patch.percent_clogged = Some(link_float(&value, &name, "update_inlet")?);
                }
                "flow_limit" => {
                    patch.flow_limit = Some(link_float(&value, &name, "update_inlet")?);
                }
                _ => {
                    return Err(link_py_error(
                        "update_inlet",
                        format!("unknown or read-only inlet field: {name}"),
                    ));
                }
            }
        }
        self.detached_checked_view(
            py,
            "update_inlet",
            generation,
            ObjectType::Link,
            link_index,
            link_id,
            link_subtype,
            move |simulation| {
                simulation.patch_inlet_persistent(link_index, design_index, &design_id, patch)
            },
        )
    }

    /// Copies one fixed-shape link hydraulic snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native link snapshot in configured project units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, identity, mutex, or panic failure.
    fn link_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<LinkSnapshot>> {
        let ids = snapshot_ids(ids, "link_snapshot")?;
        let values =
            self.detached_checked_generation(py, "link_snapshot", generation, move |handle| {
                handle.simulation.link_snapshot_read(ids.as_deref())
            })?;
        Py::new(py, LinkSnapshot::new(values))
    }
    /// Copies one selected pollutant mapping for a validated link view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    /// * `field` - Private link-quality field name.
    ///
    /// # Returns
    /// Owned pollutant IDs and the selected quality values in configured units.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, configured-storage, invalid-field, mutex, or panic failure.
    fn link_quality_mapping(
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
            "link_quality_mapping",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_quality_read(index),
        )?;
        link_quality_mapping(values, field)
    }

    /// Copies persistent external mass fluxes for one validated link view.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    ///
    /// # Returns
    /// Owned pollutant IDs and persistent external mass fluxes in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale view, invalid private subtype, poisoned owner mutex,
    /// lifecycle, index, forcing-storage, or caught-panic failure.
    fn link_external_pollutant_mass_flux(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<PollutantMappingParts> {
        self.checked_view(
            py,
            "link_external_pollutant_mass_flux",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| simulation.link_external_pollutant_mass_flux_read(index),
        )
        .map(pollutant_mapping_tuple)
    }

    /// Copies one pollutant-major link snapshot while detached.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during acquisition.
    /// * `generation` - Captured Project Generation.
    /// * `ids` - Caller selection as `None`, one ID, or an ordered ID iterable.
    ///
    /// # Returns
    /// Immutable native link-quality snapshot in configured reporting units.
    ///
    /// # Errors
    /// Returns a Python exception for a stale generation, poisoned owner mutex, lifecycle,
    /// duplicate-ID, unknown-ID, configured-storage, or caught-panic failure.
    fn link_quality_snapshot(
        &self,
        py: Python<'_>,
        generation: u64,
        ids: &Bound<'_, PyAny>,
    ) -> PyResult<Py<LinkQualitySnapshot>> {
        let ids = snapshot_ids(ids, "link_quality_snapshot")?;
        let values = self.detached_checked_generation(
            py,
            "link_quality_snapshot",
            generation,
            move |handle| handle.simulation.link_quality_snapshot_read(ids.as_deref()),
        )?;
        Py::new(py, LinkQualitySnapshot::new(values))
    }

    /// Overrides multiple link pollutant concentrations for the next quality step atomically.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    /// * `values` - Mapping or pair iterable of pollutant IDs and concentrations.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, invalid private subtype,
    /// invalid values or pollutants, non-running lifecycle, inconsistent storage, poisoned owner
    /// mutex, or caught panic.
    fn override_link_pollutant_concentrations(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: String,
        values: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let operation = "override_link_pollutant_concentrations";
        let values = PollutantValuesPatch {
            values: extract_pollutant_values(values, "pollutant concentrations", operation)?,
        };
        self.detached_checked_view(
            py,
            operation,
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            move |simulation| simulation.override_link_pollutant_concentrations(index, values),
        )
    }

    /// Reads one persistent link pollutant mass flux by configured pollutant identity.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    /// * `pollutant_id` - Canonical or case-insensitive pollutant identity.
    ///
    /// # Returns
    /// Persistent external mass flux in configured public units.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, unknown pollutant,
    /// invalid private subtype, inconsistent storage, poisoned owner mutex, or caught panic.
    fn link_external_pollutant_mass_flux_value(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: &str,
        subtype: &str,
        pollutant_id: &Bound<'_, PyAny>,
    ) -> PyResult<f64> {
        let operation = "link_external_pollutant_mass_flux_value";
        let pollutant_id = extract_pollutant_id(pollutant_id, operation)?;
        self.checked_view(
            py,
            operation,
            generation,
            ObjectType::Link,
            index,
            id,
            subtype,
            |simulation| {
                simulation.link_external_pollutant_mass_flux_value_read(index, &pollutant_id)
            },
        )
    }

    /// Atomically mutates persistent link pollutant mass fluxes.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured configured link index.
    /// * `id` - Captured canonical link ID.
    /// * `subtype` - Captured concrete link subtype.
    /// * `replace` - Whether omitted configured pollutants reset to zero.
    /// * `args` - Positional mapping-update arguments.
    /// * `kwargs` - Keyword mapping-update values.
    ///
    /// # Errors
    /// Returns a Python exception for malformed input, a stale view, duplicate or unknown
    /// pollutants, invalid values, disallowed lifecycle, inconsistent storage, poisoned owner
    /// mutex, or caught panic.
    #[allow(clippy::too_many_arguments)]
    fn update_link_external_pollutant_mass_flux(
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
        let operation = "update_link_external_pollutant_mass_flux";
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
            ObjectType::Link,
            index,
            id,
            subtype,
            move |simulation| simulation.patch_link_external_pollutant_mass_flux(index, patch),
        )
    }

    /// Sets one typed link's persistent flow limit in project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `value` - Proposed flow limit in project flow units.
    ///
    /// # Returns
    /// `Ok(())` after the owner applies the limit.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, solver, or internal native failure.
    fn set_link_flow_limit(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: &str,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let value = extract_strict_number(value, "flow_limit", "set_link_flow_limit")?;
        self.detached_checked_view(
            py,
            "set_link_flow_limit",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype.to_owned(),
            move |simulation| simulation.set_link_flow_limit(index, value),
        )
    }

    /// Applies one typed link's active target setting and coupled coefficients.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `index` - Captured link index.
    /// * `id` - Captured canonical link identifier.
    /// * `subtype` - Captured concrete link subtype.
    /// * `value` - Proposed active setting or pump speed factor.
    ///
    /// # Returns
    /// `Ok(())` after the owner applies the coupled setting update.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, validation, solver, or internal native failure.
    fn set_link_target_setting(
        &self,
        py: Python<'_>,
        generation: u64,
        index: usize,
        id: String,
        subtype: &str,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let value = extract_strict_number(value, "target_setting", "set_link_target_setting")?;
        self.detached_checked_view(
            py,
            "set_link_target_setting",
            generation,
            ObjectType::Link,
            index,
            id,
            subtype.to_owned(),
            move |simulation| simulation.set_link_target_setting(index, value),
        )
    }
}

#[derive(Default)]
struct LinkUpdate {
    patch: LinkPatch,
    inlet_node: Option<RelatedObject>,
    outlet_node: Option<RelatedObject>,
    conduit: Option<ConduitPatch>,
    cross_section: Option<NativeCrossSection>,
    pump: Option<PumpUpdate>,
    orifice: Option<OrificeUpdate>,
    weir: Option<NativeWeirUpdate>,
    outlet: Option<NativeOutletRating>,
}

#[derive(Default)]
struct NativeWeirUpdate {
    update: WeirUpdate,
    coefficient_curve: Option<Option<RelatedObject>>,
}

enum NativeCrossSection {
    Circular(f64),
    Standard(XsectType, [f64; 4], usize),
    Custom(RelatedObject, f64, usize),
    Irregular(RelatedObject, usize),
    Street(RelatedObject, usize),
}

enum NativeOutletRating {
    Functional(f64, f64, OutletHeadBasis),
    Tabular(RelatedObject, OutletHeadBasis),
}

impl LinkUpdate {
    /// Resolves relationships and builds one typed solver patch under the owner lock.
    ///
    /// # Arguments
    /// * `simulation` - Locked authoritative Simulation Owner.
    ///
    /// # Returns
    /// Typed partial link patch ready for atomic solver preparation.
    ///
    /// # Errors
    /// Returns a family, subtype, identity, or configured-storage error.
    fn into_patch(mut self, simulation: &SwmmSimulation) -> Result<LinkPatch, SwmmError> {
        self.patch.inlet_node_index = self
            .inlet_node
            .as_ref()
            .map(|relation| {
                validate_link_relationship(simulation, relation, ObjectType::Node, "inlet_node")
            })
            .transpose()?;
        self.patch.outlet_node_index = self
            .outlet_node
            .as_ref()
            .map(|relation| {
                validate_link_relationship(simulation, relation, ObjectType::Node, "outlet_node")
            })
            .transpose()?;
        self.patch.cross_section = self
            .cross_section
            .map(|value| -> Result<CrossSectionConfiguration, SwmmError> {
                Ok(match value {
                    NativeCrossSection::Circular(diameter) => {
                        CrossSectionConfiguration::Circular { diameter }
                    }
                    NativeCrossSection::Standard(shape, geometry, culvert_code) => {
                        CrossSectionConfiguration::Standard {
                            shape,
                            geometry,
                            culvert_code,
                        }
                    }
                    NativeCrossSection::Custom(relation, full_depth, culvert_code) => {
                        let shape_index = validate_link_relationship(
                            simulation,
                            &relation,
                            ObjectType::Shape,
                            "cross_section.shape",
                        )?;
                        CrossSectionConfiguration::Custom {
                            shape_curve_index: simulation
                                .custom_shape_curve_index_read(shape_index)?,
                            full_depth,
                            culvert_code,
                        }
                    }
                    NativeCrossSection::Irregular(relation, culvert_code) => {
                        CrossSectionConfiguration::Irregular {
                            transect_index: validate_link_relationship(
                                simulation,
                                &relation,
                                ObjectType::Transect,
                                "cross_section.transect",
                            )?,
                            culvert_code,
                        }
                    }
                    NativeCrossSection::Street(relation, culvert_code) => {
                        CrossSectionConfiguration::Street {
                            street_index: validate_link_relationship(
                                simulation,
                                &relation,
                                ObjectType::Street,
                                "cross_section.street",
                            )?,
                            culvert_code,
                        }
                    }
                })
            })
            .transpose()?;
        self.patch.subtype = if let Some(update) = self.conduit {
            Some(LinkSubtypePatch::Conduit(update))
        } else if let Some(update) = self.pump {
            Some(LinkSubtypePatch::Pump(update))
        } else if let Some(update) = self.orifice {
            Some(LinkSubtypePatch::Orifice(update))
        } else if let Some(update) = self.weir {
            let coefficient_curve_index = update
                .coefficient_curve
                .map(|relation| {
                    relation
                        .as_ref()
                        .map(|relation| {
                            validate_link_relationship(
                                simulation,
                                relation,
                                ObjectType::Curve,
                                "coefficient_curve",
                            )
                        })
                        .transpose()
                })
                .transpose()?;
            Some(LinkSubtypePatch::Weir(WeirUpdate {
                coefficient_curve_index,
                ..update.update
            }))
        } else if let Some(rating) = self.outlet {
            let rating = match rating {
                NativeOutletRating::Functional(coefficient, exponent, head_basis) => {
                    OutletRating::Functional {
                        coefficient,
                        exponent,
                        head_basis,
                    }
                }
                NativeOutletRating::Tabular(relation, head_basis) => OutletRating::Tabular {
                    curve_index: validate_link_relationship(
                        simulation,
                        &relation,
                        ObjectType::Curve,
                        "rating.curve",
                    )?,
                    head_basis,
                },
            };
            Some(LinkSubtypePatch::Outlet(OutletUpdate {
                rating: Some(rating),
            }))
        } else {
            None
        };
        Ok(self.patch)
    }
}

/// Creates one validation error for link-settings extraction.
///
/// # Arguments
/// * `operation` - Stable native operation name.
/// * `detail` - Validation diagnostic detail.
///
/// # Returns
/// Private native validation exception.
fn link_py_error(operation: &str, detail: impl Into<String>) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

/// Extracts one non-boolean numeric link value.
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
fn link_float(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>() {
        return Err(link_py_error(operation, format!("{name} must not be bool")));
    }
    value
        .extract::<f64>()
        .map_err(|_| link_py_error(operation, format!("{name} must be numeric")))
}

/// Extracts one strict boolean link value.
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
fn link_bool(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<bool> {
    if !value.is_instance_of::<PyBool>() {
        return Err(link_py_error(operation, format!("{name} must be bool")));
    }
    value.extract::<bool>()
}

/// Extracts one non-boolean nonnegative integer without applying its domain range.
///
/// # Arguments
/// * `value` - Python candidate value.
/// * `name` - Public field name for diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Extracted unsigned integer.
///
/// # Errors
/// Returns a validation error for booleans, negative values, or non-integers.
fn link_usize(value: &Bound<'_, PyAny>, name: &str, operation: &str) -> PyResult<usize> {
    if value.is_instance_of::<PyBool>() {
        return Err(link_py_error(operation, format!("{name} must not be bool")));
    }
    value
        .extract::<usize>()
        .map_err(|_| link_py_error(operation, format!("{name} must be a nonnegative integer")))
}

/// Extracts one four-value numeric tuple.
///
/// # Arguments
/// * `value` - Python tuple candidate.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Fixed four-value geometry array.
///
/// # Errors
/// Returns a validation error for the wrong container, length, or element type.
fn link_geometry(value: &Bound<'_, PyAny>, operation: &str) -> PyResult<[f64; 4]> {
    let values = value
        .cast::<PyTuple>()
        .map_err(|_| link_py_error(operation, "geometry must be a four-value tuple"))?;
    if values.len() != 4 {
        return Err(link_py_error(
            operation,
            "geometry must contain four values",
        ));
    }
    Ok([
        link_float(&values.get_item(0)?, "geometry", operation)?,
        link_float(&values.get_item(1)?, "geometry", operation)?,
        link_float(&values.get_item(2)?, "geometry", operation)?,
        link_float(&values.get_item(3)?, "geometry", operation)?,
    ])
}

/// Extracts every supplied link settings field without touching solver state.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `subtype` - Captured concrete link subtype.
/// * `changes` - Supplied values keyed by public settings name.
///
/// # Returns
/// Owned unresolved update ready for owner-locked relationship validation.
///
/// # Errors
/// Returns a field, type, detached-value, or relationship-owner validation error.
fn extract_link_update(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    subtype: &str,
    changes: &Bound<'_, PyDict>,
) -> PyResult<LinkUpdate> {
    let operation = "update_link_settings";
    let mut update = LinkUpdate::default();
    for (key, value) in changes.iter() {
        let name = key
            .extract::<String>()
            .map_err(|_| link_py_error(operation, "link settings fields must be strings"))?;
        match name.as_str() {
            "tag" => {
                update.patch.tag = Some(
                    value
                        .extract::<String>()
                        .map_err(|_| link_py_error(operation, "tag must be str"))?,
                );
            }
            "inlet_node" => {
                update.inlet_node = Some(
                    related_live_view(py, owner, generation, &value, &name, operation, false)?
                        .ok_or_else(|| link_py_error(operation, "inlet_node must be Node"))?,
                );
            }
            "outlet_node" => {
                update.outlet_node = Some(
                    related_live_view(py, owner, generation, &value, &name, operation, false)?
                        .ok_or_else(|| link_py_error(operation, "outlet_node must be Node"))?,
                );
            }
            "inlet_offset" => {
                update.patch.inlet_offset = Some(link_float(&value, &name, operation)?);
            }
            "outlet_offset" => {
                update.patch.outlet_offset = Some(link_float(&value, &name, operation)?);
            }
            "included_in_report" => {
                update.patch.included_in_report = Some(link_bool(&value, &name, operation)?);
            }
            "initial_flow" => {
                update.patch.initial_flow = Some(link_float(&value, &name, operation)?);
            }
            "inlet_loss_coefficient" => {
                update.patch.inlet_loss_coefficient = Some(link_float(&value, &name, operation)?);
            }
            "outlet_loss_coefficient" => {
                update.patch.outlet_loss_coefficient = Some(link_float(&value, &name, operation)?);
            }
            "average_loss_coefficient" => {
                update.patch.average_loss_coefficient = Some(link_float(&value, &name, operation)?);
            }
            "seepage_rate" => {
                update.patch.seepage_rate = Some(link_float(&value, &name, operation)?);
            }
            "has_flap_gate" => {
                update.patch.has_flap_gate = Some(link_bool(&value, &name, operation)?);
            }
            "length" if subtype == "conduit" => {
                update.conduit.get_or_insert_default().length =
                    Some(link_float(&value, &name, operation)?);
            }
            "roughness" if subtype == "conduit" => {
                update.conduit.get_or_insert_default().roughness =
                    Some(link_float(&value, &name, operation)?);
            }
            "barrels" if subtype == "conduit" => {
                if value.is_instance_of::<PyBool>() {
                    return Err(link_py_error(operation, "barrels must not be bool"));
                }
                let barrels = value.extract::<u8>().map_err(|_| {
                    link_py_error(operation, "barrels must be an integer in [0, 255]")
                })?;
                update.conduit.get_or_insert_default().barrels = Some(barrels);
            }
            "cross_section" if subtype == "conduit" => {
                update.cross_section = Some(extract_cross_section(py, owner, generation, &value)?);
            }
            "initial_setting" if subtype == "pump" => {
                update.pump.get_or_insert_default().initial_setting =
                    Some(link_float(&value, &name, operation)?);
            }
            "startup_depth" if subtype == "pump" => {
                update.pump.get_or_insert_default().startup_depth =
                    Some(link_float(&value, &name, operation)?);
            }
            "shutoff_depth" if subtype == "pump" => {
                update.pump.get_or_insert_default().shutoff_depth =
                    Some(link_float(&value, &name, operation)?);
            }
            "kind" if subtype == "orifice" => {
                update.orifice.get_or_insert_default().kind = Some(extract_orifice_kind(&value)?);
            }
            "discharge_coefficient" if subtype == "orifice" => {
                update.orifice.get_or_insert_default().discharge_coefficient =
                    Some(link_float(&value, &name, operation)?);
            }
            "opening_time_hours" if subtype == "orifice" => {
                update.orifice.get_or_insert_default().opening_time_hours =
                    Some(link_float(&value, &name, operation)?);
            }
            "kind" if subtype == "weir" => {
                update.weir.get_or_insert_default().update.kind = Some(extract_weir_kind(&value)?);
            }
            "discharge_coefficient" if subtype == "weir" => {
                update
                    .weir
                    .get_or_insert_default()
                    .update
                    .discharge_coefficient = Some(link_float(&value, &name, operation)?);
            }
            "end_discharge_coefficient" if subtype == "weir" => {
                update
                    .weir
                    .get_or_insert_default()
                    .update
                    .end_discharge_coefficient = Some(link_float(&value, &name, operation)?);
            }
            "end_contractions" if subtype == "weir" => {
                update.weir.get_or_insert_default().update.end_contractions =
                    Some(link_float(&value, &name, operation)?);
            }
            "can_surcharge" if subtype == "weir" => {
                update.weir.get_or_insert_default().update.can_surcharge =
                    Some(link_bool(&value, &name, operation)?);
            }
            "roadway_width" if subtype == "weir" => {
                update.weir.get_or_insert_default().update.roadway_width =
                    Some(link_float(&value, &name, operation)?);
            }
            "roadway_surface" if subtype == "weir" => {
                update.weir.get_or_insert_default().update.roadway_surface =
                    Some(extract_road_surface(&value)?);
            }
            "coefficient_curve" if subtype == "weir" => {
                update.weir.get_or_insert_default().coefficient_curve = Some(related_live_view(
                    py, owner, generation, &value, &name, operation, true,
                )?);
            }
            "rating" if subtype == "outlet" => {
                update.outlet = Some(extract_outlet_rating(py, owner, generation, &value)?);
            }
            _ => {
                return Err(link_py_error(
                    operation,
                    format!("unknown or read-only {subtype} link setting: {name}"),
                ));
            }
        }
    }
    Ok(update)
}

/// Extracts one required detached-declaration attribute.
///
/// # Arguments
/// * `value` - Detached declaration candidate.
/// * `name` - Required attribute name.
/// * `declaration` - Public declaration name for diagnostics.
///
/// # Returns
/// Borrowed Python attribute value.
///
/// # Errors
/// Returns a validation error when the attribute is absent.
fn link_attribute<'py>(
    value: &Bound<'py, PyAny>,
    name: &str,
    declaration: &str,
) -> PyResult<Bound<'py, PyAny>> {
    value.getattr(name).map_err(|_| {
        link_py_error(
            "update_link_settings",
            format!("{declaration} must be a typed declaration"),
        )
    })
}

/// Extracts one detached typed cross-section declaration.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Detached cross-section candidate.
///
/// # Returns
/// Owned unresolved typed cross-section declaration.
///
/// # Errors
/// Returns a declaration, numeric-type, or relationship-owner validation error.
fn extract_cross_section(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<NativeCrossSection> {
    let operation = "update_link_settings";
    if is_exact_public_python_type(value, PublicType::CircularCrossSection, operation)? {
        return Ok(NativeCrossSection::Circular(link_float(
            &link_attribute(value, "diameter", "cross_section")?,
            "cross_section.diameter",
            operation,
        )?));
    }
    let culvert_code = || {
        link_usize(
            &link_attribute(value, "culvert_code", "cross_section")?,
            "cross_section.culvert_code",
            operation,
        )
    };
    if is_exact_public_python_type(value, PublicType::StandardCrossSection, operation)? {
        return Ok(NativeCrossSection::Standard(
            extract_xsect_type(&link_attribute(value, "shape", "cross_section")?)?,
            link_geometry(
                &link_attribute(value, "geometry", "cross_section")?,
                operation,
            )?,
            culvert_code()?,
        ));
    }
    if is_exact_public_python_type(value, PublicType::CustomCrossSection, operation)? {
        return Ok(NativeCrossSection::Custom(
            related_live_view(
                py,
                owner,
                generation,
                &link_attribute(value, "shape", "cross_section")?,
                "cross_section.shape",
                operation,
                false,
            )?
            .ok_or_else(|| link_py_error(operation, "cross_section.shape is required"))?,
            link_float(
                &link_attribute(value, "full_depth", "cross_section")?,
                "cross_section.full_depth",
                operation,
            )?,
            culvert_code()?,
        ));
    }
    if is_exact_public_python_type(value, PublicType::IrregularCrossSection, operation)? {
        return Ok(NativeCrossSection::Irregular(
            related_live_view(
                py,
                owner,
                generation,
                &link_attribute(value, "transect", "cross_section")?,
                "cross_section.transect",
                operation,
                false,
            )?
            .ok_or_else(|| link_py_error(operation, "cross_section.transect is required"))?,
            culvert_code()?,
        ));
    }
    if is_exact_public_python_type(value, PublicType::StreetCrossSection, operation)? {
        return Ok(NativeCrossSection::Street(
            related_live_view(
                py,
                owner,
                generation,
                &link_attribute(value, "street", "cross_section")?,
                "cross_section.street",
                operation,
                false,
            )?
            .ok_or_else(|| link_py_error(operation, "cross_section.street is required"))?,
            culvert_code()?,
        ));
    }
    Err(link_py_error(
        operation,
        "cross_section must be a typed declaration",
    ))
}

/// Extracts one detached outlet rating declaration.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Detached outlet-rating candidate.
///
/// # Returns
/// Owned unresolved outlet-rating declaration.
///
/// # Errors
/// Returns a declaration, numeric-type, enum, or relationship-owner validation error.
fn extract_outlet_rating(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
) -> PyResult<NativeOutletRating> {
    let operation = "update_link_settings";
    if is_exact_public_python_type(value, PublicType::FunctionalOutletRating, operation)? {
        return Ok(NativeOutletRating::Functional(
            link_float(
                &link_attribute(value, "coefficient", "rating")?,
                "rating.coefficient",
                operation,
            )?,
            link_float(
                &link_attribute(value, "exponent", "rating")?,
                "rating.exponent",
                operation,
            )?,
            extract_outlet_head_basis(&link_attribute(value, "head_basis", "rating")?)?,
        ));
    }
    if is_exact_public_python_type(value, PublicType::TabularOutletRating, operation)? {
        return Ok(NativeOutletRating::Tabular(
            related_live_view(
                py,
                owner,
                generation,
                &link_attribute(value, "curve", "rating")?,
                "rating.curve",
                operation,
                false,
            )?
            .ok_or_else(|| link_py_error(operation, "rating.curve is required"))?,
            extract_outlet_head_basis(&link_attribute(value, "head_basis", "rating")?)?,
        ));
    }
    Err(link_py_error(
        operation,
        "rating must be a typed declaration",
    ))
}

/// Extracts one string enum value.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
/// * `name` - Public enum field name.
/// * `expected_type` - Required public string-enum class identity.
///
/// # Returns
/// Extracted stable enum spelling.
///
/// # Errors
/// Returns a validation error for the wrong enum class or a non-string value.
fn link_enum_name(
    value: &Bound<'_, PyAny>,
    name: &str,
    expected_type: PublicType,
) -> PyResult<String> {
    if !is_exact_public_python_type(value, expected_type, "update_link_settings")? {
        return Err(link_py_error(
            "update_link_settings",
            format!("{name} must be {}", expected_type.name()),
        ));
    }
    value.extract::<String>().map_err(|_| {
        link_py_error(
            "update_link_settings",
            format!("{name} must be {}", expected_type.name()),
        )
    })
}

/// Extracts one public orifice-kind enum.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
///
/// # Returns
/// Typed solver orifice kind.
///
/// # Errors
/// Returns a validation error for an unknown spelling.
fn extract_orifice_kind(value: &Bound<'_, PyAny>) -> PyResult<OrificeType> {
    match link_enum_name(value, "kind", PublicType::OrificeKind)?.as_str() {
        "side" => Ok(OrificeType::SideOrifice),
        "bottom" => Ok(OrificeType::BottomOrifice),
        _ => Err(link_py_error(
            "update_link_settings",
            "invalid orifice kind",
        )),
    }
}

/// Extracts one public weir-kind enum.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
///
/// # Returns
/// Typed solver weir kind.
///
/// # Errors
/// Returns a validation error for an unknown spelling.
fn extract_weir_kind(value: &Bound<'_, PyAny>) -> PyResult<WeirType> {
    match link_enum_name(value, "kind", PublicType::WeirKind)?.as_str() {
        "transverse" => Ok(WeirType::TransverseWeir),
        "sideflow" => Ok(WeirType::SideflowWeir),
        "v_notch" => Ok(WeirType::VnotchWeir),
        "trapezoidal" => Ok(WeirType::TrapezoidalWeir),
        "roadway" => Ok(WeirType::RoadwayWeir),
        _ => Err(link_py_error("update_link_settings", "invalid weir kind")),
    }
}

/// Extracts one public roadway-surface enum.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
///
/// # Returns
/// Typed solver roadway surface.
///
/// # Errors
/// Returns a validation error for an unknown spelling.
fn extract_road_surface(value: &Bound<'_, PyAny>) -> PyResult<RoadSurface> {
    match link_enum_name(value, "roadway_surface", PublicType::RoadSurface)?.as_str() {
        "unspecified" => Ok(RoadSurface::Unspecified),
        "paved" => Ok(RoadSurface::PAVED),
        "gravel" => Ok(RoadSurface::GRAVEL),
        _ => Err(link_py_error(
            "update_link_settings",
            "invalid roadway surface",
        )),
    }
}

/// Extracts one public outlet-head-basis enum.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
///
/// # Returns
/// Typed solver outlet head basis.
///
/// # Errors
/// Returns a validation error for an unknown spelling.
fn extract_outlet_head_basis(value: &Bound<'_, PyAny>) -> PyResult<OutletHeadBasis> {
    match link_enum_name(value, "head_basis", PublicType::OutletHeadBasis)?.as_str() {
        "depth" => Ok(OutletHeadBasis::Depth),
        "head" => Ok(OutletHeadBasis::Head),
        _ => Err(link_py_error(
            "update_link_settings",
            "invalid outlet head basis",
        )),
    }
}

/// Extracts one public standard cross-section shape enum.
///
/// # Arguments
/// * `value` - Python string-enum candidate.
///
/// # Returns
/// Typed solver cross-section shape.
///
/// # Errors
/// Returns a validation error for an unknown or reference-only shape.
fn extract_xsect_type(value: &Bound<'_, PyAny>) -> PyResult<XsectType> {
    let name = link_enum_name(value, "shape", PublicType::StandardCrossSectionShape)?;
    let kind = match name.as_str() {
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
        _ => {
            return Err(link_py_error(
                "update_link_settings",
                "invalid standard cross-section shape",
            ));
        }
    };
    Ok(kind)
}

/// Validates one related Live View against authoritative configured identity.
///
/// # Arguments
/// * `simulation` - Locked authoritative Simulation Owner.
/// * `relation` - Extracted related Live View identity.
/// * `expected` - Required object family.
/// * `name` - Public relationship path.
///
/// # Returns
/// Validated configured related-object index.
///
/// # Errors
/// Returns a family, subtype, stale-identity, or configured-storage error.
fn validate_link_relationship(
    simulation: &SwmmSimulation,
    relation: &RelatedObject,
    expected: ObjectType,
    name: &str,
) -> Result<usize, SwmmError> {
    validate_related_object(simulation, relation, &[expected], name)
}

/// Resolves one private link setting name to its typed selector.
fn link_configuration_field(name: &str) -> PyResult<LinkConfigurationField> {
    match name {
        "tag" => Ok(LinkConfigurationField::Tag),
        "inlet_node" => Ok(LinkConfigurationField::InletNode),
        "outlet_node" => Ok(LinkConfigurationField::OutletNode),
        "flow_direction" => Ok(LinkConfigurationField::FlowDirection),
        "full_depth" => Ok(LinkConfigurationField::FullDepth),
        "full_flow" => Ok(LinkConfigurationField::FullFlow),
        "included_in_report" => Ok(LinkConfigurationField::IncludedInReport),
        "inlet_offset" => Ok(LinkConfigurationField::InletOffset),
        "outlet_offset" => Ok(LinkConfigurationField::OutletOffset),
        "initial_flow" => Ok(LinkConfigurationField::InitialFlow),
        "flow_limit" => Ok(LinkConfigurationField::FlowLimit),
        "inlet_loss_coefficient" => Ok(LinkConfigurationField::InletLossCoefficient),
        "outlet_loss_coefficient" => Ok(LinkConfigurationField::OutletLossCoefficient),
        "average_loss_coefficient" => Ok(LinkConfigurationField::AverageLossCoefficient),
        "seepage_rate" => Ok(LinkConfigurationField::SeepageRate),
        "has_flap_gate" => Ok(LinkConfigurationField::HasFlapGate),
        "length" => Ok(LinkConfigurationField::ConduitLength),
        "roughness" => Ok(LinkConfigurationField::ConduitRoughness),
        "barrels" => Ok(LinkConfigurationField::ConduitBarrels),
        "cross_section" => Ok(LinkConfigurationField::ConduitCrossSection),
        "slope" => Ok(LinkConfigurationField::ConduitSlope),
        "curve" => Ok(LinkConfigurationField::PumpCurve),
        "initial_setting" => Ok(LinkConfigurationField::PumpInitialSetting),
        "startup_depth" => Ok(LinkConfigurationField::PumpStartupDepth),
        "shutoff_depth" => Ok(LinkConfigurationField::PumpShutoffDepth),
        "orifice_kind" => Ok(LinkConfigurationField::OrificeKind),
        "orifice_discharge_coefficient" => Ok(LinkConfigurationField::OrificeDischargeCoefficient),
        "opening_time_hours" => Ok(LinkConfigurationField::OrificeOpeningTimeHours),
        "weir_kind" => Ok(LinkConfigurationField::WeirKind),
        "weir_discharge_coefficient" => Ok(LinkConfigurationField::WeirDischargeCoefficient),
        "end_discharge_coefficient" => Ok(LinkConfigurationField::WeirEndDischargeCoefficient),
        "end_contractions" => Ok(LinkConfigurationField::WeirEndContractions),
        "can_surcharge" => Ok(LinkConfigurationField::WeirCanSurcharge),
        "roadway_width" => Ok(LinkConfigurationField::WeirRoadwayWidth),
        "roadway_surface" => Ok(LinkConfigurationField::WeirRoadwaySurface),
        "coefficient_curve" => Ok(LinkConfigurationField::WeirCoefficientCurve),
        "rating" => Ok(LinkConfigurationField::OutletRating),
        _ => Err(internal_error(
            "link_configuration_value",
            "invalid private link configuration field",
        )),
    }
}

/// Resolves one private link result name to its typed selector.
fn link_result_field(name: &str) -> PyResult<LinkResultField> {
    match name {
        "setting" => Ok(LinkResultField::Setting),
        "target_setting" => Ok(LinkResultField::TargetSetting),
        "time_open_seconds" => Ok(LinkResultField::TimeOpenSeconds),
        "time_closed_seconds" => Ok(LinkResultField::TimeClosedSeconds),
        "flow" => Ok(LinkResultField::Flow),
        "depth" => Ok(LinkResultField::Depth),
        "velocity" => Ok(LinkResultField::Velocity),
        "top_width" => Ok(LinkResultField::TopWidth),
        "volume" => Ok(LinkResultField::Volume),
        "capacity" => Ok(LinkResultField::Capacity),
        "upstream_surface_area" => Ok(LinkResultField::UpstreamSurfaceArea),
        "downstream_surface_area" => Ok(LinkResultField::DownstreamSurfaceArea),
        "froude_number" => Ok(LinkResultField::FroudeNumber),
        _ => Err(internal_error(
            "link_result",
            "invalid private link result field",
        )),
    }
}

/// Resolves one private inlet setting name to its typed selector.
fn inlet_configuration_field(name: &str) -> PyResult<InletConfigurationField> {
    match name {
        "count" => Ok(InletConfigurationField::Count),
        "percent_clogged" => Ok(InletConfigurationField::PercentClogged),
        "flow_limit" => Ok(InletConfigurationField::FlowLimit),
        "local_depression" => Ok(InletConfigurationField::LocalDepression),
        "local_width" => Ok(InletConfigurationField::LocalWidth),
        "design" => Ok(InletConfigurationField::Design),
        _ => Err(internal_error(
            "inlet_configuration_value",
            "invalid private inlet configuration field",
        )),
    }
}

/// Resolves one private inlet result name to its typed selector.
fn inlet_result_field(name: &str) -> PyResult<InletResultField> {
    match name {
        "flow_factor" => Ok(InletResultField::FlowFactor),
        "captured_flow" => Ok(InletResultField::CapturedFlow),
        "backflow" => Ok(InletResultField::Backflow),
        "backflow_ratio" => Ok(InletResultField::BackflowRatio),
        _ => Err(internal_error(
            "inlet_result",
            "invalid private inlet result field",
        )),
    }
}

/// Converts one targeted link setting to its private Python representation.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Owned targeted link configuration value.
///
/// # Returns
/// Selected Python-owned scalar, declaration parts, or final relationship view.
///
/// # Errors
/// Returns a conversion or relationship-construction failure.
fn link_configuration_value_to_py(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: LinkConfigurationValue,
) -> PyResult<Py<PyAny>> {
    match value {
        LinkConfigurationValue::Text(value) => value.into_py_any(py),
        LinkConfigurationValue::Number(value) => value.into_py_any(py),
        LinkConfigurationValue::Flag(value) => value.into_py_any(py),
        LinkConfigurationValue::Integer(value) => value.into_py_any(py),
        LinkConfigurationValue::Node(value) => {
            relationship_view(py, facade, generation, ObjectType::Node, value)
        }
        LinkConfigurationValue::OptionalRelationship(value) => value
            .map(|identity| relationship_view(py, facade, generation, ObjectType::Curve, identity))
            .transpose()?
            .into_py_any(py),
        LinkConfigurationValue::CrossSection(value) => {
            cross_section_parts(py, facade, generation, value)?.into_py_any(py)
        }
        LinkConfigurationValue::OrificeKind(value) => (value as i32).into_py_any(py),
        LinkConfigurationValue::WeirKind(value) => (value as i32).into_py_any(py),
        LinkConfigurationValue::RoadSurface(value) => (value as i32).into_py_any(py),
        LinkConfigurationValue::OutletRating(value) => {
            outlet_rating_parts(py, facade, generation, value)?.into_py_any(py)
        }
    }
}

/// Converts one targeted inlet setting to its private Python representation.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Owned targeted inlet configuration value.
///
/// # Returns
/// Selected Python-owned scalar or final relationship view.
///
/// # Errors
/// Returns a conversion or relationship-construction failure.
fn inlet_configuration_value_to_py(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: InletConfigurationValue,
) -> PyResult<Py<PyAny>> {
    match value {
        InletConfigurationValue::Integer(value) => value.into_py_any(py),
        InletConfigurationValue::Number(value) => value.into_py_any(py),
        InletConfigurationValue::Relationship(value) => {
            relationship_view(py, facade, generation, ObjectType::Inlet, value)
        }
    }
}

type CrossSectionParts = (
    &'static str,
    Option<i32>,
    [f64; 4],
    Option<Py<PyAny>>,
    Option<f64>,
    usize,
);

/// Converts an optional cross section to the private discriminated tuple shape.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Optional owned cross-section configuration.
///
/// # Returns
/// Optional Python-ready discriminated cross-section parts.
///
/// # Errors
/// Returns a relationship-construction failure for referenced shape objects.
fn cross_section_parts(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: Option<LinkCrossSectionValue>,
) -> PyResult<Option<CrossSectionParts>> {
    value
        .map(|value| {
            Ok(match value {
                LinkCrossSectionValue::Circular { diameter } => {
                    ("circular", None, [diameter, 0.0, 0.0, 0.0], None, None, 0)
                }
                LinkCrossSectionValue::Standard {
                    shape,
                    geometry,
                    culvert_code,
                } => (
                    "standard",
                    Some(shape as i32),
                    geometry,
                    None,
                    None,
                    culvert_code,
                ),
                LinkCrossSectionValue::Custom {
                    shape,
                    full_depth,
                    culvert_code,
                } => (
                    "custom",
                    None,
                    [0.0; 4],
                    Some(relationship_view(
                        py,
                        facade,
                        generation,
                        ObjectType::Shape,
                        shape,
                    )?),
                    Some(full_depth),
                    culvert_code,
                ),
                LinkCrossSectionValue::Irregular {
                    transect,
                    culvert_code,
                } => (
                    "irregular",
                    None,
                    [0.0; 4],
                    Some(relationship_view(
                        py,
                        facade,
                        generation,
                        ObjectType::Transect,
                        transect,
                    )?),
                    None,
                    culvert_code,
                ),
                LinkCrossSectionValue::Street {
                    street,
                    culvert_code,
                } => (
                    "street",
                    None,
                    [0.0; 4],
                    Some(relationship_view(
                        py,
                        facade,
                        generation,
                        ObjectType::Street,
                        street,
                    )?),
                    None,
                    culvert_code,
                ),
            })
        })
        .transpose()
}

type OutletRatingParts = (
    &'static str,
    Option<f64>,
    Option<f64>,
    Option<Py<PyAny>>,
    &'static str,
);

/// Converts one outlet rating to the private discriminated tuple shape.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade used for relationship results.
/// * `generation` - Captured Project Generation.
/// * `value` - Owned outlet-rating configuration.
///
/// # Returns
/// Python-ready discriminated outlet-rating parts.
///
/// # Errors
/// Returns a relationship-construction failure for referenced curves.
fn outlet_rating_parts(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    value: LinkOutletRatingValue,
) -> PyResult<OutletRatingParts> {
    Ok(match value {
        LinkOutletRatingValue::Functional {
            coefficient,
            exponent,
            head_basis,
        } => (
            "functional",
            Some(coefficient),
            Some(exponent),
            None,
            outlet_head_basis_name(head_basis),
        ),
        LinkOutletRatingValue::Tabular { curve, head_basis } => (
            "tabular",
            None,
            None,
            Some(relationship_view(
                py,
                facade,
                generation,
                ObjectType::Curve,
                curve,
            )?),
            outlet_head_basis_name(head_basis),
        ),
    })
}

/// Returns the stable private string for one outlet head basis.
fn outlet_head_basis_name(value: OutletHeadBasis) -> &'static str {
    match value {
        OutletHeadBasis::Depth => "depth",
        OutletHeadBasis::Head => "head",
    }
}
