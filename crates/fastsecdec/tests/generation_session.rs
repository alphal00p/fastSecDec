use fastsecdec::{
    generation::{
        CoefficientExpansionMethod, GeneratedIntegral, GenerationOptions, GenerationProgress,
        GenerationSession, GenerationSessionState, generate,
    },
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{atom::Atom, parse, symbol};

fn input(domain: ParametricDomain) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("session_x"), symbol!("session_y")],
        symbol!("session_eps"),
        domain,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-1+session_eps"), parse!("session_eps")],
            vec![PolynomialFactor::new(
                parse!("session_x+session_y"),
                if domain == ParametricDomain::ProjectiveSimplex {
                    parse!("-1-2*session_eps")
                } else {
                    parse!("-1-session_eps")
                },
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap()
}
fn equal(left: &GeneratedIntegral, right: &GeneratedIntegral) {
    assert_eq!(left.orders(), right.orders());
    assert_eq!(left.exact_coefficients(), right.exact_coefficients());
    assert_eq!(
        serde_json::to_value(fastsecdec::kernel::PortableMetadata::from_native(
            left.metadata()
        ))
        .unwrap(),
        serde_json::to_value(fastsecdec::kernel::PortableMetadata::from_native(
            right.metadata()
        ))
        .unwrap()
    );
    assert_eq!(left.sectors().len(), right.sectors().len());
    for (a, b) in left.sectors().iter().zip(right.sectors()) {
        assert_eq!(a.parameters(), b.parameters());
        assert_eq!(a.coefficients(), b.coefficients());
        assert_eq!(a.cancellation_terms(), b.cancellation_terms());
        assert_eq!(a.endpoint_profiles(), b.endpoint_profiles());
    }
}

#[test]
fn cooperative_units_resume_without_repeating_coefficients() {
    for method in [
        CoefficientExpansionMethod::Physical,
        CoefficientExpansionMethod::NativeNamed,
    ] {
        for domain in [
            ParametricDomain::UnitCube,
            ParametricDomain::ProjectiveSimplex,
        ] {
            let input = input(domain);
            let mut options = GenerationOptions::default();
            options.coefficient_expansion.method = method;
            let serial = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
            for budget in [1, 3, usize::MAX] {
                let mut session = GenerationSession::new(input.clone(), options.clone());
                assert_eq!(session.completed_units(), 0);
                assert_eq!(session.snapshot().elapsed_seconds, 0.0);
                assert!(session.take_result().is_none());
                let mut expansions = 0;
                while !session.is_complete() {
                    let before = session.completed_units();
                    let status=session.step(budget,|event|{
                        if matches!(event,GenerationProgress::PhaseTiming{phase:fastsecdec::generation::GenerationPhase::Laurent,..}|GenerationProgress::PhaseTiming{phase:fastsecdec::generation::GenerationPhase::CoefficientExpansion,..}) {expansions+=1;}
                        ControlFlow::Break(())
                    }).unwrap();
                    assert!(session.completed_units() > before);
                    assert!(session.completed_units() - before <= budget);
                    assert!(matches!(
                        status,
                        GenerationSessionState::Paused
                            | GenerationSessionState::Complete
                            | GenerationSessionState::Pending
                    ));
                    if !session.is_complete() {
                        assert!(session.take_result().is_none());
                    }
                }
                assert_eq!(expansions, session.completed_representatives());
                let completed = session.completed_units();
                assert_eq!(
                    session
                        .step(10, |_| panic!("complete session emitted work"))
                        .unwrap(),
                    GenerationSessionState::Complete
                );
                assert_eq!(session.completed_units(), completed);
                equal(&serial, &session.take_result().unwrap());
                assert!(session.take_result().is_none());
            }
        }
    }
}

#[test]
fn one_large_step_and_empty_input_have_complete_only_results() {
    let input = input(ParametricDomain::UnitCube);
    let options = GenerationOptions::default();
    let serial = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut session = GenerationSession::new(input, options.clone());
    assert!(session.step(0, |_| ControlFlow::Continue(())).is_err());
    assert_eq!(session.completed_units(), 0);
    assert_eq!(
        session
            .step(usize::MAX, |_| ControlFlow::Continue(()))
            .unwrap(),
        GenerationSessionState::Complete
    );
    equal(&serial, &session.take_result().unwrap());
    let empty = ParametricIntegrand::new(
        vec![symbol!("empty_session_x")],
        symbol!("empty_session_eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    let serial = generate(&empty, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut session = GenerationSession::new(empty, options);
    assert_eq!(
        session
            .step(usize::MAX, |_| ControlFlow::Continue(()))
            .unwrap(),
        GenerationSessionState::Complete
    );
    equal(&serial, &session.take_result().unwrap());
}

#[test]
fn geometry_failure_is_terminal_and_never_exposes_partial_integral() {
    let input = input(ParametricDomain::UnitCube);
    let mut options = GenerationOptions::default();
    options.decomposition.max_sectors = 0;
    let mut session = GenerationSession::new(input, options);
    assert!(
        session
            .step(usize::MAX, |_| ControlFlow::Continue(()))
            .is_err()
    );
    assert!(!session.is_complete());
    assert!(session.take_result().is_none());
    assert!(session.step(1, |_| ControlFlow::Continue(())).is_err());
}

#[test]
fn exact_folded_result_retains_collision_free_chart_metadata() {
    let source = symbol!("fastsecdec::sector_0::t0");
    let input = ParametricIntegrand::new(
        vec![source, symbol!("session_unused")],
        symbol!("session_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-1+session_eps"), Atom::Zero],
            vec![],
        )],
    )
    .unwrap();
    let options = GenerationOptions::default();
    let serial = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut session = GenerationSession::new(input, options);
    while !session.is_complete() {
        session.step(1, |_| ControlFlow::Continue(())).unwrap();
    }
    assert!(session.snapshot().elapsed_seconds > 0.0);
    assert_eq!(
        session.snapshot().elapsed_seconds,
        session.snapshot().timings.total_seconds
    );
    let generated = session.take_result().unwrap();
    equal(&serial, &generated);
    assert!(generated.sectors().is_empty());
    assert_eq!(generated.orders(), &[-1, 0]);
    assert_eq!(generated.exact_coefficients(), &[Atom::one(), Atom::Zero]);
    assert!(!generated.metadata().charts().is_empty());
    for chart in generated.metadata().charts() {
        assert!(!chart.coordinates().target_parameters().contains(&source));
        assert_eq!(chart.kernel_sector(), None);
    }
}
