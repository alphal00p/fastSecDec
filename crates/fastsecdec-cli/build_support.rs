use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub struct SourceState {
    pub identity: String,
    pub untracked_files: Vec<PathBuf>,
}

fn git(path: &Path, args: &[&str]) -> io::Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git {} failed in {}: {}",
            args.join(" "),
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}

/// Preserve the existing tracked-patch identity, extending it with all
/// nonignored untracked files. A newly added Rust module is not in `git diff`.
/// Fail on unreadable state instead of declaring an unverified checkout clean.
pub fn source_state(path: &Path) -> io::Result<SourceState> {
    let patch = git(
        path,
        &["diff", "--binary", "--no-ext-diff", "--no-textconv", "HEAD"],
    )?;
    let untracked = git(path, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let mut names = untracked
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    names.sort_unstable();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&patch);
    let mut untracked_files = Vec::with_capacity(names.len());
    for name in names {
        let name_text = std::str::from_utf8(name)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let file = path.join(name_text);
        // Length-delimited paths and contents avoid ambiguous concatenations.
        hasher.update(b"\0fastsecdec-untracked-file-v1\0");
        hash_field(&mut hasher, name);
        if fs::symlink_metadata(&file)?.is_symlink() {
            hasher.update(b"symlink\0");
            let target = fs::read_link(&file)?;
            let target = target.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "non-UTF-8 source symlink target",
                )
            })?;
            hash_field(&mut hasher, target.as_bytes());
        } else {
            hasher.update(b"regular\0");
        }
        hash_field(&mut hasher, &fs::read(&file)?);
        untracked_files.push(file);
    }
    let identity = if patch.is_empty() && untracked_files.is_empty() {
        "clean".to_owned()
    } else {
        format!("dirty:{}", hasher.finalize())
    };
    Ok(SourceState {
        identity,
        untracked_files,
    })
}

fn hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
