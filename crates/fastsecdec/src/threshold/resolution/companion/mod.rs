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

#[cfg(test)]
mod tests;
