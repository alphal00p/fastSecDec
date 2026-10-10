//! Serializable scalar controls for Symbolica's existing evaluator optimizer.
use super::KernelError;
use crate::contour::ContourJacobian;
use serde::{Deserialize, Serialize};

/// Numerical execution owner; `Auto` preserves the build's historical default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatorBackend {
    #[default]
    Auto,
    Eager,
    Symjit,
}

impl EvaluatorBackend {
    fn is_auto(&self) -> bool {
        *self == Self::Auto
    }

    pub fn is_eager(self) -> bool {
        self == Self::Eager || (self == Self::Auto && cfg!(feature = "portable"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompilationSettings {
    /// Explicit eager execution is available in native and portable builds.
    /// Omitting Auto preserves historical compiler-policy bytes and identities.
    #[serde(skip_serializing_if = "EvaluatorBackend::is_auto")]
    pub backend: EvaluatorBackend,
    /// Retained generation choice; omitted Symbolic preserves historical policy bytes.
    #[serde(skip_serializing_if = "ContourJacobian::is_symbolic")]
    pub contour_jacobian: ContourJacobian,
    pub horner_iterations: usize,
    /// Maximum common-pair elimination rounds; zero disables, None is unlimited.
    #[serde(with = "cpe_rounds", alias = "max_cpe_rounds")]
    pub cpe_rounds: Option<usize>,
    /// One native optimizer core preserves deterministic builds; sectors may run in parallel.
    #[serde(deserialize_with = "single_core")]
    pub cores: usize,
    pub max_horner_scheme_variables: usize,
    pub max_common_pair_cache_entries: usize,
    pub max_common_pair_distance: usize,
    pub verbose: bool,
    pub direct_translation: bool,
}

fn single_core<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
    let cores = usize::deserialize(deserializer)?;
    if cores != 1 {
        return Err(serde::de::Error::custom(
            "evaluator cores must be 1 for deterministic optimization; use generation workers for parallelism",
        ));
    }
    Ok(cores)
}

impl Default for CompilationSettings {
    fn default() -> Self {
        Self {
            backend: EvaluatorBackend::Auto,
            contour_jacobian: ContourJacobian::Symbolic,
            horner_iterations: 10,
            cpe_rounds: Some(1000),
            cores: 1,
            max_horner_scheme_variables: 500,
            max_common_pair_cache_entries: 1_000_000,
            max_common_pair_distance: 1000,
            verbose: false,
            direct_translation: true,
        }
    }
}

impl CompilationSettings {
    /// Resolve the saved computational policy from the generated native program.
    /// The default accepts its retained choice; requesting Dual cannot convert
    /// an already generated symbolic Jacobian.
    pub fn resolve_contour_jacobian(
        mut self,
        retained: ContourJacobian,
    ) -> Result<Self, KernelError> {
        if self.contour_jacobian == ContourJacobian::Dual && retained != ContourJacobian::Dual {
            return Err(KernelError::Compilation(
                "dual contour Jacobian must be selected during generation".into(),
            ));
        }
        self.contour_jacobian = retained;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), KernelError> {
        if self.backend == EvaluatorBackend::Symjit && cfg!(feature = "portable") {
            return Err(KernelError::Compilation(
                "SymJIT is unavailable in a portable build; select eager or auto".into(),
            ));
        }
        if self.cores != 1 {
            return Err(KernelError::Compilation(
                "evaluator cores must be 1 for deterministic native optimization; parallelize sectors with generation workers".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn legacy() -> Self {
        Self {
            horner_iterations: 0,
            cpe_rounds: None,
            ..Self::default()
        }
    }

    pub(crate) fn native(self) -> symbolica::evaluate::OptimizationSettings {
        symbolica::evaluate::OptimizationSettings::new()
            .horner_iterations(self.horner_iterations)
            .cpe_iterations(self.cpe_rounds)
            .cores(self.cores)
            .max_horner_scheme_variables(self.max_horner_scheme_variables)
            .max_common_pair_cache_entries(self.max_common_pair_cache_entries)
            .max_common_pair_distance(self.max_common_pair_distance)
            .verbose(self.verbose)
            .direct_translation(self.direct_translation)
    }
}

mod cpe_rounds {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(
        value: &Option<usize>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(rounds) => serializer.serialize_u64(*rounds as u64),
            None => serializer.serialize_str("unlimited"),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<usize>, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Rounds(usize),
            Name(String),
        }
        match Value::deserialize(deserializer)? {
            Value::Rounds(rounds) => Ok(Some(rounds)),
            Value::Name(name) if name == "unlimited" => Ok(None),
            Value::Name(_) => Err(D::Error::custom(
                "expected a nonnegative round count or 'unlimited'",
            )),
        }
    }
}
