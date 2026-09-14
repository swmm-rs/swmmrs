use super::*;

#[pymethods]
impl NativeSimulation {
    /// Opens and validates the first project generation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `input` - Raw input-file `str | os.PathLike[str]` value.
    /// * `report` - Optional raw report-file path.
    /// * `output` - Optional raw retained output path; `None` requests scratch output.
    ///
    /// # Returns
    /// A synchronized owner in the open lifecycle state.
    ///
    /// # Errors
    /// Returns a native exception when the project cannot open or native execution panics.
    #[new]
    #[pyo3(signature = (input, report=None, output=None))]
    fn new(
        py: Python<'_>,
        input: &Bound<'_, PyAny>,
        report: Option<&Bound<'_, PyAny>>,
        output: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let paths = project_paths(py, input, report, output)?;
        let result = py.detach(move || {
            match catch_unwind(AssertUnwindSafe(|| {
                let mut handle = HandleState::default();
                handle
                    .open_project(paths)
                    .map_err(|error| DetachedFailure::Solver("open", error))?;
                Ok(Self {
                    handle: Mutex::new(handle),
                    iterator_state: AtomicIteratorState::idle(),
                    #[cfg(feature = "test-support")]
                    test_concurrency: TestConcurrencyState::default(),
                })
            })) {
                Ok(result) => result,
                Err(_) => Err(DetachedFailure::Panicked),
            }
        });
        result.map_err(|failure| detached_error("open", failure))
    }

    /// Opens another project generation on a closed owner.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `input` - Raw input-file `str | os.PathLike[str]` value.
    /// * `report` - Optional raw report-file path.
    /// * `output` - Optional raw retained output path; `None` requests scratch output.
    ///
    /// # Returns
    /// `Ok(())` after the new Project Generation is installed.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or opening fails.
    #[pyo3(signature = (input, report=None, output=None))]
    fn open(
        &self,
        py: Python<'_>,
        input: &Bound<'_, PyAny>,
        report: Option<&Bound<'_, PyAny>>,
        output: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        self.attached(py, "open", |handle| {
            if handle.simulation.lifecycle_read().state != SimulationLifecycle::Closed {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotOpen,
                    "open requires a closed simulation",
                ));
            }
            Ok(())
        })?;
        let paths = project_paths(py, input, report, output)?;
        self.ensure_not_iterating("open")?;
        let iterator_state = &self.iterator_state;
        self.detached(py, "open", move |handle| {
            handle.open_project(paths)?;
            iterator_state.store(IteratorState::Idle);
            Ok(())
        })
    }

    /// Returns the retained absolute input path from the last successful open.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Absolute input path string.
    ///
    /// # Errors
    /// Returns a native exception when no successful project metadata exists.
    #[getter]
    fn input_path(&self, py: Python<'_>) -> PyResult<String> {
        self.attached(py, "input_path", |handle| {
            handle
                .paths
                .as_ref()
                .map(|paths| paths.input.clone())
                .ok_or_else(|| SwmmError::with_detail(ErrorCode::System, "owner paths unavailable"))
        })
    }

    /// Returns the retained absolute report path from the last successful open.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Absolute report path string.
    ///
    /// # Errors
    /// Returns a native exception when no successful project metadata exists.
    #[getter]
    fn report_path(&self, py: Python<'_>) -> PyResult<String> {
        self.attached(py, "report_path", |handle| {
            handle
                .paths
                .as_ref()
                .map(|paths| paths.report.clone())
                .ok_or_else(|| SwmmError::with_detail(ErrorCode::System, "owner paths unavailable"))
        })
    }

    /// Returns the retained absolute output path, or `None` for scratch output.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Optional absolute retained output path string.
    ///
    /// # Errors
    /// Returns a native exception when no successful project metadata exists.
    #[getter]
    fn output_path(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.attached(py, "output_path", |handle| {
            handle
                .paths
                .as_ref()
                .map(|paths| paths.output.clone())
                .ok_or_else(|| SwmmError::with_detail(ErrorCode::System, "owner paths unavailable"))
        })
    }

    /// Returns the authoritative owner lifecycle state.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Stable lowercase lifecycle name.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn state(&self, py: Python<'_>) -> PyResult<&'static str> {
        self.attached(py, "state", |handle| {
            Ok(match handle.simulation.lifecycle_read().state {
                SimulationLifecycle::Open => "open",
                SimulationLifecycle::Running => "running",
                SimulationLifecycle::Complete => "complete",
                SimulationLifecycle::Ended => "ended",
                SimulationLifecycle::Failed => "failed",
                SimulationLifecycle::Closed => "closed",
            })
        })
    }

    /// Returns lifecycle and progress state from one owner acquisition.
    fn status(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let (lifecycle, current_time) = self.attached(py, "status", |handle| {
            let lifecycle = handle.simulation.lifecycle_read();
            let current_time = if matches!(
                lifecycle.state,
                SimulationLifecycle::Running
                    | SimulationLifecycle::Complete
                    | SimulationLifecycle::Ended
            ) {
                Some(datetime_parts(handle.simulation.current_time_read()?))
            } else {
                None
            };
            Ok((lifecycle, current_time))
        })?;
        let elapsed_seconds = if current_time.is_some() {
            (lifecycle.routing_time / 1000.0).max(0.0)
        } else {
            0.0
        };
        let duration_seconds = (lifecycle.total_duration / 1000.0).max(0.0);
        let percent_complete = if duration_seconds > 0.0 {
            (100.0 * elapsed_seconds / duration_seconds).clamp(0.0, 100.0)
        } else if matches!(
            lifecycle.state,
            SimulationLifecycle::Complete | SimulationLifecycle::Ended
        ) {
            100.0
        } else {
            0.0
        };
        let state = match lifecycle.state {
            SimulationLifecycle::Open => "open",
            SimulationLifecycle::Running => "running",
            SimulationLifecycle::Complete => "complete",
            SimulationLifecycle::Ended => "ended",
            SimulationLifecycle::Failed => "failed",
            SimulationLifecycle::Closed => "closed",
        };
        let result = PyDict::new(py);
        result.set_item("state", state)?;
        result.set_item("is_open", lifecycle.is_open)?;
        result.set_item("is_started", lifecycle.is_started)?;
        result.set_item("configuration_dirty", lifecycle.configuration_dirty)?;
        result.set_item("save_results", lifecycle.save_results)?;
        result.set_item("current_time", current_time)?;
        result.set_item("elapsed_seconds", elapsed_seconds)?;
        result.set_item("duration_seconds", duration_seconds)?;
        result.set_item("percent_complete", percent_complete)?;
        result.set_item("step_count", lifecycle.total_step_count)?;
        result.set_item("report_period_count", lifecycle.report_period_count)?;
        result.set_item("warning_count", lifecycle.warning_count)?;
        Ok(result.unbind())
    }

    /// Returns whether a Project Generation remains open.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `true` when the active identity matches the owner's checked generation counter.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn is_open(&self, py: Python<'_>) -> PyResult<bool> {
        self.attached(py, "is_open", |handle| {
            Ok(handle.active_generation == Some(ProjectGeneration(handle.generation_counter)))
        })
    }

    /// Returns the active checked Project Generation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Active owner-local generation number.
    ///
    /// # Errors
    /// Returns a lifecycle or internal native failure when no healthy project is active.
    #[getter]
    fn generation(&self, py: Python<'_>) -> PyResult<u64> {
        self.attached(py, "generation", |handle| {
            if handle.simulation.lifecycle_read().state == SimulationLifecycle::Failed {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotOpen,
                    "failed project generation is unavailable",
                ));
            }
            handle
                .active_generation
                .map(|generation| generation.0)
                .ok_or_else(|| {
                    SwmmError::with_detail(ErrorCode::ApiNotOpen, "no active project generation")
                })
        })
    }

    /// Resolves one collection key and constructs its final public Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    /// * `key` - Requested string or exact non-boolean integer key.
    ///
    /// # Returns
    /// Fresh generation-bound public Live View.
    ///
    /// # Errors
    /// Returns a type, key, stale, lifecycle, owner-read, construction, mutex, or panic failure.
    fn collection_view(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        family: u8,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let object_type = object_type(family)?;
        let requested = key.clone().unbind();
        let key = collection_key(key)?.ok_or_else(|| {
            PyTypeError::new_err("collection keys must be str or non-boolean int")
        })?;
        let identity = self.checked_generation(py, "collection_view", generation, |handle| {
            handle
                .simulation
                .object_identity_read(object_type, &key)
                .map_err(|error| DetachedFailure::Solver("collection_view", error))
        })?;
        let identity = identity.ok_or_else(|| PyKeyError::new_err(requested))?;
        relationship_view(py, facade, generation, object_type, identity)
    }

    /// Resolves one zero-based collection position and constructs its final public Live View.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `facade` - Owning public `Simulation` facade.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    /// * `position` - Requested exact non-boolean integer position.
    ///
    /// # Returns
    /// Fresh generation-bound public Live View.
    ///
    /// # Errors
    /// Returns a type, index, stale, lifecycle, owner-read, construction, mutex, or panic failure.
    fn collection_view_at(
        &self,
        py: Python<'_>,
        facade: &Bound<'_, PyAny>,
        generation: u64,
        family: u8,
        position: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        if !position.get_type().is(py.get_type::<PyInt>()) {
            return Err(PyTypeError::new_err("index must be a non-boolean int"));
        }
        let requested = position.clone().unbind();
        if position.lt(0)? {
            return Err(PyIndexError::new_err(requested));
        }
        let Ok(position) = position.extract::<usize>() else {
            return Err(PyIndexError::new_err(requested));
        };
        let object_type = object_type(family)?;
        let identity = self.checked_generation(py, "collection_view_at", generation, |handle| {
            handle
                .simulation
                .object_identity_at_read(object_type, position)
                .map_err(|error| DetachedFailure::Solver("collection_view_at", error))
        })?;
        let identity = identity.ok_or_else(|| PyIndexError::new_err(requested))?;
        relationship_view(py, facade, generation, object_type, identity)
    }

    /// Returns whether a valid candidate key identifies a configured object.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    /// * `key` - Candidate collection key.
    ///
    /// # Returns
    /// `true` when the accepted key resolves in the active generation; otherwise `false`.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, conversion, mutex, or panic failure.
    fn collection_contains(
        &self,
        py: Python<'_>,
        generation: u64,
        family: u8,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        let Some(key) = collection_key(key)? else {
            return Ok(false);
        };
        let object_type = object_type(family)?;
        self.checked_generation(py, "collection_contains", generation, |handle| {
            handle
                .simulation
                .object_identity_read(object_type, &key)
                .map(|identity| identity.is_some())
                .map_err(|error| DetachedFailure::Solver("collection_contains", error))
        })
    }

    /// Returns one family's configured logical count.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    ///
    /// # Returns
    /// Configured logical family count.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, mutex, or panic failure.
    fn collection_count(&self, py: Python<'_>, generation: u64, family: u8) -> PyResult<usize> {
        let object_type = object_type(family)?;
        self.checked_generation(py, "collection_count", generation, |handle| {
            handle
                .simulation
                .object_count_read(object_type)
                .map_err(|error| DetachedFailure::Solver("collection_count", error))
        })
    }

    /// Copies canonical IDs in configured logical order.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    ///
    /// # Returns
    /// Owned canonical identifiers in configured logical order.
    ///
    /// # Errors
    /// Returns a stale, lifecycle, owner-read, mutex, or panic failure.
    fn collection_ids(&self, py: Python<'_>, generation: u64, family: u8) -> PyResult<Vec<String>> {
        let object_type = object_type(family)?;
        self.checked_generation(py, "collection_ids", generation, |handle| {
            handle
                .simulation
                .object_identity_indexes_read(object_type)
                .map(|identities| identities.into_iter().map(|identity| identity.id).collect())
                .map_err(|error| DetachedFailure::Solver("collection_ids", error))
        })
    }

    /// Revalidates one captured Live View identity under the owner lock.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation.
    /// * `family` - Private configured-object family ordinal.
    /// * `index` - Captured native family index.
    /// * `id` - Captured canonical project identifier.
    /// * `subtype` - Captured concrete subtype label.
    ///
    /// # Returns
    /// `Ok(())` only while the complete typed identity remains active.
    ///
    /// # Errors
    /// Returns a stale or internal native failure.
    fn validate_identity(
        &self,
        py: Python<'_>,
        generation: u64,
        family: u8,
        index: usize,
        id: &str,
        subtype: &str,
    ) -> PyResult<()> {
        let object_type = object_type(family)?;
        let subtype = object_subtype(object_type, subtype)
            .ok_or_else(|| internal_error("validate_identity", "invalid private subtype"))?;
        self.checked_generation(py, "validate_identity", generation, |handle| {
            match handle
                .simulation
                .object_identity_matches(object_type, index, id, subtype)
                .map_err(|error| DetachedFailure::Solver("validate_identity", error))?
            {
                true => Ok(()),
                false => Err(DetachedFailure::Stale),
            }
        })
    }

    /// Returns whether stepping is active or naturally complete.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `true` in running or naturally complete states.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn is_started(&self, py: Python<'_>) -> PyResult<bool> {
        self.attached(py, "is_started", |handle| {
            Ok(handle.simulation.lifecycle_read().is_started)
        })
    }

    /// Returns whether accepted configuration requires post-open preparation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `true` when `start()` must prepare edited configuration.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn configuration_dirty(&self, py: Python<'_>) -> PyResult<bool> {
        self.attached(py, "configuration_dirty", |handle| {
            Ok(handle.simulation.lifecycle_read().configuration_dirty)
        })
    }

    /// Returns the accumulated solver warning count.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Number of warnings accumulated by the current project.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn warning_count(&self, py: Python<'_>) -> PyResult<i32> {
        self.attached(py, "warning_count", |handle| {
            Ok(handle.simulation.lifecycle_read().warning_count)
        })
    }

    /// Returns the number of saved report periods.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Number of binary report-period records saved by the current run.
    ///
    /// # Errors
    /// Returns a native exception when the owner mutex is poisoned.
    #[getter]
    fn report_period_count(&self, py: Python<'_>) -> PyResult<i64> {
        self.attached(py, "report_period_count", |handle| {
            handle.simulation.report_period_count_read()
        })
    }

    /// Copies one selected configured simulation date.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `field` - Private configured-time field name.
    ///
    /// # Returns
    /// Selected date-time parts at whole-second resolution.
    ///
    /// # Errors
    /// Returns a native exception for an invalid field or unhealthy owner.
    fn simulation_time(&self, py: Python<'_>, field: &str) -> PyResult<DateTimeParts> {
        self.attached(py, "simulation_time", |handle| {
            let field = match field {
                "start_time" => SimulationTimeField::Start,
                "report_start" => SimulationTimeField::ReportStart,
                "end_time" => SimulationTimeField::End,
                _ => {
                    // Preserve lifecycle-error precedence for malformed private calls.
                    handle.simulation.simulation_time_read()?;
                    return Err(SwmmError::with_detail(
                        ErrorCode::System,
                        "invalid private simulation time field",
                    ));
                }
            };
            handle
                .simulation
                .simulation_time_value_read(field)
                .map(datetime_parts)
        })
    }

    /// Copies the current model time.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Current model date and time at whole-second resolution.
    ///
    /// # Errors
    /// Returns a native exception outside the supported run lifecycle.
    fn current_time(&self, py: Python<'_>) -> PyResult<DateTimeParts> {
        self.attached(py, "current_time", |handle| {
            handle.simulation.current_time_read().map(datetime_parts)
        })
    }
    /// Copies current and start model times under one owner acquisition.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Current and configured start date-time parts at whole-second resolution.
    ///
    /// # Errors
    /// Returns a native exception outside the supported run lifecycle.
    fn elapsed_times(&self, py: Python<'_>) -> PyResult<(DateTimeParts, DateTimeParts)> {
        self.attached(py, "elapsed_time", |handle| {
            let (current, start) = handle.simulation.elapsed_time_read()?;
            Ok((datetime_parts(current), datetime_parts(start)))
        })
    }

    /// Copies one selected stable unit label.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `field` - Private unit-label field name.
    ///
    /// # Returns
    /// Selected stable unit label.
    ///
    /// # Errors
    /// Returns a native exception for an invalid field or unhealthy owner.
    fn unit_label(&self, py: Python<'_>, field: &str) -> PyResult<&'static str> {
        self.attached(py, "unit_label", |handle| {
            let (flow_units, unit_system) = handle.simulation.units_read()?;
            match field {
                "flow_units" => Ok(flow_units_name(flow_units)),
                "unit_system" => Ok(unit_system_name(unit_system)),
                _ => Err(SwmmError::with_detail(
                    ErrorCode::System,
                    "invalid private unit label field",
                )),
            }
        })
    }

    /// Copies every supported option for aggregate inspection.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation to revalidate.
    ///
    /// # Returns
    /// Python dictionary containing every supported option in public units.
    ///
    /// # Errors
    /// Returns a native exception for a stale generation or unhealthy owner.
    fn options_read(&self, py: Python<'_>, generation: u64) -> PyResult<Py<PyDict>> {
        let values = self.checked_generation(py, "options_read", generation, |handle| {
            handle
                .simulation
                .simulation_options_read()
                .map_err(|error| DetachedFailure::Solver("options_read", error))
        })?;
        options_dict(py, values)
    }

    /// Copies one selected option for a Project Generation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation to revalidate.
    /// * `field` - Private option field name.
    ///
    /// # Returns
    /// Selected Python-owned option value in public units.
    ///
    /// # Errors
    /// Returns a native exception for a stale generation, invalid field, or unhealthy owner.
    fn option_value(&self, py: Python<'_>, generation: u64, field: &str) -> PyResult<Py<PyAny>> {
        let value = self.checked_generation(py, "option_value", generation, |handle| {
            let field = match field {
                "routing_step" => SimulationOptionField::RoutingStep,
                "report_step" => SimulationOptionField::ReportStep,
                "detailed_reporting_enabled" => SimulationOptionField::DetailedReportingEnabled,
                "custom_ellipse_model" => SimulationOptionField::CustomEllipseModel,
                "surcharge_method" => SimulationOptionField::SurchargeMethod,
                "allow_ponding" => SimulationOptionField::AllowPonding,
                "inertia_damping" => SimulationOptionField::InertiaDamping,
                "normal_flow_limit" => SimulationOptionField::NormalFlowLimit,
                "skip_steady_state" => SimulationOptionField::SkipSteadyState,
                "rainfall_enabled" => SimulationOptionField::RainfallEnabled,
                "rdii_enabled" => SimulationOptionField::RdiiEnabled,
                "snowmelt_enabled" => SimulationOptionField::SnowmeltEnabled,
                "groundwater_enabled" => SimulationOptionField::GroundwaterEnabled,
                "routing_enabled" => SimulationOptionField::RoutingEnabled,
                "quality_enabled" => SimulationOptionField::QualityEnabled,
                "rule_step" => SimulationOptionField::RuleStep,
                "sweep_start" => SimulationOptionField::SweepStart,
                "sweep_end" => SimulationOptionField::SweepEnd,
                "maximum_trials" => SimulationOptionField::MaximumTrials,
                "requested_threads" => SimulationOptionField::RequestedThreads,
                "minimum_routing_step" => SimulationOptionField::MinimumRoutingStep,
                "lengthening_step" => SimulationOptionField::LengtheningStep,
                "antecedent_dry_duration" => SimulationOptionField::AntecedentDryDuration,
                "courant_factor" => SimulationOptionField::CourantFactor,
                "minimum_surface_area" => SimulationOptionField::MinimumSurfaceArea,
                "minimum_conduit_slope" => SimulationOptionField::MinimumConduitSlope,
                "head_tolerance" => SimulationOptionField::HeadTolerance,
                "system_flow_tolerance" => SimulationOptionField::SystemFlowTolerance,
                "lateral_flow_tolerance" => SimulationOptionField::LateralFlowTolerance,
                _ => {
                    // Preserve lifecycle-error precedence for malformed private calls.
                    handle
                        .simulation
                        .simulation_options_read()
                        .map_err(|error| DetachedFailure::Solver("option_value", error))?;
                    return Err(DetachedFailure::Solver(
                        "option_value",
                        SwmmError::with_detail(ErrorCode::System, "invalid private option field"),
                    ));
                }
            };
            handle
                .simulation
                .simulation_option_value_read(field)
                .map_err(|error| DetachedFailure::Solver("option_value", error))
        })?;
        option_value(py, value)
    }

    /// Returns the Effective Thread Count retained after project caps.
    #[getter]
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Effective Thread Count after native project caps.
    ///
    /// # Errors
    /// Returns a native exception after owner failure or close.
    fn effective_threads(&self, py: Python<'_>) -> PyResult<i32> {
        self.attached(py, "effective_threads", |handle| {
            handle.simulation.effective_thread_count_read()
        })
    }

    /// Evaluates the native current maximum routing step.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `generation` - Captured Project Generation to revalidate.
    ///
    /// # Returns
    /// Current native maximum routing step in seconds.
    ///
    /// # Errors
    /// Returns a native exception for a stale generation or unhealthy owner.
    fn maximum_routing_step(&self, py: Python<'_>, generation: u64) -> PyResult<f64> {
        self.detached_checked_generation(py, "maximum_routing_step", generation, |handle| {
            handle.simulation.maximum_routing_step_read()
        })
    }

    /// Atomically updates the coupled simulation schedule from raw Python values.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation to revalidate.
    /// * `changes` - Raw sparse schedule mapping.
    ///
    /// # Errors
    /// Returns a validation, stale-generation, lifecycle, mutex, or panic failure.
    fn update_schedule(
        &self,
        py: Python<'_>,
        generation: u64,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let patch = extract_schedule_patch(changes)?;
        self.detached_checked_generation(py, "update_schedule", generation, move |handle| {
            handle.simulation.patch_simulation_schedule(patch)
        })
    }

    /// Atomically updates simulation policy from raw Python values.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token released during mutation.
    /// * `generation` - Captured Project Generation to revalidate.
    /// * `changes` - Raw sparse option mapping.
    ///
    /// # Errors
    /// Returns a validation, stale-generation, lifecycle, mutex, or panic failure.
    fn update_options(
        &self,
        py: Python<'_>,
        generation: u64,
        changes: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let patch = extract_options_patch(changes)?;
        self.detached_checked_generation(py, "update_options", generation, move |handle| {
            handle.simulation.patch_simulation_options(patch)
        })
    }

    /// Starts the open project.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `save_results` - Whether report-period results are saved.
    ///
    /// # Returns
    /// `Ok(())` after the run starts.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or start fails.
    fn start(&self, py: Python<'_>, save_results: &Bound<'_, PyAny>) -> PyResult<()> {
        let save_results = exact_bool(py, save_results, "save_results")?;
        self.ensure_not_iterating("start")?;
        match self.detached(py, "start", move |handle| {
            handle.simulation.start(save_results)?;
            handle.iterator_exhausted = false;
            Ok(())
        }) {
            Ok(()) => Ok(()),
            Err(error) => {
                let diagnostics = self.attached(py, "start", |handle| {
                    Ok(handle.simulation.configuration_diagnostics_read().to_vec())
                })?;
                if diagnostics.is_empty() {
                    Err(error)
                } else {
                    Err(configuration_error(diagnostics))
                }
            }
        }
    }

    /// Resets stale solver state without reparsing the opened project.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `Ok(())` after the owner is open with retained configuration.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or reset is invalid.
    fn reset_solver(&self, py: Python<'_>) -> PyResult<()> {
        self.ensure_not_iterating("reset_solver")?;
        self.detached(py, "reset_solver", |handle| {
            handle.simulation.reset_solver()?;
            handle.iterator_exhausted = false;
            Ok(())
        })
    }

    /// Transitions Dynamic Wave workers to low-power sleep.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `Ok(())` after active Dynamic Wave workers enter their sleep state.
    ///
    /// # Errors
    /// Returns a native exception when the owner is unavailable.
    fn sleep_workers(&self, py: Python<'_>) -> PyResult<()> {
        self.detached(py, "sleep_workers", move |handle| {
            handle.simulation.sleep_dynamic_wave_workers()
        })
    }

    /// Configures or clears persistent hotstart input for this Project Generation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `path` - Raw hotstart input path, or `None` to clear it.
    ///
    /// # Returns
    /// `Ok(())` after the owner configuration is replaced.
    ///
    /// # Errors
    /// Returns a native exception outside the configuration lifecycle or on managed-file failure.
    #[pyo3(signature = (path=None))]
    fn use_hotstart(&self, py: Python<'_>, path: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        self.attached(py, "use_hotstart", |handle| {
            if !matches!(
                handle.simulation.lifecycle_read().state,
                SimulationLifecycle::Open | SimulationLifecycle::Ended
            ) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotEnded,
                    "persistent hotstart input is configurable only while open or ended",
                ));
            }
            Ok(())
        })?;
        let path = path
            .map(|path| absolute_path(py, path, "path"))
            .transpose()?;
        let key = path.as_deref().map(|path| path_key(py, path)).transpose()?;
        self.ensure_not_iterating("use_hotstart")?;
        self.detached(py, "use_hotstart", move |handle| {
            let paths = handle.paths.as_ref().ok_or_else(|| {
                SwmmError::with_detail(ErrorCode::System, "owner paths unavailable")
            })?;
            if let Some((path, _)) = path
                .as_deref()
                .zip(key.as_deref())
                .filter(|(_, key)| paths.collides(key, false))
            {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiPropertyValue,
                    format!("hotstart input must not collide with an owner path: {path}"),
                ));
            }
            handle.simulation.use_hotstart(path.as_deref())?;
            let paths = handle.paths.as_mut().ok_or_else(|| {
                SwmmError::with_detail(ErrorCode::System, "owner paths unavailable")
            })?;
            paths.hotstart = path;
            paths.hotstart_key = key;
            Ok(())
        })
    }

    /// Saves a deterministic Simulation Checkpoint at the current quiescent boundary.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `path` - Raw fresh checkpoint manifest path.
    ///
    /// # Errors
    /// Returns a native exception when checkpoint publication fails.
    fn save_checkpoint(&self, py: Python<'_>, path: &Bound<'_, PyAny>) -> PyResult<()> {
        let path = absolute_path(py, path, "path")?;
        let key = path_key(py, &path)?;
        self.detached(py, "save_checkpoint", move |handle| {
            let paths = handle.paths.as_ref().ok_or_else(|| {
                SwmmError::with_detail(ErrorCode::System, "owner paths unavailable")
            })?;
            if paths.collides(&key, true) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiPropertyValue,
                    format!("checkpoint must not collide with an owner path: {path}"),
                ));
            }
            handle.simulation.save_checkpoint(&path)
        })
    }

    /// Resumes a Simulation Owner from an immutable Simulation Checkpoint.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `checkpoint` - Raw existing checkpoint manifest path.
    /// * `report` - Raw fresh report destination.
    /// * `output` - Raw fresh binary-output destination.
    ///
    /// # Returns
    /// A fresh synchronized owner restored to the checkpoint lifecycle boundary.
    ///
    /// # Errors
    /// Returns a native exception when validation, reconstruction, or destination publication fails.
    #[staticmethod]
    fn resume(
        py: Python<'_>,
        checkpoint: &Bound<'_, PyAny>,
        report: &Bound<'_, PyAny>,
        output: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let checkpoint = absolute_path(py, checkpoint, "checkpoint_path")?;
        let report = absolute_path(py, report, "report_path")?;
        let output = absolute_path(py, output, "output_path")?;
        let prepared = py
            .detach({
                let checkpoint = checkpoint.clone();
                move || {
                    catch_unwind(AssertUnwindSafe(|| {
                        SwmmSimulation::prepare_resume(&checkpoint)
                            .map_err(|error| DetachedFailure::Solver("resume", error))
                    }))
                    .unwrap_or(Err(DetachedFailure::Panicked))
                }
            })
            .map_err(|failure| detached_error("resume", failure))?;
        let input = prepared
            .input_path()
            .map(str::to_owned)
            .unwrap_or_else(|| checkpoint.clone());
        let checkpoint_key = path_key(py, &checkpoint)?;
        let input_key = path_key(py, &input)?;
        let report_key = path_key(py, &report)?;
        let output_key = path_key(py, &output)?;
        if checkpoint_key == report_key
            || checkpoint_key == output_key
            || input_key == report_key
            || input_key == output_key
            || report_key == output_key
        {
            return Err(validation_error(&format!(
                "checkpoint input, checkpoint, report, and output paths must have distinct destinations: attempted input {input}, checkpoint {checkpoint}, report {report}, output {output}"
            )));
        }
        let detached_report = report.clone();
        let detached_output = output.clone();
        let result = py.detach(move || {
            catch_unwind(AssertUnwindSafe(|| {
                SwmmSimulation::resume_prepared(prepared, &detached_report, &detached_output)
                    .map_err(|error| DetachedFailure::Solver("resume", error))
            }))
            .unwrap_or(Err(DetachedFailure::Panicked))
        });
        let simulation = result.map_err(|failure| detached_error("resume", failure))?;
        let paths = retained_paths(py, input, report, Some(output))?;
        Ok(Self::from_simulation(simulation, paths))
    }

    /// Imports checkpoint physical continuation state into this open owner.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `path` - Raw existing checkpoint manifest path.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or State Load fails atomically.
    fn load_checkpoint_state(&self, py: Python<'_>, path: &Bound<'_, PyAny>) -> PyResult<()> {
        let path = absolute_path(py, path, "checkpoint_path")?;
        self.ensure_not_iterating("load_checkpoint_state")?;
        self.detached(py, "load_checkpoint_state", move |handle| {
            handle.simulation.load_checkpoint_state(&path)
        })
    }

    /// Forks this owner into an independent in-process child.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `report` - Raw fresh child report destination.
    /// * `output` - Raw fresh child binary-output destination.
    ///
    /// # Returns
    /// A fresh synchronized child at the same lifecycle boundary.
    ///
    /// # Errors
    /// Returns a native exception when child reconstruction fails.
    fn fork(
        &self,
        py: Python<'_>,
        report: &Bound<'_, PyAny>,
        output: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let report = absolute_path(py, report, "report_path")?;
        let output = absolute_path(py, output, "output_path")?;
        let report_key = path_key(py, &report)?;
        let output_key = path_key(py, &output)?;
        if report_key == output_key {
            return Err(validation_error(
                "fork report_path and output_path must be distinct",
            ));
        }
        let child_report = report.clone();
        let child_output = output.clone();
        let (simulation, input) = self.detached(py, "fork", move |handle| {
            let paths = handle.paths.as_ref().ok_or_else(|| {
                SwmmError::with_detail(ErrorCode::System, "owner paths unavailable")
            })?;
            if paths.collides(&report_key, true) || paths.collides(&output_key, true) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiPropertyValue,
                    format!(
                        "fork destinations must not collide with parent owner paths: report {child_report}, output {child_output}"
                    ),
                ));
            }
            let input = paths.input.clone();
            handle
                .simulation
                .fork(&child_report, &child_output)
                .map(|simulation| (simulation, input))
        })?;
        let paths = retained_paths(py, input, report, Some(output))?;
        Ok(Self::from_simulation(simulation, paths))
    }

    /// Writes the current checkpoint and closes its output artifact.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `path` - Raw hotstart output path.
    ///
    /// # Returns
    /// `Ok(())` only after the complete artifact is flushed and closed.
    ///
    /// # Errors
    /// Returns a native exception outside the checkpoint lifecycle or on file failure.
    fn save_hotstart(&self, py: Python<'_>, path: &Bound<'_, PyAny>) -> PyResult<()> {
        self.attached(py, "save_hotstart", |handle| {
            if !matches!(
                handle.simulation.lifecycle_read().state,
                SimulationLifecycle::Running | SimulationLifecycle::Complete
            ) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotStarted,
                    "hotstart output is available only while running or complete",
                ));
            }
            Ok(())
        })?;
        let path = absolute_path(py, path, "path")?;
        let key = path_key(py, &path)?;
        self.detached(py, "save_hotstart", move |handle| {
            let paths = handle.paths.as_ref().ok_or_else(|| {
                SwmmError::with_detail(ErrorCode::System, "owner paths unavailable")
            })?;
            if paths.collides(&key, true) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiPropertyValue,
                    format!("hotstart output must not collide with an owner path: {path}"),
                ));
            }
            handle.simulation.save_hotstart(&path)
        })
    }

    /// Advances one routing step outside iterator ownership.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Model date/time components, or `None` at natural completion.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or stepping fails.
    fn step(&self, py: Python<'_>) -> PyResult<Option<DateTimeParts>> {
        let iterator_state = &self.iterator_state;
        let model_time = self.detached(py, "step", move |handle| {
            if iterator_state.load() != Some(IteratorState::Idle) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotStarted,
                    "iterator owns advancement",
                ));
            }
            let result = handle.simulation.step();
            if matches!(result, Ok(None)) {
                handle.iterator_exhausted = true;
            }
            result
        })?;
        Ok(model_time.map(datetime_parts))
    }

    /// Advances by a positive whole-second stride outside iterator ownership.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `duration` - Raw positive whole-second duration.
    /// * `strict` - Whether to end exactly at the requested stride interval.
    ///
    /// # Returns
    /// Model date/time components, or `None` at natural completion.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or stride fails.
    fn stride(
        &self,
        py: Python<'_>,
        duration: &Bound<'_, PyAny>,
        strict: &Bound<'_, PyAny>,
    ) -> PyResult<Option<DateTimeParts>> {
        let strict = exact_bool(py, strict, "strict")?;
        let stride_seconds = whole_seconds(py, duration)?;
        let iterator_state = &self.iterator_state;
        let model_time = self.detached(py, "stride", move |handle| {
            if iterator_state.load() != Some(IteratorState::Idle) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotStarted,
                    "iterator owns advancement",
                ));
            }
            let result = handle.simulation.stride(stride_seconds, strict);
            if matches!(result, Ok(None)) {
                handle.iterator_exhausted = true;
            }
            result
        })?;
        Ok(model_time.map(datetime_parts))
    }

    /// Sets or clears the persistent cadence used by ordinary iteration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `duration` - Positive whole-second duration, or `None` for ordinary routing steps.
    /// * `strict` - Whether to end exactly at each interval; ignored when duration is `None`.
    ///
    /// # Errors
    /// Returns a validation or lifecycle error for invalid cadence configuration.
    fn step_advance(
        &self,
        py: Python<'_>,
        duration: Option<&Bound<'_, PyAny>>,
        strict: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let strict = exact_bool(py, strict, "strict")?;
        let seconds = duration
            .map(|duration| whole_seconds(py, duration))
            .transpose()?;
        self.detached(py, "step_advance", move |handle| {
            let state = handle.simulation.lifecycle_read().state;
            if !matches!(
                state,
                SimulationLifecycle::Open
                    | SimulationLifecycle::Running
                    | SimulationLifecycle::Ended
            ) {
                return Err(SwmmError::with_detail(
                    ErrorCode::ApiNotEnded,
                    format!("step_advance is invalid while simulation is {state:?}"),
                ));
            }
            handle.advance_stride = seconds.map(|seconds| (seconds, strict));
            Ok(())
        })
    }

    /// Lazily starts and advances the exclusively owned Python iterator.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Model date/time components, or `None` when the iterator is exhausted.
    ///
    /// # Errors
    /// Returns a native exception when lazy start, advancement, or termination cleanup fails.
    fn iterator_next(&self, py: Python<'_>) -> PyResult<Option<DateTimeParts>> {
        let iterator_state = &self.iterator_state;
        let model_time = self.detached(py, "next", move |handle| {
            if handle.iterator_exhausted {
                return Ok(None);
            }
            let advance_stride = handle.advance_stride;
            let result = (|| {
                let simulation = &mut handle.simulation;
                if iterator_state.load() == Some(IteratorState::Idle) {
                    match simulation.lifecycle_read().state {
                        SimulationLifecycle::Open | SimulationLifecycle::Ended => {
                            simulation.start(true)?;
                        }
                        SimulationLifecycle::Running => {}
                        SimulationLifecycle::Complete => return Ok(None),
                        SimulationLifecycle::Failed | SimulationLifecycle::Closed => {
                            return simulation.step();
                        }
                    }
                    iterator_state.store(IteratorState::Iterating);
                }

                if iterator_state.load() == Some(IteratorState::Requested) {
                    let result = simulation.end().map(|()| None);
                    iterator_state.store(IteratorState::Idle);
                    return result;
                }

                let result = match advance_stride {
                    Some((seconds, strict)) => simulation.stride(seconds, strict),
                    None => simulation.step(),
                };
                match result {
                    Ok(Some(model_time)) => Ok(Some(model_time)),
                    Ok(None) => {
                        if !iterator_state.replace(IteratorState::Iterating, IteratorState::Idle) {
                            let end_result = simulation.end();
                            iterator_state.store(IteratorState::Idle);
                            end_result?;
                        }
                        Ok(None)
                    }
                    Err(error) => {
                        iterator_state.store(IteratorState::Idle);
                        Err(error)
                    }
                }
            })();
            if !matches!(result, Ok(Some(_))) {
                handle.iterator_exhausted = true;
            }
            result
        })?;
        Ok(model_time.map(datetime_parts))
    }

    /// Requests termination at the next iterator checkpoint without locking.
    ///
    /// # Returns
    /// `Ok(())` after publishing or retaining the termination request.
    ///
    /// # Errors
    /// Returns a lifecycle error without active iterator ownership or an internal-state error.
    fn terminate(&self) -> PyResult<()> {
        loop {
            match self.iterator_state.load() {
                Some(IteratorState::Idle) => {
                    return Err(lifecycle_error(
                        "terminate",
                        "no active iterator owns advancement",
                    ));
                }
                Some(IteratorState::Requested) => return Ok(()),
                Some(IteratorState::Iterating) => {
                    if self
                        .iterator_state
                        .replace(IteratorState::Iterating, IteratorState::Requested)
                    {
                        return Ok(());
                    }
                }
                None => return Err(internal_error("terminate", "invalid iterator state")),
            }
        }
    }

    /// Runs the complete non-interactive lifecycle and closes the project.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `save_results` - Whether report-period results and detailed reporting are saved.
    ///
    /// # Returns
    /// `Ok(())` after the full lifecycle and cleanup succeed.
    ///
    /// # Errors
    /// Returns a native exception when iteration owns advancement or execution fails.
    fn execute(&self, py: Python<'_>, save_results: &Bound<'_, PyAny>) -> PyResult<()> {
        let save_results = exact_bool(py, save_results, "save_results")?;
        self.ensure_not_iterating("execute")?;
        let iterator_state = &self.iterator_state;
        self.detached(py, "execute", move |handle| {
            let result = handle.execute(save_results);
            if handle.simulation.lifecycle_read().state == SimulationLifecycle::Closed {
                iterator_state.store(IteratorState::Idle);
            }
            result
        })
    }

    /// Finalizes the current run.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Errors
    /// Returns a native exception when iterator state is invalid or finalization fails.
    ///
    /// # Panics
    /// An invalid private atomic representation triggers a contained owner panic.
    fn end(&self, py: Python<'_>) -> PyResult<()> {
        let iterator_state = &self.iterator_state;
        self.detached(py, "end", move |handle| {
            let iterator_owned = match iterator_state.load() {
                Some(IteratorState::Idle) => false,
                Some(IteratorState::Iterating | IteratorState::Requested) => true,
                None => panic!("invalid iterator state"),
            };
            let result = handle.simulation.end();
            iterator_state.store(IteratorState::Idle);
            if iterator_owned {
                handle.iterator_exhausted = true;
            }
            result
        })
    }

    /// Closes the current project and invalidates its active generation.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Errors
    /// Returns the primary native end or close failure after cleanup is complete.
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        let iterator_state = &self.iterator_state;
        self.detached(py, "close", move |handle| {
            let result = handle.close_project();
            iterator_state.store(IteratorState::Idle);
            result
        })
    }

    /// Generates and flushes the detailed report.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `Ok(())` after detailed reporting is generated and flushed.
    ///
    /// # Errors
    /// Returns a native exception when reporting fails.
    fn report(&self, py: Python<'_>) -> PyResult<()> {
        self.detached(py, "report", |handle| handle.simulation.report())
    }

    /// Forces the next execute step onto the existing timestep-error path.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// `Ok(())` after the test-only failure is configured.
    ///
    /// # Errors
    /// Returns a native exception when the owner cannot be entered.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _force_timestep_error_for_test(&self, py: Python<'_>) -> PyResult<()> {
        self.detached(py, "force_timestep_error", |handle| {
            handle.simulation.force_timestep_error_for_test();
            Ok(())
        })
    }

    /// Poisons this owner so facade failure mapping can be tested end to end.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Panics
    /// Deliberately triggers a mutex-poisoning panic that is caught before this method returns.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _poison_for_test(&self, py: Python<'_>) {
        py.detach(|| {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let _handle = self
                    .handle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                panic!("forced Simulation Owner poison");
            }));
        });
    }

    /// Panics while holding this owner so containment and isolation can be tested.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// No successful value; the contained panic maps to an internal native error.
    ///
    /// # Errors
    /// Always returns a structured internal native error.
    ///
    /// # Panics
    /// Deliberately panics while holding the owner mutex; `detached` catches the panic.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _panic_for_test(&self, py: Python<'_>) -> PyResult<()> {
        self.detached(py, "panic_for_test", |_handle| {
            panic!("forced Simulation Owner panic")
        })
    }

    /// Arms a deterministic pause inside the next owner-locked operation.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _arm_owner_pause_for_test(&self) {
        self.test_concurrency.arm_pause();
    }

    /// Waits detached until the armed operation holds this owner's mutex.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `timeout_seconds` - Deadlock-guard timeout in seconds.
    ///
    /// # Returns
    /// `true` when the armed operation reaches the synchronization point.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _wait_owner_paused_for_test(&self, py: Python<'_>, timeout_seconds: f64) -> bool {
        let timeout = Duration::from_secs_f64(timeout_seconds);
        py.detach(|| self.test_concurrency.wait_paused(timeout))
    }

    /// Releases the deterministic owner pause.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _release_owner_pause_for_test(&self) {
        self.test_concurrency.release_pause();
    }

    /// Arms deterministic observation of the next owner acquisition.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _arm_owner_probe_for_test(&self) {
        self.test_concurrency.arm_probe();
    }

    /// Waits detached until the observed operation reaches owner acquisition.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    /// * `timeout_seconds` - Deadlock-guard timeout in seconds.
    ///
    /// # Returns
    /// `true` when the observed operation reaches owner acquisition.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _wait_owner_probe_waiting_for_test(&self, py: Python<'_>, timeout_seconds: f64) -> bool {
        let timeout = Duration::from_secs_f64(timeout_seconds);
        py.detach(|| self.test_concurrency.wait_probe_waiting(timeout))
    }

    /// Returns whether the observed operation acquired the owner mutex.
    ///
    /// # Returns
    /// `true` after the observed operation acquires the owner mutex.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    fn _owner_probe_acquired_for_test(&self) -> bool {
        self.test_concurrency.probe_acquired()
    }
}

/// Creates one Python validation error for schedule or option transport.
///
/// # Arguments
/// * `operation` - Stable native operation name.
/// * `detail` - Stable validation detail.
///
/// # Returns
/// Private native validation exception.
fn policy_py_error(operation: &str, detail: impl Into<String>) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

/// Extracts one exact timezone-naive whole-second Python datetime.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public schedule field name.
///
/// # Returns
/// Encoded SWMM date-time value.
///
/// # Errors
/// Returns a validation error for the wrong type, timezone, or resolution.
fn schedule_datetime(value: &Bound<'_, PyAny>, name: &str) -> PyResult<DateTime> {
    let operation = "update_schedule";
    if !is_exact_public_python_type(value, PublicType::DateTime, operation)? {
        return Err(policy_py_error(
            operation,
            format!("{name} must be datetime"),
        ));
    }
    let offset = value.call_method0("utcoffset").map_err(|_| {
        policy_py_error(
            operation,
            format!("{name} has invalid timezone information"),
        )
    })?;
    if !offset.is_none() {
        return Err(policy_py_error(
            operation,
            format!("{name} must be timezone-naive"),
        ));
    }
    let microsecond = value
        .getattr("microsecond")?
        .extract::<u32>()
        .map_err(|_| policy_py_error(operation, format!("{name} has invalid resolution")))?;
    if microsecond != 0 {
        return Err(policy_py_error(
            operation,
            format!("{name} must use whole-second resolution"),
        ));
    }
    let part = |field: &str| -> PyResult<i32> {
        value
            .getattr(field)?
            .extract::<i32>()
            .map_err(|_| policy_py_error(operation, format!("{name} has invalid {field}")))
    };
    Ok(
        datetime_encodeDate(part("year")?, part("month")?, part("day")?)
            + datetime_encodeTime(part("hour")?, part("minute")?, part("second")?),
    )
}

/// Extracts one exact Python timedelta as finite seconds.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public option field name.
/// * `whole_seconds` - Whether fractional seconds are forbidden.
///
/// # Returns
/// Duration in seconds.
///
/// # Errors
/// Returns a validation error for the wrong type, resolution, or integer overflow.
fn option_duration(value: &Bound<'_, PyAny>, name: &str, whole_seconds: bool) -> PyResult<f64> {
    let operation = "update_options";
    if !is_exact_public_python_type(value, PublicType::Timedelta, operation)? {
        return Err(policy_py_error(
            operation,
            format!("{name} must be timedelta"),
        ));
    }
    let days = value.getattr("days")?.extract::<i64>()?;
    let seconds = value.getattr("seconds")?.extract::<i64>()?;
    let microseconds = value.getattr("microseconds")?.extract::<i64>()?;
    if whole_seconds && microseconds != 0 {
        return Err(policy_py_error(
            operation,
            format!("{name} must use whole seconds"),
        ));
    }
    let total = days as f64 * consts::SECperDAY + seconds as f64 + microseconds as f64 / 1000000.0;
    if !total.is_finite() {
        return Err(policy_py_error(operation, format!("{name} must be finite")));
    }
    Ok(total)
}

/// Extracts one strict non-boolean Python number.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public option field name.
///
/// # Returns
/// Extracted floating-point value.
///
/// # Errors
/// Returns a validation error unless the exact type is `int` or `float`.
fn option_float(value: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    let operation = "update_options";
    let exact_int = is_exact_public_python_type(value, PublicType::Int, operation)?;
    let exact_float = is_exact_public_python_type(value, PublicType::Float, operation)?;
    if !exact_int && !exact_float {
        return Err(policy_py_error(
            operation,
            format!("{name} must be int or float, not bool"),
        ));
    }
    value
        .extract::<f64>()
        .map_err(|_| policy_py_error(operation, format!("{name} must be finite numeric")))
}

/// Extracts one strict Python boolean.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public option field name.
///
/// # Returns
/// Extracted boolean.
///
/// # Errors
/// Returns a validation error for non-boolean values.
fn option_bool(value: &Bound<'_, PyAny>, name: &str) -> PyResult<bool> {
    if !value.is_instance_of::<PyBool>() {
        return Err(policy_py_error(
            "update_options",
            format!("{name} must be bool"),
        ));
    }
    value.extract::<bool>()
}

/// Extracts one strict Python integer within native range.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public option field name.
///
/// # Returns
/// Extracted 32-bit integer.
///
/// # Errors
/// Returns a validation error for booleans, non-integers, or overflow.
fn option_integer(value: &Bound<'_, PyAny>, name: &str) -> PyResult<i32> {
    let operation = "update_options";
    if !is_exact_public_python_type(value, PublicType::Int, operation)? {
        return Err(policy_py_error(operation, format!("{name} must be int")));
    }
    value
        .extract::<i32>()
        .map_err(|_| policy_py_error(operation, format!("{name} is outside native integer range")))
}

/// Extracts one exact public enum or case-insensitive string value.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public option field name.
/// * `enum_type` - Exact exported enum class name.
///
/// # Returns
/// Lowercase categorical value.
///
/// # Errors
/// Returns a validation error for foreign enum and non-string types.
fn option_enum(value: &Bound<'_, PyAny>, name: &str, enum_type: PublicType) -> PyResult<String> {
    let operation = "update_options";
    let raw = if is_exact_public_python_type(value, enum_type, operation)? {
        value.getattr("value")?.extract::<String>()?
    } else if is_exact_public_python_type(value, PublicType::Str, operation)? {
        value.extract::<String>()?
    } else {
        return Err(policy_py_error(
            operation,
            format!("{name} must be {} or string", enum_type.name()),
        ));
    };
    Ok(raw.to_ascii_lowercase())
}

/// Converts an exact non-leap `(month, day)` tuple to day of year.
///
/// # Arguments
/// * `value` - Raw Python candidate.
/// * `name` - Public sweep field name.
///
/// # Returns
/// Non-leap day of year in `1..=365`.
///
/// # Errors
/// Returns a validation error for malformed or invalid dates.
fn option_sweep_day(value: &Bound<'_, PyAny>, name: &str) -> PyResult<i32> {
    let operation = "update_options";
    if !is_exact_public_python_type(value, PublicType::Tuple, operation)? {
        return Err(policy_py_error(
            operation,
            format!("{name} must be a (month, day) integer tuple"),
        ));
    }
    let tuple = value.cast::<PyTuple>().map_err(|_| {
        policy_py_error(
            operation,
            format!("{name} must be a (month, day) integer tuple"),
        )
    })?;
    if tuple.len() != 2 {
        return Err(policy_py_error(
            operation,
            format!("{name} must be a (month, day) integer tuple"),
        ));
    }
    let month = option_integer(&tuple.get_item(0)?, name)?;
    let day = option_integer(&tuple.get_item(1)?, name)?;
    const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&month) || !(1..=MONTH_DAYS[(month - 1) as usize]).contains(&day) {
        return Err(policy_py_error(
            operation,
            format!("{name} must be a non-leap calendar date"),
        ));
    }
    Ok(MONTH_DAYS[..(month - 1) as usize].iter().sum::<i32>() + day)
}

/// Extracts a sparse schedule patch from raw Python values.
///
/// # Arguments
/// * `changes` - Raw sparse schedule mapping.
///
/// # Returns
/// Typed schedule patch.
///
/// # Errors
/// Returns a validation error for unknown fields or malformed datetimes.
fn extract_schedule_patch(changes: &Bound<'_, PyDict>) -> PyResult<SimulationSchedulePatch> {
    let mut patch = SimulationSchedulePatch::default();
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| policy_py_error("update_schedule", "schedule fields must be strings"))?;
        match name.as_str() {
            "start_time" => patch.start_time = Some(schedule_datetime(&value, &name)?),
            "report_start" => patch.report_start = Some(schedule_datetime(&value, &name)?),
            "end_time" => patch.end_time = Some(schedule_datetime(&value, &name)?),
            _ => {
                return Err(policy_py_error(
                    "update_schedule",
                    format!("unknown schedule field: {name}"),
                ));
            }
        }
    }
    Ok(patch)
}

/// Extracts a sparse simulation-options patch from raw Python values.
///
/// # Arguments
/// * `changes` - Raw sparse option mapping.
///
/// # Returns
/// Typed simulation-options patch.
///
/// # Errors
/// Returns a validation error for unknown, read-only, or malformed fields.
fn extract_options_patch(changes: &Bound<'_, PyDict>) -> PyResult<SimulationOptionsPatch> {
    let mut patch = SimulationOptionsPatch::default();
    for (name, value) in changes.iter() {
        let name = name
            .extract::<String>()
            .map_err(|_| policy_py_error("update_options", "option fields must be strings"))?;
        match name.as_str() {
            "routing_step" => {
                patch.routing_step_seconds = Some(option_duration(&value, &name, false)?)
            }
            "report_step" => {
                let seconds = option_duration(&value, &name, true)?;
                if seconds > i32::MAX as f64 {
                    return Err(policy_py_error(
                        "update_options",
                        "report_step exceeds native integer range",
                    ));
                }
                patch.report_step_seconds = Some(seconds as i32);
            }
            "detailed_reporting_enabled" => {
                patch.detailed_reporting_enabled = Some(option_bool(&value, &name)?)
            }
            "custom_ellipse_model" => {
                patch.custom_ellipse_model = Some(
                    match option_enum(&value, &name, PublicType::CustomEllipseModel)?.as_str() {
                        "epa_legacy" => CustomEllipseModel::EpaLegacy,
                        "true_ellipse" => CustomEllipseModel::TrueEllipse,
                        _ => {
                            return Err(policy_py_error(
                                "update_options",
                                "custom_ellipse_model is not supported",
                            ));
                        }
                    },
                );
            }
            "surcharge_method" => {
                patch.surcharge_method = Some(
                    match option_enum(&value, &name, PublicType::SurchargeMethod)?.as_str() {
                        "extran" => SurchargeMethodType::Extran,
                        "slot" => SurchargeMethodType::Slot,
                        _ => {
                            return Err(policy_py_error(
                                "update_options",
                                "surcharge_method is not supported",
                            ));
                        }
                    },
                );
            }
            "allow_ponding" => patch.allow_ponding = Some(option_bool(&value, &name)?),
            "inertia_damping" => {
                patch.inertia_damping = Some(
                    match option_enum(&value, &name, PublicType::InertiaDamping)?.as_str() {
                        "none" => InertialDampingType::NoDamping,
                        "partial" => InertialDampingType::PartialDamping,
                        "full" => InertialDampingType::FullDamping,
                        _ => {
                            return Err(policy_py_error(
                                "update_options",
                                "inertia_damping is not supported",
                            ));
                        }
                    },
                );
            }
            "normal_flow_limit" => {
                patch.normal_flow_limit = Some(
                    match option_enum(&value, &name, PublicType::NormalFlowLimit)?.as_str() {
                        "slope" => NormalFlowType::Slope,
                        "froude" => NormalFlowType::Froude,
                        "both" => NormalFlowType::Both,
                        "neither" => NormalFlowType::Neither,
                        _ => {
                            return Err(policy_py_error(
                                "update_options",
                                "normal_flow_limit is not supported",
                            ));
                        }
                    },
                );
            }
            "skip_steady_state" => patch.skip_steady_state = Some(option_bool(&value, &name)?),
            "rainfall_enabled" => patch.rainfall_enabled = Some(option_bool(&value, &name)?),
            "rdii_enabled" => patch.rdii_enabled = Some(option_bool(&value, &name)?),
            "snowmelt_enabled" => patch.snowmelt_enabled = Some(option_bool(&value, &name)?),
            "groundwater_enabled" => patch.groundwater_enabled = Some(option_bool(&value, &name)?),
            "routing_enabled" => patch.routing_enabled = Some(option_bool(&value, &name)?),
            "quality_enabled" => patch.quality_enabled = Some(option_bool(&value, &name)?),
            "rule_step" => {
                let seconds = option_duration(&value, &name, true)?;
                if seconds > i32::MAX as f64 {
                    return Err(policy_py_error(
                        "update_options",
                        "rule_step exceeds native integer range",
                    ));
                }
                patch.rule_step_seconds = Some(seconds as i32);
            }
            "sweep_start" => patch.sweep_start = Some(option_sweep_day(&value, &name)?),
            "sweep_end" => patch.sweep_end = Some(option_sweep_day(&value, &name)?),
            "maximum_trials" => patch.maximum_trials = Some(option_integer(&value, &name)?),
            "requested_threads" => patch.requested_threads = Some(option_integer(&value, &name)?),
            "minimum_routing_step" => {
                patch.minimum_routing_step_seconds = Some(option_duration(&value, &name, false)?)
            }
            "lengthening_step" => {
                patch.lengthening_step_seconds = Some(option_duration(&value, &name, false)?)
            }
            "antecedent_dry_duration" => {
                patch.antecedent_dry_seconds = Some(option_duration(&value, &name, false)?)
            }
            "courant_factor" => patch.courant_factor = Some(option_float(&value, &name)?),
            "minimum_surface_area" => {
                patch.minimum_surface_area = Some(option_float(&value, &name)?)
            }
            "minimum_conduit_slope" => {
                patch.minimum_conduit_slope = Some(option_float(&value, &name)?)
            }
            _ => {
                return Err(policy_py_error(
                    "update_options",
                    format!("unknown or read-only option field: {name}"),
                ));
            }
        }
    }
    Ok(patch)
}
