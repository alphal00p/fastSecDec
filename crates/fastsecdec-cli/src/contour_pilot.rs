//! Caller-owned preflight sampling; validation never reserves production work.
use crate::CliResult;
use fastsecdec::{
    contour::{ContourSettings, ContourValidation},
    kernel::{ContourValidationReport, KernelSet},
};
use numerica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use serde::{Deserialize, Serialize};

/// Protocol identity is retained separately from the mathematical integrand.
const PROTOCOL: &str = "fastsecdec-contour-pilot-uniform-v1";

pub(crate) fn provenance(
    kernels: &KernelSet,
    seed: u64,
    report: &ContourValidationReport,
) -> fastsecdec::status::ContourPilotProvenance {
    fastsecdec::status::ContourPilotProvenance::from_report(
        PROTOCOL,
        kernels.content_id(),
        seed,
        report,
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Snapshot {
    pub protocol: String,
    pub kernel_content_id: String,
    pub seed: u64,
    pub completed: usize,
    pub total: usize,
    pub chart_index: Option<usize>,
}

/// Supplying this observer owns cancellation and presentation. No worker pool,
/// production session, adaptive grid or integration accumulator is involved.
pub(crate) fn run(
    kernels: &mut KernelSet,
    settings: &ContourSettings,
    seed: u64,
    scope: Option<&fastsecdec::results::ResultScope>,
    mut observe: impl FnMut(&Snapshot) -> CliResult<()>,
) -> CliResult<Option<ContourValidationReport>> {
    if !kernels.contour_capable() || settings.validation.policy == ContourValidation::Off {
        return Ok(None);
    }
    let charts = kernels
        .contour_validation_charts()
        .into_iter()
        .filter(|chart| scope.is_none_or(|scope| chart.required_by_scope(scope)))
        .collect::<Vec<_>>();
    let selected = charts
        .iter()
        .map(|chart| chart.chart_index)
        .collect::<Vec<_>>();
    let total = charts
        .len()
        .checked_mul(settings.validation.pilot_points)
        .ok_or("contour pilot work count overflow")?;
    let identity = kernels.content_id().to_owned();
    let mut snapshot = Snapshot {
        protocol: PROTOCOL.into(),
        kernel_content_id: identity.clone(),
        seed,
        completed: 0,
        total,
        chart_index: None,
    };
    observe(&snapshot)?;
    for chart in charts {
        snapshot.chart_index = Some(chart.chart_index);
        let mut rng = validation_rng(&identity, seed, chart.chart_index);
        let mut grid = (chart.dimension > 0)
            .then(|| ContinuousGrid::<f64>::new(chart.dimension, 1, 1, None, false))
            .transpose()?;
        let mut sample = Sample::new();
        for index in 0..settings.validation.pilot_points {
            // Include the cube centre explicitly: it can be a stationary point
            // on the negative-real causal branch, missed by random presampling.
            if let Some(grid) = grid.as_mut().filter(|_| index != 0) {
                grid.sample(&mut rng, &mut sample);
                let Sample::Continuous(_, coordinates) = &sample else {
                    return Err("native contour pilot expected a continuous sample".into());
                };
                kernels.validate_contour_point(chart.chart_index, coordinates, true)?;
            } else {
                kernels.validate_contour_point(
                    chart.chart_index,
                    &vec![0.5; chart.dimension],
                    true,
                )?;
            }
            snapshot.completed += 1;
            observe(&snapshot)?;
        }
    }
    Ok(Some(kernels.finish_contour_pilot_for_charts(&selected)?))
}

fn validation_rng(identity: &str, seed: u64, chart: usize) -> MonteCarloRng {
    let mut hash = blake3::Hasher::new();
    hash.update(PROTOCOL.as_bytes());
    hash.update(identity.as_bytes());
    hash.update(&seed.to_le_bytes());
    hash.update(&(chart as u64).to_le_bytes());
    let digest = hash.finalize();
    MonteCarloRng::new(
        u64::from_le_bytes(digest.as_bytes()[..8].try_into().unwrap()),
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_validation_sampling_replays_without_advancing_the_production_rng() {
        let production = MonteCarloRng::new(42, 0);
        let before = production.export();
        let coordinates = |chart| {
            let mut grid = ContinuousGrid::<f64>::new(3, 1, 1, None, false).unwrap();
            let mut rng = validation_rng("integral", 42, chart);
            let mut sample = Sample::new();
            (0..16)
                .map(|_| {
                    grid.sample(&mut rng, &mut sample);
                    let Sample::Continuous(_, points) = &sample else {
                        panic!()
                    };
                    points.clone()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(coordinates(0), coordinates(0));
        assert_ne!(coordinates(0), coordinates(1));
        assert_eq!(before, production.export());
        assert_ne!(before, validation_rng("integral", 42, 0).export());
    }
}
