//! Read saved results; this program does not generate or evaluate an amplitude.
use std::{error::Error, path::PathBuf};

use serde_json::{Value, json};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn number(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("missing or nonfinite result value: {value}").into())
}

fn complex(value: &Value) -> Result<[f64; 2]> {
    if value.is_array() {
        Ok([number(&value[0])?, number(&value[1])?])
    } else {
        Ok([number(&value["re"])?, number(&value["im"])?])
    }
}

fn norm(value: [f64; 2]) -> f64 {
    value[0].hypot(value[1])
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    norm([a[0] - b[0], a[1] - b[1]])
}

pub fn run() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("example/gg_hh_one_loop_ME"));
    if args.next().is_some() {
        return Err("usage: gghh_one_loop_compare [REPRODUCTION_DIRECTORY]".into());
    }
    let read = |path: &str| -> Result<Value> {
        Ok(serde_json::from_slice(&std::fs::read(base.join(path))?)?)
    };
    let fast = read("fastsecdec/result.json")?;
    let analytic = read("hepkit/result/result.json")?;
    let madloop = read("madloop/result.json")?;
    let ward1 = read("hepkit/ward1/result.json")?;
    let ward2 = read("hepkit/ward2/result.json")?;
    let fsd = complex(&fast["physical_color_coefficient"])?;
    let hep = complex(&analytic["physical_color_coefficient"])?;
    let mg = complex(&madloop["amplitude_color_coefficient"])?;
    let covariance = &fast["physical_color_coefficient_covariance"];
    let variance_re = number(&covariance[0][0])?;
    let variance_im = number(&covariance[1][1])?;
    if variance_re < 0.0 || variance_im < 0.0 {
        return Err("negative amplitude variance".into());
    }
    let sigma = (variance_re + variance_im).sqrt();
    if sigma == 0.0 {
        return Err("expected a nonzero numerical sampling uncertainty".into());
    }
    let scale = norm(hep);
    if scale == 0.0 {
        return Err("the specified benchmark requires a nonzero amplitude".into());
    }
    let reference_allowance = 1e-9 * scale;
    let numerical_allowance = 5.0 * sigma + reference_allowance;
    let checks = json!({
        "deterministic_references_agree": distance(hep, mg) <= reference_allowance,
        "fastsecdec_matches_hepkit": distance(fsd, hep) <= numerical_allowance,
        "fastsecdec_matches_madloop": distance(fsd, mg) <= numerical_allowance,
        "amplitude_relative_uncertainty_at_most_one_per_mil": sigma / norm(fsd) <= 1e-3,
        "fastsecdec_poles_compatible_with_zero": fast["checks"]["poles_compatible_with_zero"] == true,
        "hepkit_poles_cancel": analytic["checks"]["poles_cancel"] == true,
        "ward1_pass": ward1["checks"]["ward_pass"] == true,
        "ward2_pass": ward2["checks"]["ward_pass"] == true,
        "eight_fastsecdec_diagrams": fast["diagram_count"] == 8,
        "eight_hepkit_diagrams": analytic["diagram_count"] == 8,
        "eight_madloop_diagrams": madloop["loop_diagrams"] == 8,
    });
    let rows = [
        (
            "FastSecDec",
            fsd,
            &fast["color_summed_fixed_helicity_squared"],
        ),
        (
            "HEPKit",
            hep,
            &analytic["color_summed_fixed_helicity_squared"],
        ),
        ("MadLoop", mg, &madloop["color_sum_fixed_helicity"]),
    ];
    let mut summaries = Vec::new();
    for (method, amplitude, saved_square) in rows {
        let saved_square = number(saved_square)?;
        let square = 8.0 * norm(amplitude).powi(2);
        if (saved_square - square).abs() > 1e-10 * square.abs() {
            return Err(
                format!("{method}: saved colour sum has inconsistent normalization").into(),
            );
        }
        summaries.push(json!({
            "method": method,
            "physical_color_coefficient": amplitude,
            "color_summed_fixed_helicity_squared": saved_square,
        }));
        println!(
            "{method:12} A++ = {:+.15e} {:+.3e}i; color sum = {:.15e}",
            amplitude[0], amplitude[1], saved_square
        );
    }
    let passed = checks
        .as_object()
        .ok_or("invalid comparison checks")?
        .values()
        .all(|value| value == true);
    let result = json!({
        "normalization": "M_ab=delta_ab*A++; colour sum=8*|A++|^2; no spin or colour average",
        "results": summaries,
        "fastsecdec_amplitude_standard_error": sigma,
        "fastsecdec_amplitude_relative_standard_error": sigma / norm(fsd),
        "fastsecdec_hepkit_distance_over_combined_component_standard_error": distance(fsd, hep) / sigma,
        "fastsecdec_madloop_distance_over_combined_component_standard_error": distance(fsd, mg) / sigma,
        "hepkit_madloop_absolute_difference": distance(hep, mg),
        "reference_relative_allowance": 1e-9,
        "numerical_standard_error_allowance": 5,
        "checks": checks,
        "passed": passed,
    });
    std::fs::write(
        base.join("comparison.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("FastSecDec standard error: {sigma:.6e}; comparison passed: {passed}");
    if !passed {
        return Err("comparison checks failed; see comparison.json".into());
    }
    Ok(())
}
