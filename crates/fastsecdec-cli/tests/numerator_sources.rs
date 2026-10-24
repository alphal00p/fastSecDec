use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn dot(k: usize, other: &str) -> String {
    format!("spenso::dot(gammalooprs::K({k},spenso::mink(4)),{other})")
}

fn loop_momentum(k: usize) -> String {
    format!("gammalooprs::K({k},spenso::mink(4))")
}

const EXTERNAL: &str = "gammalooprs::P(0,spenso::mink(4))";

/// The two-loop massive sunset with one external momentum and the given
/// vertex numerator.
fn sunset(path: &Path, name: &str, numerator: &str, sources: Option<&str>) -> std::path::PathBuf {
    fs::write(
        path.join("model.json"),
        include_str!("../../../examples/models/scalar.json"),
    )
    .unwrap();
    fs::write(path.join("parameters.json"), r#"{"mt":[1,0]}"#).unwrap();
    fs::write(
        path.join(format!("{name}.dot")),
        format!(
            r#"digraph {name} {{
  ext0 [style=invis];
  ext1 [style=invis];
  v0 [num="{numerator}"];
  v1;
  v0 -> ext0 [id=0, particle="phi"];
  v1 -> ext1 [id=1, particle="phi"];
  v0 -> v1 [id=2, particle="phi", lmb_id=0];
  v0 -> v1 [id=3, particle="phi", lmb_id=1];
  v0 -> v1 [id=4, particle="phi"];
}}"#
        ),
    )
    .unwrap();
    let sources = sources
        .map(|sources| format!("numerator_sources = \"{sources}\""))
        .unwrap_or_default();
    let card = path.join(format!("{name}.toml"));
    fs::write(
        &card,
        format!(
            r#"
[input]
graph = "{name}.dot"
model = "model.json"
parameter_card = "parameters.json"

[kinematics]
products = [{{ left = 0, right = 0, symbol = "p0_dot_p0" }}]

[integral]
dimension = "4-2*eps"
regulator = "eps"

[generation]
order = 0
{sources}
"#
        ),
    )
    .unwrap();
    card
}

/// `(k0.k0 + 2 k0.p - 3)^2 (k1.p + 2)`: the generic (non-vacuum) jet kernel.
fn shifted_numerator() -> String {
    format!(
        "({}+2*{}-3)^2*({}+2)",
        dot(0, &loop_momentum(0)),
        dot(0, EXTERNAL),
        dot(1, EXTERNAL)
    )
}

/// Generate with a wall-clock bound: a stalled generation is killed and
/// reported instead of hanging the test.
fn generate(card: &Path, output: &Path) -> (std::process::Output, Duration) {
    const LIMIT: Duration = Duration::from_secs(60);
    let directory = output.parent().unwrap();
    let name = output.file_name().unwrap().to_string_lossy().into_owned();
    let stdout = directory.join(format!("{name}.stdout"));
    let stderr = directory.join(format!("{name}.stderr"));
    let started = Instant::now();
    let mut child = cli()
        .arg("generate")
        .arg(card)
        .arg("--output")
        .arg(output)
        .args(["--workers", "2"])
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > LIMIT {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generation of {} exceeded {LIMIT:?}", card.display());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let elapsed = started.elapsed();
    (
        std::process::Output {
            status,
            stdout: fs::read(&stdout).unwrap(),
            stderr: fs::read(&stderr).unwrap(),
        },
        elapsed,
    )
}

/// Regression: the factored route once produced a numerator factor that
/// stalled sector mapping of this two-loop external-momentum sunset (> 550 s
/// and > 18 GB, against 0.75 s on the scalar-product route).
#[test]
fn external_momentum_sunset_generates_in_seconds_on_both_routes() {
    let directory = tempfile::tempdir().unwrap();
    for (name, sources, route) in [
        ("automatic", None, "factors"),
        ("factored", Some("factors"), "factors"),
        ("scalar", Some("scalar_products"), "scalar_products"),
    ] {
        let card = sunset(directory.path(), name, &shifted_numerator(), sources);
        let output = directory.path().join(format!("{name}.fsd"));
        let (result, elapsed) = generate(&card, &output);
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        eprintln!("{name}: generated in {elapsed:?}");
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(output.with_extension("fsd.json")).unwrap()).unwrap();
        let record = &metadata["generation"];
        let numerator = &record["numerator_route"];
        assert_eq!(numerator["route"], route, "{record}");
        assert_eq!(numerator["requested"], sources.unwrap_or("auto"));
        assert!(numerator.get("fallback").is_none(), "{record}");
        if route == "factors" {
            assert_eq!(numerator["source_rows"], 2);
            assert_eq!(numerator["moments"], 1);
        }
    }
}

/// Without a compact affine structure, an explicit request fails before any
/// work, while the automatic choice reports its fallback.
#[test]
fn requested_factored_sources_fail_without_affine_structure() {
    let directory = tempfile::tempdir().unwrap();
    // Six distinct affine forms in the five loop scalar products.
    let numerator = format!(
        "({}+1)*({}+2)*({}+3)*({}+4)*({}+5)*({}+{}+6)",
        dot(0, &loop_momentum(0)),
        dot(1, &loop_momentum(1)),
        dot(0, &loop_momentum(1)),
        dot(0, EXTERNAL),
        dot(1, EXTERNAL),
        dot(0, &loop_momentum(0)),
        dot(1, &loop_momentum(1)),
    );
    let card = sunset(directory.path(), "required", &numerator, Some("factors"));
    let output = directory.path().join("required.fsd");
    let (result, _) = generate(&card, &output);
    assert!(!result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let message = report["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("numerator_sources = \"factors\" requires")
            && message.contains("more distinct affine forms"),
        "{report}"
    );
    assert!(!output.with_extension("fsd.json").exists());

    let card = sunset(directory.path(), "automatic", &numerator, None);
    let result = cli().arg("inspect").arg(&card).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["numerator"]["route"], "scalar_products", "{report}");
    assert_eq!(
        report["numerator"]["fallback"], "too_many_forms",
        "{report}"
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("fastsecdec warning"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn direct_inputs_reject_numerator_sources() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("run.toml");
    fs::write(
        &card,
        r#"
[direct]
domain="unit_cube"
parameters=["x"]
[[direct.terms]]
monomial_powers=["0"]
[generation]
numerator_sources="factors"
"#,
    )
    .unwrap();
    let output = directory.path().join("unused.fsd");
    let (result, _) = generate(&card, &output);
    assert!(!result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(
        report["error"]["message"]
            .as_str()
            .unwrap()
            .contains("numerator_sources requires native graph input"),
        "{report}"
    );
}
