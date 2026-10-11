//! Diagnostic native probes only; no production geometry authority issued here.
use super::*;
use symbolica::{domains::rational::Rational, poly::PolyVariable};
fn symbols(r: &Ring) -> Vec<symbolica::atom::Symbol> {
    r.one()
        .variables()
        .iter()
        .map(|v| match v {
            PolyVariable::Symbol(s) => *s,
            _ => panic!(),
        })
        .collect()
}
#[test]
fn nonmonomial_map_probe_composes_actual_contacts_without_foreign_slots() {
    let mut b = budget();
    let centers = super::nonmonomial_probe::nested_source_centers(1, &mut b);
    let mut checked = 0;
    for center in centers {
        let upper = center.coefficient().contact();
        let Some(lift) = center.child().lift_receipt() else {
            continue;
        };
        let lower = lift.level().contact();
        assert!(Arc::ptr_eq(lower.source().frame(), upper.contact()));
        let parent =
            super::super::embedding::SupportEmbedding::contact(upper.clone(), &mut b).unwrap();
        let nested = super::super::embedding::SupportEmbedding::nested(
            parent.clone(),
            lower.clone(),
            &mut b,
        )
        .unwrap();
        assert_eq!(nested.codimension(), 2);
        assert!(Arc::ptr_eq(nested.original_contact(), upper));
        assert!(Arc::ptr_eq(nested.nested_contact().unwrap().0, &parent));
        let original = upper.source().frame().local().ring();
        let suffix = symbols(lower.contact().local().ring())[original.len()..].to_vec();
        let extension = RingExtension::with_symbols(original.clone(), &suffix, &mut b).unwrap();
        assert_eq!(extension.target(), lower.contact().local().ring());
        let first =
            super::super::super::elimination::remap(&upper.clearing().numerator, original, &mut b)
                .unwrap();
        let lower_equation = super::super::super::elimination::remap(
            &lower.clearing().numerator,
            upper.contact().local().ring(),
            &mut b,
        )
        .unwrap();
        let c = clear_units(upper.contact().local(), &lower_equation, &mut b).unwrap();
        assert!(
            !upper
                .contact()
                .local()
                .axes()
                .contains(&upper.contact_axis())
        );
        assert_eq!(c.numerator.degree(upper.contact_axis()), 0);
        assert_eq!(c.denominator.degree(upper.contact_axis()), 0);
        let n = c
            .numerator
            .replace(upper.contact_axis(), &Rational::from(0));
        let d = c
            .denominator
            .replace(upper.contact_axis(), &Rational::from(0));
        let n = super::super::super::elimination::remap(&n, original, &mut b).unwrap();
        let d = super::super::super::elimination::remap(&d, original, &mut b).unwrap();
        assert!(
            upper
                .source()
                .frame()
                .local()
                .ideal()
                .sum(
                    &Ideal::new(original.clone(), vec![d], &mut b).unwrap(),
                    &mut b
                )
                .unwrap()
                .contains(
                    &original.one(),
                    upper.source().frame().local().unit_relations(),
                    &mut b
                )
                .unwrap()
        );
        for f in [&first, &n] {
            assert!(
                lower
                    .contact()
                    .local()
                    .zero(&extension.pull(f, &mut b).unwrap(), &mut b)
                    .unwrap()
            );
        }
        let mut expected = extension
            .ideal(upper.source().frame().local().ideal(), &mut b)
            .unwrap()
            .generators()
            .to_vec();
        expected.push(extension.pull(&first, &mut b).unwrap());
        expected.push(extension.pull(&n, &mut b).unwrap());
        let expected = Ideal::new(extension.target().clone(), expected, &mut b).unwrap();
        assert!(equal(
            &expected,
            lower.contact().local().ideal(),
            lower.contact().local().unit_relations(),
            &mut b
        ));
        assert_eq!(
            upper.source().frame().free_axes().len(),
            lower.contact().free_axes().len() + 2
        );
        for i in 0..original.len() {
            assert_eq!(original.is_parameter(i), extension.target().is_parameter(i));
        }
        checked += 1;
    }
    assert!(checked > 0);
    println!("actual nested contact compositions {checked}");
}
#[test]
fn nonmonomial_map_probe_native_union_frames_inverse_roles_and_cover() {
    let mut b = budget();
    let base = Arc::new(
        Ring::new(
            vec![
                symbol!("nested_union::x"),
                symbol!("nested_union::y"),
                symbol!("nested_union::p"),
            ],
            vec![2],
        )
        .unwrap(),
    );
    let zero = Ideal::new(base.clone(), vec![], &mut b).unwrap();
    let local = LocalizedAlgebra::new(zero.clone(), vec![0, 1], vec![], &mut b).unwrap();
    let x = base.coordinate(0).unwrap();
    let y = base.coordinate(1).unwrap();
    let a = [x.clone(), &base.one() - &x];
    let c = [y.clone(), &base.one() - &y];
    let left = OpenCoverCertificate {
        algebra: local.clone(),
        support: zero.clone(),
        opens: a.to_vec(),
    }
    .verify(&mut b)
    .unwrap();
    let right = OpenCoverCertificate {
        algebra: local.clone(),
        support: zero.clone(),
        opens: c.to_vec(),
    }
    .verify(&mut b)
    .unwrap();
    let products = left
        .opens()
        .iter()
        .flat_map(|f| right.opens().iter().map(move |g| f * g))
        .collect();
    let common_cover = OpenCoverCertificate {
        algebra: local,
        support: zero,
        opens: products,
    }
    .verify(&mut b)
    .unwrap();
    assert_eq!(common_cover.opens().len(), 4);
    let le = RingExtension::new(
        base.clone(),
        [
            symbol!("nested_union::left_inverse"),
            symbol!("nested_union::left_spare"),
        ],
        &mut b,
    )
    .unwrap();
    let re = RingExtension::new(
        base.clone(),
        [
            symbol!("nested_union::right_inverse"),
            symbol!("nested_union::right_spare"),
        ],
        &mut b,
    )
    .unwrap();
    let mk = |e: &RingExtension, g: &Poly, b: &mut Budget| {
        let local = LocalizedAlgebra::new(
            Ideal::new(e.target().clone(), vec![], b).unwrap(),
            vec![0, 1],
            vec![Guard {
                factor: e.pull(g, b).unwrap(),
                inverse_axis: 3,
            }],
            b,
        )
        .unwrap();
        Arc::new(
            EtaleCertificate {
                source: local,
                equations: vec![],
                dependent_axes: vec![],
                free_axes: vec![0, 1],
                determinant_inverse_axis: None,
            }
            .verify(b)
            .unwrap(),
        )
    };
    let l = mk(&le, &x, &mut b);
    let r = mk(&re, &y, &mut b);
    let mut all = symbols(l.local().ring());
    all.extend(symbols(r.local().ring())[base.len()..].iter());
    let union = Arc::new(Ring::new(all, vec![2]).unwrap());
    let remap =
        |f: &Poly, b: &mut Budget| super::super::super::elimination::remap(f, &union, b).unwrap();
    let joint = LocalizedAlgebra::new(
        Ideal::new(union.clone(), vec![], &mut b).unwrap(),
        vec![0, 1],
        vec![
            Guard {
                factor: remap(&x, &mut b),
                inverse_axis: 3,
            },
            Guard {
                factor: remap(&y, &mut b),
                inverse_axis: 5,
            },
        ],
        &mut b,
    )
    .unwrap();
    let joint = Arc::new(
        EtaleCertificate {
            source: joint,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0, 1],
            determinant_inverse_axis: None,
        }
        .verify(&mut b)
        .unwrap(),
    );
    for old in [&l, &r] {
        for f in old.local().unit_relations() {
            assert!(joint.local().zero(&remap(f, &mut b), &mut b).unwrap());
        }
        for axis in [0, 1, 2, 3] {
            for dir in 0..2 {
                let f = old.local().ring().coordinate(axis).unwrap();
                let expected = remap(&old.derivative(dir, &f, &mut b).unwrap(), &mut b);
                let actual = joint.derivative(dir, &remap(&f, &mut b), &mut b).unwrap();
                assert!(joint.local().zero(&(&actual - &expected), &mut b).unwrap());
            }
        }
    }
    assert_eq!(
        remap(&r.local().ring().coordinate(3).unwrap(), &mut b),
        union.coordinate(5).unwrap()
    );
    assert!(
        joint
            .local()
            .zero(
                &joint
                    .derivative(0, &union.coordinate(2).unwrap(), &mut b)
                    .unwrap(),
                &mut b
            )
            .unwrap()
    );
    // An incompatible closed support/open intersection is retained as exact
    // empty evidence before trying to manufacture a localized frame.
    let closed = Ideal::new(union.clone(), vec![remap(&x, &mut b)], &mut b).unwrap();
    assert!(
        closed
            .contains(&union.one(), joint.local().unit_relations(), &mut b)
            .unwrap()
    );
    assert!(matches!(
        LocalizedAlgebra::new(closed, vec![0, 1], joint.local().guards().to_vec(), &mut b),
        Err(Error::Invalid("empty algebraic principal open"))
    ));
}
#[test]
fn nonmonomial_map_probe_nonlinear_frames_full_ideal_and_chain_rule() {
    let mut b = budget();
    let base = Arc::new(
        Ring::new(
            vec![
                symbol!("nonlinear_union::x"),
                symbol!("nonlinear_union::y"),
                symbol!("nonlinear_union::p"),
            ],
            vec![2],
        )
        .unwrap(),
    );
    let x = base.coordinate(0).unwrap();
    let y = base.coordinate(1).unwrap();
    let equation = &y * &y - x;
    let make = |ns: &str, free: usize, dependent: usize, b: &mut Budget| {
        let e = RingExtension::new(
            base.clone(),
            [symbol!(format!("{ns}::iy")), symbol!(format!("{ns}::id"))],
            b,
        )
        .unwrap();
        let local = LocalizedAlgebra::new(
            Ideal::new(e.target().clone(), vec![e.pull(&equation, b).unwrap()], b).unwrap(),
            vec![0, 1],
            vec![Guard {
                factor: e.pull(&y, b).unwrap(),
                inverse_axis: 3,
            }],
            b,
        )
        .unwrap();
        Arc::new(
            EtaleCertificate {
                source: local,
                equations: vec![0],
                dependent_axes: vec![dependent],
                free_axes: vec![free],
                determinant_inverse_axis: Some(4),
            }
            .verify(b)
            .unwrap(),
        )
    };
    let left = make("nonlinear_left", 0, 1, &mut b);
    let right = make("nonlinear_right", 1, 0, &mut b);
    let mut variables = symbols(left.local().ring());
    variables.extend(symbols(right.local().ring())[3..].iter());
    variables.push(symbol!("nonlinear_union::idet"));
    let union = Arc::new(Ring::new(variables, vec![2]).unwrap());
    let remap =
        |f: &Poly, b: &mut Budget| super::super::super::elimination::remap(f, &union, b).unwrap();
    let mut generators = Vec::new();
    let mut guards = Vec::new();
    for (old, shift) in [(&left, 0usize), (&right, 2usize)] {
        generators.extend(
            old.local()
                .ideal()
                .generators()
                .iter()
                .map(|p| remap(p, &mut b)),
        );
        guards.extend(old.local().guards().iter().map(|g| Guard {
            factor: remap(&g.factor, &mut b),
            inverse_axis: g.inverse_axis + shift,
        }));
    }
    let all = Ideal::new(union.clone(), generators, &mut b).unwrap();
    let local = LocalizedAlgebra::new(all.clone(), vec![0, 1], guards, &mut b).unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![0],
            determinant_inverse_axis: Some(7),
        }
        .verify(&mut b)
        .unwrap(),
    );
    assert!(equal(
        &all,
        frame.local().ideal(),
        frame.local().unit_relations(),
        &mut b
    ));
    for old in [&left, &right] {
        for equation in old
            .local()
            .ideal()
            .generators()
            .iter()
            .chain(old.local().unit_relations())
        {
            assert!(
                frame
                    .local()
                    .zero(&remap(equation, &mut b), &mut b)
                    .unwrap()
            );
        }
    }
    let uy = union.coordinate(1).unwrap();
    let weight = &uy + &uy;
    for axis in [0, 1, 2, 3, 4] {
        let f = right.local().ring().coordinate(axis).unwrap();
        let dy = remap(&right.derivative(0, &f, &mut b).unwrap(), &mut b);
        let dx = frame.derivative(0, &remap(&f, &mut b), &mut b).unwrap();
        assert!(
            frame
                .local()
                .zero(&(&dy - &b.mul(&weight, &dx).unwrap()), &mut b)
                .unwrap()
        );
        let f = left.local().ring().coordinate(axis).unwrap();
        let expected = remap(&left.derivative(0, &f, &mut b).unwrap(), &mut b);
        let actual = frame.derivative(0, &remap(&f, &mut b), &mut b).unwrap();
        assert!(frame.local().zero(&(&actual - &expected), &mut b).unwrap());
    }
    assert!(
        frame
            .local()
            .zero(
                &frame
                    .derivative(0, &union.coordinate(2).unwrap(), &mut b)
                    .unwrap(),
                &mut b
            )
            .unwrap()
    );
}
#[test]
fn nonmonomial_map_probe_actual_nested_all_pivot_common_refinements() {
    let mut b = budget();
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("nested_first::x"),
                symbol!("nested_first::y"),
                symbol!("nested_first::z"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut b).unwrap(),
        vec![0, 1, 2],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0, 1, 2],
            determinant_inverse_axis: None,
        }
        .verify(&mut b)
        .unwrap(),
    );
    let x = r.coordinate(0).unwrap();
    let y = r.coordinate(1).unwrap();
    let z = r.coordinate(2).unwrap();
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r, vec![&z * &z - x.pow(3), y.pow(3)], &mut b).unwrap(),
            1,
            &b,
        )
        .unwrap(),
    );
    let SncProduction::Verified(ledger) =
        verify_initial_relative_snc(frame.clone(), vec![], &mut b).unwrap()
    else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let mut driver =
        BoundaryFreeFirstCenter::new(frame, source, "nested_first_center", false).unwrap();
    for _ in 0..100 {
        if matches!(driver.advance(&mut b).unwrap(), RecursiveAdvance::Complete) {
            break;
        }
    }
    let RecursiveOutcome::Center(center) = driver.outcome() else {
        panic!()
    };
    let upper = center.lift_receipt().unwrap().level().contact().clone();
    let lower = center
        .lift_receipt()
        .unwrap()
        .child()
        .lift_receipt()
        .unwrap()
        .level()
        .contact()
        .clone();
    let parent = super::super::embedding::SupportEmbedding::contact(upper, &mut b).unwrap();
    let child =
        super::super::embedding::SupportEmbedding::nested(parent.clone(), lower, &mut b).unwrap();
    let checked = CheckedRecursiveCenter::new(
        RecursiveCenterOrigin::BoundaryFree {
            center: center.clone(),
            history,
        },
        &mut b,
    )
    .unwrap();
    let adapted = adapt_recursive_center(checked, "nested_actual_adapt", &mut b).unwrap();
    let blowup = blowup_recursive_center(adapted, "nested_actual_blowup", &mut b).unwrap();
    let mut nonempty = 0;
    let mut pairs = 0;
    let mut empty = 0;
    for (i, chart) in blowup.charts().iter().enumerate() {
        let ps = super::super::support::transport_embedding(
            chart.geometry().clone(),
            parent.clone(),
            &format!("nested_actual_parent{i}"),
            &mut b,
        )
        .unwrap();
        let cs = super::super::support::transport_embedding(
            chart.geometry().clone(),
            child.clone(),
            &format!("nested_actual_child{i}"),
            &mut b,
        )
        .unwrap();
        if cs.saturation().empty() {
            empty += 1;
        }
        let cover = super::super::refinement::RefinedSupportCover::prepare(
            ps.clone(),
            cs.clone(),
            &format!("nested_actual_joint{i}"),
            &mut b,
        )
        .unwrap();
        let count = ps.opens().len() * cs.opens().len();
        assert_eq!(cover.opens().len() + cover.empty().len(), count);
        pairs += count;
        for open in cover.opens() {
            assert_eq!(
                open.frame().free_axes().len(),
                chart.geometry().frame().free_axes().len() - 2
            );
            assert!(open.parent_map().checked_derivatives() > 0);
            assert!(open.child_map().checked_derivatives() > 0);
            nonempty += 1;
        }
    }
    assert!(nonempty > 0);
    assert!(empty > 0);
    println!(
        "nested actual physical charts {}, common pairs {pairs}, nonempty {nonempty}, empty strict supports {empty}",
        blowup.charts().len()
    );
}
