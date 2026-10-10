use super::*;
use crate::coordinates::{CoordinateRange, RangeHasher};
use fastsecdec::{
    integration::{QmcSession, QmcSettings, RuleSource},
    status::EvaluationDiagnostics,
};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Point {
    pub sector: u64,
    pub shift: u64,
    pub index: u64,
    pub coordinates: Vec<f64>,
    pub coordinate_bits: Vec<u64>,
    pub weight: f64,
    pub weight_bits: u64,
    pub weighted_vector: Vec<f64>,
    pub ranking_max_abs_finite: f64,
}
fn retain(points: &mut Vec<Point>, point: Point, count: usize) {
    if points.len() < count
        || points
            .last()
            .is_some_and(|p| point.ranking_max_abs_finite > p.ranking_max_abs_finite)
    {
        points.push(point);
        points.sort_by(|a, b| {
            b.ranking_max_abs_finite
                .total_cmp(&a.ranking_max_abs_finite)
        });
        points.truncate(count);
    }
}
pub(super) fn run(
    plan: &Plan,
    owner: &mut KernelSet,
    parameters: &BTreeMap<Symbol, f64>,
    scope: &ResultScope,
    out: &Path,
    mut setup_spent: f64,
) -> CliResult<()> {
    let mut spent = 0.;
    let mut reports = Vec::new();
    let mut matched: Option<Vec<CoordinateRange>> = None;
    let mut points_by_arm = Vec::new();
    for (arm_index, arm) in plan.arms.iter().enumerate() {
        if spent >= plan.sampling_seconds {
            reports.push(json!({"arm":arm,"accepted":false,"reason":"cumulative sampling allowance exhausted; arm not started"}));
            continue;
        }
        if setup_spent >= plan.setup_seconds {
            reports.push(json!({"arm":arm,"accepted":false,"reason":"cumulative setup allowance exhausted; arm not started"}));
            continue;
        }
        let setup_at = Instant::now();
        let admission = admit(
            owner,
            plan,
            arm,
            parameters,
            scope,
            plan.setup_seconds - setup_spent,
        );
        setup_spent += setup_at.elapsed().as_secs_f64();
        let admitted = match admission {
            Ok(report) => report,
            Err(error) => {
                reports.push(json!({"arm":arm,"accepted":false,"reason":"native admission refused","error":error.to_string()}));
                continue;
            }
        };
        let manifest = KernelResultManifest::from_kernels(owner);
        let problem = manifest.integration_problem(scope, owner.content_id())?;
        let settings = QmcSettings {
            points: plan.points,
            shifts: plan.shifts,
            seed: plan.seed,
            package_points: 1024,
            rule: RuleSource::Kuo,
            ..Default::default()
        };
        let started = Instant::now();
        let allowance = plan.sampling_seconds - spent;
        let mut session = QmcSession::democratic(problem.clone(), settings.clone())?;
        let mut contexts = problem
            .sectors
            .iter()
            .map(|sector| {
                Ok((
                    sector.id,
                    owner.evaluation_context(sector.id as usize, Default::default())?,
                ))
            })
            .collect::<CliResult<BTreeMap<_, _>>>()?;
        let finite = owner
            .orders()
            .iter()
            .enumerate()
            .filter_map(|(i, order)| (*order == 0).then_some(i))
            .collect::<Vec<_>>();
        require(!finite.is_empty(), "no finite components in native layout")?;
        let mut hashes = Vec::new();
        let mut top = BTreeMap::<u64, Vec<Point>>::new();
        let mut diagnostics = EvaluationDiagnostics::default();
        let (mut hash_seconds, mut top_seconds, mut evaluator_seconds) = (0., 0., 0.);
        let mut failure_context = None;
        let work: CliResult<()> = (|| {
            while let Some(task) = session.next_work()? {
                deadline(started, allowance)?;
                let sector = task.sector_id();
                let mut worker = session.worker_context(sector)?;
                let dimension = worker.plan().dimension();
                let context = contexts
                    .get_mut(&sector)
                    .ok_or("missing native sector context")?;
                let mut index = task.work().start();
                let mut ranges = Vec::<RangeHasher>::new();
                let result=worker.evaluate_weighted_batch(task,64,|coordinates,weights,outputs|->CliResult<()>{
                    let begin=index;let hash=Instant::now();
                    for(point,weight)in coordinates.chunks_exact(dimension).zip(weights){
                        let shift=index/settings.points;let local=index%settings.points;
                        if ranges.last().is_none_or(|r|r.range.shift!=shift){ranges.push(RangeHasher::new(sector,shift,local,dimension));}
                        ranges.last_mut().unwrap().push(local,point,*weight);index+=1;
                    }hash_seconds+=hash.elapsed().as_secs_f64();
                    let timing=Instant::now();let evaluated=context.evaluate_weighted_batch_controlled(coordinates,weights,outputs,||started.elapsed().as_secs_f64()>=allowance);
                    evaluator_seconds+=timing.elapsed().as_secs_f64();
                    match evaluated{
                        Ok(rows)=>{for row in rows{diagnostics.record_replay(row)?;}},
                        Err(error)=>{
                            for row in &error.completed{diagnostics.record_replay(*row)?;}
                            let cancelled=matches!(error.error,fastsecdec::kernel::KernelError::Cancelled);
                            if !cancelled{diagnostics.record_failure()?;}
                            let row=error.completed.len();let point=coordinates.get(row*dimension..(row+1)*dimension);
                            failure_context=Some(json!({"sector":sector,"global_index":begin+row as u64,"point":point,
                                "point_bits":point.map(|p|p.iter().map(|v|v.to_bits()).collect::<Vec<_>>()),"weight":weights.get(row),"cancelled":cancelled,"native_error":error.to_string()}));
                            return Err(error.into());
                        }
                    }
                    let top_at=Instant::now();let output_count=owner.orders().len();
                    for (row,point) in coordinates.chunks_exact(dimension).enumerate(){
                        let vector=&outputs[row*output_count..(row+1)*output_count];
                        let score=finite.iter().map(|i|vector[*i].abs()).fold(0.,f64::max);
                        let candidates=top.entry(sector).or_default();
                        if candidates.len()<plan.top_k_per_sector||candidates.last().is_some_and(|p|score>p.ranking_max_abs_finite){
                            let global=begin+row as u64;
                            retain(candidates,Point{sector,shift:global/settings.points,index:global%settings.points,coordinates:point.to_vec(),coordinate_bits:point.iter().map(|v|v.to_bits()).collect(),weight:weights[row],weight_bits:weights[row].to_bits(),weighted_vector:vector.to_vec(),ranking_max_abs_finite:score},plan.top_k_per_sector);
                        }
                    }top_seconds+=top_at.elapsed().as_secs_f64();Ok(())
                })?;
                hashes.extend(ranges.into_iter().map(RangeHasher::finish));
                session.submit(result)?;
            }
            require(session.is_complete(), "native allocation incomplete")
        })();
        let elapsed = started.elapsed().as_secs_f64();
        spent += elapsed;
        hashes.sort_by_key(|r| (r.sector, r.shift, r.start));
        let same = if work.is_ok() {
            if let Some(expected) = &matched {
                *expected == hashes
            } else {
                true
            }
        } else {
            false
        };
        let observation = session.diagnostic_observation()?;
        let estimate = if work.is_ok() {
            let e = session.estimate()?;
            e.validate()?;
            Some(e)
        } else {
            None
        };
        let validation = contexts
            .iter()
            .map(|(id, c)| (*id, c.contour_validation_report()))
            .collect::<Vec<_>>();
        require(
            validation
                .iter()
                .all(|(_, r)| r.as_ref().is_none_or(|r| r.checked_arguments == 0)),
            "Pilot performed optional production checks",
        )?;
        let report = json!({"arm":arm,"arm_index":arm_index,"accepted":work.is_ok()&&same,"complete":work.is_ok(),"error":work.as_ref().err().map(ToString::to_string),"failure_context":failure_context,
            "scope":scope,"manifest":manifest,"admission":admitted,"settings":settings,"observation":observation,"estimate":estimate,"contributions":session.contributions()?,"design":session.design(),
            "coordinate_hashes":hashes,"same_actual_points_as_first_complete_arm":same,"top_points":top,"finite_indices":finite,
            "evaluation_diagnostics":diagnostics,"evaluator_metrics":contexts.iter().map(|(id,c)|(*id,c.evaluation_metrics())).collect::<Vec<_>>(),"production_validation":validation,
            "sampling_seconds_including_hash_and_top":elapsed,"coordinate_hash_seconds":hash_seconds,"top_selection_seconds":top_seconds,"native_evaluation_seconds":evaluator_seconds,
            "sampling_cumulative_seconds":spent,"sampling_allowance_seconds":plan.sampling_seconds,"setup_timing_excluded":true,"optional_root_diagnostics":"disabled"});
        save(&out.join(format!("arm-{arm_index}.json")), &report)?;
        reports.push(json!({"arm":arm,"accepted":work.is_ok()&&same,"native_report":format!("arm-{arm_index}.json")}));
        points_by_arm.push(json!({"arm":arm,"points":top}));
        if work.is_ok() && matched.is_none() {
            matched = Some(hashes);
        }
    }
    save(
        &out.join("points.json"),
        &json!({"seed":plan.seed,"scope":scope,"arms":points_by_arm,"selection":"bounded topK by maximum absolute finite weighted output per sector; no estimator replacement"}),
    )?;
    let accepted = reports.iter().all(|r| r["accepted"] == true);
    save(
        &out.join("summary.json"),
        &json!({"all_arms_accepted":accepted,"arms":reports,"sampling_seconds":spent,"setup_seconds":setup_spent,"scope":scope,"no_pooling":true}),
    )?;
    require(
        accepted,
        "one or more frozen cap arms refused, failed or censored",
    )
}
