//! Bookkeeping for independently seeded native integration results.
use std::{collections::BTreeSet, path::PathBuf};

use fastsecdec::{
    integration::VectorEstimate,
    results::read_result,
    status::{CoefficientComponent, StoppingReason},
};
use numerica::domains::float::{DoubleFloat, RealLike};
use serde_json::{Value, json};

use crate::Result;

fn precise_sum(values: impl Iterator<Item = f64>) -> f64 {
    values
        .fold(DoubleFloat::from(0.0), |s, x| s + DoubleFloat::from(x))
        .to_f64()
}

pub(crate) fn independent_sum(estimates: &[&VectorEstimate]) -> Result<VectorEstimate> {
    let layout = estimates
        .iter()
        .flat_map(|e| e.orders.iter().copied().zip(e.components.iter().copied()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let maps = estimates
        .iter()
        .map(|e| {
            layout
                .iter()
                .map(|(order, component)| {
                    e.orders
                        .iter()
                        .zip(&e.components)
                        .position(|(o, c)| o == order && c == component)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let n = layout.len();
    let mean = (0..n)
        .map(|i| {
            precise_sum(
                estimates
                    .iter()
                    .zip(&maps)
                    .filter_map(|(e, m)| m[i].map(|j| e.mean[j])),
            )
        })
        .collect();
    let covariance_of_mean = (0..n * n)
        .map(|i| {
            precise_sum(estimates.iter().zip(&maps).filter_map(|(e, m)| {
                Some(e.covariance_of_mean[m[i / n]? * e.mean.len() + m[i % n]?])
            }))
        })
        .collect::<Vec<_>>();
    let standard_error = (0..n)
        .map(|i| covariance_of_mean[i * n + i].sqrt())
        .collect();
    let combined = VectorEstimate {
        orders: layout.iter().map(|x| x.0).collect(),
        components: layout.iter().map(|x| x.1).collect(),
        mean,
        standard_error,
        covariance_of_mean,
        production_complete: estimates.iter().all(|e| e.production_complete),
    };
    combined.validate()?;
    Ok(combined)
}

pub(crate) fn scaled(estimate: &VectorEstimate, multiplier: f64) -> VectorEstimate {
    VectorEstimate {
        mean: estimate.mean.iter().map(|v| v * multiplier).collect(),
        standard_error: estimate
            .standard_error
            .iter()
            .map(|v| v * multiplier.abs())
            .collect(),
        covariance_of_mean: estimate
            .covariance_of_mean
            .iter()
            .map(|v| v * multiplier * multiplier)
            .collect(),
        ..estimate.clone()
    }
}

pub fn run() -> Result<()> {
    let args = std::env::args_os().skip(2).collect::<Vec<_>>();
    let [inputs, work, output] = args.as_slice() else {
        return Err("usage: gghh_one_loop_me summarize INPUTS WORK OUTPUT.json".into());
    };
    let inputs = PathBuf::from(inputs);
    let work = PathBuf::from(work);
    let manifest: Value = serde_json::from_slice(&std::fs::read(inputs.join("manifest.json"))?)?;
    let entries = manifest["diagrams"].as_array().ok_or("missing diagrams")?;
    if entries.len() != 8 {
        return Err("the full amplitude requires eight diagrams".into());
    }
    let mut seeds = BTreeSet::new();
    let mut estimates = Vec::new();
    let mut diagrams = Vec::new();
    for entry in entries {
        let directory = entry["directory"].as_str().ok_or("missing directory")?;
        let seed = entry["seed"].as_u64().ok_or("missing seed")?;
        if !seeds.insert(seed) {
            return Err("diagram sampling seeds must be distinct".into());
        }
        let saved = read_result(&std::fs::read(
            work.join(format!("{directory}.result.json")),
        )?)?;
        if !saved.scope.is_full_integral() || saved.stopping_reason != StoppingReason::TargetReached
        {
            return Err(format!(
                "{directory} has no completed target-reaching full-integral result"
            )
            .into());
        }
        let estimate = saved
            .contributions
            .total
            .ok_or("missing authoritative total")?;
        estimate.validate()?;
        if !estimate.production_complete {
            return Err("incomplete production allocation".into());
        }
        let qmc = saved.qmc_design.ok_or("missing native QMC design")?;
        if qmc.settings.seed != seed {
            return Err("saved QMC seed differs from manifest".into());
        }
        diagrams.push(json!({
            "name":entry["name"], "kind":entry["kind"], "directory":directory,
            "seed":seed,"native_laurent_estimate":estimate,
            "raw_delta_contracted_laurent_estimate":scaled(&estimate,8.0),
            "kernel_content_id":saved.manifest.kernel_content_id,
            "sectors":saved.manifest.sectors.len(),
            "qmc_design":qmc,"timings":saved.timings,
            "evaluation_diagnostics":saved.evaluation_diagnostics,
        }));
        estimates.push(estimate);
    }
    let total = independent_sum(&estimates.iter().collect::<Vec<_>>())?;
    let mut poles_compatible_with_zero = true;
    let pole_checks = total
        .orders
        .iter()
        .zip(&total.components)
        .enumerate()
        .filter(|(_, (order, _))| **order < 0)
        .map(|(i, (order, component))| {
            let scale = precise_sum(estimates.iter().filter_map(|e| {
                e.orders
                    .iter()
                    .zip(&e.components)
                    .position(|(o, c)| o == order && c == component)
                    .map(|j| e.mean[j].abs())
            }));
            let roundoff = 64.0 * f64::EPSILON * (1.0 + scale);
            let bound = 5.0 * total.standard_error[i] + roundoff;
            let compatible = total.mean[i].abs() <= bound;
            poles_compatible_with_zero &= compatible;
            json!({"order":order,"component":component,"mean":total.mean[i],
                "standard_error":total.standard_error[i],"roundoff_allowance":roundoff,
                "five_sigma_bound":bound,"compatible_with_zero":compatible})
        })
        .collect::<Vec<_>>();
    if !poles_compatible_with_zero {
        return Err("the summed Laurent poles do not cancel within five standard errors".into());
    }
    let mut grouped = serde_json::Map::new();
    for kind in ["triangle", "box"] {
        let selected = entries
            .iter()
            .zip(&estimates)
            .filter_map(|(e, v)| (e["kind"] == kind).then_some(v))
            .collect::<Vec<_>>();
        grouped.insert(
            kind.into(),
            serde_json::to_value(independent_sum(&selected)?)?,
        );
    }
    let physical = scaled(&total, 1.0 / (16.0 * std::f64::consts::PI.powi(2)));
    let position = |component| {
        physical
            .orders
            .iter()
            .zip(&physical.components)
            .position(|(o, c)| *o == 0 && *c == component)
            .ok_or("missing finite coefficient")
    };
    let re = position(CoefficientComponent::Real)?;
    let im = position(CoefficientComponent::Imag)?;
    let n = physical.mean.len();
    let covariance = [
        [
            physical.covariance_of_mean[re * n + re],
            physical.covariance_of_mean[re * n + im],
        ],
        [
            physical.covariance_of_mean[im * n + re],
            physical.covariance_of_mean[im * n + im],
        ],
    ];
    let amplitude = [physical.mean[re], physical.mean[im]];
    let error = (covariance[0][0] + covariance[1][1]).sqrt();
    let relative = error / amplitude[0].hypot(amplitude[1]);
    if relative > 1e-3 {
        return Err(format!("full amplitude relative uncertainty {relative} exceeds 1e-3").into());
    }
    let squared = 8.0 * (amplitude[0].powi(2) + amplitude[1].powi(2));
    let squared_variance = 256.0
        * (amplitude[0].powi(2) * covariance[0][0]
            + amplitude[1].powi(2) * covariance[1][1]
            + 2.0 * amplitude[0] * amplitude[1] * covariance[0][1]);
    let result = json!({
        "format_version":1,"method":"native FastSecDec sector decomposition + randomized lattice QMC",
        "diagram_count":entries.len(),"manifest":manifest,"diagrams":diagrams,
        "sampling_relation":"Independent diagram seeds; each native total retains shared-sector and Laurent covariance",
        "checks":{"poles_compatible_with_zero":poles_compatible_with_zero,"pole_checks":pole_checks,"finite_relative_target_reached":relative<=1e-3},
        "native_laurent_estimate":total,"grouped_native_laurent_estimates":grouped,
        "raw_delta_contracted_laurent_estimate":scaled(&total,8.0),
        "physical_laurent_estimate":physical,
        "physical_color_coefficient":{"re":amplitude[0],"im":amplitude[1]},
        "physical_color_coefficient_covariance":covariance,
        "physical_color_coefficient_rms_standard_error":error,
        "physical_color_coefficient_relative_uncertainty":relative,
        "color_summed_fixed_helicity_squared":squared,
        "color_summed_fixed_helicity_squared_standard_error":squared_variance.sqrt(),
        "color_summed_fixed_helicity_squared_uncertainty_method":"First-order propagation using full Re/Im covariance",
    });
    std::fs::write(output, serde_json::to_vec_pretty(&result)?)?;
    eprintln!(
        "A++ = {:+.12e} {:+.12e}i; RMS error {:.3e}; relative {:.3e}",
        amplitude[0], amplitude[1], error, relative
    );
    Ok(())
}
