//! Independent analytic integrals, exercised through generation and QMC.
use std::ops::ControlFlow;

use fastsecdec::{
    Atom,
    generation::{GenerationOptions, generate},
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{parse, symbol};

#[test]
fn fractional_endpoint_degree_has_the_correct_meromorphic_integral() {
    // The analytic continuation is 1/(eps-1/2), including its negative sign.
    let input = ParametricIntegrand::new(
        vec![symbol!("audit::x")],
        symbol!("audit::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![parse!("-3/2+audit::eps")],
            vec![],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert!(generated.sectors().is_empty());
    assert_eq!(generated.orders(), &[0, 1, 2]);
    assert_eq!(
        generated.exact_coefficients(),
        &[Atom::num(-2), Atom::num(-4), Atom::num(-8)]
    );
}

#[test]
fn complete_vector_matches_independent_logarithmic_integral_with_gamma_prefactor() {
    // J(eps)=int_0^1 x^(eps-1)/(1+x)^2 dx has coefficients
    // 1/eps, -log(2)-1/2, pi^2/12+log(2). Multiply by eps*Gamma(eps).
    // These follow directly by integration by parts and geometric series.
    let input = ParametricIntegrand::new(
        vec![symbol!("audit::x")],
        symbol!("audit::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("audit::eps*gamma(audit::eps)"),
            vec![parse!("audit::eps-1")],
            vec![PolynomialFactor::new(
                parse!("1+audit::x"),
                Atom::num(-2),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut kernels = generated.compile().unwrap();
    let problem = IntegrationProblem::new(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(index, sector)| SectorSpec {
                id: index as u64,
                dimension: sector.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )
    .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 4096,
            shifts: 16,
            seed: 7351,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|i| session.worker_context(i as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let index = task.sector_id() as usize;
        let result = workers[index]
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[index].evaluate(point, output)
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let value = session.estimate().unwrap();
    let gamma = std::f64::consts::EULER_GAMMA;
    let j0 = -2.0f64.ln() - 0.5;
    let j1 = std::f64::consts::PI.powi(2) / 12.0 + 2.0f64.ln();
    let expected = [
        1.0,
        j0 - gamma,
        j1 - gamma * j0 + gamma * gamma / 2.0 + std::f64::consts::PI.powi(2) / 12.0,
    ];
    assert_eq!(value.orders, [-1, 0, 1]);
    assert!(value.production_complete);
    for ((mean, error), target) in value.mean.iter().zip(&value.standard_error).zip(expected) {
        assert!(
            (mean - target).abs() < 6.0 * error + 5e-10,
            "{mean} +/- {error}, expected {target}"
        );
    }
}

#[test]
fn off_origin_boundary_poles_are_not_certified_by_formal_generation() {
    // P=(x-1/2)^2+y is positive at every interior point and every corner,
    // but vanishes at (1/2,0). Integrating y first gives
    // [int_0^1 ((x-1/2)^2+1)^(eps-1/2) dx - 2^(-2eps)/eps]
    // /(eps-1/2), whose Laurent residue is exactly +2.
    // An origin-only fan sees P's constant support and misses this pole. The
    // caller must resolve this geometry before using the formal coefficients;
    // generation deliberately no longer certifies faces or interior points.
    let input = ParametricIntegrand::new(
        vec![symbol!("audit_boundary::x"), symbol!("audit_boundary::y")],
        symbol!("audit_boundary::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0), Atom::num(0)],
            vec![PolynomialFactor::new(
                parse!("(audit_boundary::x-1/2)^2+audit_boundary::y"),
                parse!("-3/2+audit_boundary::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            assume_no_threshold: true,
            max_order: 0,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    );
    let generated = generated.unwrap();
    assert_user_responsibility(&generated);
    assert_eq!(
        generated.metadata().domain_assessment().factors()[0].polynomial(),
        &parse!("(audit_boundary::x-1/2)^2+audit_boundary::y")
    );
}

#[test]
fn mixed_sign_boundaries_retain_uncertified_source_factors_across_domains() {
    let parameters = vec![
        symbol!("audit_faces::x"),
        symbol!("audit_faces::y"),
        symbol!("audit_faces::z"),
    ];
    for (domain, polynomial, powers, exponent) in [
        (
            ParametricDomain::UnitCube,
            // The whole x=0 face vanishes; its leading coefficient has an
            // affine boundary zero that remains the caller's responsibility.
            parse!("audit_faces::x*((audit_faces::y-1/2)^2+audit_faces::z)"),
            vec![Atom::num(0); 3],
            parse!("-3/2+audit_faces::eps"),
        ),
        (
            ParametricDomain::PositiveOrthant,
            parse!("(audit_faces::x-1/2)^2+audit_faces::y+audit_faces::z"),
            vec![Atom::num(0); 3],
            parse!("-3/2+audit_faces::eps"),
        ),
        (
            ParametricDomain::ProjectiveSimplex,
            parse!("(audit_faces::x-audit_faces::y)^2+audit_faces::z^2"),
            vec![parse!("-2*audit_faces::eps"), Atom::num(0), Atom::num(0)],
            parse!("-3/2+audit_faces::eps"),
        ),
    ] {
        let input = ParametricIntegrand::new(
            parameters.clone(),
            symbol!("audit_faces::eps"),
            domain,
            vec![ParametricTerm::new(
                Atom::num(1),
                powers,
                vec![PolynomialFactor::new(
                    polynomial,
                    exponent,
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap();
        let generated = generate(
            &input,
            &GenerationOptions {
                assume_no_threshold: true,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_user_responsibility(&generated);
        assert_eq!(generated.metadata().domain_assessment().domain(), domain);
        assert_eq!(
            generated.metadata().domain_assessment().factors()[0].polynomial(),
            input.terms()[0].factors()[0].polynomial()
        );
    }
}

#[test]
fn tangential_origin_geometry_has_no_implicit_regularity_certificate() {
    // The original coordinate faces are positive away from the origin. After
    // x=t*y the leading form contains (t-1)^2, which vanishes at the upper
    // endpoint t=1 on the exceptional face y=0. Certifying original faces
    // alone therefore cannot certify regularity of sector residuals. No such
    // certificate is attempted; these formal outputs are not an integral check.
    let input = ParametricIntegrand::new(
        vec![symbol!("audit_tangent::x"), symbol!("audit_tangent::y")],
        symbol!("audit_tangent::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0); 2],
            vec![PolynomialFactor::new(
                parse!("(audit_tangent::x-audit_tangent::y)^2+2*audit_tangent::x^2*audit_tangent::y+2*audit_tangent::x*audit_tangent::y^2+audit_tangent::x^3+audit_tangent::y^3"),
                parse!("-3/2+audit_tangent::eps"),
                FactorRole::Singularity,
            )],
        )],
    ).unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            assume_no_threshold: true,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_user_responsibility(&generated);
}

fn assert_user_responsibility(generated: &fastsecdec::generation::GeneratedIntegral) {
    let domain = generated.metadata().domain_assessment();
    assert_eq!(
        domain.branch_policy(),
        fastsecdec::generation::BranchPolicy::UserResponsible
    );
    assert!(!domain.relies_on_assertion());
    assert!(domain.factors().iter().all(|factor| factor.certificate()
        == fastsecdec::generation::FactorCertificate::UncheckedUserResponsibility));
}
