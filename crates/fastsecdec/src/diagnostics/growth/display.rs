//! Human presentation of the same typed report consumed by native callers.
use std::fmt;

use super::{BoundaryAssessment, BoundaryScanReport};

impl fmt::Display for BoundaryScanReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "╭─ Boundary scan ──────────────────────────────────────────────────────╮"
        )?;
        writeln!(f, "  {}", self.assessment)?;
        writeln!(
            f,
            "  Growth limit: {:.4} × approached axes + {:.1e}",
            self.options.growth.max_power_per_axis, self.options.growth.numerical_slack
        )?;
        writeln!(
            f,
            "  Retry scales refer to the original endpoint distances."
        )?;
        writeln!(f)?;
        writeln!(
            f,
            "  Attempt       Scale  Sectors       Probes  Flagged  Unclear"
        )?;
        writeln!(
            f,
            "  ───────  ──────────  ───────  ───────────  ───────  ───────"
        )?;
        for attempt in &self.attempts {
            let flagged = attempt
                .growth
                .sectors
                .iter()
                .filter(|s| s.assessment == BoundaryAssessment::Flagged)
                .count();
            let unclear = attempt
                .growth
                .sectors
                .iter()
                .filter(|s| s.assessment == BoundaryAssessment::Inconclusive)
                .count();
            writeln!(
                f,
                "  {:>7}  {:>10.3e}  {:>7}  {:>11}  {:>7}  {:>7}",
                attempt
                    .index
                    .checked_add(1)
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "invalid-attempt".to_owned()),
                attempt.scale,
                attempt.selected_sectors.len(),
                format!(
                    "{}/{}",
                    attempt.samples.probes.len(),
                    attempt.samples.coverage.planned_probes
                ),
                flagged,
                unclear,
            )?;
        }
        writeln!(f)?;
        writeln!(
            f,
            "  Execution: {:?} · {}/{} total probes",
            self.stop, self.completed_probes, self.options.sampling.max_probes
        )?;
        writeln!(
            f,
            "  Checks {} · Rescues {} · Maximum precision {} bits",
            self.diagnostics.conditioning_checks,
            self.diagnostics.rescues,
            self.diagnostics.max_precision_bits
        )?;
        writeln!(
            f,
            "  Evaluations {} · Failures {}",
            self.diagnostics.evaluations, self.diagnostics.failures
        )?;
        if self.had_prior_flags {
            writeln!(
                f,
                "  Earlier flagged or inconclusive attempts remain in the report."
            )?;
        }
        writeln!(
            f,
            "  Sampled growth is a diagnostic, not an integrability certificate."
        )?;
        writeln!(
            f,
            "╰──────────────────────────────────────────────────────────────────────╯"
        )
    }
}
