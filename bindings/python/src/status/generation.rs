use fastsecdec::{
    generation::{CoefficientExpansionMethod, CoefficientExpansionStage, CoefficientRequestCounts},
    status::{
        CoefficientExpansionSnapshot, GenerationSnapshot, GenerationStage, GenerationTimings,
    },
};
use pyo3::prelude::*;

/// Generation progress copied from the native caller-owned observer.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    from_py_object,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "GenerationSnapshot"
)]
#[derive(Clone)]
pub(crate) struct PyGenerationSnapshot {
    pub(crate) inner: GenerationSnapshot,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGenerationSnapshot {
    /// Stable snake_case name of the native generation stage.
    #[getter]
    fn stage(&self) -> &'static str {
        match self.inner.stage {
            GenerationStage::Input => "input",
            GenerationStage::Parametrization => "parametrization",
            GenerationStage::Geometry => "geometry",
            GenerationStage::Mapping => "mapping",
            GenerationStage::Symmetry => "symmetry",
            GenerationStage::Subtraction => "subtraction",
            GenerationStage::Expansion => "expansion",
            GenerationStage::CoefficientExpansion => "coefficient_expansion",
            GenerationStage::Compilation => "compilation",
            GenerationStage::Complete => "complete",
        }
    }

    #[getter]
    fn completed(&self) -> usize {
        self.inner.completed
    }

    #[getter]
    fn total(&self) -> Option<usize> {
        self.inner.total
    }

    #[getter]
    fn sectors(&self) -> usize {
        self.inner.sectors
    }

    #[getter]
    fn kernels(&self) -> usize {
        self.inner.kernels
    }

    #[getter]
    fn elapsed_seconds(&self) -> f64 {
        self.inner.elapsed_seconds
    }

    #[getter]
    fn timings(&self) -> PyGenerationTimings {
        PyGenerationTimings {
            inner: self.inner.timings.clone(),
        }
    }

    /// Current or last named-coefficient attempt; None for physical-only generation.
    #[getter]
    fn coefficient_expansion(&self) -> Option<PyCoefficientExpansionSnapshot> {
        self.inner
            .coefficient_expansion
            .clone()
            .map(|inner| PyCoefficientExpansionSnapshot { inner })
    }

    #[getter]
    fn detail(&self) -> String {
        self.inner.detail.clone()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}

/// Native observed generation wall times in seconds.
///
/// The total ends after preparing kernels and metadata, before artifact writing.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    from_py_object,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "GenerationTimings"
)]
#[derive(Clone)]
pub(crate) struct PyGenerationTimings {
    inner: GenerationTimings,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGenerationTimings {
    #[getter]
    fn input_seconds(&self) -> f64 {
        self.inner.input_seconds
    }

    #[getter]
    fn parametrization_seconds(&self) -> f64 {
        self.inner.parametrization_seconds
    }

    #[getter]
    fn domain_seconds(&self) -> f64 {
        self.inner.domain_seconds
    }

    #[getter]
    fn geometry_seconds(&self) -> f64 {
        self.inner.geometry_seconds
    }

    #[getter]
    fn mapping_seconds(&self) -> f64 {
        self.inner.mapping_seconds
    }

    #[getter]
    fn symmetry_seconds(&self) -> f64 {
        self.inner.symmetry_seconds
    }

    #[getter]
    fn subtraction_seconds(&self) -> f64 {
        self.inner.subtraction_seconds
    }

    #[getter]
    fn laurent_seconds(&self) -> f64 {
        self.inner.laurent_seconds
    }

    /// Entire named phase, including any exact physical fallback.
    #[getter]
    fn coefficient_expansion_seconds(&self) -> f64 {
        self.inner.coefficient_expansion_seconds
    }

    #[getter]
    fn compilation_seconds(&self) -> f64 {
        self.inner.compilation_seconds
    }

    #[getter]
    fn total_seconds(&self) -> f64 {
        self.inner.total_seconds
    }
}

/// Native per-representative coefficient work for the current or last attempt.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    from_py_object,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "CoefficientExpansionSnapshot"
)]
#[derive(Clone)]
pub(crate) struct PyCoefficientExpansionSnapshot {
    inner: CoefficientExpansionSnapshot,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyCoefficientExpansionSnapshot {
    /// Zero-based representative being processed or most recently completed.
    #[getter]
    fn sector(&self) -> usize {
        self.inner.sector
    }

    #[getter]
    fn requested_method(&self) -> &'static str {
        coefficient_method(self.inner.requested_method)
    }

    /// Physical also identifies an exact unregulated-endpoint fallback.
    #[getter]
    fn effective_method(&self) -> &'static str {
        coefficient_method(self.inner.effective_method)
    }

    #[getter]
    fn stage(&self) -> &'static str {
        match self.inner.stage {
            CoefficientExpansionStage::Admission => "admission",
            CoefficientExpansionStage::RegularSeries => "regular_series",
            CoefficientExpansionStage::Naming => "naming",
            CoefficientExpansionStage::Endpoint => "endpoint",
            CoefficientExpansionStage::Composition => "composition",
            CoefficientExpansionStage::Coverage => "coverage",
            CoefficientExpansionStage::Lowering => "lowering",
            CoefficientExpansionStage::PhysicalFallback => "physical_fallback",
            CoefficientExpansionStage::Complete => "complete",
        }
    }

    /// One-based native attempt; zero means admission or exact physical fallback.
    #[getter]
    fn attempt(&self) -> usize {
        self.inner.attempt
    }

    /// Native relative width; zero has the same pre-attempt/fallback meaning.
    /// This is not an absolute Laurent cutoff.
    #[getter]
    fn relative_width(&self) -> i64 {
        self.inner.relative_width
    }

    /// Named-composition pieces in this attempt; zero for physical fallback.
    #[getter]
    fn formal_pieces(&self) -> usize {
        self.inner.formal_pieces
    }

    #[getter]
    fn requests(&self) -> PyCoefficientRequestCounts {
        PyCoefficientRequestCounts {
            inner: self.inner.requests,
        }
    }
}

fn coefficient_method(method: CoefficientExpansionMethod) -> &'static str {
    match method {
        CoefficientExpansionMethod::Physical => "physical",
        CoefficientExpansionMethod::NativeNamed => "native_named",
    }
}

/// Native representation counts, reset at each coefficient-series attempt.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    from_py_object,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "CoefficientRequestCounts"
)]
#[derive(Clone)]
pub(crate) struct PyCoefficientRequestCounts {
    inner: CoefficientRequestCounts,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyCoefficientRequestCounts {
    #[getter]
    fn source_bodies(&self) -> usize {
        self.inner.source_bodies
    }

    #[getter]
    fn unique_requests(&self) -> usize {
        self.inner.unique_requests
    }

    #[getter]
    fn cached_partials(&self) -> usize {
        self.inner.cached_partials
    }

    #[getter]
    fn aliases(&self) -> usize {
        self.inner.aliases
    }

    #[getter]
    fn interleaved_requests(&self) -> usize {
        self.inner.interleaved_requests
    }

    #[getter]
    fn fallback_requests(&self) -> usize {
        self.inner.fallback_requests
    }
}
