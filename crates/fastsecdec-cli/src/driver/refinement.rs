use crate::{CliResult, config::IntegrationInput};

// The bundled lattice stops at 2^20 points; beyond that size, add independent
// shifts without requesting an unavailable rule.
pub(super) fn qmc_design(points: u64, shifts: u32, round: usize) -> CliResult<(u64, u32)> {
    const MAX_POINTS: u64 = 1 << 20;
    if !(1024..=MAX_POINTS).contains(&points) || !points.is_power_of_two() {
        return Err("bundled QMC points must be a power of two in 1024..=2^20".into());
    }
    let (mut points, mut shifts) = (points, shifts);
    for _ in 0..round {
        if points < MAX_POINTS {
            points *= 2;
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
    let (_, shifts) = qmc_design(settings.points, settings.shifts, round)?;
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
