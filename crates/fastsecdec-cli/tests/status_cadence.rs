use std::{
    fs,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    c.env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--json", "--plain"]);
    c
}
fn report(output: Output) -> (serde_json::Value, Vec<serde_json::Value>) {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let statuses = String::from_utf8(output.stderr)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|status| status.get("method").is_some())
        .collect();
    (serde_json::from_slice(&output.stdout).unwrap(), statuses)
}

#[test]
fn cadence_is_observational_and_forced_stage_final_events_survive_a_long_interval() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let artifact = dir.path().join("artifact.json");
    fs::write(&card,"[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['1']\n[integration]\npoints=1024\nshifts=4\n").unwrap();
    report(
        cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let mut observations = Vec::new();
    let mut settings = Vec::new();
    let mut replay = Vec::new();
    for interval in ["0", "60000"] {
        let checkpoint = dir.path().join(format!("{interval}.json"));
        let observation = report(
            cli()
                .args([
                    "--status-json",
                    "--status-interval-ms",
                    interval,
                    "integrate",
                ])
                .arg(&artifact)
                .arg("--checkpoint")
                .arg(&checkpoint)
                .output()
                .unwrap(),
        );
        assert_eq!(observation.1.first().unwrap()["completed_points"], 0);
        let final_status = observation.1.last().unwrap();
        assert_eq!(final_status["completed_points"], 4096);
        assert!(final_status["stop_reason"].is_string());
        assert_eq!(final_status["scope"], "FullIntegral");
        let checkpoint: serde_json::Value =
            serde_json::from_slice(&fs::read(checkpoint).unwrap()).unwrap();
        settings.push(checkpoint["settings"].clone());
        replay.push(checkpoint["replay"].clone());
        observations.push(observation);
    }
    assert!(observations[1].1.len() < observations[0].1.len());
    assert_eq!(observations[0].0["estimate"], observations[1].0["estimate"]);
    assert_eq!(
        observations[0].0["qmc_design"],
        observations[1].0["qmc_design"]
    );
    assert_eq!(settings[0], settings[1]);
    assert_eq!(replay[0], replay[1]);
    assert!(settings[0].get("status_interval_ms").is_none());

    let (result, stages) = report(
        cli()
            .args([
                "--status-json",
                "--status-interval-ms",
                "60000",
                "integrate",
            ])
            .arg(&artifact)
            .args([
                "--method",
                "adaptive_mc",
                "--points",
                "128",
                "--shifts",
                "3",
                "--checkpoint",
            ])
            .arg(dir.path().join("pilot.json"))
            .output()
            .unwrap(),
    );
    assert_eq!(stages.first().unwrap()["stage"], "Pilot");
    assert!(
        stages
            .iter()
            .any(|s| s["stage"] == "Pilot" && s["completed_points"] == s["planned_points"])
    );
    assert!(
        stages
            .iter()
            .any(|s| s["stage"] == "Production" && s["completed_points"] == 0)
    );
    assert_eq!(stages.last().unwrap()["stage"], "Production");
    assert!(stages.last().unwrap()["stop_reason"].is_string());
    assert_eq!(result["snapshot"]["completed_points"], 384);
}
