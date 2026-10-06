//! Small immutable native evaluator records, not per-coefficient reconstructions.
use fastsecdec::kernel::{EvaluatorOperations, EvaluatorStatistics};
use pyo3::prelude::*;

/// Complete shared sector evaluator size on the current backend.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "EvaluatorStatistics"
)]
pub(crate) struct PyEvaluatorStatistics {
    pub(crate) inner: EvaluatorStatistics,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyEvaluatorStatistics {
    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }
    #[getter]
    fn backend(&self) -> &str {
        &self.inner.backend
    }
    #[getter]
    fn arithmetic(&self) -> &str {
        &self.inner.arithmetic
    }
    #[getter]
    fn inputs(&self) -> usize {
        self.inner.inputs
    }
    /// Shared evaluator outputs, before complex outputs split into real/imaginary components.
    #[getter]
    fn outputs(&self) -> usize {
        self.inner.outputs
    }
    #[getter]
    fn exact_program_bytes(&self) -> usize {
        self.inner.exact_program_bytes
    }
    /// Symbolica exact-program operations before SymJIT lowering/optimization.
    #[getter]
    fn operations(&self) -> PyEvaluatorOperations {
        PyEvaluatorOperations {
            inner: self.inner.operations,
        }
    }
    /// Actual compressed SymJIT application bytes, not machine-code size; None for interpreter.
    #[getter]
    fn symjit_ir_bytes(&self) -> Option<usize> {
        self.inner.symjit_ir_bytes
    }
}

/// Native shared evaluator counts; these are not optimized machine-operation counts.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "EvaluatorOperations"
)]
pub(crate) struct PyEvaluatorOperations {
    inner: EvaluatorOperations,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyEvaluatorOperations {
    #[getter]
    fn additions(&self) -> usize {
        self.inner.additions
    }
    #[getter]
    fn multiplications(&self) -> usize {
        self.inner.multiplications
    }
    #[getter]
    fn inversions(&self) -> usize {
        self.inner.inversions
    }
    #[getter]
    fn function_calls(&self) -> usize {
        self.inner.function_calls
    }
}
