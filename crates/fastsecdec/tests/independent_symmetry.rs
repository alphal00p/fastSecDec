//! Integral identities independently check whole-density symmetry multiplicities.
use std::ops::ControlFlow;

use fastsecdec::{
    Atom,
    generation::{GenerationOptions, generate},
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec, VectorEstimate},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{parse, symbol};

fn singular_factor() -> PolynomialFactor {
    PolynomialFactor::new(
        parse!("symmetry_audit::x+symmetry_audit::y"),
        parse!("symmetry_audit::eps"),
        FactorRole::Singularity,
    )
}

fn integrate(terms: Vec<ParametricTerm>, representatives: usize) -> VectorEstimate {
    let input = ParametricIntegrand::new(
        vec![symbol!("symmetry_audit::x"), symbol!("symmetry_audit::y")],
        symbol!("symmetry_audit::eps"),
        ParametricDomain::UnitCube,
        terms,
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(generated.sectors().len(), representatives);
    let mut kernels = generated.compile().unwrap();
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
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 1024,
            shifts: 16,
            seed: 94361,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..representatives)
        .map(|id| session.worker_context(id as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id() as usize;
        let value = workers[id]
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[id].evaluate(point, output)
            })
            .unwrap();
        session.submit(value).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.orders, [-1, 0]);
    assert!(estimate.production_complete);
    estimate
}

fn near(estimate: &VectorEstimate, index: usize, expected: f64) {
    assert!(
        (estimate.mean[index] - expected).abs() <= 6.0 * estimate.standard_error[index] + 1e-9,
        "order {}: {} +/- {}, expected {expected}",
        estimate.orders[index],
        estimate.mean[index],
        estimate.standard_error[index]
    );
}

#[test]
fn symmetric_chart_multiplicity_preserves_pole_and_finite_coefficient() {
    // int_square (x+y)^eps = (2^(eps+2)-2)/((eps+1)(eps+2)).
    // Multiplication by Gamma(eps) gives residue1 and the finite value below.
    // The Newton split has two charts; the complete densities are identical.
    let estimate = integrate(
        vec![ParametricTerm::new(
            parse!("gamma(symmetry_audit::eps)"),
            vec![Atom::num(0); 2],
            vec![singular_factor()],
        )],
        1,
    );
    near(&estimate, 0, 1.0);
    near(
        &estimate,
        1,
        2.0 * 2.0f64.ln() - 1.5 - std::f64::consts::EULER_GAMMA,
    );
}

#[test]
fn asymmetric_numerator_is_not_merged_by_equal_singular_support() {
    // The residue is int_square(x^2+2*y^2)=1. The two chart integrals are
    // respectively 5/12 and 7/12, so multiplying either by2 would be wrong.
    let estimate = integrate(
        vec![ParametricTerm::new(
            parse!("gamma(symmetry_audit::eps)"),
            vec![Atom::num(0); 2],
            vec![
                singular_factor(),
                PolynomialFactor::new(
                    parse!("symmetry_audit::x^2+2*symmetry_audit::y^2"),
                    Atom::num(1),
                    FactorRole::Polynomial,
                ),
            ],
        )],
        2,
    );
    near(&estimate, 0, 1.0);
}

#[test]
fn opposite_and_unequal_term_weights_are_preserved() {
    for (second_weight, representatives, residue) in [(-1, 2, 0.0), (2, 2, 1.5), (1, 1, 1.0)] {
        let estimate = integrate(
            vec![
                ParametricTerm::new(
                    parse!("gamma(symmetry_audit::eps)"),
                    vec![Atom::num(1), Atom::num(0)],
                    vec![singular_factor()],
                ),
                ParametricTerm::new(
                    Atom::num(second_weight) * parse!("gamma(symmetry_audit::eps)"),
                    vec![Atom::num(0), Atom::num(1)],
                    vec![singular_factor()],
                ),
            ],
            representatives,
        );
        near(&estimate, 0, residue);
        if second_weight == -1 {
            // Antisymmetry cancels the complete meromorphic integral, not just
            // its residue. Democratic shared shifts also expose that covariance.
            near(&estimate, 1, 0.0);
        }
    }
}
