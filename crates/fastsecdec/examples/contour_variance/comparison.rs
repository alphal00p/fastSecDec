//! Reporting over native complete-replica statistics; no separate estimator.
use super::*;
use fastsecdec::contour::DynamicConstruction;
use std::collections::BTreeSet;

const SEEDS: [u64; 8] = [34723, 92711, 125231, 169087, 203449, 250007, 301747, 351043];

fn target_variance(estimate: &VectorEstimate) -> Result<f64> {
    estimate.validate()?;
    let last = estimate.orders.iter().max().ok_or("empty Laurent vector")?;
    Ok(estimate
        .orders
        .iter()
        .enumerate()
        .filter(|(_, order)| *order == last)
        .map(|(index, _)| estimate.covariance_of_mean[index * estimate.orders.len() + index])
        .sum())
}

fn ratio(first: f64, second: f64) -> Option<f64> {
    let value = first / second;
    (first.is_finite()
        && first > 0.0
        && second.is_finite()
        && second > 0.0
        && value.is_finite()
        && value > 0.0)
        .then_some(value)
}

fn compare(first: &PrescriptionRun, second: &PrescriptionRun) -> Result<serde_json::Value> {
    if first.content_id == second.content_id || first.coordinates != second.coordinates {
        return Err(
            "distinct mathematical identities must retain matching actual coordinates and weights"
                .into(),
        );
    }
    if first.estimate.orders != second.estimate.orders
        || first.estimate.components != second.estimate.components
        || first.design != second.design
    {
        return Err(
            "matched-work comparison requires identical Laurent layouts and allocations".into(),
        );
    }
    let first_variance = target_variance(&first.estimate)?;
    let second_variance = target_variance(&second.estimate)?;
    Ok(serde_json::json!({
        "first": first.deformation, "second": second.deformation,
        "actual_coordinates_and_weights_match": true,
        "first_target_variance": first_variance,
        "second_target_variance": second_variance,
        "variance_gain": ratio(first_variance, second_variance),
        "elapsed_time_normalized_gain": ratio(
            first.production_seconds * first_variance,
            second.production_seconds * second_variance),
        "timing_includes_coordinate_audit": true,
        "undefined_ratio_is_null": true,
    }))
}

pub(super) fn execute() -> Result<()> {
    let mut dynamic = false;
    let mut smoke = false;
    let mut control = Control::ThresholdBubble;
    let mut backend = EvaluatorBackend::Eager;
    let mut validation = ContourValidation::Pilot;
    let mut diagnostics = ContourDiagnosticsMode::Disabled;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--dynamic" => dynamic = true,
            "--smoke" => smoke = true,
            "--linear-square" => control = Control::LinearSquare,
            "--cubic-cube" => control = Control::CubicCube,
            "--symjit" => backend = EvaluatorBackend::Symjit,
            "--diagnostics" => diagnostics = ContourDiagnosticsMode::Aggregate,
            "--validation=always" => validation = ContourValidation::Always,
            "--validation=pilot" => validation = ContourValidation::Pilot,
            "--validation=off" => validation = ContourValidation::Off,
            _ => return Err(format!("unknown argument {argument}; use --dynamic, --smoke, --linear-square, --cubic-cube, --symjit, --diagnostics, or --validation=always|pilot|off").into()),
        }
    }
    // Retain both original fixed controls: comparing only the weaker strength
    // would hide whether a better fixed contour already beats the dynamic one.
    let mut modes = vec![
        ContourMode::Fixed { lambda: 0.05 },
        ContourMode::Fixed { lambda: 0.25 },
    ];
    if dynamic {
        for construction in [
            DynamicConstruction::Polynomial,
            DynamicConstruction::SignAware,
        ] {
            modes.push(ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: 1.0,
                displacement_cap: 1.0,
                construction,
            });
        }
    }
    let mut artifacts = BTreeMap::new();
    let mut generation = Vec::new();
    for mode in &modes {
        let recipe = mode.program_recipe();
        if artifacts.contains_key(&recipe) {
            continue;
        }
        let started = Instant::now();
        let bytes = artifact(*mode, backend, control)?;
        generation.push(serde_json::json!({
            "recipe": recipe, "seconds": started.elapsed().as_secs_f64(),
            "artifact_bytes": bytes.len(),
        }));
        artifacts.insert(recipe, bytes);
    }
    let mut pairs = Vec::new();
    let mut coordinate_digests = BTreeSet::new();
    let full_comparison = dynamic && !smoke;
    for (pair_index, seed) in SEEDS[..if full_comparison { 8 } else { 2 }]
        .iter()
        .copied()
        .enumerate()
    {
        let settings = QmcSettings {
            points: if full_comparison { 1024 } else { 64 },
            shifts: if full_comparison { 16 } else { 8 },
            package_points: 19,
            seed,
            // Supplied rank-one rules; all generators are coprime to both
            // supported power-of-two sizes. These are fixed, not pilot tuned.
            rule: RuleSource::Supplied(match control {
                Control::ThresholdBubble => vec![1],
                Control::LinearSquare => vec![1, 433],
                Control::CubicCube => vec![1, 433, 1277],
            }),
            ..Default::default()
        };
        let mut runs = BTreeMap::new();
        let mut foreign = None;
        let mut execution_order = (0..modes.len()).collect::<Vec<_>>();
        if pair_index % 2 != 0 {
            execution_order.reverse();
        }
        for &index in &execution_order {
            let (result, replay) = run(
                &artifacts[&modes[index].program_recipe()],
                modes[index],
                validation,
                control,
                diagnostics,
                settings.clone(),
                foreign.take(),
            )?;
            check_reference(&result.estimate, control)?;
            foreign = Some(replay);
            runs.insert(index, result);
        }
        let baseline = &runs[&0];
        let mut coordinate_hash = blake3::Hasher::new();
        for range in &baseline.coordinates {
            coordinate_hash.update(range.digest.as_bytes());
        }
        if !coordinate_digests.insert(coordinate_hash.finalize().to_hex().to_string()) {
            return Err("independent seed pair repeated a previous coordinate sequence".into());
        }
        let mut comparisons = Vec::new();
        for first in 0..if dynamic { 2 } else { 1 } {
            for (_, second) in runs.range(first + 1..) {
                comparisons.push(compare(&runs[&first], second)?);
            }
        }
        pairs.push(serde_json::json!({
            "seed": seed, "execution_order": execution_order,
            "comparisons": comparisons, "runs": runs,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": if full_comparison { "matched-work analytic control; not a physical multiloop benchmark" }
                else if dynamic { "dynamic API smoke probe; too few replicas for a variance improvement claim" }
                else { "fixed-strength API smoke probe; no dynamic improvement claim" },
            "backend": backend, "validation": validation, "production_workers": 1,
            "diagnostics": diagnostics,
            "control": control,
            "generation": generation, "pairs": pairs,
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_uses_largest_signed_order_and_preserves_complex_trace() {
        let estimate = VectorEstimate {
            orders: vec![0, -2, 0],
            components: vec![
                CoefficientComponent::Real,
                CoefficientComponent::Real,
                CoefficientComponent::Imag,
            ],
            mean: vec![1., 0., 2.],
            standard_error: vec![2., 10., 3.],
            covariance_of_mean: vec![4., 0., 1., 0., 100., 0., 1., 0., 9.],
            production_complete: true,
        };
        assert_eq!(target_variance(&estimate).unwrap(), 13.);
        assert_eq!(ratio(0., 1.), None);
        assert_eq!(ratio(1., 0.), None);
        assert_eq!(ratio(1., f64::NAN), None);
        assert_eq!(ratio(f64::MAX, f64::MIN_POSITIVE), None);
        assert_eq!(ratio(f64::MIN_POSITIVE, f64::MAX), None);
        assert_eq!(ratio(4., 2.), Some(2.));
    }
}
