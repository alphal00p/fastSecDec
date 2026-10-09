//! Caller-owned matched-coordinate smoke test using two fixed strengths.
//! Dynamic production is not yet available; this makes no dynamic speed claim.
use std::{collections::BTreeMap, error::Error, ops::ControlFlow, time::Instant};

use fastsecdec::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{self, GenerationOptions},
    integration::{
        ContributionReport, QmcDesign, QmcReturn, QmcSession, QmcSettings, RuleSource,
        VectorEstimate,
    },
    kernel::{
        CompilationSettings, ContourValidationReport, EvaluationTimings, EvaluatorBackend,
        KernelSet,
    },
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::{KernelResultManifest, ResultScope},
    status::CoefficientComponent,
};
use serde::Serialize;
use symbolica::{atom::Atom, parse, symbol};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq, Serialize)]
struct CoordinateRange {
    sector: u64,
    shift: u64,
    start: u64,
    count: u64,
    dimension: usize,
    digest: String,
}

struct RangeHasher {
    range: CoordinateRange,
    hash: blake3::Hasher,
}
impl RangeHasher {
    fn new(sector: u64, shift: u64, start: u64, dimension: usize) -> Self {
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-contour-coordinate-audit-v1");
        for value in [sector, shift, start, dimension as u64] {
            hash.update(&value.to_le_bytes());
        }
        Self {
            range: CoordinateRange {
                sector,
                shift,
                start,
                count: 0,
                dimension,
                digest: String::new(),
            },
            hash,
        }
    }
    fn push(&mut self, index: u64, point: &[f64], weight: f64) {
        self.hash.update(&index.to_le_bytes());
        for coordinate in point.iter().chain(std::iter::once(&weight)) {
            self.hash.update(&coordinate.to_bits().to_le_bytes());
        }
        self.range.count += 1;
    }
    fn finish(mut self) -> CoordinateRange {
        self.hash.update(&self.range.count.to_le_bytes());
        self.range.digest = self.hash.finalize().to_hex().to_string();
        self.range
    }
}

#[derive(Serialize)]
struct PrescriptionRun {
    lambda: f64,
    content_id: String,
    validation: ContourValidationReport,
    design: QmcDesign,
    estimate: VectorEstimate,
    contributions: ContributionReport,
    evaluator_timings: Vec<(u64, EvaluationTimings)>,
    preparation_seconds: f64,
    production_seconds: f64,
    coordinate_audit_seconds: f64,
    foreign_return_rejected: Option<bool>,
    coordinates: Vec<CoordinateRange>,
}

fn artifact() -> Result<Vec<u8>> {
    // (1/eps) integral_0^1 [1-5*x*(1-x)-i0]^(-eps) dx.
    // The complete finite coefficient has a known complex threshold value.
    let input = ParametricIntegrand::new(
        vec![symbol!("contour_variance::x")],
        symbol!("contour_variance::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/contour_variance::eps"),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("1-5*contour_variance::x*(1-contour_variance::x)"),
                    parse!("-contour_variance::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )?;
    let generated = generation::generate(
        &input,
        &GenerationOptions {
            program_recipe: fastsecdec::kernel::indexed::ProgramRecipe::FixedV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )?;
    let kernels = generated.compile_with_settings_parameters_and_progress(
        Default::default(),
        &[],
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )?;
    Ok(kernels.to_bytes()?)
}

fn run(
    bytes: &[u8],
    lambda: f64,
    settings: QmcSettings,
    foreign_return: Option<QmcReturn>,
) -> Result<(PrescriptionRun, QmcReturn)> {
    let started = Instant::now();
    let mut kernels = KernelSet::from_bytes(bytes)?;
    kernels.bind_parameters_with_contour(
        &BTreeMap::new(),
        &ContourSettings {
            deformation: ContourMode::Fixed { lambda },
            validation: ContourValidationOptions {
                policy: ContourValidation::Pilot,
                pilot_points: 9,
            },
        },
    )?;
    for chart in kernels.contour_validation_charts() {
        // An explicit deterministic preflight for this one-dimensional control;
        // neither these coordinates nor their observations enter production.
        for index in 0..9 {
            kernels.validate_contour_point(chart.chart_index, &[index as f64 / 8.0], true)?;
        }
    }
    let validation = kernels.finish_contour_pilot()?;
    let manifest = KernelResultManifest::from_kernels(&kernels);
    let problem = manifest.integration_problem(&ResultScope::FullIntegral, kernels.content_id())?;
    let content_id = problem.content_id.clone();
    let mut session = QmcSession::democratic(problem, settings.clone())?;
    let foreign_return_rejected = foreign_return.map(|result| session.submit(result).is_err());
    if foreign_return_rejected == Some(false) {
        return Err("a return crossed mathematical integration identities".into());
    }
    let mut contexts = (0..kernels.sectors().len())
        .map(|sector| {
            Ok((
                sector as u64,
                kernels.evaluation_context(sector, Default::default())?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    drop(kernels);
    let preparation_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let mut coordinate_audit_seconds = 0.0;
    let mut coordinates = Vec::new();
    let mut first_return = None;
    let mut pending = Vec::new();
    while let Some(task) = session.next_work()? {
        let sector = task.sector_id();
        let mut worker = session.worker_context(sector)?;
        let dimension = worker.plan().dimension();
        let mut index = task.work().start();
        let mut ranges = Vec::<RangeHasher>::new();
        let context = contexts.get_mut(&sector).ok_or("missing native sector")?;
        let result = worker.evaluate_weighted_batch(task, 64, |points, weights, output| {
            let audit_started = Instant::now();
            for (point, weight) in points.chunks_exact(dimension).zip(weights) {
                let shift = index / settings.points;
                let local = index % settings.points;
                if ranges.last().is_none_or(|last| last.range.shift != shift) {
                    ranges.push(RangeHasher::new(sector, shift, local, dimension));
                }
                ranges
                    .last_mut()
                    .expect("range just allocated")
                    .push(local, point, *weight);
                index += 1;
            }
            coordinate_audit_seconds += audit_started.elapsed().as_secs_f64();
            context
                .evaluate_weighted_batch(points, weights, output)
                .map(|_| ())
        })?;
        if first_return.is_none() {
            first_return = Some(result.clone());
        }
        coordinates.extend(ranges.into_iter().map(RangeHasher::finish));
        pending.push(result);
        // Deliberately exercise reordered returns with bounded outstanding work.
        if pending.len() == 2 {
            while let Some(result) = pending.pop() {
                session.submit(result)?;
            }
        }
    }
    for result in pending {
        session.submit(result)?;
    }
    let production_seconds = started.elapsed().as_secs_f64();
    if !session.is_complete() {
        return Err("comparison requires a complete native production allocation".into());
    }
    coordinates.sort_by_key(|range| (range.sector, range.shift, range.start));
    let estimate = session.estimate()?;
    estimate.validate()?;
    Ok((
        PrescriptionRun {
            lambda,
            content_id,
            validation,
            design: session.design(),
            estimate,
            contributions: session.contributions()?,
            evaluator_timings: contexts
                .iter()
                .map(|(id, context)| (*id, context.evaluation_metrics()))
                .collect(),
            preparation_seconds,
            production_seconds,
            coordinate_audit_seconds,
            foreign_return_rejected,
            coordinates,
        },
        first_return.ok_or("control unexpectedly has no stochastic sector")?,
    ))
}

fn target_variance(estimate: &VectorEstimate) -> f64 {
    estimate
        .orders
        .iter()
        .enumerate()
        .filter(|(_, order)| **order == 0)
        .map(|(index, _)| estimate.covariance_of_mean[index * estimate.orders.len() + index])
        .sum()
}

fn check_reference(estimate: &VectorEstimate) -> Result<()> {
    let beta = 0.2_f64.sqrt();
    for (index, (order, component)) in estimate.orders.iter().zip(&estimate.components).enumerate()
    {
        let expected = match (*order, component) {
            (-1, CoefficientComponent::Real) => 1.0,
            (-1, CoefficientComponent::Imag) => 0.0,
            (0, CoefficientComponent::Real) => 2.0 - beta * ((1.0 + beta) / (1.0 - beta)).ln(),
            (0, CoefficientComponent::Imag) => std::f64::consts::PI * beta,
            _ => return Err("unexpected analytic control coefficient".into()),
        };
        if (estimate.mean[index] - expected).abs()
            > (8.0 * estimate.standard_error[index]).max(2e-7)
        {
            return Err(format!("analytic control disagrees at eps^{order} {component:?}").into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let bytes = artifact()?;
    let mut pairs = Vec::new();
    let mut previous_coordinate_digest = None;
    for seed in [34723, 92711] {
        let settings = QmcSettings {
            points: 64,
            shifts: 8,
            package_points: 19,
            seed,
            // In one dimension the native vector [1] visits all lattice points;
            // published multidimensional catalogues start at larger counts.
            rule: RuleSource::Supplied(vec![1]),
            ..Default::default()
        };
        let (first, first_return) = run(&bytes, 0.05, settings.clone(), None)?;
        let (second, _) = run(&bytes, 0.25, settings, Some(first_return))?;
        check_reference(&first.estimate)?;
        check_reference(&second.estimate)?;
        if first.content_id == second.content_id || first.coordinates != second.coordinates {
            return Err(
                "distinct mathematical identities did not retain matching actual coordinates"
                    .into(),
            );
        }
        let first_variance = target_variance(&first.estimate);
        let second_variance = target_variance(&second.estimate);
        if first_variance <= 0.0 || second_variance <= 0.0 {
            return Err("smoke control should resolve nonzero complete-replica covariance".into());
        }
        let digest = first
            .coordinates
            .first()
            .ok_or("missing coordinate audit")?
            .digest
            .clone();
        if previous_coordinate_digest.as_ref() == Some(&digest) {
            return Err("independent seed pair repeated the previous coordinate sequence".into());
        }
        previous_coordinate_digest = Some(digest);
        pairs.push(serde_json::json!({
            "seed": seed, "actual_coordinates_and_weights_match": true,
            "first_target_variance": first_variance, "second_target_variance": second_variance,
            "first_wall_seconds_times_variance": first.production_seconds * first_variance,
            "second_wall_seconds_times_variance": second.production_seconds * second_variance,
            "first": first, "second": second,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "fixed-strength API smoke probe; no dynamic improvement claim",
            "backend": "eager", "production_workers": 1, "pairs": pairs,
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinate_digest_observes_coordinates_weights_and_ranges() {
        let digest = |index, point, weight| {
            let mut hash = RangeHasher::new(2, 3, index, 1);
            hash.push(index, &[point], weight);
            hash.finish().digest
        };
        let original = digest(7, 0.25, 0.5);
        assert_ne!(original, digest(7, 0.5, 0.5));
        assert_ne!(original, digest(7, 0.25, 1.0));
        assert_ne!(original, digest(8, 0.25, 0.5));
    }
}
