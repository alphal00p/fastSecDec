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
fn old_outer_v1_identity_and_native_bytes_survive_load_and_resave() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.json");
    let saved_again = directory.path().join("legacy-again.json");
    let mut kernels = legacy_kernels();
    let expected = vector(&mut kernels);
    assert_eq!(expected.len(), 3);
    assert_eq!(expected[0], -1.0);
    let old_kernel: serde_json::Value =
        serde_json::from_slice(kernels.artifact_bytes().unwrap()).unwrap();
    let provenance = provenance();
    // The old protocol is written independently here: two canonical serialized
    // values, following the original domain marker, with no v2 native-ID shortcut.
    let mut old_hash = blake3::Hasher::new();
    old_hash.update(b"fastsecdec-artifact-v1");
    old_hash.update(&serde_json::to_vec(&provenance).unwrap());
    old_hash.update(&serde_json::to_vec(&old_kernel).unwrap());
    let id = old_hash.finalize().to_hex().to_string();
    let old = serde_json::json!({"format_version":1,"content_id":id,
        "provenance":provenance,"kernel":old_kernel});
    fs::write(&path, serde_json::to_vec_pretty(&old).unwrap()).unwrap();
    let (artifact, mut loaded) = Artifact::load(&path).unwrap();
    assert_eq!(artifact.format_version, 1);
    assert_eq!(artifact.content_id, id);
    assert_eq!(vector(&mut loaded), expected);
    artifact.save(&saved_again).unwrap();
    let (again, mut loaded_again) = Artifact::load(&saved_again).unwrap();
    assert_eq!(again.format_version, 1);
    assert_eq!(again.content_id, id);
    assert_eq!(
        loaded.artifact_bytes().unwrap(),
        loaded_again.artifact_bytes().unwrap()
    );
    assert_eq!(vector(&mut loaded_again), expected);
}

#[test]
fn outer_v2_keeps_native_json_verbatim_and_cold_full_vector() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native.json");
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs/analytic_endpoint.toml");
    let (artifact, mut kernels) = crate::generate::generate(
        &input,
        &path,
        &mut crate::display::Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(artifact.format_version, 2);
    assert_eq!(
        artifact.kernel.get().as_bytes(),
        kernels.artifact_bytes().unwrap()
    );
    let expected = vector(&mut kernels);
    assert_eq!(expected.len(), 3);
    let (mut loaded_artifact, mut loaded) = Artifact::load(&path).unwrap();
    assert_eq!(loaded_artifact.content_id, artifact.content_id);
    assert_eq!(
        loaded.artifact_bytes().unwrap(),
        kernels.artifact_bytes().unwrap()
    );
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
fn outer_v2_rejects_payload_tampering_even_when_both_claimed_ids_are_unchanged() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tampered.json");
    let artifact = Artifact::new(&legacy_kernels(), provenance()).unwrap();
    let original = serde_json::to_value(&artifact).unwrap();
    for mutation in ["provenance", "kernel_id", "payload"] {
        let mut value = original.clone();
        match mutation {
            "provenance" => value["provenance"]["measure_multiplier"] = "2".into(),
            "kernel_id" => value["kernel"]["content_id"] = "bad-id".into(),
            "payload" => value["kernel"]["payload"]["exact"][0] = "123".into(),
            _ => unreachable!(),
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let preflight_called = Cell::new(false);
        assert!(
            Artifact::load_with_preflight(&path, |_| {
                preflight_called.set(true);
                Ok(())
            })
            .is_err()
        );
        // The payload mutation deliberately passes the unchanged outer v2 ID;
        // native payload validation still rejects it before any successful load.
        assert_eq!(preflight_called.get(), mutation == "payload");
    }
    fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
    let error = Artifact::load_with_preflight(&path, |_| Err("caller preflight sentinel".into()))
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "caller preflight sentinel");
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
