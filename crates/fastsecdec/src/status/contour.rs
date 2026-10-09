//! Caller-aggregated contour evidence. These records describe checks actually
//! performed; they are not a certificate for unsampled points or JIT rounding.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{DiagnosticsOverflow, EvaluationDiagnostics, IntegrationStage};
use crate::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    kernel::{ContourProductionReport, ContourValidationReport},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourCheckCounters {
    /// Successfully certified polynomial-map arguments, including face checks.
    pub checked_arguments: u64,
    pub maximum_bits: u32,
}

impl ContourCheckCounters {
    pub fn merge(&mut self, other: Self) -> Result<(), DiagnosticsOverflow> {
        let checked_arguments = self
            .checked_arguments
            .checked_add(other.checked_arguments)
            .ok_or(DiagnosticsOverflow)?;
        self.checked_arguments = checked_arguments;
        self.maximum_bits = self.maximum_bits.max(other.maximum_bits);
        Ok(())
    }
}

/// Integration adaptation is separate from the independent contour preflight.
/// Neither kind of pilot is evidence counted in a production estimate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourEvaluationDiagnostics {
    pub production: ContourCheckCounters,
    pub adaptation: ContourCheckCounters,
}

impl ContourEvaluationDiagnostics {
    pub fn merge(&mut self, other: Self) -> Result<(), DiagnosticsOverflow> {
        let mut next = *self;
        next.production.merge(other.production)?;
        next.adaptation.merge(other.adaptation)?;
        *self = next;
        Ok(())
    }

    /// `report` must be a drained worker delta, never a repeatedly submitted
    /// cumulative snapshot. No production statistics or RNG state are changed.
    pub fn record(
        &mut self,
        stage: IntegrationStage,
        report: &ContourProductionReport,
    ) -> Result<(), DiagnosticsOverflow> {
        let delta = ContourCheckCounters {
            checked_arguments: u64::try_from(report.checked_arguments)
                .map_err(|_| DiagnosticsOverflow)?,
            maximum_bits: report.maximum_bits,
        };
        match stage {
            IntegrationStage::Pilot => self.adaptation.merge(delta),
            IntegrationStage::Production => self.production.merge(delta),
        }
    }
}

/// Evidence for one bound native owner. Chart indices are local to that owner;
/// its content identity includes the mathematical prescription and parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourPilotProvenance {
    pub protocol: String,
    pub kernel_content_id: String,
    /// Caller-supplied validation coordinates need not originate from an RNG.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    pub policy: ContourValidation,
    pub required_charts: Vec<usize>,
    pub validated_charts: Vec<usize>,
    pub complete: bool,
    pub sampled_points: usize,
    pub checked_arguments: usize,
    pub maximum_bits: u32,
}

impl ContourPilotProvenance {
    pub fn from_report(
        protocol: impl Into<String>,
        kernel_content_id: impl Into<String>,
        seed: impl Into<Option<u64>>,
        report: &ContourValidationReport,
    ) -> Self {
        Self {
            protocol: protocol.into(),
            kernel_content_id: kernel_content_id.into(),
            seed: seed.into(),
            policy: report.policy,
            required_charts: report.required_charts.clone(),
            validated_charts: report.validated_charts.clone(),
            complete: report.pilot_complete,
            sampled_points: report.accepted_pilot_points,
            checked_arguments: report.checked_arguments,
            maximum_bits: report.maximum_bits,
        }
    }
}

/// Durable validation evidence, separate from mathematical checkpoint identity.
/// A policy-only change preserves earlier pilots; these records never authorize
/// a freshly loaded evaluator to bypass its own required preflight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourCheckpointProvenance {
    pub validation: ContourValidationOptions,
    pub pilots: Vec<ContourPilotProvenance>,
}

impl ContourCheckpointProvenance {
    /// Keep the latest supplied evidence for each bound native owner. Storage is
    /// bounded by the number of owners, independently of residence visits.
    pub fn update(
        previous: Option<&Self>,
        validation: &ContourValidationOptions,
        pilots: impl IntoIterator<Item = ContourPilotProvenance>,
    ) -> Self {
        let mut owners: BTreeMap<String, ContourPilotProvenance> = previous
            .into_iter()
            .flat_map(|previous| previous.pilots.iter().cloned())
            .map(|pilot| (pilot.kernel_content_id.clone(), pilot))
            .collect();
        for pilot in pilots {
            owners.insert(pilot.kernel_content_id.clone(), pilot);
        }
        Self {
            validation: validation.clone(),
            pilots: owners.into_values().collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourRunReport {
    pub deformation: ContourMode,
    pub validation: ContourValidationOptions,
    pub pilots: Vec<ContourPilotProvenance>,
    /// Actual observed checks during this invocation, including unfinished work.
    pub invocation_checks: ContourEvaluationDiagnostics,
    /// Recorded diagnostics, including available checkpoint history. Legacy
    /// checkpoints can leave part of that history unknown. `None` means no
    /// contour counters were retained, not proof of zero checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_checks: Option<ContourEvaluationDiagnostics>,
}

impl ContourRunReport {
    pub fn new(
        settings: &ContourSettings,
        pilots: Vec<ContourPilotProvenance>,
        invocation: &EvaluationDiagnostics,
        recorded: &EvaluationDiagnostics,
    ) -> Self {
        Self {
            deformation: settings.deformation,
            validation: settings.validation.clone(),
            pilots,
            invocation_checks: invocation.contour.unwrap_or_default(),
            recorded_checks: recorded.contour,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pilot(owner: &str, seed: u64) -> ContourPilotProvenance {
        ContourPilotProvenance {
            protocol: "independent-validation-v1".into(),
            kernel_content_id: owner.into(),
            seed: Some(seed),
            policy: ContourValidation::Always,
            required_charts: vec![0, 2],
            validated_charts: vec![0, 2],
            complete: true,
            sampled_points: 16,
            checked_arguments: 32,
            maximum_bits: 96,
        }
    }

    #[test]
    fn checkpoint_evidence_survives_unchecked_resume_and_stays_bounded() {
        let settings = ContourValidationOptions::default();
        let original =
            ContourCheckpointProvenance::update(None, &settings, [pilot("b", 1), pilot("a", 2)]);
        let encoded = serde_json::to_vec(&original).unwrap();
        let restored: ContourCheckpointProvenance = serde_json::from_slice(&encoded).unwrap();
        let mut supplied_points = serde_json::to_value(&original.pilots[0]).unwrap();
        supplied_points.as_object_mut().unwrap().remove("seed");
        let supplied_points: ContourPilotProvenance =
            serde_json::from_value(supplied_points).unwrap();
        assert_eq!(supplied_points.seed, None);
        let unchecked = ContourValidationOptions {
            policy: ContourValidation::Off,
            ..settings.clone()
        };
        let mut current = ContourCheckpointProvenance::update(Some(&restored), &unchecked, []);
        assert_eq!(current.validation.policy, ContourValidation::Off);
        assert_eq!(current.pilots.len(), 2);
        assert_eq!(current.pilots[0].kernel_content_id, "a");
        assert_eq!(current.pilots[1].seed, Some(1));
        // Repeated residence refreshes one owner's proof, not its production
        // counters, and never grows a history proportional to run duration.
        for seed in 3..100 {
            current =
                ContourCheckpointProvenance::update(Some(&current), &settings, [pilot("a", seed)]);
        }
        assert_eq!(current.pilots.len(), 2);
        assert_eq!(current.pilots[0].seed, Some(99));
        assert_eq!(current.pilots[1].seed, Some(1));
        assert_eq!(current.pilots[0].validated_charts, vec![0, 2]);
    }

    #[test]
    fn sampling_phases_keep_check_counts_and_precision_separate() {
        let mut counters = ContourEvaluationDiagnostics::default();
        for (stage, count, bits) in [
            (IntegrationStage::Pilot, 8, 192),
            (IntegrationStage::Production, 12, 96),
            (IntegrationStage::Production, 4, 96),
        ] {
            counters
                .record(
                    stage,
                    &ContourProductionReport {
                        policy: ContourValidation::Always,
                        checked_arguments: count,
                        maximum_bits: bits,
                    },
                )
                .unwrap();
        }
        assert_eq!(counters.adaptation.checked_arguments, 8);
        assert_eq!(counters.adaptation.maximum_bits, 192);
        assert_eq!(counters.production.checked_arguments, 16);
        assert_eq!(counters.production.maximum_bits, 96);
        let before = counters;
        assert!(
            counters
                .merge(ContourEvaluationDiagnostics {
                    production: ContourCheckCounters {
                        checked_arguments: u64::MAX,
                        maximum_bits: 768,
                    },
                    ..Default::default()
                })
                .is_err()
        );
        assert_eq!(counters, before);
    }
}
