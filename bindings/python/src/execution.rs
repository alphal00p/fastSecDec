//! Common native problem/precision ownership for caller-stepped execution lanes.
use fastsecdec::{
    integration::IntegrationProblem,
    kernel::{KernelSet, ReplayPolicy, ReplayState},
    results::{KernelResultManifest, ResultScope},
};
use pyo3::prelude::*;

pub(crate) fn problem(py: Python<'_>, kernels: &KernelSet) -> PyResult<IntegrationProblem> {
    KernelResultManifest::from_kernels(kernels)
        .integration_problem(&ResultScope::FullIntegral, kernels.content_id())
        .map_err(|e| crate::error::native(py, "configuration", e))
}

pub(crate) fn replay_states(
    py: Python<'_>,
    kernels: &KernelSet,
    policy: &ReplayPolicy,
) -> PyResult<Vec<ReplayState>> {
    (0..kernels.sectors().len())
        .map(|sector| kernels.replay_state(sector, policy.clone()))
        .collect::<Result<_, _>>()
        .map_err(|e| crate::error::native(py, "configuration", e))
}
