//! Optional HEPKit Python bindings. The native FastSecDec core stays Python-free.

//! Thin HEPKit ownership and caller-stepped execution boundary.
mod citations;
mod decompose;
mod error;
mod execution;
mod generation;
mod input;
mod inspection;
mod kernels;
mod mc;
mod progress;
mod session;
mod status;

use pyo3::{prelude::*, types::PyModule};

pub use citations::get_citations;

pub fn register(hep: &Bound<'_, PyModule>) -> PyResult<()> {
    let module = PyModule::new(hep.py(), "symbolica.community.hepkit_fastsecdec_native")?;
    module.add_function(wrap_pyfunction!(decompose::sector_decompose, &module)?)?;
    module.add_function(wrap_pyfunction!(input::with_diagram_expressions, &module)?)?;
    module.add_class::<input::PyIntegral>()?;
    module.add_class::<generation::PyGeneratedIntegral>()?;
    module.add_class::<kernels::PyKernels>()?;
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
    Ok(())
}

/// Include native exception declarations in the host's generated package stub.
#[cfg(feature = "python_stubgen")]
pub fn stub_source(module: &pyo3_stub_gen::generate::Module) -> String {
    module.to_string()
        + "\nclass FastSecDecError(RuntimeError):\n    stage: str\n\nclass CancelledError(FastSecDecError): ...\n"
}
