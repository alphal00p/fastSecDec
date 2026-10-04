use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCard {
    pub input: Option<GraphInput>,
    pub direct: Option<DirectInput>,
    #[serde(default)]
    pub kinematics: KinematicsInput,
    #[serde(default)]
    pub parameters: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub integral: IntegralInput,
    #[serde(default)]
    pub generation: GenerationInput,
    #[serde(default)]
    pub integration: IntegrationInput,
    pub reference: Option<ReferenceInput>,
}

/// Observational comparison steering; never part of numerical integration settings.
#[derive(Clone, Debug, Default, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceInput {
    pub path: PathBuf,
    pub normalization_evidence: Option<String>,
    pub kinematics_evidence: Option<String>,
    pub independence_evidence: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphInput {
    pub graph: PathBuf,
    pub model: PathBuf,
    pub parameter_card: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KinematicsInput {
    #[serde(default)]
    pub products: Vec<Product>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub left: usize,
    pub right: usize,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IntegralInput {
    pub dimension: String,
    pub regulator: String,
    pub measure_multiplier: String,
    pub powers: BTreeMap<String, u32>,
}

impl Default for IntegralInput {
    fn default() -> Self {
        Self {
            dimension: "4-2*eps".into(),
            regulator: "eps".into(),
            measure_multiplier: "1".into(),
            powers: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GenerationInput {
    pub order: i32,
    pub assume_no_threshold: bool,
    pub max_sectors: usize,
    pub max_support_pairs: usize,
}

impl Default for GenerationInput {
    fn default() -> Self {
        Self {
            order: 0,
            assume_no_threshold: false,
            max_sectors: 1_000_000,
            max_support_pairs: 10_000_000,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IntegrationInput {
    pub method: String,
    pub points: u64,
    pub shifts: u32,
    pub seed: u64,
    pub package_points: u64,
    pub workers: usize,
    pub periodization: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub production_seconds: f64,
    pub max_rounds: usize,
    pub replay: fastsecdec::kernel::ReplayPolicy,
}

impl Default for IntegrationInput {
    fn default() -> Self {
        Self {
            method: "qmc".into(),
            points: 4096,
            shifts: 64,
            seed: 0,
            package_points: 1024,
            workers: 1,
            periodization: "korobov3".into(),
            absolute_tolerance: 1e-8,
            relative_tolerance: 1e-3,
            production_seconds: 10.0,
            max_rounds: 1,
            replay: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectInput {
    pub domain: String,
    pub parameters: Vec<String>,
    pub terms: Vec<DirectTerm>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectTerm {
    #[serde(default = "one")]
    pub prefactor: String,
    #[serde(default)]
    pub monomial_powers: Vec<String>,
    #[serde(default)]
    pub factors: Vec<DirectFactor>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectFactor {
    pub polynomial: Option<String>,
    pub polynomial_file: Option<PathBuf>,
    pub exponent: String,
    #[serde(default = "singularity")]
    pub role: String,
}
fn one() -> String {
    "1".into()
}
fn singularity() -> String {
    "singularity".into()
}
