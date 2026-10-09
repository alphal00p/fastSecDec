//! Selective recipe loading through the real CLI, including disposable workers.
use fastsecdec::kernel::indexed::{ProgramArchiveCatalogue, ProgramArchiveWriter, ProgramRecipe};
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{Seek, SeekFrom},
    path::Path,
    process::{Command, Output},
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

// The production recipe-set generator is a separate milestone. Build this
// transport fixture by copying actual CLI-generated native records, using the
// public native writer. No evaluator or symbolic expression is reconstructed.
fn bundle(directory: &Path) -> std::path::PathBuf {
    let input = directory.join("input.toml");
    fs::write(
        &input,
        r#"
[direct]
domain="unit_cube"
parameters=["x"]
[[direct.terms]]
monomial_powers=["0"]
[[direct.terms.factors]]
polynomial="1+x"
exponent="-1"
semantics="causal"
[generation.evaluator]
backend="eager"
[integration]
absolute_tolerance=0.0
relative_tolerance=0.0
"#,
    )
    .unwrap();
    let mut sources = Vec::new();
    for recipe in [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1] {
        let path = directory.join(format!("{}.fsd", recipe.name()));
        let mut command = cli();
        command
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&path)
            .args(["--workers", "1"]);
        if recipe == ProgramRecipe::FixedV1 {
            command.arg("--contour");
        }
        success(command.output().unwrap());
        let manifest: Value =
            serde_json::from_slice(&fs::read(path.with_extension("fsd.json")).unwrap()).unwrap();
        sources.push((recipe, manifest));
    }
    let mut manifest = sources[1].1.clone();
    let data_name = "recipes.complete.dat";
    let mut writer = ProgramArchiveWriter::new(
        File::create(directory.join(data_name)).unwrap(),
        manifest["programs"]["catalogue"]["source_identity"]
            .as_str()
            .unwrap()
            .to_owned(),
        sources.iter().map(|(recipe, _)| *recipe),
    )
    .unwrap();
    for (recipe, source) in sources {
        let catalogue: ProgramArchiveCatalogue =
            serde_json::from_value(source["programs"]["catalogue"].clone()).unwrap();
        let mut data =
            File::open(directory.join(source["programs"]["data_file"].as_str().unwrap())).unwrap();
        for record in &catalogue.recipe(recipe).unwrap().records {
            data.seek(SeekFrom::Start(record.offset)).unwrap();
            writer
                .append_record(recipe, &mut data, record.receipt.clone())
                .unwrap();
        }
    }
    let (file, catalogue) = writer.finish().unwrap();
    file.sync_all().unwrap();
    manifest["format_version"] = 5.into();
    manifest["kernel_content_id"] = catalogue.content_id.clone().into();
    manifest["indexed"] = Value::Null;
    manifest["programs"] = serde_json::json!({
        "data_file":data_name, "default_recipe":"fixed-v1", "catalogue":catalogue,
    });
    let path = directory.join("bundle.fsd");
    fs::write(
        path.with_extension("fsd.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    path
}

#[test]
fn explicit_recipes_work_in_resident_and_serial_cli_with_full_covariance() {
    let directory = tempfile::tempdir().unwrap();
    let path = bundle(directory.path());
    for (recipe, mode) in [("undeformed-v1", "off"), ("fixed-v1", "fixed")] {
        let inspected = success(
            cli()
                .arg("inspect")
                .arg(&path)
                .args(["--recipe", recipe, "--validate-artifact"])
                .output()
                .unwrap(),
        );
        assert_eq!(inspected["selected_recipe"], recipe);
        assert_eq!(inspected["binary_loaded"], false);
        let deep = success(
            cli()
                .arg("inspect")
                .arg(&path)
                .args([
                    "--recipe",
                    recipe,
                    "--sector",
                    "0",
                    "--deep",
                    "--validate-artifact",
                ])
                .output()
                .unwrap(),
        );
        assert_eq!(deep["selected_recipe"], recipe);
        assert_eq!(deep["loaded_sectors"], 1);
        let all = success(
            cli()
                .arg("inspect")
                .arg(&path)
                .args(["--recipe", recipe, "--deep", "--validate-artifact"])
                .output()
                .unwrap(),
        );
        for view in [&deep, &all] {
            assert_eq!(view["selected_recipe"], recipe);
            assert_eq!(view["available_recipes"], inspected["available_recipes"]);
            assert_eq!(
                view["selected_catalogue_content_id"],
                inspected["selected_catalogue_content_id"]
            );
        }
        assert!(all["selected_catalogue_content_id"].is_string());
        for serial in [false, true] {
            let checkpoint = directory
                .path()
                .join(format!("{mode}-{serial}.checkpoint.json"));
            let saved = directory
                .path()
                .join(format!("{mode}-{serial}.result.json"));
            let mut command = cli();
            command
                .arg("integrate")
                .arg(&path)
                .args([
                    "--contour",
                    mode,
                    "--contour-validation",
                    "always",
                    "--contour-pilot-points",
                    "8",
                    "--points",
                    "1024",
                    "--shifts",
                    "4",
                    "--workers",
                    "2",
                    "--max-rounds",
                    "1",
                    "--seed",
                    "77237",
                    "--validate-artifact",
                ])
                .arg("--checkpoint")
                .arg(&checkpoint)
                .arg("--save-result")
                .arg(&saved);
            if mode == "fixed" {
                command.args(["--lambda", "0.2"]);
            }
            if serial {
                command.args(["--serial", "0.001"]);
            }
            let report = success(command.output().unwrap());
            let estimate = &report["estimate"];
            let native: fastsecdec::integration::VectorEstimate =
                serde_json::from_value(estimate.clone()).unwrap();
            native.validate().unwrap();
            let expected_width = if mode == "fixed" { 2 } else { 1 };
            assert_eq!(native.mean.len(), expected_width);
            assert_eq!(
                native.covariance_of_mean.len(),
                expected_width * expected_width
            );
            let components = estimate["components"].as_array().unwrap();
            for (index, component) in components.iter().enumerate() {
                let expected = if component == "Real" { 2f64.ln() } else { 0. };
                let mean = estimate["mean"][index].as_f64().unwrap();
                let error = estimate["standard_error"][index].as_f64().unwrap();
                assert!(
                    (mean - expected).abs() < 8. * error + 1e-6,
                    "{recipe} serial={serial}: {estimate}"
                );
            }
            let saved = fastsecdec::results::read_result(&fs::read(saved).unwrap()).unwrap();
            assert_eq!(saved.provenance.attributes["selected_recipe"], recipe);
            let incompatible = if mode == "fixed" { "off" } else { "fixed" };
            let mut resume = cli();
            resume
                .arg("integrate")
                .arg(&path)
                .args([
                    "--contour",
                    incompatible,
                    "--resume",
                    "--max-rounds",
                    "1",
                    "--points",
                    "1024",
                    "--shifts",
                    "4",
                    "--seed",
                    "77237",
                ])
                .arg("--checkpoint")
                .arg(checkpoint);
            if incompatible == "fixed" {
                resume.args(["--lambda", "0.2"]);
            }
            if serial {
                resume.args(["--serial", "0.001"]);
            }
            assert!(
                !resume.output().unwrap().status.success(),
                "cross-recipe checkpoint was accepted"
            );
        }
    }
}
