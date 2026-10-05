//! Actual-library host/Wasm smoke; all generation, evaluation and QMC stay native.
use fastsecdec::{
    Atom,
    generation::{
        CoefficientExpansionMethod, CoefficientExpansionOptions, GenerationOptions, generate,
    },
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec},
    kernel::{KernelSet, ReplayPolicy},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = ParametricIntegrand::new(
        vec![symbol!("portable_actual::x")],
        symbol!("portable_actual::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("(2+3𝑖)*gamma(1+portable_actual::eps)"),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+portable_actual::x"),
                parse!("portable_actual::eps-1"),
                FactorRole::Singularity,
            )],
        )],
    )?;
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 2,
            coefficient_expansion: CoefficientExpansionOptions {
                method: CoefficientExpansionMethod::NativeNamed,
                ..Default::default()
            },
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )?;
    let kernels = generated.compile()?;
    drop(generated);
    assert_eq!(kernels.orders(), [0, 0, 1, 1, 2, 2]);
    assert_eq!(kernels.sectors().len(), 1);
    let bytes = kernels.to_bytes()?;
    let restored = KernelSet::from_bytes(&bytes)?;
    assert_eq!(restored.content_id(), kernels.content_id());
    assert_eq!(restored.artifact_bytes()?, bytes);
    let policy = ReplayPolicy {
        minimum_bits: 256,
        ..Default::default()
    };
    let mut point_rows = Vec::new();
    for point in [0.25, 1e-8] {
        let mut original = kernels.evaluation_context(0, policy.clone())?;
        let mut cold = restored.evaluation_context(0, policy.clone())?;
        let mut expected = [0.0; 6];
        let mut actual = [0.0; 6];
        let report = original.evaluate_weighted(&[point], 2.0, &mut expected)?;
        let replay = cold.evaluate_weighted(&[point], 2.0, &mut actual)?;
        assert!(report.weighted_check && replay.weighted_check);
        assert!(report.precision.rescued && replay.precision.rescued);
        assert!(report.precision.bits >= 512 && replay.precision.bits >= 512);
        assert!(actual.iter().all(|x| x.is_finite()));
        for (a, b) in actual.iter().zip(expected) {
            assert!((a - b).abs() < 1e-11);
        }
        assert!((actual[0] - 4.0 / (1.0 + point)).abs() < 1e-11);
        assert!((actual[1] - 6.0 / (1.0 + point)).abs() < 1e-11);
        point_rows
            .push(serde_json::json!({"point":point,"values":actual,"bits":replay.precision.bits}));
    }
    let problem = IntegrationProblem::new_with_components(
        restored.content_id().into(),
        restored.orders().to_vec(),
        restored.components().to_vec(),
        vec![SectorSpec {
            id: 0,
            dimension: 1,
        }],
        restored.exact_coefficients().to_vec(),
    )?;
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 1024,
            shifts: 4,
            seed: 73,
            package_points: 256,
            ..Default::default()
        },
    )?;
    let mut context = restored.evaluation_context(0, ReplayPolicy::default())?;
    let mut worker = session.worker_context(0)?;
    while let Some(task) = session.next_work()? {
        let result = worker.evaluate_weighted(task, |point, weight, output| {
            context.evaluate_weighted(point, weight, output).map(|_| ())
        })?;
        session.submit(result)?;
    }
    assert!(session.is_complete());
    assert_eq!(session.snapshot()?.completed_points, 4096);
    assert_eq!(session.complete_shift_estimates()?.len(), 4);
    let estimate = session.estimate()?;
    assert_eq!(estimate.mean.len(), 6);
    assert_eq!(estimate.covariance_of_mean.len(), 36);
    assert!(
        estimate
            .mean
            .iter()
            .chain(&estimate.covariance_of_mean)
            .all(|x| x.is_finite())
    );
    assert!((estimate.mean[0] - 2.0 * 2.0_f64.ln()).abs() < 1e-5);
    assert!((estimate.mean[1] - 3.0 * 2.0_f64.ln()).abs() < 1e-5);
    println!(
        "{}",
        serde_json::json!({
            "complete":true,"portable":cfg!(feature="portable"),
            "orders":restored.orders(),"point_rows":point_rows,
            "artifact_roundtrip":true,"qmc_points":4096,"complete_shifts":4,
            "mean":estimate.mean,"covariance_of_mean":estimate.covariance_of_mean,
        })
    );
    Ok(())
}
