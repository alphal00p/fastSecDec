//! Errors retain native causes at the graph and kinematics boundary.

use feynkit_graph::{DiagramError, EdgeId, IntegralFamilyError};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parametric(#[from] crate::parametric::ParametricError),
    #[error(transparent)]
    Model(#[from] feynkit_model::ModelError),
    #[error(transparent)]
    Diagram(#[from] DiagramError),
    #[error(transparent)]
    IntegralFamily(#[from] IntegralFamilyError),
    #[error("edge {0:?} is not an internal physical propagator")]
    UnknownPropagator(EdgeId),
    #[error("propagator {edge:?} requires a positive integer power, got {power}")]
    InvalidPower { edge: EdgeId, power: u32 },
    #[error(
        "edge {edge:?} has a nonzero or unresolved width ({parameter}); phase one requires zero widths"
    )]
    UnsupportedWidth { edge: EdgeId, parameter: String },
    #[error("edge {edge:?} has a non-real or non-finite mass ({parameter})")]
    UnsupportedMass { edge: EdgeId, parameter: String },
    #[error(
        "edge {edge:?} uses custom propagator denominator {propagator}; phase one accepts only the native quadratic q^2-m^2 denominator"
    )]
    UnsupportedDenominator { edge: EdgeId, propagator: String },
    #[error("tensor numerator normalization failed: {0}")]
    Tensor(String),
    #[error("the numerator retains free tensor indices; supply a scalar projector")]
    FreeTensorIndices,
    #[error("a loop-dependent numerator requires numerator parametrization")]
    LoopNumerator,
    #[error("the contracted numerator must be polynomial in native loop scalar products")]
    NonPolynomialNumerator,
    #[error(
        "Gamma({0}) is singular at the requested exact dimension; retain a symbolic dimension and take its Laurent limit after numerator cancellations"
    )]
    UnregulatedGammaPole(symbolica::atom::Atom),
    #[error("the quadratic loop form is singular (U = 0)")]
    SingularLoopForm,
    #[error(
        "a concrete tensor dimension cannot be changed after contraction; construct Kinematics in a symbolic dimension before dimensional regularization"
    )]
    ConcreteDimensionMismatch,
    #[error("expected {expected} distinct Feynman parameters, received {actual}")]
    ParameterCount { expected: usize, actual: usize },
    #[error(
        "Feynman parameter {0} already occurs in the numerator, measure multiplier or dimension"
    )]
    ParameterCollision(symbolica::atom::Atom),
    #[error("invalid scalar parameter bindings: {0}")]
    ScalarBindings(String),
}
