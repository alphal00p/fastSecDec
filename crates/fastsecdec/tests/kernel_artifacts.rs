//! Scientific and format boundaries for exact native IR and rejected expression caches.
use fastsecdec::{
    Atom,
    generation::{GenerationOptions, generate},
    kernel::{KernelSet, PrecisionPolicy},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

#[path = "inspection_metadata/mod.rs"]
mod inspection_metadata;
#[path = "kernel_artifacts/literal_zero.rs"]
mod literal_zero;

fn generated() -> fastsecdec::generation::GeneratedIntegral {
    generate(
        &ParametricIntegrand::new(
            vec![symbol!("artifact_v3::x")],
            symbol!("artifact_v3::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                parse!("(2+3𝑖)*gamma(1+artifact_v3::eps)"),
                vec![Atom::Zero],
                vec![PolynomialFactor::new(
                    parse!("1+artifact_v3::x"),
                    parse!("artifact_v3::eps-1"),
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap(),
        &GenerationOptions {
            max_order: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
}

#[test]
fn expression_only_v1_v2_are_rejected_without_restoring_evaluators() {
    for bytes in [
        include_bytes!("fixtures/kernel-v1-triangle.json").as_slice(),
        include_bytes!("fixtures/kernel-v2-triangle.json").as_slice(),
    ] {
        for bytes in [
            bytes.to_vec(),
            serde_json::to_vec_pretty(&serde_json::from_slice::<serde_json::Value>(bytes).unwrap())
                .unwrap(),
        ] {
            let mut callbacks = Vec::new();
            let error = KernelSet::from_bytes_with_progress(&bytes, |progress| {
                callbacks.push(format!("{progress:?}"));
                ControlFlow::Continue(())
            })
            .err()
            .expect("expression-only artifact was accepted");
            assert!(
                error
                    .to_string()
                    .contains("expression-only kernel artifacts are unsupported"),
                "{error}"
            );
            assert_eq!(callbacks, ["Decoding"]);
        }
    }
}

#[test]
fn native_program_is_saved_without_materialization_and_stays_immutable_after_work() {
    let generated = generated();
    let before_compile = generated
        .to_kernel_bytes(PrecisionPolicy::default())
        .unwrap();
    let mut compiled = generated.compile().unwrap();
    let compiled_bytes = compiled.to_bytes().unwrap();
    let retained_address = compiled.artifact_bytes().unwrap().as_ptr();
    assert_eq!(compiled.artifact_bytes().unwrap(), compiled_bytes);
    let eager = compiled.sectors()[0].primary_evaluator_restoration()
        == fastsecdec::kernel::PrimaryEvaluatorRestoration::Eager;
    if eager {
        assert_eq!(compiled_bytes, before_compile);
    } else {
        assert!(compiled_bytes.starts_with(b"FastSecDec\0binserde\x0d"));
    }
    assert!(before_compile.starts_with(b"FastSecDec\0binserde\x09"));
    assert!(serde_json::from_slice::<serde_json::Value>(&before_compile).is_err());
    assert!(compiled.sectors()[0].statistics().exact_program_bytes > 0);
    let mut restored = KernelSet::from_bytes(&before_compile).unwrap();
    let mut cached = KernelSet::from_bytes(&compiled_bytes).unwrap();
    assert_eq!(cached.to_bytes().unwrap(), compiled_bytes);
    assert_eq!(
        cached.sectors()[0].primary_evaluator_restoration(),
        if eager {
            fastsecdec::kernel::PrimaryEvaluatorRestoration::Eager
        } else {
            fastsecdec::kernel::PrimaryEvaluatorRestoration::CacheRestored
        }
    );
    assert_eq!(restored.orders(), [0, 0, 1, 1, 2, 2]);
    assert_eq!(compiled.content_id(), restored.content_id());
    assert_eq!(compiled.content_id(), cached.content_id());
    for point in [0.25, 1e-8] {
        let mut old = vec![0.0; 6];
        let mut new = vec![0.0; 6];
        let mut cache_values = vec![0.0; 6];
        compiled.sectors_mut()[0]
            .evaluate(&[point], &mut old)
            .unwrap();
        restored.sectors_mut()[0]
            .evaluate(&[point], &mut new)
            .unwrap();
        cached.sectors_mut()[0]
            .evaluate(&[point], &mut cache_values)
            .unwrap();
        assert!((new[0] - 2.0 / (1.0 + point)).abs() < 1e-12);
        assert!((new[1] - 3.0 / (1.0 + point)).abs() < 1e-12);
        for ((old, new), cached) in old.into_iter().zip(new).zip(cache_values) {
            assert!((old - new).abs() < 1e-11);
            assert!((old - cached).abs() < 1e-11);
        }
    }
    assert_eq!(restored.to_bytes().unwrap(), before_compile);
    assert_eq!(compiled.to_bytes().unwrap(), compiled_bytes);
    assert_eq!(cached.to_bytes().unwrap(), compiled_bytes);
    assert_eq!(
        compiled.artifact_bytes().unwrap().as_ptr(),
        retained_address
    );
    assert_eq!(restored.artifact_bytes().unwrap(), before_compile);
}

#[test]
fn binary_transport_retains_owned_and_borrowed_identity() {
    let bytes = generated()
        .to_kernel_bytes(PrecisionPolicy::default())
        .unwrap();
    let restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.artifact_bytes().unwrap(), bytes);
    let copied = restored.to_bytes().unwrap();
    assert_eq!(copied, bytes);
    assert_ne!(copied.as_ptr(), restored.artifact_bytes().unwrap().as_ptr());
}

#[test]
fn binary_integrity_header_and_exact_byte_exhaustion_are_enforced() {
    use fastsecdec::kernel::KernelLoadOptions;
    let bytes = generated()
        .to_kernel_bytes(PrecisionPolicy::default())
        .unwrap();
    let mut trailing = bytes.clone();
    trailing.push(0);
    let mut corrupt = bytes.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    let mut unsupported = bytes.clone();
    unsupported[b"FastSecDec\0binserde".len()] = 255;
    for bad in [
        Vec::new(),
        bytes[..bytes.len() - 1].to_vec(),
        trailing,
        unsupported,
    ] {
        for validate in [false, true] {
            assert!(
                KernelSet::from_bytes_with_options(&bad, KernelLoadOptions { validate }).is_err()
            );
        }
    }
    assert!(
        KernelSet::from_bytes_with_options(&corrupt, KernelLoadOptions { validate: true }).is_err()
    );
}

#[test]
fn configured_optimizer_policy_survives_binary_cold_load() {
    let settings = fastsecdec::kernel::CompilationSettings {
        horner_iterations: 2,
        cpe_rounds: Some(7),
        ..Default::default()
    };
    let original = generated()
        .compile_with_settings_parameters_and_progress(Default::default(), &[], settings, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    let bytes = original.artifact_bytes().unwrap();
    let loaded = KernelSet::from_bytes(bytes).unwrap();
    assert_eq!(loaded.compilation_settings(), &settings);
    assert_eq!(loaded.content_id(), original.content_id());
    assert_eq!(loaded.artifact_bytes().unwrap(), bytes);
}
