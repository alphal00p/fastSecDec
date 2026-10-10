//! Preserve the requested recipe's original records and numerical objects.
use super::super::{assembly::ResidentAssembly, failure, writer::for_each_unit_record};
use super::{ProgramArchiveCatalogue, ProgramArchiveWriter, ProgramRecipe};
use crate::kernel::{KernelError, KernelSet};
use std::io::{Cursor, Seek, Write};
#[cfg(test)]
mod tests;

pub(crate) struct ProgramResidentAssembly {
    recipe: ProgramRecipe,
    portable: ProgramArchiveWriter<Cursor<Vec<u8>>>,
    native: ResidentAssembly,
}
impl ProgramResidentAssembly {
    pub(crate) fn new(source_identity: String, recipe: ProgramRecipe) -> Result<Self, KernelError> {
        Ok(Self {
            recipe,
            portable: ProgramArchiveWriter::new(
                Cursor::new(Vec::new()),
                source_identity,
                [recipe],
            )?,
            native: ResidentAssembly::default(),
        })
    }
    /// Serialize each native partition once into both caller storage and the
    /// selected-only portable owner. No other recipe's bytes are retained.
    pub(crate) fn append_unit<W: Write + Seek>(
        &mut self,
        archive: &mut ProgramArchiveWriter<W>,
        kernels: KernelSet,
        sources: &[usize],
    ) -> Result<(), KernelError> {
        if kernels.program_recipe() != self.recipe || kernels.template_content_id.is_some() {
            return Err(failure("resident unit is not the requested unbound recipe"));
        }
        for_each_unit_record(&kernels, sources, |bytes, receipt| {
            let mut primary = bytes;
            archive.append_record(self.recipe, &mut primary, receipt.clone())?;
            let mut retained = bytes;
            self.portable
                .append_record(self.recipe, &mut retained, receipt)
        })?;
        self.native.push(kernels, sources, false)
    }
    pub(crate) fn finish(self, full: &ProgramArchiveCatalogue) -> Result<KernelSet, KernelError> {
        let (portable, selected) = self.portable.finish()?;
        let complete = full.recipe(self.recipe)?;
        let retained = selected.recipe(self.recipe)?;
        if selected.source_identity != full.source_identity
            || retained.content_id != complete.content_id
        {
            return Err(failure(
                "resident records differ from the completed program archive",
            ));
        }
        let mut kernels = self.native.finish(
            retained.content_id.clone(),
            &retained.orders,
            &retained.components,
            false,
        )?;
        if kernels.sectors.len() != retained.sector_count() {
            return Err(failure("resident sector count differs from its archive"));
        }
        for index in 0..kernels.sectors.len() {
            let sources = kernels
                .metadata
                .as_ref()
                .map(|metadata| {
                    metadata
                        .charts
                        .iter()
                        .filter(|chart| chart.kernel_sector == Some(index))
                        .map(|chart| chart.source_index)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if sources != retained.sector(index)?.receipt.source_indices {
                return Err(failure(
                    "resident sector order differs from its canonical archive",
                ));
            }
        }
        kernels.portable_artifact = Some(portable.into_inner());
        Ok(kernels)
    }
}
