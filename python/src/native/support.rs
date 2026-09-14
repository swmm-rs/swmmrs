use super::*;
use pyo3::types::PyModule;

/// Converts a pure-Rust detached failure after Python is reattached.
///
/// # Arguments
/// * `operation` - Lifecycle operation that failed.
/// * `failure` - Owned failure returned after releasing the owner mutex.
///
/// # Returns
/// Final public exception for the detached failure category.
pub(super) fn detached_error(operation: &str, failure: DetachedFailure) -> PyErr {
    match failure {
        DetachedFailure::Solver(failure_operation, error) => native_error(failure_operation, error),
        DetachedFailure::Stale => stale_error(operation),
        DetachedFailure::Poisoned => internal_error("lock", "simulation owner is unavailable"),
        DetachedFailure::Panicked => internal_error(operation, "native solver panicked"),
    }
}

/// Identifies one exact public value class captured at package import.
#[derive(Clone, Copy)]
pub(super) enum PublicType {
    Float,
    Int,
    Str,
    Tuple,
    DateTime,
    Timedelta,
    CustomEllipseModel,
    InertiaDamping,
    NormalFlowLimit,
    OrificeKind,
    OutletHeadBasis,
    RoadSurface,
    StandardCrossSectionShape,
    SurchargeMethod,
    WeirKind,
    CircularCrossSection,
    CustomCrossSection,
    DividerRule,
    FunctionalOutletRating,
    IrregularCrossSection,
    OutfallBoundary,
    StandardCrossSection,
    StorageExfiltration,
    StorageShape,
    StreetCrossSection,
    TabularOutletRating,
}

impl PublicType {
    /// Returns the stable public class name used in validation diagnostics.
    ///
    /// # Returns
    /// Public Python class name.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Int => "int",
            Self::Str => "str",
            Self::Tuple => "tuple",
            Self::DateTime => "datetime",
            Self::Timedelta => "timedelta",
            Self::CustomEllipseModel => "CustomEllipseModel",
            Self::InertiaDamping => "InertiaDamping",
            Self::NormalFlowLimit => "NormalFlowLimit",
            Self::OrificeKind => "OrificeKind",
            Self::OutletHeadBasis => "OutletHeadBasis",
            Self::RoadSurface => "RoadSurface",
            Self::StandardCrossSectionShape => "StandardCrossSectionShape",
            Self::SurchargeMethod => "SurchargeMethod",
            Self::WeirKind => "WeirKind",
            Self::CircularCrossSection => "CircularCrossSection",
            Self::CustomCrossSection => "CustomCrossSection",
            Self::DividerRule => "DividerRule",
            Self::FunctionalOutletRating => "FunctionalOutletRating",
            Self::IrregularCrossSection => "IrregularCrossSection",
            Self::OutfallBoundary => "OutfallBoundary",
            Self::StandardCrossSection => "StandardCrossSection",
            Self::StorageExfiltration => "StorageExfiltration",
            Self::StorageShape => "StorageShape",
            Self::StreetCrossSection => "StreetCrossSection",
            Self::TabularOutletRating => "TabularOutletRating",
        }
    }
}

/// Trusted public exception classes captured at package import.
struct ExceptionTypes {
    configuration_diagnostic: Py<PyAny>,
    configuration_error: Py<PyAny>,
    configuration_object_identity: Py<PyAny>,
    internal_simulation_error: Py<PyAny>,
    lifecycle_error: Py<PyAny>,
    solver_error: Py<PyAny>,
    stale_view_error: Py<PyAny>,
    validation_error: Py<PyAny>,
}

/// Trusted exact-value classes captured at package import.
struct ExactTypes {
    float: Py<PyAny>,
    int: Py<PyAny>,
    str_: Py<PyAny>,
    tuple: Py<PyAny>,
    datetime: Py<PyAny>,
    timedelta: Py<PyAny>,
    custom_ellipse_model: Py<PyAny>,
    inertia_damping: Py<PyAny>,
    normal_flow_limit: Py<PyAny>,
    orifice_kind: Py<PyAny>,
    outlet_head_basis: Py<PyAny>,
    road_surface: Py<PyAny>,
    standard_cross_section_shape: Py<PyAny>,
    surcharge_method: Py<PyAny>,
    weir_kind: Py<PyAny>,
    circular_cross_section: Py<PyAny>,
    custom_cross_section: Py<PyAny>,
    divider_rule: Py<PyAny>,
    functional_outlet_rating: Py<PyAny>,
    irregular_cross_section: Py<PyAny>,
    outfall_boundary: Py<PyAny>,
    standard_cross_section: Py<PyAny>,
    storage_exfiltration: Py<PyAny>,
    storage_shape: Py<PyAny>,
    street_cross_section: Py<PyAny>,
    tabular_outlet_rating: Py<PyAny>,
}

/// Trusted Live View classes captured at package import.
struct ViewTypes {
    rain_gage: Py<PyAny>,
    subcatchment: Py<PyAny>,
    pollutant: Py<PyAny>,
    land_use: Py<PyAny>,
    time_pattern: Py<PyAny>,
    curve: Py<PyAny>,
    time_series: Py<PyAny>,
    control_rule: Py<PyAny>,
    transect: Py<PyAny>,
    aquifer: Py<PyAny>,
    amm_model: Py<PyAny>,
    unit_hydrograph: Py<PyAny>,
    snowmelt_parameter_set: Py<PyAny>,
    custom_shape: Py<PyAny>,
    lid_control: Py<PyAny>,
    street: Py<PyAny>,
    inlet_design: Py<PyAny>,
    junction: Py<PyAny>,
    outfall: Py<PyAny>,
    storage: Py<PyAny>,
    divider: Py<PyAny>,
    conduit: Py<PyAny>,
    pump: Py<PyAny>,
    orifice: Py<PyAny>,
    weir: Py<PyAny>,
    outlet: Py<PyAny>,
    inlet: Py<PyAny>,
}

/// Stores trusted package classes and private constructor tokens captured at package import.
struct PublicTypes {
    exact: ExactTypes,
    exceptions: ExceptionTypes,
    solver_error_code: Py<PyAny>,
    views: ViewTypes,
    view_token: Py<PyAny>,
    inlet_token: Py<PyAny>,
}

static PUBLIC_TYPES: PyOnceLock<PublicTypes> = PyOnceLock::new();

/// Absolute project paths retained by one native owner generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OwnerPaths {
    pub(super) input: String,
    pub(super) report: String,
    pub(super) output: Option<String>,
    pub(super) hotstart: Option<String>,
    pub(super) input_key: String,
    pub(super) report_key: String,
    pub(super) output_key: Option<String>,
    pub(super) hotstart_key: Option<String>,
}

impl OwnerPaths {
    /// Returns whether one shallow lexical path key collides with a project-owned artifact.
    ///
    /// # Arguments
    /// * `key` - `normcase(normpath(...))` candidate identity.
    /// * `include_hotstart` - Whether persistent hotstart input participates in the check.
    ///
    /// # Returns
    /// `true` when the candidate matches a retained owner path.
    pub(super) fn collides(&self, key: &str, include_hotstart: bool) -> bool {
        key == self.input_key
            || key == self.report_key
            || self.output_key.as_deref() == Some(key)
            || (include_hotstart && self.hotstart_key.as_deref() == Some(key))
    }
}

/// Extracts one Python path using the public `os.fspath` and call-time anchoring policy.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Raw `str | os.PathLike[str]` value.
/// * `name` - Public argument name used in diagnostics.
///
/// # Returns
/// Absolute lexical path string captured at this call.
///
/// # Errors
/// Returns `ValidationError` when the value is not string-like or resolves to bytes, and
/// propagates non-type failures raised by user-defined `__fspath__` methods.
pub(super) fn absolute_path(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    name: &str,
) -> PyResult<String> {
    let os = py.import("os")?;
    let raw = match os.getattr("fspath")?.call1((value,)) {
        Ok(raw) => raw,
        Err(error) if error.is_instance_of::<PyTypeError>(py) => {
            return Err(validation_error(&format!(
                "{name} must be str or os.PathLike[str]"
            )));
        }
        Err(error) => return Err(error),
    };
    if !raw.is_instance_of::<PyString>() {
        return Err(validation_error(&format!(
            "{name} must resolve to str, not bytes"
        )));
    }
    let pathlib = py.import("pathlib")?;
    let path = pathlib.getattr("Path")?.call1((raw,))?;
    let absolute = if path.call_method0("is_absolute")?.is_truthy()? {
        path
    } else {
        pathlib
            .getattr("Path")?
            .call_method0("cwd")?
            .call_method1("__truediv__", (path,))?
    };
    absolute.str()?.extract()
}

/// Extracts and validates the three paths used to open one project generation.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `input` - Raw input path.
/// * `report` - Optional raw report path.
/// * `output` - Optional raw retained output path.
///
/// # Returns
/// Absolute native owner path record with scratch output represented by `None`.
///
/// # Errors
/// Returns `ValidationError` for invalid path values or shallow lexical collisions.
pub(super) fn project_paths(
    py: Python<'_>,
    input: &Bound<'_, PyAny>,
    report: Option<&Bound<'_, PyAny>>,
    output: Option<&Bound<'_, PyAny>>,
) -> PyResult<OwnerPaths> {
    let input = absolute_path(py, input, "input_path")?;
    let report = if let Some(report) = report {
        absolute_path(py, report, "report_path")?
    } else {
        py.import("pathlib")?
            .getattr("Path")?
            .call1((&input,))?
            .call_method1("with_suffix", (".rpt",))?
            .str()?
            .extract()?
    };
    let output = output
        .map(|output| absolute_path(py, output, "output_path"))
        .transpose()?;
    reject_project_collisions(py, &input, &report, output.as_deref())?;
    retained_paths(py, input, report, output)
}

/// Builds retained owner metadata and its shallow lexical collision identities.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `input` - Absolute input path.
/// * `report` - Absolute report path.
/// * `output` - Optional absolute retained output path.
///
/// # Returns
/// Native owner metadata with no configured hotstart path.
///
/// # Errors
/// Returns a Python exception when lexical path identity computation fails.
pub(super) fn retained_paths(
    py: Python<'_>,
    input: String,
    report: String,
    output: Option<String>,
) -> PyResult<OwnerPaths> {
    Ok(OwnerPaths {
        input_key: path_key(py, &input)?,
        report_key: path_key(py, &report)?,
        output_key: output
            .as_deref()
            .map(|path| path_key(py, path))
            .transpose()?,
        input,
        report,
        output,
        hotstart: None,
        hotstart_key: None,
    })
}

/// Computes the public shallow lexical identity for one absolute path.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `path` - Absolute lexical path string.
///
/// # Returns
/// `normcase(normpath(path))` identity string.
///
/// # Errors
/// Returns a Python exception only when the standard-library path operation fails.
pub(super) fn path_key(py: Python<'_>, path: &str) -> PyResult<String> {
    let os_path = py.import("os")?.getattr("path")?;
    let normalized = os_path.getattr("normpath")?.call1((path,))?;
    os_path.getattr("normcase")?.call1((normalized,))?.extract()
}

/// Rejects shallow lexical collisions among project-owned paths.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `input` - Absolute input path.
/// * `report` - Absolute report path.
/// * `output` - Optional absolute retained output path.
///
/// # Errors
/// Returns `ValidationError` when any supplied paths collide.
pub(super) fn reject_project_collisions(
    py: Python<'_>,
    input: &str,
    report: &str,
    output: Option<&str>,
) -> PyResult<()> {
    let mut keys = vec![path_key(py, input)?, path_key(py, report)?];
    if let Some(output) = output {
        keys.push(path_key(py, output)?);
    }
    let distinct = keys.iter().collect::<std::collections::HashSet<_>>().len();
    if distinct != keys.len() {
        return Err(validation_error(&format!(
            "input, report, and output paths must be distinct: attempted input {input}, report {report}, output {}",
            output.unwrap_or("<scratch>")
        )));
    }
    Ok(())
}

/// Extracts an exact Python boolean without accepting integer subclasses.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Raw Python value.
/// * `name` - Public argument name.
///
/// # Returns
/// Extracted boolean.
///
/// # Errors
/// Returns `ValidationError` unless `value` has exact `bool` type.
pub(super) fn exact_bool(py: Python<'_>, value: &Bound<'_, PyAny>, name: &str) -> PyResult<bool> {
    if !value.get_type().is(py.get_type::<PyBool>()) {
        return Err(validation_error(&format!("{name} must be bool")));
    }
    value.extract()
}

/// Extracts a positive whole-second duration in the native integer range.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `value` - Raw `timedelta` or exact numeric seconds value.
///
/// # Returns
/// Positive whole seconds as `i32`.
///
/// # Errors
/// Returns `ValidationError` for unsupported types, fractions, non-finite values, or range overflow.
pub(super) fn whole_seconds(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<i32> {
    let timedelta = exact_type(py, PublicType::Timedelta)?;
    let seconds = if value.is_instance(&timedelta)? {
        value.call_method0("total_seconds")?
    } else if value.get_type().is(py.get_type::<PyInt>())
        || value.get_type().is(py.get_type::<PyFloat>())
    {
        value.clone()
    } else {
        return Err(validation_error(
            "duration must be timedelta or numeric seconds",
        ));
    };
    let number = seconds.extract::<f64>().map_err(|_| {
        validation_error("duration must be positive whole seconds within the native integer range")
    })?;
    if !number.is_finite() || number <= 0.0 || number > f64::from(i32::MAX) || number.fract() != 0.0
    {
        return Err(validation_error(
            "duration must be positive whole seconds within the native integer range",
        ));
    }
    Ok(number as i32)
}

/// Builds the immutable native binding record from the fully initialized public package.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
///
/// # Returns
/// Trusted public classes and private Live View constructor tokens.
///
/// # Errors
/// Returns an import, attribute, or type error when a required package binding is unavailable.
fn load_public_types(py: Python<'_>) -> PyResult<PublicTypes> {
    let class = |module: &Bound<'_, PyModule>, name: &str| -> PyResult<Py<PyAny>> {
        let value = module.getattr(name)?;
        value.cast::<PyType>()?;
        Ok(value.unbind())
    };
    let builtins = py.import("builtins")?;
    let datetime = py.import("datetime")?;
    let exceptions = py.import("swmmrs.exceptions")?;
    let enums = py.import("swmmrs.enums")?;
    let objects = py.import("swmmrs.objects")?;
    let base = py.import("swmmrs.objects._base")?;

    Ok(PublicTypes {
        exact: ExactTypes {
            float: class(&builtins, "float")?,
            int: class(&builtins, "int")?,
            str_: class(&builtins, "str")?,
            tuple: class(&builtins, "tuple")?,
            datetime: class(&datetime, "datetime")?,
            timedelta: class(&datetime, "timedelta")?,
            custom_ellipse_model: class(&enums, "CustomEllipseModel")?,
            inertia_damping: class(&enums, "InertiaDamping")?,
            normal_flow_limit: class(&enums, "NormalFlowLimit")?,
            orifice_kind: class(&enums, "OrificeKind")?,
            outlet_head_basis: class(&enums, "OutletHeadBasis")?,
            road_surface: class(&enums, "RoadSurface")?,
            standard_cross_section_shape: class(&enums, "StandardCrossSectionShape")?,
            surcharge_method: class(&enums, "SurchargeMethod")?,
            weir_kind: class(&enums, "WeirKind")?,
            circular_cross_section: class(&objects, "CircularCrossSection")?,
            custom_cross_section: class(&objects, "CustomCrossSection")?,
            divider_rule: class(&objects, "DividerRule")?,
            functional_outlet_rating: class(&objects, "FunctionalOutletRating")?,
            irregular_cross_section: class(&objects, "IrregularCrossSection")?,
            outfall_boundary: class(&objects, "OutfallBoundary")?,
            standard_cross_section: class(&objects, "StandardCrossSection")?,
            storage_exfiltration: class(&objects, "StorageExfiltration")?,
            storage_shape: class(&objects, "StorageShape")?,
            street_cross_section: class(&objects, "StreetCrossSection")?,
            tabular_outlet_rating: class(&objects, "TabularOutletRating")?,
        },
        exceptions: ExceptionTypes {
            configuration_diagnostic: class(&exceptions, "ConfigurationDiagnostic")?,
            configuration_error: class(&exceptions, "ConfigurationError")?,
            configuration_object_identity: class(&exceptions, "ConfigurationObjectIdentity")?,
            internal_simulation_error: class(&exceptions, "InternalSimulationError")?,
            lifecycle_error: class(&exceptions, "LifecycleError")?,
            solver_error: class(&exceptions, "SolverError")?,
            stale_view_error: class(&exceptions, "StaleViewError")?,
            validation_error: class(&exceptions, "ValidationError")?,
        },
        solver_error_code: class(&enums, "SolverErrorCode")?,
        views: ViewTypes {
            rain_gage: class(&objects, "RainGage")?,
            subcatchment: class(&objects, "Subcatchment")?,
            pollutant: class(&objects, "Pollutant")?,
            land_use: class(&objects, "LandUse")?,
            time_pattern: class(&objects, "TimePattern")?,
            curve: class(&objects, "Curve")?,
            time_series: class(&objects, "TimeSeries")?,
            control_rule: class(&objects, "ControlRule")?,
            transect: class(&objects, "Transect")?,
            aquifer: class(&objects, "Aquifer")?,
            amm_model: class(&objects, "AmmModel")?,
            unit_hydrograph: class(&objects, "UnitHydrograph")?,
            snowmelt_parameter_set: class(&objects, "SnowmeltParameterSet")?,
            custom_shape: class(&objects, "CustomShape")?,
            lid_control: class(&objects, "LidControl")?,
            street: class(&objects, "Street")?,
            inlet_design: class(&objects, "InletDesign")?,
            junction: class(&objects, "Junction")?,
            outfall: class(&objects, "Outfall")?,
            storage: class(&objects, "StorageNode")?,
            divider: class(&objects, "Divider")?,
            conduit: class(&objects, "Conduit")?,
            pump: class(&objects, "Pump")?,
            orifice: class(&objects, "Orifice")?,
            weir: class(&objects, "Weir")?,
            outlet: class(&objects, "Outlet")?,
            inlet: class(&objects, "Inlet")?,
        },
        view_token: base.getattr("_VIEW_TOKEN")?.unbind(),
        inlet_token: base.getattr("_INLET_TOKEN")?.unbind(),
    })
}

/// Binds trusted public package types once after ordinary package initialization.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
///
/// # Errors
/// Returns an import, attribute, or type error when a required package binding is unavailable.
#[pyfunction(name = "_bind_public_types")]
pub(crate) fn bind_public_types(py: Python<'_>) -> PyResult<()> {
    if PUBLIC_TYPES.get(py).is_some() {
        return Ok(());
    }
    let bindings = load_public_types(py)?;
    PUBLIC_TYPES
        .set(py, bindings)
        .map_err(|_| PyRuntimeError::new_err("public package types were already initialized"))
}

/// Returns the immutable public binding record.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
///
/// # Returns
/// Trusted package bindings captured during package import.
///
/// # Errors
/// Returns a runtime error only when the extension is used outside ordinary package initialization.
fn public_types(py: Python<'_>) -> PyResult<&PublicTypes> {
    PUBLIC_TYPES.get(py).ok_or_else(|| {
        PyRuntimeError::new_err("native public types are unavailable before package initialization")
    })
}

/// Returns one trusted exact-value class from the immutable binding record.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `type_` - Closed exact-value class identity.
///
/// # Returns
/// Bound trusted class object.
///
/// # Errors
/// Returns a runtime error when package bindings are unavailable.
fn exact_type<'py>(py: Python<'py>, type_: PublicType) -> PyResult<Bound<'py, PyAny>> {
    let exact = &public_types(py)?.exact;
    let class = match type_ {
        PublicType::Float => &exact.float,
        PublicType::Int => &exact.int,
        PublicType::Str => &exact.str_,
        PublicType::Tuple => &exact.tuple,
        PublicType::DateTime => &exact.datetime,
        PublicType::Timedelta => &exact.timedelta,
        PublicType::CustomEllipseModel => &exact.custom_ellipse_model,
        PublicType::InertiaDamping => &exact.inertia_damping,
        PublicType::NormalFlowLimit => &exact.normal_flow_limit,
        PublicType::OrificeKind => &exact.orifice_kind,
        PublicType::OutletHeadBasis => &exact.outlet_head_basis,
        PublicType::RoadSurface => &exact.road_surface,
        PublicType::StandardCrossSectionShape => &exact.standard_cross_section_shape,
        PublicType::SurchargeMethod => &exact.surcharge_method,
        PublicType::WeirKind => &exact.weir_kind,
        PublicType::CircularCrossSection => &exact.circular_cross_section,
        PublicType::CustomCrossSection => &exact.custom_cross_section,
        PublicType::DividerRule => &exact.divider_rule,
        PublicType::FunctionalOutletRating => &exact.functional_outlet_rating,
        PublicType::IrregularCrossSection => &exact.irregular_cross_section,
        PublicType::OutfallBoundary => &exact.outfall_boundary,
        PublicType::StandardCrossSection => &exact.standard_cross_section,
        PublicType::StorageExfiltration => &exact.storage_exfiltration,
        PublicType::StorageShape => &exact.storage_shape,
        PublicType::StreetCrossSection => &exact.street_cross_section,
        PublicType::TabularOutletRating => &exact.tabular_outlet_rating,
    };
    Ok(class.bind(py).clone())
}

/// Identifies one trusted public exception class.
#[derive(Clone, Copy)]
enum ExceptionType {
    Configuration,
    InternalSimulation,
    Lifecycle,
    Solver,
    StaleView,
    Validation,
}

/// Returns one trusted exception class from the immutable binding record.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `type_` - Closed exception class identity.
///
/// # Returns
/// Bound trusted exception class.
///
/// # Errors
/// Returns a runtime error when package bindings are unavailable.
fn exception_type<'py>(py: Python<'py>, type_: ExceptionType) -> PyResult<Bound<'py, PyAny>> {
    let exceptions = &public_types(py)?.exceptions;
    let class = match type_ {
        ExceptionType::Configuration => &exceptions.configuration_error,
        ExceptionType::InternalSimulation => &exceptions.internal_simulation_error,
        ExceptionType::Lifecycle => &exceptions.lifecycle_error,
        ExceptionType::Solver => &exceptions.solver_error,
        ExceptionType::StaleView => &exceptions.stale_view_error,
        ExceptionType::Validation => &exceptions.validation_error,
    };
    Ok(class.bind(py).clone())
}

/// Classifies one solver failure without constructing Python objects.
enum PublicFailure {
    Lifecycle(String),
    Key(Option<String>),
    Index(Option<String>),
    Validation(String),
    Solver(String, Option<String>, String, Option<String>),
    Internal(String),
}

/// Instantiates one trusted public exception class after owner-lock release.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `type_` - Closed public exception class identity.
/// * `args` - Positional exception constructor arguments.
///
/// # Returns
/// Final public Python exception.
///
/// # Errors
/// Returns a Python error when the trusted class is unavailable or construction fails.
fn public_exception(
    py: Python<'_>,
    type_: ExceptionType,
    args: &Bound<'_, PyTuple>,
) -> PyResult<PyErr> {
    let class = exception_type(py, type_)?;
    let instance = class.call1(args)?;
    Ok(PyErr::from_value(instance))
}

/// Constructs one message-only final public exception.
///
/// # Arguments
/// * `type_` - Closed public exception class identity.
/// * `message` - Complete public diagnostic message.
///
/// # Returns
/// Final public exception, or the constructor failure if package invariants are broken.
fn message_exception(type_: ExceptionType, message: String) -> PyErr {
    Python::attach(|py| {
        let args = PyTuple::new(py, [message])?;
        public_exception(py, type_, &args)
    })
    .unwrap_or_else(|error| error)
}

/// Constructs a public exception carrying native failure metadata.
fn structured_message_exception(
    type_: ExceptionType,
    message: String,
    native_code: i32,
    operation: String,
    detail: Option<String>,
    semantic_code: Option<String>,
) -> PyErr {
    Python::attach(|py| {
        let args = PyTuple::new(
            py,
            [
                message.into_pyobject(py)?.into_any(),
                native_code.into_pyobject(py)?.into_any(),
                operation.into_pyobject(py)?.into_any(),
                detail.into_pyobject(py)?.into_any(),
                semantic_code.into_pyobject(py)?.into_any(),
            ],
        )?;
        public_exception(py, type_, &args)
    })
    .unwrap_or_else(|error| error)
}

/// Maps one solver error directly to its final public exception.
///
/// # Arguments
/// * `operation` - Public operation that triggered the native failure.
/// * `error` - Owned solver failure captured after owner-lock release.
///
/// # Returns
/// Final public exception for the classified failure category.
pub(super) fn native_error(operation: &str, error: SwmmError) -> PyErr {
    let message = |detail: Option<&str>| {
        format!(
            "{operation}: {}",
            detail.unwrap_or("native operation failed")
        )
    };
    let native_code = error.code.as_i32();
    let semantic_code = error.semantic_code().map(str::to_owned);
    let fallback_semantic_code = error.code.semantic_code().map(str::to_owned);
    let metadata_detail = error.detail.clone();
    let metadata_operation = operation.to_string();
    let failure = match error.code {
        ErrorCode::ApiNotOpen
        | ErrorCode::ApiNotStarted
        | ErrorCode::ApiNotEnded
        | ErrorCode::TkapiInputnotopen
        | ErrorCode::TkapiSimNrunning
        | ErrorCode::TkapiSimRunning => PublicFailure::Lifecycle(message(error.detail.as_deref())),
        ErrorCode::ApiObjectName => PublicFailure::Key(error.detail),
        ErrorCode::ApiObjectIndex
        | ErrorCode::TkapiOutbounds
        | ErrorCode::TkapiObjectIndex
        | ErrorCode::TkapiPollutIndex
        | ErrorCode::TkapiTseriesIndex
        | ErrorCode::TkapiPatternIndex
        | ErrorCode::TkapiLidunitIndex => PublicFailure::Index(error.detail),
        ErrorCode::ApiObjectType
        | ErrorCode::ApiPropertyType
        | ErrorCode::ApiPropertyValue
        | ErrorCode::ApiTimePeriod
        | ErrorCode::TkapiWrongType
        | ErrorCode::TkapiInflowtype
        | ErrorCode::TkapiUndefinedLid
        | ErrorCode::TkapiNoInlet => PublicFailure::Validation(message(error.detail.as_deref())),
        _ => match semantic_code.as_deref() {
            Some(code) => PublicFailure::Solver(
                if code == "hotstart_file_write" {
                    "out_write"
                } else {
                    code
                }
                .to_string(),
                fallback_semantic_code,
                operation.to_string(),
                error.detail,
            ),
            None => PublicFailure::Internal(message(error.detail.as_deref())),
        },
    };
    match failure {
        PublicFailure::Lifecycle(message) => structured_message_exception(
            ExceptionType::Lifecycle,
            message,
            native_code,
            metadata_operation,
            metadata_detail,
            semantic_code,
        ),
        PublicFailure::Key(value) => PyKeyError::new_err(value),
        PublicFailure::Index(value) => PyIndexError::new_err(value),
        PublicFailure::Validation(message) => structured_message_exception(
            ExceptionType::Validation,
            message,
            native_code,
            metadata_operation,
            metadata_detail,
            semantic_code,
        ),
        PublicFailure::Internal(message) => structured_message_exception(
            ExceptionType::InternalSimulation,
            message,
            native_code,
            metadata_operation,
            metadata_detail,
            semantic_code,
        ),
        PublicFailure::Solver(code, fallback_code, operation, detail) => Python::attach(|py| {
            let enum_class = public_types(py)?.solver_error_code.bind(py);
            let code = enum_class.call1((&code,)).or_else(|error| {
                fallback_code
                    .as_deref()
                    .map(|fallback| enum_class.call1((fallback,)))
                    .unwrap_or(Err(error))
            })?;
            let args = PyTuple::new(
                py,
                [
                    code,
                    operation.into_pyobject(py)?.into_any(),
                    detail.into_pyobject(py)?.into_any(),
                    native_code.into_pyobject(py)?.into_any(),
                    semantic_code.into_pyobject(py)?.into_any(),
                ],
            )?;
            public_exception(py, ExceptionType::Solver, &args)
        })
        .unwrap_or_else(|error| error),
    }
}

/// Maps ordered post-open configuration diagnostics directly to `ConfigurationError`.
///
/// # Arguments
/// * `diagnostics` - Ordered owned configuration diagnostics captured after owner-lock release.
///
/// # Returns
/// Final public configuration exception.
pub(super) fn configuration_error(diagnostics: Vec<ConfigurationDiagnostic>) -> PyErr {
    Python::attach(|py| -> PyResult<PyErr> {
        let bindings = public_types(py)?;
        let identity_class = bindings.exceptions.configuration_object_identity.bind(py);
        let diagnostic_class = bindings.exceptions.configuration_diagnostic.bind(py);
        let mut values = Vec::with_capacity(diagnostics.len());
        for diagnostic in diagnostics {
            let object = identity_class.call1((
                diagnostic.object.object_type,
                diagnostic.object.id,
                diagnostic.object.index,
            ))?;
            let conflicting = if let Some(conflicting) = diagnostic.conflicting_object {
                identity_class
                    .call1((conflicting.object_type, conflicting.id, conflicting.index))?
                    .into_any()
            } else {
                py.None().into_bound(py)
            };
            values.push(diagnostic_class.call1((
                object,
                diagnostic.property_path,
                diagnostic.rule_code,
                diagnostic.message,
                conflicting,
            ))?);
        }
        let diagnostics = PyTuple::new(py, values)?;
        let args = PyTuple::new(py, [diagnostics])?;
        public_exception(py, ExceptionType::Configuration, &args)
    })
    .unwrap_or_else(|error| error)
}

/// Creates one final public lifecycle failure.
///
/// # Arguments
/// * `operation` - Lifecycle operation that failed.
/// * `detail` - Stable failure detail.
///
/// # Returns
/// Final public lifecycle exception.
pub(super) fn lifecycle_error(operation: &str, detail: &str) -> PyErr {
    message_exception(ExceptionType::Lifecycle, format!("{operation}: {detail}"))
}

/// Creates one final public validation failure.
///
/// # Arguments
/// * `message` - Complete public validation diagnostic.
///
/// # Returns
/// Final public validation exception.
pub(super) fn validation_error(message: &str) -> PyErr {
    message_exception(ExceptionType::Validation, message.to_string())
}

/// Converts one private family ordinal to the solver object type.
///
/// # Arguments
/// * `family` - Private family ordinal supplied by the Python facade.
///
/// # Returns
/// Typed solver object family.
///
/// # Errors
/// Returns an internal native failure for an invalid private ordinal.
pub(super) fn object_type(family: u8) -> PyResult<ObjectType> {
    ObjectType::try_from(usize::from(family))
        .map_err(|_| internal_error("object_family", "invalid private object family"))
}

/// Returns the stable private subtype label used to select public Live Views.
///
/// # Arguments
/// * `subtype` - Typed solver object subtype.
///
/// # Returns
/// Stable private subtype label, or an empty label for ordinary objects.
pub(super) fn subtype_name(subtype: ObjectSubtypeRead) -> &'static str {
    match subtype {
        ObjectSubtypeRead::Ordinary => "",
        ObjectSubtypeRead::Node(NodeType::Junction) => "junction",
        ObjectSubtypeRead::Node(NodeType::Outfall) => "outfall",
        ObjectSubtypeRead::Node(NodeType::Storage) => "storage",
        ObjectSubtypeRead::Node(NodeType::Divider) => "divider",
        ObjectSubtypeRead::Link(LinkType::Conduit) => "conduit",
        ObjectSubtypeRead::Link(LinkType::Pump) => "pump",
        ObjectSubtypeRead::Link(LinkType::Orifice) => "orifice",
        ObjectSubtypeRead::Link(LinkType::Weir) => "weir",
        ObjectSubtypeRead::Link(LinkType::Outlet) => "outlet",
    }
}

/// Converts one private subtype label back to its typed solver identity.
///
/// # Arguments
/// * `object_type` - Typed configured object family.
/// * `name` - Private subtype label captured by a Python Live View.
///
/// # Returns
/// Typed subtype when the family and label form a valid pair.
///
/// # Errors
/// Returns `None` when the object family and subtype label are not a valid pair.
pub(super) fn object_subtype(object_type: ObjectType, name: &str) -> Option<ObjectSubtypeRead> {
    match (object_type, name) {
        (ObjectType::Node, "junction") => Some(ObjectSubtypeRead::Node(NodeType::Junction)),
        (ObjectType::Node, "outfall") => Some(ObjectSubtypeRead::Node(NodeType::Outfall)),
        (ObjectType::Node, "storage") => Some(ObjectSubtypeRead::Node(NodeType::Storage)),
        (ObjectType::Node, "divider") => Some(ObjectSubtypeRead::Node(NodeType::Divider)),
        (ObjectType::Link, "conduit") => Some(ObjectSubtypeRead::Link(LinkType::Conduit)),
        (ObjectType::Link, "pump") => Some(ObjectSubtypeRead::Link(LinkType::Pump)),
        (ObjectType::Link, "orifice") => Some(ObjectSubtypeRead::Link(LinkType::Orifice)),
        (ObjectType::Link, "weir") => Some(ObjectSubtypeRead::Link(LinkType::Weir)),
        (ObjectType::Link, "outlet") => Some(ObjectSubtypeRead::Link(LinkType::Outlet)),
        (object_type, "") if !matches!(object_type, ObjectType::Node | ObjectType::Link) => {
            Some(ObjectSubtypeRead::Ordinary)
        }
        _ => None,
    }
}

/// Creates one final public generation-mismatch failure.
///
/// # Arguments
/// * `operation` - Live View operation that observed the stale generation.
///
/// # Returns
/// Final public stale-view exception.
pub(super) fn stale_error(operation: &str) -> PyErr {
    message_exception(
        ExceptionType::StaleView,
        format!("{operation}: project generation is no longer active"),
    )
}

/// Creates one final public internal-failure exception.
///
/// # Arguments
/// * `operation` - Native operation that detected the invariant failure.
/// * `detail` - Stable internal diagnostic detail.
///
/// # Returns
/// Final public internal-simulation exception.
pub(super) fn internal_error(operation: &str, detail: &str) -> PyErr {
    message_exception(
        ExceptionType::InternalSimulation,
        format!("{operation}: {detail}"),
    )
}

/// Extracts one collection key with the public mapping's exact type semantics.
///
/// # Arguments
/// * `value` - Candidate string (including subclasses) or exact non-boolean integer.
///
/// # Returns
/// Normalized string key, or `None` for a type rejected by membership checks.
///
/// # Errors
/// Propagates string extraction or integer stringification failures.
pub(super) fn collection_key(value: &Bound<'_, PyAny>) -> PyResult<Option<String>> {
    let py = value.py();
    if value.is_instance_of::<PyString>() {
        return value.extract::<String>().map(Some);
    }
    if value.get_type().is(py.get_type::<PyInt>()) {
        return value
            .str()
            .and_then(|text| Ok(Some(text.to_str()?.to_owned())));
    }
    Ok(None)
}

/// Normalizes one optional ordered snapshot selection at the native boundary.
///
/// # Arguments
/// * `value` - Python `None`, one string/non-boolean integer, or an ordered iterable thereof.
/// * `operation` - Native acquisition name used in validation diagnostics.
///
/// # Returns
/// `None` for a full-family acquisition or normalized string IDs in caller order.
///
/// # Errors
/// Returns a validation error for mappings, sets, bytes, non-iterables, booleans, or malformed
/// iterable members, and propagates exceptions raised while iterating caller input.
pub(super) fn snapshot_ids(
    value: &Bound<'_, PyAny>,
    operation: &str,
) -> PyResult<Option<Vec<String>>> {
    if value.is_none() {
        return Ok(None);
    }
    if let Some(identifier) = snapshot_id(value) {
        return Ok(Some(vec![identifier?]));
    }

    let py = value.py();
    let builtins = py.import("builtins")?;
    let collections = py.import("collections.abc")?;
    if value.is_instance(&builtins.getattr("bytes")?)?
        || value.is_instance(&collections.getattr("Mapping")?)?
        || value.is_instance(&collections.getattr("Set")?)?
    {
        return Err(selection_error(
            operation,
            "snapshot IDs must be an ordered iterable",
        ));
    }

    let iterator = value.try_iter().map_err(|_| {
        selection_error(
            operation,
            "snapshot IDs must be a string, non-boolean integer, or ordered iterable",
        )
    })?;
    let mut normalized = Vec::new();
    for item in iterator {
        let item = item?;
        let identifier = snapshot_id(&item).ok_or_else(|| {
            selection_error(
                operation,
                "snapshot IDs must contain only strings or non-boolean integers",
            )
        })??;
        normalized.push(identifier);
    }
    Ok(Some(normalized))
}

/// Extracts one accepted snapshot identifier without accepting integer subclasses.
///
/// # Arguments
/// * `value` - Candidate Python snapshot identifier.
///
/// # Returns
/// `Some` containing a normalized string identifier or conversion result for accepted inputs;
/// otherwise `None`.
///
/// # Errors
/// The returned `PyResult` contains a Python string-conversion error when conversion fails.
fn snapshot_id(value: &Bound<'_, PyAny>) -> Option<PyResult<String>> {
    if value.is_instance_of::<PyString>() {
        return Some(value.extract::<String>());
    }
    if value.get_type().is(value.py().get_type::<PyInt>()) {
        return Some(value.str().map(|text| text.to_string_lossy().into_owned()));
    }
    None
}

/// Creates one native selection-validation failure.
///
/// # Arguments
/// * `operation` - Native acquisition name used in the validation diagnostic.
/// * `detail` - Stable selection-validation detail.
///
/// # Returns
/// Private native validation exception consumed by the ordinary Python facade.
fn selection_error(operation: &str, detail: &str) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
    )
}

/// Decodes one SWMM date into Python datetime constructor components.
///
/// # Arguments
/// * `value` - SWMM model date and time.
///
/// # Returns
/// Whole-second `(year, month, day, hour, minute, second)` components.
pub(super) fn datetime_parts(value: DateTime) -> (i32, u8, u8, u8, u8, u8) {
    let (mut year, mut month, mut day) = (0, 0, 0);
    datetime_decodeDate(value, &mut year, &mut month, &mut day);
    let (mut hour, mut minute, mut second) = (0, 0, 0);
    datetime_decodeTime(value, &mut hour, &mut minute, &mut second);
    (
        year,
        month as u8,
        day as u8,
        hour as u8,
        minute as u8,
        second as u8,
    )
}

/// Returns the stable private flow-unit label.
///
/// # Arguments
/// * `value` - Native flow-unit enum.
///
/// # Returns
/// Stable private lowercase flow-unit value.
pub(super) fn flow_units_name(value: FlowUnitsType) -> &'static str {
    match value {
        FlowUnitsType::Cfs => "cfs",
        FlowUnitsType::Gpm => "gpm",
        FlowUnitsType::Mgd => "mgd",
        FlowUnitsType::Cms => "cms",
        FlowUnitsType::Lps => "lps",
        FlowUnitsType::Mld => "mld",
    }
}

/// Returns the stable private unit-system label.
///
/// # Arguments
/// * `value` - Native unit-system enum.
///
/// # Returns
/// Stable private lowercase unit-system value.
pub(super) fn unit_system_name(value: UnitsType) -> &'static str {
    match value {
        UnitsType::Us => "us",
        UnitsType::Si => "si",
    }
}

/// Checks whether a value has one exact public Python class identity.
///
/// # Arguments
/// * `value` - Python value whose concrete class is inspected.
/// * `type_` - Closed authoritative public class identity.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// `true` only when the value's exact concrete class is the exported class.
///
/// # Errors
/// Returns an internal error when the trusted package class was not bound during import.
pub(super) fn is_exact_public_python_type(
    value: &Bound<'_, PyAny>,
    type_: PublicType,
    operation: &str,
) -> PyResult<bool> {
    let expected = exact_type(value.py(), type_)
        .map_err(|_| internal_error(operation, "trusted public Python type is unavailable"))?;
    Ok(value.get_type().is(&expected))
}

/// Constructs one fresh final public Live View for a native relationship result.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade.
/// * `generation` - Captured Project Generation.
/// * `object_type` - Related configured-object family.
/// * `identity` - Canonical related-object identity read under the owner lock.
///
/// # Returns
/// Fresh generation-bound public Live View.
///
/// # Errors
/// Returns an internal failure if the family/subtype has no public presentation class or public
/// relationship construction machinery is unavailable.
pub(super) fn relationship_view(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    object_type: ObjectType,
    identity: ObjectIdentityRead,
) -> PyResult<Py<PyAny>> {
    let bindings = public_types(py)?;
    let subtype = subtype_name(identity.subtype);
    let class = match (object_type, identity.subtype) {
        (ObjectType::Gage, ObjectSubtypeRead::Ordinary) => &bindings.views.rain_gage,
        (ObjectType::Subcatch, ObjectSubtypeRead::Ordinary) => &bindings.views.subcatchment,
        (ObjectType::Pollut, ObjectSubtypeRead::Ordinary) => &bindings.views.pollutant,
        (ObjectType::Landuse, ObjectSubtypeRead::Ordinary) => &bindings.views.land_use,
        (ObjectType::Timepattern, ObjectSubtypeRead::Ordinary) => &bindings.views.time_pattern,
        (ObjectType::Curve, ObjectSubtypeRead::Ordinary) => &bindings.views.curve,
        (ObjectType::Tseries, ObjectSubtypeRead::Ordinary) => &bindings.views.time_series,
        (ObjectType::Control, ObjectSubtypeRead::Ordinary) => &bindings.views.control_rule,
        (ObjectType::Transect, ObjectSubtypeRead::Ordinary) => &bindings.views.transect,
        (ObjectType::Aquifer, ObjectSubtypeRead::Ordinary) => &bindings.views.aquifer,
        (ObjectType::AmmModel, ObjectSubtypeRead::Ordinary) => &bindings.views.amm_model,
        (ObjectType::Unithyd, ObjectSubtypeRead::Ordinary) => &bindings.views.unit_hydrograph,
        (ObjectType::Snowmelt, ObjectSubtypeRead::Ordinary) => {
            &bindings.views.snowmelt_parameter_set
        }
        (ObjectType::Shape, ObjectSubtypeRead::Ordinary) => &bindings.views.custom_shape,
        (ObjectType::Lid, ObjectSubtypeRead::Ordinary) => &bindings.views.lid_control,
        (ObjectType::Street, ObjectSubtypeRead::Ordinary) => &bindings.views.street,
        (ObjectType::Inlet, ObjectSubtypeRead::Ordinary) => &bindings.views.inlet_design,
        (ObjectType::Node, ObjectSubtypeRead::Node(NodeType::Junction)) => &bindings.views.junction,
        (ObjectType::Node, ObjectSubtypeRead::Node(NodeType::Outfall)) => &bindings.views.outfall,
        (ObjectType::Node, ObjectSubtypeRead::Node(NodeType::Storage)) => &bindings.views.storage,
        (ObjectType::Node, ObjectSubtypeRead::Node(NodeType::Divider)) => &bindings.views.divider,
        (ObjectType::Link, ObjectSubtypeRead::Link(LinkType::Conduit)) => &bindings.views.conduit,
        (ObjectType::Link, ObjectSubtypeRead::Link(LinkType::Pump)) => &bindings.views.pump,
        (ObjectType::Link, ObjectSubtypeRead::Link(LinkType::Orifice)) => &bindings.views.orifice,
        (ObjectType::Link, ObjectSubtypeRead::Link(LinkType::Weir)) => &bindings.views.weir,
        (ObjectType::Link, ObjectSubtypeRead::Link(LinkType::Outlet)) => &bindings.views.outlet,
        _ => {
            return Err(internal_error(
                "relationship_view",
                "unsupported related object family or subtype",
            ));
        }
    };
    class
        .bind(py)
        .call1((
            facade,
            generation,
            object_type as i32,
            identity.index,
            identity.id,
            subtype,
            bindings.view_token.bind(py),
        ))
        .map(Bound::unbind)
}

/// Constructs one fresh link-local inlet placement view.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `facade` - Owning public `Simulation` facade.
/// * `generation` - Captured Project Generation.
/// * `link_index` - Captured configured link index.
/// * `link_id` - Captured canonical link identifier.
/// * `link_subtype` - Captured concrete link subtype label.
/// * `design` - Canonical related inlet-design identity.
///
/// # Returns
/// Fresh generation-bound public inlet placement view.
///
/// # Errors
/// Returns an internal or constructor failure when trusted inlet bindings are unavailable.
pub(super) fn inlet_view(
    py: Python<'_>,
    facade: &Bound<'_, PyAny>,
    generation: u64,
    link_index: usize,
    link_id: &str,
    link_subtype: &str,
    design: ObjectIdentityRead,
) -> PyResult<Py<PyAny>> {
    let bindings = public_types(py)?;
    bindings
        .views
        .inlet
        .bind(py)
        .call1((
            facade,
            generation,
            link_index,
            link_id,
            link_subtype,
            design.index,
            design.id,
            bindings.inlet_token.bind(py),
        ))
        .map(Bound::unbind)
}

/// Owns one related Live View identity for owner-locked validation.
#[derive(Clone)]
pub(super) struct RelatedObject {
    pub(super) object_type: ObjectType,
    pub(super) index: usize,
    pub(super) id: String,
    pub(super) subtype: String,
}

/// Extracts one same-owner related Live View without deciding its required family.
///
/// # Arguments
/// * `py` - Attached Python interpreter token.
/// * `owner` - Target Simulation Owner.
/// * `generation` - Target Project Generation.
/// * `value` - Related Live View or optional `None`.
/// * `name` - Relationship property name used in diagnostics.
/// * `operation` - Stable native operation name used in diagnostics.
/// * `allow_none` - Whether `None` clears the relationship.
///
/// # Returns
/// Owned relationship identity, or `None` for an allowed clear request.
///
/// # Errors
/// Returns a validation error for malformed, disallowed-null, foreign-owner, or foreign-generation values.
pub(super) fn related_live_view(
    py: Python<'_>,
    owner: &NativeSimulation,
    generation: u64,
    value: &Bound<'_, PyAny>,
    name: &str,
    operation: &str,
    allow_none: bool,
) -> PyResult<Option<RelatedObject>> {
    let invalid = |detail: String| {
        native_error(
            operation,
            SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail),
        )
    };
    if value.is_none() {
        return if allow_none {
            Ok(None)
        } else {
            Err(invalid(format!("{name} must be a Live View")))
        };
    }
    let malformed = || invalid(format!("{name} must be a Live View"));
    let related_simulation = value.getattr("_simulation").map_err(|_| malformed())?;
    let related_owner = related_simulation
        .getattr("_owner")
        .map_err(|_| malformed())?
        .extract::<Py<NativeSimulation>>()
        .map_err(|_| malformed())?;
    let related_generation = value
        .getattr("_generation")
        .map_err(|_| malformed())?
        .extract::<u64>()
        .map_err(|_| malformed())?;
    if !std::ptr::eq(owner, &*related_owner.borrow(py)) || related_generation != generation {
        return Err(invalid(format!(
            "{name} must belong to the same Simulation generation"
        )));
    }
    Ok(Some(RelatedObject {
        object_type: object_type(
            value
                .getattr("_family")
                .map_err(|_| malformed())?
                .extract::<u8>()
                .map_err(|_| malformed())?,
        )?,
        index: value
            .getattr("_index")
            .map_err(|_| malformed())?
            .extract::<usize>()
            .map_err(|_| malformed())?,
        id: value
            .getattr("_id")
            .map_err(|_| malformed())?
            .extract::<String>()
            .map_err(|_| malformed())?,
        subtype: value
            .getattr("_subtype")
            .map_err(|_| malformed())?
            .extract::<String>()
            .map_err(|_| malformed())?,
    }))
}

/// Validates one related Live View against authoritative configured identity.
///
/// # Arguments
/// * `simulation` - Locked authoritative Simulation Owner.
/// * `relation` - Extracted related Live View identity.
/// * `expected` - Allowed object families.
/// * `name` - Public relationship path.
///
/// # Returns
/// Validated configured related-object index.
///
/// # Errors
/// Returns a family, subtype, stale-identity, or configured-storage error.
pub(super) fn validate_related_object(
    simulation: &SwmmSimulation,
    relation: &RelatedObject,
    expected: &[ObjectType],
    name: &str,
) -> Result<usize, SwmmError> {
    if !expected.contains(&relation.object_type) {
        return Err(SwmmError::with_detail(
            ErrorCode::ApiPropertyValue,
            format!("{name} has the wrong object family"),
        ));
    }
    let Some(subtype) = object_subtype(relation.object_type, &relation.subtype) else {
        return Err(SwmmError::with_detail(
            ErrorCode::ApiPropertyValue,
            format!("{name} has an invalid subtype"),
        ));
    };
    if !simulation.object_identity_matches(
        relation.object_type,
        relation.index,
        &relation.id,
        subtype,
    )? {
        return Err(SwmmError::with_detail(
            ErrorCode::ApiPropertyValue,
            format!("{name} identity is stale"),
        ));
    }
    Ok(relation.index)
}

/// Extracts one exact string or non-boolean integer pollutant identity.
///
/// # Arguments
/// * `value` - Python pollutant identity.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Owned pollutant ID spelling; exact integers use their decimal spelling.
///
/// # Errors
/// Returns `ValidationError` when `value` is not an exact `str` or `int`.
pub(super) fn extract_pollutant_id(value: &Bound<'_, PyAny>, operation: &str) -> PyResult<String> {
    let py = value.py();
    if value.get_type().is(py.get_type::<PyString>()) {
        return value.extract::<String>();
    }
    if value.get_type().is(py.get_type::<PyInt>()) {
        return value
            .str()
            .and_then(|text| Ok(text.to_str()?.to_owned()))
            .map_err(|_| {
                pollutant_value_error(operation, "pollutant ID is outside the supported range")
            });
    }
    Err(pollutant_value_error(
        operation,
        "pollutant IDs must be exact str or non-boolean int values",
    ))
}

/// Extracts one exact non-boolean Python numeric value.
///
/// # Arguments
/// * `value` - Python value to extract.
/// * `name` - Public value name used in diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Extracted floating-point value; domain validation remains in the solver.
///
/// # Errors
/// Returns `ValidationError` for booleans, non-numeric values, or unsupported ranges.
pub(super) fn extract_strict_number(
    value: &Bound<'_, PyAny>,
    name: &str,
    operation: &str,
) -> PyResult<f64> {
    let py = value.py();
    if !value.get_type().is(py.get_type::<PyInt>())
        && !value.get_type().is(py.get_type::<PyFloat>())
    {
        return Err(pollutant_value_error(
            operation,
            format!("{name} values must be exact int or float values, not bool"),
        ));
    }
    value.extract::<f64>().map_err(|_| {
        pollutant_value_error(
            operation,
            format!("{name} values are outside the supported range"),
        )
    })
}

/// Extracts ordered pollutant-value pairs from one mapping or pair iterable.
///
/// # Arguments
/// * `source` - Python mapping or iterable of two-value entries.
/// * `name` - Public value name used in diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Owned pollutant identity spellings and numeric values in caller order.
///
/// # Errors
/// Returns `ValidationError` for malformed sources, keys, entries, or numeric types.
pub(super) fn extract_pollutant_values(
    source: &Bound<'_, PyAny>,
    name: &str,
    operation: &str,
) -> PyResult<Vec<(String, f64)>> {
    let items = if source.hasattr("items")? {
        source.call_method0("items").map_err(|_| {
            pollutant_value_error(
                operation,
                format!("{name} must be a mapping or pair iterable"),
            )
        })?
    } else {
        source.clone()
    };
    let entries = items.try_iter().map_err(|_| {
        pollutant_value_error(
            operation,
            format!("{name} must be a mapping or pair iterable"),
        )
    })?;
    let mut values = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| {
            pollutant_value_error(operation, format!("{name} contains an invalid entry"))
        })?;
        if entry.len().map_err(|_| {
            pollutant_value_error(operation, format!("{name} entries must contain two values"))
        })? != 2
        {
            return Err(pollutant_value_error(
                operation,
                format!("{name} entries must contain two values"),
            ));
        }
        let id = extract_pollutant_id(&entry.get_item(0)?, operation)?;
        let value = extract_strict_number(&entry.get_item(1)?, name, operation)?;
        values.push((id, value));
    }
    Ok(values)
}

/// Extracts standard mapping-update arguments without resolving pollutant identities.
///
/// # Arguments
/// * `args` - Positional update arguments; at most one mapping or pair iterable is accepted.
/// * `kwargs` - Keyword pollutant-value entries.
/// * `name` - Public value name used in diagnostics.
/// * `operation` - Stable native operation name.
///
/// # Returns
/// Ordered raw pollutant-value pairs for one atomic solver mutation.
///
/// # Errors
/// Returns `ValidationError` for excess arguments or malformed keys and values.
pub(super) fn extract_pollutant_update(
    args: &Bound<'_, PyTuple>,
    kwargs: &Bound<'_, PyDict>,
    name: &str,
    operation: &str,
) -> PyResult<Vec<(String, f64)>> {
    if args.len() > 1 {
        return Err(pollutant_value_error(
            operation,
            format!("{name}.update accepts at most one positional argument"),
        ));
    }
    let mut values = if args.is_empty() {
        Vec::new()
    } else {
        extract_pollutant_values(&args.get_item(0)?, name, operation)?
    };
    for (key, value) in kwargs.iter() {
        values.push((
            extract_pollutant_id(&key, operation)?,
            extract_strict_number(&value, name, operation)?,
        ));
    }
    Ok(values)
}

/// Creates a private validation exception for malformed pollutant forcing input.
///
/// # Arguments
/// * `operation` - Stable native operation name.
/// * `detail` - Specific extraction diagnostic.
///
/// # Returns
/// Private native validation exception.
fn pollutant_value_error(operation: &str, detail: impl Into<String>) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(ErrorCode::ApiPropertyValue, detail.into()),
    )
}

/// Creates a private validation exception for an unknown property selector.
///
/// # Arguments
/// * `operation` - Stable native operation name.
/// * `property` - Rejected private property selector.
///
/// # Returns
/// Private native validation exception.
pub(super) fn invalid_property(operation: &str, property: &str) -> PyErr {
    native_error(
        operation,
        SwmmError::with_detail(
            ErrorCode::ApiPropertyValue,
            format!("unknown property {property:?}"),
        ),
    )
}
