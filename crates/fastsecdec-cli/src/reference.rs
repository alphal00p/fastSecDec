//! CLI file selection and presentation around the native reference adapter.
use std::{fs, path::Path};

#[cfg(test)]
use fastsecdec::integration::VectorEstimate;
use fastsecdec::reference::{
    self, ComparisonContext, Compatibility, Independence, ReferenceComparison, ReferenceResult,
};
use serde::Serialize;

use crate::{
    CliResult,
    config::{ReferenceInput, RunCard},
};

#[derive(Clone, Serialize)]
pub struct ReferenceSource {
    path: String,
    blake3: String,
}

pub struct PreparedReference {
    pub settings: ReferenceInput,
    source: ReferenceSource,
    result: ReferenceResult,
}

/// Resolve card paths relative to the card; command-line overrides use cwd.
pub fn from_card(
    path: &Path,
    override_path: Option<&Path>,
) -> CliResult<Option<PreparedReference>> {
    let card: RunCard = toml::from_str(&fs::read_to_string(path)?)?;
    let mut settings = card.reference;
    if let Some(settings) = &mut settings {
        settings.path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&settings.path);
    }
    prepare(settings, override_path)
}

pub fn prepare(
    mut settings: Option<ReferenceInput>,
    override_path: Option<&Path>,
) -> CliResult<Option<PreparedReference>> {
    if let Some(path) = override_path {
        let path = fs::canonicalize(path)?;
        let same_target = settings.as_ref().is_some_and(|settings| {
            fs::canonicalize(&settings.path).is_ok_and(|previous| previous == path)
        });
        // Evidence about another target cannot certify the selected replacement.
        if !same_target {
            settings = Some(ReferenceInput {
                path,
                ..Default::default()
            });
        }
    }
    let Some(mut settings) = settings else {
        return Ok(None);
    };
    for (name, evidence) in [
        ("normalization_evidence", &settings.normalization_evidence),
        ("kinematics_evidence", &settings.kinematics_evidence),
        ("independence_evidence", &settings.independence_evidence),
    ] {
        if evidence.as_ref().is_some_and(|text| text.trim().is_empty()) {
            return Err(format!("reference {name} must contain evidence or be omitted").into());
        }
    }
    if settings.path.as_os_str().is_empty() {
        return Err("reference path cannot be empty".into());
    }
    settings.path =
        crate::artifact::relative_path(&fs::canonicalize(&settings.path)?, Path::new("."))?;
    let bytes = fs::read(&settings.path)?;
    let result = reference::read_reference(&bytes)?;
    let source = ReferenceSource {
        path: settings.path.to_string_lossy().into_owned(),
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    };
    Ok(Some(PreparedReference {
        settings,
        source,
        result,
    }))
}

impl PreparedReference {
    pub fn stored(&self, content_id: &str) -> CliResult<fastsecdec::results::StoredReference> {
        self.validate_identity(content_id)?;
        Ok(fastsecdec::results::StoredReference {
            reference: self.result.clone(),
            context: self.context(content_id),
        })
    }

    pub fn source(&self) -> &ReferenceSource {
        &self.source
    }

    pub fn report_saved(
        &self,
        result: &fastsecdec::results::SavedIntegrationResult,
    ) -> CliResult<ReferenceReport> {
        use fastsecdec::results::{ResultComparison, ResultComparisonUnavailable};
        let stored = result
            .stored_reference
            .as_ref()
            .ok_or("saved result has no original reference for its comparison report")?;
        Ok(match result.comparison()? {
            ResultComparison::Compared(comparison) => ReferenceReport::Compared {
                source: self.source.clone(),
                comparison: *comparison,
            },
            ResultComparison::Unavailable(ResultComparisonUnavailable::NoEstimate) => {
                ReferenceReport::WaitingForCoverage {
                    source: self.source.clone(),
                    reference: stored.reference.clone(),
                    context: stored.context.clone(),
                }
            }
            ResultComparison::Unavailable(reason) => ReferenceReport::Unavailable {
                source: self.source.clone(),
                reference: stored.reference.clone(),
                context: stored.context.clone(),
                reason,
            },
        })
    }

    pub fn validate_identity(&self, content_id: &str) -> CliResult<()> {
        if let Some(reference) = &self.result.kernel_content_id
            && reference != content_id
        {
            return Err(reference::ReferenceError::KernelIdentityMismatch {
                reference: reference.clone(),
                estimate: content_id.to_owned(),
            }
            .into());
        }
        Ok(())
    }

    fn context(&self, content_id: &str) -> ComparisonContext {
        let compatibility = |evidence: &Option<String>| {
            evidence
                .as_ref()
                .map_or(Compatibility::Unknown, |basis| Compatibility::Confirmed {
                    basis: basis.clone(),
                })
        };
        ComparisonContext {
            kernel_content_id: content_id.to_owned(),
            normalization: compatibility(&self.settings.normalization_evidence),
            kinematics: compatibility(&self.settings.kinematics_evidence),
            independence: self.settings.independence_evidence.as_ref().map_or(
                Independence::Unknown,
                |basis| Independence::Independent {
                    basis: basis.clone(),
                },
            ),
        }
    }

    #[cfg(test)]
    pub fn report(
        &self,
        content_id: &str,
        estimate: Option<&VectorEstimate>,
    ) -> CliResult<ReferenceReport> {
        self.validate_identity(content_id)?;
        let context = self.context(content_id);
        Ok(match estimate {
            Some(estimate) => ReferenceReport::Compared {
                source: self.source.clone(),
                comparison: reference::compare(estimate, &self.result, &context)?,
            },
            None => ReferenceReport::WaitingForCoverage {
                source: self.source.clone(),
                reference: self.result.clone(),
                context,
            },
        })
    }
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReferenceReport {
    Compared {
        source: ReferenceSource,
        comparison: ReferenceComparison,
    },
    WaitingForCoverage {
        source: ReferenceSource,
        reference: ReferenceResult,
        context: ComparisonContext,
    },
    Unavailable {
        source: ReferenceSource,
        reference: ReferenceResult,
        context: ComparisonContext,
        reason: fastsecdec::results::ResultComparisonUnavailable,
    },
}

impl std::fmt::Display for ReferenceReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compared { comparison, .. } => write!(formatter, "{comparison}"),
            Self::WaitingForCoverage { reference, .. } => writeln!(
                formatter,
                "Reference: {} ({}) · waiting for integration coverage; no estimate available",
                reference.provenance.source, reference.provenance.convention
            ),
            Self::Unavailable {
                reference, reason, ..
            } => writeln!(
                formatter,
                "Reference: {} ({}) · comparison unavailable: {reason}",
                reference.provenance.source, reference.provenance.convention
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastsecdec::reference::{ReferenceProvenance, encode_reference};

    #[test]
    fn absent_estimate_retains_reference_and_evidence_without_fabricated_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reference.json");
        let reference =
            ReferenceResult::new(vec![], ReferenceProvenance::new("test", "same measure"));
        fs::write(&path, encode_reference(&reference).unwrap()).unwrap();
        let prepared = prepare(
            Some(ReferenceInput {
                path,
                independence_evidence: Some(
                    "analytic derivation uses no integration samples".into(),
                ),
                ..Default::default()
            }),
            None,
        )
        .unwrap()
        .unwrap();
        let report = prepared.report("kernel", None).unwrap();
        assert!(
            report
                .to_string()
                .contains("waiting for integration coverage")
        );
        let value = serde_json::to_value(report).unwrap();
        assert_eq!(value["status"], "waiting_for_coverage");
        assert!(value.get("comparison").is_none());
        assert!(value.get("rows").is_none());
        assert_eq!(value["context"]["independence"]["status"], "independent");
        assert_eq!(value["context"]["normalization"]["status"], "unknown");
    }

    #[test]
    fn blank_compatibility_evidence_is_rejected_before_opening_the_target() {
        let error = prepare(
            Some(ReferenceInput {
                path: "missing-reference.json".into(),
                kinematics_evidence: Some(" ".into()),
                ..Default::default()
            }),
            None,
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("kinematics_evidence"));
    }
}
