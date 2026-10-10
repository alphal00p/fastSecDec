use super::*;
#[test]
fn complete_rational_bubble_matches_native_oneloop() {
    use crate::integration::{
        CoefficientComponent::{Imag, Real},
        IntegrationProblem, Periodization, QmcSession, QmcSettings, RuleSource, SectorSpec,
    };
    let whole = std::time::Instant::now();
    let (x, t, eps) = symbol!(
        "regular_bubble_qmc::x",
        "regular_bubble_qmc::t",
        "regular_bubble_qmc::eps"
    );
    let f = Atom::num(3) - Atom::num(16) * Atom::var(x) * (Atom::one() - Atom::var(x));
    let owner = solve(
        &source(x, eps, f, -Atom::var(eps), Atom::one()),
        Default::default(),
    );
    assert_eq!(owner.cells().len(), 3);
    assert_eq!(owner.request().signed_factors().len(), 1);
    let fiber = RegularizedFiber::admit(owner, BTreeMap::new(), t, Limits::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(fiber.charts().len(), 6);
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_subtractions_per_axis: 2,
                max_order: 1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
    let series = bound
        .expression()
        .series(eps, 0, SeriesDepth::absolute(2))
        .unwrap();
    let coefficients = [
        series.coefficient(0.into()).unwrap(),
        series.coefficient(1.into()).unwrap(),
    ];
    let mut evaluator = restored(&coefficients, &[Atom::var(t)], bound.functions().clone());
    // Raw F^(-eps) orders0/1 become B0 orders-1/0 in the existing HEPKit
    // rGamma convention. No unclassified Gamma prefactor enters admission.
    let problem = IntegrationProblem::new_with_components(
        "native-rational-bubble-B0-16-3-3-v1".into(),
        vec![-1, -1, 0, 0],
        vec![Real, Imag, Real, Imag],
        vec![SectorSpec {
            id: 0,
            dimension: 1,
        }],
        vec![0.; 4],
    )
    .unwrap();
    let settings = QmcSettings {
        points: 16384,
        shifts: 16,
        seed: 202610106101,
        package_points: 1024,
        periodization: Periodization::Korobov3,
        rule: RuleSource::Kuo,
    };
    let mut session = QmcSession::democratic(problem.clone(), settings.clone()).unwrap();
    let setup_seconds = whole.elapsed().as_secs_f64();
    let sampling = std::time::Instant::now();
    let mut outputs = [Complex::new(0., 0.); 2];
    let mut evaluated = 0u64;
    while let Some(task) = session.next_work().unwrap() {
        let value = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task, |point, out| {
                // Native worker applies the periodization weight exactly once.
                evaluator.evaluate(&[Complex::new(point[0], 0.)], &mut outputs);
                out.copy_from_slice(&[outputs[0].re, outputs[0].im, outputs[1].re, outputs[1].im]);
                if !out.iter().all(|v| v.is_finite()) {
                    return Err("nonfinite native Laurent vector");
                }
                evaluated += 1;
                Ok::<_, &str>(())
            })
            .unwrap();
        session.submit(value).unwrap();
    }
    let sampling_seconds = sampling.elapsed().as_secs_f64();
    assert!(session.is_complete());
    assert_eq!(evaluated, 16384 * 16);
    let estimate = session.estimate().unwrap();
    let mut reference = [Complex::new(0., 0.); 3];
    oneloop::evaluate_with_backend(
        oneloop::ScalarIntegral::B0,
        &[16., 3., 3., 1.].map(|v| Complex::new(v, 0.)),
        &mut reference,
        oneloop::EvaluationBackend::Expression,
    )
    .unwrap();
    let expected = [
        reference[1].re,
        reference[1].im,
        reference[0].re,
        reference[0].im,
    ];
    let mut errors = Vec::new();
    let mut se = Vec::new();
    let mut accepted = true;
    for (i, reference_value) in expected.iter().enumerate() {
        let error = (estimate.mean[i] - reference_value).abs();
        let uncertainty = estimate.covariance_of_mean[i * 4 + i].sqrt();
        errors.push(error);
        se.push(uncertainty);
        accepted &= error <= 6. * uncertainty + 2e-10;
    }
    let report = serde_json::json!({"scope":"native rational bridge saved Laurent program; complete raw fiber, not GeneratedIntegral or general resolver", "kinematics":{"s":16,"m1_squared":3,"m2_squared":3,"mu_squared":1},"original_causal_factor_count":1,"native_cells":3,"certified_half_charts":6,"common_strip":"Re(epsilon)<1","raw_to_master_orders":[[0,-1],[1,0]],"reference_backend":"native OneLOop Expression B0; existing HEPKit rGamma normalization through finite", "reference_vector":expected,"reference_complex":[[reference[0].re,reference[0].im],[reference[1].re,reference[1].im],[reference[2].re,reference[2].im]],"problem":problem,"settings":settings,"estimate":estimate,"contributions":session.contributions().unwrap(),"snapshot":session.snapshot().unwrap(),"evaluated":evaluated,"absolute_errors":errors,"standard_errors":se,"acceptance":"all components |error|<=6*native_SE+2e-10; predeclared single allocation, no repeat","accepted":accepted,"setup_seconds":setup_seconds,"sampling_seconds":sampling_seconds,"whole_seconds":whole.elapsed().as_secs_f64(),"precision":"native complex f64 Laurent evaluator restored from exact owner bincode","caller_workers":1});
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(accepted, "native bubble OneLOop comparison failed");
}
