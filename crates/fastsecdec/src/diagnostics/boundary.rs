use std::ops::ControlFlow;

use super::{
    AxisEndpoint, BoundaryCoverage, BoundaryOptions, BoundaryProbe, BoundaryReport, BoundarySector,
    BoundarySide, DiagnosticError, DiagnosticProgress, DiagnosticStop, Result,
};
use crate::{kernel::KernelSet, status::EvaluationDiagnostics};

// Count C(d,k)*2^k without constructing any point or assuming a machine-word
// bit mask can hold a high-dimensional face. Overflow is explicit metadata.
pub(super) fn face_count(dimension: usize, codimension: usize) -> Option<u64> {
    let mut count = 0u128;
    let mut binomial = 1u128;
    let mut sides = 1u128;
    for k in 1..=dimension.min(codimension) {
        binomial = binomial.checked_mul((dimension - k + 1) as u128)? / k as u128;
        sides = sides.checked_mul(2)?;
        count = count.checked_add(binomial.checked_mul(sides)?)?;
        if count > u64::MAX as u128 {
            return None;
        }
    }
    Some(count as u64)
}

/// Enumerate bounded face-pattern samples and stream each completed result.
/// The callback is first called before evaluating any point, allowing immediate
/// cancellation. Evaluation failures are reported as rows and do not erase data.
pub fn boundaries(
    kernels: &mut KernelSet,
    options: &BoundaryOptions,
    progress: impl FnMut(&DiagnosticProgress) -> ControlFlow<()>,
) -> Result<BoundaryReport> {
    let selected = (0..kernels.sectors().len()).collect::<Vec<_>>();
    boundaries_selected(kernels, options, &selected, 1.0, progress)
}

pub(super) fn scaled_distances(options: &BoundaryOptions, scale: f64) -> Result<Vec<f64>> {
    if !scale.is_finite() || scale <= 0.0 || scale > 1.0 {
        return Err(DiagnosticError::Invalid(
            "boundary distance scale must lie in (0,1]",
        ));
    }
    options.distances()?.into_iter().map(|distance| {
        let distance = distance * scale;
        let upper = 1.0 - distance;
        if !(0.0 < distance && distance < 1.0 && 0.0 < upper && upper < 1.0) {
            return Err(DiagnosticError::Invalid("scaled boundary distances must remain representable strictly inside both ends of the cube"));
        }
        Ok(distance)
    }).collect()
}

pub(super) fn boundaries_selected(
    kernels: &mut KernelSet,
    options: &BoundaryOptions,
    selected: &[usize],
    scale: f64,
    mut progress: impl FnMut(&DiagnosticProgress) -> ControlFlow<()>,
) -> Result<BoundaryReport> {
    let distances = scaled_distances(options, scale)?;
    let mut seen = std::collections::BTreeSet::new();
    if selected
        .iter()
        .any(|&index| index >= kernels.sectors().len() || !seen.insert(index))
    {
        return Err(DiagnosticError::Invalid(
            "selected boundary sectors are invalid or duplicated",
        ));
    }
    let sectors = selected
        .iter()
        .map(|&sector| BoundarySector {
            sector,
            dimension: kernels.sectors()[sector].dimension(),
        })
        .collect::<Vec<_>>();
    let count = |all: bool| {
        sectors.iter().try_fold(0u64, |sum, kernel| {
            let count = face_count(
                kernel.dimension,
                if all {
                    kernel.dimension
                } else {
                    options.max_codimension
                },
            )?;
            sum.checked_add(count.checked_mul(distances.len() as u64)?)
        })
    };
    let configured = count(false);
    let all = count(true);
    let planned = configured.map_or(options.max_probes, |count| {
        count.min(options.max_probes as u64) as usize
    });
    let truncated = configured.is_none_or(|count| count > options.max_probes as u64);
    let all_dimensions = sectors
        .iter()
        .all(|k| k.dimension <= options.max_codimension);
    let mut report = BoundaryReport {
        options: options.clone(),
        distance_scale: scale,
        orders: kernels.orders().to_vec(),
        components: kernels.components().to_vec(),
        sectors,
        probes: Vec::new(),
        failures: 0,
        diagnostics: EvaluationDiagnostics::default(),
        stop: DiagnosticStop::Complete,
        coverage: BoundaryCoverage {
            max_codimension: options.max_codimension,
            configured_probes: configured,
            all_face_pattern_probes: all,
            planned_probes: planned,
            completed_probes: 0,
            truncated_by_budget: truncated,
            includes_all_face_patterns: false,
        },
    };
    if progress(&DiagnosticProgress::Boundary {
        completed: 0,
        planned,
        probe: None,
    })
    .is_break()
    {
        report.stop = DiagnosticStop::Cancelled;
        return Ok(report);
    }
    'sectors: for &sector in selected {
        let kernel = &mut kernels.sectors_mut()[sector];
        let dimension = kernel.dimension();
        let mut output = vec![0.0; kernel.output_count()];
        for codimension in 1..=dimension.min(options.max_codimension) {
            let mut axes = (0..codimension).collect::<Vec<_>>();
            loop {
                let mut upper = vec![false; codimension];
                loop {
                    let endpoints = axes
                        .iter()
                        .zip(&upper)
                        .map(|(&axis, &upper)| AxisEndpoint {
                            axis,
                            side: if upper {
                                BoundarySide::Upper
                            } else {
                                BoundarySide::Lower
                            },
                        })
                        .collect::<Vec<_>>();
                    for (&distance, &exponent) in distances.iter().zip(&options.exponents) {
                        if report.probes.len() == planned {
                            report.stop = DiagnosticStop::ProbeBudget;
                            break 'sectors;
                        }
                        let mut point = vec![options.interior; dimension];
                        for endpoint in &endpoints {
                            point[endpoint.axis] = match endpoint.side {
                                BoundarySide::Lower => distance,
                                BoundarySide::Upper => 1.0 - distance,
                            };
                        }
                        let result = kernel.evaluate_with_diagnostics(&point, &mut output);
                        match &result {
                            Ok(value) => report.diagnostics.record(*value)?,
                            Err(_) => {
                                report.diagnostics.record_failure()?;
                                report.failures += 1;
                            }
                        }
                        let probe = BoundaryProbe {
                            sector,
                            endpoints: endpoints.clone(),
                            exponent,
                            distance,
                            point,
                            finite: result.is_ok(),
                            conditioning_checked: result.as_ref().is_ok_and(|r| r.checked),
                            rescued: result.as_ref().is_ok_and(|r| r.rescued),
                            precision_bits: result.as_ref().ok().map(|r| r.bits),
                            max_absolute_value: result
                                .as_ref()
                                .ok()
                                .map(|_| output.iter().map(|x| x.abs()).fold(0.0, f64::max)),
                            values: result.as_ref().ok().map(|_| output.clone()),
                            error: result.err().map(|e| e.to_string()),
                        };
                        report.probes.push(probe.clone());
                        report.coverage.completed_probes = report.probes.len();
                        if progress(&DiagnosticProgress::Boundary {
                            completed: report.probes.len(),
                            planned,
                            probe: Some(probe),
                        })
                        .is_break()
                        {
                            report.stop = DiagnosticStop::Cancelled;
                            break 'sectors;
                        }
                    }
                    // Binary side assignments on a Vec, without a dimension cap.
                    let Some(index) = upper.iter().position(|&side| !side) else {
                        break;
                    };
                    upper[..index].fill(false);
                    upper[index] = true;
                }
                let Some(index) = (0..codimension)
                    .rev()
                    .find(|&i| axes[i] < dimension - codimension + i)
                else {
                    break;
                };
                axes[index] += 1;
                for i in index + 1..codimension {
                    axes[i] = axes[i - 1] + 1;
                }
            }
        }
    }
    if report.stop == DiagnosticStop::Complete && truncated {
        report.stop = DiagnosticStop::ProbeBudget;
    }
    report.coverage.includes_all_face_patterns =
        all_dimensions && !truncated && report.stop == DiagnosticStop::Complete;
    Ok(report)
}
