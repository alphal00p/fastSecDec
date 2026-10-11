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
    let input_weak = Arc::downgrade(work.input.as_ref().unwrap());
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
