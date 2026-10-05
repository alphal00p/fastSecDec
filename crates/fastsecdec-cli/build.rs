use std::{env, path::PathBuf, process::Command};

mod build_support;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    for (name, relative) in [
        (
            "FEYNKIT",
            "DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/feynkit-fastsecdec-notebook",
        ),
        (
            "SYMBOLICA",
            "DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica",
        ),
        ("NUMERICA", "DO_NOT_PUSH_FOR_REFERENCE_ONLY/numerica"),
    ] {
        let path = root.join(relative);
        let git = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(args)
                .output()
                .ok()
                .filter(|output| output.status.success())
        };
        let revision = git(&["rev-parse", "HEAD"])
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
            .unwrap_or_else(|| "unavailable".into());
        let source = build_support::source_state(&path)
            .unwrap_or_else(|error| panic!("cannot record {name} dependency provenance: {error}"));
        println!("cargo:rustc-env=FASTSECDEC_{name}_REVISION={revision}");
        println!(
            "cargo:rustc-env=FASTSECDEC_{name}_STATE={}",
            source.identity
        );
        for file in source.untracked_files {
            println!("cargo:rerun-if-changed={}", file.display());
        }
        if let Some(output) = git(&["rev-parse", "--git-path", "HEAD"]) {
            println!(
                "cargo:rerun-if-changed={}",
                path.join(String::from_utf8_lossy(&output.stdout).trim())
                    .display()
            );
        }
        for metadata in ["index", "packed-refs"] {
            if let Some(output) = git(&["rev-parse", "--git-path", metadata]) {
                let metadata = path.join(String::from_utf8_lossy(&output.stdout).trim());
                // Missing watched files make Cargo rerun on every invocation.
                if metadata.exists() {
                    println!("cargo:rerun-if-changed={}", metadata.display());
                }
            }
        }
        if let Some(output) = git(&["symbolic-ref", "-q", "HEAD"])
            && let Some(reference) = git(&[
                "rev-parse",
                "--git-path",
                String::from_utf8_lossy(&output.stdout).trim(),
            ])
        {
            let reference = path.join(String::from_utf8_lossy(&reference.stdout).trim());
            // A packed branch has no loose ref yet. Its nearest existing
            // directory catches creation of a loose ref by a subsequent commit.
            if let Some(watched) = reference.ancestors().find(|entry| entry.exists()) {
                println!("cargo:rerun-if-changed={}", watched.display());
            }
        }
        // Reference sources are local path dependencies; Cargo detects their
        // recompilation, while these paths refresh embedded provenance too.
        // Symbolica's native Graphica workspace member lives under `lib`.
        // Watch build inputs too; a changed tracked module also exposes new
        // untracked source files to the next provenance scan.
        for input in [
            "src",
            "lib",
            "crates",
            "Cargo.toml",
            "Cargo.lock",
            "build.rs",
            ".gitignore",
        ] {
            let input = path.join(input);
            if input.exists() {
                println!("cargo:rerun-if-changed={}", input.display());
            }
        }
    }
}
