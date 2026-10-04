#!/usr/bin/env rust-script
//! Reproduce literal-variable matching in Symbolica's native function Series.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! rust-version = "1.96"
//!
//! [dependencies]
//! symbolica = { version = "=3.0.1", default-features = false, features = ["integer-gmp", "float-mpfr"] }
//!
//! [workspace]
//! ```

use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::Pattern,
    symbol,
    transcendental::{TranscendentalFunctions, euler_gamma},
};

fn rename(expression: &Atom, from: Symbol, to: Symbol) -> Atom {
    expression
        .replace(Pattern::Literal(Atom::var(from)))
        .with(Pattern::Literal(Atom::var(to)))
}

fn main() {
    let eps = symbol!("mre::eps");
    let eps_ = symbol!("mre::eps_");
    let plain_gamma = Atom::var(eps).gamma().series(eps, 0, 0).unwrap();
    let underscored_gamma = Atom::var(eps_).gamma().series(eps_, 0, 0).unwrap();
    let expected_gamma = Atom::var(eps_).pow(-1) - Atom::var(euler_gamma());
    println!(
        "Gamma(eps):  {} ({} terms)",
        plain_gamma.to_atom(),
        plain_gamma.terms().count()
    );
    println!(
        "Gamma(eps_): {} ({} terms)",
        underscored_gamma.to_atom(),
        underscored_gamma.terms().count()
    );
    println!("Expected Gamma(eps_): {expected_gamma}");

    // Also exercise the ordinary unknown-function fallback at first order.
    // The function is deliberately left undefined; its derivatives stay native.
    let function = symbol!("mre::f");
    let expression = function.call(&[Atom::one() + Atom::var(eps), Atom::var(eps).sin()][..]);
    let plain_composed = expression.series(eps, 0, 1).unwrap().to_atom();
    let underscored_composed = rename(&expression, eps, eps_)
        .series(eps_, 0, 1)
        .unwrap()
        .to_atom();
    let expected_composed = rename(&plain_composed, eps, eps_);
    println!("f(1+eps,sin(eps)):   {plain_composed}");
    println!("f(1+eps_,sin(eps_)): {underscored_composed}");
    println!("Expected composed series: {expected_composed}");

    // Check the control and nonempty vectors before equality: two empty native
    // Series must never make this reproducer report success.
    assert_eq!(
        plain_gamma.terms().count(),
        2,
        "ordinary-name Gamma control"
    );
    assert_eq!(rename(&plain_gamma.to_atom(), eps, eps_), expected_gamma);
    assert!(!plain_composed.is_zero(), "ordinary-name function control");
    assert_eq!(
        underscored_gamma.terms().count(),
        2,
        "BUG: a literal expansion variable ending in '_' becomes a wildcard"
    );
    assert_eq!(underscored_gamma.to_atom(), expected_gamma);
    assert!(
        !underscored_composed.is_zero(),
        "composed function lost all terms"
    );
    assert_eq!(underscored_composed, expected_composed);
    println!("PASS: both literal variable names preserve the same nonzero series.");
}
