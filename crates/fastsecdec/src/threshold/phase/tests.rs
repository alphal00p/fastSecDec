use super::*;
use crate::{
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
    threshold::gcad::{GcadKinematics, GcadRequest, Limits, SolverOptions, VerifiedDecomposition},
};
use symbolica::{domains::float::Complex, poly::series::SeriesDepth, symbol};

fn axes() -> (Symbol, Symbol, Symbol) {
    symbol!("cell_phase::x", "cell_phase::eps", "cell_phase::eta")
}

fn verified(terms: Vec<ParametricTerm>, kinematics: GcadKinematics) -> VerifiedDecomposition {
    let (x, eps, _) = axes();
    let input = ParametricIntegrand::new(vec![x], eps, ParametricDomain::UnitCube, terms).unwrap();
    GcadRequest::unit_cube(
        &input,
        kinematics,
        SolverOptions::default(),
        Limits {
            workers: 1,
            memory_mib: 1024,
            wall_time_secs: 10.,
            ..Limits::default()
        },
    )
    .unwrap()
    .solve_verified()
    .unwrap()
}

fn term(f: Atom, q: Atom, semantics: FactorSemantics) -> ParametricTerm {
    let (x, _, _) = axes();
    ParametricTerm::new(
        Atom::num(1) + Atom::i(),
        vec![Atom::Zero],
        vec![
            PolynomialFactor::new(f, q, FactorRole::Singularity).with_semantics(semantics),
            PolynomialFactor::new(
                Atom::i() * Atom::var(x) + Atom::num(2),
                Atom::num(1),
                FactorRole::Polynomial,
            ),
        ],
    )
}

#[test]
fn phases_are_term_local_and_keep_all_regulators_and_numerators() {
    let (x, eps, eta) = axes();
    let f = Atom::num(2) * Atom::var(x) - Atom::num(1);
    let q1 = Atom::num(-1) + Atom::var(eta);
    let q2 = Atom::num(-2) + Atom::var(eps) + Atom::num(2) * Atom::var(eta);
    let terms = vec![
        term(f.clone(), q1, FactorSemantics::Causal),
        term(-&f, q2, FactorSemantics::Causal),
    ];
    let geometry = verified(terms.clone(), GcadKinematics::default());
    assert_eq!(geometry.request().prepared_terms(), terms);
    for cell in geometry.cells() {
        let phased = CausalCell::new(cell, &[eps, eta]).unwrap();
        let factors = phased.magnitudes().collect::<Vec<_>>();
        assert_eq!(factors.len(), 2);
        assert_eq!(factors[0].sign(), -factors[1].sign());
        assert!(
            (factors[0].polynomial() - factors[1].polynomial())
                .expand()
                .is_zero()
        );
        let varying = if factors[0].sign() < 0 {
            assert_eq!(phased.term_phases()[1], Atom::num(1));
            -(-Atom::i() * Atom::var(Symbol::PI) * Atom::var(eta)).exp()
        } else {
            assert_eq!(phased.term_phases()[0], Atom::num(1));
            (-Atom::i() * Atom::var(Symbol::PI) * (Atom::var(eps) + Atom::num(2) * Atom::var(eta)))
                .exp()
        };
        let index = usize::from(factors[1].sign() < 0);
        assert_eq!(phased.term_phases()[index], varying);
        assert_eq!(phased.cell().index(), cell.index());
    }
}

#[test]
fn exact_two_cell_control_retains_the_finite_imaginary_part() {
    let (x, eps, eta) = axes();
    let f = Atom::num(2) * Atom::var(x) - Atom::num(1);
    let scalar = ParametricTerm::new(
        Atom::num(1),
        vec![Atom::Zero],
        vec![
            PolynomialFactor::new(f, Atom::num(-1) + Atom::var(eta), FactorRole::Singularity)
                .with_semantics(FactorSemantics::Causal),
        ],
    );
    let geometry = verified(vec![scalar], GcadKinematics::default());
    // Check the analytic scalar control ∫_0^1 (2x-1-i0)^(-1+eta) dx.
    // Each exact affine cell has width 1/2 and a unit positive factor t.
    let pair = geometry
        .cells()
        .map(|c| CausalCell::new(c, &[eps, eta]).unwrap().term_phases()[0].clone())
        .sum::<Atom>()
        / (Atom::num(2) * Atom::var(eta));
    let series = pair.series(eta, 0, SeriesDepth::absolute(1)).unwrap();
    assert_eq!(series.coefficient((-1).into()), Some(Atom::Zero));
    assert_eq!(
        series.coefficient(0.into()).unwrap().expand(),
        Atom::i() * Atom::var(Symbol::PI) * Atom::num((1, 2))
    );
    // Freezing the negative-cell phase at eta=0 would incorrectly give zero.
}

#[test]
fn exact_fractional_phases_follow_the_lower_causal_lip() {
    let (_, eps, eta) = axes();
    let orders = [-2, -1, 1, 2];
    let geometry = verified(
        orders
            .iter()
            .map(|n| {
                term(
                    Atom::num(-1),
                    Atom::num((*n, 3)) + Atom::var(eta),
                    FactorSemantics::Causal,
                )
            })
            .collect(),
        GcadKinematics::default(),
    );
    let phased = CausalCell::new(geometry.cells().next().unwrap(), &[eps, eta]).unwrap();
    let expressions = phased
        .term_phases()
        .iter()
        .map(|p| p.replace(Atom::var(eta)).with(Atom::Zero))
        .collect::<Vec<_>>();
    let mut evaluator = Atom::evaluator_multiple(&expressions, &[] as &[Atom])
        .build()
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut values = vec![Complex::new(0., 0.); orders.len()];
    evaluator.evaluate(&[], &mut values);
    for (n, value) in orders.into_iter().zip(values) {
        let angle = -std::f64::consts::PI * (n as f64) / 3.;
        assert!((value.re - angle.cos()).abs() < 2e-15);
        assert!((value.im - angle.sin()).abs() < 2e-15);
    }
}

#[test]
fn regulators_cannot_reclassify_kinematics_or_coordinates() {
    let (x, eps, eta) = axes();
    let a = symbol!("cell_phase::a");
    let geometry = verified(
        vec![term(
            Atom::var(x) - Atom::var(a),
            Atom::num(-1) + Atom::var(eta),
            FactorSemantics::Causal,
        )],
        GcadKinematics {
            runtime_parameters: vec![a],
            strict_positive: vec![Atom::var(a), Atom::num(1) - Atom::var(a)],
            ..Default::default()
        },
    );
    let cell = geometry.cells().next().unwrap();
    for regulators in [
        vec![],
        vec![eps, eps],
        vec![eta, eps],
        vec![eps, x],
        vec![eps, a],
    ] {
        assert!(CausalCell::new(cell, &regulators).is_err());
    }
    let generic = verified(
        vec![term(
            Atom::num(-1),
            Atom::num(-1) + Atom::var(eta),
            FactorSemantics::Generic,
        )],
        GcadKinematics::default(),
    );
    assert!(matches!(
        CausalCell::new(generic.cells().next().unwrap(), &[eps, eta]),
        Err(GcadError::Unsupported(_))
    ));
}

#[test]
fn exact_kinematic_bindings_apply_before_phase_admission() {
    let (_, eps, eta) = axes();
    let (power, slope) = symbol!("cell_phase::power", "cell_phase::slope");
    let bindings = GcadKinematics {
        exact_values: [(power, (-1).into()), (slope, 2.into())].into(),
        ..Default::default()
    };
    let exponent = Atom::var(power) + Atom::var(slope) * Atom::var(eps) + Atom::var(eta);
    let geometry = verified(
        vec![term(
            Atom::num(-1),
            exponent.clone(),
            FactorSemantics::Causal,
        )],
        bindings.clone(),
    );
    let phase = CausalCell::new(geometry.cells().next().unwrap(), &[eps, eta]).unwrap();
    assert_eq!(
        phase.term_phases()[0],
        -(-Atom::i() * Atom::var(Symbol::PI) * (Atom::num(2) * Atom::var(eps) + Atom::var(eta)))
            .exp()
    );
    assert_eq!(geometry.request().signed_factors()[0].exponent, exponent);
    let complete_density = Atom::i() * Atom::var(power) * Atom::var(slope);
    assert_eq!(
        bindings.specialize_exact(&complete_density),
        Atom::num(-2) * Atom::i()
    );

    // A vanishing bound exponent needs no branch, even for a generic factor.
    let zero = verified(
        vec![term(
            Atom::num(-1),
            Atom::var(slope),
            FactorSemantics::Generic,
        )],
        GcadKinematics {
            exact_values: [(slope, 0.into())].into(),
            ..Default::default()
        },
    );
    assert_eq!(
        CausalCell::new(zero.cells().next().unwrap(), &[eps])
            .unwrap()
            .term_phases(),
        &[Atom::one()]
    );
}
