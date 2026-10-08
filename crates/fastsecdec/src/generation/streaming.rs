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
#[cfg(test)]
mod tests;

pub use codec::{RecordRef, StreamingError};
pub use prepare::{
    build_formula, compare_symmetry, discover, finish_preparation, prepare, prepare_with_runtime,
};
pub use sector::{GeneratedUnit, generate_sector};

use serde::{Deserialize, Serialize};

/// Immutable source context and the geometry jobs to discover independently.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preparation {
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

/// A completed mapped chart. Keys locate candidates; only native equality can
/// establish either a symmetry or a shared subtraction formula.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredSector {
    pub index: usize,
    pub source_id: String,
    pub dimension: usize,
    pub record: RecordRef,
    pub symmetry_key: Option<String>,
    pub formula_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymmetryAssignment {
    pub source: usize,
    pub representative: usize,
    pub permutation: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormulaRecord {
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
    pub index: usize,
    pub source: RecordRef,
    pub charts: Vec<ChartUse>,
    pub formula: Option<FormulaRecord>,
    pub total: usize,
}

/// Final compact work catalogue. Contains no native expression/evaluator.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedGeneration {
    pub source: RecordRef,
    pub sectors: Vec<SectorJob>,
    pub source_charts: usize,
    pub unique_formulas: usize,
}
