use super::*;
use std::cell::Cell;

fn provenance() -> Provenance {
    Provenance {
        name: "persistence control".into(),
        sources: vec![],
        dependencies: dependencies(),
        domain: "ProjectiveSimplex".into(),
        assume_no_threshold: false,
        dimension: "4-2*eps".into(),
        regulator: "eps".into(),
        measure: "native".into(),
        measure_multiplier: "1".into(),
        max_order: 0,
        integration: serde_json::json!({}),
        family_preparation: None,
        model_parameter_defaults: Default::default(),
    }
}

fn legacy_kernels() -> KernelSet {
    KernelSet::from_bytes(include_bytes!(
        "../../../fastsecdec/tests/fixtures/kernel-v2-triangle.json"
    ))
    .unwrap()
}

fn vector(kernels: &mut KernelSet) -> Vec<f64> {
    let mut values = kernels.exact_coefficients().to_vec();
    assert_eq!(values.len(), kernels.orders().len());
    for sector in kernels.sectors_mut() {
        let mut row = vec![0.0; sector.output_count()];
        sector
            .evaluate(&vec![0.25; sector.dimension()], &mut row)
            .unwrap();
        assert_eq!(row.len(), values.len());
        for (total, value) in values.iter_mut().zip(row) {
            *total += value;
        }
    }
    assert!(values.iter().all(|value| value.is_finite()));
    values
}

#[test]
fn native_pair_roundtrips_legacy_kernel_and_human_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native.fsd");
    let mut kernels = legacy_kernels();
    let expected = vector(&mut kernels);
    let artifact = Artifact::new(&kernels, provenance()).unwrap();
    artifact.save(&path).unwrap();
    let (metadata, data) = paths(&path).unwrap();
    assert!(!path.exists());
    assert!(metadata.exists() && data.exists());
    let human: serde_json::Value = serde_json::from_slice(&fs::read(metadata).unwrap()).unwrap();
    assert_eq!(human["format_version"], 3);
    assert!(human["kernel"].get("payload").is_none());
    assert_eq!(
        human["kernel"]["orders"],
        serde_json::json!(kernels.orders())
    );
    let (loaded_artifact, mut loaded) = Artifact::load(&path).unwrap();
    assert_eq!(loaded_artifact.content_id, artifact.content_id);
    assert_eq!(vector(&mut loaded), expected);
    let relocated = directory.path().join("moved").join("native.fsd");
    loaded_artifact.save(&relocated).unwrap();
    let (again, mut kernels) = Artifact::load(&relocated).unwrap();
    assert_eq!(again.content_id, artifact.content_id);
    assert_eq!(vector(&mut kernels), expected);
}

#[test]
fn generation_observations_do_not_change_the_pair_identity() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native.fsd");
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs/analytic_endpoint.toml");
    let (artifact, mut kernels) = crate::generate::generate(
        &input,
        &path,
        &mut crate::display::Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    let expected = vector(&mut kernels);
    assert_eq!(expected.len(), 3);
    let (mut loaded_artifact, mut loaded) = Artifact::load(&path).unwrap();
    assert_eq!(loaded_artifact.content_id, artifact.content_id);
    assert_eq!(vector(&mut loaded), expected);
    loaded_artifact.generation_timings = Some(GenerationTimings {
        total_seconds: 999.0,
        ..Default::default()
    });
    assert_eq!(loaded_artifact.identity().unwrap(), artifact.content_id);
    loaded_artifact.save(&path).unwrap();
    assert_eq!(
        Artifact::load(&path).unwrap().0.content_id,
        artifact.content_id
    );
}

#[test]
fn pair_rejects_metadata_and_binary_tampering_and_runs_preflight_before_binary_loading() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tampered.fsd");
    let artifact = Artifact::new(&legacy_kernels(), provenance()).unwrap();
    artifact.save(&path).unwrap();
    let (metadata, data) = paths(&path).unwrap();
    let original_bytes = fs::read(&metadata).unwrap();
    let original = serde_json::to_value(&artifact).unwrap();
    for mutation in ["provenance", "kernel_id", "summary"] {
        let mut value = original.clone();
        match mutation {
            "provenance" => value["provenance"]["measure_multiplier"] = "2".into(),
            "kernel_id" => value["kernel_content_id"] = "bad-id".into(),
            "summary" => value["kernel"]["sectors"] = 123.into(),
            _ => unreachable!(),
        }
        fs::write(&metadata, serde_json::to_vec(&value).unwrap()).unwrap();
        let called = Cell::new(false);
        assert!(
            Artifact::load_with_preflight(&path, |_| {
                called.set(true);
                Ok(())
            })
            .is_err()
        );
        assert!(!called.get());
    }
    // RawValue's human layout is part of this envelope's integrity check.
    // Restore the producer's exact bytes before testing binary/preflight order.
    fs::write(&metadata, original_bytes).unwrap();
    fs::write(&data, b"truncated native data").unwrap();
    let error = Artifact::load_with_preflight(&path, |_| Err("caller preflight sentinel".into()))
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "caller preflight sentinel");
    assert!(Artifact::load(&path).is_err());
    fs::remove_file(data).unwrap();
    assert!(Artifact::load(&path).is_err());
}

#[test]
fn suffixes_are_rejected_before_reading_or_writing_files() {
    let directory = tempfile::tempdir().unwrap();
    for suffix in ["json", "dat"] {
        let path = directory.path().join(format!("native.fsd.{suffix}"));
        assert!(paths(&path).unwrap_err().to_string().contains("basename"));
        assert!(
            Artifact::load(&path)
                .err()
                .unwrap()
                .to_string()
                .contains("basename")
        );
    }
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn interrupted_atomic_writer_preserves_destination_and_cleans_its_temporary() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("artifact.json");
    fs::write(&path, b"previous artifact").unwrap();
    let error = atomic_write_with(&path, |writer| {
        writer.write_all(b"partial replacement")?;
        Err("simulated write failure".into())
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "simulated write failure");
    assert_eq!(fs::read(&path).unwrap(), b"previous artifact");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn generation_choices_roundtrip_without_inventing_historical_settings() {
    use fastsecdec::generation::{GenerationMode, SubtractionStrategy};
    let old =
        serde_json::json!({"workers": 1, "requested_coefficient_expansion": "full_expression"});
    let historical: GenerationRecord = serde_json::from_value(old).unwrap();
    assert!(historical.mode.is_none());
    assert!(historical.subtraction.is_none());
    assert!(historical.source_chart_modes.is_none());
    assert!(historical.formula_preparation.is_none());
    let old_encoded = serde_json::to_value(&historical).unwrap();
    assert!(old_encoded.get("mode").is_none());
    assert!(old_encoded.get("subtraction").is_none());
    assert!(old_encoded.get("source_chart_modes").is_none());
    assert!(old_encoded.get("formula_preparation").is_none());

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("generation-choices.fsd");
    let mut artifact = Artifact::new(&legacy_kernels(), provenance()).unwrap();
    let id = artifact.content_id.clone();
    artifact.generation = Some(GenerationRecord {
        mode: Some(GenerationMode::NumericalDual),
        subtraction: Some(SubtractionStrategy::IntegrateByParts),
        source_chart_modes: Some(
            [
                (4, GenerationMode::NumericalDual),
                (8, GenerationMode::Symbolic),
            ]
            .into(),
        ),
        formula_preparation: Some(fastsecdec::status::FormulaPreparationSnapshot {
            completed: 2,
            total: 2,
            sectors: 5,
            reused: 3,
        }),
        ..historical
    });
    artifact.generation_timings = Some(GenerationTimings {
        formula_preparation_seconds: Some(0.25),
        ..Default::default()
    });
    artifact.save(&path).unwrap();
    let (loaded, _) = Artifact::load(&path).unwrap();
    assert_eq!(loaded.content_id, id);
    assert_eq!(
        loaded
            .generation_timings
            .as_ref()
            .unwrap()
            .formula_preparation_seconds,
        Some(0.25)
    );
    let record = loaded.generation.unwrap();
    assert_eq!(record.mode, Some(GenerationMode::NumericalDual));
    assert_eq!(
        record.source_chart_modes.as_ref().unwrap()[&4],
        GenerationMode::NumericalDual
    );
    assert_eq!(
        record.source_chart_modes.as_ref().unwrap()[&8],
        GenerationMode::Symbolic
    );
    assert_eq!(
        record.subtraction,
        Some(SubtractionStrategy::IntegrateByParts)
    );
    let rows = crate::generation_report::generation_method_rows(record.mode, record.subtraction);
    assert_eq!(rows[0][1], "numerical_dual");
    assert_eq!(rows[1][1], "integrate_by_parts");
    let formulas = record.formula_preparation.unwrap();
    assert_eq!(
        (
            formulas.completed,
            formulas.total,
            formulas.sectors,
            formulas.reused
        ),
        (2, 2, 5, 3)
    );
    let rows = crate::generation_report::formula_preparation_rows(Some(formulas));
    assert_eq!(rows[0][1], "2 / 2 unique ready");
    assert_eq!(rows[1][1], "5 eligible sectors · 3 shared uses");
    assert_eq!(
        crate::generation_report::formula_preparation_rows(None)[0][1],
        "Not recorded"
    );
    assert!(
        crate::generation_report::generation_method_rows(None, None)
            .iter()
            .all(|r| r[1] == "Not recorded")
    );
}
