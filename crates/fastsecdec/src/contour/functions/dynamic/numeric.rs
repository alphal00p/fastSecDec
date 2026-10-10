//! Owner-prepared scalar roots, with native complex uncertainty propagation.
use super::{
    preparation,
    program::{self, Program},
};
use std::sync::{Arc, Mutex};
use symbolica::{
    atom::{AtomView, EvalFn},
    domains::float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, Real, RealLike},
    evaluate::{EvaluationDomain, ExpressionEvaluator},
    solve::{BracketedRootConvergence, BracketedRootOptions, nsolve_bracketed},
};

pub(crate) trait Number:
    Real + RealLike + PartialOrd + EvaluationDomain + Send + Sync
{
    fn one_at(bits: u32) -> Self;
    fn invalid(bits: u32) -> Self;
}
impl Number for f64 {
    fn one_at(_: u32) -> Self {
        1.
    }
    fn invalid(_: u32) -> Self {
        f64::NAN
    }
}
impl Number for DoubleFloat {
    fn one_at(_: u32) -> Self {
        Self::from(1.)
    }
    fn invalid(_: u32) -> Self {
        Self::from(f64::NAN)
    }
}
impl Number for Float {
    fn one_at(bits: u32) -> Self {
        Self::with_val(bits, 1)
    }
    fn invalid(bits: u32) -> Self {
        Self::with_val(bits, f64::NAN)
    }
}
impl<T: Number> Number for ErrorPropagatingFloat<T>
where
    Self: EvaluationDomain,
{
    fn one_at(bits: u32) -> Self {
        Self::new_with_accuracy(T::one_at(bits), f64::INFINITY)
    }
    fn invalid(bits: u32) -> Self {
        Self::new_with_accuracy(T::invalid(bits), f64::INFINITY)
    }
}

/// Each callback clone owns independent mutable scratch. Only immutable saved
/// helper data is shared; no global lock is taken while sampling.
struct Workspace<T> {
    helper: Arc<Program>,
    scratch: Mutex<Scratch<T>>,
    bits: u32,
    poisoned: bool,
}
#[derive(Clone)]
struct Scratch<T> {
    evaluator: ExpressionEvaluator<Complex<T>>,
    input: Vec<Complex<T>>,
    output: [Complex<T>; 2],
    options: BracketedRootOptions<T>,
}
impl<T: Clone> Clone for Workspace<T> {
    fn clone(&self) -> Self {
        let (scratch, poisoned) = match self.scratch.lock() {
            Ok(scratch) => (Mutex::new(scratch.clone()), self.poisoned),
            Err(poisoned) => (Mutex::new(poisoned.into_inner().clone()), true),
        };
        Self {
            helper: self.helper.clone(),
            scratch,
            bits: self.bits,
            poisoned,
        }
    }
}
impl<T: Number> Workspace<T>
where
    Complex<T>: EvaluationDomain,
{
    fn new(tags: &[AtomView<'_>], bits: u32) -> Result<Self, String> {
        let helper = program::resolve(tags)?;
        let one = T::one_at(bits);
        let zero = one.zero();
        let evaluator = helper.exact.clone().map_coeff_with_prec(
            &|c| Complex::new(one.from_rational(&c.re), one.from_rational(&c.im)),
            bits,
        );
        Ok(Self {
            bits,
            helper: helper.clone(),
            poisoned: false,
            scratch: Mutex::new(Scratch {
                evaluator,
                input: vec![Complex::new(zero.clone(), zero.clone()); helper.coefficient_count + 1],
                output: [
                    Complex::new(zero.clone(), zero.clone()),
                    Complex::new(zero.clone(), zero.clone()),
                ],
                options: BracketedRootOptions {
                    absolute_tolerance: zero,
                    relative_tolerance: one
                        .from_usize(2)
                        .pow(u64::from(bits.saturating_sub(5)))
                        .inv(),
                    // A finite work allowance, not a universal convergence
                    // bound. Strict bracket refinement must also accommodate
                    // the caller's high-precision domains (up to thousands of
                    // bits), rather than retain an f64-sized iteration cap.
                    max_iterations: (bits as usize).saturating_mul(4).max(256),
                    initial_guess: None,
                    convergence: BracketedRootConvergence::Bracket,
                },
            }),
        })
    }

    fn evaluate<const OBSERVE: bool>(
        &self,
        count: usize,
        argument: impl Fn(usize) -> Complex<T>,
        mut event: Option<&mut super::diagnostics::Event>,
    ) -> Result<Complex<T>, String> {
        if self.poisoned {
            return Err("dynamic root workspace poisoned".into());
        }
        let expected = self.helper.coefficient_count + 2;
        if count != expected {
            return Err(
                "dynamic root callback argument count differs from its saved schema".into(),
            );
        }
        let mut scratch = self
            .scratch
            .lock()
            .map_err(|_| "dynamic root workspace poisoned")?;
        let one = T::one_at(self.bits);
        let zero = one.zero();
        let mut scalar = None;
        let mut cap = None;
        for index in 0..count {
            let value = argument(index);
            if !value.re.is_finite() || !value.im.is_finite() || !value.im.is_zero() {
                return Err("dynamic root inputs must have finite real centres".into());
            }
            if index < self.helper.coefficient_count {
                if value.re < zero || (index == 0 && value.re < one) {
                    return Err(
                        "dynamic envelope coefficients violate nonnegativity or a2 >= 1".into(),
                    );
                }
                scratch.input[index + 1] = value;
            } else if index == self.helper.coefficient_count {
                if value.re <= zero || value.re >= one {
                    return Err(
                        "dynamic safety fraction must be strictly between zero and one".into(),
                    );
                }
                scalar = Some(value);
            } else {
                if value.re <= zero {
                    return Err("dynamic strength cap must be positive".into());
                }
                cap = Some(value);
            }
        }
        // This initial guess is an upper bound in exact arithmetic because
        // a2 >= 1 and all remaining coefficients are nonnegative.
        scratch.options.initial_guess = Some(one.clone() / scratch.input[1].re.sqrt());
        let Scratch {
            evaluator,
            input,
            output,
            options,
        } = &mut *scratch;
        let root = if self.helper.coefficient_count == 1 {
            // H(u)=a2*u^2 has the positive solution 1/sqrt(a2). Reuse
            // the native operation already used for the initial guess,
            // including its tracked real uncertainty. The native complex
            // correction below still supplies imaginary uncertainty.
            if let Some(event) = event.as_deref_mut() {
                event.closed_form = true;
            }
            options.initial_guess.as_ref().unwrap().clone()
        } else {
            if let Some(event) = event.as_deref_mut() {
                event.solver = true;
            }
            let answer = nsolve_bracketed(zero.clone(), one.clone(), options, |coordinate| {
                input[0] = Complex::new(coordinate.clone(), coordinate.zero());
                if evaluator.try_evaluate(input, output).is_err() {
                    return (T::invalid(self.bits), T::invalid(self.bits));
                }
                (output[0].re.clone(), output[1].re.clone())
            });
            let answer = answer.map_err(|error| {
                format!(
                "dynamic contour radius solver: {error}; domain {}, precision {}, rounded f64 centres: coefficients {:?}, safety {:?}, cap {:?}, initial {:?}, relative tolerance {:?}, iteration limit {}, last point {}, last value/derivative {:?}",
                std::any::type_name::<T>(),
                self.bits,
                input[1..].iter().map(|value| value.re.to_f64()).collect::<Vec<_>>(),
                scalar.as_ref().map(|value| value.re.to_f64()),
                cap.as_ref().map(|value| value.re.to_f64()),
                options.initial_guess.as_ref().map(RealLike::to_f64),
                options.relative_tolerance.to_f64(),
                options.max_iterations,
                input[0].re.to_f64(),
                output.iter().map(|value| value.re.to_f64()).collect::<Vec<_>>(),
                )
            })?;
            if let Some(event) = event.as_deref_mut() {
                event.solver_success = true;
                event.iterations = answer.iterations as u64;
                event.evaluations = answer.evaluations as u64;
                event.termination = Some(answer.termination);
            }
            answer.root
        };
        if !root.is_finite() || root <= zero || root > one {
            return Err(
                "dynamic contour radius is nonpositive, nonfinite, or exceeds its cap".into(),
            );
        }
        // The physical coefficient centres are real, but tracked zero-centred
        // imaginary parts can carry uncertainty. Native complex Newton
        // arithmetic retains that uncertainty without a second AD system.
        input[0] = Complex::new(root.clone(), zero.clone());
        if let Some(event) = event.as_deref_mut() {
            event.correction = true;
        }
        evaluator
            .try_evaluate(input, output)
            .map_err(|error| error.to_string())?;
        let correction = output[0].clone() / &output[1];
        let radius = Complex::new(root, -correction.im);
        let scalar = scalar.ok_or("missing dynamic safety fraction")?;
        let cap = cap.ok_or("missing dynamic strength cap")?;
        let display = if OBSERVE {
            Some((input[1].re.to_f64(), cap.re.to_f64()))
        } else {
            None
        };
        let strength = (scalar * cap) * radius;
        if !strength.re.is_finite()
            || !strength.im.is_finite()
            || strength.re <= zero
            || !strength.im.is_zero()
        {
            return Err(
                "dynamic contour strength is nonpositive, underflowed, nonreal, or nonfinite"
                    .into(),
            );
        }
        if let (Some((a2, cap)), Some(event)) = (display, event) {
            let lambda = strength.re.to_f64();
            event.strength = Some(lambda);
            event.normalized_displacement = (a2.is_finite()
                && a2 > 1.
                && cap.is_finite()
                && cap > 0.
                && lambda.is_finite()
                && lambda > 0.)
                .then(|| (lambda / cap) * (a2 - 1.).sqrt());
        }
        Ok(strength)
    }
}

pub(super) fn real<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<T>
where
    Complex<T>: EvaluationDomain,
{
    if super::diagnostics::Configuration::capture().enabled() {
        real_factory::<T, true>(tags)
    } else {
        real_factory::<T, false>(tags)
    }
}
fn real_factory<T: Number, const OBSERVE: bool>(tags: &[AtomView<'_>]) -> EvalFn<T>
where
    Complex<T>: EvaluationDomain,
{
    let bits = T::FIXED_PRECISION.or_else(preparation::precision);
    let workspace = bits
        .ok_or_else(|| "dynamic Float callback requires a scoped mapping precision".to_string())
        .and_then(|bits| Workspace::<T>::new(tags, bits));
    Box::new(move |arguments: &[T]| {
        let mut event = OBSERVE.then(|| super::diagnostics::Event {
            bits: bits.unwrap_or(53),
            ..Default::default()
        });
        let result = match &workspace {
            Ok(workspace) => workspace
                .evaluate::<OBSERVE>(
                    arguments.len(),
                    |i| Complex::new(arguments[i].clone(), arguments[i].zero()),
                    event.as_mut(),
                )
                .map(|value| value.re),
            Err(error) => Err(error.clone()),
        };
        if let Some(mut event) = event {
            event.failed = result.is_err();
            super::diagnostics::record(event);
        }
        result.unwrap_or_else(|error| {
            super::failure(error);
            T::invalid(bits.unwrap_or(53))
        })
    })
}
pub(super) fn complex<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<Complex<T>>
where
    Complex<T>: EvaluationDomain,
{
    if super::diagnostics::Configuration::capture().enabled() {
        complex_factory::<T, true>(tags)
    } else {
        complex_factory::<T, false>(tags)
    }
}
fn complex_factory<T: Number, const OBSERVE: bool>(tags: &[AtomView<'_>]) -> EvalFn<Complex<T>>
where
    Complex<T>: EvaluationDomain,
{
    let bits = T::FIXED_PRECISION.or_else(preparation::precision);
    let workspace = bits
        .ok_or_else(|| "dynamic Float callback requires a scoped mapping precision".to_string())
        .and_then(|bits| Workspace::<T>::new(tags, bits));
    Box::new(move |arguments: &[Complex<T>]| {
        let mut event = OBSERVE.then(|| super::diagnostics::Event {
            bits: bits.unwrap_or(53),
            ..Default::default()
        });
        let result = match &workspace {
            Ok(workspace) => workspace.evaluate::<OBSERVE>(
                arguments.len(),
                |i| arguments[i].clone(),
                event.as_mut(),
            ),
            Err(error) => Err(error.clone()),
        };
        if let Some(mut event) = event {
            event.failed = result.is_err();
            super::diagnostics::record(event);
        }
        result.unwrap_or_else(|error| {
            super::failure(error);
            Complex::new(
                T::invalid(bits.unwrap_or(53)),
                T::invalid(bits.unwrap_or(53)),
            )
        })
    })
}
