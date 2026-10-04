use std::{collections::BTreeMap, ops::ControlFlow};

use super::{analysis::aggregate, *};
use crate::{
    diagnostics::{
        DiagnosticError, DiagnosticStop, Result,
        boundary::{boundaries_selected, scaled_distances},
    },
    kernel::KernelSet,
    status::EvaluationDiagnostics,
};

fn validate(options: &BoundaryScanOptions) -> Result<()> {
    options.growth.threshold(options.sampling.max_codimension)?;
    if options.sampling.exponents.len() < 2 {
        return Err(DiagnosticError::Invalid(
            "growth analysis needs at least two distinct distances",
        ));
    }
    scaled_distances(&options.sampling, 1.0)?;
    let mut previous = 1.0;
    for &scale in &options.retry_scales {
        if !scale.is_finite() || scale <= 0.0 || scale >= previous {
            return Err(DiagnosticError::Invalid(
                "retry scales must be finite, positive and strictly decreasing from one",
            ));
        }
        scaled_distances(&options.sampling, scale)?;
        previous = scale;
    }
    Ok(())
}

/// Run an initial boundary-growth scan and bounded retries of concerned sectors.
/// Scales always refer to the original distances; every evaluation consumes the
/// same global budget. Completed raw rows and earlier concerns are never erased.
pub fn scan_boundaries(
    kernels: &mut KernelSet,
    options: &BoundaryScanOptions,
    mut progress: impl FnMut(&BoundaryScanProgress) -> ControlFlow<()>,
) -> Result<BoundaryScanReport> {
    validate(options)?;
    let mut report = BoundaryScanReport {
        options: options.clone(),
        attempts: Vec::new(),
        latest: Vec::new(),
        had_prior_flags: false,
        completed_probes: 0,
        diagnostics: EvaluationDiagnostics::default(),
        stop: DiagnosticStop::Complete,
        assessment: BoundaryAssessment::NotApplicable,
    };
    let mut selected = (0..kernels.sectors().len()).collect::<Vec<_>>();
    let mut latest = BTreeMap::new();
    for (index, scale) in std::iter::once(1.0)
        .chain(options.retry_scales.iter().copied())
        .enumerate()
    {
        if index > 0 && selected.is_empty() {
            break;
        }
        let remaining = options.sampling.max_probes - report.completed_probes;
        if remaining == 0 {
            report.stop = DiagnosticStop::ProbeBudget;
            break;
        }
        let sampling = crate::diagnostics::BoundaryOptions {
            max_probes: remaining,
            ..options.sampling.clone()
        };
        let samples = boundaries_selected(kernels, &sampling, &selected, scale, |event| {
            progress(&BoundaryScanProgress::Sampling {
                attempt: index,
                scale,
                progress: event.clone(),
            })
        })?;
        report.completed_probes += samples.probes.len();
        report.diagnostics.merge(&samples.diagnostics)?;
        let growth = analyze_boundary_growth(&samples, &options.growth)?;
        for sector in &growth.sectors {
            latest.insert(
                sector.sector,
                BoundaryLatestAssessment {
                    attempt: index,
                    sector: sector.clone(),
                },
            );
        }
        selected = growth
            .sectors
            .iter()
            .filter(|sector| sector.assessment.needs_retry())
            .map(|sector| sector.sector)
            .collect();
        let stopped = samples.stop != DiagnosticStop::Complete;
        report.stop = samples.stop;
        // Once a caller cancels sampling, do not invoke it again during unwind.
        let assessment_cancelled = samples.stop != DiagnosticStop::Cancelled
            && progress(&BoundaryScanProgress::Assessed {
                attempt: index,
                scale,
                report: growth.clone(),
            })
            .is_break();
        report.attempts.push(BoundaryScanAttempt {
            index,
            scale,
            selected_sectors: samples.sectors.iter().map(|sector| sector.sector).collect(),
            samples,
            growth,
        });
        if assessment_cancelled {
            report.stop = DiagnosticStop::Cancelled;
        }
        if stopped || assessment_cancelled {
            break;
        }
    }
    report.latest = latest.into_values().collect();
    report.assessment = aggregate(report.latest.iter().map(|value| value.sector.assessment));
    report.had_prior_flags = report
        .attempts
        .iter()
        .take(report.attempts.len().saturating_sub(1))
        .any(|attempt| attempt.growth.assessment.needs_retry());
    Ok(report)
}
