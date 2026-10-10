//! Prepared native radius callbacks. No sampling loop or production RNG lives here.
mod candidate;
pub(crate) mod diagnostics;
pub(crate) use candidate::CandidateNumber;
pub(crate) mod numeric;
pub(crate) mod observation;
mod preparation;
mod program;
pub(crate) mod requested;
pub(crate) mod requests;

pub(crate) use preparation::with_precision;
pub(crate) use program::{ProgramPreparation, ProgramScope, RootProgram};
use std::{cell::RefCell, sync::LazyLock};
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo, Symbol},
    domains::float::{DoubleFloat, ErrorPropagatingFloat, Float},
    symbol,
};

thread_local! {
    // Bounded failure information for the current caller-owned evaluation.
    static LAST_FAILURE: RefCell<Option<String>> = const { RefCell::new(None) };
}
pub(crate) fn failure(error: String) {
    LAST_FAILURE.with_borrow_mut(|failure| {
        if failure.is_none() {
            *failure = Some(error);
        }
    });
}
pub(crate) fn take_failure() -> Option<String> {
    LAST_FAILURE.take()
}

/// One caller-owned precision attempt must neither inherit an earlier failure
/// nor overwrite a surrounding evaluation's diagnostics. The guard also
/// restores the outer state if a native/user callback unwinds.
pub(crate) fn isolated_attempt<R>(evaluate: impl FnOnce() -> R) -> (R, Option<String>) {
    struct Restore(Option<String>);
    impl Drop for Restore {
        fn drop(&mut self) {
            LAST_FAILURE.replace(self.0.take());
        }
    }
    let _restore = Restore(LAST_FAILURE.take());
    let value = evaluate();
    (value, LAST_FAILURE.take())
}

pub(crate) fn register() {
    let _ = *STRENGTH;
    let _ = requested::symbol();
}

pub(crate) fn strength(
    helper: &RootProgram,
    coefficients: &[Atom],
    safety: &Atom,
    cap: &Atom,
) -> Result<Atom, String> {
    if coefficients.len() != helper.coefficient_count() {
        return Err("dynamic coefficient vector differs from its saved helper schema".into());
    }
    let arguments = [Atom::num(coefficients.len()), Atom::var(helper.tag())]
        .into_iter()
        .chain(coefficients.iter().cloned())
        .chain([safety.clone(), cap.clone()]);
    Ok(STRENGTH.call_args(arguments))
}

pub(crate) fn symbol() -> Symbol {
    *STRENGTH
}

static STRENGTH: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::contour::dynamic::strength_v1",
        der = |view, index, out| {
            let Some(arguments) = view.as_fun_view() else {
                return;
            };
            let Some(coefficient_count) = arguments.get_nargs().checked_sub(4) else {
                return;
            };
            if coefficient_count == 0 || index >= arguments.get_nargs() {
                return;
            }
            if index < 2 {
                **out = Atom::Zero;
                return;
            }
            let strength = view.to_owned();
            let safety = arguments.get(coefficient_count + 2).to_owned();
            let cap = arguments.get(coefficient_count + 3).to_owned();
            **out = if index == coefficient_count + 2 {
                strength / safety
            } else if index == coefficient_count + 3 {
                strength / cap
            } else {
                let radius = &strength / (safety * cap);
                let derivative: Atom = (0..coefficient_count)
                    .map(|i| {
                        let power = 2 * (i as i64 + 1);
                        Atom::num(power) * arguments.get(i + 2).to_owned() * radius.pow(power)
                    })
                    .sum();
                -strength * radius.pow(2 * (index as i64 - 1)) / derivative
            };
        },
        eval = EvaluationInfo::new()
            .with_tags(2)
            .register_tagged(numeric::real::<f64>)
            .register_tagged(numeric::real::<DoubleFloat>)
            .register_tagged(numeric::real::<Float>)
            .register_tagged(numeric::real::<ErrorPropagatingFloat<f64>>)
            .register_tagged(numeric::real::<ErrorPropagatingFloat<Float>>)
            .register_tagged(numeric::complex::<f64>)
            .register_tagged(numeric::complex::<DoubleFloat>)
            .register_tagged(numeric::complex::<Float>)
            .register_tagged(numeric::complex::<ErrorPropagatingFloat<f64>>)
            .register_tagged(numeric::complex::<ErrorPropagatingFloat<Float>>)
    )
});

#[cfg(test)]
mod tests;
