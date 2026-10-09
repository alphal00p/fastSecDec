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
                    max_iterations: 256,
                    initial_guess: None,
                    convergence: BracketedRootConvergence::Bracket,
                },
            }),
        })
    }

    fn evaluate(
        &self,
        count: usize,
        argument: impl Fn(usize) -> Complex<T>,
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
        let answer = nsolve_bracketed(zero.clone(), one.clone(), options, |coordinate| {
            input[0] = Complex::new(coordinate.clone(), coordinate.zero());
            if evaluator.try_evaluate(input, output).is_err() {
                return (T::invalid(self.bits), T::invalid(self.bits));
            }
            (output[0].re.clone(), output[1].re.clone())
        })
        .map_err(|error| format!("dynamic contour radius solver: {error}"))?;
        if !answer.root.is_finite() || answer.root <= zero || answer.root > one {
            return Err(
                "dynamic contour radius is nonpositive, nonfinite, or exceeds its cap".into(),
            );
        }
        // The physical coefficient centres are real, but tracked zero-centred
        // imaginary parts can carry uncertainty. Native complex Newton
        // arithmetic retains that uncertainty without a second AD system.
        input[0] = Complex::new(answer.root.clone(), zero.clone());
        evaluator
            .try_evaluate(input, output)
            .map_err(|error| error.to_string())?;
        let correction = output[0].clone() / &output[1];
        let radius = Complex::new(answer.root, -correction.im);
        let scalar = scalar.ok_or("missing dynamic safety fraction")?;
        let cap = cap.ok_or("missing dynamic strength cap")?;
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
        Ok(strength)
    }
}

pub(super) fn real<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<T>
where
    Complex<T>: EvaluationDomain,
{
    let bits = T::FIXED_PRECISION.or_else(preparation::precision);
    let workspace = bits
        .ok_or_else(|| "dynamic Float callback requires a scoped mapping precision".to_string())
        .and_then(|bits| Workspace::<T>::new(tags, bits));
    Box::new(move |arguments: &[T]| match &workspace {
        Ok(workspace) => workspace
            .evaluate(arguments.len(), |i| {
                Complex::new(arguments[i].clone(), arguments[i].zero())
            })
            .map(|value| value.re)
            .unwrap_or_else(|error| {
                super::failure(error);
                T::invalid(workspace.bits)
            }),
        Err(error) => {
            super::failure(error.clone());
            T::invalid(bits.unwrap_or(53))
        }
    })
}
pub(super) fn complex<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<Complex<T>>
where
    Complex<T>: EvaluationDomain,
{
    let bits = T::FIXED_PRECISION.or_else(preparation::precision);
    let workspace = bits
        .ok_or_else(|| "dynamic Float callback requires a scoped mapping precision".to_string())
        .and_then(|bits| Workspace::<T>::new(tags, bits));
    Box::new(move |arguments: &[Complex<T>]| match &workspace {
        Ok(workspace) => workspace
            .evaluate(arguments.len(), |i| arguments[i].clone())
            .unwrap_or_else(|error| {
                super::failure(error);
                Complex::new(T::invalid(workspace.bits), T::invalid(workspace.bits))
            }),
        Err(error) => {
            super::failure(error.clone());
            Complex::new(
                T::invalid(bits.unwrap_or(53)),
                T::invalid(bits.unwrap_or(53)),
            )
        }
    })
}
