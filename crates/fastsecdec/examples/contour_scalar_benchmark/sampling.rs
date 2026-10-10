use super::*;
use crate::{contour_pilot, coordinates::RangeHasher};
use fastsecdec::{
    integration::{QmcSession, QmcSettings, RuleSource},
    kernel::{KernelError, StabilitySettings},
    results::{KernelResultManifest, ResultScope},
    status::EvaluationDiagnostics,
};
use std::collections::{BTreeMap, BTreeSet};

fn pilot(kernels: &mut KernelSet, saved: &Saved, cap: f64, start: Instant) -> CliResult<Value> {
    deadline(start, 15.)?;
    let settings = settings(saved.recipe, cap)?;
    let bind = Instant::now();
    kernels.bind_parameters_with_contour(&BTreeMap::new(), &settings)?;
    let bind_seconds = bind.elapsed().as_secs_f64();
    let pilot_start = Instant::now();
    let seed = 202610103901 + saved.case_index as u64;
    let report = contour_pilot::run(
        kernels,
        &settings,
        seed,
        Some(&ResultScope::FullIntegral),
        |_| deadline(start, 15.),
    )?;
    kernels.validate_integration_readiness(&ResultScope::FullIntegral)?;
    deadline(start, 15.)?;
    Ok(
        json!({"settings":settings,"bind_seconds":bind_seconds,"pilot_seconds":pilot_start.elapsed().as_secs_f64(),
        "report":report,"provenance":report.as_ref().map(|r|contour_pilot::provenance(kernels,seed,r)),
        "exact_coefficients":kernels.exact_coefficients(),"bound_content_id":kernels.content_id()}),
    )
}

fn causal_refusal(error: &(dyn std::error::Error + 'static)) -> bool {
    // Only the native fixed-contour certified-sign boundary permits lowering.
    // Solver, unsupported arithmetic, layout and unknown failures stay failures.
    matches!(error.downcast_ref::<KernelError>(),Some(KernelError::Contour(reason))
        if reason.contains("homotopy fraction=") &&
          (reason.contains("certified positive imaginary part") ||
           reason.contains("certified negative real part") ||
           reason.contains("causal/positive-factor condition unresolved")))
}

pub(super) fn admit(saved_path: &Path, out: &Path) -> CliResult<()> {
    let start = Instant::now();
    let (saved, mut kernels, restore) = load(saved_path)?;
    event(out, json!({"stage":"restored","details":restore}))?;
    let mut caps = Vec::new();
    for cap in CAPS {
        match pilot(&mut kernels, &saved, cap, start) {
            Ok(report) => caps.push(json!({"cap":cap,"accepted":true,"pilot":report})),
            Err(error) => {
                let refusal = causal_refusal(error.as_ref());
                caps.push(json!({"cap":cap,"accepted":false,"causal_refusal":refusal,"error":error.to_string()}));
                event(out, caps.last().unwrap().clone())?;
                if !refusal {
                    break;
                }
            }
        }
    }
    save(
        &out.join("admission.json"),
        &json!({"case":saved.case,"case_index":saved.case_index,"recipe":saved.recipe,
        "jacobian":saved.jacobian,"saved":Pin::new(saved_path)?,"artifact":saved.artifact,
        "schema":saved.schema,"caps":caps,"restore":restore,"seconds":start.elapsed().as_secs_f64(),"budget_seconds":15}),
    )
}

pub(super) fn select(paths: &[String], out: &Path) -> CliResult<()> {
    let mut reports = Vec::new();
    let mut identities = BTreeSet::new();
    for path in paths {
        let pin = Pin::new(Path::new(path))?;
        let report: Value = serde_json::from_slice(&pin.read()?)?;
        identities.insert((report["recipe"].to_string(), report["jacobian"].to_string()));
        reports.push((pin, report));
    }
    require(
        identities.len() == 6,
        "common cap needs all six distinct arms",
    )?;
    let first = &reports[0].1;
    require(
        reports
            .iter()
            .all(|(_, r)| r["case"] == first["case"] && r["case_index"] == first["case_index"]),
        "common cap mixes cases",
    )?;
    for recipe in ["fixed-v1", "dynamic-polynomial-v1", "dynamic-sign-aware-v1"] {
        let pair = reports
            .iter()
            .filter(|(_, r)| r["recipe"] == recipe)
            .collect::<Vec<_>>();
        require(
            pair.len() == 2 && pair[0].1["schema"] == pair[1].1["schema"],
            "Jacobian pair has different native source/coordinate/layout schema",
        )?;
    }
    let mut selected = None;
    for cap in CAPS {
        let rows = reports
            .iter()
            .map(|(_, r)| {
                r["caps"]
                    .as_array()
                    .and_then(|rows| rows.iter().find(|row| row["cap"].as_f64() == Some(cap)))
            })
            .collect::<Vec<_>>();
        require(
            rows.iter().all(|row| row.is_some()),
            "missing cap admission; do not lower after a censored/structural failure",
        )?;
        if rows.iter().all(|r| r.unwrap()["accepted"] == true) {
            selected = Some(cap);
            break;
        }
        require(
            rows.iter()
                .all(|r| r.unwrap()["accepted"] == true || r.unwrap()["causal_refusal"] == true),
            "cap reduction requires actual causal refusal",
        )?;
    }
    save(
        &out.join("common-cap.json"),
        &json!({"case":first["case"],"case_index":first["case_index"],"cap":selected,
        "admissions":reports.iter().map(|(p,_)|p).collect::<Vec<_>>(),"accepted":selected.is_some(),
        "selection":"largest frozen cap admitted by all six native owners; no production/reference use"}),
    )?;
    require(selected.is_some(), "no common admitted cap")
}

pub(super) fn sample(
    saved_path: &Path,
    common_path: &Path,
    seed: u64,
    out: &Path,
) -> CliResult<()> {
    let setup = Instant::now();
    let (saved, mut kernels, restore) = load(saved_path)?;
    let common_pin = Pin::new(common_path)?;
    let common: Value = serde_json::from_slice(&common_pin.read()?)?;
    require(
        common["accepted"] == true && common["case"] == saved.case,
        "common-cap proof is missing or belongs to another case",
    )?;
    let saved_pin = Pin::new(saved_path)?;
    let admissions: Vec<Pin> = serde_json::from_value(common["admissions"].clone())?;
    let mut matches = 0;
    for pin in &admissions {
        let r: Value = serde_json::from_slice(&pin.read()?)?;
        if r["saved"] == serde_json::to_value(&saved_pin)? {
            matches += 1;
        }
    }
    require(
        admissions.len() == 6 && matches == 1,
        "common cap does not pin this native owner exactly once",
    )?;
    let cap = common["cap"].as_f64().ok_or("missing common cap")?;
    require(
        [
            202610103001 + 100 * saved.case_index as u64,
            202610103002 + 100 * saved.case_index as u64,
        ]
        .contains(&seed),
        "seed is outside frozen scalar design",
    )?;
    let pilot = pilot(&mut kernels, &saved, cap, setup)?;
    let manifest = KernelResultManifest::from_kernels(&kernels);
    let problem = manifest.integration_problem(&ResultScope::FullIntegral, kernels.content_id())?;
    let reference = saved
        .reference
        .values_for_layout(kernels.orders(), kernels.components())?;
    let finite_indices = kernels
        .orders()
        .iter()
        .enumerate()
        .filter_map(|(i, o)| (*o == 0).then_some(i))
        .collect::<Vec<_>>();
    save(
        &out.join("setup.json"),
        &json!({"case":saved.case,"recipe":saved.recipe,"jacobian":saved.jacobian,"seed":seed,
        "saved":saved_pin,"common_cap":common_pin,"restore":restore,"pilot":pilot,
        "manifest":manifest,"reference":saved.reference,"reference_vector":reference,"finite_indices":finite_indices,
        "precision":fastsecdec::kernel::PrecisionPolicy::default(),"stability":StabilitySettings::default(),
        "setup_seconds":setup.elapsed().as_secs_f64(),"setup_budget_seconds":15}),
    )?;
    deadline(setup, 15.)?;
    let sampling = Instant::now();
    event(
        out,
        json!({"stage":"sampling_started","sampling_elapsed_seconds":sampling.elapsed().as_secs_f64(),"budget_seconds":20}),
    )?;
    let mut completed = Vec::new();
    for points in EPOCHS {
        deadline(sampling, 20.)?;
        let epoch_start = Instant::now();
        let settings = QmcSettings {
            points,
            shifts: 8,
            seed,
            package_points: 1024,
            rule: RuleSource::Kuo,
            ..Default::default()
        };
        let mut session = QmcSession::democratic(problem.clone(), settings.clone())?;
        let mut contexts = (0..kernels.sectors().len())
            .map(|i| Ok((i as u64, kernels.evaluation_context(i, Default::default())?)))
            .collect::<CliResult<BTreeMap<_, _>>>()?;
        let mut coordinates = Vec::new();
        let mut hash_seconds = 0.;
        let mut diagnostics = EvaluationDiagnostics::default();
        let mut failure_context = None;
        let work: CliResult<()> = (|| {
            while let Some(task) = session.next_work()? {
                deadline(sampling, 20.)?;
                let sector = task.sector_id();
                let mut worker = session.worker_context(sector)?;
                let dimension = worker.plan().dimension();
                let mut index = task.work().start();
                let mut ranges = Vec::<RangeHasher>::new();
                let context = contexts
                    .get_mut(&sector)
                    .ok_or("missing native evaluation context")?;
                let result = worker.evaluate_weighted_batch(
                    task,
                    64,
                    |points_batch, weights, outputs| -> CliResult<()> {
                        let hash_start = Instant::now();
                        for (point, weight) in points_batch.chunks_exact(dimension).zip(weights) {
                            let shift = index / settings.points;
                            let local = index % settings.points;
                            if ranges.last().is_none_or(|r| r.range.shift != shift) {
                                ranges.push(RangeHasher::new(sector, shift, local, dimension));
                            }
                            ranges.last_mut().unwrap().push(local, point, *weight);
                            index += 1;
                        }
                        hash_seconds += hash_start.elapsed().as_secs_f64();
                        match context.evaluate_weighted_batch_controlled(points_batch, weights, outputs,
                            || sampling.elapsed().as_secs_f64()>=20.) {
                            Ok(reports)=>{
                                for report in reports {diagnostics.record_replay(report)?;}
                                Ok(())
                            }
                            Err(error)=>{
                                let row=error.completed.len();
                                for report in &error.completed {diagnostics.record_replay(*report)?;}
                                let cancelled=matches!(error.error,KernelError::Cancelled);
                                if !cancelled {diagnostics.record_failure()?;}
                                let failed_index=index-weights.len() as u64+row as u64;
                                let point=points_batch.get(row*dimension..(row+1)*dimension);
                                failure_context=Some(json!({"sector":sector,"index":failed_index,
                                    "shift":failed_index/settings.points,"point":point,
                                    "point_bits":point.map(|p|p.iter().map(|v|v.to_bits()).collect::<Vec<_>>()),
                                    "weight":weights.get(row),"cancelled":cancelled,"native_error":error.to_string()}));
                                Err(error.into())
                            }
                        }
                    },
                )?;
                coordinates.extend(ranges.into_iter().map(RangeHasher::finish));
                session.submit(result)?;
            }
            require(session.is_complete(), "native allocation incomplete")
        })();
        let elapsed = epoch_start.elapsed().as_secs_f64();
        coordinates.sort_by_key(|r| (r.sector, r.shift, r.start));
        let observation = session.diagnostic_observation()?;
        let estimate = if work.is_ok() {
            Some(session.estimate()?)
        } else {
            None
        };
        if let Some(estimate) = &estimate {
            estimate.validate()?;
        }
        let errors = estimate.as_ref().map(|e| {
            e.mean
                .iter()
                .zip(&reference)
                .map(|(a, b)| a - b)
                .collect::<Vec<_>>()
        });
        let finite_trace = estimate.as_ref().map(|e| {
            finite_indices
                .iter()
                .map(|i| e.covariance_of_mean[*i * e.mean.len() + *i])
                .sum::<f64>()
        });
        let checks = contexts
            .iter()
            .map(|(id, c)| (*id, c.contour_validation_report()))
            .collect::<Vec<_>>();
        require(
            checks
                .iter()
                .all(|(_, r)| r.as_ref().is_none_or(|r| r.checked_arguments == 0)),
            "Pilot policy performed optional production checks",
        )?;
        let row = json!({"case":saved.case,"recipe":saved.recipe,"jacobian":saved.jacobian,"seed":seed,"points":points,"settings":settings,
            "complete":work.is_ok(),"error":work.as_ref().err().map(ToString::to_string),"estimate":estimate,"observation":observation,
            "evaluation_diagnostics":diagnostics,"failure_context":failure_context,
            "reference_vector":reference,"error_vector":errors,"finite_indices":finite_indices,"finite_covariance_trace":finite_trace,
            "exact_offsets":kernels.exact_coefficients(),"native_design":session.design(),"coordinate_hashes":coordinates,
            "sampling_seconds_including_hash":elapsed,"coordinate_hash_seconds":hash_seconds,"sampling_seconds_excluding_hash":elapsed-hash_seconds,
            "evaluator_metrics":contexts.iter().map(|(id,c)|(*id,c.evaluation_metrics())).collect::<Vec<_>>(),"production_validation":checks,
            "sampling_cumulative_seconds":sampling.elapsed().as_secs_f64(),"no_pooling":true});
        save(&out.join(format!("epoch-{points}.json")), &row)?;
        work?;
        completed.push(points);
    }
    save(
        &out.join("summary.json"),
        &json!({"complete":true,"epochs":completed,"sampling_seconds":sampling.elapsed().as_secs_f64(),
        "sampling_budget_seconds":20,"deadline_overshoot_seconds":(sampling.elapsed().as_secs_f64()-20.).max(0.),
        "case":saved.case,"recipe":saved.recipe,"jacobian":saved.jacobian,"seed":seed,"no_pooling":true}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cap_ladder_never_relabels_a_structural_or_precision_error_as_causality() {
        assert!(!causal_refusal(&KernelError::Contour(
            "unsupported certificate".into()
        )));
        assert!(!causal_refusal(&KernelError::PrecisionExhausted {
            bits: 1536
        }));
        assert!(causal_refusal(&KernelError::Contour(
            "chart0 homotopy fraction=1: F has a certified positive imaginary part".into()
        )));
    }
}
