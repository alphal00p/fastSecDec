//! Metadata codec controls. The artificial contour attached below does not
//! claim to be the mathematical contour of the unchanged numerical fixture.
use super::*;
use crate::contour::{ContourDefinitions, ContourMetadata};
use std::sync::Arc;

fn compact_kernel() -> KernelSet {
    let mut kernels = kernel(Atom::one());
    let chart = &mut kernels.metadata.as_mut().unwrap().charts[0];
    let x = chart.coordinates.target_parameters[0];
    let body = (Atom::var(x) + Atom::one()).pow(5) + (Atom::var(x) + Atom::num(2)).pow(4);
    let (definitions, calls) =
        ContourDefinitions::coefficients(&[x], std::slice::from_ref(&body)).unwrap();
    assert!(!definitions.is_empty());
    let call = calls[0].clone();
    assert_eq!(definitions.materialize(&call).unwrap(), body);
    chart.contour = Some(ContourMetadata {
        causal_polynomial: Atom::num(2) + Atom::var(x),
        positive_polynomials: vec![],
        images: vec![call.clone()],
        ratios: vec![call.clone()],
        jacobian: call.derivative(x),
        validation_faces: vec![vec![], vec![(0, 0)]],
        definitions: Arc::new(definitions),
    });
    kernels
}

fn decode(bytes: &[u8]) -> (Envelope, PayloadV12) {
    let (envelope, used): (Envelope, _) = bincode::decode_from_slice(
        bytes.strip_prefix(MAGIC_V12).unwrap(),
        bincode::config::standard(),
    )
    .unwrap();
    assert_eq!(used, bytes.len() - MAGIC_V12.len());
    let context = State::import(&mut envelope.state.as_slice(), None).unwrap();
    let (payload, used) = bincode::decode_from_slice_with_context(
        &envelope.payload,
        bincode::config::standard(),
        context,
    )
    .unwrap();
    assert_eq!(used, envelope.payload.len());
    (envelope, payload)
}

fn rewrite(bytes: &[u8], edit: impl FnOnce(&mut PayloadV12)) -> Vec<u8> {
    let (mut envelope, mut payload) = decode(bytes);
    edit(&mut payload);
    envelope.payload = bincode::encode_to_vec(payload, bincode::config::standard()).unwrap();
    envelope.digest = *digest(MAGIC_V12, &envelope.state, &envelope.payload).as_bytes();
    let mut bytes = MAGIC_V12.to_vec();
    bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).unwrap());
    bytes
}

#[test]
fn compact_metadata_roundtrips_without_rebuilding_numerical_programs() {
    let kernels = compact_kernel();
    let expected = serde_json::to_value(PortableMetadata::from_native(
        kernels.metadata.as_ref().unwrap(),
    ))
    .unwrap();
    let (identity, bytes) = compiled(&kernels).unwrap();
    assert!(bytes.starts_with(MAGIC_V12));
    for validate in [false, true] {
        let restored =
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate }).unwrap();
        assert_eq!(restored.content_id(), identity);
        assert_eq!(
            restored.sectors[0].program_bytes,
            kernels.sectors[0].program_bytes
        );
        assert_eq!(
            serde_json::to_value(PortableMetadata::from_native(
                restored.metadata.as_ref().unwrap()
            ))
            .unwrap(),
            expected
        );
        let contour = restored.metadata.as_ref().unwrap().charts[0]
            .contour
            .as_ref()
            .unwrap();
        assert_eq!(contour.version(), 2);
        let decoded = contour
            .function_definitions()
            .materialize(&contour.images()[0])
            .unwrap();
        let x = restored.metadata.as_ref().unwrap().charts[0]
            .coordinates
            .target_parameters[0];
        assert_eq!(decoded.replace(Atom::var(x)).with(0), Atom::num(17));
        let derivative = contour
            .function_definitions()
            .materialize(contour.jacobian())
            .unwrap();
        assert_eq!(derivative.replace(Atom::var(x)).with(0), Atom::num(37));
        let (new_identity, _) = compiled(&restored).unwrap();
        assert_eq!(new_identity, identity);
    }
    // The existing readable metadata owner retains definitions too; this is
    // not a second binary Atom representation or an evaluator rebuild.
    let portable: PortableMetadata = serde_json::from_value(expected.clone()).unwrap();
    let restored = portable
        .into_native(&[kernels.sectors[0].parameters.clone()], true)
        .unwrap();
    assert_eq!(
        serde_json::to_value(PortableMetadata::from_native(&restored)).unwrap(),
        expected
    );

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("compact.bin");
    std::fs::write(&path, bytes).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "kernel::artifact::binary::tests::v12::compact_child",
            "--ignored",
            "--test-threads=1",
        ])
        .env("FASTSECDEC_COMPACT_METADATA_CHILD", &path)
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
        b"compact metadata restored"
    );
}

#[test]
#[ignore = "spawned by the actual compact metadata restoration test"]
fn compact_child() {
    let path = std::path::PathBuf::from(
        std::env::var_os("FASTSECDEC_COMPACT_METADATA_CHILD").expect("issued fixture"),
    );
    for index in 0..17 {
        let _ = symbol!(format!("compact_child_scramble::v{index}"));
    }
    let kernels = KernelSet::from_bytes_with_options(
        &std::fs::read(&path).unwrap(),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let chart = &kernels.metadata.as_ref().unwrap().charts[0];
    let contour = chart.contour.as_ref().unwrap();
    assert!(!contour.function_definitions().is_empty());
    let value = contour
        .function_definitions()
        .materialize(contour.jacobian())
        .unwrap();
    assert_eq!(
        value
            .replace(Atom::var(chart.coordinates.target_parameters[0]))
            .with(0),
        Atom::num(37)
    );
    std::fs::write(
        path.with_extension("verified"),
        b"compact metadata restored",
    )
    .unwrap();
}

#[test]
fn compact_sidecar_rejects_missing_duplicate_foreign_and_mismatched_definitions() {
    let (_, bytes) = compiled(&compact_kernel()).unwrap();
    for edit in [
        |payload: &mut PayloadV12| {
            payload.contour_definitions.clear();
        },
        |payload: &mut PayloadV12| {
            payload
                .contour_definitions
                .push(payload.contour_definitions[0].clone());
        },
        |payload: &mut PayloadV12| {
            payload.contour_definitions[0].0 = usize::MAX;
        },
        |payload: &mut PayloadV12| {
            payload.contour_definitions[0].1 = ContourDefinitions::default();
        },
        |payload: &mut PayloadV12| {
            payload.base.metadata = None;
        },
    ] {
        let corrupt = rewrite(&bytes, edit);
        for validate in [false, true] {
            assert!(
                KernelSet::from_bytes_with_options(&corrupt, KernelLoadOptions { validate })
                    .is_err()
            );
        }
    }
    // Corrupt a body without changing its declared function key. Structural
    // admission remains separate from requested semantic identity validation.
    let corrupt = rewrite(&bytes, |payload| {
        let definitions = &mut payload.contour_definitions[0].1;
        let entry = &definitions.entries()[0];
        *definitions = ContourDefinitions::from_parts(vec![(
            entry.function(),
            entry.parameters().to_vec(),
            entry.body() + Atom::one(),
        )])
        .unwrap();
    });
    assert!(KernelSet::from_bytes(&corrupt).is_ok());
    assert!(
        KernelSet::from_bytes_with_options(&corrupt, KernelLoadOptions { validate: true }).is_err()
    );
}

#[test]
fn generated_compact_dynamic_vectors_restore_with_checked_native_requests() {
    use crate::{
        contour::{
            ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
            DynamicConstruction,
        },
        generation::{GenerationMode, SubtractionStrategy},
        kernel::{CompilationSettings, EvaluatorBackend, ProgramRecipe},
        parametric::{FactorRole, FactorSemantics, PolynomialFactor},
    };
    use std::collections::BTreeMap;
    let x = symbol!("compact_dynamic_wire::x");
    let eps = symbol!("compact_dynamic_wire::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
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
    let settings = CompilationSettings {
        backend: EvaluatorBackend::Eager,
        horner_iterations: 0,
        ..Default::default()
    };
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for construction in [
            DynamicConstruction::Polynomial,
            DynamicConstruction::SignAware,
        ] {
            let recipe = match construction {
                DynamicConstruction::Polynomial => ProgramRecipe::DynamicPolynomialV1,
                DynamicConstruction::SignAware => ProgramRecipe::DynamicSignAwareV1,
            };
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode,
                    subtraction: SubtractionStrategy::IntegrateByParts,
                    program_recipe: recipe,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            let mut original = generated.compile_with_settings(settings).unwrap();
            let bytes = generated
                .to_kernel_bytes_with_settings(PrecisionPolicy::default(), settings)
                .unwrap();
            assert!(
                bytes.starts_with(MAGIC_V12),
                "fixture must exercise retained definitions"
            );
            assert!(
                original
                    .metadata
                    .as_ref()
                    .unwrap()
                    .charts
                    .iter()
                    .any(|chart| chart
                        .contour
                        .as_ref()
                        .is_some_and(|c| !c.function_definitions().is_empty()))
            );
            drop(generated);
            let mut archive = crate::kernel::indexed::ProgramArchiveWriter::new(
                std::io::Cursor::new(Vec::new()),
                "a".repeat(64),
                [recipe],
            )
            .unwrap();
            archive.append_kernels(recipe, &original).unwrap();
            let archive = archive.finish().unwrap().0.into_inner();
            let contour = ContourSettings {
                deformation: ContourMode::Dynamical {
                    safety_fraction: 0.8,
                    lambda_cap: 0.2,
                    displacement_cap: 1.,
                    construction,
                },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 1,
                },
            };
            fn samples(kernels: &mut KernelSet, contour: &ContourSettings) -> Vec<Vec<f64>> {
                kernels
                    .bind_parameters_with_contour(&BTreeMap::new(), contour)
                    .unwrap();
                for chart in kernels.contour_validation_charts() {
                    kernels
                        .validate_contour_point(
                            chart.chart_index,
                            &vec![0.37; chart.dimension],
                            true,
                        )
                        .unwrap();
                }
                assert!(kernels.finish_contour_pilot().unwrap().pilot_complete);
                let mut result = vec![kernels.exact_coefficients().to_vec()];
                for sector in &mut kernels.sectors {
                    let mut output = vec![0.; kernels.orders.len()];
                    sector
                        .evaluate(&vec![0.37; sector.dimension()], &mut output)
                        .unwrap();
                    result.push(output);
                }
                assert!(
                    kernels
                        .contour_validation_report()
                        .unwrap()
                        .checked_arguments
                        > 0
                );
                result
            }
            let expected = samples(&mut original, &contour);
            drop(original);
            for validate in [false, true] {
                let mut restored =
                    KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate })
                        .unwrap();
                let actual = samples(&mut restored, &contour);
                assert_eq!(actual.len(), expected.len());
                for (actual, expected) in actual.iter().flatten().zip(expected.iter().flatten()) {
                    assert!(
                        (actual - expected).abs() <= 1e-11 * expected.abs().max(1.),
                        "{actual} != {expected}"
                    );
                }
                let mut reader = crate::kernel::indexed::ProgramArchiveReader::from_reader(
                    std::io::Cursor::new(&archive),
                    KernelLoadOptions { validate },
                )
                .unwrap();
                let mut selected = reader.select(recipe).unwrap().load_sector(0).unwrap();
                assert_eq!(selected.sectors.len(), 1);
                assert!(
                    selected
                        .metadata
                        .as_ref()
                        .unwrap()
                        .charts
                        .iter()
                        .all(|chart| chart.kernel_sector == Some(0))
                );
                let actual = samples(&mut selected, &contour);
                for (actual, expected) in actual[1].iter().zip(&expected[1]) {
                    assert!(
                        (actual - expected).abs() <= 1e-11 * expected.abs().max(1.),
                        "selected {actual} != {expected}"
                    );
                }
            }
        }
    }
}
