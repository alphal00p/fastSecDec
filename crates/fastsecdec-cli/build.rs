use std::{env, fs, path::PathBuf, process::Command};

mod build_source;
mod build_support;

fn main() {
    // This binary's consuming workspace owns the resolved lockfile. locate-project
    // only locates it: it does not resolve dependencies, fetch or mutate the lock.
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let located = Command::new(env::var_os("CARGO").unwrap())
        .args([
            "locate-project",
            "--workspace",
            "--message-format",
            "plain",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .expect("cannot locate the CLI workspace");
    assert!(
        located.status.success(),
        "cannot locate the CLI workspace: {}",
        String::from_utf8_lossy(&located.stderr)
    );
    let workspace_manifest = PathBuf::from(String::from_utf8(located.stdout).unwrap().trim());
    let lockfile = workspace_manifest.with_file_name("Cargo.lock");
    let source = build_source::identity(workspace_manifest.parent().unwrap())
        .expect("cannot fingerprint native FastSecDec build inputs");
    println!("cargo:rustc-env=FASTSECDEC_SOURCE_IDENTITY={source}");
    let lock: toml::Table = fs::read_to_string(&lockfile)
        .unwrap_or_else(|error| panic!("cannot read resolved lock {}: {error}", lockfile.display()))
        .parse()
        .expect("cannot parse the CLI workspace Cargo.lock");
    println!("cargo:rerun-if-changed={}", lockfile.display());
    println!("cargo:rerun-if-changed={}", workspace_manifest.display());
    for (name, package) in [
        ("FEYNKIT", "feynkit-graph"),
        ("SYMBOLICA", "symbolica"),
        ("NUMERICA", "numerica"),
    ] {
        let variable = format!("FASTSECDEC_{name}_SOURCE_ROOT");
        println!("cargo:rerun-if-env-changed={variable}");
        let configured = env::var_os(&variable).map(PathBuf::from);
        let identity = build_support::dependency_identity(&lock, package, configured.as_deref())
            .unwrap_or_else(|error| panic!("cannot record {name} dependency provenance: {error}"));
        println!(
            "cargo:rustc-env=FASTSECDEC_{name}_REVISION={}",
            identity.revision
        );
        println!("cargo:rustc-env=FASTSECDEC_{name}_STATE={}", identity.state);
        for file in identity.untracked_files {
            println!("cargo:rerun-if-changed={}", file.display());
        }
        let Some(path) = identity.source_root else {
            continue;
        };
        let git = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(args)
                .output()
                .ok()
                .filter(|output| output.status.success())
        };
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
