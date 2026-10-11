use super::*;
use crate::threshold::resolution::{Limits, Ring};
use std::sync::Arc;
use symbolica::{atom::Atom, symbol};
fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("basis_test::x"),
                symbol!("basis_test::u"),
                symbol!("basis_test::z"),
                symbol!("basis_test::v"),
            ],
            vec![],
        )
        .unwrap(),
    )
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "basis_test", Default::default()).unwrap())
        .unwrap()
}
fn b() -> Budget {
    Budget::new(Limits::default())
}
#[test]
fn native_basis_zero_inputs_and_original_inclusion() {
    let r = ring();
    let f = p(&r, "basis_test::x");
    let zero = r.one().zero();
    let mut b = b();
    assert!(checked(&[], &mut b).unwrap().is_empty());
    assert!(
        checked(std::slice::from_ref(&zero), &mut b)
            .unwrap()
            .is_empty()
    );
    let out = checked(&[zero, f.clone()], &mut b).unwrap();
    assert!(f.reduce(&out).is_zero());
    assert!(matches!(
        verify(&[f], &[], &mut b),
        Err(Error::Invalid("native basis loses an input equation"))
    ));
}
#[test]
fn native_basis_original_f4_cache_regression_and_closure_check() {
    let r = ring();
    let input = [
        "basis_test::x^2-basis_test::x-basis_test::z",
        "basis_test::u-basis_test::x*basis_test::u-1",
        "2*basis_test::x*basis_test::v-basis_test::v-1",
    ]
    .map(|s| p(&r, s));
    let mut b = b();
    let out = checked(&input, &mut b).unwrap();
    assert!(
        p(&r, "(4*basis_test::z+1)*basis_test::v^2-1")
            .reduce(&out)
            .is_zero()
    );
    assert!(matches!(
        verify(&input, &input, &mut b),
        Err(Error::Invalid("native basis fails Buchberger closure"))
    ));
    let reproduced_incomplete = [
        "basis_test::x-1/2-2*basis_test::z*basis_test::v-basis_test::v/2",
        "basis_test::u*basis_test::z+2*basis_test::z*basis_test::v+basis_test::v/2+1/2",
        "basis_test::u*basis_test::v-basis_test::u-2*basis_test::v",
    ]
    .map(|s| p(&r, s));
    assert!(matches!(
        verify(&input, &reproduced_incomplete, &mut b),
        Err(Error::Invalid("native basis fails Buchberger closure"))
    ));
}
#[test]
fn native_basis_shape_and_spair_allocation_caps() {
    let r = ring();
    let foreign = Arc::new(Ring::new(vec![symbol!("basis_test::foreign")], vec![]).unwrap());
    let mut b = b();
    assert!(matches!(
        checked(&[r.one(), foreign.one()], &mut b),
        Err(Error::Invalid("native basis input variable maps"))
    ));
    let basis = [
        p(&r, "basis_test::x^2+basis_test::u^2"),
        p(&r, "basis_test::x*basis_test::u+1"),
    ];
    let mut cap = Budget::new(Limits {
        max_degree_per_axis: 2,
        ..Limits::default()
    });
    assert!(matches!(
        verify(&[], &basis, &mut cap),
        Err(Error::ResourceIncomplete("native S-polynomial degree"))
    ));
    let mut cap = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        verify(&[], &basis, &mut cap),
        Err(Error::ResourceIncomplete("operation budget"))
    ));
}

#[path = "membership_tests.rs"]
mod membership_tests;
