//! Caller-owned sampling phase attribution of native operational observations.
use crate::contour::ContourRuntimeReport;
use serde::{Deserialize, Serialize};

use super::{DiagnosticsOverflow, IntegrationStage};

/// Actual callback work, including retries and discarded batches. The nested
/// native report preserves preparation, exact binding and independent pilots;
/// none of these counters describes accepted integration samples.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourRuntimeDiagnostics {
    pub adaptation: ContourRuntimeReport,
    pub production: ContourRuntimeReport,
}

impl ContourRuntimeDiagnostics {
    /// Merge atomically using the native report's checked counters.
    pub fn merge(&mut self, other: &Self) -> Result<(), DiagnosticsOverflow> {
        let mut next = self.clone();
        next.adaptation.merge(&other.adaptation)?;
        next.production.merge(&other.production)?;
        *self = next;
        Ok(())
    }

    /// Record one drained owner delta at its actual sampling-phase boundary.
    /// Repeated cumulative snapshots must not be passed to this method.
    pub fn record(
        &mut self,
        stage: IntegrationStage,
        report: &ContourRuntimeReport,
    ) -> Result<(), DiagnosticsOverflow> {
        match stage {
            IntegrationStage::Pilot => self.adaptation.merge(report),
            IntegrationStage::Production => self.production.merge(report),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::EvaluationDiagnostics;

    #[test]
    fn runtime_work_preserves_sampling_and_native_phases_without_sample_counts() {
        let mut diagnostics = EvaluationDiagnostics::default();
        let mut report = ContourRuntimeReport::default();
        report.preparation.callback_calls = 2;
        report.exact.callback_calls = 3;
        report.pilot.callback_calls = 5;
        report.evaluation.callback_calls = 7;
        report.conditioning.callback_calls = 11;
        diagnostics
            .record_contour_runtime(IntegrationStage::Pilot, &report)
            .unwrap();
        report.evaluation.callback_calls = 13;
        diagnostics
            .record_contour_runtime(IntegrationStage::Production, &report)
            .unwrap();
        assert_eq!(diagnostics.evaluations, 0);
        assert!(diagnostics.contour.is_none());
        let phases = diagnostics.contour_runtime.as_ref().unwrap();
        assert_eq!(phases.adaptation.evaluation.callback_calls, 7);
        assert_eq!(phases.production.evaluation.callback_calls, 13);
        assert_eq!(phases.production.preparation.callback_calls, 2);
        assert_eq!(phases.production.exact.callback_calls, 3);
        assert_eq!(phases.production.pilot.callback_calls, 5);
        assert_eq!(phases.production.conditioning.callback_calls, 11);
        let saved = serde_json::to_vec(&diagnostics).unwrap();
        assert_eq!(
            serde_json::from_slice::<EvaluationDiagnostics>(&saved).unwrap(),
            diagnostics
        );
        let old: EvaluationDiagnostics = serde_json::from_str("{\"evaluations\":2}").unwrap();
        assert!(old.contour_runtime.is_none());
        assert!(
            !serde_json::to_value(old)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("contour_runtime")
        );
    }

    #[test]
    fn runtime_overflow_never_partially_merges_other_diagnostics_or_phases() {
        let mut total = EvaluationDiagnostics::default();
        let mut full = ContourRuntimeReport::default();
        full.evaluation.callback_calls = u64::MAX;
        total
            .record_contour_runtime(IntegrationStage::Production, &full)
            .unwrap();
        let mut incoming = EvaluationDiagnostics {
            evaluations: 3,
            ..Default::default()
        };
        let mut report = ContourRuntimeReport::default();
        report.evaluation.callback_calls = 1;
        incoming
            .record_contour_runtime(IntegrationStage::Pilot, &report)
            .unwrap();
        incoming
            .record_contour_runtime(IntegrationStage::Production, &report)
            .unwrap();
        let before = total.clone();
        assert!(total.merge(&incoming).is_err());
        assert_eq!(total, before);
        assert!(
            total
                .record_contour_runtime(IntegrationStage::Production, &report)
                .is_err()
        );
        assert_eq!(total, before);
    }
}
