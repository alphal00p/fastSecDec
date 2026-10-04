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
