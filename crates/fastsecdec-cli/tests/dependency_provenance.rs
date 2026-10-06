#[path = "../build_support.rs"]
mod build_support;

use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn base() {}\n").unwrap();
    fs::write(root.join(".gitignore"), "/target/\n").unwrap();
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=Codex",
            "-c",
            "user.email=codex@openai.com",
            "commit",
            "-qm",
            "baseline",
        ],
    );
    directory
}

#[test]
fn newly_added_module_contents_change_dependency_identity() {
    let directory = repository();
    let root = directory.path();
    assert_eq!(build_support::source_state(root).unwrap().identity, "clean");
    fs::write(root.join("src/lib.rs"), "pub mod added;\n").unwrap();
    let tracked_only = build_support::source_state(root).unwrap().identity;
    let module = root.join("src/added.rs");
    fs::write(&module, "pub const VALUE: u32 = 1;\n").unwrap();
    let first = build_support::source_state(root).unwrap();
    assert_ne!(tracked_only, first.identity);
    assert_eq!(first.untracked_files, vec![module.clone()]);
    fs::write(&module, "pub const VALUE: u32 = 2;\n").unwrap();
    assert_ne!(
        first.identity,
        build_support::source_state(root).unwrap().identity
    );
    fs::remove_file(module).unwrap();
    assert_eq!(
        tracked_only,
        build_support::source_state(root).unwrap().identity
    );
    git(root, &["checkout", "--", "src/lib.rs"]);
    assert_eq!(build_support::source_state(root).unwrap().identity, "clean");
}

#[test]
fn ignored_build_outputs_do_not_change_identity_and_staged_edits_do() {
    let directory = repository();
    let root = directory.path();
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/output"), "build product").unwrap();
    assert_eq!(build_support::source_state(root).unwrap().identity, "clean");
    fs::write(root.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
    let unstaged = build_support::source_state(root).unwrap().identity;
    git(root, &["add", "src/lib.rs"]);
    assert_eq!(
        unstaged,
        build_support::source_state(root).unwrap().identity
    );
    assert_ne!(unstaged, "clean");
}

#[test]
fn missing_git_identity_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    assert!(build_support::source_state(directory.path()).is_err());
}

fn locked_package(source: Option<&str>, checksum: Option<&str>) -> toml::Table {
    let mut package = toml::Table::new();
    package.insert("name".into(), "numerica".into());
    package.insert("version".into(), "3.0.1".into());
    if let Some(source) = source {
        package.insert("source".into(), source.into());
    }
    if let Some(checksum) = checksum {
        package.insert("checksum".into(), checksum.into());
    }
    let mut lock = toml::Table::new();
    lock.insert("package".into(), toml::Value::Array(vec![package.into()]));
    lock
}

#[test]
fn registry_owner_ignores_stale_development_source_and_binds_checksum() {
    let checksum = "a".repeat(64);
    let registry = "registry+https://github.com/rust-lang/crates.io-index";
    let lock = locked_package(Some(registry), Some(&checksum));
    let stale = Path::new("/nonexistent/old-numerica-fork");
    let actual = build_support::dependency_identity(&lock, "numerica", Some(stale)).unwrap();
    assert_eq!(actual.revision, "3.0.1");
    assert_eq!(actual.state, format!("{registry}#checksum={checksum}"));
    assert!(actual.source_root.is_none());
    assert!(actual.untracked_files.is_empty());
    assert_eq!(
        actual.state,
        build_support::dependency_identity(&lock, "numerica", None)
            .unwrap()
            .state
    );
    let changed = locked_package(Some(registry), Some(&"b".repeat(64)));
    assert_ne!(
        actual.state,
        build_support::dependency_identity(&changed, "numerica", None)
            .unwrap()
            .state
    );
}

#[test]
fn git_owner_records_resolved_revision_and_source_without_local_env() {
    let revision = "0123456789abcdef0123456789abcdef01234567";
    let source = format!("git+https://example.org/owner?branch=main#{revision}");
    let lock = locked_package(Some(&source), None);
    let actual =
        build_support::dependency_identity(&lock, "numerica", Some(Path::new("/stale"))).unwrap();
    assert_eq!(actual.revision, revision);
    assert_eq!(actual.state, source);
    assert!(actual.source_root.is_none());
}

#[test]
fn ambiguous_missing_and_malformed_resolved_owners_are_errors() {
    let registry = "registry+https://github.com/rust-lang/crates.io-index";
    let checksum = "a".repeat(64);
    let mut lock = locked_package(Some(registry), Some(&checksum));
    assert!(build_support::dependency_identity(&lock, "missing", None).is_err());
    let packages = lock.get_mut("package").unwrap().as_array_mut().unwrap();
    packages.push(packages[0].clone());
    assert!(build_support::dependency_identity(&lock, "numerica", None).is_err());
    for checksum in [None, Some("short"), Some("z".repeat(64).as_str())] {
        let lock = locked_package(Some(registry), checksum);
        assert!(build_support::dependency_identity(&lock, "numerica", None).is_err());
    }
    for source in [
        "git+https://example.org/repo#main",
        "git+https://example.org/repo",
        "registry+",
        "file:///unrecorded",
    ] {
        let lock = locked_package(Some(source), Some(&checksum));
        assert!(build_support::dependency_identity(&lock, "numerica", None).is_err());
    }
    let mut lock = locked_package(Some(registry), Some(&checksum));
    lock["package"].as_array_mut().unwrap()[0]["version"] = "".into();
    assert!(build_support::dependency_identity(&lock, "numerica", None).is_err());
}

#[test]
fn only_path_owners_use_existing_dirty_source_identity() {
    let directory = repository();
    let lock = locked_package(None, None);
    assert!(build_support::dependency_identity(&lock, "numerica", None).is_err());
    let before =
        build_support::dependency_identity(&lock, "numerica", Some(directory.path())).unwrap();
    assert_eq!(before.state, "clean");
    assert_eq!(before.source_root.as_deref(), Some(directory.path()));
    assert_eq!(before.revision.len(), 40);
    fs::write(directory.path().join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
    let after =
        build_support::dependency_identity(&lock, "numerica", Some(directory.path())).unwrap();
    assert_eq!(after.revision, before.revision);
    assert_ne!(after.state, before.state);
}
