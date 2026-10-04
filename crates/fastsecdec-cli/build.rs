use std::{env, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    for (name, relative) in [
        (
            "FEYNKIT",
            "DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/feynkit",
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
        let patch = git(&["diff", "--binary", "HEAD"])
            .map(|output| output.stdout)
            .unwrap_or_default();
        let dirty = if patch.is_empty() {
            "clean".into()
        } else {
            format!("dirty:{}", blake3::hash(&patch))
        };
        println!("cargo:rustc-env=FASTSECDEC_{name}_REVISION={revision}");
        println!("cargo:rustc-env=FASTSECDEC_{name}_STATE={dirty}");
        if let Some(output) = git(&["rev-parse", "--git-path", "HEAD"]) {
            println!(
                "cargo:rerun-if-changed={}",
                path.join(String::from_utf8_lossy(&output.stdout).trim())
                    .display()
            );
        }
        for metadata in ["index", "packed-refs"] {
            if let Some(output) = git(&["rev-parse", "--git-path", metadata]) {
                println!(
                    "cargo:rerun-if-changed={}",
                    path.join(String::from_utf8_lossy(&output.stdout).trim())
                        .display()
                );
            }
        }
        if let Some(output) = git(&["symbolic-ref", "-q", "HEAD"])
            && let Some(reference) = git(&[
                "rev-parse",
                "--git-path",
                String::from_utf8_lossy(&output.stdout).trim(),
            ])
        {
            println!(
                "cargo:rerun-if-changed={}",
                path.join(String::from_utf8_lossy(&reference.stdout).trim())
                    .display()
            );
        }
        // Reference sources are local path dependencies; Cargo detects their
        // recompilation, while these paths refresh embedded provenance too.
        let sources = if name == "FEYNKIT" { "crates" } else { "src" };
        println!("cargo:rerun-if-changed={}", path.join(sources).display());
        println!(
            "cargo:rerun-if-changed={}",
            path.join("Cargo.toml").display()
        );
    }
}
