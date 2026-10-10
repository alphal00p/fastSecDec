use super::*;
use crate::{kernel::indexed::ProgramRecipe, parametric::FactorSemantics};

fn dynamic_source() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("dynamic_stream::x")],
        symbol!("dynamic_stream::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("1-5*dynamic_stream::x*(1-dynamic_stream::x)"),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn physical_source_identity_is_recipe_independent_but_receipts_are_not() {
    let source = dynamic_source();
    let directory = tempfile::tempdir().unwrap();
    let fixed = prepare(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::FixedV1,
            ..Default::default()
        },
        directory.path(),
        keep,
    )
    .unwrap();
    let dynamic = prepare(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        directory.path(),
        keep,
    )
    .unwrap();
    assert_eq!(fixed.source_identity, dynamic.source_identity);
    assert_eq!(
        fixed.source_identity,
        crate::generation::source_identity(&source, &[], &[]).unwrap()
    );
    assert_ne!(fixed.source.blake3, dynamic.source.blake3);
    assert_eq!(fixed.charts[0].map, dynamic.charts[0].map);
    let chart = discover(directory.path(), &dynamic, &dynamic.charts[0], keep).unwrap();
    assert!(compare_symmetry(directory.path(), &fixed, &chart, &[], keep).is_err());
    let assignment = compare_symmetry(directory.path(), &dynamic, &chart, &[], keep).unwrap();
    let mut foreign_assignment = assignment.clone();
    foreign_assignment.program_recipe = ProgramRecipe::FixedV1;
    assert!(
        finish_preparation(
            &dynamic,
            vec![chart.clone()],
            vec![foreign_assignment],
            vec![]
        )
        .is_err()
    );
    let mut foreign_chart = chart.clone();
    foreign_chart.program_recipe = ProgramRecipe::FixedV1;
    assert!(
        finish_preparation(
            &dynamic,
            vec![foreign_chart],
            vec![assignment.clone()],
            vec![]
        )
        .is_err()
    );
    let prepared = finish_preparation(&dynamic, vec![chart], vec![assignment], vec![]).unwrap();
    let mut foreign_job = prepared.sectors[0].clone();
    foreign_job.program_recipe = ProgramRecipe::FixedV1;
    assert!(generate_sector(directory.path(), &foreign_job, keep).is_err());
    let mut forged = dynamic.clone();
    forged.program_recipe = ProgramRecipe::FixedV1;
    assert!(discover(directory.path(), &forged, &forged.charts[0], keep).is_err());
}

#[test]
fn dynamic_helpers_restore_before_chart_atoms_in_a_fresh_worker() {
    let directory = tempfile::tempdir().unwrap();
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        let source = if recipe == ProgramRecipe::DynamicSignAwareV1 {
            ParametricIntegrand::new(
                vec![symbol!("dynamic_stream::x")],
                symbol!("dynamic_stream::eps"),
                ParametricDomain::UnitCube,
                vec![ParametricTerm::new(
                    Atom::one(),
                    vec![Atom::Zero],
                    vec![
                        PolynomialFactor::new(
                            parse!("(1/4-dynamic_stream::x)*(1+dynamic_stream::x^2)"),
                            Atom::num(-1),
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Causal),
                        PolynomialFactor::new(
                            parse!("1+dynamic_stream::x^2"),
                            Atom::Zero,
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Positive),
                    ],
                )],
            )
            .unwrap()
        } else {
            dynamic_source()
        };
        for (mode, contour_jacobian) in [
            (
                GenerationMode::Symbolic,
                crate::contour::ContourJacobian::Symbolic,
            ),
            (
                GenerationMode::NumericalDual,
                crate::contour::ContourJacobian::Symbolic,
            ),
            (
                GenerationMode::NumericalDual,
                crate::contour::ContourJacobian::Dual,
            ),
        ] {
            let generated = prepared(
                directory.path(),
                &source,
                &GenerationOptions {
                    program_recipe: recipe,
                    mode,
                    contour_jacobian,
                    ..Default::default()
                },
                &[],
                &[],
            );
            std::fs::write(
                directory.path().join("job.json"),
                serde_json::to_vec(&generated.sectors[0]).unwrap(),
            )
            .unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "generation::streaming::tests::program::dynamic_staged_worker",
                    "--nocapture",
                ])
                .env("FASTSECDEC_DYNAMIC_STAGED_WORKER", directory.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(directory.path().join("restored.ok").is_file());
            std::fs::remove_file(directory.path().join("restored.ok")).unwrap();
        }
    }
}

#[test]
fn dynamic_staged_worker() {
    let Some(directory) = std::env::var_os("FASTSECDEC_DYNAMIC_STAGED_WORKER") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let job: SectorJob =
        serde_json::from_slice(&std::fs::read(directory.join("job.json")).unwrap()).unwrap();
    let unit = generate_sector(&directory, &job, keep).unwrap();
    assert_eq!(
        unit.generated.program_descriptor().unwrap().recipe(),
        job.program_recipe
    );
    assert_eq!(unit.generated.dynamic_check_sources().len(), 1);
    use symbolica::atom::AtomCore;
    let sector = &unit.generated.sectors()[0];
    let mut inputs = sector
        .parameters()
        .iter()
        .map(|p| Atom::var(*p))
        .collect::<Vec<_>>();
    inputs.extend([
        Atom::var(crate::contour::dynamic::safety_fraction_symbol()),
        Atom::var(crate::contour::dynamic::lambda_cap_symbol()),
        Atom::var(crate::contour::dynamic::displacement_cap_symbol()),
    ]);
    let mut evaluator = {
        let _preparing = unit.generated.program_descriptor().unwrap().enter();
        sector.coefficients()[0]
            .evaluator(&inputs)
            .build()
            .unwrap()
            .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
    };
    let kernels = unit
        .generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(kernels.program_recipe(), job.program_recipe);
    let bytes = kernels.to_bytes().unwrap();
    assert!(bytes.starts_with(b"FastSecDec\0binserde\x0c"));
    drop(kernels);
    drop(unit);
    // The staged evaluator's native callback must remain functional after
    // every staged/generation owner is gone.
    let value = evaluator.evaluate_single(&[
        Complex::new(0.31, 0.),
        Complex::new(0.8, 0.),
        Complex::new(0.2, 0.),
        Complex::new(1., 0.),
    ]);
    assert!(value.re.is_finite() && value.im.is_finite());
    assert!(value.im.abs() > 1e-8);
    drop(evaluator);
    // Restore the actual native record only after every previous callback
    // owner has gone. This child process has no parent-generation registry.
    let mut restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.program_recipe(), job.program_recipe);
    assert_eq!(restored.to_bytes().unwrap(), bytes);
    restored
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &crate::contour::ContourSettings {
                deformation: crate::contour::ContourMode::Dynamical {
                    safety_fraction: 0.8,
                    lambda_cap: 0.2,
                    displacement_cap: 1.,
                    construction: match job.program_recipe {
                        ProgramRecipe::DynamicPolynomialV1 => {
                            crate::contour::DynamicConstruction::Polynomial
                        }
                        ProgramRecipe::DynamicSignAwareV1 => {
                            crate::contour::DynamicConstruction::SignAware
                        }
                        _ => unreachable!(),
                    },
                },
                validation: crate::contour::ContourValidationOptions {
                    policy: crate::contour::ContourValidation::Off,
                    ..Default::default()
                },
            },
        )
        .unwrap();
    assert_eq!(restored.orders(), &[0, 0]);
    assert_eq!(
        restored.components(),
        &[CoefficientComponent::Real, CoefficientComponent::Imag]
    );
    let mut output = [0.; 2];
    restored.sectors_mut()[0]
        .evaluate(&[0.31], &mut output)
        .unwrap();
    for (actual, expected) in output.into_iter().zip([value.re, value.im]) {
        assert!((actual - expected).abs() < 1e-10 * expected.abs().max(1.));
    }
    std::fs::write(directory.join("restored.ok"), b"retained").unwrap();
}

#[test]
fn legacy_generation_record_version_is_rejected_before_native_import() {
    let directory = tempfile::tempdir().unwrap();
    let preparation = prepare(
        &dynamic_source(),
        &GenerationOptions::default(),
        directory.path(),
        keep,
    )
    .unwrap();
    let mut old = preparation.source.clone();
    let path = old.resolve(directory.path()).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let version = b"FastSecDec\0generation-record".len();
    assert_eq!(bytes[version], 4);
    for old_version in [1, 2, 3] {
        bytes[version] = old_version;
        std::fs::write(&path, &bytes).unwrap();
        old.blake3 = blake3::hash(&bytes).to_hex().to_string();
        let error = records::read_source(directory.path(), &old).err().unwrap();
        assert!(error.to_string().contains("unsupported record version"));
    }
}
