//! Factory-selected bounded callback observations. Disabled callbacks compile
//! out event construction/recording; disabled owners never enter this TLS scope.
use crate::{
    contour::{ContourDiagnosticsMode, ContourRuntimeReport, ContourRuntimeWork},
    status::DiagnosticsOverflow,
};
use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Configuration {
    pub mode: ContourDiagnosticsMode,
    pub displacement_cap: f64,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            mode: ContourDiagnosticsMode::Disabled,
            displacement_cap: 1.,
        }
    }
}
thread_local! {
    static MAPPING: Cell<Configuration> = Cell::new(Configuration::default());
    static ATTEMPT: RefCell<Option<State>> = const {RefCell::new(None)};
}
impl Configuration {
    pub(crate) fn capture() -> Self {
        MAPPING.get()
    }
    pub(crate) fn enabled(self) -> bool {
        self.mode == ContourDiagnosticsMode::Aggregate
    }
    pub(crate) fn enter(self) -> Preparation {
        Preparation(MAPPING.replace(self), PhantomData)
    }
}
pub(crate) struct Preparation(Configuration, PhantomData<Rc<()>>);
impl Drop for Preparation {
    fn drop(&mut self) {
        MAPPING.set(self.0);
    }
}

#[derive(Default)]
pub(crate) struct Event {
    pub bits: u32,
    pub failed: bool,
    pub solver: bool,
    pub solver_success: bool,
    pub closed_form: bool,
    pub iterations: u64,
    pub evaluations: u64,
    pub correction: bool,
    pub termination: Option<symbolica::solve::BracketedRootTermination>,
    pub strength: Option<f64>,
    pub normalized_displacement: Option<f64>,
}
struct State {
    configuration: Configuration,
    work: ContourRuntimeWork,
    overflow: bool,
}
impl State {
    fn record(&mut self, event: Event) {
        let mut work = ContourRuntimeWork {
            callback_calls: 1,
            callback_failures: u64::from(event.failed),
            solver_calls: u64::from(event.solver),
            solver_successes: u64::from(event.solver_success),
            solver_failures: u64::from(event.solver && !event.solver_success),
            closed_form_calls: u64::from(event.closed_form),
            solver_iterations: event.iterations,
            solver_evaluations: event.evaluations,
            maximum_solver_iterations: event.iterations,
            maximum_solver_evaluations: event.evaluations,
            solver_numerical_zero: u64::from(matches!(
                event.termination,
                Some(symbolica::solve::BracketedRootTermination::NumericalZero)
            )),
            solver_bracket_width: u64::from(matches!(
                event.termination,
                Some(symbolica::solve::BracketedRootTermination::BracketWidth)
            )),
            solver_newton_correction: u64::from(matches!(
                event.termination,
                Some(symbolica::solve::BracketedRootTermination::NewtonCorrection)
            )),
            correction_evaluations: u64::from(event.correction),
            maximum_bits: event.bits,
            ..Default::default()
        };
        if !event.failed {
            // A rounded a2=1 contributes unavailable displacement, never zero.
            let _ = work.strength.record(event.strength);
            let _ = work
                .normalized_displacement
                .record(event.normalized_displacement);
            let _ = work.physical_displacement.record(
                event
                    .normalized_displacement
                    .map(|v| v * self.configuration.displacement_cap),
            );
        }
        self.overflow |= self.work.merge(&work).is_err();
    }
}
pub(crate) fn record(event: Event) {
    ATTEMPT.with_borrow_mut(|state| {
        if let Some(state) = state {
            state.record(event);
        }
    });
}
struct Attempt {
    outer: Option<State>,
    marker: PhantomData<Rc<()>>,
}
impl Attempt {
    fn begin(configuration: Configuration) -> Self {
        Self {
            outer: ATTEMPT.replace(Some(State {
                configuration,
                work: Default::default(),
                overflow: false,
            })),
            marker: PhantomData,
        }
    }
    fn finish(self) -> State {
        ATTEMPT.take().expect("active diagnostics attempt")
    }
}
impl Drop for Attempt {
    fn drop(&mut self) {
        ATTEMPT.replace(self.outer.take());
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Evaluation,
    Conditioning,
    Preparation,
    Exact,
    Pilot,
}

/// A numerical owner's bounded accumulator. Clone starts independent history.
#[derive(Default)]
pub(crate) struct Accumulator {
    report: ContourRuntimeReport,
    observed: bool,
    overflow: bool,
}
impl Clone for Accumulator {
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl Accumulator {
    pub(crate) fn measure<R>(
        &mut self,
        configuration: Configuration,
        phase: Phase,
        evaluate: impl FnOnce() -> R,
    ) -> R {
        if !configuration.enabled() {
            return evaluate();
        }
        let attempt = Attempt::begin(configuration);
        let result = evaluate();
        let state = attempt.finish();
        self.observed = true;
        self.overflow |= state.overflow;
        let work = match phase {
            Phase::Evaluation => &mut self.report.evaluation,
            Phase::Conditioning => &mut self.report.conditioning,
            Phase::Preparation => &mut self.report.preparation,
            Phase::Exact => &mut self.report.exact,
            Phase::Pilot => &mut self.report.pilot,
        };
        self.overflow |= work.merge(&state.work).is_err();
        result
    }
    pub(crate) fn snapshot(&self) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        if self.overflow {
            Err(DiagnosticsOverflow)
        } else {
            Ok(self.observed.then(|| self.report.clone()))
        }
    }
    pub(crate) fn absorb(&mut self, other: &Self) {
        self.observed |= other.observed;
        self.overflow |= other.overflow;
        self.overflow |= self.report.merge(&other.report).is_err();
    }
    pub(crate) fn absorb_as_pilot(&mut self, other: &Self) {
        self.observed |= other.observed;
        self.overflow |= other.overflow;
        for work in [
            &other.report.evaluation,
            &other.report.conditioning,
            &other.report.preparation,
            &other.report.exact,
            &other.report.pilot,
        ] {
            self.overflow |= self.report.pilot.merge(work).is_err();
        }
    }
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
    pub(crate) fn take(&mut self) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        let result = self.snapshot()?;
        self.clear();
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
