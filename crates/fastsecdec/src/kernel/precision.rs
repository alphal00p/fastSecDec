use super::{KernelError, cancellation::Cancellation, precision_cache::PrecisionCache};
use symbolica::{
    domains::{
        float::{Complex, Float, RealLike},
        rational::Rational,
    },
    evaluate::ExpressionEvaluator,
};

/// Boundary rescue thresholds are a policy, not a mathematical error bound.
/// Rescued vectors must agree at two increasing MPFR precisions before return.
/// Complex coefficients use their real/imaginary infinity norm as the relative
/// scale; absolute tolerance still applies to both numerical components.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrecisionPolicy {
    pub boundary_threshold: f64,
    pub initial_bits: u32,
    pub max_bits: u32,
    pub relative_tolerance: f64,
    pub absolute_tolerance: f64,
}

impl Default for PrecisionPolicy {
    fn default() -> Self {
        Self {
            boundary_threshold: 1e-3,
            initial_bits: 128,
            max_bits: 4096,
            relative_tolerance: 1e-12,
            absolute_tolerance: 1e-300,
        }
    }
}

impl PrecisionPolicy {
    pub(super) fn validate(&self) -> Result<(), KernelError> {
        if !self.boundary_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.boundary_threshold)
            || self.initial_bits < 64
            || self.max_bits < self.initial_bits.saturating_mul(2)
            || !self.relative_tolerance.is_finite()
            || self.relative_tolerance <= 0.0
            || !self.absolute_tolerance.is_finite()
            || self.absolute_tolerance < 0.0
        {
            return Err(KernelError::PrecisionPolicy);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrecisionReport {
    pub rescued: bool,
    pub checked: bool,
    pub bits: u32,
}

pub(super) fn rescue(
    exact: &ExpressionEvaluator<Complex<Rational>>,
    cache: &mut PrecisionCache<Float>,
    point: &[f64],
    output: &mut [f64],
    cancellation: &Cancellation,
    policy: &PrecisionPolicy,
) -> Result<PrecisionReport, KernelError> {
    converge(point, output, cancellation, policy, 1, |bits| {
        cache
            .evaluate(
                exact,
                point,
                bits,
                |coefficient| coefficient.re.to_multi_prec_float(bits),
                |value| Float::with_val(bits, value),
            )
            .iter()
            .map(RealLike::to_f64)
            .collect()
    })
}

pub(super) fn rescue_complex(
    exact: &ExpressionEvaluator<Complex<Rational>>,
    cache: &mut PrecisionCache<Complex<Float>>,
    point: &[f64],
    output: &mut [f64],
    cancellation: &Cancellation,
    policy: &PrecisionPolicy,
) -> Result<PrecisionReport, KernelError> {
    converge(point, output, cancellation, policy, 2, |bits| {
        cache
            .evaluate(
                exact,
                point,
                bits,
                |coefficient| {
                    Complex::new(
                        coefficient.re.to_multi_prec_float(bits),
                        coefficient.im.to_multi_prec_float(bits),
                    )
                },
                |value| Complex::new(Float::with_val(bits, value), Float::with_val(bits, 0)),
            )
            .iter()
            .flat_map(|value| [value.re.to_f64(), value.im.to_f64()])
            .collect()
    })
}

// Native numeric evaluator IR is built before worker dispatch. Rescue only
// remaps exact constants; it never expands an Atom or consults a Workspace.
fn converge(
    point: &[f64],
    output: &mut [f64],
    cancellation: &Cancellation,
    policy: &PrecisionPolicy,
    width: usize,
    mut evaluate: impl FnMut(u32) -> Vec<f64>,
) -> Result<PrecisionReport, KernelError> {
    // Taylor differences can lose degree * log2(1/x) bits. Account for this
    // before testing agreement, so two equally rounded zeros are not accepted.
    let lost_bits = cancellation.lost_bits(point).ceil();
    let required = policy
        .initial_bits
        .max((lost_bits.min(u32::MAX as f64) as u32).saturating_add(64));
    if required > policy.max_bits / 2 {
        return Err(KernelError::PrecisionExhausted {
            bits: policy.max_bits,
        });
    }
    let mut bits = initial_tier(required, policy);
    let mut previous = evaluate(bits);
    loop {
        bits = bits.saturating_mul(2).min(policy.max_bits);
        let current = evaluate(bits);
        if current
            .chunks_exact(width)
            .zip(previous.chunks_exact(width))
            .all(|(a, b)| {
                let scale = a
                    .iter()
                    .chain(b)
                    .map(|value| value.abs())
                    .fold(0.0, f64::max);
                let tolerance = policy.absolute_tolerance + policy.relative_tolerance * scale;
                a.iter()
                    .zip(b)
                    .all(|(a, b)| a.is_finite() && b.is_finite() && (a - b).abs() <= tolerance)
            })
        {
            output.copy_from_slice(&current);
            return Ok(PrecisionReport {
                rescued: true,
                checked: true,
                bits,
            });
        }
        if bits == policy.max_bits {
            return Err(KernelError::PrecisionExhausted { bits });
        }
        previous = current;
    }
}

// Round upward in half-limb steps: doubling for the agreement check adds at
// most one native 64-bit limb, unlike binary tiers which can double arithmetic
// cost. Never reject a point merely because the final tier is incomplete.
fn initial_tier(required: u32, policy: &PrecisionPolicy) -> u32 {
    (required.saturating_add(31) / 32 * 32)
        .max(policy.initial_bits)
        .min(policy.max_bits / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_preserve_usable_precision_for_non_binary_limits() {
        let policy = PrecisionPolicy {
            max_bits: 3000,
            ..Default::default()
        };
        for required in 128..=1500 {
            let tier = initial_tier(required, &policy);
            assert!(tier >= required && 2 * tier <= policy.max_bits);
        }
        assert_eq!(initial_tier(1400, &policy), 1408);
        assert_eq!(initial_tier(1490, &policy), 1500);
        assert_eq!(initial_tier(129, &policy), 160);
    }
}
