use super::{
    KernelCatalogue, failure,
    record_reader::{self, RecordSelection},
};
use crate::kernel::{KernelError, KernelLoadOptions, KernelLoadProgress, KernelSet};
use std::{
    io::{Read, Seek, SeekFrom},
    ops::ControlFlow,
};

/// A catalogue and a seekable source, with no retained evaluator or rich chart
/// metadata. Loading a record reads and restores only its own native payload.
pub struct IndexedReader<R> {
    reader: R,
    catalogue: KernelCatalogue,
    options: KernelLoadOptions,
}
impl<R: Read + Seek> IndexedReader<R> {
    /// Refresh optional native JIT payloads one record at a time. Mathematical
    /// identities and source/output mappings remain unchanged. This does not
    /// generate expressions, bind parameters or evaluate samples. The caller
    /// owns staging and atomic publication; cancellation leaves partial output.
    #[cfg(feature = "native")]
    pub fn write_with_native_cache<W: std::io::Write + Seek>(
        &mut self,
        writer: W,
        mut progress: impl FnMut(&crate::kernel::CompilationProgress) -> ControlFlow<()>,
    ) -> Result<(W, KernelCatalogue), KernelError> {
        let started = std::time::Instant::now();
        let total = self.catalogue.records.len();
        let mut writer = super::IndexedWriter::new(writer)?;
        for (index, record) in self.catalogue.records.iter().enumerate() {
            if progress(&crate::kernel::CompilationProgress {
                completed: index,
                total,
                elapsed_seconds: started.elapsed().as_secs_f64(),
            })
            .is_break()
            {
                return Err(KernelError::Cancelled);
            }
            let kernels = record_reader::read_record(&mut self.reader, record, self.options)?;
            let bytes = super::super::cached::refresh(&kernels)?;
            let mut receipt = record.receipt.clone();
            receipt.length = bytes.len() as u64;
            receipt.digest = blake3::hash(&bytes).to_hex().to_string();
            writer.append_record(&mut bytes.as_slice(), receipt)?;
        }
        let (writer, catalogue) = writer.finish()?;
        if catalogue.content_id != self.catalogue.content_id {
            return Err(failure(
                "native cache refresh changed mathematical catalogue identity",
            ));
        }
        if progress(&crate::kernel::CompilationProgress {
            completed: total,
            total,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        })
        .is_break()
        {
            return Err(KernelError::Cancelled);
        }
        Ok((writer, catalogue))
    }

    pub fn new(
        mut reader: R,
        catalogue: KernelCatalogue,
        options: KernelLoadOptions,
    ) -> Result<Self, KernelError> {
        catalogue.validate(options.validate)?;
        reader.seek(SeekFrom::Start(0)).map_err(failure)?;
        let mut header = vec![0u8; super::MAGIC.len()];
        reader.read_exact(&mut header).map_err(failure)?;
        let length = reader.seek(SeekFrom::End(0)).map_err(failure)?;
        let trailer = (super::FOOTER.len() + 8) as u64;
        if header != super::MAGIC || length < catalogue.records_end.saturating_add(trailer) {
            return Err(failure("missing header or truncated indexed data"));
        }
        reader
            .seek(SeekFrom::Start(length - trailer))
            .map_err(failure)?;
        let mut count = [0u8; 8];
        reader.read_exact(&mut count).map_err(failure)?;
        let mut footer = vec![0u8; super::FOOTER.len()];
        reader.read_exact(&mut footer).map_err(failure)?;
        if footer != super::FOOTER
            || u64::from_le_bytes(count) != length - trailer - catalogue.records_end
        {
            return Err(failure("incomplete indexed catalogue commit"));
        }
        Ok(Self {
            reader,
            catalogue,
            options,
        })
    }
    /// Read just the bounded footer index, without decoding any native record.
    pub fn from_reader(mut reader: R, options: KernelLoadOptions) -> Result<Self, KernelError> {
        let length = reader.seek(SeekFrom::End(0)).map_err(failure)?;
        let trailer = (super::FOOTER.len() + 8) as u64;
        let footer_start = length
            .checked_sub(trailer)
            .ok_or_else(|| failure("truncated indexed footer"))?;
        reader
            .seek(SeekFrom::Start(footer_start))
            .map_err(failure)?;
        let mut count = [0u8; 8];
        reader.read_exact(&mut count).map_err(failure)?;
        let mut magic = vec![0; super::FOOTER.len()];
        reader.read_exact(&mut magic).map_err(failure)?;
        let count = u64::from_le_bytes(count);
        if magic != super::FOOTER || count > 128 * 1024 * 1024 {
            return Err(failure("invalid or excessive catalogue footer"));
        }
        let start = footer_start
            .checked_sub(count)
            .ok_or_else(|| failure("catalogue range underflow"))?;
        reader.seek(SeekFrom::Start(start)).map_err(failure)?;
        let mut bytes = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).map_err(failure)?;
        let catalogue: KernelCatalogue = serde_json::from_slice(&bytes)?;
        if catalogue.records_end != start {
            return Err(failure("catalogue extent differs from trailer"));
        }
        Self::new(reader, catalogue, options)
    }
    pub fn catalogue(&self) -> &KernelCatalogue {
        &self.catalogue
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
    pub fn load_record(&mut self, index: usize) -> Result<KernelSet, KernelError> {
        let descriptor = self
            .catalogue
            .records
            .get(index)
            .ok_or_else(|| failure(format!("unknown record {index}")))?;
        record_reader::read_record(&mut self.reader, descriptor, self.options)
    }
    /// Return the original local output layout. `catalogue().sector(id)` supplies
    /// the map into the complete Laurent vector; no parent evaluator is retained.
    pub fn load_sector(&mut self, sector: usize) -> Result<KernelSet, KernelError> {
        let index = self
            .catalogue
            .records
            .iter()
            .position(|r| r.sector == Some(sector))
            .ok_or_else(|| failure(format!("unknown sector {sector}")))?;
        self.load_record(index)
    }
    pub fn load_all(&mut self) -> Result<KernelSet, KernelError> {
        self.load_all_with_progress(&mut |_| ControlFlow::Continue(()))
    }
    /// Restore only exact-offset/constraint records. No numerical-sector record
    /// is read or compiled. Call `bind_parameters` on this small native owner in
    /// the caller's setup process, then return its compact numerical manifest.
    pub fn load_exact(&mut self) -> Result<KernelSet, KernelError> {
        self.load_with_progress(true, &mut |_| ControlFlow::Continue(()))
    }
    /// Ordinary resident loading, with only numerical output scattering when
    /// records have different local layouts. Never rewrites or optimizes IR.
    pub fn load_all_with_progress(
        &mut self,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.load_with_progress(false, progress)
    }
    fn load_with_progress(
        &mut self,
        exact_only: bool,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        record_reader::load_selected(
            &mut self.reader,
            RecordSelection {
                content_id: &self.catalogue.content_id,
                orders: &self.catalogue.orders,
                components: &self.catalogue.components,
                records: &self.catalogue.records,
            },
            self.options,
            exact_only,
            progress,
        )
    }
}
