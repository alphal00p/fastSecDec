use crate::{CliResult, config::IntegrationInput};
use fastsecdec::integration::{IntegrationObservation, IntegrationProblem, QmcDesign};

/// A completed earlier statistical design retained separately from current
/// sampling. It is never pooled with the replacement epoch or used to stop it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PreviousProduction {
    pub round: usize,
    pub observation: IntegrationObservation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qmc_design: Option<QmcDesign>,
}
impl PreviousProduction {
    pub(super) fn validate(
        &self,
        problem: &IntegrationProblem,
        active_round: usize,
    ) -> CliResult<()> {
        let estimate = self
            .observation
            .snapshot
            .estimate
            .as_ref()
            .ok_or("previous production has no estimate")?;
        let n = problem.orders.len();
        if self.round >= active_round
            || !estimate.production_complete
            || self.observation.snapshot.stage != fastsecdec::status::IntegrationStage::Production
            || estimate.orders != problem.orders
            || estimate.components != problem.components
            || estimate.mean.len() != n
            || estimate.standard_error.len() != n
            || estimate.covariance_of_mean.len() != n * n
            || estimate
                .mean
                .iter()
                .chain(&estimate.standard_error)
                .chain(&estimate.covariance_of_mean)
                .any(|v| !v.is_finite())
            || estimate.standard_error.iter().any(|v| *v < 0.)
            || self
                .observation
                .snapshot
                .sectors
                .iter()
                .map(|s| s.id)
                .ne(problem.sectors.iter().map(|s| s.id))
            || self.observation.contributions.total.as_ref() != Some(estimate)
        {
            return Err(
                "previous completed production does not match the resumed problem or epoch".into(),
            );
        }
        Ok(())
    }
}

// Grow within the selected native catalogue, then add independent shifts.
pub(super) fn qmc_design(settings: &IntegrationInput, round: usize) -> CliResult<(u64, u32)> {
    settings.qmc_settings()?;
    let maximum = settings.published_lattice()?.max_points();
    let (mut points, mut shifts) = (settings.points, settings.shifts);
    for _ in 0..round {
        if settings.double_points && points < maximum {
            points = points
                .checked_mul(2)
                .ok_or("lattice point growth overflow")?;
        } else {
            shifts = shifts
                .checked_mul(2)
                .ok_or("independent shift growth overflow")?;
        }
    }
    Ok((points, shifts))
}

pub(super) fn mc_design(settings: &IntegrationInput, round: usize) -> CliResult<(u64, u32)> {
    if settings.double_points {
        Ok((mc_points(settings.points, round)?, settings.shifts))
    } else {
        let factor = 1u32
            .checked_shl(round.try_into()?)
            .ok_or("MC batch target overflow")?;
        let batches = settings
            .shifts
            .checked_mul(factor)
            .ok_or("MC batch target overflow")?;
        Ok((settings.points, batches))
    }
}

pub(super) fn mc_points(points: u64, round: usize) -> CliResult<u64> {
    points
        .checked_mul(
            1u64.checked_shl(round.try_into()?)
                .ok_or("MC growth overflow")?,
        )
        .ok_or_else(|| "MC growth overflow".into())
}

pub(super) fn adaptive_budget(settings: &IntegrationInput, round: usize) -> CliResult<(f64, u32)> {
    let (_, shifts) = qmc_design(settings, round)?;
    let multiplier = shifts
        .checked_div(settings.shifts)
        .ok_or("at least two shifts are required")?;
    let seconds = settings.production_seconds * f64::from(multiplier);
    let minimum_shifts = 2u32
        .checked_mul(multiplier)
        .ok_or("production shift growth overflow")?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("adaptive production budget must be finite and positive".into());
    }
    Ok((seconds, minimum_shifts))
}
