use super::*;
use crate::{
    contour::{ContourMode, DynamicConstruction, functions::dynamic},
    generation::{GenerationOptions, generate},
    kernel::{
        CompilationSettings, NativeProgramDescriptor, PrecisionPolicy, ProgramRecipe, compilation,
        contour::dynamic::validation::Specification,
    },
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{ops::ControlFlow, sync::Arc};
use symbolica::function;

struct Fixture {
    owner: NativeProgramDescriptor,
    root: Atom,
    requests: Vec<ExactRequest>,
    runtime: Vec<Symbol>,
    condition: Symbol,
    p: Symbol,
    q: Symbol,
}

fn fixture(sign: i64) -> Fixture {
    let x = symbol!("checked_exact::x");
    let eps = symbol!("checked_exact::eps");
    let condition = symbol!("checked_exact::condition");
    let p = symbol!("checked_exact::p");
    let q = symbol!("checked_exact::q");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    sign * (2 + Atom::var(x) + Atom::var(x).pow(3)),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(1 + Atom::var(x).pow(2), Atom::Zero, FactorRole::Polynomial)
                    .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let runtime = compilation::runtime_inputs(&generated, &[condition, p, q]);
    let source = &generated.dynamic_check_sources()[0];
    let prepared =
        compilation::PreparedDynamic::build(&generated, &runtime, CompilationSettings::default())
            .unwrap()
            .unwrap();
    let root = source
        .full_strength
        .replace(Atom::var(source.parameters[0]))
        .with(0);
    let requests = prepared
        .lookup
        .exact_requests(std::slice::from_ref(&root))
        .unwrap();
    Fixture {
        owner: prepared.descriptor.as_ref().clone(),
        root,
        requests,
        runtime,
        condition,
        p,
        q,
    }
}

impl Fixture {
    fn point(&self) -> BTreeMap<Symbol, f64> {
        BTreeMap::from([
            (self.condition, 0.),
            (self.p, 2.),
            (self.q, 1.),
            (crate::contour::dynamic::safety_fraction_symbol(), 0.8),
            (crate::contour::dynamic::lambda_cap_symbol(), 1.),
            (crate::contour::dynamic::displacement_cap_symbol(), 1.),
        ])
    }
    fn checked(
        &self,
        point: &BTreeMap<Symbol, f64>,
        requests: &[ExactRequest],
    ) -> (Validation, Vec<f64>) {
        let point = self.runtime.iter().map(|s| point[s]).collect::<Vec<_>>();
        let specification = Specification::new(
            &self.owner,
            &[],
            point.clone(),
            ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: 1.,
                displacement_cap: 1.,
                construction: DynamicConstruction::Polynomial,
            },
            requests.iter().map(|r| r.bundle.clone()).collect(),
            &PrecisionPolicy {
                max_bits: 256,
                ..Default::default()
            },
        )
        .unwrap();
        (Validation::new(Arc::clone(&specification)), point)
    }
}

#[test]
fn only_executed_exact_roots_are_certified_once_across_the_vector() {
    let fixture = fixture(1);
    let _owner = fixture.owner.enter();
    let invalid = fixture
        .root
        .replace(Atom::var(crate::contour::dynamic::safety_fraction_symbol()))
        .with(0);
    let mut requests = fixture.requests.clone();
    requests.push(ExactRequest {
        root: invalid.clone(),
        bundle: requests[0].bundle.clone(),
    });
    let expressions = vec![
        function!(
            Symbol::IF,
            Atom::var(fixture.condition),
            invalid,
            fixture.root.clone()
        ),
        &fixture.root * 2,
    ];
    let point = fixture.point();
    for complex in [false, true] {
        let (mut validation, validation_point) = fixture.checked(&point, &requests);
        let values = evaluate_checked(
            &expressions,
            &point,
            complex,
            &requests,
            &mut validation,
            &validation_point,
        )
        .unwrap();
        let expected = evaluate(&expressions, &point, complex).unwrap();
        assert_eq!(values, expected);
        assert_eq!(
            validation.checked_arguments, 1,
            "repeated output roots must not execute/certify again"
        );
        assert!(dynamic::take_failure().is_none());
    }
}

#[test]
fn summed_exact_atoms_preserve_cancellation_and_merge_distinct_causal_contexts() {
    let mut left = fixture(1);
    let mut right = fixture(-1);
    assert_eq!(left.root, right.root);
    assert_ne!(left.requests[0].bundle, right.requests[0].bundle);
    right.owner.remap_charts(&[1]).unwrap();
    left.owner.merge(right.owner).unwrap();
    let _owner = left.owner.enter();
    let a = &left.root / Atom::var(left.p);
    let b = -&right.root / Atom::var(left.p);
    let expressions = vec![a + b, &left.root + &right.root];
    assert!(expressions[0].is_zero());
    let requests = dynamic::requests::merge_exact_requests(
        &expressions,
        left.requests.iter().cloned().chain(right.requests),
    )
    .unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].bundle.0.len(), 2);
    let mut point = left.point();
    point.insert(left.p, 0.);
    let (mut validation, validation_point) = left.checked(&point, &requests);
    let values = evaluate_checked(
        &expressions,
        &point,
        true,
        &requests,
        &mut validation,
        &validation_point,
    )
    .unwrap();
    assert_eq!(values, [0., 0., 1.6, 0.]);
    assert_eq!(
        validation.checked_arguments, 2,
        "one executed radius certifies both retained contexts"
    );

    let cancelled = vec![&left.root - &right.root, Atom::num(6)];
    let requests = dynamic::requests::merge_exact_requests(&cancelled, requests).unwrap();
    assert!(requests.is_empty());
    let (mut validation, validation_point) = left.checked(&point, &requests);
    assert_eq!(
        evaluate_checked(
            &cancelled,
            &point,
            false,
            &requests,
            &mut validation,
            &validation_point
        )
        .unwrap(),
        [0., 6.]
    );
    assert_eq!(validation.checked_arguments, 0);
    assert_eq!(
        validation.maximum_bits, 0,
        "cancelled roots must not construct ball scratch"
    );
}

#[test]
fn checked_exact_vector_retries_range_failures_and_rejects_missing_associations() {
    let fixture = fixture(1);
    let _owner = fixture.owner.enter();
    let mut point = fixture.point();
    point.insert(fixture.p, 1e200);
    point.insert(fixture.q, 1e-200);
    let expressions = vec![
        &fixture.root * Atom::var(fixture.p).pow(2) * Atom::var(fixture.q),
        fixture.root.clone(),
    ];
    for complex in [false, true] {
        let (mut validation, validation_point) = fixture.checked(&point, &fixture.requests);
        let values = evaluate_checked(
            &expressions,
            &point,
            complex,
            &fixture.requests,
            &mut validation,
            &validation_point,
        )
        .unwrap();
        assert!((values[0] / 8e199 - 1.).abs() < 2e-14);
        assert!((values[if complex { 2 } else { 1 }] - 0.8).abs() < 1e-15);
        assert!(
            validation.checked_arguments >= 2,
            "retry certifies actual higher-precision candidates"
        );
        assert!(dynamic::take_failure().is_none());
    }
    let (mut validation, validation_point) = fixture.checked(&point, &fixture.requests);
    assert!(
        matches!(evaluate_checked(std::slice::from_ref(&fixture.root),&point,false,&[],&mut validation,&validation_point),Err(KernelError::Contour(reason)) if reason.contains("undeclared dynamic strength"))
    );
}

#[test]
fn final_exact_error_retains_certificate_and_later_callback_failures() {
    use symbolica::atom::EvaluationInfo;
    let fixture = fixture(1);
    let _owner = fixture.owner.enter();
    // Deliberately associate an altered mathematical root with the original
    // proof. The certificate must reject it before the MP callback failure.
    let altered = fixture
        .root
        .replace(Atom::var(crate::contour::dynamic::safety_fraction_symbol()))
        .with(Atom::num((2, 5)));
    let requests = vec![ExactRequest {
        root: altered.clone(),
        bundle: fixture.requests[0].bundle.clone(),
    }];
    let fail_at_mp = symbol!(
        "checked_exact::fail_at_mp",
        eval = EvaluationInfo::new()
            .register(|_: &[f64]| 1.)
            .register(|_: &[DoubleFloat]| DoubleFloat::from(1.))
            .register(|_: &[Float]| {
                dynamic::failure("deliberate high precision exact callback failure".into());
                Float::with_val(212, 1)
            })
    );
    let point = fixture.point();
    let (mut validation, validation_point) = fixture.checked(&point, &requests);
    let error = evaluate_checked(
        &[altered, function!(fail_at_mp, Atom::var(fixture.p))],
        &point,
        false,
        &requests,
        &mut validation,
        &validation_point,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("last certificate failure"), "{error}");
    assert!(error.contains("namespace"), "{error}");
    assert!(
        error.contains("deliberate high precision exact callback failure"),
        "{error}"
    );
    assert!(dynamic::take_failure().is_none());
}
