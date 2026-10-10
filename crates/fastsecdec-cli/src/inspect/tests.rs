use super::*;
use std::fs;

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, Artifact, KernelSet) {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let path = dir.path().join("new.fsd");
    fs::write(&card,"[direct]\ndomain='unit_cube'\nparameters=['x','y']\n[[direct.terms]]\nmonomial_powers=['0','0']\n[[direct.terms.factors]]\npolynomial='x+y'\nexponent='-1+eps'\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    let (artifact, kernels) = crate::generate::generate(
        &card,
        &path,
        &mut crate::display::Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    (dir, path, artifact, kernels)
}
fn metadata(path: &Path, artifact: &Artifact) {
    fs::write(
        crate::artifact::paths(path).unwrap().0,
        serde_json::to_vec_pretty(artifact).unwrap(),
    )
    .unwrap();
}

#[test]
fn threshold_inspection_retains_scope_and_lineage_without_opening_other_records() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let path = dir.path().join("threshold.fsd");
    fs::write(&input, "[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nprefactor='1/eps'\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='x-1/3'\nexponent='-eps'\nsemantics='causal'\n[generation]\nthreshold_decomposition=true\norder=0\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    let (artifact, native) = crate::generate::generate(
        &input,
        &path,
        &mut crate::display::Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    let summary = artifact.kernel_summary().unwrap();
    assert!(summary.sectors > 1);
    let selected = summary.sectors - 1;
    let mut archive = artifact
        .open_program_archive(&path, KernelLoadOptions { validate: true })
        .unwrap();
    let mut reader = archive.select(artifact.selected_recipe().unwrap()).unwrap();
    let local = reader.load_sector(selected).unwrap();
    assert_eq!(local.sectors().len(), 1);
    let global_doc = threshold::native_document(&native).unwrap();
    let local_doc = threshold::native_document(&local).unwrap();
    assert_eq!(global_doc["resident"]["selection"]["kind"], "complete");
    assert_eq!(local_doc["resident"]["selection"]["kind"], "selected");
    assert_eq!(local_doc["global_proof_replayed"], false);
    assert!(
        !local_doc["lineage"]["endpoint_charts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let expected = serde_json::to_value(
        &artifact
            .catalogue()
            .unwrap()
            .sector(selected)
            .unwrap()
            .receipt
            .threshold,
    )
    .unwrap();
    fs::remove_file(artifact.data_path(&path).unwrap()).unwrap();
    let restored = Artifact::load_metadata(&path).unwrap();
    let doc = lightweight::document(&restored, &summary, Some(selected)).unwrap();
    assert_eq!(doc["binary_loaded"], false);
    assert_eq!(doc["selected_sector"]["threshold_record"], expected);
    assert!(
        !doc["threshold"]["contributions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let plain = ColorPolicy::for_stream(true, false);
    for width in [40, 80, 120] {
        let text =
            lightweight::render(&path, &restored, &summary, Some(selected), width, plain).unwrap();
        assert!(text.contains("Threshold lineage"));
        assert!(text.contains("binary not read"));
        let text =
            presentation::render(&path, &restored, &local, Some(0), false, width, plain).unwrap();
        assert!(text.contains("Threshold lineage"));
        assert!(!text.contains("older artifact"));
    }
}
#[test]
fn metadata_only_inspection_never_opens_binary_and_preserves_native_summary() {
    let (_dir, path, artifact, kernels) = fixture();
    let original_id = artifact.content_id.clone();
    let binary = artifact.data_path(&path).unwrap();
    let expected = kernels
        .sectors()
        .iter()
        .map(|s| s.statistics().clone())
        .collect::<Vec<_>>();
    let (restored, native) = Artifact::load(&path).unwrap();
    assert_eq!(restored.content_id, original_id);
    assert_eq!(native.to_bytes().unwrap(), kernels.to_bytes().unwrap());
    // A directory at the sibling path proves inspection never opens a native
    // artifact: a deep load fails rather than attempting any decoder.
    fs::remove_file(&binary).unwrap();
    fs::create_dir(&binary).unwrap();
    let loaded = Artifact::load_metadata(&path).unwrap();
    let summary = loaded.kernel_summary().unwrap();
    assert_eq!(summary.evaluator_statistics.as_ref().unwrap(), &expected);
    assert_eq!(summary.orders, native.orders());
    assert!(Artifact::load(&path).is_err());
    let doc = lightweight::document(&loaded, &summary, None).unwrap();
    assert_eq!(doc["binary_loaded"], false);
    assert_eq!(doc["binary_validated"], false);
    assert_eq!(doc["metadata_identity_validated"], false);
    let checked =
        Artifact::load_metadata_with_options(&path, KernelLoadOptions { validate: true }).unwrap();
    let checked_doc =
        lightweight::document(&checked, &checked.kernel_summary().unwrap(), None).unwrap();
    assert_eq!(checked_doc["metadata_identity_validated"], true);
    assert_eq!(checked_doc["binary_loaded"], false);
    assert_eq!(checked_doc["binary_validated"], false);
    assert!(doc["exact_coefficients"].is_null());
    assert!(doc["largest_sectors"].as_array().unwrap().len() <= 10);
    let selected = lightweight::document(&loaded, &summary, Some(0)).unwrap();
    assert_eq!(selected["selected_sector"]["id"], 0);
    assert!(selected["selected_sector"]["content_id"].is_null());
    assert!(
        !selected["selected_sector"]["chart_previews"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(lightweight::document(&loaded, &summary, Some(summary.sectors)).is_err());
    let colors = ColorPolicy::for_stream(true, false);
    for width in [40, 80, 120] {
        let text = lightweight::render(&path, &loaded, &summary, Some(0), width, colors).unwrap();
        assert!(text.contains("pre-subtraction preview"));
        assert!(text.contains("binary not read"));
        assert!(text.contains("Positive measure"));
    }
    // A sparse multi-GiB sibling costs no read or evaluator restoration.
    fs::remove_dir(&binary).unwrap();
    fs::File::create(&binary)
        .unwrap()
        .set_len(1_500_000_000)
        .unwrap();
    let sparse = Artifact::load_metadata(&path).unwrap();
    assert_eq!(sparse.content_id, original_id);
    assert_eq!(
        lightweight::document(&sparse, &sparse.kernel_summary().unwrap(), None).unwrap()["largest_sectors"],
        doc["largest_sectors"]
    );
    fs::remove_file(&binary).unwrap();
    assert!(Artifact::load_metadata(&path).is_ok());
}
#[test]
fn missing_and_future_indexes_are_explicitly_unknown_without_binary_fallback() {
    let (_dir, path, mut artifact, kernels) = fixture();
    let original_id = artifact.content_id.clone();
    let original_data = kernels.to_bytes().unwrap();
    artifact.inspection = None;
    artifact.programs.as_mut().unwrap().inspection.clear();
    metadata(&path, &artifact);
    let legacy = Artifact::load_metadata(&path).unwrap();
    let summary = legacy.kernel_summary().unwrap();
    let doc = lightweight::document(&legacy, &summary, Some(0)).unwrap();
    assert!(doc["selected_sector"]["chart_previews"].is_null());
    assert!(doc["retained_metadata_available"].is_null());
    assert!(
        lightweight::render(
            &path,
            &legacy,
            &summary,
            None,
            100,
            ColorPolicy::for_stream(true, false)
        )
        .unwrap()
        .contains("not indexed")
    );
    let (full, restored) = Artifact::load(&path).unwrap();
    assert_eq!(full.content_id, original_id);
    assert_eq!(restored.to_bytes().unwrap(), original_data);
    assert!(document(&full, &restored).unwrap()["generation_metadata"].is_object());
    // An unknown future display-index schema does not invalidate old native data.
    let mut raw: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
        serde_json::from_slice(&fs::read(crate::artifact::paths(&path).unwrap().0).unwrap())
            .unwrap();
    raw.insert(
        "inspection".into(),
        serde_json::value::RawValue::from_string(r#"{"version":999,"future":"schema"}"#.into())
            .unwrap(),
    );
    let mut programs: serde_json::Value = serde_json::from_str(raw["programs"].get()).unwrap();
    let recipe = serde_json::to_value(artifact.selected_recipe().unwrap()).unwrap();
    programs["inspection"] = serde_json::json!({
        recipe.as_str().unwrap(): {"version":999,"future":"schema"}
    });
    raw.insert(
        "programs".into(),
        serde_json::value::RawValue::from_string(serde_json::to_string(&programs).unwrap())
            .unwrap(),
    );
    fs::write(
        crate::artifact::paths(&path).unwrap().0,
        serde_json::to_vec_pretty(&raw).unwrap(),
    )
    .unwrap();
    assert!(
        Artifact::load_metadata(&path)
            .unwrap()
            .inspection_index()
            .is_none()
    );
    assert_eq!(
        Artifact::load(&path).unwrap().1.to_bytes().unwrap(),
        original_data
    );
    // Identity recomputation is explicitly requested, including metadata-only inspection.
    raw.insert(
        "content_id".into(),
        serde_json::value::RawValue::from_string("\"wrong\"".into()).unwrap(),
    );
    fs::write(
        crate::artifact::paths(&path).unwrap().0,
        serde_json::to_vec_pretty(&raw).unwrap(),
    )
    .unwrap();
    assert!(Artifact::load_metadata(&path).is_ok());
    assert!(
        Artifact::load_metadata_with_options(&path, KernelLoadOptions { validate: true }).is_err()
    );
}
#[test]
fn metadata_inspection_tolerates_incompatible_native_dependencies_but_deep_does_not() {
    let (_dir, path, artifact, kernels) = fixture();
    let mut provenance = artifact.provenance;
    provenance.dependencies[0].revision = "another native revision".into();
    Artifact::new(&kernels, provenance)
        .unwrap()
        .save(&path)
        .unwrap();
    let loaded = Artifact::load_metadata(&path).unwrap();
    assert!(!loaded.dependencies_compatible());
    assert_eq!(
        lightweight::document(&loaded, &loaded.kernel_summary().unwrap(), None).unwrap()["dependencies_compatible"],
        false
    );
    assert!(Artifact::load(&path).is_err());
}
#[test]
fn expression_only_binary_is_rejected_without_preventing_metadata_inspection() {
    let (_dir, path, artifact, _) = fixture();
    let binary = artifact.data_path(&path).unwrap();
    for expression_only in [
        include_bytes!("../../../fastsecdec/tests/fixtures/kernel-v1-triangle.json").as_slice(),
        include_bytes!("../../../fastsecdec/tests/fixtures/kernel-v2-triangle.json").as_slice(),
    ] {
        fs::write(&binary, expression_only).unwrap();
        let loaded = Artifact::load_metadata(&path).unwrap();
        assert_eq!(loaded.content_id, artifact.content_id);
        let error = Artifact::load(&path)
            .err()
            .expect("expression-only binary accepted");
        assert!(
            error
                .to_string()
                .contains("unsupported program archive header"),
            "{error}"
        );
    }
}

#[test]
fn observed_loading_keeps_preflight_cancellation_and_complete_boundaries() {
    use crate::artifact::ArtifactLoadProgress;
    use fastsecdec::kernel::KernelLoadProgress;
    use std::ops::ControlFlow;
    let (_dir, path, _, kernels) = fixture();
    let mut events = Vec::new();
    let (_, loaded) = Artifact::load_observed(
        &path,
        |_| Ok(()),
        |event| {
            events.push(event.clone());
            ControlFlow::Continue(())
        },
    )
    .unwrap();
    assert_eq!(loaded.to_bytes().unwrap(), kernels.to_bytes().unwrap());
    assert!(matches!(
        events.first(),
        Some(ArtifactLoadProgress::Metadata)
    ));
    assert!(matches!(
        events.last(),
        Some(ArtifactLoadProgress::Native(KernelLoadProgress::Complete))
    ));
    let read = events
        .iter()
        .filter_map(|event| {
            if let ArtifactLoadProgress::ReadingBinary { completed, total } = event {
                Some((*completed, *total))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(read.windows(2).all(|w| w[0].0 <= w[1].0));
    // Indexed loading streams records directly; it does not first fill a
    // monolithic byte buffer. Native restoring events provide the progress.
    assert_eq!(read.first().unwrap().0, 0);
    let restored = events
        .iter()
        .filter_map(|event| {
            if let ArtifactLoadProgress::Native(KernelLoadProgress::Restoring(p)) = event {
                Some((p.completed, p.total))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(restored.first(), Some(&(0, kernels.sectors().len())));
    assert_eq!(
        restored.last(),
        Some(&(kernels.sectors().len(), kernels.sectors().len()))
    );
    let mut cancelled_events = Vec::new();
    assert!(Artifact::load_observed(&path, |_|Ok(()),|event| {
        cancelled_events.push(event.clone());
        if matches!(event,ArtifactLoadProgress::Native(KernelLoadProgress::Restoring(p)) if p.completed==0) {ControlFlow::Break(())} else {ControlFlow::Continue(())}
    }).is_err());
    assert!(!cancelled_events.iter().any(|e| matches!(
        e,
        ArtifactLoadProgress::Native(KernelLoadProgress::Complete)
    )));
    fs::remove_file(
        Artifact::load_metadata(&path)
            .unwrap()
            .data_path(&path)
            .unwrap(),
    )
    .unwrap();
    let mut phases = Vec::new();
    let error = Artifact::load_observed(
        &path,
        |_| Err("preflight sentinel".into()),
        |event| {
            phases.push(event.clone());
            ControlFlow::Continue(())
        },
    )
    .err()
    .unwrap();
    assert_eq!(error.to_string(), "preflight sentinel");
    assert_eq!(phases.len(), 1);
    let error = Artifact::load_observed(
        &path,
        |_| Ok(()),
        |event| {
            if matches!(event, ArtifactLoadProgress::ReadingBinary { .. }) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("cancelled"));
}
