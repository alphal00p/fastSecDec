use super::*;
#[test]
fn branch_identity_is_independent_of_optimized_transport() {
    use crate::threshold::maps::{
        CellMap,
        regular::{BracketProposal, Budget, RegularSection, SectionSide, callback::RootProgram},
    };
    let (x, y, e, s, t, tag) = symbol!(
        "root_bracket_gate::x",
        "root_bracket_gate::y",
        "root_bracket_gate::eps",
        "root_bracket_gate::s",
        "root_bracket_gate::t",
        "root_bracket_gate::tag"
    );
    let polynomial =
        Atom::var(y).pow(3) + Atom::var(y) - Atom::num((10, 27)) - Atom::var(x) / Atom::num(2);
    let input = ParametricIntegrand::new(
        vec![x, y],
        e,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::zero(); 2],
            vec![
                PolynomialFactor::new(polynomial, -Atom::var(e), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let owner = Arc::new(
        GcadRequest::unit_cube(
            &input,
            Default::default(),
            SolverOptions {
                order: vec!["v0".into(), "v1".into()],
                ..Default::default()
            },
            GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    assert_eq!(owner.cells().len(), 2);
    let map = CellMap::new(owner, 0, vec![s, t]).unwrap();
    for lower in [
        Rational::from((1, 3)),
        Rational::from((1, 3)) - Rational::from((1, 1_000_000)),
    ] {
        let section = RegularSection::prepare(
            map.clone(),
            1,
            SectionSide::Upper,
            BracketProposal {
                lower,
                upper: Rational::one(),
                derivative_margin: Rational::from((1, 2)),
            },
            &mut Budget::new(Default::default()),
        )
        .unwrap();
        assert_eq!(section.coefficients().len(), 4);
        assert!(matches!(
            RootProgram::prepare(&section, Default::default()),
            Err(crate::threshold::maps::regular::Error::Unsupported(_))
        ));
    }
    let section = RegularSection::prepare(
        map,
        1,
        SectionSide::Upper,
        BracketProposal {
            lower: Rational::zero(),
            upper: Rational::one(),
            derivative_margin: Rational::from((1, 2)),
        },
        &mut Budget::new(Default::default()),
    )
    .unwrap();
    let helper = RootProgram::prepare(&section, Default::default()).unwrap();
    let unoptimized = RootProgram::prepare(
        &section,
        symbolica::evaluate::OptimizationSettings::default()
            .horner_iterations(0)
            .cpe_iterations(Some(0)),
    )
    .unwrap();
    assert_eq!(helper.semantic_identity(), unoptimized.semantic_identity());
    assert_ne!(helper.bytes(), unoptimized.bytes());
    assert_ne!(helper.transport_symbol(), unoptimized.transport_symbol());
    let call = helper.call(tag, section.coefficients()).unwrap();
    let exact = Atom::evaluator_multiple(&[call.clone(), call.derivative(x)], &[Atom::var(x)])
        .build()
        .unwrap();
    let mut scope = crate::kernel::algebraic::Scope::default();
    scope.insert(tag, helper).unwrap();
    let mut evaluator = scope.enter(53, || exact.map_coeff(&|c| c.re.to_f64()));
    let algebra =
        symgcad::algebra::Algebra::new(&section.source().decomposition().native_result().order)
            .unwrap();
    let mut rows = Vec::new();
    for point in [
        Rational::zero(),
        Rational::from((1, 1_000_000)),
        Rational::from((1, 8)),
    ] {
        let fiber = algebra
            .specialize_univariate(section.polynomial(), 1, std::slice::from_ref(&point))
            .unwrap();
        let mut roots = symgcad::roots::isolate_union(vec![(0, fiber)]).unwrap();
        assert_eq!(roots.len(), 1);
        for _ in 0..72 {
            symgcad::roots::refine_once(&mut roots[0]);
        }
        let oracle = (&roots[0].interval.0 + &roots[0].interval.1).to_f64() / 2.;
        let mut out = [0.; 2];
        crate::kernel::algebraic::attempt(|| evaluator.evaluate(&[point.to_f64()], &mut out))
            .unwrap();
        assert!((out[0] - oracle).abs() < 2e-15);
        assert!((out[1] - 0.5 / (3. * oracle * oracle + 1.)).abs() < 3e-15);
        rows.push(out);
    }
}
