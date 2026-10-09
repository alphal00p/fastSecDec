//! Recipe-addressable transport. Selection precedes native decoding/JIT, and
//! every active numerical owner still contains exactly one mathematical recipe.
mod catalogue;
mod reader;
#[cfg(test)]
mod tests;
mod writer;

pub use catalogue::{ProgramArchiveCatalogue, ProgramRecipe, ProgramRecipeCatalogue};
pub use reader::{ProgramArchiveReader, SelectedProgramReader};
pub use writer::ProgramArchiveWriter;

pub(crate) const MAGIC: &[u8] = b"FastSecDec\0indexed\x02";
const FOOTER: &[u8] = b"FastSecDec\0catalogue\x02";

/// Standalone native owner restoration requires an unambiguous single recipe.
/// Full archives are opened through ProgramArchiveReader and selected explicitly.
pub(crate) fn from_bytes(
    bytes: &[u8],
    options: crate::kernel::KernelLoadOptions,
    progress: &mut impl FnMut(&crate::kernel::KernelLoadProgress) -> std::ops::ControlFlow<()>,
) -> Result<crate::kernel::KernelSet, crate::kernel::KernelError> {
    let mut reader = ProgramArchiveReader::from_reader(std::io::Cursor::new(bytes), options)?;
    if reader.catalogue().recipes.len() != 1 {
        return Err(super::failure(
            "multi-recipe artifact requires explicit ProgramArchiveReader::select before loading",
        ));
    }
    let recipe = reader.catalogue().recipes[0].recipe;
    reader.select(recipe)?.load_all_with_progress(progress)
}
