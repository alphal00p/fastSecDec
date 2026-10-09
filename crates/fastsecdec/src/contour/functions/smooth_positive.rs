//! Stable smooth envelopes with native arithmetic and Symbolica-owned jets.
//!
//! Both numerical branches evaluate the same analytic positive-part function.
//! Derivative hooks describe that function, never the branch selection used to
//! avoid cancellation. Complex arguments retain the holomorphic square root.
use super::dynamic::{failure, numeric::Number};
use std::sync::LazyLock;
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo, Symbol},
    domains::float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, FloatLike, Real},
    function, symbol,
};

#[cfg(test)]
mod tests;

pub(crate) fn register() {
    let _ = *HYPOT;
    let _ = *POSITIVE;
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "sign-aware recipe lowering follows callback validation"
    )
)]
pub(crate) fn positive_part(t: &Atom, delta: &Atom) -> Atom {
    function!(*POSITIVE, t, delta)
}

fn hypot(t: &Atom, delta: &Atom) -> Atom {
    function!(*HYPOT, t, delta)
}

static HYPOT: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::contour::smooth_hypot_v1",
        der = |view, index, out| {
            let Some(args) = view.as_fun_view() else {
                return;
            };
            if args.get_nargs() == 2 && index < 2 {
                **out = args.get(index).to_owned() / view.to_owned();
            }
        },
        eval = evaluation(false)
    )
});

static POSITIVE: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::contour::smooth_positive_v1",
        der = |view, index, out| {
            let Some(args) = view.as_fun_view() else {
                return;
            };
            if args.get_nargs() != 2 || index >= 2 {
                return;
            }
            let t = args.get(0).to_owned();
            let delta = args.get(1).to_owned();
            let h = hypot(&t, &delta);
            **out = if index == 0 {
                view.to_owned() / h
            } else {
                delta / (Atom::num(2) * h)
            };
        },
        eval = evaluation(true)
    )
});

fn evaluation(positive: bool) -> EvaluationInfo {
    EvaluationInfo::new()
        .register(move |args: &[f64]| real(args, positive))
        .register(move |args: &[DoubleFloat]| real(args, positive))
        .register(move |args: &[Float]| real(args, positive))
        .register(move |args: &[ErrorPropagatingFloat<f64>]| real(args, positive))
        .register(move |args: &[ErrorPropagatingFloat<Float>]| real(args, positive))
        .register(move |args: &[Complex<f64>]| complex(args, positive))
        .register(move |args: &[Complex<DoubleFloat>]| complex(args, positive))
        .register(move |args: &[Complex<Float>]| complex(args, positive))
        .register(move |args: &[Complex<ErrorPropagatingFloat<f64>>]| complex(args, positive))
        .register(move |args: &[Complex<ErrorPropagatingFloat<Float>>]| complex(args, positive))
}

/// Centre-zero is insufficient when the imaginary component carries tracked
/// uncertainty. This predicate only selects the existing native real primitive.
trait ExactZero: Number {
    fn exact_zero(&self) -> bool;
}
macro_rules! exact_plain {
    ($($ty:ty),*) => {$(impl ExactZero for $ty {
        fn exact_zero(&self) -> bool { self.is_fully_zero() }
    })*};
}
exact_plain!(f64, DoubleFloat, Float);
impl<T: Number> ExactZero for ErrorPropagatingFloat<T>
where
    Self: Number,
{
    fn exact_zero(&self) -> bool {
        self.is_fully_zero() && self.get_absolute_error() == 0.
    }
}

fn real<T: Number>(args: &[T], positive: bool) -> T {
    let bits = args.first().map_or(53, |value| value.get_precision());
    let Some([t, delta]) = args.first_chunk::<2>().filter(|_| args.len() == 2) else {
        return invalid(bits, "smooth envelope requires two arguments");
    };
    if !t.is_finite() || !delta.is_finite() || delta <= &delta.zero() {
        return invalid(
            bits,
            "smooth envelope requires finite inputs and positive regularity",
        );
    }
    let h = t.hypot(delta);
    let result = if positive {
        let two = t.from_usize(2);
        let half_t = t.clone() / &two;
        let half_h = h / &two;
        if t < &t.zero() {
            let half_delta = delta.clone() / &two;
            half_delta.clone() * (half_delta / (half_h - half_t))
        } else {
            half_t + half_h
        }
    } else {
        h
    };
    if !result.is_finite() || result <= result.zero() {
        return invalid(bits, "smooth envelope became nonfinite or nonpositive");
    }
    result
}

fn complex<T: ExactZero>(args: &[Complex<T>], positive: bool) -> Complex<T> {
    let bits = args.first().map_or(53, |value| value.re.get_precision());
    let error = |reason| Complex::new(invalid::<T>(bits, reason), T::invalid(bits));
    let Some([t, delta]) = args.first_chunk::<2>().filter(|_| args.len() == 2) else {
        return error("smooth complex envelope requires two arguments");
    };
    if !t.re.is_finite()
        || !t.im.is_finite()
        || !delta.re.is_finite()
        || !delta.im.is_finite()
        || delta.re <= delta.re.zero()
    {
        return error("smooth complex envelope requires finite inputs and positive regularity");
    }
    if t.im.exact_zero() && delta.im.exact_zero() {
        let result = real(&[t.re.clone(), delta.re.clone()], positive);
        return Complex::new(result, t.im.zero());
    }
    // Complex::hypot is the analytic sqrt(t²+delta²), not the Hermitian norm.
    // In uncertain fixed-precision complex domains its intermediate squares can
    // overflow; preserve the failure so the existing caller can rescue in MP.
    let h = t.hypot(delta);
    let result = if positive {
        let two = t.from_usize(2);
        let half_t = t.clone() / &two;
        let half_h = h / &two;
        if t.re < t.re.zero() {
            let half_delta = delta.clone() / &two;
            half_delta.clone() * (half_delta / (half_h - half_t))
        } else {
            half_t + half_h
        }
    } else {
        h
    };
    if !result.re.is_finite() || !result.im.is_finite() || result.re <= result.re.zero() {
        return error("smooth complex envelope became nonfinite or nonpositive");
    }
    result
}

fn invalid<T: Number>(bits: u32, reason: &str) -> T {
    failure(reason.into());
    T::invalid(bits)
}
