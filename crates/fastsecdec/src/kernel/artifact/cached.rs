//! Optional native JIT transport around an unchanged mathematical record.
//! The owner codec reconstructs callbacks and native machine code; no process
//! addresses, mutable workspaces or FastSecDec pilot state are persisted.
use crate::kernel::{
    CompilationProgress, KernelError, KernelLoadOptions, KernelSet, evaluator::SavedPrimary,
};
use std::ops::ControlFlow;

pub(super) const MAGIC: &[u8] = b"FastSecDec\0binserde\x0d";

#[derive(bincode::Encode)]
struct RecordRef<'a> {
    content_id: &'a str,
    base: &'a [u8],
    primaries: &'a [Option<SavedPrimary>],
}

#[derive(bincode::BorrowDecode)]
struct Record<'a> {
    content_id: &'a str,
    base: &'a [u8],
    primaries: Vec<Option<SavedPrimary>>,
}

fn failure(error: impl std::fmt::Display) -> KernelError {
    KernelError::Artifact(format!("native primary cache: {error}"))
}

pub(super) fn wrap(
    content_id: &str,
    base: Vec<u8>,
    primaries: Vec<Option<SavedPrimary>>,
) -> Result<Vec<u8>, KernelError> {
    if primaries.iter().all(Option::is_none) {
        return Ok(base);
    }
    encode(content_id, &base, &primaries)
}

fn encode(
    content_id: &str,
    base: &[u8],
    primaries: &[Option<SavedPrimary>],
) -> Result<Vec<u8>, KernelError> {
    let mut bytes = MAGIC.to_vec();
    bincode::encode_into_std_write(
        RecordRef {
            content_id,
            base,
            primaries,
        },
        &mut bytes,
        bincode::config::standard(),
    )
    .map_err(failure)?;
    Ok(bytes)
}

fn decode(bytes: &[u8]) -> Result<Record<'_>, KernelError> {
    let wire = bytes
        .strip_prefix(MAGIC)
        .ok_or_else(|| failure("missing header"))?;
    let (record, used): (Record<'_>, _) =
        bincode::borrow_decode_from_slice(wire, bincode::config::standard()).map_err(failure)?;
    if used != wire.len()
        || !record.base.starts_with(super::binary::PREFIX)
        || record.base.starts_with(MAGIC)
    {
        return Err(failure("trailing bytes or nested/non-native base record"));
    }
    Ok(record)
}

#[cfg(test)]
pub(super) fn base(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(MAGIC) {
        decode(bytes).unwrap().base
    } else {
        bytes
    }
}

pub(super) fn load(
    bytes: &[u8],
    options: KernelLoadOptions,
    progress: &mut impl FnMut(&CompilationProgress) -> ControlFlow<()>,
) -> Result<KernelSet, KernelError> {
    load_with_retention(bytes, options, true, progress)
}

pub(super) fn load_with_retention(
    bytes: &[u8],
    options: KernelLoadOptions,
    retain: bool,
    progress: &mut impl FnMut(&CompilationProgress) -> ControlFlow<()>,
) -> Result<KernelSet, KernelError> {
    let record = decode(bytes)?;
    let mut kernels = super::binary::load_with_primary(
        record.base,
        options,
        Some(record.primaries),
        Some(record.content_id),
        false,
        progress,
    )?;
    if retain {
        kernels.portable_artifact = Some(bytes.to_vec().into());
    }
    Ok(kernels)
}

/// Refresh only the transport of an already admitted standalone record. Exact
/// metadata and programs are not regenerated, reserialized or given new IDs.
#[cfg(feature = "native")]
pub(super) fn refresh(kernels: &KernelSet) -> Result<Vec<u8>, KernelError> {
    let original = kernels.artifact_bytes()?;
    let cached;
    let base = if original.starts_with(MAGIC) {
        cached = decode(original)?;
        cached.base
    } else {
        original
    };
    if !base.starts_with(super::binary::PREFIX) {
        return Err(failure("refresh requires a standalone native record"));
    }
    let primaries = kernels
        .sectors
        .iter()
        .map(|sector| sector.saved_primary())
        .collect::<Result<Vec<_>, _>>()?;
    if primaries.iter().all(Option::is_none) {
        return Ok(base.to_vec());
    }
    encode(kernels.template_content_id(), base, &primaries)
}

#[cfg(all(test, feature = "native"))]
mod tests;
