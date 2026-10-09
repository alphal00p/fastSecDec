//! Immutable data publication. The manifest is the sole commit point; a failed
//! replacement cannot invalidate the previous manifest's data file.
use super::*;
#[cfg(test)]
use fastsecdec::kernel::indexed::IndexedReader;
use fastsecdec::kernel::indexed::KernelCatalogue;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedStorage {
    pub data_file: String,
    pub catalogue: KernelCatalogue,
}
impl IndexedStorage {
    pub(super) fn validate(&self, integrity: bool) -> CliResult<()> {
        super::programs::validate_filename(&self.data_file)?;
        self.catalogue.validate(integrity)?;
        Ok(())
    }
}

impl Artifact {
    /// Resolve only the manifest's immutable sibling name, never arbitrary paths.
    pub fn data_path(&self, base: &Path) -> CliResult<PathBuf> {
        if let Some(data_file) = self.indexed_data_file()? {
            Ok(base
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(data_file))
        } else {
            Ok(paths(base)?.1)
        }
    }
    /// Adopt a completed staged archive without reading any sector payload. The
    /// normal and serial writers share exactly the same catalogue and file codec.
    #[cfg(test)]
    pub fn from_indexed_file(
        staged: &Path,
        catalogue: KernelCatalogue,
        provenance: Provenance,
    ) -> CliResult<Self> {
        let indexed =
            IndexedReader::from_reader(File::open(staged)?, KernelLoadOptions { validate: true })?;
        if serde_json::to_value(indexed.catalogue())? != serde_json::to_value(&catalogue)? {
            return Err("staged archive catalogue differs from completed receipts".into());
        }
        let sectors = catalogue
            .records
            .iter()
            .filter(|record| record.sector.is_some())
            .collect::<Vec<_>>();
        let summary = serde_json::json!({
            "threshold_policy":"user_responsible", "threshold_certification":"not_performed",
            "orders":catalogue.orders, "components":catalogue.components,
            "sectors":sectors.len(),
            "dimensions":sectors.iter().map(|record|record.receipt.dimension.unwrap()).collect::<Vec<_>>(),
            "runtime_parameters":catalogue.runtime_parameters,
            "evaluator_statistics":sectors.iter().map(|record|record.receipt.statistics.as_ref().unwrap()).collect::<Vec<_>>(),
        });
        let mut artifact = Self {
            format_version: 4,
            content_id: String::new(),
            kernel_content_id: catalogue.content_id.clone(),
            provenance,
            kernel: RawValue::from_string(serde_json::to_string_pretty(&summary)?)?,
            data: Vec::new(),
            indexed: Some(IndexedStorage {
                data_file: String::new(),
                catalogue,
            }),
            programs: None,
            selected_recipe: None,
            staged_data: Some(staged.to_path_buf()),
            source_root: PathBuf::from("."),
            generation_timings: None,
            generation: None,
            reference: None,
            loading_seconds: 0.0,
            validation: ArtifactValidation::default(),
            inspection: None,
        };
        artifact.content_id = artifact.identity()?;
        Ok(artifact)
    }
    /// Publication does not materialize staged data in a byte vector.
    pub fn save_staged(&self, base: &Path) -> CliResult<()> {
        if !self.is_indexed() || self.staged_data.is_none() {
            return Err("save_staged requires a completed staged indexed artifact".into());
        }
        self.save(base)
    }
    pub(super) fn save_indexed(&self, base: &Path) -> CliResult<()> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let (metadata, _) = paths(base)?;
        let directory = metadata
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(directory)?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let data_name = format!(
            "{}.{}-{}-{}.dat",
            base.file_name()
                .ok_or("missing artifact basename")?
                .to_string_lossy(),
            stamp,
            std::process::id(),
            sequence
        );
        let data_path = directory.join(&data_name);
        let mut data = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&data_path)?;
        let mut published = false;
        let result = (|| -> CliResult<()> {
            if let Some(staged) = &self.staged_data {
                std::io::copy(&mut File::open(staged)?, &mut data)?;
            } else if !self.data.is_empty() {
                data.write_all(&self.data)?;
            } else {
                return Err("indexed artifact has no available data source".into());
            }
            data.sync_all()?;
            #[cfg(unix)]
            File::open(directory)?.sync_all()?;
            // Roundtrip only the small metadata document; native data is skipped.
            // RawValue preserves its exact existing scientific-summary encoding.
            let mut manifest: Self = serde_json::from_str(&serde_json::to_string(self)?)?;
            if let Some(indexed) = &mut manifest.indexed {
                indexed.data_file = data_name;
            } else if let Some(programs) = &mut manifest.programs {
                programs.data_file = data_name;
            } else {
                return Err("indexed artifact lost its directory".into());
            }
            atomic_write_with(&metadata, |writer| {
                serde_json::to_writer_pretty(writer, &manifest)?;
                Ok(())
            })?;
            published = true;
            #[cfg(unix)]
            File::open(directory)?.sync_all()?;
            Ok(())
        })();
        drop(data);
        if result.is_err() && !published {
            let _ = fs::remove_file(data_path);
        }
        result
    }
}
