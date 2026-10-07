//! One-shot native evaluation of the saved exact offset at a physical point.
//! This never constructs an evaluator from an expression during artifact loading.
use super::{KernelError, PrecisionClass};
use std::collections::{BTreeMap, HashMap};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::{Complex, DoubleFloat, Float, Real, RealLike},
    evaluate::EvaluationDomain,
};

fn native<T: Real + EvaluationDomain>(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    bits: u32,
    number: impl Fn(f64) -> T,
    components: impl Fn(T) -> Vec<f64>,
) -> Result<Vec<f64>, KernelError> {
    let point = point
        .iter()
        .map(|(symbol, value)| (Atom::var(*symbol), number(*value)))
        .collect::<HashMap<_, _>>();
    expressions
        .iter()
        .map(|expression| {
            expression
                .evaluate_with_prec(&point, bits)
                .map(&components)
                .map_err(|error| KernelError::Compilation(format!("exact offset: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|values| values.into_iter().flatten().collect())
}

pub(super) fn evaluate(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    complex: bool,
) -> Result<Vec<f64>, KernelError> {
    // Match the former zero-dimensional kernel's native distance policy: a
    // nonfinite final output retries the whole coefficient vector in the
    // next numeric domain. Conversion happens after native evaluation.
    let mut confirm_zero = false;
    for class in [
        PrecisionClass::F64,
        PrecisionClass::DoubleFloat,
        PrecisionClass::Arbitrary,
    ] {
        if confirm_zero && class == PrecisionClass::DoubleFloat {
            continue;
        }
        let bits = class.bits();
        let values = match (class, complex) {
            (PrecisionClass::F64, false) => native(expressions, point, bits, |v| v, |v| vec![v])?,
            (PrecisionClass::F64, true) => native(
                expressions,
                point,
                bits,
                |v| Complex::new(v, 0.0),
                |v| vec![v.re, v.im],
            )?,
            (PrecisionClass::DoubleFloat, false) => {
                native(expressions, point, bits, DoubleFloat::from, |v| {
                    vec![v.to_f64()]
                })?
            }
            (PrecisionClass::DoubleFloat, true) => native(
                expressions,
                point,
                bits,
                |v| Complex::new(DoubleFloat::from(v), DoubleFloat::from(0.0)),
                |v| vec![v.re.to_f64(), v.im.to_f64()],
            )?,
            (PrecisionClass::Arbitrary, false) => native(
                expressions,
                point,
                bits,
                |v| Float::with_val(bits, v),
                |v| vec![v.to_f64()],
            )?,
            (PrecisionClass::Arbitrary, true) => native(
                expressions,
                point,
                bits,
                |v| Complex::new(Float::with_val(bits, v), Float::with_val(bits, 0)),
                |v| vec![v.re.to_f64(), v.im.to_f64()],
            )?,
            (PrecisionClass::Unstable, _) => unreachable!(),
        };
        if values.iter().all(|value| value.is_finite()) {
            // Direct expression order can underflow an intermediate that the
            // optimized helper happened to avoid. Confirm a whole coefficient
            // zero once at binding; an individual zero Re/Im part is ordinary.
            // Literal zeros need no evaluation, and true point cancellations
            // remain zero after the native arbitrary-precision confirmation.
            confirm_zero = class != PrecisionClass::Arbitrary
                && expressions
                    .iter()
                    .zip(values.chunks_exact(if complex { 2 } else { 1 }))
                    .any(|(expression, value)| {
                        !expression.is_zero() && value.iter().all(|v| *v == 0.0)
                    });
            if confirm_zero {
                continue;
            }
            return Ok(values);
        }
    }
    Err(KernelError::NonFinite)
}

#[cfg(test)]
mod tests;
