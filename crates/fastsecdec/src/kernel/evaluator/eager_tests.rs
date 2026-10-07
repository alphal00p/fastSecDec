use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSession, CompilationSettings, EvaluatorBackend, KernelSet, ReplayPolicy},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{atom::Atom, parse, symbol};

#[test]
fn explicit_eager_retains_native_vector_program_runtime_binding_and_resumable_compilation() {
    let x = symbol!("eager_session::x");
    let y = symbol!("eager_session::y");
    let p = symbol!("eager_session::p");
    let eps = symbol!("eager_session::eps");
    let input = ParametricIntegrand::new(
        vec![x, y],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("eager_session::p*(1+2𝑖)"),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("eager_session::x+eager_session::y"),
                parse!("-1-eager_session::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = Arc::new(
        generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap(),
    );
    let settings = CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    };
    let synchronous = generated
        .compile_with_settings_parameters_and_progress(Default::default(), &[p], settings, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    let mut session =
        CompilationSession::new(generated, vec![p], Default::default(), settings).unwrap();
    assert_eq!(session.completed_sectors(), 0);
    assert!(session.take_result().is_none());
    assert!(!session.step(1, |_| ControlFlow::Break(())).unwrap());
    assert_eq!(session.completed_sectors(), 0);
    let mut steps = 0;
    while !session.is_complete() {
        let before = session.completed_sectors();
        session.step(1, |_| ControlFlow::Continue(())).unwrap();
        assert!(session.completed_sectors() <= before + 1);
        steps += 1;
    }
    assert_eq!(steps, session.total_sectors() + 1);
    let template = session.take_result().unwrap();
    assert_eq!(
        template.artifact_bytes().unwrap(),
        synchronous.artifact_bytes().unwrap()
    );
    assert!(
        template
            .sectors()
            .iter()
            .all(|s| s.statistics().backend == "symbolica_interpreter"
                && s.statistics().symjit_ir_bytes.is_none())
    );
    let mut kernels = KernelSet::from_bytes(template.artifact_bytes().unwrap()).unwrap();
    assert_eq!(
        kernels.compilation_settings().backend,
        EvaluatorBackend::Eager
    );
    kernels
        .bind_parameters(&BTreeMap::from([(p, 2.0)]))
        .unwrap();
    let mut other = kernels.try_clone().unwrap();
    other.bind_parameters(&BTreeMap::from([(p, 3.0)])).unwrap();
    assert_ne!(kernels.content_id(), other.content_id());
    assert_eq!(
        kernels.artifact_bytes().unwrap(),
        other.artifact_bytes().unwrap()
    );
    for index in 0..kernels.sectors().len() {
        let dimension = kernels.sectors()[index].dimension();
        let points = vec![0.37; 7 * dimension];
        let mut a = kernels
            .evaluation_context(index, ReplayPolicy::default())
            .unwrap();
        let mut b = other
            .evaluation_context(index, ReplayPolicy::default())
            .unwrap();
        let mut left = vec![0.0; 7 * a.output_count()];
        let mut right = left.clone();
        a.evaluate_weighted_batch(&points, &[1.0; 7], &mut left)
            .unwrap();
        b.evaluate_weighted_batch(&points, &[1.0; 7], &mut right)
            .unwrap();
        assert_eq!(a.evaluation_metrics().f64.matrix_invocations, 0);
        assert!(a.evaluation_metrics().f64.calls >= 7);
        for (a, b) in left.iter().zip(right) {
            assert!((b - 1.5 * a).abs() < 1e-12 * (1.0 + b.abs()));
        }
    }
}

#[test]
fn auto_settings_serialization_preserves_historical_policy_shape() {
    let settings = CompilationSettings::default();
    let json = serde_json::to_value(settings).unwrap();
    assert!(json.get("backend").is_none());
    assert_eq!(
        serde_json::from_value::<CompilationSettings>(json).unwrap(),
        settings
    );
}
