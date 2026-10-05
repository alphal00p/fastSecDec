use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Instant,
};

use fastsecdec::kernel::KernelSet;
use fastsecdec::status::GenerationTimings;
use serde::{Deserialize, Serialize};

use crate::CliResult;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub revision: String,
    pub source_state: String,
}

pub fn dependencies() -> Vec<Dependency> {
    [
        (
            "feynkit",
            env!("FASTSECDEC_FEYNKIT_REVISION"),
            env!("FASTSECDEC_FEYNKIT_STATE"),
        ),
        (
            "symbolica",
            env!("FASTSECDEC_SYMBOLICA_REVISION"),
            env!("FASTSECDEC_SYMBOLICA_STATE"),
        ),
        (
            "numerica",
            env!("FASTSECDEC_NUMERICA_REVISION"),
            env!("FASTSECDEC_NUMERICA_STATE"),
        ),
        ("symjit", "2.26.4", "published Rust crate"),
    ]
    .into_iter()
    .map(|(name, revision, source_state)| Dependency {
        name: name.into(),
        revision: revision.into(),
        source_state: source_state.into(),
    })
    .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub blake3: String,
    #[serde(default, skip_serializing_if = "SourceFingerprint::is_bytes")]
    pub fingerprint: SourceFingerprint,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFingerprint {
    #[default]
    Bytes,
    RunCardWithoutReference,
}

impl SourceFingerprint {
    fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes)
    }

    pub fn hash(self, bytes: &[u8]) -> CliResult<String> {
        let hash = match self {
            Self::Bytes => blake3::hash(bytes),
            Self::RunCardWithoutReference => {
                // Preserve every parsed root field except observational reference
                // steering, including fields unknown to this application's schema.
                let mut card: toml::Table = toml::from_str(std::str::from_utf8(bytes)?)?;
                card.remove("reference");
                blake3::hash(toml::to_string(&card)?.as_bytes())
            }
        };
        Ok(hash.to_hex().to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub name: String,
    pub sources: Vec<SourceFile>,
    pub dependencies: Vec<Dependency>,
    pub domain: String,
    pub assume_no_threshold: bool,
    pub dimension: String,
    pub regulator: String,
    pub measure: String,
    pub measure_multiplier: String,
    pub max_order: i32,
    pub integration: serde_json::Value,
    /// Native preparation of the original graph family, when explicitly requested.
    /// Omission preserves historical/original artifact identities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_preparation: Option<fastsecdec::parametric::FamilyPreparationReport>,
}

#[derive(Serialize, Deserialize)]
pub struct Artifact {
    format_version: u32,
    pub content_id: String,
    pub provenance: Provenance,
    kernel: serde_json::Value,
    /// Observations are intentionally excluded from the scientific content hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_timings: Option<GenerationTimings>,
    /// Comparison steering is observational and excluded from scientific identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<crate::config::ReferenceInput>,
    #[serde(skip)]
    pub loading_seconds: f64,
}

impl Artifact {
    pub fn new(kernels: &KernelSet, provenance: Provenance) -> CliResult<Self> {
        let kernel = serde_json::from_slice(&kernels.to_bytes()?)?;
        let mut result = Self {
            format_version: 1,
            content_id: String::new(),
            provenance,
            kernel,
            generation_timings: None,
            reference: None,
            loading_seconds: 0.0,
        };
        result.content_id = result.identity()?;
        Ok(result)
    }
    fn identity(&self) -> CliResult<String> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"fastsecdec-artifact-v1");
        hasher.update(&serde_json::to_vec(&self.provenance)?);
        hasher.update(&serde_json::to_vec(&self.kernel)?);
        Ok(hasher.finalize().to_hex().to_string())
    }
    pub fn save(&self, path: &Path) -> CliResult<()> {
        atomic_write(path, &serde_json::to_vec_pretty(self)?)
    }
    pub fn kernel_content_id(&self) -> CliResult<&str> {
        self.kernel["content_id"]
            .as_str()
            .ok_or_else(|| "portable artifact has no kernel content identity".into())
    }
    pub fn verify_input_sources(&self, input: &Path) -> CliResult<()> {
        let first = self
            .provenance
            .sources
            .first()
            .ok_or("artifact has no source provenance")?;
        if fs::canonicalize(input)? != fs::canonicalize(&first.path)? {
            return Err("resume run card differs from the artifact's input; regenerate before integrating changed inputs".into());
        }
        for source in &self.provenance.sources {
            if source.fingerprint.hash(&fs::read(&source.path)?)? != source.blake3 {
                return Err(format!(
                    "input source {} changed since generation; checkpoint resume refused",
                    source.path
                )
                .into());
            }
        }
        Ok(())
    }
    pub fn load(path: &Path) -> CliResult<(Self, KernelSet)> {
        Self::load_with_preflight(path, |_| Ok(()))
    }

    /// Validate optional caller-side input before recompiling portable kernels.
    pub fn load_with_preflight(
        path: &Path,
        preflight: impl FnOnce(&Self) -> CliResult<()>,
    ) -> CliResult<(Self, KernelSet)> {
        let started = Instant::now();
        let mut artifact: Self = serde_json::from_slice(&fs::read(path)?)?;
        if artifact.format_version != 1 || artifact.content_id != artifact.identity()? {
            return Err("artifact version or complete content identity is invalid".into());
        }
        if artifact.provenance.dependencies != dependencies() {
            return Err("artifact dependency identities differ from this build; regenerate with the recorded dependency revisions".into());
        }
        preflight(&artifact)?;
        let kernels = KernelSet::from_bytes(&serde_json::to_vec(&artifact.kernel)?)?;
        artifact.loading_seconds = started.elapsed().as_secs_f64();
        Ok((artifact, kernels))
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> CliResult<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_preparation_preserves_legacy_provenance_serialization() {
        let legacy = r#"{"name":"old","sources":[],"dependencies":[],"domain":"ProjectiveSimplex","assume_no_threshold":false,"dimension":"4-2*eps","regulator":"eps","measure":"native","measure_multiplier":"1","max_order":0,"integration":{}}"#;
        let provenance: Provenance = serde_json::from_str(legacy).unwrap();
        assert!(provenance.family_preparation.is_none());
        assert_eq!(serde_json::to_string(&provenance).unwrap(), legacy);
        let mode = SourceFingerprint::RunCardWithoutReference;
        assert_ne!(
            mode.hash(b"[generation]\norder=0\n").unwrap(),
            mode.hash(b"[generation]\norder=0\n[generation.family_preparation.SingleUnitTerm]\nmax_states=32\n").unwrap(),
        );
    }

    #[test]
    fn scientific_card_fingerprint_excludes_only_root_reference_and_preserves_legacy_bytes() {
        let mode = SourceFingerprint::RunCardWithoutReference;
        let original = b"unknown_root = 1\n[integration]\npoints = 1024\n";
        let comparison =
            b"unknown_root=1\n[integration]\npoints=1024\n[reference]\npath='target.json'\n";
        assert_eq!(mode.hash(original).unwrap(), mode.hash(comparison).unwrap());
        assert_ne!(
            mode.hash(original).unwrap(),
            mode.hash(b"unknown_root=2\n[integration]\npoints=1024\n")
                .unwrap()
        );
        assert_ne!(
            mode.hash(b"unknown_root=nan\n").unwrap(),
            mode.hash(b"unknown_root=inf\n").unwrap()
        );
        assert_ne!(
            mode.hash(b"unknown_root=2026-10-04\n").unwrap(),
            mode.hash(b"unknown_root='2026-10-04'\n").unwrap()
        );
        assert_ne!(
            mode.hash(original).unwrap(),
            mode.hash(b"unknown_root=1\n[integration]\npoints=2048\n")
                .unwrap()
        );
        let legacy: SourceFile =
            serde_json::from_str(r#"{"path":"input.toml","blake3":"old"}"#).unwrap();
        assert!(matches!(legacy.fingerprint, SourceFingerprint::Bytes));
        assert!(
            !serde_json::to_value(&legacy)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("fingerprint")
        );
        assert_ne!(
            legacy.fingerprint.hash(original).unwrap(),
            legacy.fingerprint.hash(comparison).unwrap()
        );
    }
}
