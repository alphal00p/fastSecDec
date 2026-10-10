use fastsecdec::{
    Atom,
    contour::{
        ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    generation::{GenerationOptions, generate, source_identity},
    integration::{IntegrationProblem, VectorEstimate},
    kernel::{
        CompilationSettings, EvaluatorBackend, KernelSet, WeightedEvaluationContext,
        indexed::{ProgramArchiveReader, ProgramArchiveWriter},
    },
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::{KernelResultManifest, ResultScope},
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, io::Cursor, ops::ControlFlow};
use symbolica::symbol;

pub const CONSTRUCTIONS: [DynamicConstruction; 2] = [
    DynamicConstruction::Polynomial,
    DynamicConstruction::SignAware,
];
pub const CASES: [(i64, f64); 2] = [(1, 1.), (40000, 1e-5)];

pub fn cube(scale: i64) -> ParametricIntegrand {
    let [x, y, z, eps] = [
        symbol!("sampling_cube::x"),
        symbol!("sampling_cube::y"),
        symbol!("sampling_cube::z"),
        symbol!("sampling_cube::eps"),
    ];
    let f = Atom::num(scale) * (1 - 2 * Atom::var(x)) * (1 + Atom::var(y)) * (1 + Atom::var(z));
    ParametricIntegrand::new(
        vec![x, y, z],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() / Atom::var(eps),
            vec![Atom::Zero; 3],
            vec![
                PolynomialFactor::new(f, -Atom::var(eps), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

pub fn cube_reference(scale: i64, order: i32, part: CoefficientComponent) -> f64 {
    match (order, part) {
        (-1, CoefficientComponent::Real) => 1.,
        (-1, CoefficientComponent::Imag) => 0.,
        (0, CoefficientComponent::Real) => 3. - 4. * 2_f64.ln() - (scale as f64).ln(),
        (0, CoefficientComponent::Imag) => std::f64::consts::PI / 2.,
        _ => panic!("unexpected coefficient {order}, {part:?}"),
    }
}

pub fn mode(construction: DynamicConstruction, cap: f64) -> ContourMode {
    ContourMode::Dynamical {
        safety_fraction: 0.8,
        lambda_cap: cap,
        displacement_cap: 1.,
        construction,
    }
}

/// Both native programs are appended independently, then released. Restoration
/// below selects only the requested directory and each actual sector record.
pub fn archive(input: &ParametricIntegrand) -> Vec<u8> {
    let recipes = CONSTRUCTIONS.map(|c| mode(c, 1.).program_recipe());
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        source_identity(input, &[], &[]).unwrap(),
        recipes,
    )
    .unwrap();
    for recipe in recipes {
        let generated = generate(
            input,
            &GenerationOptions {
                program_recipe: recipe,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let kernels = generated
            .compile_with_settings(CompilationSettings {
                backend: EvaluatorBackend::Eager,
                horner_iterations: 0,
                ..Default::default()
            })
            .unwrap();
        writer.append_kernels(recipe, &kernels).unwrap();
    }
    writer.finish().unwrap().0.into_inner()
}

fn prepare(kernels: &mut KernelSet, construction: DynamicConstruction, cap: f64) {
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: mode(construction, cap),
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 3,
                },
            },
        )
        .unwrap();
    assert!(
        kernels
            .validate_integration_readiness(&ResultScope::FullIntegral)
            .is_err()
    );
    for chart in kernels.contour_validation_charts() {
        for x in [0.25, 0.5, 0.75] {
            kernels
                .validate_contour_point(chart.chart_index, &vec![x; chart.dimension], true)
                .unwrap();
        }
    }
    let pilot = kernels.finish_contour_pilot().unwrap();
    assert!(pilot.pilot_complete && pilot.checked_arguments > 0 && pilot.maximum_bits >= 96);
    kernels
        .validate_integration_readiness(&ResultScope::FullIntegral)
        .unwrap();
    // Exercise production checking on the nonlinear envelope once, then retain
    // the accepted pilot while avoiding per-sample ball work in the samplers.
    let mut output = vec![0.; kernels.orders().len()];
    for sector in kernels.sectors_mut() {
        sector
            .evaluate(&vec![0.37; sector.dimension()], &mut output)
            .unwrap();
    }
    assert!(
        kernels
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments
            > 0
    );
    let identity = kernels.content_id().to_owned();
    kernels
        .set_contour_validation(ContourValidationOptions {
            policy: ContourValidation::Pilot,
            pilot_points: 3,
        })
        .unwrap();
    assert_eq!(kernels.content_id(), identity);
    assert!(kernels.contour_validation_report().unwrap().pilot_complete);
}

pub struct Fixture {
    pub problem: IntegrationProblem,
    pub contexts: Vec<WeightedEvaluationContext>,
}

pub fn restored(bytes: &[u8], construction: DynamicConstruction, cap: f64) -> Fixture {
    let mut reader =
        ProgramArchiveReader::from_reader(Cursor::new(bytes), Default::default()).unwrap();
    assert_eq!(reader.catalogue().recipes.len(), 2);
    let recipe = mode(construction, cap).program_recipe();
    let mut selected = reader.select(recipe).unwrap();
    let mut resident = selected.load_all().unwrap();
    assert_eq!(resident.program_recipe(), recipe);
    prepare(&mut resident, construction, cap);
    let problem = KernelResultManifest::integration_problem_from_kernels(
        &resident,
        &ResultScope::FullIntegral,
        resident.content_id(),
    )
    .unwrap();
    let mut contexts = Vec::new();
    for index in 0..resident.sectors().len() {
        let mut isolated = selected.load_sector(index).unwrap();
        assert_eq!(isolated.program_recipe(), recipe);
        assert_eq!(isolated.sectors().len(), 1);
        assert_eq!(isolated.orders(), resident.orders());
        assert_eq!(isolated.components(), resident.components());
        prepare(&mut isolated, construction, cap);
        let point = vec![0.37; isolated.sectors()[0].dimension()];
        let mut actual = vec![0.; resident.orders().len()];
        let mut expected = actual.clone();
        isolated.sectors_mut()[0]
            .evaluate(&point, &mut actual)
            .unwrap();
        resident.sectors_mut()[index]
            .evaluate(&point, &mut expected)
            .unwrap();
        assert_eq!(
            actual, expected,
            "selected native record differs from resident sector"
        );
        let mut context = isolated.evaluation_context(0, Default::default()).unwrap();
        // Sampling counters start after the deliberately checked preparation point.
        context.take_contour_validation_report();
        contexts.push(context);
    }
    assert!(!contexts.is_empty());
    // Contexts retain their native helper owners independently of the reader,
    // isolated KernelSets, and full resident object, all dropped on return.
    Fixture { problem, contexts }
}

pub fn check_unchecked(contexts: &[WeightedEvaluationContext]) {
    for context in contexts {
        assert_eq!(
            context
                .contour_validation_report()
                .unwrap()
                .checked_arguments,
            0,
            "pilot-only production retained optional causal checks"
        );
    }
}

pub fn check_estimate(
    estimate: &VectorEstimate,
    reference: impl Fn(i32, CoefficientComponent) -> f64,
    error_ceiling: f64,
    label: &str,
) {
    estimate.validate().unwrap();
    assert!(estimate.production_complete);
    assert_eq!(estimate.orders, [-1, -1, 0, 0]);
    assert_eq!(
        estimate.components,
        [
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
            CoefficientComponent::Real,
            CoefficientComponent::Imag
        ]
    );
    assert_eq!(estimate.covariance_of_mean.len(), 16);
    for i in 0..4 {
        let expected = reference(estimate.orders[i], estimate.components[i]);
        let error = estimate.standard_error[i];
        assert!(
            error <= error_ceiling,
            "{label}: imprecise component {i}: sigma={error}"
        );
        assert!(
            (estimate.mean[i] - expected).abs() <= (8. * error).max(2e-6),
            "{label}, component {i}: {} +/- {error}, expected {expected}",
            estimate.mean[i]
        );
        assert!(
            (estimate.covariance_of_mean[5 * i] - error * error).abs()
                <= 1e-12 * (1. + error * error)
        );
        for j in 0..4 {
            assert_eq!(
                estimate.covariance_of_mean[4 * i + j],
                estimate.covariance_of_mean[4 * j + i]
            );
        }
    }
    assert!(
        estimate
            .covariance_of_mean
            .iter()
            .enumerate()
            .any(|(i, x)| i / 4 != i % 4 && x.abs() > 1e-16),
        "{label}: full Laurent covariance unexpectedly absent"
    );
    eprintln!(
        "{label}: means={:?}, standard_errors={:?}",
        estimate.mean, estimate.standard_error
    );
}
