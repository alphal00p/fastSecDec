//! Native-process streaming, durable recovery and normal/serial artifact parity.
use fastsecdec::kernel::{KernelLoadOptions, KernelSet, indexed::ProgramArchiveReader};
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn card(path: &Path, mode: &str) {
    fs::write(
        path,
        format!(
            r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
prefactor="2+3𝑖"
monomial_powers=["-1+eps","eps"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1-eps"
[[direct.terms.factors]]
polynomial="1+2*x+3*y"
exponent="1"
role="polynomial"
[generation]
mode="{mode}"
order=0
[generation.evaluator]
backend="eager"
"#
        ),
    )
    .unwrap();
}

fn manifest(base: &Path) -> Value {
    serde_json::from_slice(&fs::read(base.with_extension("fsd.json")).unwrap()).unwrap()
}

fn load(base: &Path) -> KernelSet {
    let saved = manifest(base);
    let path = base
        .parent()
        .unwrap()
        .join(saved["programs"]["data_file"].as_str().unwrap());
    let mut indexed = ProgramArchiveReader::from_reader(
        File::open(path).unwrap(),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let recipe = indexed.catalogue().recipes[0].recipe;
    indexed.select(recipe).unwrap().load_all().unwrap()
}

fn sample(mut kernels: KernelSet) -> Vec<f64> {
    let mut values = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut contribution = vec![0.; values.len()];
        sector.evaluate(&[0.37, 0.61], &mut contribution).unwrap();
        for (sum, value) in values.iter_mut().zip(contribution) {
            *sum += value;
        }
    }
    values
}

fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 2e-10 * expected.abs().max(1.));
    }
}

#[test]
fn original_source_subset_roundtrips_normal_serial_and_rejects_changed_resume() {
    let directory = tempfile::tempdir().unwrap();
    for mode in ["symbolic", "numerical_dual"] {
        let input = directory.path().join(format!("subset-{mode}.toml"));
        card(&input, mode);
        let selected_card = fs::read_to_string(&input)
            .unwrap()
            .replace("[generation]", "[generation]\nsource_sectors=[1]");
        fs::write(&input, &selected_card).unwrap();
        let ordinary = directory.path().join(format!("selected-{mode}.fsd"));
        success(
            cli()
                .arg("generate")
                .arg(&input)
                .arg("--output")
                .arg(&ordinary)
                .output()
                .unwrap(),
        );
        let normal = load(&ordinary);
        let scope = normal
            .generation_metadata()
            .unwrap()
            .source_scope()
            .unwrap();
        assert_eq!(scope.selection().original_source_count(), 2);
        assert_eq!(scope.selection().source_sectors(), [1]);
        assert_eq!(scope.chart_source_sectors(), [1]);
        assert_eq!(
            normal.generation_metadata().unwrap().charts()[0].source_index(),
            0
        );
        let reference = sample(normal);
        let streamed = directory.path().join(format!("selected-{mode}-serial.fsd"));
        success(
            cli()
                .arg("generate")
                .arg(&input)
                .args(["--serial", "--workers", "2", "--output"])
                .arg(&streamed)
                .output()
                .unwrap(),
        );
        let resident = load(&streamed);
        assert_eq!(
            resident
                .generation_metadata()
                .unwrap()
                .source_scope()
                .unwrap()
                .selection()
                .source_sectors(),
            [1]
        );
        close(&sample(resident), &reference);
        assert_eq!(
            manifest(&ordinary)["content_id"],
            manifest(&streamed)["content_id"]
        );
        let inspected = success(cli().arg("inspect").arg(&streamed).output().unwrap());
        assert_eq!(
            inspected["source_selection"]["source_sectors"],
            serde_json::json!([1])
        );
        let deep = success(
            cli()
                .arg("inspect")
                .arg(&streamed)
                .args(["--deep", "--sector", "0"])
                .output()
                .unwrap(),
        );
        assert_eq!(
            deep["source_selection"]["source_sectors"],
            serde_json::json!([1])
        );
        let integrated = success(
            cli()
                .arg("integrate")
                .arg(&streamed)
                .args([
                    "--full-integral",
                    "--points",
                    "1024",
                    "--shifts",
                    "2",
                    "--max-rounds",
                    "1",
                    "--workers",
                    "1",
                ])
                .output()
                .unwrap(),
        );
        assert_eq!(
            integrated["source_selection"]["source_sectors"],
            serde_json::json!([1])
        );
        assert!(integrated["scope"].get("SelectedSectors").is_some());
        assert_eq!(integrated["converged"], false);
        success(
            cli()
                .arg("generate")
                .arg(&input)
                .args(["--resume", "--output"])
                .arg(&streamed)
                .output()
                .unwrap(),
        );
        fs::write(
            &input,
            selected_card.replace("source_sectors=[1]", "source_sectors=[0]"),
        )
        .unwrap();
        let changed = cli()
            .arg("generate")
            .arg(&input)
            .args(["--resume", "--output"])
            .arg(&streamed)
            .output()
            .unwrap();
        assert!(
            !changed.status.success(),
            "resume accepted a different original source selection"
        );
    }
}

#[test]
fn normal_and_streamed_native_processes_preserve_the_complete_laurent_vector() {
    let directory = tempfile::tempdir().unwrap();
    for mode in ["symbolic", "numerical_dual"] {
        let input = directory.path().join(format!("{mode}.toml"));
        card(&input, mode);
        let ordinary = directory.path().join(format!("{mode}-ordinary.fsd"));
        let normal = success(
            cli()
                .arg("generate")
                .arg(&input)
                .arg("--output")
                .arg(&ordinary)
                .output()
                .unwrap(),
        );
        let reference = sample(load(&ordinary));
        for workers in ["1", "2"] {
            let streamed = directory.path().join(format!("{mode}-{workers}.fsd"));
            let generated = success(
                cli()
                    .arg("generate")
                    .arg(&input)
                    .args(["--serial", "--workers", workers, "--output"])
                    .arg(&streamed)
                    .output()
                    .unwrap(),
            );
            assert_eq!(generated["content_id"], normal["content_id"]);
            assert_eq!(generated["orders"], normal["orders"]);
            assert_eq!(generated["components"], normal["components"]);
            close(&sample(load(&streamed)), &reference);
            let before = fs::read(streamed.with_extension("fsd.json")).unwrap();
            // Successful recovery is metadata-only and leaves the committed
            // immutable artifact untouched even with another worker count.
            success(
                cli()
                    .arg("generate")
                    .arg(&input)
                    .args(["--resume", "--workers", "3", "--output"])
                    .arg(&streamed)
                    .output()
                    .unwrap(),
            );
            assert_eq!(
                before,
                fs::read(streamed.with_extension("fsd.json")).unwrap()
            );
        }
    }
}

#[test]
fn killed_coordinator_reclaims_children_before_changed_worker_resume() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.toml");
    card(&input, "numerical_dual");
    let output = directory.path().join("recovered.fsd");
    let mut child = cli()
        .args(["--status-json", "--status-interval-ms", "0", "generate"])
        .arg(&input)
        .args(["--serial", "--workers", "1", "--output"])
        .arg(&output)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut reached = false;
    for line in BufReader::new(stderr).lines() {
        let line = line.unwrap();
        if let Ok(status) = serde_json::from_str::<Value>(&line)
            && status["stage"] == "Mapping"
            && status["completed"].as_u64().is_some_and(|n| n > 0)
        {
            reached = true;
            break;
        }
    }
    assert!(reached, "no durable discovery boundary was observed");
    child.kill().unwrap();
    child.wait().unwrap();
    // Every native child inherits the same locked open-file description. Its
    // last close is proof that crash-lost work cannot overlap the resumed run.
    let staging = directory.path().join("recovered.fsd.generation");
    let lock = File::open(staging.join("lock")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if lock.try_lock().is_ok() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "orphaned native worker retained residency"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(lock);
    let resumed = success(
        cli()
            .arg("generate")
            .arg(&input)
            .args(["--resume", "--workers", "2", "--output"])
            .arg(&output)
            .output()
            .unwrap(),
    );
    assert!(resumed["sectors"].as_u64().unwrap() > 0);
    let values = sample(load(&output));
    assert!(values.iter().all(|value| value.is_finite()));
}
