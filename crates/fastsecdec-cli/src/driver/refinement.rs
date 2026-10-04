use crate::{CliResult, config::IntegrationInput};

// Grow within the selected native catalogue, then add independent shifts.
pub(super) fn qmc_design(settings: &IntegrationInput, round: usize) -> CliResult<(u64, u32)> {
    settings.qmc_settings()?;
    let maximum = settings.published_lattice()?.max_points();
    let (mut points, mut shifts) = (settings.points, settings.shifts);
    for _ in 0..round {
        if points < maximum {
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
