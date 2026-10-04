use std::{hint::black_box, ops::ControlFlow, time::Instant};

use super::{
    BenchmarkMeasurement, BenchmarkOptions, BenchmarkReport, DiagnosticProgress, DiagnosticStop,
    KernelTiming, Result,
};
use crate::{kernel::KernelSet, status::EvaluationDiagnostics};

/// Benchmark native kernels, polling after at most `batch_size` evaluations.
/// Completed repetitions and a labelled incomplete repetition survive cancellation.
pub fn benchmark(
    kernels: &mut KernelSet,
    options: &BenchmarkOptions,
    mut progress: impl FnMut(&DiagnosticProgress) -> ControlFlow<()>,
) -> Result<BenchmarkReport> {
    options.validate()?;
    let mut report = BenchmarkReport {
        options: options.clone(),
        sectors: Vec::new(),
        planned_sectors: kernels.sectors().len(),
        stop: DiagnosticStop::Complete,
    };
    'sectors: for (sector, kernel) in kernels.sectors_mut().iter_mut().enumerate() {
        let mut row = KernelTiming {
            sector,
            dimension: kernel.dimension(),
            outputs: kernel.output_count(),
            warmup_evaluations: 0,
            measurements: Vec::new(),
            median_seconds: None,
            evaluations_per_second: None,
            diagnostics: EvaluationDiagnostics::default(),
            error: None,
        };
        let mut point = vec![0.37; kernel.dimension()];
        let mut output = vec![0.0; kernel.output_count()];
        for repetition in std::iter::once(None).chain((0..options.repetitions).map(Some)) {
            let planned = if repetition.is_some() {
                options.points
            } else {
                options.warmup
            };
            let mut completed = 0;
            let mut seconds = 0.0;
            loop {
                let event = DiagnosticProgress::Benchmark {
                    sector,
                    repetition,
                    completed_evaluations: completed,
                    planned_evaluations: planned,
                    measurement: None,
                };
                if progress(&event).is_break() {
                    report.stop = DiagnosticStop::Cancelled;
                    break;
                }
                if completed == planned {
                    break;
                }
                let end = completed.saturating_add(options.batch_size).min(planned);
                let begin = completed;
                let start = Instant::now();
                for i in begin..end {
                    for (axis, x) in point.iter_mut().enumerate() {
                        *x = (((i as u64 + 1).wrapping_mul(104729 + 2 * axis as u64) % 1000003)
                            as f64
                            + 0.5)
                            / 1000003.0;
                    }
                    match kernel
                        .evaluate_with_diagnostics(black_box(&point), black_box(&mut output))
                    {
                        Ok(diagnostics) => row.diagnostics.record(diagnostics)?,
                        Err(error) => {
                            row.diagnostics.record_failure()?;
                            row.error = Some(error.to_string());
                            report.stop = DiagnosticStop::EvaluationFailure;
                            break;
                        }
                    }
                    black_box(&output);
                    completed += 1;
                }
                seconds += start.elapsed().as_secs_f64();
                if row.error.is_some() {
                    break;
                }
            }
            if let Some(repetition) = repetition {
                let measurement = BenchmarkMeasurement {
                    repetition,
                    evaluations: completed,
                    seconds,
                    complete: completed == planned && row.error.is_none(),
                    error: row.error.clone(),
                };
                row.measurements.push(measurement.clone());
                if progress(&DiagnosticProgress::Benchmark {
                    sector,
                    repetition: Some(repetition),
                    completed_evaluations: completed,
                    planned_evaluations: planned,
                    measurement: Some(measurement),
                })
                .is_break()
                    && report.stop == DiagnosticStop::Complete
                {
                    report.stop = DiagnosticStop::Cancelled;
                }
            } else {
                row.warmup_evaluations = completed;
            }
            if report.stop != DiagnosticStop::Complete {
                break;
            }
        }
        let mut complete = row
            .measurements
            .iter()
            .filter(|sample| sample.complete)
            .map(|sample| sample.seconds)
            .collect::<Vec<_>>();
        complete.sort_by(f64::total_cmp);
        if !complete.is_empty() {
            let median = complete[complete.len() / 2];
            row.median_seconds = Some(median);
            row.evaluations_per_second = (median > 0.0).then(|| options.points as f64 / median);
        }
        report.sectors.push(row);
        if report.stop != DiagnosticStop::Complete {
            break 'sectors;
        }
    }
    Ok(report)
}
