use super::*;

#[test]
fn descriptorless_native_json_cannot_inherit_a_callers_dynamic_owner() {
    use crate::contour::functions::dynamic::{RootProgram, strength};
    use symbolica::atom::{Atom, AtomCore};
    let template = super::super::load_tests::template();
    let helper = RootProgram::build(1).unwrap();
    let inputs = template.sectors[0]
        .parameters
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let expression = strength(
        &helper,
        &[Atom::one() + inputs[0].pow(2)],
        &Atom::num((4, 5)),
        &Atom::one(),
    )
    .unwrap();
    let exact = Atom::evaluator_multiple(
        &vec![expression; template.coefficient_orders.len()],
        &inputs,
    )
    .build()
    .unwrap();
    let mut sectors = template
        .sectors
        .iter()
        .map(|sector| PortableSector {
            parameters: super::super::parameter_names(&sector.parameters),
            program: sector.program_bytes.to_vec(),
            cancellation_degree: sector.cancellation.degree(),
            cancellation_terms: sector.cancellation.terms().map(<[_]>::to_vec),
        })
        .collect::<Vec<_>>();
    sectors[0].program = program::encode(&exact).unwrap();
    let payload = Payload {
        version: 3,
        program_codec: CODEC.into(),
        compiler_policy: compiler_policy_with_settings(template.compilation_settings),
        orders: template.coefficient_orders.clone(),
        components: component_layout(template.coefficient_orders.len(), true),
        exact: template
            .exact_expressions
            .iter()
            .map(AtomCore::to_canonical_string)
            .collect(),
        precision: template.precision.clone(),
        sectors,
        metadata: template
            .metadata
            .as_ref()
            .map(PortableMetadata::from_native),
    };
    let (_, bytes) = encoded(&payload).unwrap();
    helper.prepare(|| {
        for validate in [false, true] {
            let error = KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate })
                .err()
                .expect("descriptorless native v3 must reject the semantic callback");
            assert!(
                error
                    .to_string()
                    .contains("selected descriptor preparation scope"),
                "{error}"
            );
        }
        // Failed nested loading restores the valid caller's preparation.
        let mut evaluator = exact.map_coeff(&|value| value.re.to_f64());
        let mut output = vec![0.0; template.coefficient_orders.len()];
        evaluator.evaluate(&vec![0.5; inputs.len()], &mut output);
        assert!((output[0] - 0.8 / 1.25_f64.sqrt()).abs() < 1e-14);
    });
}

#[test]
fn native_json_identity_is_optional_but_layout_remains_mandatory() {
    let kernels = super::super::load_tests::template();
    let payload = PayloadRef {
        version: 3,
        program_codec: CODEC,
        compiler_policy: &compiler_policy_with_settings(kernels.compilation_settings),
        orders: &kernels.coefficient_orders,
        components: &kernels.components,
        exact: kernels
            .exact_expressions
            .iter()
            .map(symbolica::atom::AtomCore::to_canonical_string)
            .collect(),
        precision: &kernels.precision,
        sectors: kernels
            .sectors
            .iter()
            .map(|sector| PortableSectorRef {
                parameters: super::super::parameter_names(&sector.parameters),
                program: &sector.program_bytes,
                cancellation_degree: sector.cancellation.degree(),
                cancellation_terms: sector.cancellation.terms(),
            })
            .collect(),
        metadata: kernels.metadata.as_ref().map(PortableMetadata::from_native),
    };
    let (_, bytes) = encoded(&payload).unwrap();
    for validate in [false, true] {
        let loaded =
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate }).unwrap();
        assert_eq!(loaded.coefficient_orders, kernels.coefficient_orders);
        assert_eq!(loaded.artifact_bytes().unwrap(), bytes);
    }
    let mut record: Artifact = serde_json::from_slice(&bytes).unwrap();
    record.content_id = "f".repeat(64);
    let changed = serde_json::to_vec(&record).unwrap();
    assert!(KernelSet::from_bytes(&changed).is_ok());
    assert!(
        KernelSet::from_bytes_with_options(&changed, KernelLoadOptions { validate: true }).is_err()
    );
    record.payload.components.clear();
    let (_, broken_layout) = encoded(&record.payload).unwrap();
    for validate in [false, true] {
        assert!(
            KernelSet::from_bytes_with_options(&broken_layout, KernelLoadOptions { validate })
                .is_err()
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn borrowed_v3_encoding_matches_the_previous_owned_wire_and_hash() {
    // Transport-only data exercises every byte value, escaping, signed orders,
    // component layout, optional terms and numeric policy without another codec.
    // Native decoding and scientific replay are covered by kernel_artifacts and
    // artifact_process using real native programs.
    let owned = Payload {
        version: 3,
        program_codec: CODEC.into(),
        compiler_policy: compiler_policy().into(),
        orders: vec![-2, -1, 0],
        components: component_layout(3, true),
        exact: vec!["0".into(), "-1/3".into(), "2+3𝑖".into()],
        precision: PrecisionPolicy::default(),
        sectors: vec![
            PortableSector {
                parameters: vec!["a\"b\\c".into()],
                program: (0..=u8::MAX).cycle().take(65_536).collect(),
                cancellation_degree: 2,
                cancellation_terms: Some(vec![vec![0], vec![1], vec![2]]),
            },
            PortableSector {
                parameters: vec!["δ".into()],
                program: vec![0, 255],
                cancellation_degree: 0,
                cancellation_terms: None,
            },
        ],
        metadata: None,
    };
    let borrowed = PayloadRef {
        version: owned.version,
        program_codec: &owned.program_codec,
        compiler_policy: &owned.compiler_policy,
        orders: &owned.orders,
        components: &owned.components,
        exact: owned.exact.clone(),
        precision: &owned.precision,
        sectors: owned
            .sectors
            .iter()
            .map(|sector| PortableSectorRef {
                parameters: sector.parameters.clone(),
                program: &sector.program,
                cancellation_degree: sector.cancellation_degree,
                cancellation_terms: sector.cancellation_terms.as_deref(),
            })
            .collect(),
        metadata: None,
    };
    let mut previous_hash = blake3::Hasher::new();
    previous_hash.update(b"fastsecdec-portable-kernel-v3:symbolica-3:symjit-2.26:f64");
    previous_hash.update(&serde_json::to_vec(&owned).unwrap());
    let previous_id = previous_hash.finalize().to_hex().to_string();
    let (actual_id, actual_bytes) = encoded(&borrowed).unwrap();
    assert_eq!(actual_id, previous_id);
    assert_eq!(content_id(&owned).unwrap(), previous_id);
    let previous_bytes = serde_json::to_vec(&Artifact {
        content_id: previous_id,
        payload: owned,
    })
    .unwrap();
    assert_eq!(actual_bytes, previous_bytes);
}

#[cfg(feature = "native")]
#[test]
fn compiler_policy_checks_the_linked_code_and_narrow_legacy_spelling() {
    assert!(compiler_policy_matches(
        "symjit-version-code=22700:O2:direct:horner-iterations=0",
        22700
    ));
    assert!(!compiler_policy_matches(
        "symjit-version-code=22604:O2:direct:horner-iterations=0",
        22700
    ));
    let legacy = "symjit-2.26.4:O2:direct:horner-iterations=0";
    assert!(compiler_policy_matches(legacy, 22604));
    assert!(!compiler_policy_matches(legacy, 22700));
    assert!(!compiler_policy_matches(
        "symjit-version-code=22604:O3:direct:horner-iterations=0",
        22604
    ));
    assert!(!compiler_policy_matches(
        "symjit-2.26.0:O2:direct:horner-iterations=0",
        22604
    ));
}

#[test]
fn contour_jacobian_policy_preserves_legacy_default_and_records_dual() {
    use crate::contour::ContourJacobian;
    let symbolic = CompilationSettings::default();
    let historical = serde_json::to_value(symbolic).unwrap();
    assert!(historical.get("contour_jacobian").is_none());
    assert_eq!(
        serde_json::from_value::<CompilationSettings>(historical).unwrap(),
        symbolic
    );
    let symbolic_policy = compiler_policy_with_settings(symbolic);
    assert_eq!(settings_from_policy(&symbolic_policy), Some(symbolic));
    let dual = symbolic
        .resolve_contour_jacobian(ContourJacobian::Dual)
        .unwrap();
    let dual_policy = compiler_policy_with_settings(dual);
    assert_ne!(symbolic_policy, dual_policy);
    assert_eq!(settings_from_policy(&dual_policy), Some(dual));
    assert_eq!(
        serde_json::to_value(dual).unwrap()["contour_jacobian"],
        "dual"
    );
    assert!(
        dual.resolve_contour_jacobian(ContourJacobian::Symbolic)
            .is_err()
    );
    assert_eq!(
        dual.resolve_contour_jacobian(ContourJacobian::Dual)
            .unwrap(),
        dual
    );
}
