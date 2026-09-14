use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use swmmrs::engine::consts::{SOLVER_BUILD_ID, SOLVER_VERSION};

create_exception!(_swmmrs, NativeOutputError, PyException);

mod native;

/// Initializes the package-private native module.
///
/// # Arguments
/// * `module` - Python module receiving owner and immutable solver identity values.
///
/// # Returns
/// `Ok(())` after every native module attribute is installed.
///
/// # Errors
/// Returns a Python exception when a module attribute cannot be installed.
#[pymodule(gil_used = false)]
fn _swmmrs(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<native::NativeSimulation>()?;
    module.add_class::<native::NativeOutputReader>()?;
    module.add_function(wrap_pyfunction!(native::bind_public_types, module)?)?;
    native::register_records(module)?;
    module.add(
        "NativeOutputError",
        module.py().get_type::<NativeOutputError>(),
    )?;
    module.add("solver_version", SOLVER_VERSION)?;
    module.add("solver_build_id", SOLVER_BUILD_ID)?;
    Ok(())
}
