use super::*;
use crate::threshold::resolution::{
    Budget, EtaleCertificate, Ideal, Limits, LocalizedAlgebra, Ring,
};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{
        Field, Ring as _, RingOps,
        algebraic::Root,
        rational::{Q, Rational},
    },
    poly::PolyVariable,
    symbol,
};
fn budget() -> Budget {
    Budget::new(Limits {
        max_terms: 2_000_000,
        max_operations: 2_000_000,
        ..Default::default()
    })
}
fn affine(
    symbols: Vec<Symbol>,
    params: Vec<usize>,
    free: Vec<usize>,
    b: &mut Budget,
) -> Arc<crate::threshold::resolution::EtaleFrame> {
    let ring = Arc::new(Ring::new(symbols, params).unwrap());
    let local = LocalizedAlgebra::new(
        Ideal::new(ring, vec![], b).unwrap(),
        free.clone(),
        vec![],
        b,
    )
    .unwrap();
    Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: free,
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn fresh(prefix: &str, n: usize) -> Vec<Symbol> {
    (0..n)
        .map(|i| symbol!(format!("aj_owner::{prefix}_{i}")))
        .collect()
}
#[test]
fn rational_factor_owner_preserves_bulk_polynomial_and_mixed_jets() {
    let mut b = budget();
    let x = symbol!("aj_owner::x");
    let y = symbol!("aj_owner::y");
    let z = symbol!("aj_owner::z");
    let frame = affine(vec![x, y], vec![], vec![0, 1], &mut b);
    let r = frame.local().ring();
    let source = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![
            r.atom(&(-Atom::one() - Atom::var(x) - Atom::var(y)))
                .unwrap(),
            r.one().zero(),
            r.one(),
        ],
        &mut b,
    )
    .unwrap();
    let seed = FactorSeed::verify(
        source,
        vec![Atom::Zero, Atom::Zero],
        vec![Atom::num(-1), Atom::one()],
        vec![Atom::one(), Atom::one()],
        &mut b,
    )
    .unwrap();
    let branch = FactorBranch::prepare(seed, &fresh("rational", 3), &mut b).unwrap();
    let f = branch.seed().context().field();
    assert_eq!(
        branch.jet(false, 0, &[0], &mut b).unwrap().at_fiber(),
        &f.constant(Rational::from((-1, 2)))
    );
    assert_eq!(
        branch.jet(false, 0, &[0, 1], &mut b).unwrap().at_fiber(),
        &f.constant(Rational::from((1, 4)))
    );
    assert_eq!(
        branch.jet(false, 0, &[1, 0], &mut b).unwrap().at_fiber(),
        &f.constant(Rational::from((1, 4)))
    );
    assert_eq!(branch.seed().polynomial().coefficients().len(), 3);
    assert!(branch.constant_axis().is_none());
}
#[test]
fn nested_real_constant_embedding_preserves_parameter_inertia() {
    let mut b = budget();
    let x = symbol!("aj_owner::nx");
    let p = symbol!("aj_owner::np");
    let z = symbol!("aj_owner::nz");
    let frame = affine(vec![x, p], vec![1], vec![0], &mut b);
    let r = frame.local().ring();
    let source = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![
            r.atom(&(-Atom::var(p) - Atom::var(x))).unwrap(),
            r.one().zero(),
            r.one(),
        ],
        &mut b,
    )
    .unwrap();
    let sqrt2 = Atom::num(2).pow(Atom::num(Rational::from((1, 2))));
    let pvalue = Atom::num(2) + sqrt2;
    let beta = pvalue.clone().pow(Atom::num(Rational::from((1, 2))));
    let seed = FactorSeed::verify(
        source,
        vec![Atom::Zero, pvalue],
        vec![-&beta, Atom::one()],
        vec![beta.clone(), Atom::one()],
        &mut b,
    )
    .unwrap();
    assert_eq!(seed.context().field().poly().degree(0), 4);
    let branch = FactorBranch::prepare(seed, &fresh("nested", 4), &mut b).unwrap();
    let field = branch.seed().context().field();
    let beta = branch.seed().context().image(&beta).unwrap();
    assert_eq!(
        branch.jet(false, 0, &[0], &mut b).unwrap().at_fiber(),
        &field.div(
            &field.constant(Rational::from(-1)),
            &field.mul(&field.constant(Rational::from(2)), beta)
        )
    );
    let a = branch
        .frame()
        .local()
        .ring()
        .coordinate(branch.constant_axis().unwrap())
        .unwrap();
    assert!(branch.frame().derivative(0, &a, &mut b).unwrap().is_zero());
    let p = branch.frame().local().ring().coordinate(1).unwrap();
    assert!(branch.frame().derivative(0, &p, &mut b).unwrap().is_zero());
}
#[test]
fn degree_five_native_selected_root_is_not_a_quadratic_special_case() {
    let mut b = budget();
    b.limits.max_terms = 100_000_000;
    let x = symbol!("aj_owner::qx");
    let z = symbol!("aj_owner::qz");
    let t = symbol!("aj_owner::root_t");
    let frame = affine(vec![x], vec![], vec![0], &mut b);
    let r = frame.local().ring();
    let source = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![
            r.atom(&(-Atom::one() - Atom::var(x))).unwrap(),
            -r.one(),
            r.one().zero(),
            r.one().zero(),
            r.one().zero(),
            r.one(),
        ],
        &mut b,
    )
    .unwrap();
    let root = Root::<Q>::from_atom_with_variable(
        (Atom::var(t).pow(5u64) - Atom::var(t) - Atom::one()).as_view(),
        PolyVariable::from(t),
        0,
    )
    .unwrap()
    .to_atom();
    let left = vec![-&root, Atom::one()];
    let right = vec![
        root.clone().pow(4u64) - Atom::one(),
        root.clone().pow(3u64),
        root.clone().pow(2u64),
        root.clone(),
        Atom::one(),
    ];
    let seed = FactorSeed::verify(source, vec![Atom::Zero], left, right, &mut b).unwrap();
    let branch = FactorBranch::prepare(seed, &fresh("quintic", 7), &mut b).unwrap();
    assert_eq!(branch.seed().polynomial().degree(), 5);
    assert_eq!(branch.factors().1.len(), 5);
    let field = branch.seed().context().field();
    let rho = branch.seed().context().image(&root).unwrap();
    let expected = field.div(
        &field.constant(Rational::from(-1)),
        &field.sub(
            &field.mul(&field.constant(Rational::from(5)), &field.pow(rho, 4)),
            &field.one(),
        ),
    );
    assert_eq!(
        branch.jet(false, 0, &[0], &mut b).unwrap().at_fiber(),
        &expected
    );
}
#[test]
fn incorrect_nonreal_or_unbounded_seeds_are_refused() {
    let mut b = budget();
    let x = symbol!("aj_owner::rx");
    let z = symbol!("aj_owner::rz");
    let frame = affine(vec![x], vec![], vec![0], &mut b);
    let r = frame.local().ring();
    let source = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![-r.one(), r.one().zero(), r.one()],
        &mut b,
    )
    .unwrap();
    assert!(
        FactorSeed::verify(
            source.clone(),
            vec![Atom::Zero],
            vec![Atom::one(), Atom::one()],
            vec![Atom::one(), Atom::one()],
            &mut b
        )
        .is_err()
    );
    assert!(
        FactorSeed::verify(
            source.clone(),
            vec![Atom::num(0.1)],
            vec![Atom::num(-1), Atom::one()],
            vec![Atom::one(), Atom::one()],
            &mut b
        )
        .is_err()
    );
    let seed = FactorSeed::verify(
        source,
        vec![Atom::Zero],
        vec![Atom::num(-1), Atom::one()],
        vec![Atom::one(), Atom::one()],
        &mut b,
    )
    .unwrap();
    assert!(
        FactorBranch::prepare(
            seed.clone(),
            &[x, symbol!("aj_owner::alias1"), symbol!("aj_owner::alias2")],
            &mut b
        )
        .is_err()
    );
    let mut limited = Budget::new(Limits {
        max_generators: 1,
        ..Default::default()
    });
    assert!(FactorBranch::prepare(seed, &fresh("limited", 3), &mut limited).is_err());
    let plus = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![r.one(), r.one().zero(), r.one()],
        &mut b,
    )
    .unwrap();
    let i = Atom::num(-1).pow(Atom::num(Rational::from((1, 2))));
    assert!(
        FactorSeed::verify(
            plus,
            vec![Atom::Zero],
            vec![-&i, Atom::one()],
            vec![i, Atom::one()],
            &mut b
        )
        .is_err()
    );
    let repeated = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![r.one(), r.atom(&Atom::num(-2)).unwrap(), r.one()],
        &mut b,
    )
    .unwrap();
    assert!(
        FactorSeed::verify(
            repeated,
            vec![Atom::Zero],
            vec![Atom::num(-1), Atom::one()],
            vec![Atom::num(-1), Atom::one()],
            &mut b
        )
        .is_err()
    );
}
#[test]
fn existing_etale_base_guards_and_selected_seed_order_are_preserved() {
    let mut b = budget();
    let x = symbol!("aj_owner::bx");
    let y = symbol!("aj_owner::by");
    let p = symbol!("aj_owner::bp");
    let q = symbol!("aj_owner::bq");
    let z = symbol!("aj_owner::bz");
    let ring = Arc::new(Ring::new(vec![x, y, p, q], vec![2]).unwrap());
    let local = LocalizedAlgebra::new(
        Ideal::new(
            ring.clone(),
            vec![
                ring.atom(&(Atom::var(y).pow(2u64) - Atom::var(x) - Atom::num(2)))
                    .unwrap(),
            ],
            &mut b,
        )
        .unwrap(),
        vec![0, 1],
        vec![],
        &mut b,
    )
    .unwrap();
    let base = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![0],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let source = MonicPolynomial::new(
        base.clone(),
        z,
        vec![
            ring.atom(&(-Atom::var(y) - Atom::var(p))).unwrap(),
            ring.one().zero(),
            ring.one(),
        ],
        &mut b,
    )
    .unwrap();
    let point = vec![
        Atom::num(-1),
        Atom::one(),
        Atom::Zero,
        Atom::num(Rational::from((1, 2))),
    ];
    let seed = FactorSeed::verify(
        source.clone(),
        point.clone(),
        vec![Atom::num(-1), Atom::one()],
        vec![Atom::one(), Atom::one()],
        &mut b,
    )
    .unwrap();
    let branch = FactorBranch::prepare(seed, &fresh("oldbase", 3), &mut b).unwrap();
    let field = branch.seed().context().field();
    assert_eq!(
        branch.jet(false, 0, &[0], &mut b).unwrap().at_fiber(),
        &field.constant(Rational::from((-1, 4)))
    );
    assert!(branch.frame().local().ring().is_parameter(2));
    assert_eq!(branch.frame().local().guards().len(), 2);
    let seed = FactorSeed::verify(
        source.clone(),
        point.clone(),
        vec![Atom::one(), Atom::one()],
        vec![Atom::num(-1), Atom::one()],
        &mut b,
    )
    .unwrap();
    let swapped = FactorBranch::prepare(seed, &fresh("oldbase_swapped", 3), &mut b).unwrap();
    assert_eq!(
        swapped.jet(false, 0, &[0], &mut b).unwrap().at_fiber(),
        &field.constant(Rational::from((1, 4)))
    );
    let mut wrong = point;
    wrong[3] = Atom::one();
    assert!(
        FactorSeed::verify(
            source,
            wrong,
            vec![Atom::num(-1), Atom::one()],
            vec![Atom::one(), Atom::one()],
            &mut b
        )
        .is_err()
    );
}
#[test]
fn local_germ_resource_limits_and_jet_owner_lifetime_are_explicit() {
    let mut b = budget();
    let x = symbol!("aj_owner::lx");
    let z = symbol!("aj_owner::lz");
    let frame = affine(vec![x], vec![], vec![0], &mut b);
    let r = frame.local().ring();
    let mut degree_limit = budget();
    degree_limit.limits.max_mark = 1;
    assert!(matches!(
        MonicPolynomial::new(
            frame.clone(),
            z,
            vec![-r.one(), r.one().zero(), r.one()],
            &mut degree_limit
        ),
        Err(crate::threshold::resolution::Error::ResourceIncomplete(_))
    ));
    let source = MonicPolynomial::new(
        frame.clone(),
        z,
        vec![-r.one(), r.one().zero(), r.one()],
        &mut b,
    )
    .unwrap();
    let too_large = Atom::num(2).pow(Atom::num(Rational::from((1, 1000))));
    assert!(matches!(
        FactorSeed::verify(
            source.clone(),
            vec![too_large],
            vec![Atom::num(-1), Atom::one()],
            vec![Atom::one(), Atom::one()],
            &mut b
        ),
        Err(crate::threshold::resolution::Error::ResourceIncomplete(_))
    ));
    let seed = FactorSeed::verify(
        source,
        vec![Atom::Zero],
        vec![Atom::num(-1), Atom::one()],
        vec![Atom::one(), Atom::one()],
        &mut b,
    )
    .unwrap();
    let mut limited = budget();
    limited.limits.max_terms = 1;
    assert!(matches!(
        FactorBranch::prepare(seed.clone(), &fresh("preflight", 3), &mut limited),
        Err(crate::threshold::resolution::Error::ResourceIncomplete(_))
    ));
    let branch = FactorBranch::prepare(seed, &fresh("lifetime", 3), &mut b).unwrap();
    let weak = Arc::downgrade(&branch);
    let jet = branch.jet(false, 0, &[0], &mut b).unwrap();
    drop(branch);
    assert!(weak.upgrade().is_some());
    assert!(Arc::ptr_eq(jet.owner(), &weak.upgrade().unwrap()));
    drop(jet);
    assert!(weak.upgrade().is_none());
}
