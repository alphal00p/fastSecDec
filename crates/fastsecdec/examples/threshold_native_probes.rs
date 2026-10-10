//! Small native API controls for the planned algebraic endpoint resolver.
//!
//! Run explicitly: `cargo run -p fastsecdec --example threshold_native_probes`.
//! These are algebra/representation probes, not threshold generation or sampling.
#[path = "threshold_native_probes/geometry.rs"]
mod geometry;
#[path = "threshold_native_probes/moving_root.rs"]
mod moving_root;
#[path = "threshold_native_probes/resultants.rs"]
mod resultants;
#[path = "threshold_native_probes/series.rs"]
mod series;

type ProbeResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> ProbeResult<()> {
    let cases = [
        ("native_puiseux", series::run()?),
        ("polynomial_blowup", geometry::run()?),
        ("resultant_and_shifted_norm", resultants::run()?),
        ("moving_root_composition", moving_root::run()?),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "passed": true,
            "general_resolver_implemented": false,
            "integration_or_generation_run": false,
            "cases": cases.into_iter().collect::<std::collections::BTreeMap<_, _>>()
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_series_preserves_known_zero_and_unknown_remainder() -> ProbeResult<()> {
        series::run().map(|_| ())
    }

    #[test]
    fn native_ideals_verify_relative_blowup_and_ramified_cell() -> ProbeResult<()> {
        geometry::run().map(|_| ())
    }

    #[test]
    fn elimination_resultant_is_distinct_from_shifted_norm() -> ProbeResult<()> {
        resultants::run().map(|_| ())
    }

    #[test]
    fn symbolic_root_derivatives_and_composed_full_density_agree() -> ProbeResult<()> {
        moving_root::run().map(|_| ())
    }
}
