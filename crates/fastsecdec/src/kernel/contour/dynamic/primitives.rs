//! Certified scalar operations for saved dynamic primitives. Every arithmetic
//! operation and square root is Numerica's native ball operation. Tightening
//! uses an independently admitted mathematical lower bound; it never repairs
//! a production coefficient or substitutes a positive numerical floor.
use super::radius::RadiusFailure;
use symbolica::domains::{
    float::{FloatLike, Real, RealBall},
    rational::Rational,
};

fn finite_bounds(value: &RealBall) -> Result<(Rational, Rational), RadiusFailure> {
    let lower = value.lower_bound();
    let upper = value.upper_bound();
    if !lower.is_finite() || !upper.is_finite() {
        return Err(RadiusFailure::Unresolved("nonfinite primitive enclosure"));
    }
    Ok((lower.to_rational(), upper.to_rational()))
}

/// Intersect with an exact, separately established lower bound. A negative
/// upper bound for a sum of squares is contradictory evidence, not rounding
/// noise to discard. Callers must escalate or report this explicit failure.
pub(crate) fn with_proved_lower_bound(
    value: &RealBall,
    proved_lower: &Rational,
) -> Result<RealBall, RadiusFailure> {
    let (lower, upper) = finite_bounds(value)?;
    if upper < *proved_lower {
        return Err(RadiusFailure::Invalid(
            "primitive enclosure contradicts its admitted lower bound",
        ));
    }
    let lower = lower.max(proved_lower.clone());
    Ok(RealBall::from_rational_bounds(
        &lower,
        &upper,
        value.get_precision(),
    ))
}

pub(crate) fn nonnegative(value: &RealBall) -> Result<RealBall, RadiusFailure> {
    with_proved_lower_bound(value, &Rational::from(0))
}

/// mu(t,delta), with delta an admitted positive exact regularity constant.
/// Rationalization is used only when the entire argument is negative. Both
/// forms use outward native arithmetic and enclose the same smooth function.
pub(crate) fn smooth_positive(
    argument: &RealBall,
    regularity: &Rational,
) -> Result<RealBall, RadiusFailure> {
    if regularity <= &Rational::from(0) {
        return Err(RadiusFailure::Invalid("nonpositive smoothing regularity"));
    }
    finite_bounds(argument)?;
    let bits = argument.get_precision();
    let delta = RealBall::from_rational_bounds(regularity, regularity, bits);
    let delta_squared = delta.clone() * &delta;
    let square = nonnegative(&(argument.clone() * argument))?;
    let radicand = square + &delta_squared;
    let root = radicand.sqrt();
    finite_bounds(&root)?;
    let two = RealBall::from_rational_bounds(&Rational::from(2), &Rational::from(2), bits);
    let value = if argument.is_strictly_negative() {
        delta_squared / (two * (root - argument))
    } else {
        (argument.clone() + root) / two
    };
    nonnegative(&value)
}

/// The extra delta^2 under the spectral gap root is essential at repeated
/// eigenvalues. The outer smooth positive part alone would not remove the
/// spectral square-root cusp.
pub(crate) fn spectral_positive(
    mean: &RealBall,
    gap_squared: &RealBall,
    regularity: &Rational,
) -> Result<RealBall, RadiusFailure> {
    finite_bounds(mean)?;
    if regularity <= &Rational::from(0) {
        return Err(RadiusFailure::Invalid("nonpositive smoothing regularity"));
    }
    let bits = mean.get_precision().min(gap_squared.get_precision());
    let delta = RealBall::from_rational_bounds(regularity, regularity, bits);
    let gap = nonnegative(gap_squared)?;
    let root = (gap + delta.clone() * delta).sqrt();
    finite_bounds(&root)?;
    smooth_positive(&(mean.clone() + root), regularity)
}

/// Independent baseline C_k=d_k^2/U^2. The exact U proof is associated with
/// the saved factor identity at admission, before this numerical operation.
pub(crate) fn normalized_square(
    coefficient: &RealBall,
    original_positive: &RealBall,
    proved_lower: &Rational,
) -> Result<RealBall, RadiusFailure> {
    if proved_lower <= &Rational::from(0) {
        return Err(RadiusFailure::Invalid("nonpositive residual proof"));
    }
    let positive = with_proved_lower_bound(original_positive, proved_lower)?;
    if !positive.is_strictly_positive() {
        return Err(RadiusFailure::Unresolved(
            "positive residual bound is unresolved",
        ));
    }
    let ratio = coefficient.clone() / positive;
    nonnegative(&(ratio.clone() * ratio))
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::domains::integer::Integer;

    fn exact(value: Rational, bits: u32) -> RealBall {
        RealBall::from_rational_bounds(&value, &value, bits)
    }
    fn quadratic_residual(value: &Rational, t: &Rational, delta: &Rational) -> Rational {
        value * value - t * value - delta * delta / Rational::from(4)
    }

    #[test]
    fn native_smoothing_enclosures_cover_extreme_and_zero_arguments() {
        let delta = Rational::from((1, 1000));
        let enormous = Rational::from(Integer::from(10).pow(308));
        for bits in [53, 96, 192] {
            for t in [
                Rational::from(0),
                Rational::from(1),
                Rational::from(-1),
                enormous.clone(),
                -enormous.clone(),
            ] {
                let result = smooth_positive(&exact(t.clone(), bits), &delta).unwrap();
                let (lower, upper) = finite_bounds(&result).unwrap();
                assert!(lower >= 0);
                assert!(upper > 0);
                assert!(quadratic_residual(&lower, &t, &delta) <= 0);
                assert!(quadratic_residual(&upper, &t, &delta) >= 0);
                if t == 0 {
                    assert!(lower <= delta.clone() / Rational::from(2));
                    assert!(upper >= delta.clone() / Rational::from(2));
                }
            }
        }
    }

    #[test]
    fn proved_intersections_keep_degeneracy_and_reject_contradictions() {
        let bits = 128;
        let zero_crossing = RealBall::from_rational_bounds(
            &Rational::from((-1, 100)),
            &Rational::from((1, 100)),
            bits,
        );
        let tightened = nonnegative(&zero_crossing).unwrap();
        assert_eq!(tightened.lower_bound().to_rational(), Rational::from(0));
        assert!(nonnegative(&exact(Rational::from(-1), bits)).is_err());
        assert!(
            normalized_square(
                &exact(Rational::from(1), bits),
                &exact(Rational::from(1), bits),
                &Rational::from(2)
            )
            .is_err()
        );
        let normalized = normalized_square(
            &exact(Rational::from(0), bits),
            &zero_crossing,
            &Rational::from((1, 1000)),
        )
        .unwrap();
        assert!(normalized.is_fully_zero());
        let spectral = spectral_positive(
            &exact(Rational::from(0), bits),
            &exact(Rational::from(0), bits),
            &Rational::from((1, 1000)),
        )
        .unwrap();
        assert!(spectral.is_strictly_positive());
    }
}
