use super::*;
use fastsecdec::{
    kernel::{EvaluatorBackend, KernelLoadOptions, KernelSet, ProgramRecipe},
    parametric::*,
    threshold::generation::Stage,
};
use symbolica::{
    atom::{Atom, AtomCore},
    symbol,
};
fn fixture() -> (ParametricIntegrand, PreparationOptions) {
    let (x, eps, t) = symbol!(
        "hepkit_threshold_test::x",
        "hepkit_threshold_test::eps",
        "hepkit_threshold_test::unit"
    );
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::var(eps).pow(-1),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::var(x) - Atom::num((1, 3)),
                    -Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let mut options = PreparationOptions::new(t);
    options.generation.max_order = 0;
    options.compilation.backend = EvaluatorBackend::Eager;
    (input, options)
}
#[test]
fn inert_owner_and_native_pause_reverify_drop_and_local_compilation() {
    let (input, options) = fixture();
    let mut work = Work::new(options, 1 << 26);
    assert!(!work.initialized());
    assert!(work.storage.is_none());
    assert!(work.receipt.is_none());
    assert!(work.compile_one().is_err());
    work.initialize(input).unwrap();
    let Input::Parametric(input) = work.input.as_ref().unwrap() else {
        panic!("expected parametric fixture")
    };
    let input_weak = Arc::downgrade(input);
    assert!(
        !work
            .prepare(|p| if p.receipts.raw.is_some() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            })
            .unwrap()
    );
    assert!(work.checkpoint.is_some());
    assert!(work.receipt.is_none());
    assert_eq!(work.next, 0);
    let mut stages = Vec::new();
    assert!(
        work.prepare(|p| {
            stages.push(p.stage);
            ControlFlow::Continue(())
        })
        .unwrap()
    );
    assert!(!stages.contains(&Stage::Solve));
    assert!(stages.contains(&Stage::Verify));
    assert!(input_weak.upgrade().is_none());
    let id = work
        .receipt
        .as_ref()
        .unwrap()
        .publication
        .prepared_identity
        .clone();
    assert!(work.finish().is_err());
    let jobs = work.jobs();
    assert!(jobs > 1);
    for index in 0..jobs {
        assert_eq!(work.next, index);
        work.compile_one().unwrap();
    }
    let (writer, catalogue) = work.finish().unwrap();
    writer.sync_all().unwrap();
    assert_eq!(
        catalogue
            .recipe(ProgramRecipe::ThresholdV1)
            .unwrap()
            .content_id,
        id
    );
    let bytes = fs::read(work.storage.as_ref().unwrap().path().join("integral.fsd")).unwrap();
    drop(work);
    drop(writer);
    let mut kernels =
        KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: true }).unwrap();
    assert!(
        kernels
            .threshold_metadata()
            .unwrap()
            .full_original_scope()
            .unwrap()
    );
    assert!(kernels.runtime_parameters().is_empty());
    // Existing native QMC is a separate test action; work preparation/compilation
    // never creates a numerical session.
    use fastsecdec::{
        integration::{Periodization, QmcSession, QmcSettings, RuleSource},
        results::{KernelResultManifest, ResultScope},
    };
    let problem = KernelResultManifest::integration_problem_from_kernels(
        &kernels,
        &ResultScope::FullIntegral,
        "hepkit-threshold-work",
    )
    .unwrap();
    let mut qmc = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 4096,
            shifts: 8,
            seed: 202610108001,
            package_points: 1024,
            periodization: Periodization::Korobov3,
            rule: RuleSource::Kuo,
        },
    )
    .unwrap();
    while let Some(task) = qmc.next_work().unwrap() {
        let id = task.sector_id();
        let result = qmc
            .worker_context(id)
            .unwrap()
            .evaluate(task, |p, o| {
                kernels.sectors_mut()[id as usize].evaluate(p, o)
            })
            .unwrap();
        qmc.submit(result).unwrap();
    }
    let result = qmc.estimate().unwrap();
    let expected = [
        1.,
        0.,
        1. - (1f64 / 3.).ln() / 3. - 2. * (2f64 / 3.).ln() / 3.,
        std::f64::consts::PI / 3.,
    ];
    for (i, expected) in expected.iter().enumerate() {
        assert!(
            (result.mean[i] - expected).abs()
                < 8. * result.covariance_of_mean[4 * i + i].sqrt() + 1e-8,
            "{result:?}"
        );
    }
}
#[test]
fn cancellation_before_raw_restarts_only_owned_empty_preparation() {
    let (input, options) = fixture();
    let mut work = Work::new(options, 1 << 26);
    work.initialize(input).unwrap();
    assert!(!work.prepare(|_| ControlFlow::Break(())).unwrap());
    assert!(work.checkpoint.is_none());
    let first = work.root.clone().unwrap();
    assert!(first.read_dir().unwrap().next().is_none());
    assert!(work.prepare(|_| ControlFlow::Continue(())).unwrap());
    assert_ne!(work.root.as_ref(), Some(&first));
    assert!(first.exists());
}
#[test]
fn unsupported_native_options_never_publish_partial_archive() {
    let (input, mut options) = fixture();
    options.threshold.threshold_cells = Some(vec![0]);
    let mut work = Work::new(options, 1 << 26);
    work.initialize(input).unwrap();
    assert!(work.prepare(|_| ControlFlow::Continue(())).is_err());
    assert!(work.receipt.is_none());
    assert!(work.archive.is_none());
    assert!(work.finish().is_err());
}

#[test]
fn represented_request_identity_is_native_issued_and_survives_raw_resume() {
    use fastsecdec::threshold::{gcad::staging::StagedRequest, represented};
    fn at_half(value: Atom) -> (ParametricIntegrand, PreparationOptions) {
        let (input, options) = fixture();
        let x = input.parameters()[0];
        let eps = input.regulator();
        (
            ParametricIntegrand::new(
                vec![x],
                eps,
                ParametricDomain::UnitCube,
                vec![ParametricTerm::new(
                    Atom::var(eps).pow(-1),
                    vec![Atom::Zero],
                    vec![
                        PolynomialFactor::new(
                            Atom::var(x) - value,
                            -Atom::var(eps),
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Causal),
                    ],
                )],
            )
            .unwrap(),
            options,
        )
    }
    let (float, mut options) = at_half(Atom::num(0.5f64));
    let ordinary_float_id = fastsecdec::generation::source_identity(&float, &[], &[]).unwrap();
    options.represented = Some((
        represented::NumericalMeaning::RepresentedValues,
        represented::Limits::default(),
    ));
    let mut work = Work::new(options, 1 << 26);
    work.initialize(float).unwrap();
    assert!(work.initialized());
    assert!(
        work.source.is_none(),
        "only native request progress may establish identity"
    );
    assert!(
        !work
            .prepare(|p| if p.receipts.raw.is_some() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            })
            .unwrap()
    );
    let source = work.source.clone().unwrap();
    assert_ne!(source, ordinary_float_id);
    let request = work
        .progress
        .as_ref()
        .unwrap()
        .receipts
        .request
        .as_ref()
        .unwrap();
    assert_eq!(request.source_identity, source);
    let staged = StagedRequest::read(work.root.as_ref().unwrap(), request, 1 << 26).unwrap();
    assert!(
        !staged
            .request()
            .represented_input()
            .unwrap()
            .conversions()
            .is_empty()
    );
    assert_eq!(staged.request().source_identity().unwrap(), source);
    let saved_source = work.source.take();
    assert!(
        work.prepare(|_| ControlFlow::Continue(()))
            .unwrap_err()
            .to_string()
            .contains("native-issued")
    );
    assert!(work.receipt.is_none());
    work.source = saved_source;
    let mut stages = Vec::new();
    assert!(
        work.prepare(|p| {
            stages.push(p.stage);
            ControlFlow::Continue(())
        })
        .unwrap()
    );
    assert!(!stages.contains(&Stage::Solve));
    assert_eq!(
        work.receipt.as_ref().unwrap().publication.source_identity,
        source
    );
    let (exact, exact_options) = at_half(Atom::num((1, 2)));
    let exact_id = fastsecdec::generation::source_identity(&exact, &[], &[]).unwrap();
    let mut exact_work = Work::new(exact_options, 1 << 26);
    exact_work.initialize(exact).unwrap();
    assert!(exact_work.prepare(|_| ControlFlow::Continue(())).unwrap());
    assert_eq!(exact_work.source.as_deref(), Some(exact_id.as_str()));
    assert_ne!(source, exact_id);
    for active in [&mut work, &mut exact_work] {
        while active.next < active.jobs() {
            active.compile_one().unwrap();
        }
        active.finish().unwrap().0.sync_all().unwrap();
    }
    let load = |w: &Work| {
        KernelSet::from_bytes_with_options(
            &fs::read(w.storage.as_ref().unwrap().path().join("integral.fsd")).unwrap(),
            KernelLoadOptions { validate: true },
        )
        .unwrap()
    };
    let mut floated = load(&work);
    let mut exact = load(&exact_work);
    assert_ne!(floated.content_id(), exact.content_id());
    assert_eq!(floated.exact_coefficients(), exact.exact_coefficients());
    assert_eq!(floated.sectors().len(), exact.sectors().len());
    for (a, b) in floated.sectors_mut().iter_mut().zip(exact.sectors_mut()) {
        let mut left = vec![0.; a.output_count()];
        let mut right = vec![0.; b.output_count()];
        a.evaluate(&[0.271], &mut left).unwrap();
        b.evaluate(&[0.271], &mut right).unwrap();
        assert_eq!(left, right);
    }
}

#[test]
fn mixed_live_native_request_is_rejected_without_replacing_authority() {
    let (input, options) = fixture();
    let mut work = Work::new(options, 1 << 26);
    work.initialize(input).unwrap();
    assert!(
        !work
            .prepare(|p| if p.receipts.raw.is_some() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            })
            .unwrap()
    );
    let trusted = work.source.clone();
    let mut wrong = work.progress.clone().unwrap();
    wrong.receipts.request.as_mut().unwrap().source_identity = "foreign".into();
    assert!(accept_native_source(&mut work.source, &wrong).is_err());
    assert_eq!(work.source, trusted);
    let mut wrong_raw = work.progress.clone().unwrap();
    wrong_raw
        .receipts
        .raw
        .as_mut()
        .unwrap()
        .request
        .source_identity = "foreign".into();
    assert!(accept_native_source(&mut work.source, &wrong_raw).is_err());
    assert_eq!(work.source, trusted);
}
