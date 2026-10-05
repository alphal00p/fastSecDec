use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;

#[cfg(test)]
mod tests;

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
    /// Additional formal vectors in a numerator, such as external helicities.
    #[serde(default)]
    pub auxiliary_momenta: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub left: MomentumName,
    pub right: MomentumName,
    pub value: String,
}

/// Integer labels preserve the native graph's P(i) shorthand. Expressions
/// allow existing HEPKit momentum names without a second vector notation.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum MomentumName {
    External(usize),
    Expression(String),
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
    pub coefficient_expansion: fastsecdec::generation::CoefficientExpansionOptions,
    /// Native graph-family preparation; unlike the library policy's default,
    /// historical CLI cards keep their original propagator representation.
    pub family_preparation: fastsecdec::parametric::FamilyPreparationPolicy,
}

impl Default for GenerationInput {
    fn default() -> Self {
        Self {
            order: 0,
            assume_no_threshold: false,
            max_sectors: 1_000_000,
            max_support_pairs: 10_000_000,
            coefficient_expansion: Default::default(),
            family_preparation: fastsecdec::parametric::FamilyPreparationPolicy::Original,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IntegrationInput {
    /// Native scope; absent historical settings continue to mean the full integral.
    #[serde(skip_serializing_if = "fastsecdec::results::ResultScope::is_full_integral")]
    pub scope: fastsecdec::results::ResultScope,
    pub method: String,
    pub points: u64,
    pub shifts: u32,
    pub seed: u64,
    pub package_points: u64,
    pub workers: usize,
    pub periodization: String,
    /// Omitted in legacy/default serialization to preserve old checkpoint settings.
    #[serde(skip_serializing_if = "is_legacy_lattice")]
    pub lattice: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub production_seconds: f64,
    pub max_rounds: usize,
    pub replay: fastsecdec::kernel::ReplayPolicy,
}

impl Default for IntegrationInput {
    fn default() -> Self {
        Self {
            scope: Default::default(),
            method: "qmc".into(),
            points: 4096,
            shifts: 64,
            seed: 0,
            package_points: 1024,
            workers: 1,
            periodization: "korobov3".into(),
            lattice: "kuo33002".into(),
            absolute_tolerance: 1e-8,
            relative_tolerance: 1e-3,
            production_seconds: 10.0,
            max_rounds: 1,
            replay: Default::default(),
        }
    }
}

fn is_legacy_lattice(value: &str) -> bool {
    value == "kuo33002"
}

impl IntegrationInput {
    pub fn published_lattice(&self) -> crate::CliResult<fastsecdec::integration::PublishedLattice> {
        use fastsecdec::integration::PublishedLattice;
        match self.lattice.as_str() {
            "kuo33002" => Ok(PublishedLattice::Kuo33002),
            "kuo38005" => Ok(PublishedLattice::Kuo38005),
            "kuo39101" => Ok(PublishedLattice::Kuo39101),
            "hkkn-alpha3" => Ok(PublishedLattice::HkknAlpha3),
            _ => Err("lattice must be kuo33002, kuo38005, kuo39101, or hkkn-alpha3".into()),
        }
    }

    pub fn qmc_settings(&self) -> crate::CliResult<fastsecdec::integration::QmcSettings> {
        use fastsecdec::integration::{Periodization, PublishedLattice, QmcSettings, RuleSource};
        let catalogue = self.published_lattice()?;
        let settings = QmcSettings {
            points: self.points,
            shifts: self.shifts,
            seed: self.seed,
            package_points: self.package_points,
            periodization: match self.periodization.as_str() {
                "none" => Periodization::None,
                "korobov3" => Periodization::Korobov3,
                _ => return Err("periodization must be none or korobov3".into()),
            },
            rule: if catalogue == PublishedLattice::Kuo33002 {
                RuleSource::Kuo
            } else {
                RuleSource::Published(catalogue)
            },
        };
        settings.validate()?;
        Ok(settings)
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
