use super::super::{
    IndexedReader, failure,
    record_reader::{self, RecordSelection},
    transport,
};
use super::{ProgramArchiveCatalogue, ProgramRecipe, ProgramRecipeCatalogue};
use crate::kernel::{KernelError, KernelLoadOptions, KernelLoadProgress, KernelSet};
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    ops::ControlFlow,
};

/// Metadata-only archive owner. Constructing it validates the complete physical
/// record directory but never reads, decodes or restores any native payload.
pub struct ProgramArchiveReader<R> {
    reader: R,
    catalogue: ProgramArchiveCatalogue,
    options: KernelLoadOptions,
}
impl<R: Read + Seek> ProgramArchiveReader<R> {
    /// Refresh optional primary JIT payloads for this explicitly opened family.
    /// Only one record's native owners are resident; all recipe and catalogue
    /// mathematical IDs are preserved. The caller owns durable publication.
    /// Legacy version-one archives use `IndexedReader::write_with_native_cache`.
    #[cfg(feature = "native")]
    pub fn write_with_native_cache<W: std::io::Write + Seek>(
        &mut self,
        writer: W,
        mut progress: impl FnMut(&crate::kernel::CompilationProgress) -> ControlFlow<()>,
    ) -> Result<(W, ProgramArchiveCatalogue), KernelError> {
        let source = self.catalogue.source_identity.clone().ok_or_else(|| {
            failure("legacy archive refresh requires IndexedReader::write_with_native_cache")
        })?;
        let started = std::time::Instant::now();
        let total = self
            .catalogue
            .recipes
            .iter()
            .map(|recipe| recipe.records.len())
            .sum();
        let mut writer = super::ProgramArchiveWriter::new(
            writer,
            source,
            self.catalogue.recipes.iter().map(|recipe| recipe.recipe),
        )?;
        let mut completed = 0;
        for recipe in &self.catalogue.recipes {
            for record in &recipe.records {
                if progress(&crate::kernel::CompilationProgress {
                    completed,
                    total,
                    elapsed_seconds: started.elapsed().as_secs_f64(),
                })
                .is_break()
                {
                    return Err(KernelError::Cancelled);
                }
                let kernels = record_reader::read_record(&mut self.reader, record, self.options)?;
                let bytes = super::super::super::cached::refresh(&kernels)?;
                let mut receipt = record.receipt.clone();
                receipt.length = bytes.len() as u64;
                receipt.digest = blake3::hash(&bytes).to_hex().to_string();
                writer.append_record(recipe.recipe, &mut bytes.as_slice(), receipt)?;
                completed += 1;
            }
        }
        let (writer, catalogue) = writer.finish()?;
        if catalogue.content_id != self.catalogue.content_id {
            return Err(failure(
                "native cache refresh changed mathematical family identity",
            ));
        }
        if progress(&crate::kernel::CompilationProgress {
            completed,
            total,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        })
        .is_break()
        {
            return Err(KernelError::Cancelled);
        }
        Ok((writer, catalogue))
    }

    pub fn from_reader(mut reader: R, options: KernelLoadOptions) -> Result<Self, KernelError> {
        reader.seek(SeekFrom::Start(0)).map_err(failure)?;
        let mut header = vec![0; super::MAGIC.len()];
        reader.read_exact(&mut header).map_err(failure)?;
        if header == super::super::MAGIC {
            let legacy = IndexedReader::from_reader(reader, options)?;
            let catalogue =
                ProgramArchiveCatalogue::from_legacy(legacy.catalogue().clone(), options.validate)?;
            catalogue.validate(options.validate)?;
            return Ok(Self {
                reader: legacy.into_inner(),
                catalogue,
                options,
            });
        }
        if header != super::MAGIC {
            return Err(failure("unsupported program archive header"));
        }
        let (catalogue, start): (ProgramArchiveCatalogue, _) =
            transport::read_footer(&mut reader, super::FOOTER)?;
        if catalogue.version != 2 || catalogue.records_end != start {
            return Err(failure(
                "program directory extent/version differs from transport",
            ));
        }
        catalogue.validate(options.validate)?;
        Ok(Self {
            reader,
            catalogue,
            options,
        })
    }
    pub fn catalogue(&self) -> &ProgramArchiveCatalogue {
        &self.catalogue
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
    /// Borrow a selected directory. No evaluator is decoded until a load method
    /// is called on this view; other recipes remain entirely on disk.
    pub fn select(
        &mut self,
        recipe: ProgramRecipe,
    ) -> Result<SelectedProgramReader<'_, R>, KernelError> {
        let catalogue = self.catalogue.recipe(recipe)?;
        Ok(SelectedProgramReader {
            reader: &mut self.reader,
            catalogue,
            options: self.options,
            source_identity: self.catalogue.source_identity.as_deref(),
        })
    }
}

/// Lightweight view of one recipe. Source IDs, exact offsets, coefficient maps
/// and pilot/check metadata are scoped to this selected recipe.
pub struct SelectedProgramReader<'a, R> {
    reader: &'a mut R,
    catalogue: &'a ProgramRecipeCatalogue,
    options: KernelLoadOptions,
    source_identity: Option<&'a str>,
}
impl<R: Read + Seek> SelectedProgramReader<'_, R> {
    pub fn catalogue(&self) -> &ProgramRecipeCatalogue {
        self.catalogue
    }
    pub fn load_record(&mut self, record: usize) -> Result<KernelSet, KernelError> {
        let descriptor = self
            .catalogue
            .records
            .get(record)
            .ok_or_else(|| failure(format!("unknown recipe record {record}")))?;
        record_reader::read_record(self.reader, descriptor, self.options)
    }
    pub fn load_sector(&mut self, sector: usize) -> Result<KernelSet, KernelError> {
        let descriptor = self.catalogue.sector(sector)?;
        record_reader::read_record(self.reader, descriptor, self.options)
    }
    pub fn load_all(&mut self) -> Result<KernelSet, KernelError> {
        self.load_all_with_progress(&mut |_| ControlFlow::Continue(()))
    }
    pub fn load_all_with_progress(
        &mut self,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        let mut kernels = self.load(false, progress)?;
        // Resident owners already retain portable programs. Keep only this
        // recipe's original records, so a later to_bytes cannot silently drop
        // its mathematical/source identity by writing a version-one archive.
        kernels.portable_artifact = Some(self.selected_bytes()?.into());
        Ok(kernels)
    }
    /// Compact exact aggregation. Validate each selected exact record's contour
    /// pilot first when required; aggregation intentionally drops its rich maps.
    pub fn load_exact(&mut self) -> Result<KernelSet, KernelError> {
        self.load(true, &mut |_| ControlFlow::Continue(()))
    }
    fn load(
        &mut self,
        exact: bool,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        record_reader::load_selected(
            self.reader,
            RecordSelection {
                content_id: &self.catalogue.content_id,
                orders: &self.catalogue.orders,
                components: &self.catalogue.components,
                records: &self.catalogue.records,
            },
            self.options,
            exact,
            progress,
        )
    }

    fn selected_bytes(&mut self) -> Result<Vec<u8>, KernelError> {
        if let Some(source_identity) = self.source_identity {
            let mut writer = super::ProgramArchiveWriter::new(
                Cursor::new(Vec::new()),
                source_identity.to_owned(),
                [self.catalogue.recipe],
            )?;
            for record in &self.catalogue.records {
                self.reader
                    .seek(SeekFrom::Start(record.offset))
                    .map_err(failure)?;
                writer.append_record(self.catalogue.recipe, self.reader, record.receipt.clone())?;
            }
            let (writer, catalogue) = writer.finish()?;
            if catalogue.recipe(self.catalogue.recipe)?.content_id != self.catalogue.content_id {
                return Err(failure(
                    "selected recipe changed identity while retaining portable records",
                ));
            }
            Ok(writer.into_inner())
        } else {
            let mut writer = super::super::IndexedWriter::new(Cursor::new(Vec::new()))?;
            for record in &self.catalogue.records {
                self.reader
                    .seek(SeekFrom::Start(record.offset))
                    .map_err(failure)?;
                writer.append_record(self.reader, record.receipt.clone())?;
            }
            let (writer, catalogue) = writer.finish()?;
            if catalogue.content_id != self.catalogue.content_id {
                return Err(failure(
                    "legacy recipe changed identity while retaining portable records",
                ));
            }
            Ok(writer.into_inner())
        }
    }
}
