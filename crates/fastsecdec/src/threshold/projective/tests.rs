use super::*;
use crate::threshold::gcad::{GcadError, GcadKinematics, GcadRequest, SolverOptions};
use std::collections::BTreeMap;
use symbolica::{domains::atom::AtomField, parse, symbol, tensors::matrix::Matrix};

fn density(coordinates: &[Symbol], terms: &[ParametricTerm]) -> Atom {
    terms
        .iter()
        .map(|t| {
            t.prefactor()
                * coordinates
                    .iter()
                    .zip(t.monomial_powers())
                    .map(|(s, p)| Atom::var(*s).pow(p))
                    .product::<Atom>()
                * t.factors()
                    .iter()
                    .map(|f| f.polynomial().pow(f.exponent()))
                    .product::<Atom>()
        })
        .sum()
}

fn rich_input() -> ParametricIntegrand {
    let (x, y, z, eps, eta) = symbol!(
        "affine_projective_test::x",
        "affine_projective_test::y",
        "affine_projective_test::z",
        "affine_projective_test::eps",
        "affine_projective_test::eta"
    );
    let u = Atom::var(x) + Atom::var(y) + Atom::var(z);
    let f = u.pow(2) - Atom::num(5) * Atom::var(x) * Atom::var(z);
    let numerator = Atom::var(x) + parse!("2*I") * Atom::var(y);
    let factors = vec![
        PolynomialFactor::new(
            u,
            Atom::num((-7, 2)) + Atom::var(eps) - Atom::var(eta),
            FactorRole::Singularity,
        )
        .with_semantics(FactorSemantics::Positive),
        PolynomialFactor::new(f, -Atom::num(1) - Atom::var(eps), FactorRole::Singularity)
            .with_semantics(FactorSemantics::Causal),
        PolynomialFactor::new(numerator, Atom::num(2), FactorRole::Polynomial),
    ];
    ParametricIntegrand::new(
        vec![x, y, z],
        eps,
        ParametricDomain::ProjectiveSimplex,
        [parse!("1+I"), Atom::num(3)]
            .into_iter()
            .map(|prefactor| {
                ParametricTerm::new(
                    prefactor,
                    vec![Atom::var(eps), Atom::var(eta), Atom::num((1, 2))],
                    factors.clone(),
                )
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn all_native_terms_powers_numerators_and_branches_survive_every_gauge() {
    let input = rich_input();
    for d in 0..input.parameters().len() {
        let p = AffineProjectivePreparation::eliminate(&input, d).unwrap();
        assert_eq!(p.input(), &input);
        assert_eq!(p.terms().len(), input.terms().len());
        assert_eq!(
            p.coordinates(),
            input
                .parameters()
                .iter()
                .enumerate()
                .filter_map(|(i, s)| (i != d).then_some(*s))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.images().iter().cloned().sum::<Atom>().expand(),
            Atom::num(1)
        );
        assert_eq!(p.measure_jacobian(), &Atom::num(1));
        assert_eq!(p.strict_positive(), p.boundary_polynomials());
        let replacement = [Replacement::new(
            Pattern::Literal(Atom::var(input.parameters()[d])),
            Pattern::Literal(p.images()[d].clone()),
        )];
        assert_eq!(
            density(p.coordinates(), p.terms()),
            input.density().replace_multiple(&replacement)
        );
        for (i, (source, mapped)) in input.terms().iter().zip(p.terms()).enumerate() {
            assert_eq!(mapped.prefactor(), source.prefactor());
            for (j, (original, pulled)) in source.factors().iter().zip(mapped.factors()).enumerate()
            {
                assert_eq!(pulled.role(), original.role());
                assert_eq!(pulled.semantics(), original.semantics());
                assert_eq!(pulled.exponent(), original.exponent());
                assert_eq!(
                    p.factor_origins()[i][j],
                    FactorOrigin::OriginalFactor { factor_index: j }
                );
                assert!(!pulled.polynomial().contains_symbol(input.parameters()[d]));
            }
            let extra = mapped.factors().last().unwrap();
            assert_eq!(extra.exponent(), &source.monomial_powers()[d]);
            assert_eq!(extra.semantics(), FactorSemantics::Positive);
            assert_eq!(
                p.factor_origins()[i].last(),
                Some(&FactorOrigin::OriginalMonomial { parameter_index: d })
            );
        }
    }
}

#[test]
fn delta_measure_is_not_induced_surface_measure() {
    let input = rich_input();
    let c = symbol!("affine_projective_test::delta_axis");
    for d in 0..3 {
        let p = AffineProjectivePreparation::eliminate(&input, d).unwrap();
        let mut axes = p.coordinates().to_vec();
        axes.push(c);
        let mut images = p.images().to_vec();
        images[d] -= Atom::var(c);
        let entries = images
            .iter()
            .flat_map(|image| axes.iter().map(|s| image.derivative(*s)))
            .collect();
        let determinant = Matrix::from_linear(
            entries,
            3,
            3,
            AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        )
        .unwrap()
        .det()
        .unwrap();
        assert_eq!(determinant.pow(2), Atom::num(1));
        assert_eq!(
            (Atom::num(1) - images.into_iter().sum::<Atom>()).expand(),
            Atom::var(c)
        );
    }
    let tangent_gram = Matrix::from_linear(
        vec![Atom::num(2), Atom::num(1), Atom::num(1), Atom::num(2)],
        2,
        2,
        AtomField::new(),
    )
    .unwrap();
    assert_eq!(tangent_gram.det().unwrap(), Atom::num(3));
}

fn physical_bubble() -> ParametricIntegrand {
    use crate::{Kinematics, Model, input::GraphIntegral};
    use feynkit_graph::symbols;
    let model = Arc::new(
        Model::from_json(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/scalar.json"
        )))
        .unwrap(),
    );
    let kin = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_mass_squared(&symbols::external_momentum().call(1), Atom::num(5))
        .unwrap();
    let graph = GraphIntegral::from_dot(
        model,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/graphs/bubble.dot"
        )),
        &kin,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::num(1))]))
    .unwrap();
    ParametricIntegrand::from_graph(
        &graph,
        vec![symbol!("affine_bubble::x"), symbol!("affine_bubble::y")],
        symbol!("affine_bubble::eps"),
        parse!("4-2*affine_bubble::eps"),
    )
    .unwrap()
}

fn request(
    p: &AffineProjectivePreparation,
    kinematics: GcadKinematics,
) -> Result<GcadRequest, GcadError> {
    let mut limits = GcadRequest::default_limits();
    limits.wall_time_secs = 10.;
    limits.memory_mib = 1024;
    GcadRequest::projective(p.clone(), kinematics, SolverOptions::default(), limits)
}

#[test]
fn native_graph_bubble_retains_causal_polynomial_and_verifies_both_signs() {
    let input = physical_bubble();
    let p = AffineProjectivePreparation::last_coordinate(&input).unwrap();
    let x = Atom::var(p.coordinates()[0]);
    let f = p.terms()[0]
        .factors()
        .iter()
        .find(|f| f.semantics() == FactorSemantics::Causal)
        .unwrap();
    assert_eq!(
        (f.polynomial() - (Atom::num(1) - Atom::num(5) * &x * (Atom::num(1) - &x))).expand(),
        Atom::num(0)
    );
    let r = request(&p, GcadKinematics::default()).unwrap();
    assert_eq!(r.input(), &input);
    assert_eq!(r.prepared_terms(), p.terms());
    assert_eq!(r.domain().coordinates(), p.coordinates());
    let verified = r.solve_verified().unwrap();
    let signs = verified
        .cells()
        .map(|cell| {
            cell.signed_factors()
                .find(|(f, _)| f.semantics == FactorSemantics::Causal)
                .unwrap()
                .1
        })
        .collect::<Vec<_>>();
    assert_eq!(signs, vec![1, -1, 1]);
    let associated = r
        .signed_factors()
        .iter()
        .find(|f| f.semantics == FactorSemantics::Causal)
        .unwrap();
    assert_eq!(associated.prepared_polynomial, *f.polynomial());
    assert_eq!(
        associated.original_polynomial,
        *input.terms()[0].factors()[associated.factor_index].polynomial()
    );
}

#[test]
fn projective_request_identity_binds_density_gauge_and_original_coordinate_roles() {
    let input = physical_bubble();
    let p = AffineProjectivePreparation::last_coordinate(&input).unwrap();
    let r = request(&p, GcadKinematics::default()).unwrap();
    let evidence = r.solve().unwrap();
    let alternative = AffineProjectivePreparation::eliminate(&input, 0).unwrap();
    assert!(matches!(
        request(&alternative, GcadKinematics::default())
            .unwrap()
            .verify(evidence.clone()),
        Err(GcadError::RequestMismatch)
    ));
    let t = &input.terms()[0];
    let changed = ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![ParametricTerm::new(
            Atom::num(2) * t.prefactor(),
            t.monomial_powers().to_vec(),
            t.factors().to_vec(),
        )],
    )
    .unwrap();
    let changed = AffineProjectivePreparation::last_coordinate(&changed).unwrap();
    let changed = request(&changed, GcadKinematics::default()).unwrap();
    assert_eq!(changed.problem().split, r.problem().split);
    assert!(matches!(
        changed.verify(evidence),
        Err(GcadError::RequestMismatch)
    ));
    let eliminated = input.parameters()[p.eliminated_index()];
    for kinematics in [
        GcadKinematics {
            runtime_parameters: vec![eliminated],
            ..Default::default()
        },
        GcadKinematics {
            exact_values: BTreeMap::from([(eliminated, 1.into())]),
            ..Default::default()
        },
    ] {
        assert!(matches!(
            request(&p, kinematics),
            Err(GcadError::Invalid(_))
        ));
    }
}

#[test]
fn zero_dimensional_preparation_remains_exact_and_wrong_domains_are_refused() {
    let x = symbol!("affine_point::x");
    let eps = symbol!("affine_point::eps");
    let t = ParametricTerm::new(Atom::num((3, 2)), vec![Atom::num(-1)], vec![]);
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![t.clone()],
    )
    .unwrap();
    let p = AffineProjectivePreparation::last_coordinate(&input).unwrap();
    assert!(p.coordinates().is_empty());
    assert_eq!(p.images(), &[Atom::num(1)]);
    assert_eq!(density(p.coordinates(), p.terms()), Atom::num((3, 2)));
    assert!(matches!(
        request(&p, GcadKinematics::default()),
        Err(GcadError::Unsupported(_))
    ));
    assert!(AffineProjectivePreparation::eliminate(&input, 1).is_err());
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::PositiveOrthant,
    ] {
        let ordinary = ParametricIntegrand::new(vec![x], eps, domain, vec![t.clone()]).unwrap();
        assert!(AffineProjectivePreparation::last_coordinate(&ordinary).is_err());
    }
}

#[test]
fn request_reuses_both_original_and_prepared_owners() {
    let prepared =
        Arc::new(AffineProjectivePreparation::last_coordinate(&physical_bubble()).unwrap());
    let original = prepared.input_owner().clone();
    let request = GcadRequest::projective(
        prepared.clone(),
        GcadKinematics::default(),
        SolverOptions::default(),
        GcadRequest::default_limits(),
    )
    .unwrap();
    assert!(std::ptr::eq(request.input(), original.as_ref()));
    assert!(std::ptr::eq(
        request.projective_preparation().unwrap(),
        prepared.as_ref()
    ));
    let cloned = request.clone();
    assert!(std::ptr::eq(cloned.prepared_terms(), prepared.terms()));
    drop(request);
    drop(prepared);
    assert!(std::ptr::eq(cloned.input(), original.as_ref()));
    assert_eq!(cloned.prepared_terms().len(), original.terms().len());
}
