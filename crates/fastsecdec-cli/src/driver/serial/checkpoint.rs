use crate::{CliResult, artifact::atomic_write, config::IntegrationInput};
use fastsecdec::{
    integration::{IntegrationProblem, OperationalMetrics, serial::SerialSession},
    kernel::ReplayState,
    status::EvaluationDiagnostics,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize, Deserialize)]
pub(super) struct Checkpoint {
    version: u32,
    settings: serde_json::Value,
    session: Vec<u8>,
    pub replay: BTreeMap<u64, ReplayState>,
    pub diagnostics: EvaluationDiagnostics,
    pub operational: OperationalMetrics,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contour: Option<fastsecdec::status::ContourCheckpointProvenance>,
}
#[derive(Serialize, Deserialize)]
struct Envelope {
    digest: String,
    checkpoint: Checkpoint,
}
fn identity(settings: &IntegrationInput) -> CliResult<serde_json::Value> {
    let mut value = super::super::checkpoint::settings_identity(settings)?;
    value.as_object_mut().unwrap().remove("serial_seconds");
    Ok(value)
}
#[allow(clippy::too_many_arguments)]
pub(super) fn save(
    path: &Path,
    settings: &IntegrationInput,
    session: &SerialSession,
    replay: &BTreeMap<u64, ReplayState>,
    diagnostics: &EvaluationDiagnostics,
    operational: &OperationalMetrics,
    contour_pilots: &BTreeMap<String, fastsecdec::status::ContourPilotProvenance>,
) -> CliResult<()> {
    let checkpoint = Checkpoint {
        version: 1,
        settings: identity(settings)?,
        session: session.checkpoint()?,
        replay: replay.clone(),
        diagnostics: diagnostics.clone(),
        operational: operational.clone(),
        contour: (settings.contour.deformation != fastsecdec::contour::ContourMode::Off).then(
            || fastsecdec::status::ContourCheckpointProvenance {
                validation: settings.contour.validation.clone(),
                pilots: contour_pilots.values().cloned().collect(),
            },
        ),
    };
    let digest = blake3::hash(&serde_json::to_vec(&checkpoint)?)
        .to_hex()
        .to_string();
    atomic_write(path, &serde_json::to_vec(&Envelope { digest, checkpoint })?)?;
    #[cfg(unix)]
    {
        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::File::open(directory)?.sync_all()?;
    }
    Ok(())
}
pub(super) fn restore(
    path: &Path,
    settings: &IntegrationInput,
    problem: &IntegrationProblem,
) -> CliResult<(SerialSession, Checkpoint)> {
    let envelope: Envelope = serde_json::from_slice(&fs::read(path)?)?;
    if blake3::hash(&serde_json::to_vec(&envelope.checkpoint)?)
        .to_hex()
        .as_str()
        != envelope.digest
    {
        return Err("serial checkpoint transaction checksum differs".into());
    }
    let checkpoint = envelope.checkpoint;
    if checkpoint.version != 1 || checkpoint.settings != identity(settings)? {
        return Err("serial checkpoint settings differ; only worker count, residence time, evaluation batch size, contour validation policy and contour diagnostics may change".into());
    }
    if checkpoint
        .replay
        .keys()
        .any(|id| !problem.sectors.iter().any(|s| s.id == *id))
    {
        return Err("serial replay checkpoint contains an unknown sector".into());
    }
    let mut session = SerialSession::restore(&checkpoint.session, problem)?;
    session.set_max_in_flight(settings.workers)?;
    Ok((session, checkpoint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_preserves_transaction_and_allows_only_operational_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("checkpoint.json");
        let settings = IntegrationInput {
            serial_seconds: Some(1.),
            ..Default::default()
        };
        let problem = IntegrationProblem::new(
            "serial-checkpoint-probe".into(),
            vec![0],
            Vec::new(),
            vec![2.],
        )
        .unwrap();
        let session = SerialSession::new(
            problem.clone(),
            super::super::coordinator::native_settings(&settings).unwrap(),
        )
        .unwrap();
        save(
            &path,
            &settings,
            &session,
            &BTreeMap::new(),
            &EvaluationDiagnostics::default(),
            &OperationalMetrics::default(),
            &BTreeMap::new(),
        )
        .unwrap();
        let mut changed = settings.clone();
        changed.workers = 3;
        changed.evaluation_batch_size = 19;
        changed.serial_seconds = Some(3.);
        changed.contour_diagnostics = fastsecdec::contour::ContourDiagnosticsMode::Aggregate;
        assert!(restore(&path, &changed, &problem).is_ok());
        changed.double_points = false;
        assert!(restore(&path, &changed, &problem).is_err());
        let mut bytes: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        bytes["checkpoint"]["diagnostics"]["evaluations"] = 1.into();
        fs::write(&path, serde_json::to_vec(&bytes).unwrap()).unwrap();
        assert!(restore(&path, &settings, &problem).is_err());
    }
}
