use super::ProbeResult;
use serde_json::{Value, json};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::{atom::AtomField, rational::Q},
    parse,
    poly::{PolyVariable, groebner::GroebnerBasis, polynomial::MultivariatePolynomial},
    symbol,
    tensors::matrix::Matrix,
};

pub(super) fn run() -> ProbeResult<Value> {
    let (x, y, z, p, u, v) = symbol!(
        "threshold_probe_geometry::x",
        "threshold_probe_geometry::y",
        "threshold_probe_geometry::z",
        "threshold_probe_geometry::p",
        "threshold_probe_geometry::u",
        "threshold_probe_geometry::v"
    );
    let variables: Arc<Vec<PolyVariable>> = Arc::new(
        vec![x, y, z, p, u, v]
            .into_iter()
            .map(PolyVariable::from)
            .collect(),
    );
    let poly = |a: Atom| -> MultivariatePolynomial<_, u16> {
        a.to_polynomial(&Q, Some(variables.clone()))
    };
    let (x, y, z, p, ua, va) = (
        Atom::var(x),
        Atom::var(y),
        Atom::var(z),
        Atom::var(p),
        Atom::var(u),
        Atom::var(v),
    );
    // Blow up the relative smooth center (x,y-p*z^2), leaving p and z fixed.
    let equations = [poly(&x - &ua), poly(&y - &p * z.pow(2) - &ua * &va)];
    let basis = GroebnerBasis::new(&equations, false);
    assert!(GroebnerBasis::is_groebner_basis(&basis.system));
    let positive_dimension_error = basis
        .solve()
        .expect_err("a positive-dimensional chart graph is not a finite root list");
    let center = [x.clone(), &y - &p * z.pow(2)];
    let center_axes = [
        x.as_var_view().unwrap().get_symbol(),
        y.as_var_view().unwrap().get_symbol(),
    ];
    let center_minor = Matrix::from_linear(
        center
            .iter()
            .flat_map(|equation| center_axes.map(|axis| equation.derivative(axis)))
            .collect(),
        2,
        2,
        AtomField {
            statistical_zero_test: false,
            cancel_check_on_division: true,
            ..AtomField::new()
        },
    )?
    .det()?;
    assert_eq!(center_minor, Atom::num(1));
    let source = (&y - &p * z.pow(2)).pow(2) + x.pow(3);
    let expected = ua.pow(2) * (va.pow(2) + &ua);
    assert!(poly(&source - &expected).reduce(&basis.system).is_zero());
    // A missing exceptional factor must be detected, even though point
    // comparisons at an exceptional endpoint could accidentally agree.
    assert!(
        !poly(&source - (va.pow(2) + &ua))
            .reduce(&basis.system)
            .is_zero()
    );

    let images = [ua.clone(), &p * z.pow(2) + &ua * &va, z.clone()];
    let coordinates = [u, v, z.as_var_view().unwrap().get_symbol()];
    let entries = images
        .iter()
        .flat_map(|image| coordinates.iter().map(move |axis| image.derivative(*axis)))
        .collect();
    let field = AtomField {
        statistical_zero_test: false,
        cancel_check_on_division: true,
        ..AtomField::new()
    };
    let determinant = Matrix::from_linear(entries, 3, 3, field)?.det()?;
    assert_eq!((determinant - &ua).expand(), Atom::Zero);
    assert_eq!(images[2], z);
    assert_eq!(p.derivative(u), Atom::Zero);

    // Algebraic cell 0<y<sqrt(x), 0<x<1: x=u^2, y=u*v,
    // positive root r=u and J=2*u^2. This verifies identities, not a CAD cover.
    let r = symbol!("threshold_probe_geometry::r");
    let r = Atom::var(r);
    let cell_variables: Arc<Vec<PolyVariable>> = Arc::new(
        [x.clone(), y.clone(), r.clone(), ua.clone(), va.clone()]
            .iter()
            .map(|a| PolyVariable::from(a.as_var_view().unwrap().get_symbol()))
            .collect(),
    );
    let cell_poly = |a: Atom| -> MultivariatePolynomial<_, u16> {
        a.to_polynomial(&Q, Some(cell_variables.clone()))
    };
    let cell_basis = GroebnerBasis::new(
        &[
            cell_poly(&x - ua.pow(2)),
            cell_poly(&y - &ua * &va),
            cell_poly(&r - &ua),
        ],
        false,
    );
    assert!(
        cell_poly(r.pow(2) - &x)
            .reduce(&cell_basis.system)
            .is_zero()
    );
    assert!(
        cell_poly(&r - &y - &ua * (Atom::num(1) - &va))
            .reduce(&cell_basis.system)
            .is_zero()
    );
    let cell_images = [ua.pow(2), &ua * &va];
    let entries = cell_images
        .iter()
        .flat_map(|image| [u, v].map(|axis| image.derivative(axis)))
        .collect();
    let determinant = Matrix::from_linear(
        entries,
        2,
        2,
        AtomField {
            statistical_zero_test: false,
            cancel_check_on_division: true,
            ..AtomField::new()
        },
    )?
    .det()?;
    assert_eq!(
        (determinant - parse!("2*threshold_probe_geometry::u^2")).expand(),
        Atom::Zero
    );
    Ok(json!({
        "relative_center_graph_identities": true,
        "relative_center_rank_minor": "1",
        "finite_root_enumerator_refuses_chart_graph": positive_dimension_error,
        "incorrect_exceptional_factor_rejected": true,
        "native_bareiss_jacobians": ["u", "2*u^2"],
        "parameter_kept_fixed": true,
        "algebraic_cell_gap": "r-y=u*(1-v)",
        "real_branch_condition": "u>=0; 0<=v<=1",
        "general_resolution_or_coverage_proved": false
    }))
}
