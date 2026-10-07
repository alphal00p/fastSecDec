//! A new process must recover built-in aliases and numeric callbacks solely
//! from the portable artifact, without a preceding generation pass.
use std::{ops::ControlFlow, process::Command};

#[test]
fn gamma_artifact_loads_in_a_fresh_process() {
    let temporary = tempfile::tempdir().unwrap();
    for method in ["expanded_expression", "coefficient_series"] {
        for mode in ["write", "read"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "artifact_process_child",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("FASTSECDEC_ARTIFACT_PROCESS_MODE", mode)
                .env("FASTSECDEC_ARTIFACT_COEFFICIENT_METHOD", method)
                .env("FASTSECDEC_ARTIFACT_PROCESS_DIR", temporary.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode} child failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn artifact_process_child() {
    let Ok(mode) = std::env::var("FASTSECDEC_ARTIFACT_PROCESS_MODE") else {
        return;
    };
    let directory =
        std::path::PathBuf::from(std::env::var_os("FASTSECDEC_ARTIFACT_PROCESS_DIR").unwrap());
    let artifact = directory.join("gamma.fsd");
    let expected = directory.join("expected.json");
    if mode == "write" {
        use fastsecdec::{
            Atom,
            generation::{
                CoefficientExpansionMethod, CoefficientExpansionOptions, GenerationOptions,
                generate,
            },
            parametric::{
                FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
            },
        };
        use symbolica::{parse, symbol};
        let input = ParametricIntegrand::new(
            vec![symbol!("fresh_artifact::x")],
            symbol!("fresh_artifact::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                parse!("(2+3𝑖)*gamma(3+fresh_artifact::eps)"),
                vec![Atom::one()],
                vec![PolynomialFactor::new(
                    parse!("1+fresh_artifact::x"),
                    parse!("fresh_artifact::eps"),
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap();
        let generated = generate(
            &input,
            &GenerationOptions {
                max_order: 4,
                coefficient_expansion: CoefficientExpansionOptions {
                    method: match std::env::var("FASTSECDEC_ARTIFACT_COEFFICIENT_METHOD")
                        .unwrap()
                        .as_str()
                    {
                        "expanded_expression" => CoefficientExpansionMethod::Physical,
                        "coefficient_series" => CoefficientExpansionMethod::NativeNamed,
                        other => panic!("unknown coefficient method {other}"),
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut kernels = generated.compile().unwrap();
        kernels
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        assert_eq!(kernels.orders(), &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
        let mut values = vec![0.0; kernels.orders().len()];
        let report = kernels
            .evaluation_context(0, Default::default())
            .unwrap()
            .evaluate_weighted(&[0.25], 1e40, &mut values)
            .unwrap();
        assert!(report.replayed);
        assert!(values.iter().all(|value| value.is_finite()));
        assert!((values[0] / 1e40 - 1.0).abs() < 1e-13);
        assert!((values[1] / 1e40 - 1.5).abs() < 1e-13);
        std::fs::write(artifact, kernels.to_bytes().unwrap()).unwrap();
        std::fs::write(expected, serde_json::to_vec(&values).unwrap()).unwrap();
    } else {
        assert_eq!(mode, "read");
        let mut kernels =
            fastsecdec::kernel::KernelSet::from_bytes(&std::fs::read(artifact).unwrap()).unwrap();
        kernels
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        let expected: Vec<f64> = serde_json::from_slice(&std::fs::read(expected).unwrap()).unwrap();
        let count = kernels.orders().len();
        let mut worker = kernels.evaluation_context(0, Default::default()).unwrap();
        let actual = std::thread::spawn(move || {
            let mut values = vec![0.0; count];
            let report = worker
                .evaluate_weighted(&[0.25], 1e40, &mut values)
                .unwrap();
            assert!(report.replayed);
            assert!(report.precision.bits >= 256);
            values
        })
        .join()
        .unwrap();
        for (actual, expected) in actual.iter().zip(&expected) {
            assert!(
                (actual - expected).abs() < 1e-13 * expected.abs().max(1.0),
                "{actual} != {expected}"
            );
        }
    }
}
