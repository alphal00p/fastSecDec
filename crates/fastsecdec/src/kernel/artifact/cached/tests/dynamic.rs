//! A full dynamic owner crosses a process boundary using the native JIT codec.
use super::*;
use crate::{
    contour::{
        ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    generation::{GenerationMode, SubtractionStrategy},
    kernel::{PrecisionClass, ProgramRecipe},
    parametric::FactorSemantics,
    results::ResultScope,
};
use std::collections::BTreeMap;
use symbolica::atom::AtomCore;

pub(super) fn fixture(recipe: ProgramRecipe) -> KernelSet {
    let x = symbol!("cached_dynamic::x");
    let eps = symbol!("cached_dynamic::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() + Atom::i(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    Atom::num(2) + Atom::var(x) + Atom::var(x).pow(3),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::one() + Atom::var(x).pow(2),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            mode: GenerationMode::Symbolic,
            subtraction: SubtractionStrategy::IntegrateByParts,
            program_recipe: recipe,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Symjit,
        horner_iterations: 0,
        ..Default::default()
    })
    .unwrap()
}

fn samples(kernels: &mut KernelSet) -> Vec<f64> {
    let construction = match kernels.program_recipe() {
        ProgramRecipe::DynamicPolynomialV1 => DynamicConstruction::Polynomial,
        ProgramRecipe::DynamicSignAwareV1 => DynamicConstruction::SignAware,
        _ => panic!("dynamic fixture"),
    };
    let mut result = Vec::new();
    for (cap, policy) in [
        (0.125, ContourValidation::Pilot),
        (0.0625, ContourValidation::Always),
    ] {
        kernels
            .bind_parameters_with_contour(
                &BTreeMap::new(),
                &ContourSettings {
                    deformation: ContourMode::Dynamical {
                        safety_fraction: 0.8,
                        lambda_cap: cap,
                        displacement_cap: 1.,
                        construction,
                    },
                    validation: ContourValidationOptions {
                        policy,
                        pilot_points: 2,
                    },
                },
            )
            .unwrap();
        assert!(
            kernels
                .validate_integration_readiness(&ResultScope::FullIntegral)
                .is_err()
        );
        for chart in kernels.contour_validation_charts() {
            for point in [0.25, 0.75] {
                kernels
                    .validate_contour_point(chart.chart_index, &vec![point; chart.dimension], true)
                    .unwrap();
            }
        }
        assert!(kernels.finish_contour_pilot().unwrap().pilot_complete);
        kernels
            .validate_integration_readiness(&ResultScope::FullIntegral)
            .unwrap();
        result.extend(kernels.exact_coefficients());
        for sector in kernels.sectors_mut() {
            let mut output = vec![0.; sector.output_count()];
            sector
                .evaluate(&vec![0.37; sector.dimension()], &mut output)
                .unwrap();
            let primary = output.clone();
            result.extend(&primary);
            for class in [PrecisionClass::DoubleFloat, PrecisionClass::Arbitrary] {
                sector
                    .evaluate_distance_class(&mut output, 1., class)
                    .unwrap();
                for (a, b) in output.iter().zip(&primary) {
                    assert!((a - b).abs() < 1e-10 * b.abs().max(1.));
                }
            }
        }
    }
    result
}

#[test]
fn dynamic_native_primary_restores_callbacks_in_fresh_process_and_rebinds_pilot_owners() {
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        let mut kernels = fixture(recipe);
        let bytes = kernels.to_bytes().unwrap();
        assert!(bytes.starts_with(MAGIC));
        let expected = samples(&mut kernels);
        assert_eq!(
            kernels.to_bytes().unwrap(),
            bytes,
            "bound values and pilot state are not saved"
        );
        drop(kernels);
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dynamic.bin");
        std::fs::write(&path, bytes).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "kernel::artifact::cached::tests::dynamic::child",
                "--ignored",
                "--test-threads=1",
            ])
            .env("FASTSECDEC_NATIVE_PRIMARY_CHILD", &path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let actual: Vec<f64> =
            serde_json::from_slice(&std::fs::read(path.with_extension("result")).unwrap()).unwrap();
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected) {
            assert!((a - b).abs() < 1e-11 * b.abs().max(1.));
        }
    }
}

#[test]
#[ignore = "spawned by the native primary owner restoration test"]
fn child() {
    let path =
        std::path::PathBuf::from(std::env::var_os("FASTSECDEC_NATIVE_PRIMARY_CHILD").unwrap());
    for i in 0..19 {
        let _ = symbol!(format!("cache_child_scramble::x{i}"));
    }
    let bytes = std::fs::read(&path).unwrap();
    let mut kernels =
        KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: true }).unwrap();
    assert!(
        kernels
            .sectors()
            .iter()
            .all(|s| s.primary_evaluator_restoration() == Restoration::CacheRestored)
    );
    let mut output = vec![0.; kernels.orders().len()];
    assert!(
        kernels.sectors_mut()[0]
            .evaluate(&[0.37], &mut output)
            .is_err(),
        "saved pilot/binding must not be reused"
    );
    let actual = samples(&mut kernels);
    let mut clone = kernels.try_clone().unwrap();
    drop(kernels);
    let repeated = std::thread::spawn(move || samples(&mut clone))
        .join()
        .unwrap();
    assert_eq!(actual, repeated);
    std::fs::write(
        path.with_extension("result"),
        serde_json::to_vec(&actual).unwrap(),
    )
    .unwrap();
}
