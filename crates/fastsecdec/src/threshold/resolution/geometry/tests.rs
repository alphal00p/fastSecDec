use super::super::*;
use super::*;
use std::sync::Arc;
use symbolica::{atom::Symbol, parse, symbol};
fn budget() -> Budget {
    Budget::new(Limits::default())
}
fn symbols() -> [Symbol; 5] {
    [
        symbol!("boundary_probe::p"),
        symbol!("boundary_probe::x"),
        symbol!("boundary_probe::y"),
        symbol!("boundary_probe::q"),
        symbol!("boundary_probe::w"),
    ]
}
fn ring() -> Arc<Ring> {
    Arc::new(Ring::new(symbols()[..4].to_vec(), vec![0]).unwrap())
}
fn p(r: &Ring, a: symbolica::atom::Atom) -> Poly {
    r.atom(&a).unwrap()
}
fn affine(r: Arc<Ring>, axes: Vec<usize>, guards: Vec<Guard>, b: &mut Budget) -> Arc<EtaleFrame> {
    let local =
        LocalizedAlgebra::new(Ideal::new(r, vec![], b).unwrap(), axes.clone(), guards, b).unwrap();
    Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: axes,
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn normal(
    normalizer: &Arc<QuotientNormalizer>,
    source: Arc<MarkedIdeal>,
    b: &mut Budget,
) -> Box<VerifiedQuotientNormalization> {
    match normalizer.normalize(source, b).unwrap() {
        NormalizationOutcome::Complete(c) => c,
        other => panic!("normalization {other:?}"),
    }
}
fn marked(r: Arc<Ring>, generators: Vec<Poly>, mark: usize, b: &mut Budget) -> Arc<MarkedIdeal> {
    Arc::new(MarkedIdeal::new(Ideal::new(r, generators, b).unwrap(), mark, b).unwrap())
}
fn snc(frame: Arc<EtaleFrame>, equations: Vec<Poly>, b: &mut Budget) -> Arc<VerifiedRelativeSnc> {
    let input = equations
        .into_iter()
        .enumerate()
        .map(|(i, equation)| InitialDivisor {
            id: BoundaryId(i as u64),
            equation,
        })
        .collect();
    match verify_initial_relative_snc(frame, input, b).unwrap() {
        SncProduction::Verified(v) => v,
        other => panic!("SNC {other:?}"),
    }
}
#[test]
fn normalization_keeps_guards_parameters_marks_and_resource_evidence() {
    let r = ring();
    let mut b = budget();
    let g = p(&r, parse!("1+boundary_probe::y^2"));
    let frame = affine(
        r.clone(),
        vec![1, 2],
        vec![Guard {
            factor: g,
            inverse_axis: 3,
        }],
        &mut b,
    );
    let n = QuotientNormalizer::prepare(frame.local().clone(), &mut b).unwrap();
    let source = marked(
        r.clone(),
        vec![
            p(
                &r,
                parse!("boundary_probe::q*boundary_probe::p*boundary_probe::x"),
            ),
            p(
                &r,
                parse!("boundary_probe::q^2*boundary_probe::p*boundary_probe::x^2"),
            ),
        ],
        3,
        &mut b,
    );
    let out = normal(&n, source.clone(), &mut b);
    assert_eq!(out.normalized().mark(), 3);
    assert_eq!(out.progress().original_generators_checked, 2);
    let expected = marked(
        r.clone(),
        vec![p(&r, parse!("boundary_probe::p*boundary_probe::x"))],
        3,
        &mut b,
    );
    assert!(n.verify_candidate(&source, &expected, &mut b).unwrap());
    let wrong = marked(r.clone(), vec![r.coordinate(1).unwrap()], 3, &mut b);
    assert!(!n.verify_candidate(&source, &wrong, &mut b).unwrap());
    let wrong_mark = marked(r.clone(), expected.ideal().generators().to_vec(), 2, &mut b);
    assert!(n.verify_candidate(&source, &wrong_mark, &mut b).is_err());
    let mut fresh = budget();
    let complete = normal(&n, source.clone(), &mut fresh);
    let mut limited = Budget::new(Limits {
        max_operations: complete.progress().operations - 1,
        ..Limits::default()
    });
    let NormalizationOutcome::Incomplete {
        source: retained,
        progress,
        ..
    } = n.normalize(source.clone(), &mut limited).unwrap()
    else {
        panic!("budget")
    };
    assert!(Arc::ptr_eq(&source, &retained));
    assert!(progress.candidate.is_some());
    assert!(progress.candidate_basis.is_some());
    let zero = marked(r.clone(), vec![], 7, &mut b);
    assert!(
        normal(&n, zero, &mut b)
            .normalized()
            .ideal()
            .generators()
            .is_empty()
    );
    let unit = marked(r.clone(), vec![r.one()], 7, &mut b);
    assert_eq!(
        normal(&n, unit, &mut b).normalized().ideal().generators(),
        &[r.one()]
    );
}
#[test]
fn normalization_consumes_actual_differential_coefficient() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let input = Arc::new(
        Ideal::new(
            r.clone(),
            vec![p(&r, parse!("boundary_probe::x^2+boundary_probe::y^3"))],
            &mut b,
        )
        .unwrap(),
    );
    let OrderProduction::ContactCover(c) =
        produce_ordinary_contact_cover(frame, input, &mut b).unwrap()
    else {
        panic!("cover")
    };
    let index = c
        .candidates()
        .iter()
        .position(|v| v.equation == p(&r, parse!("2*boundary_probe::x")))
        .unwrap();
    let ContactProduction::Constructed(q) = construct_contact_quotient(
        Arc::new(c),
        index,
        [symbol!("boundary_probe::z"), symbol!("boundary_probe::v")],
        &mut b,
    )
    .unwrap() else {
        panic!("contact")
    };
    let n = QuotientNormalizer::prepare(q.contact().local().clone(), &mut b).unwrap();
    let result = normal(&n, Arc::new(q.differential_coefficient().clone()), &mut b);
    assert_eq!(result.normalized().mark(), 2);
    assert_eq!(
        result.normalized().ideal().generators(),
        &[p(q.contact().local().ring(), parse!("boundary_probe::y^3"))]
    );
    let OrderProduction::ContactCover(next) = produce_ordinary_contact_cover(
        q.contact().clone(),
        Arc::new(result.normalized().ideal().clone()),
        &mut b,
    )
    .unwrap() else {
        panic!("recursive cover")
    };
    let index = next
        .candidates()
        .iter()
        .position(|v| v.equation == p(q.contact().local().ring(), parse!("6*boundary_probe::y")))
        .unwrap();
    assert!(matches!(
        construct_contact_quotient(
            Arc::new(next),
            index,
            [symbol!("boundary_probe::zz"), symbol!("boundary_probe::vv")],
            &mut b
        )
        .unwrap(),
        ContactProduction::Constructed(_)
    ));
}
#[test]
fn all_strata_snc_and_monomial_residual_unit_combination() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let ledger = snc(
        frame,
        vec![r.coordinate(1).unwrap(), r.coordinate(2).unwrap()],
        &mut b,
    );
    assert_eq!(ledger.progress().intersections.len(), 3);
    assert!(
        ledger
            .progress()
            .intersections
            .iter()
            .all(|s| !s.empty && !s.minors.is_empty())
    );
    let m = p(&r, parse!("boundary_probe::x^2*boundary_probe::y^2"));
    let q1 = r.coordinate(1).unwrap();
    let q2 = &r.one() - &q1;
    let source = marked(r.clone(), vec![&m * &q1, &m * &q2], 3, &mut b);
    let witness = MonomialWitness {
        source,
        powers: vec![2, 2],
        quotients: vec![q1, q2],
    };
    let MonomialProduction::Center(center) =
        produce_monomial_center(ledger.clone(), witness.clone(), &mut b).unwrap()
    else {
        panic!("center")
    };
    assert_eq!(center.indices(), &[0, 1]);
    assert_eq!(center.exceptional_power(), 1);
    assert!(center.progress().residual_unit);
    for q in center.residual().generators() {
        let ideal = Ideal::new(r.clone(), vec![q.clone()], &mut b).unwrap();
        assert!(!ideal.contains(&r.one(), &[], &mut b).unwrap());
    }
    let mut wrong = witness.clone();
    wrong.quotients[0] = r.one();
    assert!(produce_monomial_center(ledger.clone(), wrong, &mut b).is_err());
    let mut wrong = witness;
    wrong.powers[0] = 1;
    assert!(produce_monomial_center(ledger, wrong, &mut b).is_err());
}
#[test]
fn empty_intersections_nonlinear_snc_and_parameter_collision() {
    let r = ring();
    let mut b = budget();
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let ledger = snc(
        affine(
            r.clone(),
            vec![1, 2],
            vec![Guard {
                factor: x.clone(),
                inverse_axis: 3,
            }],
            &mut b,
        ),
        vec![x.clone(), y.clone()],
        &mut b,
    );
    assert!(ledger.intersection(&[0, 1]).unwrap().empty);
    let m = &x.pow(2) * &y.pow(2);
    let source = marked(r.clone(), vec![m], 3, &mut b);
    assert!(matches!(
        produce_monomial_center(
            ledger,
            MonomialWitness {
                source,
                powers: vec![2, 2],
                quotients: vec![r.one()]
            },
            &mut b
        )
        .unwrap(),
        MonomialProduction::EmptyCosupport { .. }
    ));
    let nonlinear = snc(
        affine(r.clone(), vec![1, 2], vec![], &mut b),
        vec![
            x.clone(),
            p(
                &r,
                parse!("boundary_probe::y-boundary_probe::p*boundary_probe::x^2"),
            ),
        ],
        &mut b,
    );
    assert_eq!(nonlinear.intersection(&[0, 1]).unwrap().minors, &[r.one()]);
    let divisors = vec![
        InitialDivisor {
            id: BoundaryId(0),
            equation: x.clone(),
        },
        InitialDivisor {
            id: BoundaryId(1),
            equation: p(
                &r,
                parse!("boundary_probe::x-boundary_probe::p*boundary_probe::y"),
            ),
        },
    ];
    let result = verify_initial_relative_snc(
        affine(r.clone(), vec![1, 2], vec![], &mut b),
        divisors.clone(),
        &mut b,
    )
    .unwrap();
    let SncProduction::Unresolved { obstruction, .. } = result else {
        panic!("collision")
    };
    assert!(!obstruction.contains(&r.one(), &[], &mut b).unwrap());
    assert!(matches!(
        verify_initial_relative_snc(
            affine(
                r.clone(),
                vec![1, 2],
                vec![Guard {
                    factor: r.coordinate(0).unwrap(),
                    inverse_axis: 3
                }],
                &mut b
            ),
            divisors,
            &mut b
        )
        .unwrap(),
        SncProduction::Verified(_)
    ));
}
#[test]
fn nonreduced_excess_intersections_ids_and_budget() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    assert!(matches!(
        verify_initial_relative_snc(
            frame.clone(),
            vec![InitialDivisor {
                id: BoundaryId(0),
                equation: r.coordinate(1).unwrap().pow(2)
            }],
            &mut b
        )
        .unwrap(),
        SncProduction::Unresolved { .. }
    ));
    let mut divisors = vec![
        InitialDivisor {
            id: BoundaryId(0),
            equation: r.coordinate(1).unwrap(),
        },
        InitialDivisor {
            id: BoundaryId(1),
            equation: r.coordinate(2).unwrap(),
        },
        InitialDivisor {
            id: BoundaryId(2),
            equation: p(&r, parse!("boundary_probe::x+boundary_probe::y")),
        },
    ];
    let SncProduction::Unresolved { indices, .. } =
        verify_initial_relative_snc(frame.clone(), divisors.clone(), &mut b).unwrap()
    else {
        panic!("triple")
    };
    assert_eq!(indices, &[0, 1, 2]);
    divisors[1].id = BoundaryId(0);
    assert!(verify_initial_relative_snc(frame.clone(), divisors, &mut b).is_err());
    let mut low = Budget::new(Limits {
        max_generators: 1,
        ..Limits::default()
    });
    let ds = vec![
        InitialDivisor {
            id: BoundaryId(0),
            equation: r.coordinate(1).unwrap(),
        },
        InitialDivisor {
            id: BoundaryId(1),
            equation: r.coordinate(2).unwrap(),
        },
    ];
    assert!(matches!(
        verify_initial_relative_snc(frame, ds, &mut low).unwrap(),
        SncProduction::Incomplete { .. }
    ));
}
#[test]
fn ordered_minimal_center_and_cartier_case() {
    let r = Arc::new(Ring::new(symbols().to_vec(), vec![0]).unwrap());
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2, 4], vec![], &mut b);
    let axes = vec![
        r.coordinate(1).unwrap(),
        r.coordinate(2).unwrap(),
        r.coordinate(4).unwrap(),
    ];
    let ledger = snc(frame, axes.clone(), &mut b);
    let monomial = &(&axes[0] * &axes[1]) * &axes[2];
    let source = marked(r.clone(), vec![monomial], 2, &mut b);
    let MonomialProduction::Center(center) = produce_monomial_center(
        ledger.clone(),
        MonomialWitness {
            source,
            powers: vec![1, 1, 1],
            quotients: vec![r.one()],
        },
        &mut b,
    )
    .unwrap() else {
        panic!("center")
    };
    assert_eq!(center.indices(), &[0, 1]);
    assert_eq!(center.progress().candidates.len(), 3);
    // BM §5 II.A uses incidence lexicographic order, not minimum cardinality.
    let source = marked(
        r.clone(),
        vec![&(&axes[0] * &axes[1]) * &axes[2].pow(2)],
        2,
        &mut b,
    );
    let MonomialProduction::Center(center) = produce_monomial_center(
        ledger.clone(),
        MonomialWitness {
            source,
            powers: vec![1, 1, 2],
            quotients: vec![r.one()],
        },
        &mut b,
    )
    .unwrap() else {
        panic!("incidence order")
    };
    assert_eq!(center.indices(), &[0, 1]);
    assert!(
        center
            .progress()
            .candidates
            .iter()
            .any(|c| c.indices == vec![2])
    );
    let source = marked(r.clone(), vec![axes[0].pow(3)], 3, &mut b);
    let MonomialProduction::Center(center) = produce_monomial_center(
        ledger,
        MonomialWitness {
            source,
            powers: vec![3, 0, 0],
            quotients: vec![r.one()],
        },
        &mut b,
    )
    .unwrap() else {
        panic!("Cartier")
    };
    assert_eq!(center.indices(), &[0]);
    assert_eq!(center.exceptional_power(), 0);
}

#[test]
fn finite_minor_cover_and_native_etale_quotient() {
    let r = ring();
    let mut b = budget();
    let circle = p(&r, parse!("boundary_probe::x^2+boundary_probe::y^2-1"));
    let ledger = snc(
        affine(r.clone(), vec![1, 2], vec![], &mut b),
        vec![circle.clone()],
        &mut b,
    );
    let stratum = ledger.intersection(&[0]).unwrap();
    assert_eq!(stratum.minors.len(), 2);
    for minor in &stratum.minors {
        let only = Ideal::new(r.clone(), vec![minor.clone()], &mut b).unwrap();
        let obstruction = stratum.locus.sum(&only, &mut b).unwrap();
        assert!(!obstruction.contains(&r.one(), &[], &mut b).unwrap());
    }
    // The same circle as a source quotient, with y-dependent and x-free.
    // D_x y=-x/y; parameter p stays inert. The inverse of 2y is q.
    let source = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![circle.clone()], &mut b).unwrap(),
        vec![1, 2],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source,
            equations: vec![0],
            dependent_axes: vec![2],
            free_axes: vec![1],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let ledger = snc(frame.clone(), vec![r.coordinate(1).unwrap()], &mut b);
    assert_eq!(ledger.intersection(&[0]).unwrap().minors, vec![r.one()]);
    assert!(
        frame
            .derivative(0, &r.coordinate(0).unwrap(), &mut b)
            .unwrap()
            .is_zero()
    );
    let n = QuotientNormalizer::prepare(frame.local().clone(), &mut b).unwrap();
    let input = marked(r.clone(), vec![circle], 2, &mut b);
    assert!(
        normal(&n, input, &mut b)
            .normalized()
            .ideal()
            .generators()
            .is_empty()
    );
}
