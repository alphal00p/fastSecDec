//! Optional external-process acceptance test; symGCAD is never a Cargo dependency.
#![cfg(target_os = "linux")]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[path = "support/external_workflow.rs"]
mod external_workflow;
use external_workflow::run;

#[test]
#[ignore = "requires an explicit SYMGCAD_BIN and external wall/memory allocation"]
fn native_bubble_export_import_solve_and_independent_verify() {
    let symgcad = PathBuf::from(
        std::env::var_os("SYMGCAD_BIN")
            .expect("set SYMGCAD_BIN to a tested native symGCAD executable"),
    );
    assert!(symgcad.is_file());
    let seconds = std::env::var("FASTSECDEC_INTEROP_SECONDS")
        .ok()
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(180);
    assert!((1..=600).contains(&seconds));
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let directory = if let Some(path) = std::env::var_os("FASTSECDEC_INTEROP_OUTPUT") {
        let path = PathBuf::from(path);
        fs::create_dir(&path).expect("interop output must be fresh");
        path
    } else {
        tempfile::tempdir().unwrap().keep()
    };
    println!(
        "retained interoperability evidence: {}",
        directory.display()
    );
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/no_deformation");
    let export_dir = directory.join("export");
    let mut export = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    export
        .args(["--json", "--plain", "export-symanzik"])
        .arg(fixture.join("bubble.toml"))
        .arg("--point")
        .arg(fixture.join("bubble-point.toml"))
        .arg("--output")
        .arg(&export_dir);
    run(export, &directory, "export", deadline);
    let checked = directory.join("checked.toml");
    let mut import = Command::new(&symgcad);
    import
        .arg("import-fastsecdec")
        .arg(export_dir.join("symanzik.json"))
        .arg("--problem")
        .arg(export_dir.join("problem.toml"))
        .arg("--output")
        .arg(&checked);
    let imported = run(import, &directory, "import", deadline);
    assert_eq!(imported["coverage_claimed"], false);
    // Only operational limits change after the exact companion import.
    let mut problem: toml::Value = toml::from_str(&fs::read_to_string(&checked).unwrap()).unwrap();
    let mut limits = toml::Table::new();
    limits.insert("wall_time_secs".into(), toml::Value::Float(30.0));
    limits.insert("memory_mib".into(), toml::Value::Integer(2048));
    limits.insert("workers".into(), toml::Value::Integer(1));
    problem
        .as_table_mut()
        .unwrap()
        .insert("limits".into(), toml::Value::Table(limits));
    let bounded = directory.join("bounded.toml");
    fs::write(&bounded, toml::to_string_pretty(&problem).unwrap()).unwrap();
    let proof = directory.join("result.json");
    let mut solve = Command::new(&symgcad);
    solve.arg("solve").arg(&bounded).arg("--output").arg(&proof);
    run(solve, &directory, "solve", deadline);
    let result: serde_json::Value = serde_json::from_slice(&fs::read(&proof).unwrap()).unwrap();
    assert_eq!(result["status"], "complete_generic");
    let mut verify = Command::new(&symgcad);
    verify.arg("verify").arg(&proof);
    let verified = run(verify, &directory, "verify", deadline);
    assert_eq!(verified["verified"], true);
    assert!(verified["cells"].as_u64().unwrap() > 0);
}
