//! Thin storage owner around the native preparation/publication APIs. No Python
//! or global verified geometry is retained after native preparation returns.
use fastsecdec::{
    generation::source_identity,
    kernel::{
        ThresholdPublicationPlan,
        indexed::{ProgramArchiveCatalogue, ProgramArchiveWriter},
    },
    parametric::ParametricIntegrand,
    threshold::generation::{
        self as native, PreparationCheckpoint, PreparationOptions, PreparationReceipt, Progress,
    },
};
use std::{
    fs::{self, File},
    io::{Seek, SeekFrom},
    ops::ControlFlow,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};
use tempfile::TempDir;
pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(super) struct Work {
    pub options: PreparationOptions,
    pub limit: u64,
    pub storage: Option<Rc<TempDir>>,
    pub receipt: Option<PreparationReceipt>,
    pub progress: Option<Progress>,
    pub next: usize,
    input: Option<Arc<ParametricIntegrand>>,
    source: Option<String>,
    root: Option<PathBuf>,
    checkpoint: Option<PreparationCheckpoint>,
    attempt: usize,
    plan: Option<ThresholdPublicationPlan>,
    archive: Option<ProgramArchiveWriter<File>>,
}
impl Work {
    pub fn new(options: PreparationOptions, limit: u64) -> Self {
        Self {
            options,
            limit,
            storage: None,
            receipt: None,
            progress: None,
            next: 0,
            input: None,
            source: None,
            root: None,
            checkpoint: None,
            attempt: 0,
            plan: None,
            archive: None,
        }
    }
    pub fn initialized(&self) -> bool {
        self.source.is_some()
    }
    pub fn initialize(&mut self, input: ParametricIntegrand) -> Result<()> {
        let source = source_identity(&input, &[], &[])?;
        self.storage = Some(Rc::new(
            tempfile::Builder::new()
                .prefix("fastsecdec-threshold-")
                .tempdir()?,
        ));
        self.source = Some(source);
        self.input = Some(Arc::new(input));
        Ok(())
    }
    pub fn prepare(
        &mut self,
        mut observer: impl FnMut(&Progress) -> ControlFlow<()>,
    ) -> Result<bool> {
        if self.receipt.is_some() {
            return Ok(true);
        }
        let storage = self
            .storage
            .as_ref()
            .ok_or("threshold input not initialized")?;
        let source = self
            .source
            .as_deref()
            .ok_or("missing original source identity")?;
        self.attempt = self
            .attempt
            .checked_add(1)
            .ok_or("threshold retry count overflow")?;
        if self.checkpoint.is_none() {
            let root = storage.path().join(format!("preparation-{}", self.attempt));
            fs::create_dir(&root)?;
            self.root = Some(root);
        }
        let root = self
            .root
            .as_ref()
            .ok_or("missing native preparation root")?;
        let prior = self.checkpoint.clone();
        let progress = &mut self.progress;
        let checkpoint = &mut self.checkpoint;
        let mut observe = |p: &Progress| {
            *progress = Some(p.clone());
            if let Some(c) = p.checkpoint() {
                *checkpoint = Some(c)
            }
            observer(p)
        };
        let result = if let Some(c) = prior {
            native::resume_evidence(
                root,
                &c,
                &format!("vectors-{}", self.attempt),
                source,
                self.limit,
                &mut observe,
            )
        } else {
            native::prepare(
                self.input.as_ref().ok_or("missing native input")?.clone(),
                self.options.clone(),
                root,
                &mut observe,
            )
        };
        let prepared = match result {
            Ok(p) => p,
            Err(native::Error::Cancelled) => return Ok(false),
            Err(e) => return Err(e.into()),
        };
        let receipt = prepared.receipt;
        if receipt.publication.source_identity != source {
            return Err("native threshold source identity changed".into());
        }
        let plan = ThresholdPublicationPlan::from_trusted_preparer(
            &root.join(&receipt.work_directory),
            receipt.publication.clone(),
            source,
            &receipt.publication.prepared_identity,
            self.limit,
        )?;
        let archive = plan.archive_writer(File::create(storage.path().join("integral.fsd"))?)?;
        self.plan = Some(plan);
        self.archive = Some(archive);
        self.receipt = Some(receipt);
        // Original input/Atom density is no longer needed by compilation jobs.
        self.input = None;
        Ok(true)
    }
    pub fn jobs(&self) -> usize {
        self.plan
            .as_ref()
            .map_or(0, ThresholdPublicationPlan::job_count)
    }
    pub fn compile_one(&mut self) -> Result<()> {
        let plan = self
            .plan
            .as_ref()
            .ok_or("call prepare() before compilation")?;
        if self.next >= plan.job_count() {
            return Ok(());
        }
        let root = self.root.as_ref().ok_or("missing preparation root")?.join(
            &self
                .receipt
                .as_ref()
                .ok_or("missing receipt")?
                .work_directory,
        );
        // A single temporary file holds one local record; completed records are
        // copied immediately into the native archive, not collected in memory.
        let mut record =
            tempfile::tempfile_in(self.storage.as_ref().ok_or("missing storage")?.path())?;
        let receipt = plan
            .work(self.next)?
            .compile_record(&root, self.limit, &mut record)?;
        record.seek(SeekFrom::Start(0))?;
        plan.append_work_record(
            self.archive.as_mut().ok_or("missing archive writer")?,
            &mut record,
            receipt,
        )?;
        self.next += 1;
        Ok(())
    }
    pub fn finish(&mut self) -> Result<(File, ProgramArchiveCatalogue)> {
        if self.receipt.is_none() || self.next != self.jobs() {
            return Err("threshold compilation is incomplete".into());
        }
        Ok(self
            .archive
            .take()
            .ok_or("archive already published")?
            .finish()?)
    }
}

#[cfg(test)]
#[path = "work/tests.rs"]
mod tests;
