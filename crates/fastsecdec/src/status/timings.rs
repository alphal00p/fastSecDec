use serde::{Deserialize, Serialize};

/// Observed caller wall times, separate from the mathematical integral identity.
/// Generation total covers input through prepared kernels and artifact metadata;
/// writing the final artifact file is outside that interval.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GenerationTimings {
    pub input_seconds: f64,
    pub parametrization_seconds: f64,
    pub domain_seconds: f64,
    pub geometry_seconds: f64,
    pub mapping_seconds: f64,
    /// Distinct unique-formula precomputation phase. Parallel runs record its
    /// coordinator wall interval, retained sessions sum active units only.
    /// None means this phase was not observed, including historical artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula_preparation_seconds: Option<f64>,
    pub symmetry_seconds: f64,
    pub subtraction_seconds: f64,
    pub laurent_seconds: f64,
    /// Entire named or caller-dispatched coefficient phase, including any exact
    /// physical fallback. In dispatched physical generation, subtraction and
    /// Laurent expansion overlap across workers: their individual wall fields
    /// remain zero and this field records the aggregate phase wall interval.
    /// With a measured formula-preparation phase, this instead covers sector
    /// instantiation and exact chart fallbacks after the shared recipes exist.
    /// This duration does not overlap subtraction_seconds or laurent_seconds.
    pub coefficient_expansion_seconds: f64,
    pub compilation_seconds: f64,
    pub total_seconds: f64,
}
