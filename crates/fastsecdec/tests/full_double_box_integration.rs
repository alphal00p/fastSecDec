//! Ignored complete-shift validation of the preserved fresh double-box artifact.
use fastsecdec::{
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec},
    kernel::{KernelSet, ReplayPolicy},
    status::EvaluationDiagnostics,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[test]
#[ignore = "complete double-box QMC validation and performance evidence"]
fn complete_double_box_laurent_vector() {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    let strategy =
        std::env::var("FASTSECDEC_GENERATION_PROBE_STRATEGY").unwrap_or_else(|_| "taylor".into());
    let points = std::env::var("FASTSECDEC_INTEGRAL_POINTS")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(1024);
    let shifts = std::env::var("FASTSECDEC_INTEGRAL_SHIFTS")
        .map(|s| s.parse::<u32>().unwrap())
        .unwrap_or(8);
    let seed = std::env::var("FASTSECDEC_INTEGRAL_SEED")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(18931);
    let worker_count = std::env::var("FASTSECDEC_INTEGRAL_WORKERS")
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(1);
    assert!(worker_count > 0);
    let started = Instant::now();
    let kernels = KernelSet::from_bytes(
        &std::fs::read(evidence.join(format!("double-box-{strategy}-factored.fsd"))).unwrap(),
    )
    .unwrap();
    let load_seconds = started.elapsed().as_secs_f64();
    let problem = IntegrationProblem::new_with_components(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(id, kernel)| SectorSpec {
                id: id as u64,
                dimension: kernel.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )
    .unwrap();
    let settings = QmcSettings {
        points,
        shifts,
        package_points: points,
        seed,
        ..Default::default()
    };
    let mut session = QmcSession::democratic(problem, settings.clone()).unwrap();
    let mut tasks = vec![Vec::new(); kernels.sectors().len()];
    while let Some(task) = session.next_work().unwrap() {
        tasks[task.sector_id() as usize].push(task);
    }
    let mut assignments = (0..worker_count).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, tasks) in tasks.into_iter().enumerate() {
        // Fixed sector ownership preserves each context's point/task history
        // regardless of worker count. All symbolic construction stays here.
        assignments[index % worker_count].push((
            session.worker_context(index as u64).unwrap(),
            kernels
                .evaluation_context(index, ReplayPolicy::default())
                .unwrap(),
            tasks,
        ));
    }
    let mut diagnostics = EvaluationDiagnostics::default();
    eprintln!(
        "Loaded {} complete sector vectors in {load_seconds:.3}s; starting {points}×{shifts} QMC",
        kernels.sectors().len()
    );
    let started = Instant::now();
    let mut last = started;
    let mut completed = 0u64;
    std::thread::scope(|scope| {
        let (send, receive) = std::sync::mpsc::channel();
        for assignment in assignments {
            let send = send.clone();
            scope.spawn(move || {
                for (mut worker, mut context, tasks) in assignment {
                    for task in tasks {
                        let count = task.point_count();
                        let mut diagnostic = EvaluationDiagnostics::default();
                        let result = worker.evaluate_weighted(task, |point, weight, output| {
                            let report = context
                                .evaluate_weighted(point, weight, output)
                                .map_err(|error| error.to_string())?;
                            diagnostic
                                .record_replay(report)
                                .map_err(|error| error.to_string())
                        });
                        let failed = result.is_err();
                        send.send((result, count, diagnostic)).unwrap();
                        if failed {
                            return;
                        }
                    }
                }
            });
        }
        drop(send);
        for (result, count, diagnostic) in receive {
            session.submit(result.unwrap()).unwrap();
            diagnostics.merge(&diagnostic).unwrap();
            completed += count;
            if last.elapsed() > Duration::from_secs(5) {
                eprintln!(
                    "{completed} points; {diagnostics}; {:.1}s",
                    started.elapsed().as_secs_f64()
                );
                last = Instant::now();
            }
        }
    });
    let seconds = started.elapsed().as_secs_f64();
    let estimate = session.estimate().unwrap();
    let shift_estimates = session.complete_shift_estimates().unwrap();
    assert!(estimate.production_complete);
    assert_eq!(estimate.orders, [-4, -3, -2, -1, 0]);
    let target: serde_json::Value =
        serde_json::from_str(include_str!("../../../examples/targets/double_box.json")).unwrap();
    let differences = estimate
        .mean
        .iter()
        .enumerate()
        .map(|(index, value)| value - target["coefficients"][index]["re"].as_f64().unwrap())
        .collect::<Vec<_>>();
    eprintln!(
        "{strategy} complete vector: {estimate:?}; {diagnostics}; {seconds:.3}s, load{load_seconds:.3}s"
    );
    let report = serde_json::json!({
        "content_id": kernels.content_id(), "strategy": strategy, "settings": settings,
        "load_seconds": load_seconds, "integration_seconds": seconds, "evaluations": completed,
        "diagnostics": diagnostics, "estimate": estimate,
        "worker_count": worker_count, "same_shift_totals": shift_estimates,
        "replay_ordering": "fixed sector ownership; increasing shift and lattice point order within each sector independent of worker count",
        "historical_target": target, "differences_from_historical_target": differences,
        "comparison_status": "historical target uncertainty unavailable; differences are diagnostic, not certified discrepancies"
    });
    std::fs::write(
        evidence.join(format!(
            "double-box-integral-{strategy}-{points}x{shifts}-seed{seed}-workers{worker_count}.json"
        )),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "independent exact pole check against the preserved double-box artifact"]
fn double_box_leading_pole_is_an_exact_total_derivative() {
    use symbolica::{
        atom::{Atom, AtomCore, AtomView},
        id::Pattern,
    };
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/probes/double-box-taylor-factored.fsd");
    let artifact: serde_json::Value =
        serde_json::from_slice(&std::fs::read(evidence).unwrap()).unwrap();
    assert_eq!(artifact["payload"]["orders"][0], -4);
    let parse = |value: &serde_json::Value| {
        Atom::parse(
            value.as_str().unwrap(),
            "double_box_pole",
            Default::default(),
        )
        .unwrap()
    };
    assert!(parse(&artifact["payload"]["exact"][0]).is_zero());
    let sectors = artifact["payload"]["sectors"].as_array().unwrap();
    let coefficient = sectors
        .iter()
        .map(|sector| parse(&sector["coefficients"][0]))
        .sum::<Atom>();
    let x = parse(&sectors[0]["parameters"][0]);
    let y = parse(&sectors[0]["parameters"][3]);
    let AtomView::Var(x_symbol) = x.as_view() else {
        panic!("coordinate is not a symbol")
    };
    // The complete leading coefficient is d/dx of this rational function.
    // The primitive vanishes at both endpoints for every remaining coordinate,
    // proving its integral is zero without a fitted numerical target.
    let primitive = Atom::num(9)
        * x.pow(Atom::num(2))
        * ((Atom::one() + &x * (Atom::one() + &y)).pow(Atom::num(-4))
            - (&x + Atom::one() + &y).pow(Atom::num(-4)));
    assert!(
        (primitive.derivative(x_symbol.get_symbol()) - &coefficient)
            .together()
            .is_zero()
    );
    for endpoint in [0, 1] {
        assert!(
            primitive
                .replace(Pattern::Literal(x.clone()))
                .with(Atom::num(endpoint))
                .together()
                .is_zero()
        );
    }
    // Existing Symbolica rational integration is the reusable CAS capability;
    // keep this as an independent probe, not a production special-case rule.
    use symbolica::domains::{integer::Z, rational::Q};
    let AtomView::Var(y_symbol) = y.as_view() else {
        panic!("coordinate is not a symbol")
    };
    let rational = coefficient
        .try_to_rational_polynomial::<_, _, u16>(
            &Q,
            &Z,
            Some(std::sync::Arc::new(vec![
                x_symbol.get_symbol().into(),
                y_symbol.get_symbol().into(),
            ])),
        )
        .unwrap();
    let native = rational.integrate(0);
    assert!(native.logarithmic_parts.is_empty());
    let primitive = native
        .rational_parts
        .iter()
        .map(|part| part.to_expression())
        .sum::<Atom>();
    assert!(
        (primitive.derivative(x_symbol.get_symbol()) - coefficient)
            .together()
            .is_zero()
    );
    let upper = primitive
        .replace(Pattern::Literal(x.clone()))
        .with(Atom::one());
    let lower = primitive.replace(Pattern::Literal(x)).with(Atom::Zero);
    assert!((upper - lower).together().is_zero());
}
