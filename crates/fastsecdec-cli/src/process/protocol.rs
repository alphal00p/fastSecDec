//! Small, framed control messages. Evaluators and checkpoint payloads use files.
use std::io::{self, Read, Write};

use serde::{Serialize, de::DeserializeOwned};

const MAGIC: &[u8; 8] = b"FSDIPC01";
pub(super) const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Envelope<T> {
    pub run_id: String,
    pub lease_id: u64,
    pub payload: T,
}

/// Check the size before deserializing or allocating an untrusted frame.
pub(crate) fn read<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<Option<T>> {
    let mut magic = [0; 8];
    loop {
        match reader.read(&mut magic[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    reader.read_exact(&mut magic[1..])?;
    if &magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid worker protocol header",
        ));
    }
    let mut length = [0; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worker control frame exceeds size limit; use a staging file",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub(crate) fn write<T: Serialize>(writer: &mut impl Write, value: &T) -> io::Result<()> {
    // Serde writes into a bounded buffer, so an accidentally large control
    // object cannot first allocate an arbitrarily large temporary Vec.
    struct Buffer(Vec<u8>);
    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > MAX_MESSAGE_BYTES.saturating_sub(self.0.len()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "worker control frame exceeds size limit; use a staging file",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut bytes = Buffer(Vec::new());
    serde_json::to_writer(&mut bytes, value).map_err(io::Error::other)?;
    writer.write_all(MAGIC)?;
    writer.write_all(&(bytes.0.len() as u32).to_le_bytes())?;
    writer.write_all(&bytes.0)?;
    writer.flush()
}
