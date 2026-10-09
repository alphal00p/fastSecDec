//! The causal boundary value is a domain convention, not a replacement CAS.
//!
//! Symbolica owns differentiation, series, evaluation and serialization. Its
//! principal logarithm needs one prescribed boundary value at negative-real
//! arguments, where arithmetic may erase the sign of an infinitesimal zero.
#[expect(
    dead_code,
    reason = "dynamic callback foundation precedes recipe admission"
)]
pub(crate) mod dynamic;
pub(crate) mod smooth_positive;

use std::sync::LazyLock;
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo, Symbol},
    domains::float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, Real, RealLike},
    function, symbol,
};

/// Register before importing an artifact's Symbolica state in a fresh process.
pub(crate) fn register() {
    let _ = *CAUSAL_LOG;
    smooth_positive::register();
}

pub(crate) fn causal_log(value: &Atom) -> Atom {
    function!(*CAUSAL_LOG, value)
}

static CAUSAL_LOG: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::contour::causal_log_v1",
        der = |view, index, out| {
            if index == 0 {
                let argument = view.as_fun_view().unwrap().get(0);
                **out = argument.pow(-1);
            }
        },
        eval = EvaluationInfo::new()
            .register(|args: &[Complex<f64>]| lower_lip(&args[0]))
            .register(|args: &[Complex<DoubleFloat>]| lower_lip(&args[0]))
            .register(|args: &[Complex<Float>]| lower_lip(&args[0]))
            .register(|args: &[Complex<ErrorPropagatingFloat<f64>>]| lower_lip(&args[0]))
            .register(|args: &[Complex<ErrorPropagatingFloat<DoubleFloat>>]| lower_lip(&args[0]))
            .register(|args: &[Complex<ErrorPropagatingFloat<Float>>]| lower_lip(&args[0]))
    )
});

fn lower_lip<T: Real + RealLike + PartialOrd>(value: &Complex<T>) -> Complex<T> {
    let mut logarithm = value.log();
    if value.im.is_zero() && value.re < value.re.zero() {
        // Keep the owner's propagated uncertainty, including uncertainty in an
        // imaginary component whose central value happens to be zero.
        logarithm.im = logarithm.im.norm().neg();
    }
    logarithm
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::parse;

    #[test]
    fn symbolic_causal_log_uses_native_higher_derivatives() {
        let x = symbol!("contour_callback_test::x");
        let value = Atom::var(x);
        let logarithm = causal_log(&value);
        assert_eq!(logarithm.derivative(x), value.pow(-1));
        assert_eq!(logarithm.derivative(x).derivative(x), -value.pow(-2));
        assert_eq!(
            causal_log(&parse!("1+contour_callback_test::x^2")).derivative(x),
            parse!("2*contour_callback_test::x/(1+contour_callback_test::x^2)")
        );
    }

    #[test]
    fn causal_boundary_does_not_hide_wrong_side_values() {
        for zero in [0.0, -0.0] {
            let value = lower_lip(&Complex::new(-2.0, zero));
            assert_eq!(value.im, -std::f64::consts::PI);
            assert_eq!(value.re, 2_f64.ln());
        }
        assert!(lower_lip(&Complex::new(-2.0, 1e-20)).im > 0.0);
        assert!(lower_lip(&Complex::new(-2.0, -1e-20)).im < 0.0);
        assert_eq!(lower_lip(&Complex::new(2.0, 0.0)).im, 0.0);
    }
}
