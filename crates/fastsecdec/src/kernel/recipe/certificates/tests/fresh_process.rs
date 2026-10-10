use super::*;

#[test]
fn saved_checker_replays_in_a_fresh_process_without_generation_sources() {
    let (owner, source, _) = fixture(CompilationSettings::default());
    let exact = exact_request(&source, &owner.certificates().unwrap()[0].faces);
    let owner = owner.for_payload(&[], &[exact], None).unwrap();
    let saved = SavedProgramDescriptorV2::from_native(&owner).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("owner.bin");
    std::fs::write(
        &input,
        bincode::serde::encode_to_vec(saved, bincode::config::standard()).unwrap(),
    )
    .unwrap();
    drop(owner);
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "kernel::recipe::certificates::tests::fresh_process::saved_checker_child",
            "--ignored",
            "--test-threads=1",
        ])
        .env("FASTSECDEC_V11_CERTIFICATE_CHILD", &input)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::read(input.with_extension("verified")).unwrap(),
        b"native arithmetic restored"
    );
}

#[test]
#[ignore = "spawned only by the fresh-process certificate transport test"]
fn saved_checker_child() {
    let path = std::path::PathBuf::from(
        std::env::var_os("FASTSECDEC_V11_CERTIFICATE_CHILD").expect("issued fixture path"),
    );
    let bytes = std::fs::read(&path).unwrap();
    let (saved, used): (SavedProgramDescriptorV2, _) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    let owner = saved.restore().unwrap();
    assert!(owner.charts.is_empty());
    assert_eq!(owner.certificates().unwrap().len(), 1);
    assert!(owner.certificates().unwrap()[0].chart_index.is_none());
    for certificate in owner.certificates().unwrap() {
        let (raw, coefficients) = certificate.restore_arithmetic().unwrap();
        let mut raw = raw.map_coeff(&|value| Complex::new(value.re.to_f64(), value.im.to_f64()));
        let mut outputs = vec![Complex::new(0., 0.); raw.get_output_len()];
        raw.evaluate(
            &vec![Complex::new(0.2, 0.); raw.get_input_len()],
            &mut outputs,
        );
        assert!(
            outputs
                .iter()
                .all(|value| value.re.is_finite() && value.im.is_finite())
        );
        assert_eq!(
            coefficients.get_output_len(),
            certificate.structure.coefficient_count
        );
    }
    std::fs::write(
        path.with_extension("verified"),
        b"native arithmetic restored",
    )
    .unwrap();
}
