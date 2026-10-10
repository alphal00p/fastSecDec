//! Checked local Cartier extraction and companion arithmetic. Whole-equation
//! powers do not imply componentwise BM maximality or a complete resolution.
mod division;
mod factor;
mod old_boundary;
mod order;

pub use division::{CartierDivision, QuotientMethod, VerifiedCartierQuotient, divide_cartier};
pub use factor::{
    FactorProduction, FactorProgress, WholeCartierFactorization, factor_whole_cartier_equations,
};
pub use old_boundary::{
    OldBoundaryCoefficient, OldBoundaryProduction, OldBoundaryProgress, OldIncidence,
    produce_old_boundary_coefficient,
};
pub use order::{
    RelativeOrderLayer, ResidualOrderProduction, ResidualOrderProgress, RestrictedResidualOrder,
    produce_restricted_residual_order,
};

mod component_order;
#[cfg(test)]
mod component_order_tests;
#[cfg(test)]
mod tests;
pub use component_order::{
    ComponentResidualOrder, ComponentResidualProduction, produce_component_residual_order,
};
pub(crate) use old_boundary::produce_old_boundary_coefficient_for_cycle;
