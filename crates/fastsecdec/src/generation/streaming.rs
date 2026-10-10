//! Disk-backed, synchronous generation units for caller-owned process workers.
//!
//! Only descriptors cross the caller's scheduling boundary. Native source,
//! mapped expressions and shared endpoint formulas live in independently
//! verified records. Each scientific call restores at most its active chart
//! and an exact symmetry candidate, or one final representative sector.
mod codec;
mod prepare;
mod records;
mod sector;
mod shared;
#[cfg(test)]
mod tests;

pub use codec::{RecordRef, StreamingError};
pub use prepare::{
    build_formula, compare_symmetry, discover, finish_preparation, prepare, prepare_with_runtime,
};
pub use sector::{GeneratedUnit, generate_sector};
pub use shared::{
    discover_prepared, prepare_chart_source, prepare_recipes_with_runtime,
    prepare_recipes_with_runtime_and_dispatch,
};

use crate::kernel::indexed::ProgramRecipe;
use serde::{Deserialize, Serialize};

/// Immutable source context and the geometry jobs to discover independently.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preparation {
    /// Canonical physical source identity, independent of the selected recipe.
    pub source_identity: String,
    /// Selected mathematical recipe; never inferred from an old staging payload.
    pub program_recipe: ProgramRecipe,
    pub source: RecordRef,
    pub charts: Vec<MapJob>,
    pub mode: super::GenerationMode,
    pub dimension: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapJob {
    pub index: usize,
    pub map: RecordRef,
}

/// A compact directory of recipes sharing the same physical input and geometry.
/// Every recipe still owns its exact symmetry and subtraction-formula context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedRecipeSet {
    pub source_identity: String,
    pub recipes: Vec<Preparation>,
}

/// Recipe-neutral monomial-extracted source for one complete chart. Heavy native
/// residuals live in `record`; this receipt is safe to retain in the coordinator.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedChartSource {
    pub index: usize,
    pub source_identity: String,
    pub map: RecordRef,
    pub dimension: usize,
    pub record: RecordRef,
}

/// A completed mapped chart. Keys locate candidates; only native equality can
/// establish either a symmetry or a shared subtraction formula.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredSector {
    /// Selected mathematical recipe; never inferred from an old staging payload.
    pub program_recipe: ProgramRecipe,
    pub index: usize,
    pub source_id: String,
    pub dimension: usize,
    pub record: RecordRef,
    pub symmetry_key: Option<String>,
    pub formula_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymmetryAssignment {
    pub program_recipe: ProgramRecipe,
    pub source_id: String,
    pub source: usize,
    pub representative: usize,
    pub permutation: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormulaRecord {
    /// Selected mathematical recipe; never inferred from an old staging payload.
    pub program_recipe: ProgramRecipe,
    pub key: String,
    pub source_id: String,
    pub record: RecordRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChartUse {
    pub chart: DiscoveredSector,
    pub permutation: Vec<usize>,
}

/// One independently consumable representative, including its source charts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectorJob {
    /// Selected mathematical recipe; never inferred from an old staging payload.
    pub program_recipe: ProgramRecipe,
    pub index: usize,
    pub source: RecordRef,
    pub charts: Vec<ChartUse>,
    pub formula: Option<FormulaRecord>,
    pub total: usize,
}

/// Final compact work catalogue. Contains no native expression/evaluator.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedGeneration {
    /// Canonical physical source identity, independent of the selected recipe.
    pub source_identity: String,
    /// Selected mathematical recipe; never inferred from an old staging payload.
    pub program_recipe: ProgramRecipe,
    pub source: RecordRef,
    pub sectors: Vec<SectorJob>,
    pub source_charts: usize,
    pub unique_formulas: usize,
}
