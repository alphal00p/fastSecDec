//! CLI delivery phases, grouping the native per-sector work that can repeat.
use fastsecdec::{
    generation::{CoefficientExpansionMethod, GenerationMode},
    status::GenerationStage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum State {
    Complete,
    Current,
    Pending,
    NotNeeded,
}

pub(in crate::display) struct Plan {
    mode: Option<GenerationMode>,
    method: CoefficientExpansionMethod,
    current: usize,
    complete: bool,
    seen: [bool; 8],
    empty_formulas: bool,
    started: f64,
    elapsed: f64,
    durations: [Option<f64>; 8],
}
impl Default for Plan {
    fn default() -> Self {
        Self {
            mode: None,
            method: CoefficientExpansionMethod::NativeNamed,
            current: 0,
            complete: false,
            seen: [true, false, false, false, false, false, false, false],
            empty_formulas: false,
            started: 0.0,
            elapsed: 0.0,
            durations: [None; 8],
        }
    }
}
impl Plan {
    pub(in crate::display) fn configure(
        &mut self,
        mode: GenerationMode,
        method: CoefficientExpansionMethod,
    ) {
        // Parsing reveals the plan while the input phase is already running.
        // Keep its start and elapsed time rather than restarting the clock.
        self.mode = Some(mode);
        self.method = method;
    }
    pub(in crate::display) fn observe(&mut self, stage: GenerationStage, elapsed: f64) {
        if self.complete && stage == GenerationStage::Input {
            *self = Self::default();
        }
        self.advance_clock(elapsed);
        if self.complete {
            return;
        }
        let index = match stage {
            GenerationStage::Input => 0,
            GenerationStage::Parametrization => 1,
            GenerationStage::Geometry => 2,
            GenerationStage::Mapping => 3,
            GenerationStage::Symmetry | GenerationStage::FormulaPreparation => 4,
            GenerationStage::Subtraction
            | GenerationStage::Expansion
            | GenerationStage::CoefficientExpansion => 5,
            GenerationStage::Compilation => 6,
            GenerationStage::Complete => {
                self.finish_current();
                self.complete = true;
                self.current = 7;
                return;
            }
        };
        // Nested subtraction/epsilon passes and fallback events never undo a
        // completed parent phase. Counts/timing updates alone do not finish it.
        self.enter(index);
    }
    pub(in crate::display) fn formula_count(&mut self, total: usize) {
        self.empty_formulas = self.mode == Some(GenerationMode::NumericalDual) && total == 0;
    }
    pub(in crate::display) fn saving(&mut self, elapsed: f64) {
        self.advance_clock(elapsed);
        if !self.complete {
            self.enter(7);
        }
    }
    fn advance_clock(&mut self, elapsed: f64) {
        if elapsed.is_finite() {
            self.elapsed = self.elapsed.max(elapsed);
        }
    }
    fn enter(&mut self, index: usize) {
        if index > self.current {
            self.finish_current();
            self.current = index;
            self.started = self.elapsed;
            self.seen[index] = true;
        }
    }
    fn finish_current(&mut self) {
        if self.state(self.current) == State::Current {
            self.durations[self.current] = Some(self.elapsed - self.started);
        }
    }
    pub(super) fn seconds(&self, index: usize, elapsed: f64) -> Option<f64> {
        match self.state(index) {
            State::Current => Some(self.elapsed.max(elapsed) - self.started),
            State::Complete => self.durations[index],
            State::Pending | State::NotNeeded => None,
        }
    }
    pub(super) fn labels(&self) -> Vec<&'static str> {
        let Some(mode) = self.mode else {
            return vec!["Read run card · loading plan"];
        };
        let (preparation, coefficients) = match mode {
            GenerationMode::NumericalDual => (
                "Prepare subtraction formulas",
                "Assemble sector coefficients",
            ),
            GenerationMode::Symbolic => (
                "Find equivalent sectors",
                match self.method {
                    CoefficientExpansionMethod::NativeNamed => "Expand coefficient series",
                    CoefficientExpansionMethod::Physical => "Subtract + expand expressions",
                },
            ),
        };
        vec![
            "Read input",
            "Prepare integral",
            "Build sector geometry",
            "Map sectors",
            preparation,
            coefficients,
            "Compile evaluators",
            "Save artifact",
        ]
    }
    pub(super) fn state(&self, index: usize) -> State {
        if index == 4 && self.empty_formulas {
            return State::NotNeeded;
        }
        if self.complete || index < self.current {
            if self.seen[index] {
                State::Complete
            } else {
                State::NotNeeded
            }
        } else if index == self.current {
            State::Current
        } else {
            State::Pending
        }
    }
    pub(super) fn active_label(&self) -> &'static str {
        if self.complete {
            "Complete"
        } else {
            self.labels()
                .get(self.current)
                .copied()
                .unwrap_or("Reading run card")
        }
    }

    pub(super) fn description(&self) -> &'static str {
        match self.mode {
            None => "Generation steps",
            Some(GenerationMode::NumericalDual) => "Steps · numerical dual",
            Some(GenerationMode::Symbolic) => "Steps · symbolic",
        }
    }
}
