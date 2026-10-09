use super::super::{RecordDescriptor, RecordReceipt, failure, transport};
use super::{ProgramArchiveCatalogue, ProgramRecipe, ProgramRecipeCatalogue};
use crate::kernel::KernelError;
use std::{
    collections::BTreeMap,
    io::{Read, Seek, Write},
};

/// Caller-owned append-only storage for a declared complete recipe set. Records
/// are native worker output, copied in bounded blocks without decoding or JIT.
/// A failed append poisons this staging writer. The caller owns durable atomic
/// publication, just as with the version-one writer.
pub struct ProgramArchiveWriter<W> {
    writer: W,
    source_identity: String,
    recipes: BTreeMap<ProgramRecipe, Vec<RecordDescriptor>>,
    failed: bool,
}
impl<W: Write + Seek> ProgramArchiveWriter<W> {
    pub fn new(
        mut writer: W,
        source_identity: String,
        recipes: impl IntoIterator<Item = ProgramRecipe>,
    ) -> Result<Self, KernelError> {
        super::super::super::validate_content_id(&source_identity)?;
        let mut requested = BTreeMap::new();
        for recipe in recipes {
            if requested.insert(recipe, Vec::new()).is_some() {
                return Err(failure("duplicate requested recipe"));
            }
        }
        if requested.is_empty() || writer.stream_position().map_err(failure)? != 0 {
            return Err(failure(
                "program writer requires recipes and must start at byte zero",
            ));
        }
        writer.write_all(super::MAGIC).map_err(failure)?;
        Ok(Self {
            writer,
            source_identity,
            recipes: requested,
            failed: false,
        })
    }
    pub fn append_record(
        &mut self,
        recipe: ProgramRecipe,
        reader: &mut impl Read,
        receipt: RecordReceipt,
    ) -> Result<(), KernelError> {
        if self.failed {
            return Err(failure("cannot append to a failed archive writer"));
        }
        let records = self
            .recipes
            .get_mut(&recipe)
            .ok_or_else(|| failure("record recipe was not requested"))?;
        receipt.validate()?;
        if receipt.program_recipe() != recipe {
            return Err(failure(
                "record runtime schema differs from its advertised recipe",
            ));
        }
        self.failed = true;
        let offset = self.writer.stream_position().map_err(failure)?;
        transport::copy_record(&mut self.writer, reader, &receipt)?;
        records.push(RecordDescriptor {
            offset,
            receipt,
            output_indices: Vec::new(),
            sector: None,
        });
        self.failed = false;
        Ok(())
    }
    pub fn finish(mut self) -> Result<(W, ProgramArchiveCatalogue), KernelError> {
        if self.failed {
            return Err(failure("cannot finish a failed archive writer"));
        }
        let end = self.writer.stream_position().map_err(failure)?;
        let recipes = self
            .recipes
            .into_iter()
            .map(|(recipe, records)| {
                ProgramRecipeCatalogue::finish(recipe, records, &self.source_identity)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let catalogue = ProgramArchiveCatalogue::finish(self.source_identity, recipes, end)?;
        transport::write_footer(&mut self.writer, &catalogue, super::FOOTER)?;
        Ok((self.writer, catalogue))
    }
}
