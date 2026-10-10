//! Computational construction of the same mathematical deformation Jacobian.
use crate::generation::GenerationError;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
    symbol,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContourJacobian {
    /// Construct the native symbolic Jacobian before evaluator lowering.
    #[default]
    Symbolic,
    /// Compose native image derivatives and determinant instructions before
    /// the complete smooth density receives its subtraction jets.
    Dual,
}

impl ContourJacobian {
    pub fn is_symbolic(&self) -> bool {
        *self == Self::Symbolic
    }
}

/// Native semantic source retained until composition produces ordinary IR.
/// The complete function Atom is an evaluator input, never a numeric callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContourJacobianPlan {
    pub parameters: Vec<Symbol>,
    pub images: Vec<Atom>,
    pub jacobian: Atom,
}

#[derive(Clone, Debug)]
pub(crate) struct JacobianTemplate {
    pub entries: Vec<Symbol>,
    pub expression: Atom,
}

impl JacobianTemplate {
    pub fn new(dimension: usize) -> Result<Self, GenerationError> {
        if !(1..=6).contains(&dimension) {
            return Err(GenerationError::Contour(
                "dual contour Jacobians currently require one through six coordinates; use symbolic for larger charts".into(),
            ));
        }
        let entries = (0..dimension * dimension)
            .map(|index| {
                symbol!(format!(
                    "fastsecdec::contour::jacobian_entry_{dimension}_{index}"
                ))
            })
            .collect::<Vec<_>>();
        let expression = super::determinant::determinant(
            entries.iter().copied().map(Atom::var).collect(),
            dimension as u32,
        )?;
        Ok(Self {
            entries,
            expression,
        })
    }

    pub fn substitute(&self, parameters: &[Symbol], images: &[Atom]) -> Atom {
        let derivatives = images
            .iter()
            .flat_map(|image| parameters.iter().map(|p| image.derivative(*p)));
        self.expression
            .replace_multiple(self.entries.iter().zip(derivatives).map(|(from, to)| {
                Replacement::new(Pattern::Literal(Atom::var(*from)), Pattern::Literal(to))
            }))
    }
}
