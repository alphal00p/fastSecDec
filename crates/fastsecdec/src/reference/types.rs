use crate::status::CoefficientComponent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CoefficientKey {
    pub order: i32,
    pub component: CoefficientComponent,
}

/// Missing or rounded uncertainties never imply exactness.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ReferenceUncertainty {
    Exact,
    StandardError(f64),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceCoefficient {
    pub key: CoefficientKey,
    pub value: f64,
    pub uncertainty: ReferenceUncertainty,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReferenceValidation {
    Unverified,
    /// Caller-recorded independent evidence, not a claim made by this adapter.
    Checked {
        evidence: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceProvenance {
    pub source: String,
    pub convention: String,
    pub revision: Option<String>,
    pub location: Option<String>,
    pub recorded_utc: Option<String>,
    pub engine: Option<String>,
    pub notes: Vec<String>,
    /// Preserve additional source metadata without assigning it scientific meaning.
    pub attributes: BTreeMap<String, serde_json::Value>,
}

impl ReferenceProvenance {
    pub fn validate(&self) -> super::Result<()> {
        if self.source.trim().is_empty() || self.convention.trim().is_empty() {
            return Err(super::ReferenceError::Invalid(
                "empty reference provenance or convention".into(),
            ));
        }
        Ok(())
    }
    pub fn new(source: impl Into<String>, convention: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            convention: convention.into(),
            revision: None,
            location: None,
            recorded_utc: None,
            engine: None,
            notes: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceResult {
    pub coefficients: Vec<ReferenceCoefficient>,
    pub provenance: ReferenceProvenance,
    pub validation: ReferenceValidation,
    pub kernel_content_id: Option<String>,
}

impl ReferenceResult {
    /// Construct directly from native numerical results, without serialization.
    pub fn new(coefficients: Vec<ReferenceCoefficient>, provenance: ReferenceProvenance) -> Self {
        Self {
            coefficients,
            provenance,
            validation: ReferenceValidation::Unverified,
            kernel_content_id: None,
        }
    }

    /// Validate metadata, unique keys and finite values before expensive work.
    /// This checks representation; it does not establish reference accuracy.
    pub fn validate(&self) -> super::Result<()> {
        super::compare::validate_reference(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Compatibility {
    Unknown,
    Confirmed { basis: String },
    Mismatch { detail: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Independence {
    #[default]
    Unknown,
    Independent {
        basis: String,
    },
    Correlated {
        detail: String,
    },
}

/// The caller owns convention conversion and kinematic matching. This adapter
/// records those assertions and never performs symbolic conversion itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonContext {
    pub kernel_content_id: String,
    pub normalization: Compatibility,
    pub kinematics: Compatibility,
    pub independence: Independence,
}

impl ComparisonContext {
    /// Validate caller-recorded assertions even before an estimate exists.
    pub fn validate(&self) -> super::Result<()> {
        if self.kernel_content_id.is_empty() {
            return Err(super::ReferenceError::Invalid(
                "empty estimate kernel identity".into(),
            ));
        }
        for compatibility in [&self.normalization, &self.kinematics] {
            let text = match compatibility {
                Compatibility::Unknown => continue,
                Compatibility::Confirmed { basis } => basis,
                Compatibility::Mismatch { detail } => detail,
            };
            if text.trim().is_empty() {
                return Err(super::ReferenceError::Invalid(
                    "empty compatibility evidence/explanation".into(),
                ));
            }
        }
        let text = match &self.independence {
            Independence::Unknown => return Ok(()),
            Independence::Independent { basis } => basis,
            Independence::Correlated { detail } => detail,
        };
        if text.trim().is_empty() {
            return Err(super::ReferenceError::Invalid(
                "empty independence/correlation evidence".into(),
            ));
        }
        Ok(())
    }

    pub fn validate_reference(&self, reference: &ReferenceResult) -> super::Result<()> {
        self.validate()?;
        reference.validate()?;
        if let Some(identity) = &reference.kernel_content_id
            && identity != &self.kernel_content_id
        {
            return Err(super::ReferenceError::KernelIdentityMismatch {
                reference: identity.clone(),
                estimate: self.kernel_content_id.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IneligibilityReason {
    IncompleteProduction,
    UnverifiedReference,
    NormalizationUnconfirmed,
    NormalizationMismatch,
    KinematicsUnconfirmed,
    KinematicsMismatch,
    IndependenceUnconfirmed,
    EstimatesCorrelated,
    MissingEstimate,
    MissingReference,
    UnknownReferenceUncertainty,
}

/// Eligibility describes supplied evidence and coverage, not mathematical truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonEligibility {
    pub eligible: bool,
    pub reasons: Vec<IneligibilityReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EstimatedCoefficient {
    pub value: f64,
    pub standard_error: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnavailablePull {
    MissingEstimate,
    MissingReference,
    UnknownReferenceUncertainty,
    IndependenceUnconfirmed,
    EstimatesCorrelated,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Pull {
    /// Diagnostic signed pull; consult report eligibility before interpreting it.
    Value(f64),
    ZeroCombinedError {
        equal: bool,
    },
    Unavailable(UnavailablePull),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub key: CoefficientKey,
    pub estimate: Option<EstimatedCoefficient>,
    pub reference: Option<ReferenceCoefficient>,
    pub difference: Option<f64>,
    /// Signed difference divided by the absolute reference; absent at zero.
    pub relative_difference: Option<f64>,
    pub combined_standard_error: Option<f64>,
    pub pull: Pull,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceComparison {
    pub provenance: ReferenceProvenance,
    pub validation: ReferenceValidation,
    pub context: ComparisonContext,
    pub eligibility: ComparisonEligibility,
    /// Union of estimate/reference keys; no absent coefficient is assumed zero.
    pub rows: Vec<ComparisonRow>,
}
