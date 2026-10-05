use std::{env, path::PathBuf, process::Command};

mod build_support;

fn main() {
    // One generated Cargo config supplies both dependency patches and source roots.
    // CLI artifact provenance must never collapse distinct owners to an unknown identity.
    for name in ["FEYNKIT", "SYMBOLICA", "NUMERICA"] {
        let variable = format!("FASTSECDEC_{name}_SOURCE_ROOT");
        println!("cargo:rerun-if-env-changed={variable}");
        let path = env::var_os(&variable).map(PathBuf::from).unwrap_or_else(|| {
            panic!("{variable} is required for CLI dependency provenance; prepare the pinned dependency overlay and pass its generated config with cargo --config /path/to/overlay.toml")
        });
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
            .unwrap_or_else(|| {
                panic!(
                    "cannot read {name} dependency revision at {}; use the source root from the generated dependency config",
                    path.display()
                )
            });
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
        // Configured owners are Cargo path patches; Cargo detects their
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
