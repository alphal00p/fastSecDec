//! Late executable callback with static diagnostic associations. Factories
//! choose checked/plain behavior once at native mapping. Plain sampling never
//! enters the observer or converts candidates to exact rationals.
use super::{candidate::CandidateNumber, numeric, observation, requests::Bundle};
use std::{cell::Cell, marker::PhantomData, rc::Rc, sync::LazyLock};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, EvalFn, EvaluationInfo, Symbol},
    domains::float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float},
    evaluate::EvaluationDomain,
    symbol,
};

thread_local! {
    static CHECKED:Cell<bool>=const {Cell::new(false)};
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Mode(bool);
impl Mode {
    pub(crate) fn checked() -> Self {
        Self(true)
    }
    pub(crate) fn capture() -> Self {
        Self(CHECKED.get())
    }
    pub(crate) fn enter(self) -> Preparation {
        Preparation(CHECKED.replace(self.0), PhantomData)
    }
}
pub(crate) struct Preparation(bool, PhantomData<Rc<()>>);
impl Drop for Preparation {
    fn drop(&mut self) {
        CHECKED.set(self.0);
    }
}

pub(crate) fn symbol() -> Symbol {
    *REQUESTED
}

fn real<T: CandidateNumber>(tags: &[AtomView<'_>]) -> EvalFn<T>
where
    Complex<T>: EvaluationDomain,
{
    // Native tag schema admission precedes factory resolution. Malformed
    // metadata still returns an explicit failing callback, never panics.
    if tags.len() != 3 {
        return Box::new(|_| {
            super::failure("requested radius requires three static tags".into());
            T::invalid(53)
        });
    }
    let plain = numeric::real::<T>(&tags[..2]);
    if !Mode::capture().0 {
        return plain;
    }
    let bundle = Bundle::from_atom(tags[2]);
    Box::new(move |arguments| {
        let value = plain(arguments);
        if !value.is_finite() {
            return value;
        }
        let outcome = bundle.as_ref().map_err(Clone::clone).and_then(|bundle| {
            observation::record(bundle, value.exact_centre()?, value.get_precision())
        });
        if let Err(error) = outcome {
            super::failure(error);
            T::invalid(value.get_precision())
        } else {
            value
        }
    })
}

fn complex<T: CandidateNumber>(tags: &[AtomView<'_>]) -> EvalFn<Complex<T>>
where
    Complex<T>: EvaluationDomain,
{
    if tags.len() != 3 {
        return Box::new(|_| {
            super::failure("requested radius requires three static tags".into());
            Complex::new(T::invalid(53), T::invalid(53))
        });
    }
    let plain = numeric::complex::<T>(&tags[..2]);
    if !Mode::capture().0 {
        return plain;
    }
    let bundle = Bundle::from_atom(tags[2]);
    Box::new(move |arguments| {
        let value = plain(arguments);
        if !value.re.is_finite() || !value.im.is_finite() {
            return value;
        }
        let outcome = bundle.as_ref().map_err(Clone::clone).and_then(|bundle| {
            observation::record(bundle, value.re.exact_centre()?, value.re.get_precision())
        });
        if let Err(error) = outcome {
            super::failure(error);
            let invalid = T::invalid(value.re.get_precision());
            Complex::new(invalid.clone(), invalid)
        } else {
            value
        }
    })
}

static REQUESTED: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::contour::dynamic::requested_strength_v1",
        der = |view, index, out| {
            let Some(arguments) = view.as_fun_view() else {
                return;
            };
            let Some(count) = arguments.get_nargs().checked_sub(5) else {
                return;
            };
            if count == 0 || index >= arguments.get_nargs() {
                return;
            }
            if index < 3 {
                **out = Atom::Zero;
                return;
            }
            let strength = view.to_owned();
            let safety = arguments.get(count + 3).to_owned();
            let cap = arguments.get(count + 4).to_owned();
            **out = if index == count + 3 {
                strength / safety
            } else if index == count + 4 {
                strength / cap
            } else {
                let radius = &strength / (safety * cap);
                let derivative: Atom = (0..count)
                    .map(|i| {
                        let power = 2 * (i as i64 + 1);
                        Atom::num(power) * arguments.get(i + 3).to_owned() * radius.pow(power)
                    })
                    .sum();
                -strength * radius.pow(2 * (index as i64 - 2)) / derivative
            };
        },
        eval = EvaluationInfo::new()
            .with_tags(3)
            .register_tagged(real::<f64>)
            .register_tagged(real::<DoubleFloat>)
            .register_tagged(real::<Float>)
            .register_tagged(real::<ErrorPropagatingFloat<f64>>)
            .register_tagged(real::<ErrorPropagatingFloat<Float>>)
            .register_tagged(complex::<f64>)
            .register_tagged(complex::<DoubleFloat>)
            .register_tagged(complex::<Float>)
            .register_tagged(complex::<ErrorPropagatingFloat<f64>>)
            .register_tagged(complex::<ErrorPropagatingFloat<Float>>)
    )
});

#[cfg(test)]
mod tests {
    use super::super::{ProgramScope, RootProgram, requests::Lookup, strength};
    use super::*;
    use symbolica::domains::rational::Rational;

    #[test]
    fn observation_mode_is_chosen_at_mapping_and_plain_callbacks_do_not_capture() {
        let helper = RootProgram::build(2).unwrap();
        let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
        let x = symbol!("requested_root_tests::x");
        let s = symbol!("requested_root_tests::s");
        let l = symbol!("requested_root_tests::l");
        let original = strength(
            &helper,
            &[Atom::one() + Atom::var(x).pow(2), Atom::var(x).pow(2)],
            &Atom::var(s),
            &Atom::var(l),
        )
        .unwrap();
        let mut lookup = Lookup::default();
        lookup
            .insert(&"a".repeat(64), &original, &[x], &[vec![]])
            .unwrap();
        let bundle = lookup.bundle(&original).unwrap();
        let requests = observation::Requests::new([bundle]).unwrap();
        let expression = lookup.lower(&original, symbol()).unwrap();
        let inputs = [Atom::var(x), Atom::var(s), Atom::var(l)];
        let exact = Atom::evaluator_multiple(
            &[
                expression.clone(),
                expression.derivative(x),
                expression.derivative(x).derivative(x),
            ],
            &inputs,
        )
        .build()
        .unwrap();
        let mut plain = exact.clone().map_coeff(&|c| c.re.to_f64());
        let mut checked = {
            let _mode = Mode::checked().enter();
            exact.clone().map_coeff(&|c| c.re.to_f64())
        };
        let mut plain_output = [0.; 3];
        let mut checked_output = [0.; 3];
        let attempt = requests.begin();
        plain.evaluate(&[0.31, 0.8, 1.], &mut plain_output);
        assert!(attempt.finish().unwrap_err().contains("not evaluated"));
        let attempt = requests.begin();
        checked.evaluate(&[0.31, 0.8, 1.], &mut checked_output);
        let observations = attempt.finish().unwrap();
        assert_eq!(plain_output, checked_output);
        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].lambda,
            Rational::try_from(plain_output[0]).unwrap()
        );
        // Same parsed program, new numeric domain, and a cloned checked owner.
        let mut complex = {
            let _mode = Mode::checked().enter();
            exact
                .clone()
                .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
        };
        let attempt = requests.begin();
        let mut output = [Complex::new(0., 0.); 3];
        complex.evaluate(
            &[
                Complex::new(0.31, 0.),
                Complex::new(0.8, 0.),
                Complex::new(1., 0.),
            ],
            &mut output,
        );
        assert_eq!(attempt.finish().unwrap().len(), 1);
        assert_eq!(output[0].re, plain_output[0]);
        let mut moved = checked.clone();
        std::thread::spawn(move || {
            let attempt = requests.begin();
            moved.evaluate(&[0.31, 0.8, 1.], &mut [0.; 3]);
            assert_eq!(attempt.finish().unwrap().len(), 1);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn checked_preparation_restores_nested_modes_after_unwind() {
        assert!(!Mode::capture().0);
        let outer = Mode::checked().enter();
        assert!(Mode::capture().0);
        assert!(
            std::panic::catch_unwind(|| {
                let _inner = Mode::default().enter();
                assert!(!Mode::capture().0);
                panic!("injected");
            })
            .is_err()
        );
        assert!(Mode::capture().0);
        drop(outer);
        assert!(!Mode::capture().0);
    }
}
