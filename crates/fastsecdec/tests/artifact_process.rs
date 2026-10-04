//! A new process must recover built-in aliases and numeric callbacks solely
//! from the portable artifact, without a preceding generation pass.
use std::{ops::ControlFlow, process::Command};

#[test]
fn gamma_artifact_loads_in_a_fresh_process() {
    let temporary = tempfile::tempdir().unwrap();
    for mode in ["write", "read"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "artifact_process_child", "--test-threads=1"])
            .env("FASTSECDEC_ARTIFACT_PROCESS_MODE", mode)
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
            generation::{GenerationOptions, generate},
            parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
        };
        use symbolica::{parse, symbol};
        let input = ParametricIntegrand::new(
            vec![symbol!("fresh_artifact::x")],
            symbol!("fresh_artifact::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                parse!("(2+3𝑖)*gamma(3+fresh_artifact::eps)"),
                vec![Atom::one()],
                vec![],
            )],
        )
        .unwrap();
        let generated = generate(
            &input,
            &GenerationOptions {
                max_order: 4,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut kernels = generated.compile().unwrap();
        assert_eq!(kernels.orders(), &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
        let mut values = vec![0.0; kernels.orders().len()];
        kernels.sectors_mut()[0]
            .evaluate(&[0.25], &mut values)
            .unwrap();
        std::fs::write(artifact, kernels.to_bytes().unwrap()).unwrap();
        std::fs::write(expected, serde_json::to_vec(&values).unwrap()).unwrap();
    } else {
        assert_eq!(mode, "read");
        let kernels =
            fastsecdec::kernel::KernelSet::from_bytes(&std::fs::read(artifact).unwrap()).unwrap();
        let expected: Vec<f64> = serde_json::from_slice(&std::fs::read(expected).unwrap()).unwrap();
        let mut worker = kernels.sectors()[0].try_clone().unwrap();
        let actual = std::thread::spawn(move || {
            let mut values = vec![0.0; worker.output_count()];
            worker.evaluate(&[0.25], &mut values).unwrap();
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
