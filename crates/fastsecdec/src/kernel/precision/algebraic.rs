//! Algebraic callbacks additionally require native tracked whole-vector
//! acceptance at BOTH rescue tiers. Agreement of two rounded zeros is insufficient.
use super::*;
use symbolica::domains::float::SingleFloat;
type Tracked = ErrorPropagatingFloat<Float>;
pub(in crate::kernel) fn accepts_weighted_primary(
    value: ErrorPropagatingFloat<f64>,
    compiled_weighted: f64,
    weight: f64,
    tolerance: f64,
) -> bool {
    let weighted = value * ErrorPropagatingFloat::new_with_accuracy(weight, f64::INFINITY);
    weighted.to_f64().is_finite()
        && weighted.get_absolute_error() <= tolerance
        && (weighted.to_f64() - compiled_weighted).abs() <= tolerance
}
fn tracked(value: Float, bits: u32) -> Tracked {
    if value.is_zero() {
        Tracked::new_with_accuracy(value, f64::INFINITY)
    } else {
        // Native EPF range refusal is required before this successor can be
        // accepted. Never mask its finite error range with an arbitrary cap.
        Tracked::new(value, f64::from(bits) * std::f64::consts::LOG10_2 - 2.)
    }
}
fn accepts(
    checked: &[Tracked],
    raw: &[f64],
    width: usize,
    weight: f64,
    policy: &PrecisionPolicy,
    bits: u32,
) -> bool {
    // The supplied f64 weight is exactly represented at every rescue precision.
    // Let the native tracker detect underflow of the scaled uncertainty too.
    let weight = Tracked::new_with_accuracy(Float::with_val(bits, weight), f64::INFINITY);
    checked
        .chunks_exact(width)
        .zip(raw.chunks_exact(width))
        .all(|(values, raw)| {
            let scale = raw.iter().map(|v| v.abs()).fold(0., f64::max);
            let tolerance = policy.absolute_tolerance + policy.relative_tolerance * scale;
            values.iter().zip(raw).all(|(value, raw)| {
                let weighted = value.clone() * &weight;
                let center = weighted.to_f64();
                center.is_finite()
                    && raw.is_finite()
                    && weighted.get_absolute_error() <= tolerance
                    && (center - raw).abs() <= tolerance
            })
        })
}
pub(in crate::kernel) fn rescue_with_roots(
    exact: &ExpressionEvaluator<Complex<Rational>>,
    caches: (
        &mut PrecisionCache<Float>,
        Option<&mut PrecisionCache<Tracked>>,
    ),
    point: &[f64],
    output: &mut [f64],
    cancellation: &Cancellation,
    policy: &PrecisionPolicy,
    weight: f64,
) -> Result<PrecisionReport, KernelError> {
    let (cache, roots) = caches;
    let Some(roots) = roots else {
        return rescue(exact, cache, point, output, cancellation, policy, weight);
    };
    converge_checked(point, output, cancellation, policy, 1, |bits| {
        let scale = Float::with_val(bits, weight);
        let raw = cache
            .evaluate(
                exact,
                point,
                bits,
                |c| c.re.to_multi_prec_float(bits),
                |v| Float::with_val(bits, v),
            )?
            .iter()
            .map(|v| (v.clone() * &scale).to_f64())
            .collect::<Vec<_>>();
        let values = roots.evaluate(
            exact,
            point,
            bits,
            |c| tracked(c.re.to_multi_prec_float(bits), bits),
            |v| tracked(Float::with_val(bits, v), bits),
        )?;
        let accepted = accepts(values, &raw, 1, weight, policy, bits);
        Ok((raw, accepted))
    })
}
pub(in crate::kernel) fn rescue_complex_with_roots(
    exact: &ExpressionEvaluator<Complex<Rational>>,
    caches: (
        &mut PrecisionCache<Complex<Float>>,
        Option<&mut PrecisionCache<Complex<Tracked>>>,
    ),
    point: &[f64],
    output: &mut [f64],
    cancellation: &Cancellation,
    policy: &PrecisionPolicy,
    weight: f64,
) -> Result<PrecisionReport, KernelError> {
    let (cache, roots) = caches;
    let Some(roots) = roots else {
        return rescue_complex(exact, cache, point, output, cancellation, policy, weight);
    };
    converge_checked(point, output, cancellation, policy, 2, |bits| {
        let scale = Float::with_val(bits, weight);
        let raw = cache
            .evaluate(
                exact,
                point,
                bits,
                |c| {
                    Complex::new(
                        c.re.to_multi_prec_float(bits),
                        c.im.to_multi_prec_float(bits),
                    )
                },
                |v| Complex::new(Float::with_val(bits, v), Float::with_val(bits, 0)),
            )?
            .iter()
            .flat_map(|v| {
                [
                    (v.re.clone() * &scale).to_f64(),
                    (v.im.clone() * &scale).to_f64(),
                ]
            })
            .collect::<Vec<_>>();
        let values = roots
            .evaluate(
                exact,
                point,
                bits,
                |c| {
                    Complex::new(
                        tracked(c.re.to_multi_prec_float(bits), bits),
                        tracked(c.im.to_multi_prec_float(bits), bits),
                    )
                },
                |v| {
                    Complex::new(
                        tracked(Float::with_val(bits, v), bits),
                        tracked(Float::with_val(bits, 0), bits),
                    )
                },
            )?
            .iter()
            .flat_map(|v| [v.re.clone(), v.im.clone()])
            .collect::<Vec<_>>();
        let accepted = accepts(&values, &raw, 2, weight, policy, bits);
        Ok((raw, accepted))
    })
}
