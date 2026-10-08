//! Small envelopes around the existing native context-aware Atom codec.
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    state::{State, StateMap},
};

const MAGIC: &[u8] = b"FastSecDec\0generation-record\x01";
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error(transparent)]
    Generation(#[from] super::super::GenerationError),
    #[error("generation record I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid generation record: {0}")]
    Invalid(String),
}
pub(super) fn invalid(error: impl std::fmt::Display) -> StreamingError {
    StreamingError::Invalid(error.to_string())
}

/// Relative to the caller's staging root. Immutable content identity is checked
/// before native decoding; this is not a format for untrusted native programs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRef {
    pub path: String,
    pub blake3: String,
    pub bytes: u64,
}
impl RecordRef {
    pub fn verify(&self, root: &Path) -> Result<(), StreamingError> {
        let mut file = File::open(self.resolve(root)?)?;
        if file.metadata()?.len() != self.bytes {
            return Err(invalid("record length mismatch"));
        }
        let mut digest = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            digest.update(&buffer[..n]);
        }
        if digest.finalize().to_hex().as_str() != self.blake3 {
            return Err(invalid("record digest mismatch"));
        }
        Ok(())
    }
    pub fn resolve(&self, root: &Path) -> Result<PathBuf, StreamingError> {
        let path = Path::new(&self.path);
        if path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(invalid("record path must be a relative staging path"));
        }
        Ok(root.join(path))
    }
}

#[derive(Encode, Decode)]
struct Envelope {
    state: Vec<u8>,
    payload: Vec<u8>,
}
#[derive(Encode, Decode)]
#[bincode(decode_context = "StateMap")]
struct Payload {
    kind: String,
    metadata: Vec<u8>,
    atoms: Vec<Atom>,
    symbols: Vec<Symbol>,
}

/// A native Atom table lets the small surrounding DTO use ordinary serde
/// without converting expressions to strings or expanding native aliases.
#[derive(Default)]
pub(super) struct Atoms {
    pub atoms: Vec<Atom>,
}
impl Atoms {
    pub fn push(&mut self, value: &Atom) -> usize {
        let id = self.atoms.len();
        self.atoms.push(value.clone());
        id
    }
    pub fn take(&self, index: usize) -> Result<Atom, StreamingError> {
        self.atoms
            .get(index)
            .cloned()
            .ok_or_else(|| invalid("Atom index out of bounds"))
    }
}

pub(super) fn write<M: Serialize>(
    root: &Path,
    name: &str,
    kind: &str,
    metadata: &M,
    atoms: Atoms,
    symbols: Vec<Symbol>,
) -> Result<RecordRef, StreamingError> {
    fs::create_dir_all(root)?;
    let mut exported = Atom::Zero.get_all_symbols(true);
    for atom in &atoms.atoms {
        exported.extend(atom.get_all_symbols(true));
    }
    exported.extend(symbols.iter().copied());
    let mut state = Vec::new();
    State::export_partial(&mut state, exported)?;
    let payload = Payload {
        kind: kind.into(),
        metadata: serde_json::to_vec(metadata).map_err(invalid)?,
        atoms: atoms.atoms,
        symbols,
    };
    let payload = bincode::encode_to_vec(payload, bincode::config::standard()).map_err(invalid)?;
    let envelope = Envelope { state, payload };
    let mut bytes = MAGIC.to_vec();
    bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).map_err(invalid)?);
    let digest = blake3::hash(&bytes).to_hex().to_string();
    let reference = RecordRef {
        path: format!("{name}-{digest}.bin"),
        blake3: digest,
        bytes: bytes.len() as u64,
    };
    let destination = reference.resolve(root)?;
    if destination.exists() {
        reference.verify(root)?;
        return Ok(reference);
    }
    let temp = root.join(format!(
        ".record-{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let outcome = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, &destination)?;
        File::open(root)?.sync_all()?;
        Ok::<_, std::io::Error>(())
    })();
    if outcome.is_err() {
        let _ = fs::remove_file(&temp);
    }
    outcome?;
    Ok(reference)
}

pub(super) fn read<M: DeserializeOwned>(
    root: &Path,
    reference: &RecordRef,
    kind: &str,
) -> Result<(M, Atoms, Vec<Symbol>), StreamingError> {
    reference.verify(root)?;
    let bytes = fs::read(reference.resolve(root)?)?;
    let bytes = bytes
        .strip_prefix(MAGIC)
        .ok_or_else(|| invalid("unsupported record version"))?;
    let (envelope, used): (Envelope, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard()).map_err(invalid)?;
    if used != bytes.len() {
        return Err(invalid("trailing envelope bytes"));
    }
    let _ = symbolica::transcendental::gamma();
    let mut source = envelope.state.as_slice();
    let state = State::import(&mut source, None)?;
    if !source.is_empty() {
        return Err(invalid("trailing native context bytes"));
    }
    let (payload, used): (Payload, usize) = bincode::decode_from_slice_with_context(
        &envelope.payload,
        bincode::config::standard(),
        state,
    )
    .map_err(invalid)?;
    if used != envelope.payload.len() || payload.kind != kind {
        return Err(invalid("record kind or payload length mismatch"));
    }
    Ok((
        serde_json::from_slice(&payload.metadata).map_err(invalid)?,
        Atoms {
            atoms: payload.atoms,
        },
        payload.symbols,
    ))
}
