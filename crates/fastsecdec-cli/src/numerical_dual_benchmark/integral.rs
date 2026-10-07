//! Paired native-QMC channels, preserving the full cross-lane covariance.
use super::*;
use fastsecdec::{
    integration::{
        IntegrationProblem, Periodization, QmcSession, QmcSettings, RuleSource, SectorSpec,
    },
    kernel::WeightedEvaluationContext,
};

struct ChartContext {
    evaluator: Option<WeightedEvaluationContext>,
    permutation: Vec<usize>,
    multiplicity: f64,
    slots: Vec<usize>,
    mapped: Vec<f64>,
    weighted: Vec<f64>,
    values: Vec<f64>,
}
impl ChartContext {
    fn new(case: &Loaded, chart: usize, layout: &[(i32, CoefficientComponent)]) -> CliResult<Self> {
        let record = &case.charts[chart];
        let id = record.kernel_sector();
        let evaluator = id
            .map(|id| case.kernels.evaluation_context(id, ReplayPolicy::default()))
            .transpose()?;
        let multiplicity = id.map_or(1, |id| {
            case.charts
                .iter()
                .filter(|chart| chart.kernel_sector() == Some(id))
                .count()
        });
        Ok(Self {
            evaluator,
            permutation: record.representative_permutation().to_vec(),
            multiplicity: multiplicity as f64,
            slots: case
                .layout
                .iter()
                .map(|key| {
                    layout
                        .iter()
                        .position(|candidate| candidate == key)
                        .unwrap()
                })
                .collect(),
            mapped: Vec::new(),
            weighted: Vec::new(),
            values: Vec::new(),
        })
    }
    fn evaluate(
        &mut self,
        points: &[f64],
        weights: &[f64],
        out: &mut [f64],
        width: usize,
        offset: usize,
    ) -> CliResult<()> {
        let Some(evaluator) = self.evaluator.as_mut() else {
            return Ok(());
        };
        let dimension = evaluator.dimension();
        self.mapped.resize(points.len(), 0.0);
        for (row, point) in points.chunks_exact(dimension).enumerate() {
            for (axis, target) in self.permutation.iter().copied().enumerate() {
                self.mapped[row * dimension + target] = point[axis];
            }
        }
        self.weighted.clear();
        self.weighted
            .extend(weights.iter().map(|weight| weight / self.multiplicity));
        self.values
            .resize(weights.len() * evaluator.output_count(), 0.0);
        evaluator
            .evaluate_weighted_batch(&self.mapped, &self.weighted, &mut self.values)
            .map_err(|failure| failure.error)?;
        for (target, values) in out
            .chunks_exact_mut(width)
            .zip(self.values.chunks_exact(evaluator.output_count()))
        {
            for (slot, value) in self.slots.iter().zip(values) {
                target[offset + slot] = *value;
            }
        }
        Ok(())
    }
}

pub(super) fn compare(
    left: &Loaded,
    right: &Loaded,
    configuration: &Configuration,
) -> CliResult<Value> {
    let layout = left
        .layout
        .iter()
        .chain(&right.layout)
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let count = layout.len();
    let width = count * 2;
    let mut exact = vec![0.0; width];
    for (offset, case) in [(0, left), (count, right)] {
        for (key, value) in case.layout.iter().zip(case.kernels.exact_coefficients()) {
            exact[offset
                + layout
                    .iter()
                    .position(|candidate| candidate == key)
                    .unwrap()] = *value;
        }
    }
    // These orders label statistical channels only. The saved mapping below
    // retains each channel's real scientific epsilon order and component.
    let orders = (0..width)
        .map(i32::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let sectors = left
        .charts
        .iter()
        .enumerate()
        .filter(|(_, chart)| !chart.coordinates().target_parameters().is_empty())
        .map(|(id, chart)| SectorSpec {
            id: id as u64,
            dimension: chart.coordinates().target_parameters().len(),
        })
        .collect();
    let problem = IntegrationProblem::new(
        format!(
            "paired:{}:{}",
            left.kernels.content_id(),
            right.kernels.content_id()
        ),
        orders,
        sectors,
        exact,
    )?;
    let settings = QmcSettings {
        points: configuration.integral_points,
        shifts: configuration.integral_shifts,
        seed: configuration.seed,
        package_points: configuration.integral_points,
        periodization: Periodization::Korobov3,
        rule: RuleSource::Kuo,
    };
    let mut session = QmcSession::democratic(problem, settings.clone())?;
    let mut contexts = (0..left.charts.len())
        .map(|chart| {
            Ok((
                ChartContext::new(left, chart, &layout)?,
                ChartContext::new(right, chart, &layout)?,
            ))
        })
        .collect::<CliResult<Vec<_>>>()?;
    let started = Instant::now();
    while let Some(task) = session.next_work()? {
        let (a, b) = &mut contexts[task.sector_id() as usize];
        let returned = session
            .worker_context(task.sector_id())?
            .evaluate_weighted_batch(
                task,
                configuration.batch_rows,
                |points, weights, out| -> CliResult<()> {
                    out.fill(0.0);
                    a.evaluate(points, weights, out, width, 0)?;
                    b.evaluate(points, weights, out, width, count)
                },
            )?;
        session.submit(returned)?;
    }
    let estimate = session.estimate()?;
    if !estimate.production_complete {
        return Err("paired QMC allocation did not complete".into());
    }
    let mut differences = Vec::new();
    let mut agreement = true;
    for (index, key) in layout.iter().enumerate() {
        let a = index;
        let b = count + index;
        let variance = estimate.covariance_of_mean[a * width + a]
            + estimate.covariance_of_mean[b * width + b]
            - 2.0 * estimate.covariance_of_mean[a * width + b];
        let scale = estimate.covariance_of_mean[a * width + a].abs()
            + estimate.covariance_of_mean[b * width + b].abs()
            + 2.0 * estimate.covariance_of_mean[a * width + b].abs();
        if !variance.is_finite() || variance < -16.0 * f64::EPSILON * scale {
            return Err(
                "paired difference covariance exceeded floating-point cancellation allowance"
                    .into(),
            );
        }
        let error = variance.max(0.0).sqrt();
        let difference = estimate.mean[a] - estimate.mean[b];
        let tolerance = configuration.absolute_tolerance
            + configuration.relative_tolerance * estimate.mean[a].abs().max(estimate.mean[b].abs());
        let consistent = difference.abs() <= tolerance + 6.0 * error;
        agreement &= consistent;
        differences.push(json!({"layout":key,"reference":estimate.mean[a],"candidate":estimate.mean[b],
            "difference":difference,"standard_error_of_paired_difference":error,"numeric_tolerance":tolerance,
            "within_six_standard_errors":consistent}));
    }
    Ok(
        json!({"settings":settings,"wall_seconds":started.elapsed().as_secs_f64(),
        "channel_layout":{"reference":layout,"candidate":layout},"native_full_vector_estimate":estimate,
        "differences":differences,"agreement":agreement,
        "meaning":"finite-statistics consistency control, not an exact proof or an accuracy claim"}),
    )
}
