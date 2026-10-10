use super::*;
use super::{Budget, Guard, Ideal, Limits, Poly, Ring};
use std::sync::Arc;
use symbolica::{domains::rational::Rational, parse, symbol};

fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("etale_draft::p"),
                symbol!("etale_draft::x"),
                symbol!("etale_draft::y"),
                symbol!("etale_draft::qx"),
                symbol!("etale_draft::qy"),
                symbol!("etale_draft::w"),
                symbol!("etale_draft::t"),
                symbol!("etale_draft::r"),
                symbol!("etale_draft::s"),
                symbol!("etale_draft::q"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn budget() -> Budget {
    Budget::new(Limits::default())
}
fn circle(r: &Ring) -> Poly {
    r.atom(&parse!("etale_draft::x^2+etale_draft::y^2-etale_draft::p"))
        .unwrap()
}
fn source(
    r: &Arc<Ring>,
    gens: Vec<Poly>,
    axes: Vec<usize>,
    guards: Vec<Guard>,
    b: &mut Budget,
) -> Arc<LocalizedAlgebra> {
    let ideal = Ideal::new(r.clone(), gens, b).unwrap();
    LocalizedAlgebra::new(ideal, axes, guards, b).unwrap()
}

#[test]
fn finite_cover_is_exact_and_keeps_real_fibers_separate() {
    let r = ring();
    let mut b = budget();
    let parent = source(
        &r,
        vec![],
        vec![1, 2],
        vec![Guard {
            factor: r.coordinate(0).unwrap(),
            inverse_axis: 5,
        }],
        &mut b,
    );
    let support = Ideal::new(r.clone(), vec![circle(&r)], &mut b).unwrap();
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let checked = OpenCoverCertificate {
        algebra: parent.clone(),
        support: support.clone(),
        opens: vec![x.clone(), y.clone()],
    }
    .verify(&mut b)
    .unwrap();
    assert!(!checked.algebraic_locus_empty());
    assert_eq!(checked.opens().len(), 2);
    for only in [x.clone(), y] {
        assert!(
            OpenCoverCertificate {
                algebra: parent.clone(),
                support: support.clone(),
                opens: vec![only]
            }
            .verify(&mut b)
            .is_err()
        );
    }
    let at_collision = source(
        &r,
        vec![r.coordinate(0).unwrap()],
        vec![1, 2],
        vec![],
        &mut b,
    );
    assert!(
        OpenCoverCertificate {
            algebra: at_collision,
            support,
            opens: vec![x, r.coordinate(2).unwrap()]
        }
        .verify(&mut b)
        .is_err()
    );
    let empty = Ideal::new(r.clone(), vec![r.one()], &mut b).unwrap();
    assert!(
        OpenCoverCertificate {
            algebra: parent,
            support: empty,
            opens: vec![]
        }
        .verify(&mut b)
        .unwrap()
        .algebraic_locus_empty()
    );
}

#[test]
fn circle_relative_frame_preserves_units_parameters_and_second_derivative() {
    let r = ring();
    let mut b = budget();
    let s = source(
        &r,
        vec![circle(&r)],
        vec![1, 2],
        vec![Guard {
            factor: r.coordinate(0).unwrap(),
            inverse_axis: 5,
        }],
        &mut b,
    );
    let frame = EtaleCertificate {
        source: s,
        equations: vec![0],
        dependent_axes: vec![1],
        free_axes: vec![2],
        determinant_inverse_axis: Some(3),
    }
    .verify(&mut b)
    .unwrap();
    assert_eq!(
        frame.determinant(),
        &r.atom(&parse!("2*etale_draft::x")).unwrap()
    );
    assert!(frame.coefficients(0).unwrap()[0].is_zero());
    assert!(frame.coefficients(0).unwrap()[5].is_zero());
    let first = frame
        .derivative(0, &r.coordinate(1).unwrap(), &mut b)
        .unwrap();
    let second = frame.derivative(0, &first, &mut b).unwrap();
    let expected = r
        .atom(&parse!("-8*etale_draft::p*etale_draft::qx^3"))
        .unwrap();
    assert!(frame.local().zero(&(second - expected), &mut b).unwrap());
    let mut point = vec![Rational::zero(); r.len()];
    point[0] = 1.into();
    point[1] = 1.into();
    point[3] = (1, 2).into();
    point[5] = 1.into();
    assert!(frame.local().check_real_point(&point).unwrap());
    point[0] = (-1).into();
    point[5] = (-1).into();
    assert!(!frame.local().check_real_point(&point).unwrap());
}

#[test]
fn relation_tower_mixed_lifts_have_native_coefficients_and_commute() {
    let r = ring();
    let mut b = budget();
    let equations = vec![
        r.atom(&parse!(
            "etale_draft::r^2-etale_draft::p-etale_draft::t^2-etale_draft::y^2"
        ))
        .unwrap(),
        r.atom(&parse!("etale_draft::s^2-etale_draft::r-1"))
            .unwrap(),
    ];
    let s = source(&r, equations, vec![2, 6, 7, 8], vec![], &mut b);
    let f = EtaleCertificate {
        source: s,
        equations: vec![0, 1],
        dependent_axes: vec![7, 8],
        free_axes: vec![6, 2],
        determinant_inverse_axis: Some(9),
    }
    .verify(&mut b)
    .unwrap();
    assert_eq!(
        f.determinant(),
        &r.atom(&parse!("4*etale_draft::r*etale_draft::s")).unwrap()
    );
    let dr = f.derivative(0, &r.coordinate(7).unwrap(), &mut b).unwrap();
    let expected = r
        .atom(&parse!("4*etale_draft::s*etale_draft::t*etale_draft::q"))
        .unwrap();
    assert!(f.local().zero(&(dr.clone() - expected), &mut b).unwrap());
    let mixed = f.derivative(1, &dr, &mut b).unwrap();
    let expected = r
        .atom(&parse!(
            "-64*etale_draft::t*etale_draft::y*etale_draft::s^3*etale_draft::q^3"
        ))
        .unwrap();
    assert!(f.local().zero(&(mixed - expected), &mut b).unwrap());
}

#[test]
fn redundant_relations_are_checked_and_uncovered_constraints_rejected() {
    let r = ring();
    let mut b = budget();
    let g = circle(&r);
    let redundant = &r.coordinate(1).unwrap() * &g;
    let s = source(&r, vec![g.clone(), redundant], vec![1, 2], vec![], &mut b);
    assert!(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3)
        }
        .verify(&mut b)
        .is_ok()
    );
    let s = source(
        &r,
        vec![g, r.coordinate(2).unwrap()],
        vec![1, 2],
        vec![],
        &mut b,
    );
    assert!(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3)
        }
        .verify(&mut b)
        .is_err()
    );
}

#[test]
fn empty_localization_singular_minor_invalid_roles_and_limits_refuse() {
    let r = ring();
    let mut b = budget();
    let square = r.atom(&parse!("etale_draft::x^2")).unwrap();
    let s = source(&r, vec![square], vec![1, 2], vec![], &mut b);
    assert!(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3)
        }
        .verify(&mut b)
        .is_err()
    );
    let equation = r.atom(&parse!("etale_draft::y-etale_draft::p")).unwrap();
    let s = source(&r, vec![equation], vec![1, 2], vec![], &mut b);
    assert!(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3)
        }
        .verify(&mut b)
        .is_err()
    );
    let s = source(&r, vec![circle(&r)], vec![1, 2], vec![], &mut b);
    for (dependent, free, q) in [
        (vec![0], vec![1, 2], 3),
        (vec![1], vec![1], 3),
        (vec![1], vec![2], 0),
    ] {
        assert!(
            EtaleCertificate {
                source: s.clone(),
                equations: vec![0],
                dependent_axes: dependent,
                free_axes: free,
                determinant_inverse_axis: Some(q)
            }
            .verify(&mut b)
            .is_err()
        );
    }
    let mut tight = Budget::new(Limits {
        max_terms: 1,
        ..Limits::default()
    });
    assert!(matches!(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3)
        }
        .verify(&mut tight),
        Err(super::Error::ResourceIncomplete(_))
    ));
}

fn producer_frame(r: &Arc<Ring>, b: &mut Budget) -> Arc<EtaleFrame> {
    let s = source(r, vec![circle(r)], vec![1, 2], vec![], b);
    Arc::new(
        EtaleCertificate {
            source: s,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![2],
            determinant_inverse_axis: Some(3),
        }
        .verify(b)
        .unwrap(),
    )
}

#[test]
fn constructive_order_and_contact_open_producer_keeps_inclusive_towers() {
    let r = ring();
    let mut b = budget();
    let frame = producer_frame(&r, &mut b);
    let original = Arc::new(
        Ideal::new(
            r.clone(),
            vec![r.atom(&parse!("etale_draft::y^2")).unwrap()],
            &mut b,
        )
        .unwrap(),
    );
    let OrderProduction::ContactCover(produced) =
        produce_ordinary_contact_cover(frame, original, &mut b).unwrap()
    else {
        panic!("expected order-two contact cover");
    };
    assert_eq!(produced.algebraic_maximum_order(), 2);
    assert_eq!(
        produced
            .progress()
            .stages
            .iter()
            .map(|s| s.is_unit)
            .collect::<Vec<_>>(),
        vec![Some(false), Some(false), Some(true)]
    );
    for pair in produced.progress().stages.windows(2) {
        for f in pair[0].ideal.generators() {
            assert!(pair[1].ideal.generators().contains(f));
        }
    }
    assert!(
        produced
            .candidates()
            .iter()
            .any(|c| c.differential == r.atom(&parse!("2")).unwrap())
    );
    assert!(!produced.cover().algebraic_locus_empty());
    assert!(produced.progress().operations > 0);
}

#[test]
fn parameter_only_and_resource_frontiers_are_preserved_without_false_maxima() {
    let r = ring();
    let mut b = budget();
    let frame = producer_frame(&r, &mut b);
    let parameter =
        Arc::new(Ideal::new(r.clone(), vec![r.coordinate(0).unwrap()], &mut b).unwrap());
    let mut limited = Budget::new(Limits {
        max_mark: 2,
        ..Limits::default()
    });
    let OrderProduction::Incomplete {
        progress, source, ..
    } = produce_ordinary_contact_cover(frame.clone(), parameter.clone(), &mut limited).unwrap()
    else {
        panic!("parameter locus cannot be differentiated away");
    };
    assert_eq!(source, parameter);
    assert_eq!(progress.stages.len(), 3);
    assert!(
        progress
            .stages
            .iter()
            .all(|s| s.is_unit == Some(false) && s.ideal == parameter)
    );
    let mut exhausted = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    let OrderProduction::Incomplete { progress, .. } =
        produce_ordinary_contact_cover(frame.clone(), parameter, &mut exhausted).unwrap()
    else {
        panic!("expected explicit resource frontier");
    };
    assert_eq!(progress.stages.len(), 1);
    assert_eq!(progress.stages[0].is_unit, None);
    assert_eq!(progress.operations, 0);
    let unit = Arc::new(Ideal::new(r.clone(), vec![r.one()], &mut b).unwrap());
    assert!(matches!(
        produce_ordinary_contact_cover(frame, unit, &mut b).unwrap(),
        OrderProduction::UnitIdeal { .. }
    ));
}

fn affine_frame(r: &Arc<Ring>, axes: Vec<usize>, b: &mut Budget) -> Arc<EtaleFrame> {
    let s = source(r, vec![], axes.clone(), vec![], b);
    Arc::new(
        EtaleCertificate {
            source: s,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: axes,
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}

#[test]
fn ordinary_plane_cusp_has_a_real_affine_base_and_generated_contact() {
    let r = ring();
    let mut b = budget();
    let frame = affine_frame(&r, vec![1, 2], &mut b);
    assert_eq!(frame.determinant(), &r.one());
    assert!(frame.local().guards().is_empty());
    assert!(Arc::ptr_eq(frame.source(), frame.local()));
    let ideal = Arc::new(
        Ideal::new(
            r.clone(),
            vec![
                r.atom(&parse!("etale_draft::x^2+etale_draft::y^3"))
                    .unwrap(),
            ],
            &mut b,
        )
        .unwrap(),
    );
    let OrderProduction::ContactCover(c) =
        produce_ordinary_contact_cover(frame, ideal, &mut b).unwrap()
    else {
        panic!("cusp has maximum order two");
    };
    assert_eq!(c.algebraic_maximum_order(), 2);
    let two_x = r.atom(&parse!("2*etale_draft::x")).unwrap();
    assert!(
        c.candidates()
            .iter()
            .any(|h| h.equation == two_x && h.differential == r.atom(&parse!("2")).unwrap())
    );
    let mut point = vec![Rational::zero(); r.len()];
    point[0] = 1.into();
    assert!(
        c.cover()
            .support()
            .generators()
            .iter()
            .all(|f| f.replace_all(&point).is_zero())
    );
}

#[test]
fn affine_zero_unit_and_dimension_zero_states_are_explicit() {
    let r = ring();
    let mut b = budget();
    let frame = affine_frame(&r, vec![1, 2], &mut b);
    let zero = Arc::new(Ideal::new(r.clone(), vec![], &mut b).unwrap());
    let unit = Arc::new(Ideal::new(r.clone(), vec![r.one()], &mut b).unwrap());
    assert!(matches!(
        produce_ordinary_contact_cover(frame.clone(), zero.clone(), &mut b).unwrap(),
        OrderProduction::ZeroIdeal { .. }
    ));
    assert!(matches!(
        produce_ordinary_contact_cover(frame, unit.clone(), &mut b).unwrap(),
        OrderProduction::UnitIdeal { .. }
    ));
    let terminal = affine_frame(&r, vec![], &mut b);
    assert!(terminal.free_axes().is_empty());
    assert!(matches!(
        produce_ordinary_contact_cover(terminal.clone(), zero, &mut b).unwrap(),
        OrderProduction::ZeroIdeal { .. }
    ));
    assert!(matches!(
        produce_ordinary_contact_cover(terminal.clone(), unit, &mut b).unwrap(),
        OrderProduction::UnitIdeal { .. }
    ));
    let parameter =
        Arc::new(Ideal::new(r.clone(), vec![r.coordinate(0).unwrap()], &mut b).unwrap());
    assert!(matches!(
        produce_ordinary_contact_cover(terminal, parameter, &mut b).unwrap(),
        OrderProduction::TerminalParameterLocus { .. }
    ));
}
