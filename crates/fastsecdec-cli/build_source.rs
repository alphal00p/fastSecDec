//! Content identity for native generation staging and worker handshakes.
//! Documentation, output and Git bookkeeping must not invalidate recovery.
use std::{fs, io, path::Path};

pub fn identity(workspace: &Path) -> io::Result<String> {
    let mut hash = blake3::Hasher::new();
    for name in ["Cargo.toml", "Cargo.lock"] {
        hash_path(workspace, &workspace.join(name), &mut hash)?;
    }
    for name in [
        "fastsecdec",
        "fastsecdec-cli",
        "fastsecdec-qmc",
        "fastsecdec-sectors",
    ] {
        let root = workspace.join("crates").join(name);
        for entry in [
            "Cargo.toml",
            "src",
            "build.rs",
            "build_support.rs",
            "build_source.rs",
        ] {
            let path = root.join(entry);
            if path.exists() {
                // Watching each source directory catches newly introduced modules.
                println!("cargo:rerun-if-changed={}", path.display());
                hash_path(workspace, &path, &mut hash)?;
            }
        }
    }
    Ok(hash.finalize().to_hex().to_string())
}

fn hash_path(root: &Path, path: &Path, hash: &mut blake3::Hasher) -> io::Result<()> {
    if path.is_dir() {
        let mut children = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?;
        children.sort();
        for child in children {
            hash_path(root, &child, hash)?;
        }
    } else {
        let relative = path
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .to_string_lossy();
        let bytes = fs::read(path)?;
        hash.update(&(relative.len() as u64).to_le_bytes());
        hash.update(relative.as_bytes());
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(&bytes);
    }
    Ok(())
}
