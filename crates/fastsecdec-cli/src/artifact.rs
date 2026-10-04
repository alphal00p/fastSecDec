use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use fastsecdec::kernel::KernelSet;
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
        ("symjit", "2.26.0", "published Rust crate"),
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
}

#[derive(Serialize, Deserialize)]
pub struct Artifact {
    format_version: u32,
    pub content_id: String,
    pub provenance: Provenance,
    kernel: serde_json::Value,
}

impl Artifact {
    pub fn new(kernels: &KernelSet, provenance: Provenance) -> CliResult<Self> {
        let kernel = serde_json::from_slice(&kernels.to_bytes()?)?;
        let mut result = Self {
            format_version: 1,
            content_id: String::new(),
            provenance,
            kernel,
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
            if blake3::hash(&fs::read(&source.path)?).to_hex().as_str() != source.blake3 {
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
        let artifact: Self = serde_json::from_slice(&fs::read(path)?)?;
        if artifact.format_version != 1 || artifact.content_id != artifact.identity()? {
            return Err("artifact version or complete content identity is invalid".into());
        }
        if artifact.provenance.dependencies != dependencies() {
            return Err("artifact dependency identities differ from this build; regenerate with the recorded dependency revisions".into());
        }
        let kernels = KernelSet::from_bytes(&serde_json::to_vec(&artifact.kernel)?)?;
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
