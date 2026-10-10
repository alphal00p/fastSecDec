//! Wire-version gates use actual native payloads and owner serialization.
use super::*;

fn decode_v11(bytes: &[u8]) -> PayloadV11 {
    let (envelope, used): (Envelope, usize) = bincode::decode_from_slice(
        bytes.strip_prefix(MAGIC_V11).unwrap(),
        bincode::config::standard(),
    )
    .unwrap();
    assert_eq!(used, bytes.len() - MAGIC_V11.len());
    crate::contour::functions::register();
    crate::contour::functions::dynamic::register();
    let context = State::import(&mut envelope.state.as_slice(), None).unwrap();
    let (payload, used) = bincode::decode_from_slice_with_context(
        &envelope.payload,
        bincode::config::standard(),
        context,
    )
    .unwrap();
    assert_eq!(used, envelope.payload.len());
    payload
}

#[test]
fn generated_v11_keeps_exact_mathematics_and_associations_in_separate_fields() {
    use crate::{
        generation::GenerationMode,
        kernel::ProgramRecipe,
        parametric::{FactorRole, FactorSemantics, PolynomialFactor},
    };
    let x = symbol!("exact_request_wire::x");
    let eps = symbol!("exact_request_wire::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::num(3) - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    Atom::num(2) + Atom::var(x) + Atom::var(x).pow(3),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let mut generated = generate(
            &input,
            &GenerationOptions {
                mode,
                program_recipe: ProgramRecipe::DynamicPolynomialV1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        // This is a transport fixture, not another scientific integral. Its
        // genuine generated offset can cancel all radii, so retain one actual
        // generated face request explicitly to exercise the nonempty codec.
        let source = &generated.dynamic_check_sources()[0];
        let root = source
            .full_strength
            .replace(Atom::var(source.parameters[0]))
            .with(0);
        generated.exact_coefficients[0] += root;
        let original = generated.exact_coefficients().to_vec();
        let bytes = generated
            .to_kernel_bytes(PrecisionPolicy::default())
            .unwrap();
        drop(generated);
        let payload = decode_v11(&bytes);
        assert_eq!(payload.base.exact, original);
        assert!(!payload.exact_requests.is_empty());
        let requests =
            merge_exact_requests(&payload.base.exact, payload.exact_requests.clone()).unwrap();
        assert_eq!(requests.len(), payload.exact_requests.len());
        let owner = payload.descriptor.restore().unwrap();
        let selected = owner
            .for_payload_with_requests(&[], &payload.base.exact, None, &requests)
            .unwrap();
        assert!(selected.charts().is_empty());
        assert!(!selected.certificates().unwrap().is_empty());
        assert!(
            crate::contour::functions::dynamic::requests::referenced_bundles(&payload.base.exact)
                .unwrap()
                .is_empty(),
            "diagnostic tags must not change mathematical exact coefficient equality"
        );
        let legacy = crate::kernel::recipe::SavedProgramDescriptor::from_native(&owner);
        let (_, legacy) =
            encode_with_descriptor(payload.base, Some(SavedDescriptor::V10(legacy))).unwrap();
        for validate in [false, true] {
            let error = KernelSet::from_bytes_with_options(&legacy, KernelLoadOptions { validate })
                .err()
                .unwrap()
                .to_string();
            assert!(
                error.contains("dynamic v10") && error.contains("regenerate"),
                "{error}"
            );
        }
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("exact.bin");
        std::fs::write(&path, bytes).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "kernel::artifact::binary::tests::v11::exact_association_child",
                "--ignored",
                "--test-threads=1",
            ])
            .env("FASTSECDEC_EXACT_ASSOCIATION_CHILD", &path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read(path.with_extension("verified")).unwrap(),
            b"native exact associations restored"
        );
    }
}

#[test]
#[ignore = "spawned by the actual v11 exact-association transport test"]
fn exact_association_child() {
    let path = std::path::PathBuf::from(
        std::env::var_os("FASTSECDEC_EXACT_ASSOCIATION_CHILD").expect("issued fixture"),
    );
    let payload = decode_v11(&std::fs::read(&path).unwrap());
    let owner = payload.descriptor.restore().unwrap();
    let requests = merge_exact_requests(&payload.base.exact, payload.exact_requests).unwrap();
    assert!(!requests.is_empty());
    let selected = owner
        .for_payload_with_requests(&[], &payload.base.exact, None, &requests)
        .unwrap();
    assert!(!selected.certificates().unwrap().is_empty());
    for certificate in selected.certificates().unwrap() {
        certificate.restore_arithmetic().unwrap();
    }
    std::fs::write(
        path.with_extension("verified"),
        b"native exact associations restored",
    )
    .unwrap();
}

#[test]
fn explicit_empty_v11_certificates_preserve_legacy_readers_and_native_programs() {
    let mut kernels = kernel(Atom::one());
    let historical = kernels.artifact_bytes().unwrap().to_vec();
    let owner = crate::kernel::NativeProgramDescriptor::static_recipe(
        crate::kernel::ProgramRecipe::UndeformedV1,
    )
    .unwrap()
    .with_certificates(vec![])
    .unwrap();
    kernels.program_descriptor = Some(owner);
    let (identity, bytes) = compiled(&kernels).unwrap();
    assert!(bytes.starts_with(MAGIC_V11));
    for validate in [false, true] {
        let restored =
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate }).unwrap();
        assert_eq!(restored.content_id(), identity);
        assert_eq!(
            restored
                .program_descriptor()
                .unwrap()
                .certificates()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(restored.exact_expressions, kernels.exact_expressions);
        assert_eq!(restored.sectors.len(), kernels.sectors.len());
        assert_eq!(
            restored.sectors[0].program_bytes,
            kernels.sectors[0].program_bytes
        );
    }
    let legacy =
        KernelSet::from_bytes_with_options(&historical, KernelLoadOptions { validate: true })
            .unwrap();
    assert!(legacy.program_descriptor().is_none());
}

#[test]
fn v11_descriptor_version_and_trailing_bytes_are_rejected_without_digest_checks() {
    let kernels = kernel(Atom::one());
    let owner = crate::kernel::NativeProgramDescriptor::static_recipe(
        crate::kernel::ProgramRecipe::UndeformedV1,
    )
    .unwrap()
    .with_certificates(vec![])
    .unwrap();
    let saved = crate::kernel::recipe::SavedProgramDescriptorV2::from_native(&owner).unwrap();
    let mut json = serde_json::to_value(saved).unwrap();
    json["version"] = 3.into();
    let corrupt = serde_json::from_value(json).unwrap();
    let (_, bytes) =
        encode_with_descriptor(payload(&kernels), Some(SavedDescriptor::V11(corrupt))).unwrap();
    for validate in [false, true] {
        let error = KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate })
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("certificate descriptor version"), "{error}");
    }
    let (_, mut bytes) = encode_with_descriptor(
        payload(&kernels),
        Some(SavedDescriptor::from_native(&owner).unwrap()),
    )
    .unwrap();
    bytes.push(0);
    assert!(
        KernelSet::from_bytes(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("trailing envelope")
    );
}
