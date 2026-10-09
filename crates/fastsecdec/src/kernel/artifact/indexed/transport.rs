//! Bounded byte transport shared by indexed archive versions.
use super::{RecordReceipt, failure};
use crate::kernel::KernelError;
use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Seek, SeekFrom, Write};

pub(super) fn copy_record(
    writer: &mut impl Write,
    reader: &mut impl Read,
    receipt: &RecordReceipt,
) -> Result<(), KernelError> {
    let mut remaining = receipt.length;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    while remaining > 0 {
        let requested = remaining.min(buffer.len() as u64) as usize;
        reader
            .read_exact(&mut buffer[..requested])
            .map_err(failure)?;
        writer.write_all(&buffer[..requested]).map_err(failure)?;
        hash.update(&buffer[..requested]);
        remaining -= requested as u64;
    }
    if hash.finalize().to_hex().as_str() != receipt.digest {
        return Err(failure("worker record digest differs from its receipt"));
    }
    Ok(())
}

pub(super) fn write_footer(
    writer: &mut impl Write,
    catalogue: &impl Serialize,
    footer: &[u8],
) -> Result<(), KernelError> {
    let bytes = serde_json::to_vec(catalogue)?;
    if bytes.len() > 128 * 1024 * 1024 {
        return Err(failure("excessive catalogue footer"));
    }
    writer.write_all(&bytes).map_err(failure)?;
    writer
        .write_all(&(bytes.len() as u64).to_le_bytes())
        .map_err(failure)?;
    writer.write_all(footer).map_err(failure)?;
    writer.flush().map_err(failure)
}

pub(super) fn read_footer<T: DeserializeOwned>(
    reader: &mut (impl Read + Seek),
    footer: &[u8],
) -> Result<(T, u64), KernelError> {
    let length = reader.seek(SeekFrom::End(0)).map_err(failure)?;
    let trailer = (footer.len() + 8) as u64;
    let footer_start = length
        .checked_sub(trailer)
        .ok_or_else(|| failure("truncated indexed footer"))?;
    reader
        .seek(SeekFrom::Start(footer_start))
        .map_err(failure)?;
    let mut count = [0u8; 8];
    reader.read_exact(&mut count).map_err(failure)?;
    let mut magic = vec![0; footer.len()];
    reader.read_exact(&mut magic).map_err(failure)?;
    let count = u64::from_le_bytes(count);
    if magic != footer || count > 128 * 1024 * 1024 {
        return Err(failure("invalid or excessive catalogue footer"));
    }
    let start = footer_start
        .checked_sub(count)
        .ok_or_else(|| failure("catalogue range underflow"))?;
    reader.seek(SeekFrom::Start(start)).map_err(failure)?;
    let mut bytes = vec![0u8; count as usize];
    reader.read_exact(&mut bytes).map_err(failure)?;
    Ok((serde_json::from_slice(&bytes)?, start))
}
