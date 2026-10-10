use super::super::*;
use super::{
    CartierDivision, FactorProduction, OldBoundaryProduction, QuotientMethod,
    ResidualOrderProduction, divide_cartier, factor_whole_cartier_equations,
    produce_old_boundary_coefficient, produce_restricted_residual_order,
};
use std::sync::Arc;
use symbolica::symbol;

fn ledger(
    frame: Arc<EtaleFrame>,
    equations: Vec<Poly>,
    b: &mut Budget,
) -> Arc<VerifiedRelativeSnc> {
    match verify_initial_relative_snc(
        frame,
        equations
            .into_iter()
            .enumerate()
            .map(|(i, equation)| InitialDivisor {
                id: BoundaryId(i as u64 + 1),
                equation,
            })
            .collect(),
        b,
    )
    .unwrap()
    {
        SncProduction::Verified(owner) => owner,
        outcome => panic!("missing ledger {outcome:?}"),
    }
}
fn marked(r: &Arc<Ring>, expression: &str, mark: usize, b: &mut Budget) -> Arc<MarkedIdeal> {
    Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![p(r, expression)], b).unwrap(),
            mark,
            b,
        )
        .unwrap(),
    )
}

#[test]
fn companion_probe_production_cartier_discovery_and_factorization() {
    let r = ring();
    let mut b = budget();
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let s = local(&r, vec![&y - &(&x * &x)], vec![1, 2], vec![], &mut b);
    let f = relative(s, &mut b);
    let owner = ledger(f, vec![x.clone()], &mut b);
    let result =
        divide_cartier(owner.clone(), 0, y.clone(), "production_division", &mut b).unwrap();
    let CartierDivision::Quotient(q) = result else {
        panic!("missing quotient")
    };
    assert_eq!(q.method(), QuotientMethod::NativeElimination);
    assert!(Arc::ptr_eq(q.owner(), &owner));
    assert!(
        owner
            .frame()
            .local()
            .zero(&(q.quotient() - &x), &mut b)
            .unwrap()
    );
    let source = marked(&r, "companion_probe::y^2", 1, &mut b);
    let FactorProduction::WholeEquations(factors) = factor_whole_cartier_equations(
        owner.clone(),
        source.clone(),
        8,
        "production_factor",
        &mut b,
    )
    .unwrap() else {
        panic!("missing factors")
    };
    assert_eq!(factors.progress().powers, vec![4]);
    assert_eq!(factors.progress().recombinations, 1);
    assert_eq!(factors.indivisible_generators(), &[Some(0)]);
    let ResidualOrderProduction::Order(order) =
        produce_restricted_residual_order(Arc::new(*factors), &mut b).unwrap()
    else {
        panic!("missing order")
    };
    assert_eq!(order.algebraic_maximum_on_cosupport(), 0);
    assert!(order.companion_arithmetic().is_none());
    assert!(matches!(
        factor_whole_cartier_equations(owner, source, 2, "production_cap", &mut b).unwrap(),
        FactorProduction::Incomplete { .. }
    ));
}

#[test]
fn companion_probe_production_restricted_order_and_parameter_refusal() {
    let r = ring();
    let mut b = budget();
    let s = local(&r, vec![], vec![1, 2], vec![], &mut b);
    let f = flat(s, &mut b);
    let owner = ledger(f, vec![r.coordinate(1).unwrap()], &mut b);
    for (expression, mark, expected) in [
        (
            "companion_probe::x^4*companion_probe::y*(companion_probe::x-1)^2",
            5,
            1,
        ),
        ("companion_probe::x^4*(companion_probe::x-1)^2", 4, 0),
    ] {
        let source = marked(&r, expression, mark, &mut b);
        let FactorProduction::WholeEquations(factors) = factor_whole_cartier_equations(
            owner.clone(),
            source,
            8,
            "production_restricted",
            &mut b,
        )
        .unwrap() else {
            panic!("missing factors")
        };
        let ResidualOrderProduction::Order(order) =
            produce_restricted_residual_order(Arc::new(*factors), &mut b).unwrap()
        else {
            panic!("missing order")
        };
        assert_eq!(order.algebraic_maximum_on_cosupport(), expected);
        assert!(!order.upper_order_cover().algebraic_locus_empty());
        assert_eq!(
            order.companion_arithmetic().map(MarkedIdeal::mark),
            if expected == 0 { None } else { Some(4) }
        );
    }
    let source = marked(&r, "companion_probe::p*companion_probe::x", 1, &mut b);
    let FactorProduction::WholeEquations(factors) =
        factor_whole_cartier_equations(owner, source, 8, "production_param", &mut b).unwrap()
    else {
        panic!("missing factors")
    };
    b.limits.max_mark = 3;
    let ResidualOrderProduction::Incomplete { progress, .. } =
        produce_restricted_residual_order(Arc::new(*factors), &mut b).unwrap()
    else {
        panic!("parameter locus silently resolved")
    };
    assert_eq!(progress.residual_layers.len(), 4);
    assert!(
        progress
            .residual_layers
            .iter()
            .all(|layer| layer.unit_on_cosupport == Some(false))
    );
}

#[test]
fn companion_probe_production_old_coefficient_uses_checked_source_history() {
    let r = ring();
    let mut b = budget();
    let s = local(&r, vec![], vec![1, 2], vec![], &mut b);
    let f = flat(s, &mut b);
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let owner = ledger(f.clone(), vec![x.clone(), y.clone(), &x - &r.one()], &mut b);
    let history = ResolutionHistory::initial(owner).unwrap();
    let source = marked(&r, "companion_probe::x^2+companion_probe::y^3", 2, &mut b);
    let OrderProduction::ContactCover(cover) =
        produce_ordinary_contact_cover(f.clone(), Arc::new(source.ideal().clone()), &mut b)
            .unwrap()
    else {
        panic!("missing contact")
    };
    let index = cover
        .candidates()
        .iter()
        .position(|c| {
            c.free_derivative_index == 0 && c.equation.degree(1) == 1 && c.equation.degree(2) == 0
        })
        .unwrap();
    let ContactProduction::Constructed(contact) = construct_contact_quotient(
        Arc::new(cover),
        index,
        [symbol!("production_old::z"), symbol!("production_old::u")],
        &mut b,
    )
    .unwrap() else {
        panic!("missing coefficient")
    };
    let contact = Arc::new(*contact);
    let OldBoundaryProduction::Coefficient(stage) =
        produce_old_boundary_coefficient(contact.clone(), history, &mut b).unwrap()
    else {
        panic!("missing old boundary stage")
    };
    assert_eq!(stage.algebraic_maximum_old_count(), 2);
    assert_eq!(stage.progress().boundary_factors_completed, 3);
    let target = contact.contact().local();
    let y = contact.extension().pull(&y, &mut b).unwrap();
    let expected = MarkedIdeal::new(
        Ideal::new(target.ring().clone(), vec![&y * &y], &mut b).unwrap(),
        2,
        &b,
    )
    .unwrap();
    assert!(
        QuotientNormalizer::prepare(target.clone(), &mut b)
            .unwrap()
            .verify_candidate(stage.coefficient(), &expected, &mut b)
            .unwrap()
    );
    let other = ledger(Arc::new((*f).clone()), vec![x], &mut b);
    assert!(
        produce_old_boundary_coefficient(
            contact,
            ResolutionHistory::initial(other).unwrap(),
            &mut b
        )
        .is_err()
    );
}

#[test]
fn companion_probe_production_whole_equation_scope_and_zero_controls() {
    let r = ring();
    let mut b = budget();
    let s = local(&r, vec![], vec![1], vec![], &mut b);
    let frame = flat(s, &mut b);
    let h = p(&r, "companion_probe::x*(companion_probe::x-1)");
    let owner = ledger(frame, vec![h], &mut b);
    let source = marked(&r, "companion_probe::x^2", 1, &mut b);
    let FactorProduction::WholeEquations(factors) = factor_whole_cartier_equations(
        owner.clone(),
        source.clone(),
        8,
        "production_disconnected",
        &mut b,
    )
    .unwrap() else {
        panic!("missing factors")
    };
    assert_eq!(factors.progress().powers, vec![0]); // Explicitly whole-equation only.
    let zero =
        Arc::new(MarkedIdeal::new(Ideal::new(r.clone(), vec![], &mut b).unwrap(), 1, &b).unwrap());
    assert!(matches!(
        factor_whole_cartier_equations(owner.clone(), zero, 8, "production_zero", &mut b).unwrap(),
        FactorProduction::ZeroIdeal { .. }
    ));
    let mut cap = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        factor_whole_cartier_equations(owner, source, 8, "production_budget", &mut cap).unwrap(),
        FactorProduction::Incomplete { .. }
    ));
}

fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("companion_probe::p"),
                symbol!("companion_probe::x"),
                symbol!("companion_probe::y"),
                symbol!("companion_probe::u"),
                symbol!("companion_probe::v"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn budget() -> Budget {
    Budget::new(Limits::default())
}
fn p(r: &Ring, expression: &str) -> Poly {
    r.atom(
        &symbolica::atom::Atom::parse(expression, "companion_probe", Default::default()).unwrap(),
    )
    .unwrap()
}
fn local(
    r: &Arc<Ring>,
    equations: Vec<Poly>,
    axes: Vec<usize>,
    guards: Vec<Guard>,
    b: &mut Budget,
) -> Arc<LocalizedAlgebra> {
    LocalizedAlgebra::new(
        Ideal::new(r.clone(), equations, b).unwrap(),
        axes,
        guards,
        b,
    )
    .unwrap()
}
fn flat(s: Arc<LocalizedAlgebra>, b: &mut Budget) -> Arc<EtaleFrame> {
    Arc::new(
        EtaleCertificate {
            free_axes: s.axes().to_vec(),
            source: s,
            equations: vec![],
            dependent_axes: vec![],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn relative(s: Arc<LocalizedAlgebra>, b: &mut Budget) -> Arc<EtaleFrame> {
    Arc::new(
        EtaleCertificate {
            free_axes: vec![1],
            source: s,
            equations: vec![0],
            dependent_axes: vec![2],
            determinant_inverse_axis: Some(4),
        }
        .verify(b)
        .unwrap(),
    )
}
