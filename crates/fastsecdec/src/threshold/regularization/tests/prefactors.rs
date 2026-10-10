use crate::threshold::regularization::meromorphic::*;
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, AtomCore},
    parse,
    prelude::Rational,
    symbol,
};
#[test]
fn rational_gamma_prefactors_and_resource_admission() {
    let eps = symbol!("prefactor_gamma::eps");
    let too_large = Atom::var(eps).pow(65536u32);
    assert!(matches!(
        RationalPrefactor::admit(
            too_large,
            eps,
            Limits {
                integer_power: 100000,
                ..Limits::default()
            }
        ),
        Err(Error::ResourceIncomplete(_))
    ));
    let factored = (Atom::var(eps) + 1).pow(200u32) * (Atom::var(eps) + 2).pow(200u32);
    assert!(matches!(
        RationalPrefactor::admit(factored, eps, Limits::default()),
        Err(Error::ResourceIncomplete(_))
    ));
    assert!(matches!(
        RationalPrefactor::admit(
            Atom::one(),
            eps,
            Limits {
                degree: u16::MAX,
                ..Limits::default()
            }
        ),
        Err(Error::Invalid(_))
    ));
    println!(
        "PASS conservative native-syntax degree/term preflight refuses expansion before conversion"
    );
    for expression in [
        parse!("1/prefactor_gamma::eps"),
        parse!(
            "gamma(prefactor_gamma::eps)*gamma(1-2*prefactor_gamma::eps)/(gamma(1+prefactor_gamma::eps)*gamma(1-prefactor_gamma::eps)^2)"
        ),
        parse!(
            "(1+𝑖)*gamma(-1+2*prefactor_gamma::eps)^2/(prefactor_gamma::eps^3*gamma(1/3-prefactor_gamma::eps))"
        ),
    ] {
        let p = MeromorphicPrefactor::admit(expression.clone(), eps, Limits::default()).unwrap();
        assert_eq!(p.original(), &expression);
        let w = MeromorphicWitness::construct(
            vec![p],
            Some(&Rational::from(-1)),
            Some(&Rational::one()),
            Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert!(w.rational_witness().epsilon() > &Rational::from(-1));
        assert!(w.rational_witness().epsilon() < &Rational::one());
        for g in w.gamma_witnesses() {
            if let Some(d) = g.pole_distance() {
                assert!(d > &Rational::zero());
            }
        }
        println!(
            "PASS shared nonpole witness {} for {}",
            w.rational_witness().epsilon(),
            expression
        );
    }
    let reciprocal = MeromorphicPrefactor::admit(
        parse!("1/gamma(prefactor_gamma::eps)"),
        eps,
        Limits::default(),
    )
    .unwrap();
    let w = MeromorphicWitness::construct(vec![reciprocal], None, None, Limits::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(w.rational_witness().epsilon(), &Rational::zero());
    assert_eq!(w.gamma_witnesses()[0].argument_value(), &Rational::zero());
    assert!(w.gamma_witnesses()[0].pole_distance().is_none());
    for bad in [
        parse!("gamma(prefactor_gamma::eps^2)"),
        parse!("gamma(a+prefactor_gamma::eps)"),
        parse!("gamma(prefactor_gamma::eps)^(1/2)"),
        parse!("exp(1/prefactor_gamma::eps)"),
        parse!("gamma(0)"),
        parse!("sqrt(2)*gamma(prefactor_gamma::eps)"),
        Atom::num(0.25) * parse!("gamma(prefactor_gamma::eps)"),
    ] {
        assert!(
            matches!(
                MeromorphicPrefactor::admit(bad.clone(), eps, Limits::default()),
                Err(Error::Unsupported(_))
            ),
            "accepted {bad}"
        );
    }
    println!(
        "PASS reciprocal-Gamma zero is regular; exact family/refusal controls; no Gamma evaluation or infinity-cancellation at witness"
    );
}

use super::{restored, solve, source};
use crate::{generation, threshold};
use std::collections::BTreeMap;
use symbolica::{atom::Symbol, domains::float::Complex, evaluate::FunctionMap};
use threshold::{
    gcad::GcadKinematics,
    regularization::{Error as BridgeError, Limits as BridgeLimits, RegularizedFiber},
};
#[test]
fn exact_materialization() {
    let (x, t, a, eps) = symbol!(
        "exact_bound_probe::x",
        "exact_bound_probe::t",
        "exact_bound_probe::a",
        "exact_bound_probe::eps"
    );
    let numerator = Atom::one() + Atom::i() * Atom::var(a) * Atom::var(x) + Atom::var(x).pow(2);
    let input = source(
        x,
        eps,
        Atom::var(x) - Atom::var(a),
        -Atom::one() - Atom::var(eps),
        numerator,
    );
    let owner = solve(
        &input,
        GcadKinematics {
            runtime_parameters: vec![a],
            strict_positive: vec![Atom::var(a), Atom::one() - Atom::var(a)],
            ..Default::default()
        },
    );
    let fiber = RegularizedFiber::admit(
        owner,
        BTreeMap::from([(a, Rational::from((1, 2)))]),
        t,
        BridgeLimits::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_subtractions_per_axis: 3,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
    let name = bound.definitions()[0].head();
    let face = name.call(&[Atom::Zero, Atom::one()][..]);
    let expected = Atom::num(2) + Atom::i() * Atom::num((1, 2));
    assert_eq!(bound.materialize_exact(&face).unwrap(), expected);
    let derivative = |order: Atom, tag: Atom| {
        Symbol::DERIVATIVE.call(&[Atom::Zero, order, Atom::var(name), tag, Atom::one()][..])
    };
    assert_eq!(
        bound
            .materialize_exact(&derivative(Atom::one(), Atom::Zero))
            .unwrap(),
        Atom::num(2) + Atom::i() * Atom::num((1, 2))
    );
    assert_eq!(
        bound
            .materialize_exact(&derivative(Atom::num(2), Atom::Zero))
            .unwrap(),
        Atom::num(2)
    );
    assert!(
        bound
            .materialize_exact(&(derivative(Atom::num(2), Atom::Zero) - Atom::num(2)))
            .unwrap()
            .is_zero()
    );
    for bad in [
        name.call(&[Atom::Zero, Atom::var(t)][..]),
        Atom::var(eps),
        Atom::var(a),
        Atom::var(x),
        name.call(&[Atom::num(12), Atom::one()][..]),
        name.call(&[Atom::num((1, 2)), Atom::one()][..]),
        derivative(Atom::num(4), Atom::Zero),
        derivative(Atom::num(-1), Atom::Zero),
        derivative(Atom::num((1, 2)), Atom::Zero),
        name.call(Atom::Zero),
        Symbol::DERIVATIVE.call(
            &[
                Atom::one(),
                Atom::one(),
                Atom::var(name),
                Atom::Zero,
                Atom::one(),
            ][..],
        ),
        Atom::var(symbol!("exact_bound_probe::foreign")),
    ] {
        assert!(bound.materialize_exact(&bad).is_err(), "accepted {bad}");
    }
    let mut rebuilt = FunctionMap::new();
    for d in bound.definitions() {
        assert!(!d.body().contains_symbol(a));
        let options = if d.derivative_order().is_some() {
            symbolica::evaluate::FunctionRegistrationOptions::new()
                .inlining(symbolica::evaluate::InliningPolicy::Always)
        } else {
            symbolica::evaluate::FunctionRegistrationOptions::new()
        };
        rebuilt
            .add_tagged_function_with_options(
                d.head(),
                d.tags().to_vec(),
                d.formals().to_vec(),
                d.body().clone(),
                options,
            )
            .unwrap();
    }
    let outputs = [face, derivative(Atom::one(), Atom::Zero)];
    let mut native = restored(&outputs, &[], bound.functions().clone());
    let mut replay = restored(&outputs, &[], rebuilt);
    let mut before = [Complex::new(0., 0.); 2];
    let mut after = before;
    native.evaluate(&[], &mut before);
    replay.evaluate(&[], &mut after);
    assert_eq!(before, after);
    let huge = source(
        x,
        eps,
        Atom::var(x) - Atom::num((1, 2)),
        -Atom::var(eps),
        Atom::var(x).pow(4294967296u64),
    );
    assert!(matches!(
        RegularizedFiber::admit(
            solve(&huge, Default::default()),
            BTreeMap::new(),
            t,
            BridgeLimits::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(BridgeError::ResourceIncomplete(_))
    ));

    println!(
        "PASS exact-only bound numerator/derivative materialization, malformed/nonconstant refusals, exact cancellation, same-loop staged definition reconstruction"
    );
}

#[test]
fn native_qi_rational_certificate_and_source_class() {
    let eps = symbol!("prefactor_certificate::eps");
    for expression in [
        parse!("1/prefactor_certificate::eps"),
        parse!("(1+𝑖)/(prefactor_certificate::eps*(1-prefactor_certificate::eps))"),
        parse!("(2+3𝑖*prefactor_certificate::eps)/(1+prefactor_certificate::eps^2)"),
        Atom::Zero,
    ] {
        let proof = RationalPrefactor::admit(expression.clone(), eps, Limits::default())
            .unwrap_or_else(|error| panic!("{expression}: {error:?}"));
        assert_eq!(proof.original(), &expression);
        assert_eq!(proof.regulator(), eps);
        assert!(!proof.denominator().is_zero());
        assert!(!proof.numerator().contains_symbol(symbol!("unknown")));
        assert!(proof.denominator_degree() <= 2);
        let witness = SharedWitness::construct(&[proof], None, None, Limits::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        assert_eq!(witness.denominator_values().len(), 1);
    }
    for bad in [
        Atom::num(0.125) / (1 + Atom::var(eps)),
        parse!("sqrt(2)/(1+prefactor_certificate::eps)"),
        parse!("exp(1/prefactor_certificate::eps)"),
        parse!("gamma(prefactor_certificate::eps)"),
        parse!("f(prefactor_certificate::eps)"),
        parse!("a/(1+prefactor_certificate::eps)"),
        parse!("prefactor_certificate::eps^(1/2)"),
    ] {
        assert!(
            matches!(
                RationalPrefactor::admit(bad.clone(), eps, Limits::default()),
                Err(Error::Unsupported(_))
            ),
            "wrong admission for {bad}"
        );
    }
}

#[test]
fn one_shared_witness_avoids_every_term_pole_before_partition() {
    let eps = symbol!("prefactor_certificate::eps");
    let proofs = [
        parse!("1/(prefactor_certificate::eps+1/6)"),
        parse!("1/prefactor_certificate::eps"),
    ]
    .into_iter()
    .map(|e| RationalPrefactor::admit(e, eps, Limits::default()).unwrap())
    .collect::<Vec<_>>();
    let witness = SharedWitness::construct(
        &proofs,
        Some(&Rational::from(-1)),
        Some(&Rational::one()),
        Limits::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(witness.tested(), 3);
    assert_eq!(witness.prefactors().len(), 2);
    assert_eq!(
        witness.strip(),
        (&Some(Rational::from(-1)), &Some(Rational::one()))
    );
    assert_eq!(witness.epsilon(), &Rational::from((1, 6)));
    assert_eq!(witness.denominator_values().len(), 2);
    assert!(
        witness
            .denominator_values()
            .iter()
            .all(|v| !v.re.is_zero() || !v.im.is_zero())
    );
    assert!(matches!(
        SharedWitness::construct(
            &proofs,
            Some(&Rational::one()),
            Some(&Rational::zero()),
            Limits::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(Error::Invalid(_))
    ));
    let foreign = RationalPrefactor::admit(
        Atom::one(),
        symbol!("prefactor_certificate::eta"),
        Limits::default(),
    )
    .unwrap();
    assert!(matches!(
        SharedWitness::construct(
            &[proofs[0].clone(), foreign],
            None,
            None,
            Limits::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn witness_has_finite_budget_and_cooperative_cancellation() {
    let eps = symbol!("prefactor_certificate::eps");
    let proof = RationalPrefactor::admit(
        parse!("1/prefactor_certificate::eps^2"),
        eps,
        Limits::default(),
    )
    .unwrap();
    assert!(matches!(
        SharedWitness::construct(
            std::slice::from_ref(&proof),
            None,
            None,
            Limits {
                witness_candidates: 2,
                ..Limits::default()
            },
            |_| ControlFlow::Continue(())
        ),
        Err(Error::ResourceIncomplete(_))
    ));
    assert_eq!(
        SharedWitness::construct(&[proof], None, None, Limits::default(), |_| {
            ControlFlow::Break(())
        })
        .unwrap_err(),
        Error::Cancelled
    );
    assert!(matches!(
        RationalPrefactor::admit(
            parse!("prefactor_certificate::eps^3"),
            eps,
            Limits {
                degree: 2,
                ..Limits::default()
            }
        ),
        Err(Error::ResourceIncomplete(_))
    ));
}
