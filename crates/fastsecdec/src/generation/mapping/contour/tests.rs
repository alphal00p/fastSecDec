use super::*;
use crate::contour::dynamic::DynamicEnvelope;
use symbolica::{parse, symbol};

#[test]
fn every_term_contributes_positive_factors_before_the_single_map_is_built() {
    let x = symbol!("complete_contour_chart::x");
    let f = parse!("1/4-complete_contour_chart::x");
    let positive = [
        parse!("1+complete_contour_chart::x"),
        parse!("2+complete_contour_chart::x^2"),
    ];
    let terms = positive
        .iter()
        .enumerate()
        .map(|(index, u)| PreparedTerm {
            powers: vec![Atom::Zero],
            prefactor: Atom::num(index + 1),
            residuals: vec![
                (f.clone(), Atom::num(-1), FactorSemantics::Causal),
                (u.clone(), Atom::num(-1), FactorSemantics::Positive),
            ],
        })
        .collect();
    let mut calls = 0;
    let (mapped, metadata) = deform_with(&[x], terms, |parameters, f, all_positive| {
        calls += 1;
        assert_eq!(all_positive, positive);
        let envelope = DynamicEnvelope::new(parameters, f.clone(), all_positive)?;
        // Only the second term introduces a quartic radius condition.
        assert_eq!(envelope.maximum_even_order(), 4);
        assert_eq!(envelope.positive_factors().len(), 2);
        SmoothContourMap::new(parameters, f, Atom::num((1, 10)))
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(mapped.len(), 2);
    assert_eq!(metadata.unwrap().positive_polynomials(), positive);
}

#[test]
fn chart_terms_cannot_change_the_causal_polynomial_or_build_a_partial_map() {
    let x = symbol!("complete_contour_conflict::x");
    let terms = [1, 2]
        .map(|offset| PreparedTerm {
            powers: vec![Atom::Zero],
            prefactor: Atom::one(),
            residuals: vec![(
                Atom::num(offset) - Atom::var(x),
                Atom::num(-1),
                FactorSemantics::Causal,
            )],
        })
        .into();
    let result = deform_with(&[x], terms, |_, _, _| {
        panic!("must check the entire chart first")
    });
    assert!(result.is_err());
}
