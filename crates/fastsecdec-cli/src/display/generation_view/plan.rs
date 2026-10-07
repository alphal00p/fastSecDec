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
        }
    }
}
impl Plan {
    pub(in crate::display) fn configure(
        &mut self,
        mode: GenerationMode,
        method: CoefficientExpansionMethod,
    ) {
        *self = Self {
            mode: Some(mode),
            method,
            ..Self::default()
        };
    }
    pub(in crate::display) fn observe(&mut self, stage: GenerationStage) {
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
                self.complete = true;
                7
            }
        };
        // Nested subtraction/epsilon passes and fallback events never undo a
        // completed parent phase. Counts/timing updates alone do not finish it.
        if stage != GenerationStage::Complete {
            self.seen[index] = true;
        }
        self.current = self.current.max(index);
    }
    pub(in crate::display) fn formula_count(&mut self, total: usize) {
        self.empty_formulas = self.mode == Some(GenerationMode::NumericalDual) && total == 0;
    }
    pub(in crate::display) fn saving(&mut self) {
        self.current = 7;
        self.seen[7] = true;
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
