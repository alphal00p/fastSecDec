//! Optional HEPKit Python bindings. The native FastSecDec core stays Python-free.

//! Thin HEPKit ownership and caller-stepped execution boundary.
mod citations;
mod contour;
mod decompose;
mod error;
mod execution;
mod family;
mod generation;
mod generation_session;
mod input;
mod inspection;
mod kernels;
mod mc;
mod presentation;
mod progress;
mod session;
mod settings;
mod status;
#[cfg(feature = "native")]
mod threshold;

use pyo3::{
    prelude::*,
    types::{PyDict, PyModule, PyTuple},
};

pub use citations::get_citations;
use symbolica::api::python::{ExtensionImplementation, PythonExtensionFunction};

static SECTOR_DECOMPOSE: ExtensionImplementation<PythonExtensionFunction> =
    ExtensionImplementation::new("FastSecDec", sector_decompose);

fn sector_decompose(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    // Reuse the binding's argument parser and defaults without resolving a
    // Python module or duplicating the scientific input conversion.
    let function = wrap_pyfunction!(decompose::sector_decompose, args.py())?;
    Ok(function.call(args, kwargs)?.unbind())
}

pub fn register(hep: &Bound<'_, PyModule>) -> PyResult<()> {
    let module = PyModule::new(hep.py(), "symbolica.community.hepkit_fastsecdec_native")?;
    module.add_function(wrap_pyfunction!(decompose::sector_decompose, &module)?)?;
    module.add_function(wrap_pyfunction!(input::with_diagram_expressions, &module)?)?;
    module.add_class::<input::PyIntegral>()?;
    module.add_class::<generation::PyGeneratedIntegral>()?;
    module.add_class::<generation_session::PyGenerationSession>()?;
    module.add_class::<kernels::PyKernels>()?;
    module.add_class::<settings::PyCompilationSettings>()?;
    module.add_class::<settings::PyStabilitySettings>()?;
    contour::register(&module)?;
    family::register(&module)?;
    #[cfg(feature = "native")]
    threshold::register(&module)?;
    module.add_class::<session::PyQmcSettings>()?;
    module.add_class::<session::PyQmcSession>()?;
    module.add_class::<mc::PyHavanaDiscreteSettings>()?;
    module.add_class::<mc::PyHavanaDiscreteSession>()?;
    error::register(&module)?;
    status::register(&module)?;
    inspection::register(&module)?;
    hep.add("_fastsecdec_native", &module)?;
    hep.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("symbolica.community.hepkit_fastsecdec_native", &module)?;
    feynkit_py::SECTOR_DECOMPOSE.register(&SECTOR_DECOMPOSE)?;
    Ok(())
}

/// Include native exception declarations in the host's generated package stub.
#[cfg(feature = "python_stubgen")]
pub fn stub_source(module: &pyo3_stub_gen::generate::Module) -> String {
    module.to_string()
        + "\nclass FastSecDecError(RuntimeError):\n    stage: str\n\nclass CancelledError(FastSecDecError): ...\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use symbolica::api::python::{SymbolicaCommunityModule, create_symbolica_module};

    #[test]
    fn owner_methods_use_registered_backend_without_canonical_namespaces() {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let core = PyModule::new(py, "standalone_core")?;
            create_symbolica_module(&core)?;
            let hep = PyModule::new(py, "standalone_hep")?;
            feynkit_py::FeynkitModule::register_module(&hep)?;
            feynkit_py::FeynkitModule::initialize(py)?;
            register(&hep)?;
            register(&hep)?;
            let locals = PyDict::new(py);
            locals.set_item("core", core)?;
            locals.set_item("sd", hep.getattr("_fastsecdec_native")?)?;
            locals.set_item("hep", hep)?;
            let source = CString::new(include_str!("../tests/standalone_backend.py")).unwrap();
            py.run(&source, Some(&locals), Some(&locals))
        })
        .unwrap();
    }
}
