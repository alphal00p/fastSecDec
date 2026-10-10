//! Caller-owned matched-coordinate variance comparison. The default is a small
//! fixed/fixed control. `--dynamic` selects fixed/polynomial/sign-aware recipes
//! through the public admission path; it never bypasses an unfinished gate.
use std::{collections::BTreeMap, error::Error, ops::ControlFlow, time::Instant};
#[path = "contour_variance/comparison.rs"]
mod comparison;

use fastsecdec::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{self, GenerationOptions},
    integration::{
        ContributionReport, QmcDesign, QmcReturn, QmcSession, QmcSettings, RuleSource,
        VectorEstimate,
    },
    kernel::{
        CompilationSettings, ContourProductionReport, ContourValidationReport, EvaluationTimings,
        EvaluatorBackend, KernelSet,
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

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Control {
    ThresholdBubble,
    LinearSquare,
}

impl Control {
    fn dimension(self) -> usize {
        match self {
            Self::ThresholdBubble => 1,
            Self::LinearSquare => 2,
        }
    }

    fn finite(self) -> (f64, f64) {
        match self {
            Self::ThresholdBubble => {
                let beta = 0.2_f64.sqrt();
                (
                    2.0 - beta * ((1.0 + beta) / (1.0 - beta)).ln(),
                    std::f64::consts::PI * beta,
                )
            }
            // -integral log(1-2*x-3*y-i0) dx dy on the unit square.
            Self::LinearSquare => (
                1.5 - (7.0 / 3.0) * 2.0_f64.ln(),
                11.0 * std::f64::consts::PI / 12.0,
            ),
        }
    }
}

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
    deformation: ContourMode,
    content_id: String,
    pilot: ContourValidationReport,
    production_validation: Vec<(u64, Option<ContourProductionReport>)>,
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

fn artifact(mode: ContourMode, backend: EvaluatorBackend, control: Control) -> Result<Vec<u8>> {
    // (1/eps) integral_0^1 [1-5*x*(1-x)-i0]^(-eps) dx.
    // The complete finite coefficient has a known complex threshold value.
    let mut coordinates = vec![symbol!("contour_variance::x")];
    let causal = match control {
        Control::ThresholdBubble => parse!("1-5*contour_variance::x*(1-contour_variance::x)"),
        Control::LinearSquare => {
            coordinates.push(symbol!("contour_variance::y"));
            parse!("1-2*contour_variance::x-3*contour_variance::y")
        }
    };
    let input = ParametricIntegrand::new(
        coordinates,
        symbol!("contour_variance::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/contour_variance::eps"),
            vec![Atom::Zero; control.dimension()],
            vec![
                PolynomialFactor::new(
                    causal,
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
            program_recipe: mode.program_recipe(),
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )?;
    let kernels = generated.compile_with_settings_parameters_and_progress(
        Default::default(),
        &[],
        CompilationSettings {
            backend,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )?;
    Ok(kernels.to_bytes()?)
}

fn run(
    bytes: &[u8],
    deformation: ContourMode,
    policy: ContourValidation,
    control: Control,
    settings: QmcSettings,
    foreign_return: Option<QmcReturn>,
) -> Result<(PrescriptionRun, QmcReturn)> {
    let started = Instant::now();
    let mut kernels = KernelSet::from_bytes(bytes)?;
    kernels.bind_parameters_with_contour(
        &BTreeMap::new(),
        &ContourSettings {
            deformation,
            validation: ContourValidationOptions {
                policy,
                pilot_points: 9,
            },
        },
    )?;
    for chart in kernels
        .contour_validation_charts()
        .into_iter()
        .filter(|_| policy != ContourValidation::Off)
    {
        // An explicit deterministic preflight for this analytic control;
        // neither these coordinates nor their observations enter production.
        for index in 0..9 {
            let point = match control {
                Control::ThresholdBubble => vec![index as f64 / 8.0],
                Control::LinearSquare => vec![(index % 3) as f64 / 2., (index / 3) as f64 / 2.],
            };
            if point.len() != chart.dimension {
                return Err("analytic control changed its source-coordinate schema".into());
            }
            kernels.validate_contour_point(chart.chart_index, &point, true)?;
        }
    }
    let pilot = if policy == ContourValidation::Off {
        kernels
            .contour_validation_report()
            .ok_or("missing unchecked contour report")?
    } else {
        kernels.finish_contour_pilot()?
    };
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
            deformation,
            content_id,
            pilot,
            production_validation: contexts
                .iter()
                .map(|(id, context)| (*id, context.contour_validation_report()))
                .collect(),
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

fn check_reference(estimate: &VectorEstimate, control: Control) -> Result<()> {
    let (real, imaginary) = control.finite();
    for (index, (order, component)) in estimate.orders.iter().zip(&estimate.components).enumerate()
    {
        let expected = match (*order, component) {
            (-1, CoefficientComponent::Real) => 1.0,
            (-1, CoefficientComponent::Imag) => 0.0,
            (0, CoefficientComponent::Real) => real,
            (0, CoefficientComponent::Imag) => imaginary,
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
    comparison::execute()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linear_square_reference_has_the_native_mixed_primitive() {
        use symbolica::atom::AtomCore;
        let x = symbol!("variance_reference::x");
        let y = symbol!("variance_reference::y");
        let f = parse!("1-2*variance_reference::x-3*variance_reference::y");
        let h: Atom = f.pow(2) * f.log() / 2 - Atom::num((3, 4)) * f.pow(2);
        let difference: Atom = h.derivative(x).derivative(y) - 6 * f.log();
        assert!(difference.together().cancel().is_zero());
    }

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
