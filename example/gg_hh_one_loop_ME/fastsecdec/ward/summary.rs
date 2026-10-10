//! Absolute Ward acceptance using the existing complete-vector covariance sum.
use crate::{
    Result,
    bookkeeping::{independent_sum, scaled},
};
use fastsecdec::{
    results::read_result,
    status::{CoefficientComponent, StoppingReason},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub fn run(
    work: &std::path::Path,
    reference: &std::path::Path,
    output: &std::path::Path,
) -> Result<()> {
    let manifest: Value = serde_json::from_slice(&std::fs::read(work.join("manifest.json"))?)?;
    let reference: Value = serde_json::from_slice(&std::fs::read(reference)?)?;
    let entries = manifest["diagrams"].as_array().ok_or("diagrams")?;
    let ward = manifest["ward"]
        .as_u64()
        .ok_or("Ward manifest requires replaced leg")?;
    if !(1..=2).contains(&ward)
        || reference["ward_replaced_incoming_index"].as_u64() != Some(ward - 1)
    {
        return Err("Ward manifest and native reference identify different legs".into());
    }
    if entries.len() != 8 {
        return Err("Ward summary requires all eight diagram rows".into());
    }
    let names = entries
        .iter()
        .map(|d| Ok(d["name"].as_str().ok_or("diagram name")?))
        .collect::<Result<BTreeSet<_>>>()?;
    let directories = entries
        .iter()
        .map(|d| Ok(d["directory"].as_str().ok_or("diagram directory")?))
        .collect::<Result<BTreeSet<_>>>()?;
    let reference_names = reference["contributions"]
        .as_array()
        .ok_or("native contributions")?
        .iter()
        .map(|d| Ok(d["diagram"].as_str().ok_or("native diagram name")?))
        .collect::<Result<BTreeSet<_>>>()?;
    if names.len() != 8
        || directories.len() != 8
        || names != reference_names
        || reference["checks"]["ward_pass"] != true
        || reference["checks"]["poles_cancel"] != true
    {
        return Err("Ward catalogue differs from the complete accepted native reference".into());
    }
    let tolerance = reference["checks"]["ward_relative_tolerance"]
        .as_f64()
        .ok_or("native Ward tolerance")?;
    let scale = reference["checks"]["cancellation_scale"]
        .as_f64()
        .ok_or("native Ward scale")?;
    let allowance = tolerance * scale;
    if !tolerance.is_finite()
        || tolerance < 0.
        || !scale.is_finite()
        || scale < 0.
        || !allowance.is_finite()
    {
        return Err("invalid native absolute Ward allowance".into());
    }
    let mut seeds = BTreeSet::new();
    let mut estimates = vec![];
    let mut rows = vec![];
    let mut targets = true;
    let mut individual = true;
    for entry in entries {
        let directory = entry["directory"].as_str().ok_or("directory")?;
        let seed = entry["seed"].as_u64().ok_or("seed")?;
        if !seeds.insert(seed) {
            return Err("Ward diagram seeds must be distinct".into());
        }
        let saved = read_result(&std::fs::read(
            work.join(format!("{directory}.result.json")),
        )?)?;
        if !saved.scope.is_full_integral() {
            return Err("Ward result not full scope".into());
        }
        let estimate = saved.contributions.total.ok_or("total")?;
        estimate.validate()?;
        if !estimate.production_complete {
            return Err("incomplete Ward production".into());
        }
        let qmc = saved.qmc_design.ok_or("QMC")?;
        if qmc.settings.seed != seed {
            return Err("seed mismatch".into());
        }
        targets &= saved.stopping_reason == StoppingReason::TargetReached;
        let native = reference["contributions"]
            .as_array()
            .ok_or("native contributions")?
            .iter()
            .find(|d| d["diagram"] == entry["name"])
            .ok_or("native diagram")?;
        let mut comparisons = vec![];
        for (i, (o, c)) in estimate.orders.iter().zip(&estimate.components).enumerate() {
            if !(-2..=0).contains(o) {
                return Err("reference supplies only orders -2 through 0".into());
            }
            let wanted = native["native_finite_simple_double_pole"][(-*o) as usize][if *c
                == CoefficientComponent::Real
            {
                "re"
            } else {
                "im"
            }]
            .as_f64()
            .ok_or("native coefficient")?;
            let residual = estimate.mean[i] - wanted;
            let pass = residual.abs() <= 5. * estimate.standard_error[i] + allowance;
            individual &= pass;
            comparisons.push(json!({"order":o,"component":c,"reference":wanted,"residual":residual,"passed":pass}));
        }
        rows.push(json!({"name":entry["name"],"seed":seed,"stopping_reason":saved.stopping_reason,"kernel_content_id":saved.manifest.kernel_content_id,"estimate":estimate,"qmc":qmc,"diagnostics":saved.evaluation_diagnostics,"reference_comparisons":comparisons}));
        estimates.push(estimate);
    }
    let total = independent_sum(&estimates.iter().collect::<Vec<_>>())?;
    if !total.orders.contains(&0) {
        return Err("Ward summary requires a finite-order output".into());
    }
    let physical = scaled(&total, 1. / (16. * std::f64::consts::PI.powi(2)));
    let mut checks = vec![];
    let mut zeros = true;
    for (i, (o, c)) in total.orders.iter().zip(&total.components).enumerate() {
        let pass = total.mean[i].abs() <= 5. * total.standard_error[i] + allowance;
        zeros &= pass;
        checks.push(json!({"order":o,"component":c,"mean":total.mean[i],"SE":total.standard_error[i],"bound":5.*total.standard_error[i]+allowance,"passed":pass}));
    }
    let finite_error = total
        .orders
        .iter()
        .zip(&total.standard_error)
        .filter(|(o, _)| **o == 0)
        .map(|(_, s)| s * s)
        .sum::<f64>()
        .sqrt();
    let precise = finite_error <= (8f64).sqrt() * 1e-3;
    let passed = entries.len() == 8 && targets && individual && zeros && precise;
    let out = json!({"ward":manifest["ward"],"passed":passed,"targets_reached":targets,"individual_reference_checks_pass":individual,"full_vector_zero_checks_pass":zeros,"finite_precision_pass":precise,"native_absolute_reference_allowance":allowance,"finite_joint_standard_error":finite_error,"native_laurent_estimate":total,"physical_laurent_estimate":physical,"full_vector_zero_checks":checks,"diagrams":rows,"normalization":manifest["normalization"]});
    std::fs::write(output, serde_json::to_vec_pretty(&out)?)?;
    eprintln!(
        "Ward {}: pass={passed}; target={targets}; individual={individual}; zero={zeros}; finite_error={finite_error}",
        manifest["ward"]
    );
    if !passed {
        return Err("Ward scientific gate failed; retained complete result".into());
    }
    Ok(())
}
