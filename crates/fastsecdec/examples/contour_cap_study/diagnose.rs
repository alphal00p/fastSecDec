//! Bounded replay of frozen native sample points; no new sampling or estimator.
use super::partition::PreparedPartition;
use super::qmc::Point;
use super::*;
use fastsecdec::contour::ContourMode;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticPlan {
    sample_plan: Pin,
    sample_invocation: Pin,
    sample_points: Pin,
    top_per_arm_sector: usize,
    seconds: f64,
    include_partition: bool,
}
#[derive(Deserialize)]
struct RecordedPoints {
    seed: u64,
    scope: ResultScope,
    arms: Vec<ArmPoints>,
}
#[derive(Deserialize)]
struct ArmPoints {
    arm: Arm,
    points: BTreeMap<u64, Vec<Point>>,
}
#[derive(Serialize)]
struct Origin {
    arm: String,
    weighted_vector: Vec<f64>,
}
type PointKey = (u64, Vec<u64>, u64);
fn emit(file: &mut File, value: &Value) -> CliResult<()> {
    serde_json::to_writer(&mut *file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

pub(super) fn run(path: &Path, out: &Path) -> CliResult<()> {
    let started = Instant::now();
    let diagnostic: DiagnosticPlan = serde_json::from_reader(File::open(path)?)?;
    require(
        diagnostic.seconds.is_finite()
            && diagnostic.seconds > 0.
            && diagnostic.top_per_arm_sector > 0,
        "invalid diagnostic limits",
    )?;
    for pin in [
        &diagnostic.sample_plan,
        &diagnostic.sample_invocation,
        &diagnostic.sample_points,
    ] {
        pin.verify()?;
    }
    require(
        diagnostic.sample_points.path.parent() == diagnostic.sample_invocation.path.parent(),
        "point/invocation directories differ",
    )?;
    let invocation: Value =
        serde_json::from_reader(File::open(&diagnostic.sample_invocation.path)?)?;
    require(
        invocation["plan"] == serde_json::to_value(&diagnostic.sample_plan)?,
        "point producer plan differs",
    )?;
    let plan: Plan = serde_json::from_reader(File::open(&diagnostic.sample_plan.path)?)?;
    let points: RecordedPoints =
        serde_json::from_reader(File::open(&diagnostic.sample_points.path)?)?;
    require(points.seed == plan.seed, "point seed differs")?;
    let (mut owner, loading) = load(&plan)?;
    let parameters = physics(&plan)?;
    let scope = bound_scope(&mut owner, &plan, &parameters)?;
    require(points.scope == scope, "point scope differs")?;
    let mut union = BTreeMap::<u64, Vec<Point>>::new();
    let mut seen = BTreeSet::new();
    let mut origins = BTreeMap::<PointKey, Vec<Origin>>::new();
    for recorded in points.arms {
        let arm = plan
            .arms
            .iter()
            .find(|arm| arm.id == recorded.arm.id)
            .ok_or("unknown point-producing arm")?;
        require(
            arm.settings == recorded.arm.settings,
            "point arm prescription differs",
        )?;
        for (sector, points) in recorded.points {
            for point in points.into_iter().take(diagnostic.top_per_arm_sector) {
                require(
                    point.sector == sector && (sector as usize) < owner.sectors().len(),
                    "point sector differs",
                )?;
                require(
                    plan.sector_ids
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&sector)),
                    "point outside selected scope",
                )?;
                require(
                    point.coordinates.len() == owner.sectors()[sector as usize].dimension()
                        && point
                            .coordinates
                            .iter()
                            .all(|x| x.is_finite() && (0.0..=1.0).contains(x)),
                    "invalid native cube point",
                )?;
                require(
                    point.coordinate_bits
                        == point
                            .coordinates
                            .iter()
                            .map(|x| x.to_bits())
                            .collect::<Vec<_>>()
                        && point.weight_bits == point.weight.to_bits()
                        && point.weight.is_finite()
                        && point.weight >= 0.,
                    "recorded point bits differ",
                )?;
                require(
                    point.index < plan.points && point.shift < u64::from(plan.shifts),
                    "point index outside native design",
                )?;
                let key = (sector, point.coordinate_bits.clone(), point.weight_bits);
                origins.entry(key.clone()).or_default().push(Origin {
                    arm: arm.id.clone(),
                    weighted_vector: point.weighted_vector.clone(),
                });
                if seen.insert(key) {
                    union.entry(sector).or_default().push(point);
                }
            }
        }
    }
    require(!union.is_empty(), "no recorded points")?;
    save(
        &out.join("setup.json"),
        &json!({"loading":loading,"plan":Pin::new(path)?,"sample_plan":diagnostic.sample_plan,"points":diagnostic.sample_points,"scope":scope,"union_points":union.values().map(Vec::len).sum::<usize>(),"sampling":false,"estimator":false,"description":"Frozen top-point union replay under each original prescription; callback ranges and base-chart attribution are distinct"}),
    )?;
    let mut log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("callbacks.jsonl"))?;
    owner.set_contour_diagnostics(ContourDiagnosticsMode::Aggregate)?;
    let mut failures = 0;
    let mut admission_refused = 0;
    let mut parity_failures = 0;
    let mut replayed = 0;
    for arm in &plan.arms {
        deadline(started, diagnostic.seconds)?;
        match admit(
            &mut owner,
            &plan,
            arm,
            &parameters,
            &scope,
            diagnostic.seconds - started.elapsed().as_secs_f64(),
        ) {
            Ok(report) => emit(
                &mut log,
                &json!({"kind":"admission","arm":arm,"report":report}),
            )?,
            Err(error) => {
                admission_refused += 1;
                emit(
                    &mut log,
                    &json!({"kind":"admission","arm":arm,"error":error.to_string()}),
                )?;
                continue;
            }
        }
        for (&sector, points) in &union {
            let mut context = owner.evaluation_context(sector as usize, Default::default())?;
            for point in points {
                deadline(started, diagnostic.seconds)?;
                context.take_contour_runtime_report()?;
                let mut vector = vec![0.; owner.orders().len()];
                let at = Instant::now();
                let result =
                    context.evaluate_weighted(&point.coordinates, point.weight, &mut vector);
                let seconds = at.elapsed().as_secs_f64();
                let runtime = context.take_contour_runtime_report()?;
                let mut accounting = fastsecdec::status::EvaluationDiagnostics::default();
                if let Ok(report) = &result {
                    accounting.record_replay(*report)?;
                }
                let error = result.as_ref().err().map(ToString::to_string);
                if error.is_some() {
                    failures += 1;
                }
                let origin = &origins[&(sector, point.coordinate_bits.clone(), point.weight_bits)];
                let same_arm = origin.iter().find(|source| source.arm == arm.id);
                let parity = same_arm.map(|source| {
                    result.is_ok()
                        && source.weighted_vector.len() == vector.len()
                        && source
                            .weighted_vector
                            .iter()
                            .zip(&vector)
                            .all(|(old, new)| {
                                old.is_finite()
                                    && new.is_finite()
                                    && (old - new).abs() <= 1e-10 * old.abs().max(new.abs()).max(1.)
                            })
                });
                if parity == Some(false) {
                    parity_failures += 1;
                }
                emit(
                    &mut log,
                    &json!({"kind":"point","arm":arm,"point":point,"recorded_sources":origin,"same_arm_full_vector_parity":parity,"parity_scaled_tolerance":1e-10,"weighted_vector":result.as_ref().ok().map(|_|&vector),"replay":accounting,"error":error,"seconds_with_diagnostics":seconds,"actual_callback_report":runtime,"callback_scope":"all root requests made by this residual evaluation, including faces and precision retries; not one unique base-chart lambda"}),
                )?;
                replayed += 1;
            }
        }
    }
    if diagnostic.include_partition {
        let mut log = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("partitions.jsonl"))?;
        for (&sector, points) in &union {
            deadline(started, diagnostic.seconds)?;
            let charts = owner
                .generation_metadata()
                .ok_or("native generation charts unavailable")?
                .charts();
            let matches = charts
                .iter()
                .filter(|chart| {
                    chart.kernel_sector() == Some(sector as usize)
                        && chart.source_index() == chart.representative()
                        && chart.coordinates().target_parameters().len()
                            == owner.sectors()[sector as usize].dimension()
                })
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                emit(
                    &mut log,
                    &json!({"sector":sector,"unavailable":"no unique representative base chart with residual dimension","candidate_count":matches.len()}),
                )?;
                continue;
            }
            let chart = matches[0];
            let construction = match plan.arms[0].settings.deformation {
                ContourMode::Dynamical { construction, .. } => construction,
                _ => {
                    emit(
                        &mut log,
                        &json!({"sector":sector,"unavailable":"fixed prescription has no dynamic envelope"}),
                    )?;
                    continue;
                }
            };
            let at = Instant::now();
            let prepared = PreparedPartition::new(chart, &parameters, construction);
            let preparation_seconds = at.elapsed().as_secs_f64();
            let mut partition = match prepared {
                Ok(value) => value,
                Err(error) => {
                    emit(
                        &mut log,
                        &json!({"sector":sector,"unavailable":error.to_string(),"preparation_seconds":preparation_seconds}),
                    )?;
                    continue;
                }
            };
            emit(
                &mut log,
                &json!({"kind":"prepared","sector":sector,"preparation_seconds":preparation_seconds}),
            )?;
            for arm in &plan.arms {
                let ContourMode::Dynamical {
                    safety_fraction,
                    lambda_cap,
                    displacement_cap,
                    construction: other,
                } = arm.settings.deformation
                else {
                    continue;
                };
                require(
                    construction == other,
                    "mixed envelope constructions in one saved recipe",
                )?;
                for point in points {
                    deadline(started, diagnostic.seconds)?;
                    let at = Instant::now();
                    let value = partition.evaluate(
                        &point.coordinates,
                        safety_fraction,
                        lambda_cap,
                        displacement_cap,
                    );
                    emit(
                        &mut log,
                        &json!({"kind":"point","sector":sector,"arm":arm,"point":point,"partition":value.as_ref().ok(),"unavailable":value.as_ref().err().map(ToString::to_string),"seconds":at.elapsed().as_secs_f64(),"scope":"base representative chart at original coordinates; not an identification of endpoint-face callbacks"}),
                    )?;
                }
            }
        }
    }
    save(
        &out.join("summary.json"),
        &json!({"replayed_points":replayed,"native_failures":failures,"admission_refused":admission_refused,"parity_failures":parity_failures,"seconds":started.elapsed().as_secs_f64(),"new_samples":0,"new_estimates":0,"sampling":false,"complete_replay":failures==0&&admission_refused==0&&parity_failures==0,"partial_attribution_may_be_unavailable":true}),
    )?;
    require(
        failures == 0 && parity_failures == 0,
        "native recorded-point replay failed",
    )
}
