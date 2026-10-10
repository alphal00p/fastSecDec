use super::*;
use crate::parametric::{
    FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
    PolynomialFactor,
};
use std::collections::BTreeMap;
use symbolica::{atom::Atom, parse, prelude::Rational, symbol};

fn limits() -> Limits {
    Limits {
        workers: 1,
        wall_time_secs: 10.0,
        memory_mib: 1024,
        ..Limits::default()
    }
}
fn input(f: Atom, exponent: Atom, semantics: FactorSemantics) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("gcad_test::x")],
        symbol!("gcad_test::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1+I"),
            vec![Atom::num(0)],
            vec![
                PolynomialFactor::new(f, exponent, FactorRole::Singularity)
                    .with_semantics(semantics),
            ],
        )],
    )
    .unwrap()
}
fn cube(input: &ParametricIntegrand, kinematics: GcadKinematics) -> GcadRequest {
    GcadRequest::unit_cube(input, kinematics, SolverOptions::default(), limits()).unwrap()
}

#[test]
fn signed_original_density_and_auxiliary_exponent_survive_verified_cells() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let exponent = parse!("-1+gcad_test::eps+gcad_test::eta");
    let source = input(
        -(x - Atom::num((1, 3))) * Atom::num((3, 2)),
        exponent.clone(),
        FactorSemantics::Causal,
    );
    let request = cube(&source, GcadKinematics::default());
    let verified = request.solve_verified().unwrap();
    let signs = verified
        .cells()
        .map(|c| c.signed_factors().next().unwrap().1)
        .collect::<Vec<_>>();
    assert_eq!(signs, vec![1, -1]);
    let f = &verified.request().signed_factors()[0];
    assert_eq!(f.exponent, exponent);
    assert_eq!(
        &f.original_polynomial,
        source.terms()[0].factors()[0].polynomial()
    );
    assert_eq!(
        verified.request().input().terms()[0].prefactor(),
        &parse!("1+I")
    );
    assert_eq!(verified.verification().cells, 2);
}

#[test]
fn parameter_first_family_and_exact_float_binding_use_native_symbols() {
    let a = symbol!("gcad_test::a");
    let x = Atom::var(symbol!("gcad_test::x"));
    let source = input(x - Atom::var(a), Atom::num(-1), FactorSemantics::Causal);
    let kinematics = GcadKinematics {
        runtime_parameters: vec![a],
        strict_positive: vec![Atom::var(a), Atom::num(1) - Atom::var(a)],
        ..GcadKinematics::default()
    };
    let request = cube(&source, kinematics);
    assert_eq!(request.aliases()[0].role, AliasRole::RuntimeParameter);
    assert_eq!(request.aliases()[1].role, AliasRole::IntegrationCoordinate);
    let result = request.solve_verified().unwrap();
    assert_eq!(result.native_result().order, vec!["v0", "v1"]);
    assert_eq!(result.cells().len(), 2);
    let (identity, mut changed) = request.solve().unwrap().into_parts();
    changed.order.reverse();
    assert!(matches!(
        request.verify(NativeDecomposition::from_parts(identity, changed)),
        Err(GcadError::Verification(_))
    ));
    let bound = GcadKinematics::from_f64(&BTreeMap::from([(a, 0.1)])).unwrap();
    assert_eq!(
        bound.exact_values[&a],
        Rational::from((3602879701896397i64, 36028797018963968i64))
    );
    assert_eq!(
        cube(&source, bound).solve_verified().unwrap().cells().len(),
        2
    );
    assert!(GcadKinematics::from_f64(&BTreeMap::from([(a, f64::NAN)])).is_err());
}

#[test]
fn only_exact_equal_geometry_is_shared_and_every_factor_association_remains() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let f = x.clone() - Atom::num((1, 2));
    let mut terms = Vec::new();
    for (polynomial, exponent) in [
        (f.clone(), Atom::num(-1)),
        (f.clone(), parse!("-2+gcad_test::eps")),
        (-f.clone(), Atom::num(-3)),
    ] {
        terms.push(ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0)],
            vec![
                PolynomialFactor::new(polynomial, exponent, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    parse!("I") * x.clone(),
                    Atom::num(1),
                    FactorRole::Polynomial,
                ),
            ],
        ));
    }
    let source = ParametricIntegrand::new(
        vec![symbol!("gcad_test::x")],
        symbol!("gcad_test::eps"),
        ParametricDomain::UnitCube,
        terms,
    )
    .unwrap();
    let request = cube(&source, GcadKinematics::default());
    assert_eq!(request.problem().split.len(), 2);
    assert_eq!(
        request
            .signed_factors()
            .iter()
            .map(|f| f.split_index)
            .collect::<Vec<_>>(),
        vec![0, 0, 1]
    );
    let result = request.solve_verified().unwrap();
    for cell in result.cells() {
        let signs = cell.signed_factors().map(|(_, s)| s).collect::<Vec<_>>();
        assert_eq!(signs[0], signs[1]);
        assert_eq!(signs[0], -signs[2]);
    }
}

#[test]
fn identity_includes_density_and_preparation_provenance_not_only_problem() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let a = input(
        x.clone() - Atom::num((1, 2)),
        Atom::num(-1),
        FactorSemantics::Causal,
    );
    let b = input(
        x.clone() - Atom::num((1, 2)),
        Atom::num(-2),
        FactorSemantics::Causal,
    );
    let first = cube(&a, GcadKinematics::default());
    let second = cube(&b, GcadKinematics::default());
    assert_eq!(
        serde_json::to_vec(first.problem()).unwrap(),
        serde_json::to_vec(second.problem()).unwrap()
    );
    assert!(matches!(
        second.verify(first.solve().unwrap()),
        Err(GcadError::RequestMismatch)
    ));
    let domain = PreparedDomain::explicit(
        a.parameters().to_vec(),
        vec![x.clone(), Atom::num(1) - x],
        "caller-prepared source chart 7".into(),
    )
    .unwrap();
    let prepared = GcadRequest::prepared(
        &a,
        domain,
        GcadKinematics::default(),
        SolverOptions::default(),
        limits(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(first.problem()).unwrap(),
        serde_json::to_vec(prepared.problem()).unwrap()
    );
    assert!(matches!(
        prepared.verify(first.solve().unwrap()),
        Err(GcadError::RequestMismatch)
    ));
    assert!(matches!(
        prepared.domain().origin(),
        DomainOrigin::ExplicitPrepared { .. }
    ));
}

#[test]
fn incomplete_tampered_and_foreign_native_certificates_are_refused() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let source = input(
        x - Atom::num((1, 2)),
        Atom::num(-1),
        FactorSemantics::Causal,
    );
    let request = cube(&source, GcadKinematics::default());
    let (identity, mut raw) = request.solve().unwrap().into_parts();
    raw.cells[0].signs[0] *= -1;
    assert!(matches!(
        request.verify(NativeDecomposition::from_parts(identity, raw)),
        Err(GcadError::Verification(_))
    ));
    let (identity, mut raw) = request.solve().unwrap().into_parts();
    raw.problem.constraints.push("v0>2".into());
    assert!(matches!(
        request.verify(NativeDecomposition::from_parts(identity, raw)),
        Err(GcadError::RequestMismatch)
    ));
    let mut bounded = limits();
    bounded.max_cells = Some(1);
    let request = GcadRequest::unit_cube(
        &source,
        GcadKinematics::default(),
        SolverOptions::default(),
        bounded,
    )
    .unwrap();
    assert!(matches!(
        request.solve_verified(),
        Err(GcadError::Incomplete(_))
    ));
}

#[test]
fn positive_semantics_cannot_silently_remove_a_negative_domain_region() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let source = input(
        x.clone() - Atom::num((1, 2)),
        Atom::num(-1),
        FactorSemantics::Positive,
    );
    let request = cube(&source, GcadKinematics::default());
    assert_eq!(request.problem().constraints.len(), 2);
    assert!(matches!(
        request.solve_verified(),
        Err(GcadError::Unsupported(_))
    ));
    let source = input(x + Atom::num(1), Atom::num(-1), FactorSemantics::Positive);
    assert!(
        cube(&source, GcadKinematics::default())
            .solve_verified()
            .is_ok()
    );
}

#[test]
fn unsupported_geometry_and_symbol_roles_fail_before_native_solve() {
    let x = Atom::var(symbol!("gcad_test::x"));
    let a = symbol!("gcad_test::a");
    let source = input(
        x.clone() - Atom::var(a),
        Atom::num(-1),
        FactorSemantics::Causal,
    );
    assert!(matches!(
        GcadRequest::unit_cube(
            &source,
            GcadKinematics::default(),
            SolverOptions::default(),
            limits()
        ),
        Err(GcadError::Unsupported(_))
    ));
    let roles = GcadKinematics {
        runtime_parameters: vec![symbol!("gcad_test::x")],
        ..GcadKinematics::default()
    };
    assert!(matches!(
        GcadRequest::unit_cube(&source, roles, SolverOptions::default(), limits()),
        Err(GcadError::Invalid(_))
    ));
    let source = input(x * parse!("I"), Atom::num(-1), FactorSemantics::Causal);
    assert!(matches!(
        GcadRequest::unit_cube(
            &source,
            GcadKinematics::default(),
            SolverOptions::default(),
            limits()
        ),
        Err(GcadError::Unsupported(_))
    ));
    let source = input(Atom::var(a), Atom::num(-1), FactorSemantics::Causal);
    let kinematics = GcadKinematics {
        exact_values: BTreeMap::from([(a, Rational::from(0))]),
        ..GcadKinematics::default()
    };
    assert!(matches!(
        GcadRequest::unit_cube(&source, kinematics, SolverOptions::default(), limits()),
        Err(GcadError::Invalid(_))
    ));
}

#[test]
fn projective_domain_never_receives_an_implicit_cube_map() {
    let source = ParametricIntegrand::new(
        vec![symbol!("gcad_test::x")],
        symbol!("gcad_test::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(-1)],
            vec![],
        )],
    )
    .unwrap();
    assert!(matches!(
        GcadRequest::unit_cube(
            &source,
            GcadKinematics::default(),
            SolverOptions::default(),
            limits()
        ),
        Err(GcadError::Unsupported(_))
    ));
    let domain = PreparedDomain::explicit(
        source.parameters().to_vec(),
        vec![],
        "no gauge proof".into(),
    )
    .unwrap();
    assert!(matches!(
        GcadRequest::prepared(
            &source,
            domain,
            GcadKinematics::default(),
            SolverOptions::default(),
            limits()
        ),
        Err(GcadError::Unsupported(_))
    ));
}
