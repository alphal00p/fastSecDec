use serde::{Deserialize, Serialize};

use crate::kernel::PrecisionReport;

/// Caller-aggregated numerical kernel diagnostics. Evaluations include failed
/// attempts; conditioning checks and rescues count successful reports.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvaluationDiagnostics {
    pub evaluations: u64,
    pub conditioning_checks: u64,
    pub rescues: u64,
    pub max_precision_bits: u32,
    pub failures: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("evaluation diagnostics counter overflow")]
pub struct DiagnosticsOverflow;

impl std::fmt::Display for EvaluationDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} evaluations; {} conditioning checks; {} rescues; {} bits maximum; {} failures",
            self.evaluations,
            self.conditioning_checks,
            self.rescues,
            self.max_precision_bits,
            self.failures
        )
    }
}

impl EvaluationDiagnostics {
    pub fn record(&mut self, report: PrecisionReport) -> Result<(), DiagnosticsOverflow> {
        self.merge(&Self {
            evaluations: 1,
            conditioning_checks: u64::from(report.checked),
            rescues: u64::from(report.rescued),
            max_precision_bits: report.bits,
            failures: 0,
        })
    }

    pub fn record_failure(&mut self) -> Result<(), DiagnosticsOverflow> {
        self.merge(&Self {
            evaluations: 1,
            failures: 1,
            ..Self::default()
        })
    }

    /// Merge atomically: an overflow leaves all counters unchanged.
    pub fn merge(&mut self, other: &Self) -> Result<(), DiagnosticsOverflow> {
        let sum = |a: u64, b: u64| a.checked_add(b).ok_or(DiagnosticsOverflow);
        let combined = Self {
            evaluations: sum(self.evaluations, other.evaluations)?,
            conditioning_checks: sum(self.conditioning_checks, other.conditioning_checks)?,
            rescues: sum(self.rescues, other.rescues)?,
            max_precision_bits: self.max_precision_bits.max(other.max_precision_bits),
            failures: sum(self.failures, other.failures)?,
        };
        *self = combined;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_merge_success_failure_and_precision_without_partial_overflow() {
        let mut total = EvaluationDiagnostics::default();
        total
            .record(PrecisionReport {
                rescued: false,
                checked: false,
                bits: 53,
            })
            .unwrap();
        let mut worker = EvaluationDiagnostics::default();
        worker
            .record(PrecisionReport {
                rescued: true,
                checked: true,
                bits: 512,
            })
            .unwrap();
        worker.record_failure().unwrap();
        total.merge(&worker).unwrap();
        assert_eq!(
            total,
            EvaluationDiagnostics {
                evaluations: 3,
                conditioning_checks: 1,
                rescues: 1,
                max_precision_bits: 512,
                failures: 1,
            }
        );
        let prior = total.clone();
        assert!(
            total
                .merge(&EvaluationDiagnostics {
                    failures: u64::MAX,
                    ..Default::default()
                })
                .is_err()
        );
        assert_eq!(total, prior);
    }
}
