use fastsecdec::{
    integration::mc_discrete::HavanaDiscreteSession,
    kernel::{KernelSet, ReplayPolicy, ReplayState},
    status::{EvaluationDiagnostics, StoppingReason},
};
use pyo3::{prelude::*, types::PyBytes};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use super::PyHavanaDiscreteSession;
use crate::{error, execution::problem};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    version: u32,
    lane: String,
    session: Vec<u8>,
    replay: Vec<ReplayState>,
    policy: ReplayPolicy,
    diagnostics: EvaluationDiagnostics,
    stop_reason: Option<StoppingReason>,
}

impl PyHavanaDiscreteSession {
    pub(crate) fn restore_native(
        py: Python<'_>,
        kernels: Rc<KernelSet>,
        bytes: &[u8],
    ) -> PyResult<Self> {
        let state: Checkpoint =
            serde_json::from_slice(bytes).map_err(|e| error::native(py, "checkpoint", e))?;
        if state.version != 1
            || state.lane != "havana_discrete_mc"
            || state.replay.len() != kernels.sectors().len()
        {
            return Err(error::native(
                py,
                "checkpoint",
                "checkpoint lane, version or replay sector count differs",
            ));
        }
        let session = HavanaDiscreteSession::restore(&state.session, &problem(py, &kernels)?)
            .map_err(|e| error::native(py, "checkpoint", e))?;
        for (sector, replay) in state.replay.iter().enumerate() {
            kernels
                .validate_replay_state(sector, &state.policy, replay)
                .map_err(|e| error::native(py, "checkpoint", e))?;
        }
        let contexts = (0..kernels.sectors().len()).map(|_| None).collect();
        Ok(Self {
            kernels,
            session,
            replay: state.replay,
            policy: state.policy,
            worker: None,
            contexts,
            diagnostics: state.diagnostics,
            stop_reason: state.stop_reason,
            live_batches: Vec::new(),
            live_source: fastsecdec::integration::LiveSource::SinceResume,
        })
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyHavanaDiscreteSession {
    /// Persist accepted frozen-production batches and native replay state.
    /// Pilot training is deliberately not serialized; retain the session to resume it.
    fn checkpoint<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let state = Checkpoint {
            version: 1,
            lane: "havana_discrete_mc".into(),
            session: self
                .session
                .checkpoint()
                .map_err(|e| error::native(py, "checkpoint", e))?,
            replay: self.replay.clone(),
            policy: self.policy.clone(),
            diagnostics: self.diagnostics.clone(),
            stop_reason: self.stop_reason.clone(),
        };
        let bytes = serde_json::to_vec(&state).map_err(|e| error::native(py, "checkpoint", e))?;
        Ok(PyBytes::new(py, &bytes))
    }
}
