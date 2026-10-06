use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::Path,
    time::Instant,
};

use fastsecdec::kernel::KernelSet;
use fastsecdec::status::GenerationTimings;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::CliResult;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub revision: String,
    pub source_state: String,
}

pub fn dependencies() -> Vec<Dependency> {
    let symjit = format!(
        "native-version-code:{}",
        fastsecdec::kernel::symjit_version_code()
    );
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
        (
            "symjit",
            symjit.as_str(),
            "linked SymJIT Application::measure(version)",
        ),
    ]
    .into_iter()
    .map(|(name, revision, source_state)| Dependency {
        name: name.into(),
        revision: revision.into(),
        source_state: source_state.into(),
    })
    .collect()
}

fn dependency_matches(recorded: &Dependency, current: &Dependency) -> bool {
    recorded == current
        // Old artifacts stored exact Symbolica IR, never SymJIT machine code.
        // Keep their existing provenance/identity only for the same reported
        // backend; all other dependency identities remain exact comparisons.
        || (recorded.name == "symjit"
            && current.name == "symjit"
            && recorded.revision == "2.26.4"
            && recorded.source_state == "published Rust crate"
            && current.revision == "native-version-code:22604"
            && current.source_state == "linked SymJIT Application::measure(version)")
}

fn dependencies_match(recorded: &[Dependency], current: &[Dependency]) -> bool {
    recorded.len() == current.len()
        && recorded
            .iter()
            .zip(current)
            .all(|(old, new)| dependency_matches(old, new))
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
    // Keep the native JSON envelope verbatim: program byte arrays must never
    // become a second tree of per-byte JSON values on the current path.
    kernel: Box<RawValue>,
    #[serde(skip)]
    kernel_content_id: String,
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
        let kernel = serde_json::from_slice(kernels.artifact_bytes()?)?;
        let mut result = Self {
            format_version: 2,
            content_id: String::new(),
            provenance,
            kernel,
            kernel_content_id: kernels.content_id().to_owned(),
            generation_timings: None,
            reference: None,
            loading_seconds: 0.0,
        };
        result.content_id = result.identity()?;
        Ok(result)
    }

    fn identity(&self) -> CliResult<String> {
        let mut hasher = blake3::Hasher::new();
        match self.format_version {
            1 => {
                // Historical identities canonicalized the native JSON as Value.
                // Keep this branch exact for existing files, including their IDs.
                let kernel: serde_json::Value = serde_json::from_str(self.kernel.get())?;
                hasher.update(b"fastsecdec-artifact-v1");
                serde_json::to_writer(&mut hasher, &self.provenance)?;
                serde_json::to_writer(&mut hasher, &kernel)?;
            }
            2 => {
                // The native loader validates the payload against this native
                // content ID before returning kernels to a caller.
                hasher.update(b"fastsecdec-artifact-v2");
                serde_json::to_writer(&mut hasher, &self.provenance)?;
                hasher.update(self.kernel_content_id()?.as_bytes());
            }
            _ => return Err("artifact version is unsupported".into()),
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn save(&self, path: &Path) -> CliResult<()> {
        atomic_write_with(path, |writer| {
            serde_json::to_writer_pretty(writer, self)?;
            Ok(())
        })
    }

    pub fn kernel_content_id(&self) -> CliResult<&str> {
        if self.kernel_content_id.is_empty() {
            return Err("portable artifact has no kernel content identity".into());
        }
        Ok(&self.kernel_content_id)
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
        let mut artifact: Self = serde_json::from_reader(BufReader::new(File::open(path)?))?;
        if !matches!(artifact.format_version, 1 | 2) {
            return Err("artifact version is unsupported".into());
        }
        #[derive(Deserialize)]
        struct KernelIdentity {
            content_id: String,
        }
        artifact.kernel_content_id =
            serde_json::from_str::<KernelIdentity>(artifact.kernel.get())?.content_id;
        if artifact.content_id != artifact.identity()? {
            return Err("artifact complete content identity is invalid".into());
        }
        if !dependencies_match(&artifact.provenance.dependencies, &dependencies()) {
            return Err("artifact dependency identities differ from this build; regenerate with the recorded dependency revisions".into());
        }
        preflight(&artifact)?;
        let kernels = KernelSet::from_bytes(artifact.kernel.get().as_bytes())?;
        if kernels.content_id() != artifact.kernel_content_id()? {
            return Err("validated native kernel identity differs from the artifact".into());
        }
        artifact.loading_seconds = started.elapsed().as_secs_f64();
        Ok((artifact, kernels))
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> CliResult<()> {
    atomic_write_with(path, |writer| {
        writer.write_all(bytes)?;
        Ok(())
    })
}

fn atomic_write_with(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> CliResult<()>,
) -> CliResult<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut writer = BufWriter::new(file);
    let result = (|| -> CliResult<()> {
        write(&mut writer)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    drop(writer);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
#[path = "artifact/persistence_tests.rs"]
mod persistence_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_provenance_reports_actual_code_and_only_accepts_same_legacy_backend() {
        let current = dependencies();
        let symjit = current
            .iter()
            .find(|dependency| dependency.name == "symjit")
            .unwrap();
        assert_eq!(
            symjit.revision,
            format!(
                "native-version-code:{}",
                fastsecdec::kernel::symjit_version_code()
            )
        );
        assert!(dependencies_match(&current, &current));
        let legacy = Dependency {
            name: "symjit".into(),
            revision: "2.26.4".into(),
            source_state: "published Rust crate".into(),
        };
        let exact = Dependency {
            name: "symjit".into(),
            revision: "native-version-code:22604".into(),
            source_state: "linked SymJIT Application::measure(version)".into(),
        };
        assert!(dependency_matches(&legacy, &exact));
        let mut newer = exact.clone();
        newer.revision = "native-version-code:22700".into();
        assert!(!dependency_matches(&legacy, &newer));
        newer = exact.clone();
        newer.source_state = "unverified".into();
        assert!(!dependency_matches(&legacy, &newer));
        newer = exact;
        newer.name = "another dependency".into();
        assert!(!dependency_matches(&legacy, &newer));
        assert!(!dependencies_match(&current[..current.len() - 1], &current));
    }

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
