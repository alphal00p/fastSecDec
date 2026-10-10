//! Computational construction of the same mathematical deformation Jacobian.
mod program;
use crate::generation::GenerationError;
pub(crate) use program::image_partials;
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
    /// Evaluate contour-image first derivatives through native dual arithmetic.
    /// Endpoint reduction follows the independently selected generation mode.
    Dual,
}

impl ContourJacobian {
    pub fn is_symbolic(&self) -> bool {
        *self == Self::Symbolic
    }
}

/// Native semantic source retained until composition produces ordinary IR.
/// NumericalDual endpoints bind the complete function Atom; Symbolic endpoints
/// bind surviving image partials only. Neither route adds a Jacobian callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContourJacobianPlan {
    pub parameters: Vec<Symbol>,
    pub images: Vec<Atom>,
    pub jacobian: Atom,
}

/// Final symbolic coefficients retain the original images and actual endpoint
/// faces. Endpoint derivatives have already been taken by the native CAS.
#[derive(Clone, Debug)]
pub(crate) struct SymbolicContourJacobian {
    pub plan: std::sync::Arc<ContourJacobianPlan>,
    pub faces: Vec<Vec<(usize, u8)>>,
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
