mod coefficient;
mod companion;
pub use coefficient::{
    CompanionCoefficientChart, CompanionCoefficientProduction, construct_companion_coefficient,
    construct_companion_coefficient_for_cycle,
};
pub use companion::{
    CompanionContactOpen, CompanionOpenProduction, CompanionOpenProgress, produce_companion_open,
};
#[cfg(test)]
mod tests;
