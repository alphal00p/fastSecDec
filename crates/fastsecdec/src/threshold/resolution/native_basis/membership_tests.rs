use super::*;
use crate::threshold::resolution::Ideal;
use symbolica::poly::GrevLexOrder;

#[test]
fn membership_direct_is_only_one_way_and_order_agrees() {
    let r = ring();
    let input = [p(&r, "x+u"), p(&r, "x-u")];
    let query = p(&r, "u");
    assert!(!query.reduce(&input).is_zero());
    let ideal = Ideal::new(r.clone(), input.to_vec(), &mut b()).unwrap();
    assert!(ideal.contains(&query, &[], &mut b()).unwrap());
    assert!(!ideal.contains(&p(&r, "z"), &[], &mut b()).unwrap());
    for equations in [
        input.to_vec(),
        vec![p(&r, "x^2-z"), p(&r, "x*u-1")],
        vec![p(&r, "x^2-x-z"), p(&r, "u-x*u-1"), p(&r, "2*x*v-v-1")],
    ] {
        let lex = checked(&equations, &mut b()).unwrap();
        let reordered = equations
            .iter()
            .map(|f| f.reorder::<GrevLexOrder>())
            .collect::<Vec<_>>();
        let grevlex = checked_ordered(&reordered, &mut b()).unwrap();
        let ideal = Ideal::new(r.clone(), equations.clone(), &mut b()).unwrap();
        for query in equations.iter().cloned().chain([
            r.one(),
            p(&r, "x"),
            p(&r, "u"),
            p(&r, "z*u^2-1"),
            p(&r, "(4*z+1)*v^2-1"),
        ]) {
            let expected = query.reduce(&lex).is_zero();
            assert_eq!(
                query.reorder::<GrevLexOrder>().reduce(&grevlex).is_zero(),
                expected
            );
            assert_eq!(ideal.contains(&query, &[], &mut b()).unwrap(), expected);
        }
    }
}

#[test]
fn membership_fast_path_preserves_admission_and_budget() {
    let r = ring();
    let f = p(&r, "x");
    let ideal = Ideal::new(r.clone(), vec![f.clone()], &mut b()).unwrap();
    let mut one = Budget::new(Limits {
        max_operations: 1,
        ..Limits::default()
    });
    assert!(ideal.contains(&f, &[], &mut one).unwrap());
    assert_eq!(one.operations(), 1);
    let mut zero = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        ideal.contains(&f, &[], &mut zero),
        Err(Error::ResourceIncomplete("operation budget"))
    ));
    let mut slots = Budget::new(Limits {
        max_total_ideal_slots: 0,
        ..Limits::default()
    });
    assert!(matches!(
        ideal.contains(&f, &[], &mut slots),
        Err(Error::ResourceIncomplete("total ideal allocation budget"))
    ));
    let foreign = Ring::new(vec![symbol!("basis_test::foreign")], vec![]).unwrap();
    assert!(matches!(
        ideal.contains(&foreign.one(), &[], &mut b()),
        Err(Error::Invalid("polynomial variable map"))
    ));
    // A zero query must still validate supplied quotient equations.
    assert!(matches!(
        ideal.contains(&f.zero(), &[foreign.one()], &mut b()),
        Err(Error::Invalid("polynomial variable map"))
    ));
    let mut terms = Budget::new(Limits {
        max_terms: 1,
        ..Limits::default()
    });
    assert!(matches!(
        ideal.contains(&f, &[p(&r, "u+z")], &mut terms),
        Err(Error::ResourceIncomplete("polynomial term budget"))
    ));
    assert!(
        ideal
            .contains(&p(&r, "u"), &[p(&r, "u-x")], &mut b())
            .unwrap()
    );
}

#[test]
fn membership_zero_source_and_remainder_output_cap() {
    let r = ring();
    let zero = r.one().zero();
    let empty = Ideal::new(r.clone(), vec![zero.clone()], &mut b()).unwrap();
    assert!(empty.generators().is_empty());
    assert!(empty.contains(&zero, &[], &mut b()).unwrap());
    assert!(!empty.contains(&r.one(), &[], &mut b()).unwrap());
    let ideal = Ideal::new(r.clone(), vec![p(&r, "x-u^2")], &mut b()).unwrap();
    let query = p(&r, "x^2");
    assert_eq!(query.reduce(ideal.generators()), p(&r, "u^4"));
    let mut capped = Budget::new(Limits {
        max_degree_per_axis: 2,
        ..Limits::default()
    });
    assert!(matches!(
        ideal.contains(&query, &[], &mut capped),
        Err(Error::ResourceIncomplete("polynomial degree budget"))
    ));
}
