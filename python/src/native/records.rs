use super::*;

/// Registers every public native snapshot and statistics record class.
///
/// # Arguments
/// * `module` - Private extension module receiving public record types.
///
/// # Errors
/// Returns a Python registration error.
pub(crate) fn register_records(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<SubcatchmentSnapshot>()?;
    module.add_class::<NodeSnapshot>()?;
    module.add_class::<LinkSnapshot>()?;
    module.add_class::<SubcatchmentQualitySnapshot>()?;
    module.add_class::<NodeStatistics>()?;
    module.add_class::<LidUnitSnapshot>()?;
    module.add_class::<StorageStatistics>()?;
    module.add_class::<NodeQualitySnapshot>()?;
    module.add_class::<OutfallStatistics>()?;
    module.add_class::<LinkStatistics>()?;
    module.add_class::<PumpStatistics>()?;
    module.add_class::<SubcatchmentStatistics>()?;
    module.add_class::<NodeStatisticsSnapshot>()?;
    module.add_class::<LinkQualitySnapshot>()?;
    module.add_class::<LinkStatisticsSnapshot>()?;
    module.add_class::<SubcatchmentLidSnapshot>()?;
    module.add_class::<SubcatchmentStatisticsSnapshot>()?;
    module.add_class::<RoutingDiagnostics>()?;
    module.add_class::<RoutingTotals>()?;
    module.add_class::<RunoffTotals>()?;
    module.add_class::<QualityBalance>()?;
    module.add_class::<SimulationStatistics>()?;
    Ok(())
}

macro_rules! record_type {
    (
        $name:ident,
        $read:ty,
        $python_name:literal,
        $class_doc:literal,
        [$($field:literal),+ $(,)?]
    ) => {
        #[doc = $class_doc]
        #[pyclass(
            frozen,
            skip_from_py_object,
            module = "swmmrs.snapshots",
            name = $python_name
        )]
        pub(crate) struct $name {
            values: $read,
        }

        impl $name {
            /// Creates one immutable native record from an owned solver acquisition.
            ///
            /// # Arguments
            /// * `values` - Owned typed solver values.
            ///
            /// # Returns
            /// Native immutable record.
            pub(super) fn new(values: $read) -> Self {
                Self { values }
            }
        }

        #[pymethods]
        impl $name {
            /// Marks the native record as final for Python introspection.
            ///
            /// # Returns
            /// Always `true`.
            #[classattr]
            fn __final__() -> bool {
                true
            }

            /// Compares records through their exact final public field values.
            ///
            /// # Arguments
            /// * `slf` - Borrowed native record on the left side.
            /// * `other` - Python object on the right side.
            /// * `py` - Attached Python interpreter token.
            ///
            /// # Returns
            /// `true` when `other` has the exact record type and every exposed field is equal.
            ///
            /// # Errors
            /// Returns a Python attribute, conversion, or comparison error.
            fn __eq__(
                slf: PyRef<'_, Self>,
                other: &Bound<'_, PyAny>,
                py: Python<'_>,
            ) -> PyResult<bool> {
                if !other.get_type().is(py.get_type::<Self>()) {
                    return Ok(false);
                }
                let object = match slf.into_pyobject(py) {
                    Ok(object) => object,
                    Err(never) => match never {},
                };
                if object.as_any().is(other) {
                    return Ok(true);
                }
                record_equal(object.as_any(), other, &[$($field),+])
            }
        }

        impl $name {
            /// Formats the complete immutable record through its public fields.
            ///
            /// # Arguments
            /// * `slf` - Borrowed native record.
            /// * `py` - Attached Python interpreter token.
            ///
            /// # Returns
            /// Dataclass-like record representation.
            ///
            /// # Errors
            /// Returns a Python attribute or representation error.
            fn repr(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
                let object = match slf.into_pyobject(py) {
                    Ok(object) => object,
                    Err(never) => match never {},
                };
                record_repr(object.as_any(), $python_name, &[$($field),+])
            }
        }
    };
}

record_type!(
    SubcatchmentSnapshot,
    SubcatchmentSnapshotRead,
    "SubcatchmentSnapshot",
    "Aligned current subcatchment hydraulics in quantity-specific project reporting units.",
    [
        "object_ids",
        "rainfall",
        "evaporation",
        "infiltration",
        "runon",
        "runoff",
        "snow_depth"
    ]
);
#[pymethods]
impl SubcatchmentSnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns rainfall intensity in inches/hour or millimeters/hour.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn rainfall(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.rainfall)
    }
    /// Returns evaporation rate in inches/day or millimeters/day.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn evaporation(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.evaporation)
    }
    /// Returns infiltration rate in inches/hour or millimeters/hour.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn infiltration(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.infiltration)
    }
    /// Returns runon rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runon(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.runon)
    }
    /// Returns runoff rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runoff(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.runoff)
    }
    /// Returns snow depth in inches or millimeters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn snow_depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.snow_depth)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    NodeSnapshot,
    NodeSnapshotRead,
    "NodeSnapshot",
    "Aligned current node hydraulics in project length, volume, flow, and duration units.",
    [
        "object_ids",
        "depth",
        "head",
        "volume",
        "lateral_inflow",
        "total_inflow",
        "total_outflow",
        "losses",
        "flooding",
        "hydraulic_retention_time"
    ]
);
#[pymethods]
impl NodeSnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns water depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of water depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.depth)
    }
    /// Returns hydraulic head in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of hydraulic head aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn head(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.head)
    }
    /// Returns water volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of water volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.volume)
    }
    /// Returns lateral inflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of lateral inflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn lateral_inflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.lateral_inflow)
    }
    /// Returns total inflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of total inflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn total_inflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.total_inflow)
    }
    /// Returns total outflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of total outflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn total_outflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.total_outflow)
    }
    /// Returns hydraulic loss rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of loss rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn losses(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.losses)
    }
    /// Returns flooding rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python floats aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn flooding(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.flooding)
    }
    /// Returns hydraulic retention time.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of optional Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn hydraulic_retention_time(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        optional_duration_tuple(py, &self.values.hydraulic_retention_seconds)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    LinkSnapshot,
    LinkSnapshotRead,
    "LinkSnapshot",
    "Aligned current link hydraulics in project length, volume, flow, duration, and dimensionless units.",
    [
        "object_ids",
        "setting",
        "target_setting",
        "time_open",
        "time_closed",
        "flow",
        "depth",
        "velocity",
        "top_width",
        "volume",
        "capacity",
        "upstream_surface_area",
        "downstream_surface_area",
        "froude_number"
    ]
);
#[pymethods]
impl LinkSnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns the current dimensionless control setting.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of current control setting aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn setting(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.setting)
    }
    /// Returns the target dimensionless control setting.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of target control setting aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn target_setting(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.target_setting)
    }
    /// Returns time the link has been open.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_open(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_open_seconds)
    }
    /// Returns time the link has been closed.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_closed(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_closed_seconds)
    }
    /// Returns flow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of flow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn flow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.flow)
    }
    /// Returns water depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of water depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.depth)
    }
    /// Returns flow velocity in feet/second or meters/second.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of flow velocity aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn velocity(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.velocity)
    }
    /// Returns optional top width in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of optional top width aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn top_width(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        optional_float_tuple(py, &self.values.top_width)
    }
    /// Returns water volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of water volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.volume)
    }
    /// Returns the dimensionless conduit-full-area ratio or regulator setting.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of capacity ratio aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn capacity(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.capacity)
    }
    /// Returns upstream surface area in square feet or square meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of upstream surface area aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn upstream_surface_area(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.upstream_surface_area)
    }
    /// Returns downstream surface area in square feet or square meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of downstream surface area aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn downstream_surface_area(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.downstream_surface_area)
    }
    /// Returns the dimensionless Froude number.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Froude number aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn froude_number(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.froude_number)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    SubcatchmentQualitySnapshot,
    SubcatchmentQualitySnapshotRead,
    "SubcatchmentQualitySnapshot",
    "Pollutant-major current subcatchment quality in configured pollutant reporting units; each inner row follows object_ids.",
    [
        "object_ids",
        "pollutant_ids",
        "runoff_concentrations",
        "ponded_concentrations",
        "buildup_loads",
        "total_washoff_loads"
    ]
);
#[pymethods]
impl SubcatchmentQualitySnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns canonical pollutant identifiers defining the outer matrix axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical pollutant identifiers in configured pollutant order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn pollutant_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.pollutant_ids)
    }
    /// Returns runoff concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runoff_concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.runoff_concentrations)
    }
    /// Returns ponded concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn ponded_concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.ponded_concentrations)
    }
    /// Returns buildup loads in each pollutant's configured buildup units per project land area.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn buildup_loads(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.buildup_loads)
    }
    /// Returns cumulative washoff in configured pollutant mass/count reporting units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn total_washoff_loads(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.total_washoff_loads)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    NodeStatistics,
    NodeStatisticsRead,
    "NodeStatistics",
    "Cumulative depth, volume, flow, count, datetime, and duration statistics for one node.",
    [
        "average_depth",
        "maximum_depth",
        "maximum_depth_time",
        "maximum_reported_depth",
        "flooded_volume",
        "time_flooded",
        "time_surcharged",
        "time_courant_critical",
        "total_lateral_inflow",
        "maximum_lateral_inflow",
        "maximum_inflow",
        "maximum_overflow",
        "maximum_ponded_volume",
        "nonconverged_count",
        "maximum_inflow_time",
        "maximum_overflow_time"
    ]
);
#[pymethods]
impl NodeStatistics {
    /// Returns time-weighted average depth in feet or meters.
    ///
    /// # Returns
    /// Python float containing time-weighted average depth.
    #[getter]
    fn average_depth(&self) -> f64 {
        self.values.average_depth
    }
    /// Returns maximum depth in feet or meters.
    ///
    /// # Returns
    /// Python float containing maximum depth.
    #[getter]
    fn maximum_depth(&self) -> f64 {
        self.values.maximum_depth
    }
    /// Returns time of maximum depth.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Naive Python `datetime.datetime` for the time of maximum depth.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_depth_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        datetime(py, self.values.maximum_depth_time)
    }
    /// Returns maximum reported depth in feet or meters.
    ///
    /// # Returns
    /// Python float containing maximum reported depth.
    #[getter]
    fn maximum_reported_depth(&self) -> f64 {
        self.values.maximum_reported_depth
    }
    /// Returns flooded volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing flooded volume.
    #[getter]
    fn flooded_volume(&self) -> f64 {
        self.values.flooded_volume
    }
    /// Returns cumulative flooded duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative flooded duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_flooded(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_flooded_seconds)
    }
    /// Returns cumulative surcharged duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative surcharged duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_surcharged(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_surcharged_seconds)
    }
    /// Returns cumulative Courant-critical duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative Courant-critical duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_courant_critical(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_courant_critical_seconds)
    }
    /// Returns total lateral inflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing total lateral inflow volume.
    #[getter]
    fn total_lateral_inflow(&self) -> f64 {
        self.values.total_lateral_inflow
    }
    /// Returns maximum lateral inflow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum lateral inflow rate.
    #[getter]
    fn maximum_lateral_inflow(&self) -> f64 {
        self.values.maximum_lateral_inflow
    }
    /// Returns maximum inflow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum inflow rate.
    #[getter]
    fn maximum_inflow(&self) -> f64 {
        self.values.maximum_inflow
    }
    /// Returns maximum overflow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum overflow rate.
    #[getter]
    fn maximum_overflow(&self) -> f64 {
        self.values.maximum_overflow
    }
    /// Returns maximum ponded volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing maximum ponded volume.
    #[getter]
    fn maximum_ponded_volume(&self) -> f64 {
        self.values.maximum_ponded_volume
    }
    /// Returns nonconverged solution count.
    ///
    /// # Returns
    /// Python integer containing nonconverged solution count.
    #[getter]
    fn nonconverged_count(&self) -> i32 {
        self.values.nonconverged_count
    }
    /// Returns time of maximum inflow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Naive Python `datetime.datetime` for the time of maximum inflow.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_inflow_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        datetime(py, self.values.maximum_inflow_time)
    }
    /// Returns time of maximum overflow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Naive Python `datetime.datetime` for the time of maximum overflow.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_overflow_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        datetime(py, self.values.maximum_overflow_time)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    LidUnitSnapshot,
    LidUnitSnapshotRead,
    "LidUnitSnapshot",
    "Current runtime values for one LID unit in equivalent rain-depth, rate, flow, and state units.",
    [
        "inflow",
        "evaporation",
        "infiltration",
        "surface_outflow",
        "drain_outflow",
        "initial_volume",
        "final_volume",
        "surface_depth",
        "pavement_depth",
        "storage_depth",
        "soil_moisture",
        "dry_time",
        "old_drain_flow",
        "new_drain_flow",
        "evaporation_rate",
        "maximum_native_infiltration_rate",
        "surface_inflow_rate",
        "surface_infiltration_rate",
        "surface_evaporation_rate",
        "surface_outflow_rate",
        "pavement_evaporation_rate",
        "pavement_percolation_rate",
        "soil_evaporation_rate",
        "soil_percolation_rate",
        "storage_inflow_rate",
        "storage_exfiltration_rate",
        "storage_evaporation_rate",
        "storage_drain_rate",
        "surface_flux_rate",
        "soil_flux_rate",
        "storage_flux_rate",
        "pavement_flux_rate"
    ]
);
#[pymethods]
impl LidUnitSnapshot {
    /// Returns cumulative LID inflow as equivalent rain depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing cumulative equivalent rain depth.
    #[getter]
    fn inflow(&self) -> f64 {
        self.values.inflow
    }
    /// Returns cumulative LID evaporation as equivalent rain depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing cumulative equivalent rain depth.
    #[getter]
    fn evaporation(&self) -> f64 {
        self.values.evaporation
    }
    /// Returns cumulative LID infiltration as equivalent rain depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing cumulative equivalent rain depth.
    #[getter]
    fn infiltration(&self) -> f64 {
        self.values.infiltration
    }
    /// Returns cumulative surface outflow as equivalent rain depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing cumulative equivalent rain depth.
    #[getter]
    fn surface_outflow(&self) -> f64 {
        self.values.surface_outflow
    }
    /// Returns cumulative drain outflow as equivalent rain depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing cumulative equivalent rain depth.
    #[getter]
    fn drain_outflow(&self) -> f64 {
        self.values.drain_outflow
    }
    /// Returns initial stored-water equivalent depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing initial stored-water equivalent depth.
    #[getter]
    fn initial_volume(&self) -> f64 {
        self.values.initial_volume
    }
    /// Returns final stored-water equivalent depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing final stored-water equivalent depth.
    #[getter]
    fn final_volume(&self) -> f64 {
        self.values.final_volume
    }
    /// Returns surface-layer depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing surface layer depth.
    #[getter]
    fn surface_depth(&self) -> f64 {
        self.values.surface_depth
    }
    /// Returns pavement-layer depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing pavement layer depth.
    #[getter]
    fn pavement_depth(&self) -> f64 {
        self.values.pavement_depth
    }
    /// Returns storage-layer depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing storage layer depth.
    #[getter]
    fn storage_depth(&self) -> f64 {
        self.values.storage_depth
    }
    /// Returns soil moisture fraction.
    ///
    /// # Returns
    /// Python float containing soil moisture fraction.
    #[getter]
    fn soil_moisture(&self) -> f64 {
        self.values.soil_moisture
    }
    /// Returns continuous dry duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the continuous dry duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn dry_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.dry_time_seconds)
    }
    /// Returns previous drain flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing previous drain flow rate.
    #[getter]
    fn old_drain_flow(&self) -> f64 {
        self.values.old_drain_flow
    }
    /// Returns current drain flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing current drain flow rate.
    #[getter]
    fn new_drain_flow(&self) -> f64 {
        self.values.new_drain_flow
    }
    /// Returns LID evaporation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing evaporation rate.
    #[getter]
    fn evaporation_rate(&self) -> f64 {
        self.values.evaporation_rate
    }
    /// Returns maximum native-soil infiltration rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing maximum native infiltration rate.
    #[getter]
    fn maximum_native_infiltration_rate(&self) -> f64 {
        self.values.maximum_native_infiltration_rate
    }
    /// Returns surface inflow rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing surface inflow rate.
    #[getter]
    fn surface_inflow_rate(&self) -> f64 {
        self.values.surface_inflow_rate
    }
    /// Returns surface infiltration rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing surface infiltration rate.
    #[getter]
    fn surface_infiltration_rate(&self) -> f64 {
        self.values.surface_infiltration_rate
    }
    /// Returns surface evaporation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing surface evaporation rate.
    #[getter]
    fn surface_evaporation_rate(&self) -> f64 {
        self.values.surface_evaporation_rate
    }
    /// Returns surface outflow rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing surface outflow rate.
    #[getter]
    fn surface_outflow_rate(&self) -> f64 {
        self.values.surface_outflow_rate
    }
    /// Returns pavement evaporation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing pavement evaporation rate.
    #[getter]
    fn pavement_evaporation_rate(&self) -> f64 {
        self.values.pavement_evaporation_rate
    }
    /// Returns pavement percolation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing pavement percolation rate.
    #[getter]
    fn pavement_percolation_rate(&self) -> f64 {
        self.values.pavement_percolation_rate
    }
    /// Returns soil evaporation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing soil evaporation rate.
    #[getter]
    fn soil_evaporation_rate(&self) -> f64 {
        self.values.soil_evaporation_rate
    }
    /// Returns soil percolation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing soil percolation rate.
    #[getter]
    fn soil_percolation_rate(&self) -> f64 {
        self.values.soil_percolation_rate
    }
    /// Returns storage inflow rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing storage inflow rate.
    #[getter]
    fn storage_inflow_rate(&self) -> f64 {
        self.values.storage_inflow_rate
    }
    /// Returns storage exfiltration rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing storage exfiltration rate.
    #[getter]
    fn storage_exfiltration_rate(&self) -> f64 {
        self.values.storage_exfiltration_rate
    }
    /// Returns storage evaporation rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing storage evaporation rate.
    #[getter]
    fn storage_evaporation_rate(&self) -> f64 {
        self.values.storage_evaporation_rate
    }
    /// Returns storage drain rate in inches/hour or millimeters/hour.
    ///
    /// # Returns
    /// Python float containing storage drain rate.
    #[getter]
    fn storage_drain_rate(&self) -> f64 {
        self.values.storage_drain_rate
    }
    /// Returns the native surface-layer flux rate in feet/second or meters/second.
    ///
    /// # Returns
    /// Python float containing surface-layer flux rate.
    #[getter]
    fn surface_flux_rate(&self) -> f64 {
        self.values.surface_flux_rate
    }
    /// Returns the native soil-layer flux rate in feet/second or meters/second.
    ///
    /// # Returns
    /// Python float containing soil-layer flux rate.
    #[getter]
    fn soil_flux_rate(&self) -> f64 {
        self.values.soil_flux_rate
    }
    /// Returns the native storage-layer flux rate in feet/second or meters/second.
    ///
    /// # Returns
    /// Python float containing storage-layer flux rate.
    #[getter]
    fn storage_flux_rate(&self) -> f64 {
        self.values.storage_flux_rate
    }
    /// Returns the native pavement-layer flux rate in feet/second or meters/second.
    ///
    /// # Returns
    /// Python float containing pavement-layer flux rate.
    #[getter]
    fn pavement_flux_rate(&self) -> f64 {
        self.values.pavement_flux_rate
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    StorageStatistics,
    StorageStatisticsRead,
    "StorageStatistics",
    "Cumulative statistics for one storage node in project volume and flow units.",
    [
        "initial_volume",
        "average_volume",
        "maximum_volume",
        "maximum_inflow",
        "evaporation_losses",
        "exfiltration_losses",
        "maximum_volume_time"
    ]
);
#[pymethods]
impl StorageStatistics {
    /// Returns initial stored volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing initial stored volume.
    #[getter]
    fn initial_volume(&self) -> f64 {
        self.values.initial_volume
    }
    /// Returns time-weighted average volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing time-weighted average volume.
    #[getter]
    fn average_volume(&self) -> f64 {
        self.values.average_volume
    }
    /// Returns maximum volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing maximum volume.
    #[getter]
    fn maximum_volume(&self) -> f64 {
        self.values.maximum_volume
    }
    /// Returns maximum inflow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum inflow rate.
    #[getter]
    fn maximum_inflow(&self) -> f64 {
        self.values.maximum_inflow
    }
    /// Returns cumulative evaporation loss volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing evaporation loss volume.
    #[getter]
    fn evaporation_losses(&self) -> f64 {
        self.values.evaporation_losses
    }
    /// Returns cumulative exfiltration loss volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing exfiltration loss volume.
    #[getter]
    fn exfiltration_losses(&self) -> f64 {
        self.values.exfiltration_losses
    }
    /// Returns time of maximum volume.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Naive Python `datetime.datetime` for the time of maximum volume.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_volume_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        datetime(py, self.values.maximum_volume_time)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    NodeQualitySnapshot,
    NodeQualitySnapshotRead,
    "NodeQualitySnapshot",
    "Pollutant-major current node quality in configured concentration units; each inner row follows object_ids.",
    [
        "object_ids",
        "pollutant_ids",
        "concentrations",
        "inflow_concentrations",
        "reactor_concentrations"
    ]
);
#[pymethods]
impl NodeQualitySnapshot {
    /// Returns canonical node identifiers defining each matrix's inner axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns canonical pollutant identifiers defining each matrix's outer axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical pollutant identifiers in configured pollutant order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn pollutant_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.pollutant_ids)
    }
    /// Returns concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.concentrations)
    }
    /// Returns inflow concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn inflow_concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.inflow_concentrations)
    }
    /// Returns reactor concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn reactor_concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.reactor_concentrations)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    OutfallStatistics,
    OutfallStatisticsRead,
    "OutfallStatistics",
    "Cumulative flow and pollutant-load statistics for one outfall in configured reporting units.",
    [
        "average_flow",
        "maximum_flow",
        "pollutant_loads",
        "period_count"
    ]
);
#[pymethods]
impl OutfallStatistics {
    /// Returns average flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing average flow rate.
    #[getter]
    fn average_flow(&self) -> f64 {
        self.values.average_flow
    }
    /// Returns maximum flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum flow rate.
    #[getter]
    fn maximum_flow(&self) -> f64 {
        self.values.maximum_flow
    }
    /// Returns cumulative loads in each pollutant's configured mass/count reporting units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Read-only insertion-ordered mapping from canonical pollutant IDs to Python float loads.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn pollutant_loads(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        float_mapping(py, &self.values.pollutant_loads)
    }
    /// Returns reporting period count.
    ///
    /// # Returns
    /// Python integer containing reporting period count.
    #[getter]
    fn period_count(&self) -> i32 {
        self.values.period_count
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    LinkStatistics,
    LinkStatisticsRead,
    "LinkStatistics",
    "Cumulative flow, depth, velocity, fraction, count, and duration statistics for one link.",
    [
        "maximum_flow",
        "maximum_flow_time",
        "maximum_velocity",
        "maximum_depth",
        "maximum_street_fill_fraction",
        "time_normal_flow",
        "time_inlet_control",
        "time_surcharged",
        "time_full_upstream",
        "time_full_downstream",
        "time_full_flow",
        "time_capacity_limited",
        "time_in_flow_class",
        "time_courant_critical",
        "flow_turns",
        "flow_turn_sign"
    ]
);
#[pymethods]
impl LinkStatistics {
    /// Returns maximum flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum flow rate.
    #[getter]
    fn maximum_flow(&self) -> f64 {
        self.values.maximum_flow
    }
    /// Returns time of maximum flow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Naive Python `datetime.datetime` for the time of maximum flow.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_flow_time(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        datetime(py, self.values.maximum_flow_time)
    }
    /// Returns maximum velocity in feet/second or meters/second.
    ///
    /// # Returns
    /// Python float containing maximum velocity.
    #[getter]
    fn maximum_velocity(&self) -> f64 {
        self.values.maximum_velocity
    }
    /// Returns maximum depth in feet or meters.
    ///
    /// # Returns
    /// Python float containing maximum depth.
    #[getter]
    fn maximum_depth(&self) -> f64 {
        self.values.maximum_depth
    }
    /// Returns the optional dimensionless maximum street fill fraction.
    ///
    /// # Returns
    /// Python float fraction for street cross sections; otherwise `None`.
    #[getter]
    fn maximum_street_fill_fraction(&self) -> Option<f64> {
        self.values.maximum_street_fill_fraction
    }
    /// Returns cumulative normal-flow duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative normal-flow duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_normal_flow(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_normal_flow_seconds)
    }
    /// Returns cumulative inlet-control duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative inlet-control duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_inlet_control(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_inlet_control_seconds)
    }
    /// Returns cumulative surcharged duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative surcharged duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_surcharged(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_surcharged_seconds)
    }
    /// Returns cumulative upstream-full duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative upstream-full duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_upstream(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_full_upstream_seconds)
    }
    /// Returns cumulative downstream-full duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative downstream-full duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_downstream(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_full_downstream_seconds)
    }
    /// Returns cumulative full-flow duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative full-flow duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_flow(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_full_flow_seconds)
    }
    /// Returns cumulative capacity-limited duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative capacity-limited duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_capacity_limited(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_capacity_limited_seconds)
    }
    /// Returns durations in each FlowClass order.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values in `FlowClass` enum order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_in_flow_class(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_in_flow_class_seconds)
    }
    /// Returns cumulative Courant-critical duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative Courant-critical duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_courant_critical(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_courant_critical_seconds)
    }
    /// Returns flow-direction turn count.
    ///
    /// # Returns
    /// Python integer containing flow-direction turn count.
    #[getter]
    fn flow_turns(&self) -> i64 {
        self.values.flow_turns
    }
    /// Returns the latest flow-direction sign.
    ///
    /// # Returns
    /// Python integer equal to -1, 0, or 1.
    #[getter]
    fn flow_turn_sign(&self) -> i32 {
        self.values.flow_turn_sign
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    PumpStatistics,
    PumpStatisticsRead,
    "PumpStatistics",
    "Cumulative flow, volume, energy, and duration statistics for one pump.",
    [
        "time_utilized",
        "minimum_flow",
        "average_flow",
        "maximum_flow",
        "pumped_volume",
        "energy_consumed",
        "off_curve_low",
        "off_curve_high",
        "startup_count",
        "period_count"
    ]
);
#[pymethods]
impl PumpStatistics {
    /// Returns cumulative pump utilization duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the cumulative pump utilization duration.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_utilized(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.time_utilized_seconds)
    }
    /// Returns minimum flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing minimum flow rate.
    #[getter]
    fn minimum_flow(&self) -> f64 {
        self.values.minimum_flow
    }
    /// Returns average flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing average flow rate.
    #[getter]
    fn average_flow(&self) -> f64 {
        self.values.average_flow
    }
    /// Returns maximum flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum flow rate.
    #[getter]
    fn maximum_flow(&self) -> f64 {
        self.values.maximum_flow
    }
    /// Returns total pumped volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing total pumped volume.
    #[getter]
    fn pumped_volume(&self) -> f64 {
        self.values.pumped_volume
    }
    /// Returns total pump energy consumed in kilowatt-hours.
    ///
    /// # Returns
    /// Python float containing total energy consumed.
    #[getter]
    fn energy_consumed(&self) -> f64 {
        self.values.energy_consumed
    }
    /// Returns the duration below the pump curve as a Python `datetime.timedelta`.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the duration below the pump curve.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn off_curve_low(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.off_curve_low_seconds)
    }
    /// Returns the duration above the pump curve as a Python `datetime.timedelta`.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the duration above the pump curve.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn off_curve_high(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.off_curve_high_seconds)
    }
    /// Returns pump startup count.
    ///
    /// # Returns
    /// Python integer containing pump startup count.
    #[getter]
    fn startup_count(&self) -> i32 {
        self.values.startup_count
    }
    /// Returns reporting period count.
    ///
    /// # Returns
    /// Python integer containing reporting period count.
    #[getter]
    fn period_count(&self) -> i32 {
        self.values.period_count
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    SubcatchmentStatistics,
    SubcatchmentStatisticsRead,
    "SubcatchmentStatistics",
    "Cumulative subcatchment depth, volume, and flow statistics in project reporting units.",
    [
        "precipitation",
        "runon_volume",
        "evaporation_volume",
        "infiltration_volume",
        "runoff_volume",
        "maximum_runoff",
        "impervious_runoff_volume",
        "pervious_runoff_volume"
    ]
);
#[pymethods]
impl SubcatchmentStatistics {
    /// Returns precipitation depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing precipitation depth.
    #[getter]
    fn precipitation(&self) -> f64 {
        self.values.precipitation
    }
    /// Returns runon volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing runon volume.
    #[getter]
    fn runon_volume(&self) -> f64 {
        self.values.runon_volume
    }
    /// Returns evaporation volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing evaporation volume.
    #[getter]
    fn evaporation_volume(&self) -> f64 {
        self.values.evaporation_volume
    }
    /// Returns infiltration volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing infiltration volume.
    #[getter]
    fn infiltration_volume(&self) -> f64 {
        self.values.infiltration_volume
    }
    /// Returns runoff volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing runoff volume.
    #[getter]
    fn runoff_volume(&self) -> f64 {
        self.values.runoff_volume
    }
    /// Returns maximum runoff rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing maximum runoff rate.
    #[getter]
    fn maximum_runoff(&self) -> f64 {
        self.values.maximum_runoff
    }
    /// Returns impervious-area runoff volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing impervious-area runoff volume.
    #[getter]
    fn impervious_runoff_volume(&self) -> f64 {
        self.values.impervious_runoff_volume
    }
    /// Returns pervious-area runoff volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing pervious-area runoff volume.
    #[getter]
    fn pervious_runoff_volume(&self) -> f64 {
        self.values.pervious_runoff_volume
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    NodeStatisticsSnapshot,
    NodeStatisticsSnapshotRead,
    "NodeStatisticsSnapshot",
    "Aligned cumulative node statistics from one coherent owner acquisition.",
    [
        "object_ids",
        "average_depth",
        "maximum_depth",
        "maximum_depth_time",
        "maximum_reported_depth",
        "flooded_volume",
        "time_flooded",
        "time_surcharged",
        "time_courant_critical",
        "total_lateral_inflow",
        "maximum_lateral_inflow",
        "maximum_inflow",
        "maximum_overflow",
        "maximum_ponded_volume",
        "nonconverged_count",
        "maximum_inflow_time",
        "maximum_overflow_time"
    ]
);
#[pymethods]
impl NodeStatisticsSnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns time-weighted average depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of time-weighted average depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn average_depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.average_depth)
    }
    /// Returns maximum depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_depth)
    }
    /// Returns time of maximum depth.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of naive Python `datetime.datetime` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_depth_time(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        datetime_tuple(py, &self.values.maximum_depth_time)
    }
    /// Returns maximum reported depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum reported depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_reported_depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_reported_depth)
    }
    /// Returns flooded volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of flooded volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn flooded_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.flooded_volume)
    }
    /// Returns cumulative flooded duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_flooded(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_flooded_seconds)
    }
    /// Returns cumulative surcharged duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_surcharged(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_surcharged_seconds)
    }
    /// Returns cumulative Courant-critical duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_courant_critical(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_courant_critical_seconds)
    }
    /// Returns total lateral inflow volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of total lateral inflow volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn total_lateral_inflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.total_lateral_inflow)
    }
    /// Returns maximum lateral inflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum lateral inflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_lateral_inflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_lateral_inflow)
    }
    /// Returns maximum inflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum inflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_inflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_inflow)
    }
    /// Returns maximum overflow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum overflow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_overflow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_overflow)
    }
    /// Returns maximum ponded volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum ponded volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_ponded_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_ponded_volume)
    }
    /// Returns nonconverged solution count.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of nonconverged solution count aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn nonconverged_count(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        i32_tuple(py, &self.values.nonconverged_count)
    }
    /// Returns time of maximum inflow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of naive Python `datetime.datetime` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_inflow_time(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        datetime_tuple(py, &self.values.maximum_inflow_time)
    }
    /// Returns time of maximum overflow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of naive Python `datetime.datetime` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_overflow_time(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        datetime_tuple(py, &self.values.maximum_overflow_time)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    LinkQualitySnapshot,
    LinkQualitySnapshotRead,
    "LinkQualitySnapshot",
    "Pollutant-major current link quality in configured reporting units; each inner row follows object_ids.",
    [
        "object_ids",
        "pollutant_ids",
        "concentrations",
        "reactor_concentrations",
        "total_loads"
    ]
);
#[pymethods]
impl LinkQualitySnapshot {
    /// Returns canonical link identifiers defining each matrix's inner axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns canonical pollutant identifiers defining each matrix's outer axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical pollutant identifiers in configured pollutant order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn pollutant_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.pollutant_ids)
    }
    /// Returns concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.concentrations)
    }
    /// Returns reactor concentrations in each pollutant's configured concentration units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn reactor_concentrations(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.reactor_concentrations)
    }
    /// Returns cumulative loads in each pollutant's configured mass/count reporting units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable pollutant-major tuple matrix; outer rows follow `pollutant_ids` and inner values follow `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn total_loads(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_matrix(py, &self.values.total_loads)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    LinkStatisticsSnapshot,
    LinkStatisticsSnapshotRead,
    "LinkStatisticsSnapshot",
    "Aligned cumulative link statistics from one coherent owner acquisition.",
    [
        "object_ids",
        "maximum_flow",
        "maximum_flow_time",
        "maximum_velocity",
        "maximum_depth",
        "maximum_street_fill_fraction",
        "time_normal_flow",
        "time_inlet_control",
        "time_surcharged",
        "time_full_upstream",
        "time_full_downstream",
        "time_full_flow",
        "time_capacity_limited",
        "time_in_flow_class",
        "time_courant_critical",
        "flow_turns",
        "flow_turn_sign"
    ]
);
#[pymethods]
impl LinkStatisticsSnapshot {
    /// Returns canonical link identifiers defining each aligned column's outer axis.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns maximum flow rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum flow rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_flow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_flow)
    }
    /// Returns time of maximum flow.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of naive Python `datetime.datetime` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_flow_time(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        datetime_tuple(py, &self.values.maximum_flow_time)
    }
    /// Returns maximum velocity in feet/second or meters/second.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum velocity aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_velocity(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_velocity)
    }
    /// Returns maximum depth in feet or meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_depth(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_depth)
    }
    /// Returns the optional dimensionless maximum street fill fraction.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of optional maximum street fill fraction aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_street_fill_fraction(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        optional_float_tuple(py, &self.values.maximum_street_fill_fraction)
    }
    /// Returns cumulative normal-flow duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_normal_flow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_normal_flow_seconds)
    }
    /// Returns cumulative inlet-control duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_inlet_control(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_inlet_control_seconds)
    }
    /// Returns cumulative surcharged duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_surcharged(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_surcharged_seconds)
    }
    /// Returns cumulative upstream-full duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_upstream(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_full_upstream_seconds)
    }
    /// Returns cumulative downstream-full duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_downstream(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_full_downstream_seconds)
    }
    /// Returns cumulative full-flow duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_full_flow(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_full_flow_seconds)
    }
    /// Returns cumulative capacity-limited duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_capacity_limited(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_capacity_limited_seconds)
    }
    /// Returns durations in each FlowClass order.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable object-major tuple matrix; outer rows follow `object_ids` and inner timedeltas follow `FlowClass` enum order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_in_flow_class(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_matrix(py, &self.values.time_in_flow_class_seconds)
    }
    /// Returns cumulative Courant-critical duration.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of Python `datetime.timedelta` values aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn time_courant_critical(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        duration_tuple(py, &self.values.time_courant_critical_seconds)
    }
    /// Returns flow-direction turn count.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of flow-direction turn count aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn flow_turns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        i64_tuple(py, &self.values.flow_turns)
    }
    /// Returns latest flow-direction sign.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of latest flow-direction sign aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn flow_turn_sign(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        i32_tuple(py, &self.values.flow_turn_sign)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    SubcatchmentLidSnapshot,
    SubcatchmentLidSnapshotRead,
    "SubcatchmentLidSnapshot",
    "Current runtime values for one subcatchment LID group.",
    [
        "pervious_area",
        "flow_to_pervious_area",
        "old_drain_flow",
        "new_drain_flow"
    ]
);
#[pymethods]
impl SubcatchmentLidSnapshot {
    /// Returns pervious area outside LID controls in square feet or square meters.
    ///
    /// # Returns
    /// Python float containing pervious area outside LID controls.
    #[getter]
    fn pervious_area(&self) -> f64 {
        self.values.pervious_area
    }
    /// Returns flow routed to pervious area in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing flow routed to pervious area.
    #[getter]
    fn flow_to_pervious_area(&self) -> f64 {
        self.values.flow_to_pervious_area
    }
    /// Returns previous drain flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing previous drain flow rate.
    #[getter]
    fn old_drain_flow(&self) -> f64 {
        self.values.old_drain_flow
    }
    /// Returns current drain flow rate in the configured project flow units.
    ///
    /// # Returns
    /// Python float containing current drain flow rate.
    #[getter]
    fn new_drain_flow(&self) -> f64 {
        self.values.new_drain_flow
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    SubcatchmentStatisticsSnapshot,
    SubcatchmentStatisticsSnapshotRead,
    "SubcatchmentStatisticsSnapshot",
    "Aligned cumulative subcatchment statistics from one coherent acquisition.",
    [
        "object_ids",
        "precipitation",
        "runon_volume",
        "evaporation_volume",
        "infiltration_volume",
        "runoff_volume",
        "maximum_runoff",
        "impervious_runoff_volume",
        "pervious_runoff_volume"
    ]
);
#[pymethods]
impl SubcatchmentStatisticsSnapshot {
    /// Returns canonical object identifiers.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of canonical object identifiers in acquisition order.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn object_ids(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        string_tuple(py, &self.values.object_ids)
    }
    /// Returns precipitation depth in inches or millimeters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of precipitation depth aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn precipitation(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.precipitation)
    }
    /// Returns runon volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of runon volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runon_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.runon_volume)
    }
    /// Returns evaporation volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of evaporation volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn evaporation_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.evaporation_volume)
    }
    /// Returns infiltration volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of infiltration volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn infiltration_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.infiltration_volume)
    }
    /// Returns runoff volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of runoff volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runoff_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.runoff_volume)
    }
    /// Returns maximum runoff rate in the configured project flow units.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of maximum runoff rate aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_runoff(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.maximum_runoff)
    }
    /// Returns impervious-area runoff volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of impervious-area runoff volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn impervious_runoff_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.impervious_runoff_volume)
    }
    /// Returns pervious-area runoff volume in cubic feet or cubic meters.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Immutable tuple of pervious-area runoff volume aligned by `object_ids`.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn pervious_runoff_volume(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        float_tuple(py, &self.values.pervious_runoff_volume)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    RoutingDiagnostics,
    RoutingDiagnosticsRead,
    "RoutingDiagnostics",
    "Routing-step diagnostics from one coherent owner acquisition.",
    [
        "average_time_step",
        "minimum_time_step",
        "maximum_time_step",
        "step_count",
        "nonconverged_step_count",
        "nonconverged_step_percentage",
        "average_iterations"
    ]
);
#[pymethods]
impl RoutingDiagnostics {
    /// Returns average routing time step.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the average routing time step.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn average_time_step(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.average_time_step_seconds)
    }
    /// Returns minimum routing time step.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the minimum routing time step.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn minimum_time_step(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.minimum_time_step_seconds)
    }
    /// Returns maximum routing time step.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Python `datetime.timedelta` for the maximum routing time step.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn maximum_time_step(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        duration(py, self.values.maximum_time_step_seconds)
    }
    /// Returns routing step count.
    ///
    /// # Returns
    /// Python integer containing routing step count.
    #[getter]
    fn step_count(&self) -> i32 {
        self.values.step_count
    }
    /// Returns nonconverged routing step count.
    ///
    /// # Returns
    /// Python integer containing nonconverged routing step count.
    #[getter]
    fn nonconverged_step_count(&self) -> i64 {
        self.values.nonconverged_step_count
    }
    /// Returns percentage of routing steps that did not converge.
    ///
    /// # Returns
    /// Python float containing percentage of routing steps that did not converge.
    #[getter]
    fn nonconverged_step_percentage(&self) -> f64 {
        self.values.nonconverged_step_percentage
    }
    /// Returns average routing iterations per step.
    ///
    /// # Returns
    /// Python float containing average routing iterations per step.
    #[getter]
    fn average_iterations(&self) -> f64 {
        self.values.average_iterations
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    RoutingTotals,
    RoutingTotalsRead,
    "RoutingTotals",
    "System routing-continuity totals in configured project volume units.",
    [
        "dry_weather_inflow",
        "wet_weather_inflow",
        "groundwater_inflow",
        "rdii_inflow",
        "external_inflow",
        "flooding",
        "outflow",
        "evaporation_loss",
        "seepage_loss",
        "reaction_loss",
        "initial_storage",
        "final_storage",
        "continuity_error"
    ]
);
#[pymethods]
impl RoutingTotals {
    /// Returns dry-weather inflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing dry-weather inflow volume.
    #[getter]
    fn dry_weather_inflow(&self) -> f64 {
        self.values.dry_weather_inflow
    }
    /// Returns wet-weather inflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing wet-weather inflow volume.
    #[getter]
    fn wet_weather_inflow(&self) -> f64 {
        self.values.wet_weather_inflow
    }
    /// Returns groundwater inflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing groundwater inflow volume.
    #[getter]
    fn groundwater_inflow(&self) -> f64 {
        self.values.groundwater_inflow
    }
    /// Returns rainfall-derived inflow and infiltration volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing rainfall-derived inflow and infiltration volume.
    #[getter]
    fn rdii_inflow(&self) -> f64 {
        self.values.rdii_inflow
    }
    /// Returns external inflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing external inflow volume.
    #[getter]
    fn external_inflow(&self) -> f64 {
        self.values.external_inflow
    }
    /// Returns flooding volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing flooding volume.
    #[getter]
    fn flooding(&self) -> f64 {
        self.values.flooding
    }
    /// Returns outflow volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing outflow volume.
    #[getter]
    fn outflow(&self) -> f64 {
        self.values.outflow
    }
    /// Returns evaporation loss volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing evaporation loss volume.
    #[getter]
    fn evaporation_loss(&self) -> f64 {
        self.values.evaporation_loss
    }
    /// Returns seepage loss volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing seepage loss volume.
    #[getter]
    fn seepage_loss(&self) -> f64 {
        self.values.seepage_loss
    }
    /// Returns reaction loss volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing reaction loss volume.
    #[getter]
    fn reaction_loss(&self) -> f64 {
        self.values.reaction_loss
    }
    /// Returns initial stored volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing initial storage.
    #[getter]
    fn initial_storage(&self) -> f64 {
        self.values.initial_storage
    }
    /// Returns final stored volume in cubic feet or cubic meters.
    ///
    /// # Returns
    /// Python float containing final stored volume.
    #[getter]
    fn final_storage(&self) -> f64 {
        self.values.final_storage
    }
    /// Returns the dimensionless continuity error in percent.
    ///
    /// # Returns
    /// Python float containing continuity error percentage.
    #[getter]
    fn continuity_error(&self) -> f64 {
        self.values.continuity_error
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    RunoffTotals,
    RunoffTotalsRead,
    "RunoffTotals",
    "System runoff-continuity totals in configured report rain-depth units.",
    [
        "rainfall",
        "evaporation_loss",
        "infiltration_loss",
        "runoff",
        "lid_drain_flow",
        "outfall_runon",
        "initial_surface_storage",
        "final_surface_storage",
        "initial_snow_cover",
        "final_snow_cover",
        "snow_removed",
        "continuity_error"
    ]
);
#[pymethods]
impl RunoffTotals {
    /// Returns rainfall depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing rainfall depth.
    #[getter]
    fn rainfall(&self) -> f64 {
        self.values.rainfall
    }
    /// Returns evaporation loss depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing evaporation loss depth.
    #[getter]
    fn evaporation_loss(&self) -> f64 {
        self.values.evaporation_loss
    }
    /// Returns infiltration loss depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing infiltration loss depth.
    #[getter]
    fn infiltration_loss(&self) -> f64 {
        self.values.infiltration_loss
    }
    /// Returns runoff depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing runoff depth.
    #[getter]
    fn runoff(&self) -> f64 {
        self.values.runoff
    }
    /// Returns LID drain-flow depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing LID drain-flow depth.
    #[getter]
    fn lid_drain_flow(&self) -> f64 {
        self.values.lid_drainage
    }
    /// Returns outfall runon depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing outfall runon depth.
    #[getter]
    fn outfall_runon(&self) -> f64 {
        self.values.outfall_runon
    }
    /// Returns initial surface-storage depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing initial surface-storage depth.
    #[getter]
    fn initial_surface_storage(&self) -> f64 {
        self.values.initial_surface_storage
    }
    /// Returns final surface-storage depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing final surface-storage depth.
    #[getter]
    fn final_surface_storage(&self) -> f64 {
        self.values.final_surface_storage
    }
    /// Returns initial snow-cover depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing initial snow-cover depth.
    #[getter]
    fn initial_snow_cover(&self) -> f64 {
        self.values.initial_snow_cover
    }
    /// Returns final snow-cover depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing final snow-cover depth.
    #[getter]
    fn final_snow_cover(&self) -> f64 {
        self.values.final_snow_cover
    }
    /// Returns removed snow depth in inches or millimeters.
    ///
    /// # Returns
    /// Python float containing removed snow depth.
    #[getter]
    fn snow_removed(&self) -> f64 {
        self.values.snow_removed
    }
    /// Returns the dimensionless continuity error in percent.
    ///
    /// # Returns
    /// Python float containing continuity error percentage.
    #[getter]
    fn continuity_error(&self) -> f64 {
        self.values.continuity_error
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    QualityBalance,
    QualityBalanceRead,
    "QualityBalance",
    "Continuity balance for one pollutant in configured mass/count reporting units.",
    ["continuity_error", "seepage_loss"]
);
#[pymethods]
impl QualityBalance {
    /// Returns the dimensionless pollutant continuity error in percent.
    ///
    /// # Returns
    /// Python float containing signed continuity error percentage.
    #[getter]
    fn continuity_error(&self) -> f64 {
        self.values.continuity_error
    }
    /// Returns seepage pollutant mass in pounds, kilograms, or safe log10(count).
    ///
    /// # Returns
    /// Python float using the pollutant's configured reporting convention.
    #[getter]
    fn seepage_loss(&self) -> f64 {
        self.values.seepage_loss
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

record_type!(
    SimulationStatistics,
    SimulationStatisticsRead,
    "SimulationStatistics",
    "Coherent system continuity statistics from one owner acquisition.",
    [
        "routing_totals",
        "runoff_totals",
        "groundwater_continuity_error",
        "quality_continuity_error",
        "routing_diagnostics",
        "quality_balances"
    ]
);
#[pymethods]
impl SimulationStatistics {
    /// Returns routing continuity totals.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Detached immutable routing continuity totals record.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn routing_totals(&self, py: Python<'_>) -> PyResult<Py<RoutingTotals>> {
        Py::new(py, RoutingTotals::new(self.values.routing_totals))
    }
    /// Returns runoff continuity totals.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Detached immutable runoff continuity totals record.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn runoff_totals(&self, py: Python<'_>) -> PyResult<Py<RunoffTotals>> {
        Py::new(py, RunoffTotals::new(self.values.runoff_totals))
    }
    /// Returns groundwater continuity error percentage.
    ///
    /// # Returns
    /// Python float containing groundwater continuity error percentage.
    #[getter]
    fn groundwater_continuity_error(&self) -> f64 {
        self.values.groundwater_continuity_error
    }
    /// Returns aggregate quality continuity error percentage.
    ///
    /// # Returns
    /// Python float containing aggregate quality continuity error percentage.
    #[getter]
    fn quality_continuity_error(&self) -> f64 {
        self.values.quality_continuity_error
    }
    /// Returns routing-step diagnostics.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Detached immutable routing-step diagnostics record.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn routing_diagnostics(&self, py: Python<'_>) -> PyResult<Py<RoutingDiagnostics>> {
        Py::new(py, RoutingDiagnostics::new(self.values.routing_diagnostics))
    }
    /// Returns canonical pollutant quality balances.
    ///
    /// # Arguments
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Read-only insertion-ordered mapping of canonical pollutant quality balances.
    ///
    /// # Errors
    /// Returns a Python allocation, conversion, or import error.
    #[getter]
    fn quality_balances(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dictionary = PyDict::new(py);
        for (identifier, values) in &self.values.quality_balances {
            dictionary.set_item(identifier, Py::new(py, QualityBalance::new(*values))?)?;
        }
        mapping_proxy(py, &dictionary)
    }
    /// Formats the complete immutable record through its public fields.
    ///
    /// # Arguments
    /// * `slf` - Borrowed native record.
    /// * `py` - Attached Python interpreter token.
    ///
    /// # Returns
    /// Dataclass-like record representation.
    ///
    /// # Errors
    /// Returns a Python attribute or representation error.
    fn __repr__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
        Self::repr(slf, py)
    }
}

/// Converts borrowed strings into an owned Python tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed strings in canonical order.
///
/// # Returns
/// Immutable owned Python string tuple.
///
/// # Errors
/// Returns a python tuple allocation or string conversion error.
fn string_tuple(py: Python<'_>, values: &[String]) -> PyResult<Py<PyTuple>> {
    Ok(PyTuple::new(py, values.iter().map(String::as_str))?.unbind())
}

/// Converts borrowed floats into an owned Python tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed floating-point values in canonical order.
///
/// # Returns
/// Immutable owned Python float tuple.
///
/// # Errors
/// Returns a python tuple allocation or float conversion error.
fn float_tuple(py: Python<'_>, values: &[f64]) -> PyResult<Py<PyTuple>> {
    Ok(PyTuple::new(py, values.iter().copied())?.unbind())
}

/// Converts borrowed signed 32-bit integers into an owned Python tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed 32-bit integer values in canonical order.
///
/// # Returns
/// Immutable owned Python integer tuple.
///
/// # Errors
/// Returns a python tuple allocation or integer conversion error.
fn i32_tuple(py: Python<'_>, values: &[i32]) -> PyResult<Py<PyTuple>> {
    Ok(PyTuple::new(py, values.iter().copied())?.unbind())
}

/// Converts borrowed signed 64-bit integers into an owned Python tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed 64-bit integer values in canonical order.
///
/// # Returns
/// Immutable owned Python integer tuple.
///
/// # Errors
/// Returns a python tuple allocation or integer conversion error.
fn i64_tuple(py: Python<'_>, values: &[i64]) -> PyResult<Py<PyTuple>> {
    Ok(PyTuple::new(py, values.iter().copied())?.unbind())
}

/// Converts pollutant-major float rows into an immutable tuple matrix.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed pollutant-major floating-point rows.
///
/// # Returns
/// Immutable owned Python tuple matrix.
///
/// # Errors
/// Returns a python tuple allocation or float conversion error.
fn float_matrix(py: Python<'_>, values: &[Vec<f64>]) -> PyResult<Py<PyTuple>> {
    let rows = values
        .iter()
        .map(|row| float_tuple(py, row))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, rows)?.unbind())
}

/// Converts optional floats into an immutable tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed optional floating-point values.
///
/// # Returns
/// Immutable owned Python tuple containing floats and `None`.
///
/// # Errors
/// Returns a python tuple allocation or value conversion error.
fn optional_float_tuple(py: Python<'_>, values: &[Option<f64>]) -> PyResult<Py<PyTuple>> {
    Ok(PyTuple::new(py, values.iter().copied())?.unbind())
}

/// Converts second values into an immutable tuple of timedeltas.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed durations in seconds.
///
/// # Returns
/// Immutable owned Python timedelta tuple.
///
/// # Errors
/// Returns a python import, timedelta construction, or tuple allocation error.
fn duration_tuple(py: Python<'_>, values: &[f64]) -> PyResult<Py<PyTuple>> {
    let converted = values
        .iter()
        .map(|seconds| duration(py, *seconds))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, converted)?.unbind())
}

/// Converts optional second values into timedeltas and `None`.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed optional durations in seconds.
///
/// # Returns
/// Immutable owned Python tuple containing timedeltas and `None`.
///
/// # Errors
/// Returns a python import, timedelta construction, or tuple allocation error.
fn optional_duration_tuple(py: Python<'_>, values: &[Option<f64>]) -> PyResult<Py<PyTuple>> {
    let mut converted = Vec::with_capacity(values.len());
    for value in values {
        converted.push(match value {
            Some(seconds) => duration(py, *seconds)?,
            None => py.None(),
        });
    }
    Ok(PyTuple::new(py, converted)?.unbind())
}

/// Converts fixed duration rows into an immutable tuple matrix.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed duration rows in seconds.
///
/// # Returns
/// Immutable owned Python timedelta tuple matrix.
///
/// # Errors
/// Returns a python import, timedelta construction, or tuple allocation error.
fn duration_matrix<const N: usize>(py: Python<'_>, values: &[[f64; N]]) -> PyResult<Py<PyTuple>> {
    let rows = values
        .iter()
        .map(|row| duration_tuple(py, row))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, rows)?.unbind())
}

/// Converts SWMM datetimes into an immutable Python datetime tuple.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Borrowed SWMM date-time values.
///
/// # Returns
/// Immutable owned tuple of naive whole-second Python datetimes.
///
/// # Errors
/// Returns a python import, datetime construction, or tuple allocation error.
fn datetime_tuple(py: Python<'_>, values: &[DateTime]) -> PyResult<Py<PyTuple>> {
    let converted = values
        .iter()
        .map(|value| datetime(py, *value))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, converted)?.unbind())
}

/// Constructs one Python timedelta using Python's float-second rounding.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `seconds` - Duration in seconds.
///
/// # Returns
/// Detached Python timedelta rounded to the nearest microsecond by Python.
///
/// # Errors
/// Returns a python import or timedelta construction error.
fn duration(py: Python<'_>, seconds: f64) -> PyResult<Py<PyAny>> {
    Ok(py
        .import("datetime")?
        .getattr("timedelta")?
        .call1((0, seconds))?
        .unbind())
}

/// Constructs one whole-second Python datetime from a SWMM datetime.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - SWMM date-time value.
///
/// # Returns
/// Detached naive Python datetime with solver-compatible whole-second semantics.
///
/// # Errors
/// Returns a python import or datetime construction error.
fn datetime(py: Python<'_>, value: DateTime) -> PyResult<Py<PyAny>> {
    let parts = datetime_parts(value);
    Ok(py
        .import("datetime")?
        .getattr("datetime")?
        .call1(parts)?
        .unbind())
}

/// Constructs a read-only insertion-ordered float mapping.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `values` - Canonically ordered identifier and float pairs.
///
/// # Returns
/// Detached read-only insertion-ordered mapping.
///
/// # Errors
/// Returns a python dictionary, mapping proxy, or value conversion error.
fn float_mapping(py: Python<'_>, values: &[(String, f64)]) -> PyResult<Py<PyAny>> {
    let dictionary = PyDict::new(py);
    for (identifier, value) in values {
        dictionary.set_item(identifier, value)?;
    }
    mapping_proxy(py, &dictionary)
}

/// Wraps a detached Python dictionary in `types.MappingProxyType`.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `dictionary` - Detached Python dictionary to expose read-only.
///
/// # Returns
/// Detached read-only mapping proxy.
///
/// # Errors
/// Returns a python import or mapping proxy construction error.
fn mapping_proxy(py: Python<'_>, dictionary: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    Ok(py
        .import("types")?
        .getattr("MappingProxyType")?
        .call1((dictionary,))?
        .unbind())
}

/// Compares two native records through their final public immutable attributes.
///
/// # Arguments
/// * `left` - Native record on the left side.
/// * `right` - Native record on the right side.
/// * `fields` - Public fields in canonical schema order.
///
/// # Returns
/// `true` when every exposed field compares equal under Python value semantics.
///
/// # Errors
/// Returns a Python attribute or comparison error.
fn record_equal(
    left: &Bound<'_, PyAny>,
    right: &Bound<'_, PyAny>,
    fields: &[&str],
) -> PyResult<bool> {
    for field in fields {
        if !left.getattr(*field)?.eq(right.getattr(*field)?)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Formats a native record through its public immutable attributes.
///
/// # Arguments
/// * `object` - Native record whose attributes are rendered.
/// * `class_name` - Public class name used as the representation prefix.
/// * `fields` - Public fields in canonical schema order.
///
/// # Returns
/// Dataclass-like representation containing every public field.
///
/// # Errors
/// Returns a Python attribute or representation error.
fn record_repr(object: &Bound<'_, PyAny>, class_name: &str, fields: &[&str]) -> PyResult<String> {
    let mut rendered = Vec::with_capacity(fields.len());
    for field in fields {
        let value = object.getattr(*field)?;
        rendered.push(format!("{field}={}", value.repr()?.to_string_lossy()));
    }
    Ok(format!("{class_name}({})", rendered.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates node statistics with one controllable public date-time value.
    ///
    /// # Arguments
    /// * `maximum_depth_time` - SWMM date-time stored before public whole-second conversion.
    ///
    /// # Returns
    /// Minimal node statistics read used by native-record equality tests.
    fn node_statistics(maximum_depth_time: DateTime) -> NodeStatisticsRead {
        NodeStatisticsRead {
            average_depth: 0.0,
            maximum_depth: 0.0,
            maximum_depth_time,
            maximum_reported_depth: 0.0,
            flooded_volume: 0.0,
            time_flooded_seconds: 0.0,
            time_surcharged_seconds: 0.0,
            time_courant_critical_seconds: 0.0,
            total_lateral_inflow: 0.0,
            maximum_lateral_inflow: 0.0,
            maximum_inflow: 0.0,
            maximum_overflow: 0.0,
            maximum_ponded_volume: 0.0,
            nonconverged_count: 0,
            maximum_inflow_time: DateTime::ZERO,
            maximum_overflow_time: DateTime::ZERO,
        }
    }

    /// Verifies equality follows converted Python values instead of raw solver storage.
    #[test]
    fn record_equality_uses_public_datetime_duration_and_mapping_semantics() {
        Python::initialize();
        Python::attach(|py| {
            let rounded_duration =
                duration(py, 1.2345675).expect("fractional duration should convert");
            let total_seconds = rounded_duration
                .bind(py)
                .call_method0("total_seconds")
                .expect("timedelta should report total seconds")
                .extract::<f64>()
                .expect("total seconds should be a float");
            assert_eq!(total_seconds, 1.234568);

            let base = datetime_encodeDate(2024, 1, 1);
            let rounded_datetime = datetime(py, base + (59.6 / 86400.0))
                .expect("boundary-sensitive datetime should convert");
            assert_eq!(
                rounded_datetime
                    .bind(py)
                    .getattr("minute")
                    .expect("datetime should expose minutes")
                    .extract::<u8>()
                    .expect("minute should be an integer"),
                1
            );
            assert_eq!(
                rounded_datetime
                    .bind(py)
                    .getattr("second")
                    .expect("datetime should expose seconds")
                    .extract::<u8>()
                    .expect("second should be an integer"),
                0
            );
            assert_eq!(
                rounded_datetime
                    .bind(py)
                    .getattr("microsecond")
                    .expect("datetime should expose microseconds")
                    .extract::<u32>()
                    .expect("microsecond should be an integer"),
                0
            );

            let first_diagnostics = Py::new(
                py,
                RoutingDiagnostics::new(RoutingDiagnosticsRead {
                    average_time_step_seconds: 1.0000001,
                    minimum_time_step_seconds: 2.0,
                    maximum_time_step_seconds: 3.0,
                    step_count: 1,
                    nonconverged_step_count: 0,
                    nonconverged_step_percentage: 0.0,
                    average_iterations: 1.0,
                }),
            )
            .expect("first diagnostics record should be created");
            let second_diagnostics = Py::new(
                py,
                RoutingDiagnostics::new(RoutingDiagnosticsRead {
                    average_time_step_seconds: 1.0000002,
                    minimum_time_step_seconds: 2.0,
                    maximum_time_step_seconds: 3.0,
                    step_count: 1,
                    nonconverged_step_count: 0,
                    nonconverged_step_percentage: 0.0,
                    average_iterations: 1.0,
                }),
            )
            .expect("second diagnostics record should be created");
            assert!(
                first_diagnostics
                    .bind(py)
                    .eq(second_diagnostics.bind(py))
                    .expect("duration-backed record equality should succeed")
            );

            let first_node = Py::new(
                py,
                NodeStatistics::new(node_statistics(base + (0.1 / 86400.0))),
            )
            .expect("first node record should be created");
            let second_node = Py::new(
                py,
                NodeStatistics::new(node_statistics(base + (0.2 / 86400.0))),
            )
            .expect("second node record should be created");
            assert!(
                first_node
                    .bind(py)
                    .eq(second_node.bind(py))
                    .expect("datetime-backed record equality should succeed")
            );

            let first_outfall = Py::new(
                py,
                OutfallStatistics::new(OutfallStatisticsRead {
                    average_flow: 1.0,
                    maximum_flow: 2.0,
                    pollutant_loads: vec![("TSS".to_string(), 3.0), ("Count".to_string(), 4.0)],
                    period_count: 1,
                }),
            )
            .expect("first outfall record should be created");
            let second_outfall = Py::new(
                py,
                OutfallStatistics::new(OutfallStatisticsRead {
                    average_flow: 1.0,
                    maximum_flow: 2.0,
                    pollutant_loads: vec![("Count".to_string(), 4.0), ("TSS".to_string(), 3.0)],
                    period_count: 1,
                }),
            )
            .expect("second outfall record should be created");
            assert!(
                first_outfall
                    .bind(py)
                    .eq(second_outfall.bind(py))
                    .expect("mapping-backed record equality should succeed")
            );

            let unequal = Py::new(
                py,
                RoutingDiagnostics::new(RoutingDiagnosticsRead {
                    average_time_step_seconds: 1.000002,
                    minimum_time_step_seconds: 2.0,
                    maximum_time_step_seconds: 3.0,
                    step_count: 1,
                    nonconverged_step_count: 0,
                    nonconverged_step_percentage: 0.0,
                    average_iterations: 1.0,
                }),
            )
            .expect("unequal diagnostics record should be created");
            assert!(
                !first_diagnostics
                    .bind(py)
                    .eq(unequal.bind(py))
                    .expect("unequal record comparison should succeed")
            );

            let nan_record = Py::new(
                py,
                RoutingDiagnostics::new(RoutingDiagnosticsRead {
                    average_time_step_seconds: 1.0,
                    minimum_time_step_seconds: 2.0,
                    maximum_time_step_seconds: 3.0,
                    step_count: 1,
                    nonconverged_step_count: 0,
                    nonconverged_step_percentage: 0.0,
                    average_iterations: f64::NAN,
                }),
            )
            .expect("NaN-bearing diagnostics record should be created");
            let distinct_nan_record = Py::new(
                py,
                RoutingDiagnostics::new(RoutingDiagnosticsRead {
                    average_time_step_seconds: 1.0,
                    minimum_time_step_seconds: 2.0,
                    maximum_time_step_seconds: 3.0,
                    step_count: 1,
                    nonconverged_step_count: 0,
                    nonconverged_step_percentage: 0.0,
                    average_iterations: f64::NAN,
                }),
            )
            .expect("distinct NaN-bearing diagnostics record should be created");
            assert!(
                nan_record
                    .bind(py)
                    .eq(nan_record.bind(py))
                    .expect("record identity comparison should succeed")
            );
            assert!(
                !nan_record
                    .bind(py)
                    .eq(distinct_nan_record.bind(py))
                    .expect("distinct NaN-bearing record comparison should succeed")
            );
        });
    }
}
