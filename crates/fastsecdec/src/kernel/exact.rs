//! One-shot native evaluation of the saved exact offset at a physical point.
//! This never constructs an evaluator from an expression during artifact loading.
use super::contour::dynamic::validation::Validation;
use super::{KernelError, PrecisionClass};
use crate::contour::functions::dynamic::{
    CandidateNumber,
    observation::Candidate,
    requests::{Bundle, ExactRequest},
};
use std::collections::{BTreeMap, HashMap};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::float::{Complex, DoubleFloat, Float, Real, RealLike},
    domains::rational::Rational,
    evaluate::{EvaluationDomain, evaluate_multiple_with_prec},
};

struct Checked<'a> {
    requests: HashMap<AtomView<'a>, &'a Bundle>,
    validation: &'a mut Validation,
    point: &'a [f64],
}

fn real_candidate<T: CandidateNumber>(value: &T) -> Result<Rational, String> {
    value.exact_centre()
}

fn complex_candidate<T: CandidateNumber>(value: &Complex<T>) -> Result<Rational, String> {
    if value.im.exact_centre()? != 0 {
        return Err("exact contour strength has a nonzero imaginary centre".into());
    }
    value.re.exact_centre()
}

fn native<T: Real + EvaluationDomain>(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    bits: u32,
    number: impl Fn(f64) -> T,
    components: impl Fn(T) -> Vec<f64>,
    checked: Option<&mut Checked<'_>>,
    candidate: impl Fn(&T) -> Result<Rational, String>,
) -> Result<Vec<f64>, KernelError> {
    let point = point
        .iter()
        .map(|(symbol, value)| (Atom::var(*symbol), number(*value)))
        .collect::<HashMap<_, _>>();
    crate::contour::functions::dynamic::with_precision(bits, || {
        // Exact expressions remain mathematical Atoms throughout aggregation.
        // The owner's shared cache preserves native cancellations and lazy IF
        // branches; callbacks are not eagerly prebound or evaluated again.
        let values = evaluate_multiple_with_prec(expressions, &point, bits)
            .map_err(|error| KernelError::Compilation(format!("exact offset: {error}")))?;
        if let Some(checked) = checked {
            let mut executed = Vec::new();
            for (root, value) in values.function_values().filter(|(root, _)| {
                root.as_fun_view().is_some_and(|function| {
                    function.get_symbol() == crate::contour::functions::dynamic::symbol()
                })
            }) {
                let bundle = checked.requests.get(&root).ok_or_else(|| {
                    KernelError::Contour(
                        "exact offset executed an undeclared dynamic strength".into(),
                    )
                })?;
                executed.push((
                    (*bundle).clone(),
                    Candidate {
                        lambda: candidate(value).map_err(KernelError::Contour)?,
                        bits,
                    },
                ));
            }
            checked
                .validation
                .certify_executed(checked.point, &executed)
                .map_err(KernelError::Contour)?;
        }
        // Unchecked binding never iterates or copies the function-value cache.
        Ok(values
            .into_values()
            .into_iter()
            .flat_map(components)
            .collect())
    })
}

pub(super) fn evaluate(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    complex: bool,
) -> Result<Vec<f64>, KernelError> {
    evaluate_impl(expressions, point, complex, None)
}

pub(super) fn evaluate_checked(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    complex: bool,
    requests: &[ExactRequest],
    validation: &mut Validation,
    validation_point: &[f64],
) -> Result<Vec<f64>, KernelError> {
    let mut lookup = HashMap::new();
    for request in requests {
        if lookup
            .insert(request.root.as_view(), &request.bundle)
            .is_some()
        {
            return Err(KernelError::Contour(
                "duplicate saved exact strength association".into(),
            ));
        }
    }
    evaluate_impl(
        expressions,
        point,
        complex,
        Some(Checked {
            requests: lookup,
            validation,
            point: validation_point,
        }),
    )
}

fn evaluate_impl(
    expressions: &[Atom],
    point: &BTreeMap<Symbol, f64>,
    complex: bool,
    mut checked: Option<Checked<'_>>,
) -> Result<Vec<f64>, KernelError> {
    // Match the former zero-dimensional kernel's native distance policy: a
    // nonfinite final output retries the whole coefficient vector in the
    // next numeric domain. Conversion happens after native evaluation.
    let mut confirm_zero = false;
    let mut callback_failure = None;
    let mut validation_failure = None;
    for class in [
        PrecisionClass::F64,
        PrecisionClass::DoubleFloat,
        PrecisionClass::Arbitrary,
    ] {
        if confirm_zero && class == PrecisionClass::DoubleFloat {
            continue;
        }
        let bits = class.bits();
        let (values, failure) = crate::contour::functions::dynamic::isolated_attempt(
            || -> Result<Vec<f64>, KernelError> {
                Ok(match (class, complex) {
                    (PrecisionClass::F64, false) => native(
                        expressions,
                        point,
                        bits,
                        |v| v,
                        |v| vec![v],
                        checked.as_mut(),
                        real_candidate,
                    )?,
                    (PrecisionClass::F64, true) => native(
                        expressions,
                        point,
                        bits,
                        |v| Complex::new(v, 0.0),
                        |v| vec![v.re, v.im],
                        checked.as_mut(),
                        complex_candidate,
                    )?,
                    (PrecisionClass::DoubleFloat, false) => native(
                        expressions,
                        point,
                        bits,
                        DoubleFloat::from,
                        |v| vec![v.to_f64()],
                        checked.as_mut(),
                        real_candidate,
                    )?,
                    (PrecisionClass::DoubleFloat, true) => native(
                        expressions,
                        point,
                        bits,
                        |v| Complex::new(DoubleFloat::from(v), DoubleFloat::from(0.0)),
                        |v| vec![v.re.to_f64(), v.im.to_f64()],
                        checked.as_mut(),
                        complex_candidate,
                    )?,
                    (PrecisionClass::Arbitrary, false) => native(
                        expressions,
                        point,
                        bits,
                        |v| Float::with_val(bits, v),
                        |v| vec![v.to_f64()],
                        checked.as_mut(),
                        real_candidate,
                    )?,
                    (PrecisionClass::Arbitrary, true) => native(
                        expressions,
                        point,
                        bits,
                        |v| Complex::new(Float::with_val(bits, v), Float::with_val(bits, 0)),
                        |v| vec![v.re.to_f64(), v.im.to_f64()],
                        checked.as_mut(),
                        complex_candidate,
                    )?,
                    (PrecisionClass::Unstable, _) => unreachable!(),
                })
            },
        );
        if failure.is_some() {
            callback_failure = failure;
            continue;
        }
        let values = match values {
            Err(KernelError::Contour(reason)) if checked.is_some() => {
                validation_failure = Some(reason);
                continue;
            }
            result => result?,
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
    match (validation_failure, callback_failure) {
        (Some(validation), Some(callback)) => Err(KernelError::Contour(format!(
            "exact offset did not reach a certified accepted value; last certificate failure: {validation}; last callback failure: {callback}"
        ))),
        (Some(reason), None) => Err(KernelError::Contour(format!(
            "exact offset did not reach a certified accepted value: {reason}"
        ))),
        (None, Some(reason)) => Err(KernelError::Contour(format!(
            "exact offset did not reach a finite accepted value; last callback failure: {reason}"
        ))),
        (None, None) => Err(KernelError::NonFinite),
    }
}

#[cfg(test)]
mod tests;
