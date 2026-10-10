//! Sufficient-bound certification of the actual production candidate. H is
//! evaluated by saved native programs outside this scalar inequality helper.
use symbolica::domains::{
    float::{Float, RealBall},
    rational::Rational,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RadiusFailure {
    Invalid(&'static str),
    /// Increasing certificate precision can resolve this enclosure. If it
    /// persists, retry production in a higher numeric domain or report failure.
    Unresolved(&'static str),
    Unsafe(&'static str),
}

#[derive(Clone, Debug)]
pub(crate) struct RadiusProof {
    // Production needs only successful certificate admission. Keep the full
    // independently asserted enclosure diagnostics in numerical unit tests.
    #[cfg(test)]
    pub actual_level_upper: Rational,
    #[cfg(test)]
    pub intended_root_lower: Rational,
    #[cfg(test)]
    pub intended_root_upper: Rational,
    #[cfg(test)]
    pub relative_width_upper: Rational,
}

/// `actual_level` encloses H(lambda/L); `intended_level` encloses
/// H(lambda/(S*L)). The same independent saved coefficient enclosures supply
/// both values. Their structural proof guarantees nonnegative coefficients and
/// powers >=2; this helper never substitutes rounded production coefficients.
pub(crate) fn certify(
    intended_candidate: &Rational,
    actual_level: &RealBall,
    intended_level: &RealBall,
    maximum_relative_width: &Rational,
    bits: u32,
) -> Result<RadiusProof, RadiusFailure> {
    if intended_candidate <= &Rational::from(0)
        || maximum_relative_width <= &Rational::from(0)
        || bits < 2
    {
        return Err(RadiusFailure::Invalid(
            "candidate and relative accuracy must be positive",
        ));
    }
    let actual_lower = actual_level.lower_bound();
    let actual_upper = actual_level.upper_bound();
    let intended_lower = intended_level.lower_bound();
    let intended_upper = intended_level.upper_bound();
    if [
        &actual_lower,
        &actual_upper,
        &intended_lower,
        &intended_upper,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(RadiusFailure::Unresolved("nonfinite envelope enclosure"));
    }
    let zero = Float::with_val(bits, 0);
    let one = Float::with_val(bits, 1);
    if actual_upper < zero {
        return Err(RadiusFailure::Invalid(
            "envelope enclosure contradicts structural nonnegativity",
        ));
    }
    if actual_lower >= one {
        return Err(RadiusFailure::Unsafe(
            "actual strength is outside the sufficient causal radius",
        ));
    }
    if actual_upper >= one {
        return Err(RadiusFailure::Unresolved(
            "actual-strength causal slack is not certified",
        ));
    }
    if intended_lower <= zero {
        return Err(RadiusFailure::Unresolved(
            "positive intended-root level is not certified",
        ));
    }

    // For every admissible positive polynomial, its root lies between x and
    // x/H(x). Native outward division encloses the latter for all coefficient
    // values in the independently certified input intervals.
    let candidate = RealBall::from_rational_bounds(intended_candidate, intended_candidate, bits);
    let scaled = candidate.clone() / intended_level;
    let candidate_lower = candidate.lower_bound();
    let candidate_upper = candidate.upper_bound();
    let scaled_lower = scaled.lower_bound();
    let scaled_upper = scaled.upper_bound();
    if !scaled_lower.is_finite() || !scaled_upper.is_finite() {
        return Err(RadiusFailure::Unresolved(
            "nonfinite intended-root enclosure",
        ));
    }
    let lower = if scaled_lower < candidate_lower {
        scaled_lower
    } else {
        candidate_lower
    }
    .to_rational();
    let upper = if scaled_upper > candidate_upper {
        scaled_upper
    } else {
        candidate_upper
    }
    .to_rational();
    if lower <= 0 {
        return Err(RadiusFailure::Unresolved(
            "positive intended-root lower bound is not certified",
        ));
    }
    let relative_width = (&upper - &lower) / &lower;
    if relative_width > *maximum_relative_width {
        return Err(RadiusFailure::Unresolved(
            "intended-root approximation accuracy is not certified",
        ));
    }
    Ok(RadiusProof {
        #[cfg(test)]
        actual_level_upper: actual_upper.to_rational(),
        #[cfg(test)]
        intended_root_lower: lower,
        #[cfg(test)]
        intended_root_upper: upper,
        #[cfg(test)]
        relative_width_upper: relative_width,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(value: Rational, bits: u32) -> RealBall {
        RealBall::from_rational_bounds(&value, &value, bits)
    }

    #[test]
    fn production_slack_and_intended_root_accuracy_are_separate_gates() {
        let bits = 128;
        let root = Rational::from((1, 2));
        let tolerance = Rational::from((1, 1_000_000));
        let result = certify(
            &root,
            &point(Rational::from((16, 25)), bits),
            &point(Rational::from(1), bits),
            &tolerance,
            bits,
        )
        .unwrap();
        assert_eq!(result.intended_root_lower, root);
        assert_eq!(result.intended_root_upper, root);
        assert_eq!(result.relative_width_upper, Rational::from(0));
        assert!(result.actual_level_upper < 1);
        // A smaller but substantially wrong solved radius is safe; it must
        // nevertheless fail the mathematical prescription's accuracy gate.
        let wrong = Rational::from((2, 5));
        assert!(matches!(
            certify(
                &wrong,
                &point(Rational::from((256, 625)), bits),
                &point(Rational::from((16, 25)), bits),
                &tolerance,
                bits
            ),
            Err(RadiusFailure::Unresolved(
                "intended-root approximation accuracy is not certified"
            ))
        ));
        assert!(matches!(
            certify(
                &root,
                &point(Rational::from(1), bits),
                &point(Rational::from(1), bits),
                &tolerance,
                bits
            ),
            Err(RadiusFailure::Unsafe(_))
        ));
    }

    #[test]
    fn finite_precision_slack_can_require_a_higher_certificate_precision() {
        let root = Rational::from((1, 2));
        let tolerance = Rational::from((1, 1_000_000));
        let lower = Rational::from((999_999, 1_000_000));
        let uncertain =
            RealBall::from_rational_bounds(&lower, &Rational::from((1_000_001, 1_000_000)), 128);
        assert!(matches!(
            certify(
                &root,
                &uncertain,
                &point(Rational::from(1), 128),
                &tolerance,
                128
            ),
            Err(RadiusFailure::Unresolved(_))
        ));
        let resolved = point(lower, 192);
        assert!(
            certify(
                &root,
                &resolved,
                &point(Rational::from(1), 192),
                &tolerance,
                192
            )
            .is_ok()
        );
    }

    #[test]
    fn residual_scaling_encloses_roots_on_both_sides_for_positive_even_polynomials() {
        // Native exact rational operations form independent controls with the
        // known root r=1/2. Mix powers2,4,8 with positive coefficients chosen so
        // H(r)=1; no numerical solver or rounded production inputs enter here.
        for weights in [(1, 0, 0), (1, 2, 0), (2, 3, 5), (0, 0, 1)] {
            let total = weights.0 + weights.1 + weights.2;
            for (n, d) in [(1, 4), (2, 5), (1, 2), (3, 5), (3, 4)] {
                let candidate = Rational::from((n, d));
                let square = &candidate * &candidate;
                let fourth = &square * &square;
                let eighth = &fourth * &fourth;
                let level = (Rational::from(4 * weights.0) * square
                    + Rational::from(16 * weights.1) * fourth
                    + Rational::from(256 * weights.2) * eighth)
                    / Rational::from(total);
                let result = certify(
                    &candidate,
                    &point(Rational::from((1, 4)), 192),
                    &point(level, 192),
                    &Rational::from(1000),
                    192,
                )
                .unwrap();
                assert!(result.intended_root_lower <= (1, 2));
                assert!(result.intended_root_upper >= (1, 2));
            }
        }
    }
}
