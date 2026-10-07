use std::rc::Rc;

use fastsecdec::{
    integration::mc_discrete::{
        HavanaDiscreteSession, HavanaDiscreteSettings, HavanaDiscreteWorker,
    },
    kernel::{KernelSet, ReplayPolicy, ReplayState, WeightedEvaluationContext},
    status::{EvaluationDiagnostics, IntegrationStage, StoppingReason},
};
use pyo3::prelude::*;

use super::PyHavanaDiscreteSettings;
use crate::{
    error,
    execution::{problem, replay_states},
    status::PyIntegrationSnapshot,
};

/// Caller-owned native sector importance sampling. Pilot pauses retain this object;
/// persistent checkpoints require frozen production. No worker pool is created.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "HavanaDiscreteSession",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable
)]
pub(crate) struct PyHavanaDiscreteSession {
    pub(super) kernels: Rc<KernelSet>,
    pub(super) session: HavanaDiscreteSession,
    pub(super) replay: Vec<ReplayState>,
    pub(super) policy: ReplayPolicy,
    pub(super) worker: Option<HavanaDiscreteWorker>,
    pub(super) contexts: Vec<Option<WeightedEvaluationContext>>,
    pub(super) diagnostics: EvaluationDiagnostics,
    pub(super) stop_reason: Option<StoppingReason>,
    pub(super) live_batches: Vec<fastsecdec::integration::McLiveBatch>,
    pub(super) live_source: fastsecdec::integration::LiveSource,
}

impl PyHavanaDiscreteSession {
    pub(crate) fn new(
        py: Python<'_>,
        kernels: Rc<KernelSet>,
        settings: HavanaDiscreteSettings,
        pilot: bool,
        probabilities: Option<&[f64]>,
    ) -> PyResult<Self> {
        let problem = problem(py, &kernels)?;
        let session = if pilot {
            HavanaDiscreteSession::pilot(problem, settings)
        } else {
            HavanaDiscreteSession::production(problem, settings)
        }
        .map_err(|e| error::native(py, "configuration", e))?;
        let session = match probabilities {
            Some(values) => session
                .with_sector_probabilities(values)
                .map_err(|e| error::native(py, "configuration", e))?,
            None => session,
        };
        let policy = ReplayPolicy::default();
        let replay = replay_states(py, &kernels, &policy)?;
        let contexts = (0..kernels.sectors().len()).map(|_| None).collect();
        Ok(Self {
            kernels,
            session,
            replay,
            policy,
            worker: None,
            contexts,
            diagnostics: EvaluationDiagnostics::default(),
            stop_reason: None,
            live_batches: Vec::new(),
            live_source: fastsecdec::integration::LiveSource::CurrentIteration,
        })
    }

    pub(super) fn discard_contexts(&mut self) {
        self.worker = None;
        for context in &mut self.contexts {
            *context = None;
        }
    }

    fn begin_phase(&mut self) {
        self.discard_contexts();
        // Accepted precision knowledge survives adaptation; phase statistics do not.
        self.diagnostics = EvaluationDiagnostics::default();
        self.stop_reason = None;
        self.live_batches.clear();
        self.live_source = fastsecdec::integration::LiveSource::CurrentIteration;
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyHavanaDiscreteSession {
    fn observation(&self, py: Python<'_>) -> PyResult<crate::status::PyIntegrationObservation> {
        let mut inner = self
            .session
            .diagnostic_observation()
            .map_err(|e| error::native(py, "integration", e))?;
        inner.snapshot.evaluation_diagnostics = Some(self.diagnostics.clone());
        inner.snapshot.stop_reason = self.stop_reason.clone();
        Ok(crate::status::PyIntegrationObservation { inner })
    }
    /// Native point-statistics preview, separate from accepted complete-batch covariance.
    fn live_observation(&self, py: Python<'_>) -> PyResult<crate::status::PyLiveObservation> {
        let inner = fastsecdec::integration::mc_live_observation(
            self.session.problem(),
            self.session.stage(),
            self.live_source,
            &self.live_batches,
            true,
        )
        .map_err(|e| error::native(py, "integration", e))?;
        Ok(crate::status::PyLiveObservation { inner })
    }
    /// Whether this pilot epoch or production allocation has completed.
    #[getter]
    fn complete(&self) -> bool {
        self.session.is_complete()
    }
    #[getter]
    fn stage(&self) -> &'static str {
        match self.session.stage() {
            IntegrationStage::Pilot => "pilot",
            IntegrationStage::Production => "production",
        }
    }
    #[getter]
    fn checkpoint_available(&self) -> bool {
        self.session.stage() == IntegrationStage::Production
    }
    #[getter]
    fn settings(&self) -> PyHavanaDiscreteSettings {
        PyHavanaDiscreteSettings {
            inner: self.session.settings().clone(),
        }
    }
    /// (Native sector id, current selection probability) in native problem order.
    #[getter]
    fn sector_probabilities(&self) -> Vec<(u64, f64)> {
        self.session.sector_probabilities()
    }

    pub(super) fn snapshot(&self, py: Python<'_>) -> PyResult<PyIntegrationSnapshot> {
        let mut inner = self
            .session
            .diagnostic_observation()
            .map_err(|e| error::native(py, "integration", e))?
            .snapshot;
        inner.evaluation_diagnostics = Some(self.diagnostics.clone());
        inner.stop_reason = self.stop_reason.clone();
        Ok(PyIntegrationSnapshot { inner })
    }

    /// Adapt only a completed pilot and start another independent native pilot epoch.
    #[pyo3(signature = (*, discrete_learning_rate=1.0, continuous_learning_rate=1.0))]
    fn adapt_pilot(
        &mut self,
        py: Python<'_>,
        discrete_learning_rate: f64,
        continuous_learning_rate: f64,
    ) -> PyResult<PyIntegrationSnapshot> {
        self.session
            .adapt_pilot(discrete_learning_rate, continuous_learning_rate)
            .map_err(|e| error::native(py, "integration", e))?;
        self.begin_phase();
        self.snapshot(py)
    }

    /// Adapt a completed pilot, freeze both grids, and discard all pilot estimates.
    #[pyo3(signature = (*, points_per_batch, batches, discrete_learning_rate=1.0, continuous_learning_rate=1.0))]
    fn freeze_production(
        &mut self,
        py: Python<'_>,
        points_per_batch: usize,
        batches: u32,
        discrete_learning_rate: f64,
        continuous_learning_rate: f64,
    ) -> PyResult<PyIntegrationSnapshot> {
        self.session
            .freeze_production(
                discrete_learning_rate,
                continuous_learning_rate,
                points_per_batch,
                batches,
            )
            .map_err(|e| error::native(py, "integration", e))?;
        self.begin_phase();
        self.snapshot(py)
    }
}
