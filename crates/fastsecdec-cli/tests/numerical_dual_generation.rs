//! Saved observations and caller-owned formula preparation are independent of
//! the portable evaluator identity and worker count.
#[path = "support/artifact.rs"]
mod artifact_data;
use fastsecdec::kernel::{
    KernelLoadOptions, KernelSet, PrimaryEvaluatorRestoration, indexed::ProgramArchiveReader,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Cursor,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn success(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

struct Artifact {
    content_id: String,
    archive_content_id: String,
    bytes: Vec<u8>,
    record_ids: Vec<String>,
    kernels: KernelSet,
}

fn load(content_id: &str, archive_content_id: &str, bytes: Vec<u8>) -> Artifact {
    // Validate the native exact-program identity and every record/cache digest,
    // then restore the actual saved evaluators through the public archive API.
    let mut archive = ProgramArchiveReader::from_reader(
        Cursor::new(&bytes),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    // CLI mathematical identity, archive identity and selected recipe identity
    // are distinct namespaces; compare each only with its matching owner.
    assert_eq!(archive.catalogue().content_id, archive_content_id);
    assert_eq!(archive.catalogue().recipes.len(), 1);
    let recipe = &archive.catalogue().recipes[0];
    let recipe_id = recipe.content_id.clone();
    let record_ids = recipe
        .records
        .iter()
        .map(|record| record.receipt.native_content_id.clone())
        .collect();
    let kernels = archive.select(recipe.recipe).unwrap().load_all().unwrap();
    assert_eq!(kernels.content_id(), recipe_id);
    Artifact {
        content_id: content_id.to_owned(),
        archive_content_id: archive_content_id.to_owned(),
        bytes,
        record_ids,
        kernels,
    }
}

fn worker_count_check(eager: bool) {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("input.toml");
    fs::write(
        &card,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["0","0"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1-eps"
[[direct.terms.factors]]
polynomial="1+2*x+3*y"
exponent="1"
role="polynomial"
[generation]
mode="numerical_dual"
order=1
"#,
    )
    .unwrap();
    if eager {
        let mut input = fs::read_to_string(&card).unwrap();
        input.push_str("\n[generation.evaluator]\nbackend=\"eager\"\n");
        fs::write(&card, input).unwrap();
    }
    let expected = json!({"completed":1,"total":1,"sectors":2,"reused":1});
    let mut artifacts = Vec::new();
    for workers in ["1", "4"] {
        let artifact = directory.path().join(format!("workers-{workers}.fsd"));
        let output = cli()
            .args(["--status-json", "--status-interval-ms", "0", "generate"])
            .arg(&card)
            .args(["--workers", workers, "--output"])
            .arg(&artifact)
            .output()
            .unwrap();
        let report = success(&output);
        assert_eq!(report["generation"]["formula_preparation"], expected);
        let timing = &report["generation_timings"];
        assert!(timing["formula_preparation_seconds"].as_f64().unwrap() > 0.0);
        assert!(timing["coefficient_expansion_seconds"].as_f64().unwrap() > 0.0);
        let phases = timing
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.as_str() != "total_seconds")
            .map(|(_, value)| value.as_f64().unwrap())
            .sum::<f64>();
        assert!(
            phases <= timing["total_seconds"].as_f64().unwrap(),
            "phase walls must not overlap"
        );
        let statuses = String::from_utf8_lossy(&output.stderr)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect::<Vec<_>>();
        let first_formula = statuses
            .iter()
            .position(|row| row["stage"] == "FormulaPreparation")
            .unwrap();
        let first_assembly = statuses
            .iter()
            .position(|row| row["stage"] == "CoefficientExpansion")
            .unwrap();
        assert!(first_formula < first_assembly);
        for row in statuses
            .iter()
            .filter(|row| row["stage"] == "FormulaPreparation")
        {
            assert_eq!(row["formula_preparation"]["total"], 1);
            assert_eq!(row["formula_preparation"]["sectors"], 2);
            assert!(row.get("coefficient_expansion").is_none());
            assert!(row["completed"].as_u64().unwrap() <= 1);
            if let Some(workload) = row.get("workload") {
                assert_eq!(workload["total"], 1);
            }
        }
        let saved: Value =
            serde_json::from_slice(&fs::read(artifact.with_extension("fsd.json")).unwrap())
                .unwrap();
        assert_eq!(saved["generation"], report["generation"]);
        assert_eq!(saved["generation_timings"], report["generation_timings"]);
        let inspected = success(&cli().arg("inspect").arg(&artifact).output().unwrap());
        assert_eq!(inspected["generation"], saved["generation"]);
        assert_eq!(inspected["generation_timings"], saved["generation_timings"]);
        artifacts.push(load(
            report["content_id"].as_str().unwrap(),
            saved["kernel_content_id"].as_str().unwrap(),
            fs::read(artifact_data::data_path(&artifact)).unwrap(),
        ));
    }
    let [mut one, mut four]: [Artifact; 2] = artifacts.try_into().ok().unwrap();
    assert_eq!(one.content_id, four.content_id);
    assert_eq!(one.archive_content_id, four.archive_content_id);
    assert_eq!(one.record_ids, four.record_ids);
    assert_eq!(one.kernels.content_id(), four.kernels.content_id());
    assert_eq!(one.kernels.orders(), four.kernels.orders());
    assert_eq!(one.kernels.orders(), [0, 0, 1, 1]);
    assert_eq!(one.kernels.components(), four.kernels.components());
    assert!(one.kernels.runtime_parameters().is_empty());
    assert!(four.kernels.runtime_parameters().is_empty());
    assert_eq!(
        one.kernels.exact_coefficients(),
        four.kernels.exact_coefficients()
    );
    assert_eq!(one.kernels.sectors().len(), 2);
    assert_eq!(four.kernels.sectors().len(), 2);
    for point in [[0.17, 0.29], [0.37, 0.61], [0.83, 0.73]] {
        let mut totals = [
            one.kernels.exact_coefficients().to_vec(),
            four.kernels.exact_coefficients().to_vec(),
        ];
        for (left, right) in one
            .kernels
            .sectors_mut()
            .iter_mut()
            .zip(four.kernels.sectors_mut())
        {
            assert_eq!(left.dimension(), point.len());
            assert_eq!(right.dimension(), point.len());
            let restored = if eager {
                PrimaryEvaluatorRestoration::Eager
            } else {
                PrimaryEvaluatorRestoration::CacheRestored
            };
            assert_eq!(left.primary_evaluator_restoration(), restored);
            assert_eq!(right.primary_evaluator_restoration(), restored);
            let mut a = vec![0.; totals[0].len()];
            let mut b = vec![0.; totals[1].len()];
            left.evaluate(&point, &mut a).unwrap();
            right.evaluate(&point, &mut b).unwrap();
            close(&a, &b);
            let [sum_left, sum_right] = &mut totals;
            for ((sum_a, sum_b), (a, b)) in
                sum_left.iter_mut().zip(sum_right).zip(a.into_iter().zip(b))
            {
                *sum_a += a;
                *sum_b += b;
            }
        }
        close(&totals[0], &totals[1]);
    }
    if eager {
        // Exact/eager transport is deterministic. The default SymJIT primary
        // cache serializes a native function-name HashSet whose iteration order
        // can differ without changing the exact program or restored values.
        assert!(one.bytes == four.bytes, "eager archive bytes differ");
    }
}

fn close(left: &[f64], right: &[f64]) {
    assert_eq!(left.len(), right.len());
    for (left, right) in left.iter().zip(right) {
        assert!(left.is_finite() && right.is_finite());
        assert!((left - right).abs() <= 2e-12 * right.abs().max(1.));
    }
}

#[test]
fn formula_phase_is_saved_inspectable_and_worker_count_independent() {
    worker_count_check(false);
}

#[test]
fn eager_formula_artifacts_are_byte_identical_across_worker_counts() {
    worker_count_check(true);
}
