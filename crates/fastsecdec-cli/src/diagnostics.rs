use crate::CliResult;
use fastsecdec::kernel::KernelSet;
use serde::Serialize;
use std::{hint::black_box, time::Instant};

#[derive(Serialize)]
pub struct KernelTiming {
    pub sector: usize,
    pub dimension: usize,
    pub outputs: usize,
    pub evaluations: usize,
    pub median_seconds: f64,
    pub evaluations_per_second: f64,
}

pub fn benchmark(
    kernels: &mut KernelSet,
    points: usize,
    repetitions: usize,
) -> CliResult<Vec<KernelTiming>> {
    if points == 0 || repetitions == 0 {
        return Err("benchmark points and repetitions must be positive".into());
    }
    let mut results = Vec::new();
    for (sector, kernel) in kernels.sectors_mut().iter_mut().enumerate() {
        let mut point = vec![0.37; kernel.dimension()];
        let mut output = vec![0.0; kernel.output_count()];
        for _ in 0..16 {
            kernel.evaluate(&point, &mut output)?;
        }
        let mut times = Vec::new();
        for _ in 0..repetitions {
            let start = Instant::now();
            for i in 0..points {
                for (axis, x) in point.iter_mut().enumerate() {
                    *x = (((i as u64 + 1).wrapping_mul(104729 + 2 * axis as u64) % 1000003) as f64
                        + 0.5)
                        / 1000003.0;
                }
                kernel.evaluate(black_box(&point), black_box(&mut output))?;
                black_box(&output);
            }
            times.push(start.elapsed().as_secs_f64());
        }
        times.sort_by(f64::total_cmp);
        let median = times[times.len() / 2];
        results.push(KernelTiming {
            sector,
            dimension: kernel.dimension(),
            outputs: kernel.output_count(),
            evaluations: points,
            median_seconds: median,
            evaluations_per_second: points as f64 / median,
        });
    }
    Ok(results)
}

#[derive(Serialize)]
pub struct BoundaryProbe {
    pub sector: usize,
    pub axes: Vec<usize>,
    pub approach: String,
    pub distance: f64,
    pub finite: bool,
    pub conditioning_checked: bool,
    pub rescued: bool,
    pub precision_bits: Option<u32>,
    pub max_absolute_value: Option<f64>,
    pub error: Option<String>,
}

pub fn boundaries(kernels: &mut KernelSet, exponents: &[i32]) -> Vec<BoundaryProbe> {
    let mut results = Vec::new();
    for (sector, kernel) in kernels.sectors_mut().iter_mut().enumerate() {
        let dimension = kernel.dimension();
        let mut output = vec![0.0; kernel.output_count()];
        let mut approaches = Vec::new();
        for axis in 0..dimension {
            approaches.push((format!("axis {axis} lower"), vec![(axis, false)]));
            approaches.push((format!("axis {axis} upper"), vec![(axis, true)]));
        }
        if dimension > 1 {
            for (name, pattern) in [
                ("all lower", 0),
                ("all upper", 1),
                ("alternating lower", 2),
                ("alternating upper", 3),
            ] {
                approaches.push((
                    name.into(),
                    (0..dimension)
                        .map(|axis| {
                            (
                                axis,
                                match pattern {
                                    0 => false,
                                    1 => true,
                                    2 => axis % 2 == 1,
                                    _ => axis % 2 == 0,
                                },
                            )
                        })
                        .collect(),
                ));
            }
        }
        for (approach, axes) in approaches {
            for exponent in exponents {
                let distance = 10f64.powi(-exponent);
                let mut point = vec![0.371; dimension];
                for &(axis, upper) in &axes {
                    point[axis] = if upper { 1.0 - distance } else { distance };
                }
                let result = kernel.evaluate_with_diagnostics(&point, &mut output);
                results.push(BoundaryProbe {
                    sector,
                    axes: axes.iter().map(|(axis, _)| *axis).collect(),
                    approach: approach.clone(),
                    distance,
                    finite: result.is_ok(),
                    conditioning_checked: result.as_ref().is_ok_and(|report| report.checked),
                    rescued: result.as_ref().is_ok_and(|report| report.rescued),
                    precision_bits: result.as_ref().ok().map(|report| report.bits),
                    max_absolute_value: result
                        .as_ref()
                        .ok()
                        .map(|_| output.iter().map(|x| x.abs()).fold(0.0, f64::max)),
                    error: result.err().map(|error| error.to_string()),
                });
            }
        }
    }
    results
}
