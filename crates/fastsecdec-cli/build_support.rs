use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub struct SourceState {
    pub identity: String,
    pub untracked_files: Vec<PathBuf>,
}

pub struct DependencyIdentity {
    pub revision: String,
    pub state: String,
    /// Set only for the path-owner case, whose source state needs watching.
    pub source_root: Option<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
}

/// Identify a unique resolved owner from the consuming CLI workspace's lock.
/// A configured development source root is relevant only for a path package;
/// it must never override a registry or Git identity selected by Cargo.
pub fn dependency_identity(
    lock: &toml::Table,
    package: &str,
    configured_source_root: Option<&Path>,
) -> io::Result<DependencyIdentity> {
    let invalid = |message: String| io::Error::new(io::ErrorKind::InvalidData, message);
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| invalid("Cargo.lock has no package inventory".into()))?;
    let matches = packages
        .iter()
        .filter(|entry| entry.get("name").and_then(toml::Value::as_str) == Some(package))
        .collect::<Vec<_>>();
    let [entry] = matches.as_slice() else {
        return Err(invalid(format!(
            "expected one resolved {package} owner, found {}",
            matches.len()
        )));
    };
    let version = entry
        .get("version")
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid(format!("{package} has no resolved version")))?;
    let hex =
        |value: &str, length| value.len() == length && value.bytes().all(|c| c.is_ascii_hexdigit());
    if let Some(source) = entry.get("source") {
        let source = source
            .as_str()
            .ok_or_else(|| invalid(format!("invalid {package} source")))?;
        let (revision, state) = if source.starts_with("git+") {
            let (repository, revision) = source
                .rsplit_once('#')
                .filter(|(repository, revision)| repository.len() > 4 && hex(revision, 40))
                .ok_or_else(|| invalid(format!("{package} has no full resolved Git revision")))?;
            let _ = repository;
            (revision.to_owned(), source.to_owned())
        } else if source.starts_with("registry+") && source.len() > "registry+".len() {
            let checksum = entry
                .get("checksum")
                .and_then(toml::Value::as_str)
                .filter(|checksum| hex(checksum, 64))
                .ok_or_else(|| invalid(format!("{package} has no valid registry checksum")))?;
            (version.to_owned(), format!("{source}#checksum={checksum}"))
        } else {
            return Err(invalid(format!(
                "unsupported resolved {package} source {source}"
            )));
        };
        return Ok(DependencyIdentity {
            revision,
            state,
            source_root: None,
            untracked_files: vec![],
        });
    }
    let path = configured_source_root.ok_or_else(|| invalid(format!(
        "{package} resolves to a local path; its FASTSECDEC source-root setting is required for development provenance"
    )))?;
    let revision = String::from_utf8(git(path, &["rev-parse", "HEAD"])?)
        .map_err(|error| invalid(error.to_string()))?
        .trim()
        .to_owned();
    if !hex(&revision, 40) {
        return Err(invalid(format!(
            "{package} source root has no full Git revision"
        )));
    }
    let source = source_state(path)?;
    Ok(DependencyIdentity {
        revision,
        state: source.identity,
        source_root: Some(path.to_owned()),
        untracked_files: source.untracked_files,
    })
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
