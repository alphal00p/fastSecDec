#![cfg(feature = "threshold-decomposition")]
use fastsecdec::{
    Atom, FeynmanDiagram, Kinematics, Model,
    input::GraphIntegral,
    parametric::ParametricIntegrand,
    threshold::represented::{
        Error, Limits, NumericalMeaning,
        graph::{ExactRepresentedGraphInput, GraphLocation, GraphPoint, Stage},
    },
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{domains::float::Float, parse, symbol};

use fastsecdec::threshold::gcad::{
    GcadRequest, SourceProvenance,
    staging::{self, EvidenceRecord, StagedRequest},
};

fn point(invariant: Atom) -> Arc<GraphPoint> {
    let model = Arc::new(
        Model::from_json(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/scalar.json"
        )))
        .unwrap(),
    );
    let diagram = Arc::new(
        FeynmanDiagram::from_dot(
            model,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/graphs/bubble.dot"
            )),
        )
        .unwrap(),
    );
    let p = feynkit_graph::symbols::external_momentum().call(1);
    let (x, y, eps) = symbol!(
        "preparametric_probe::x",
        "preparametric_probe::y",
        "preparametric_probe::eps"
    );
    let kin = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_mass_squared(&p, invariant)
        .unwrap();
    Arc::new(GraphPoint {
        diagram,
        kinematics: Arc::new(kin),
        scalar_values: BTreeMap::from([(symbol!("UFO::mt"), Atom::num(1.5))]),
        auxiliary_momenta: vec![],
        powers: BTreeMap::new(),
        measure_multiplier: Atom::num(0.5),
        coordinates: vec![x, y],
        regulator: eps,
        dimension: Atom::num(4) - Atom::num(2) * Atom::var(eps),
    })
}
fn prepare(point: Arc<GraphPoint>) -> ExactRepresentedGraphInput {
    ExactRepresentedGraphInput::prepare(
        point,
        NumericalMeaning::RepresentedValues,
        Limits::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
}

fn replay_point() -> Arc<GraphPoint> {
    // Exact represented 12,3/2 yields rational threshold roots 1/4 and 3/4.
    point(Atom::num(12.0))
}
fn request(source: Arc<GraphPoint>) -> GcadRequest {
    GcadRequest::preparametric_projective(
        Arc::new(prepare(source)),
        1,
        Default::default(),
        GcadRequest::default_limits(),
    )
    .unwrap()
}

#[test]
fn exclusive_graph_source_and_cold_original_owner_replay_verify_saved_raw_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let source = replay_point();
    let request = request(source.clone());
    assert!(matches!(
        request.source_provenance(),
        SourceProvenance::PreparametricGraph(_)
    ));
    assert!(request.represented_input().is_none());
    assert_eq!(
        request.input(),
        request
            .preparametric_graph_input()
            .unwrap()
            .exact()
            .as_ref()
    );
    let exact = GcadRequest::projective(
        fastsecdec::threshold::projective::AffineProjectivePreparation::eliminate(
            request.input(),
            1,
        )
        .unwrap(),
        Default::default(),
        Default::default(),
        GcadRequest::default_limits(),
    )
    .unwrap();
    assert!(matches!(exact.source_provenance(), SourceProvenance::Exact));
    assert_ne!(
        request.source_identity().unwrap(),
        exact.source_identity().unwrap()
    );
    assert_eq!(request.problem().split, exact.problem().split);
    let raw = request.solve().unwrap();
    let staged = StagedRequest::write(dir.path(), Arc::new(request)).unwrap();
    let evidence = staged.write_evidence(dir.path(), &raw).unwrap();
    assert!(staged.request().verify(raw.clone()).is_ok());
    assert!(exact.verify(raw).is_err());
    assert!(matches!(
        StagedRequest::read(dir.path(), staged.receipt(), 10_000_000),
        Err(staging::Error::OriginalGraphRequired)
    ));
    let mut stages = Vec::new();
    let restored = StagedRequest::read_with_original_graph(
        dir.path(),
        staged.receipt(),
        10_000_000,
        source,
        Limits::default(),
        |p| {
            stages.push(p.stage);
            ControlFlow::Continue(())
        },
    )
    .unwrap();
    assert!(stages.contains(&Stage::Parameterization));
    assert_eq!(restored.request().identity(), staged.request().identity());
    assert!(
        restored
            .verify_evidence(dir.path(), &evidence, 10_000_000)
            .unwrap()
            .result
            .is_ok()
    );
    std::fs::write(
        dir.path().join("evidence.json"),
        serde_json::to_vec(&evidence).unwrap(),
    )
    .unwrap();
    drop(restored);
    drop(staged);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cold_graph_replay_child",
            "--ignored",
            "--nocapture",
        ])
        .env("FASTSECDEC_GRAPH_REPLAY_TEST", dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("verified-child.json").exists());
}

#[test]
#[ignore = "invoked as a fresh-process replay child"]
fn cold_graph_replay_child() {
    let root = std::path::PathBuf::from(std::env::var_os("FASTSECDEC_GRAPH_REPLAY_TEST").unwrap());
    let receipt: EvidenceRecord =
        serde_json::from_slice(&std::fs::read(root.join("evidence.json")).unwrap()).unwrap();
    let staged = StagedRequest::read_with_original_graph(
        &root,
        &receipt.request,
        10_000_000,
        replay_point(),
        Limits::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    // Deliberately no solve call in this process: consume raw record and current verifier.
    let verified = staged
        .verify_evidence(&root, &receipt, 10_000_000)
        .unwrap()
        .result
        .unwrap();
    std::fs::write(root.join("verified-child.json"), serde_json::to_vec(&serde_json::json!({
        "cells": verified.cells().len(), "source": staged.request().source_identity().unwrap(), "solve_called": false,
    })).unwrap()).unwrap();
}

#[test]
fn replay_refuses_changed_original_precision_roles_receipt_limits_and_cancellation() {
    let dir = tempfile::tempdir().unwrap();
    let staged = StagedRequest::write(dir.path(), Arc::new(request(replay_point()))).unwrap();
    let mut unused = (*replay_point()).clone();
    unused.kinematics = Arc::new(
        unused
            .kinematics
            .as_ref()
            .clone()
            .with_momenta([parse!("preparametric_probe::extra")])
            .unwrap(),
    );
    let mut weight = (*replay_point()).clone();
    weight.diagram = Arc::new(
        weight
            .diagram
            .as_ref()
            .clone()
            .with_overall_factor(Atom::num(2)),
    );
    for changed in [
        point(Atom::num(13.0)),
        point(Atom::num(Float::with_val(192, 12.0))),
        Arc::new(unused),
        Arc::new(weight),
    ] {
        assert!(
            StagedRequest::read_with_original_graph(
                dir.path(),
                staged.receipt(),
                10_000_000,
                changed,
                Limits::default(),
                |_| ControlFlow::Continue(())
            )
            .is_err()
        );
    }
    let mut wrong = staged.receipt().clone();
    wrong.source_identity = "wrong".into();
    assert!(
        StagedRequest::read_with_original_graph(
            dir.path(),
            &wrong,
            10_000_000,
            replay_point(),
            Limits::default(),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
    assert!(
        StagedRequest::read_with_original_graph(
            dir.path(),
            staged.receipt(),
            10_000_000,
            replay_point(),
            Limits {
                precision_bits: 32,
                ..Default::default()
            },
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
    assert!(
        StagedRequest::read_with_original_graph(
            dir.path(),
            staged.receipt(),
            10_000_000,
            replay_point(),
            Limits::default(),
            |_| ControlFlow::Break(())
        )
        .is_err()
    );
    assert!(
        StagedRequest::read_with_original_graph(
            dir.path(),
            staged.receipt(),
            1,
            replay_point(),
            Limits::default(),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
}
fn exact_density(source: &GraphPoint, mass: Atom) -> ParametricIntegrand {
    let kin = source
        .kinematics
        .try_map_scalar_values(|_| Ok::<_, ()>(Atom::num(16)))
        .unwrap();
    let graph = GraphIntegral::new_with_scalar_values(
        source.diagram.clone(),
        &kin,
        &BTreeMap::from([(symbol!("UFO::mt"), mass)]),
    )
    .unwrap()
    .with_measure_multiplier(Atom::num((1, 2)));
    ParametricIntegrand::from_graph(
        &graph,
        source.coordinates.clone(),
        source.regulator,
        source.dimension.clone(),
    )
    .unwrap()
}
#[test]
fn original_float_point_precedes_family_arithmetic_and_matches_exact_density() {
    let source = point(Atom::num(16.0));
    assert!(
        GraphIntegral::new_with_scalar_values(
            source.diagram.clone(),
            &source.kinematics,
            &source.scalar_values
        )
        .is_err()
    );
    let result = prepare(source.clone());
    assert!(Arc::ptr_eq(result.original(), &source));
    assert!(Arc::ptr_eq(&result.original().diagram, &source.diagram));
    assert_eq!(
        result.exact().as_ref(),
        &exact_density(&source, Atom::num((3, 2)))
    );
    assert_eq!(
        result.exact_scalar_values()[&symbol!("UFO::mt")],
        Atom::num((3, 2))
    );
    assert_eq!(result.exact_measure(), &Atom::num((1, 2)));
    assert_eq!(result.conversions().len(), 3);
    let keys = result
        .conversions()
        .iter()
        .find_map(|row| match &row.location {
            GraphLocation::KinematicValue { keys } => Some(keys),
            _ => None,
        })
        .unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(
        result.original().scalar_values[&symbol!("UFO::mt")],
        Atom::num(1.5)
    );
}
#[test]
fn represented_decimal_mass_is_converted_before_native_mass_squared_arithmetic() {
    let mut source = (*point(Atom::num(16.0))).clone();
    source
        .scalar_values
        .insert(symbol!("UFO::mt"), Atom::num(0.1f64));
    let source = Arc::new(source);
    let owner = prepare(source.clone());
    // Exact binary64 0.1, not 1/10 or the already rounded binary64 0.1*0.1.
    let mass = Atom::num((3_602_879_701_896_397i64, 36_028_797_018_963_968i64));
    assert_eq!(owner.exact_scalar_values()[&symbol!("UFO::mt")], mass);
    assert_eq!(owner.exact().as_ref(), &exact_density(&source, mass));
}
#[test]
fn original_precision_and_unused_assumptions_change_provenance_without_rational_guessing() {
    let low = prepare(point(Atom::num(Float::with_val(53, 16.0))));
    let high = prepare(point(Atom::num(Float::with_val(192, 16.0))));
    assert_eq!(low.exact(), high.exact());
    assert_ne!(low.source_identity(), high.source_identity());
    let exact = prepare(point(Atom::num(16)));
    assert_eq!(low.exact(), exact.exact());
    assert_ne!(low.source_identity(), exact.source_identity());
    let mut additional = (*point(Atom::num(16.0))).clone();
    additional.kinematics = Arc::new(
        additional
            .kinematics
            .as_ref()
            .clone()
            .with_momenta([parse!("preparametric_probe::unused")])
            .unwrap(),
    );
    let additional = prepare(Arc::new(additional));
    assert_eq!(additional.exact(), low.exact());
    assert_ne!(additional.source_identity(), low.source_identity());
    let next = prepare(point(Atom::num(f64::from_bits(16.0f64.to_bits() + 1))));
    assert_ne!(next.exact(), low.exact());
    assert_ne!(next.source_identity(), low.source_identity());
}
#[test]
fn unsupported_payload_policy_and_limits_refuse_before_parameterization() {
    let mut payload = (*point(Atom::num(16.0))).clone();
    payload.diagram = Arc::new(
        payload
            .diagram
            .as_ref()
            .clone()
            .with_overall_factor(Atom::num(0.1)),
    );
    let mut stages = Vec::new();
    let error = ExactRepresentedGraphInput::prepare(
        Arc::new(payload),
        NumericalMeaning::RepresentedValues,
        Limits::default(),
        |p| {
            stages.push(p.stage);
            ControlFlow::Continue(())
        },
    )
    .unwrap_err();
    assert!(matches!(error, Error::Unsupported(_)));
    assert!(!stages.contains(&Stage::Parameterization));
    let source = point(Atom::num(16.0));
    assert!(matches!(
        ExactRepresentedGraphInput::prepare(
            source.clone(),
            NumericalMeaning::UncertaintyBounds,
            Limits::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(Error::Unsupported(_))
    ));
    let mut stages = Vec::new();
    assert!(matches!(
        ExactRepresentedGraphInput::prepare(
            source.clone(),
            NumericalMeaning::RepresentedValues,
            Limits {
                precision_bits: 32,
                ..Limits::default()
            },
            |p| {
                stages.push(p.stage);
                ControlFlow::Continue(())
            }
        ),
        Err(Error::ResourceIncomplete(_))
    ));
    assert!(!stages.contains(&Stage::Parameterization));
    assert!(matches!(
        ExactRepresentedGraphInput::prepare(
            source,
            NumericalMeaning::RepresentedValues,
            Limits::default(),
            |_| ControlFlow::Break(())
        ),
        Err(Error::Cancelled)
    ));
}

fn compile_request(request: GcadRequest) -> fastsecdec::kernel::KernelSet {
    use fastsecdec::{
        generation::GenerationOptions,
        kernel::{CompilationSettings, EvaluatorBackend, KernelSet, PrecisionPolicy},
        threshold::regularization::RegularizedFiber,
    };
    let owner = Arc::new(request.solve_verified().unwrap());
    let fiber = RegularizedFiber::admit(
        owner,
        BTreeMap::new(),
        symbol!("preparametric_probe::unit"),
        Default::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let continued = fiber
        .continue_symbolically(
            &GenerationOptions {
                max_order: 0,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
    let dir = tempfile::tempdir().unwrap();
    KernelSet::compile_threshold_fiber(
        &bound,
        dir.path(),
        0,
        PrecisionPolicy::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap()
}
fn vectors(kernels: &mut fastsecdec::kernel::KernelSet) -> Vec<Vec<f64>> {
    [0.125, 0.375, 0.625, 0.875]
        .into_iter()
        .map(|x| {
            let mut sum = kernels.exact_coefficients().to_vec();
            for sector in kernels.sectors_mut() {
                let mut value = vec![0.; sum.len()];
                sector.evaluate(&[x], &mut value).unwrap();
                for (sum, value) in sum.iter_mut().zip(value) {
                    *sum += value;
                }
            }
            sum
        })
        .collect()
}
#[test]
fn compiled_vectors_match_exact_native_point_and_fresh_reload_needs_no_graph_owner() {
    let represented = request(replay_point());
    let exact = GcadRequest::projective(
        fastsecdec::threshold::projective::AffineProjectivePreparation::eliminate(
            represented.input(),
            1,
        )
        .unwrap(),
        Default::default(),
        Default::default(),
        GcadRequest::default_limits(),
    )
    .unwrap();
    let mut represented_kernels = compile_request(represented);
    let mut exact_kernels = compile_request(exact);
    assert_ne!(represented_kernels.content_id(), exact_kernels.content_id());
    assert_eq!(represented_kernels.orders(), exact_kernels.orders());
    let actual = vectors(&mut represented_kernels);
    let expected = vectors(&mut exact_kernels);
    for (a, b) in actual.iter().flatten().zip(expected.iter().flatten()) {
        assert!((a - b).abs() < 2e-12 * (1. + b.abs()), "{a} != {b}");
    }
    assert!(
        actual
            .iter()
            .any(|row| row.iter().skip(1).step_by(2).any(|v| v.abs() > 0.1))
    );
    let bytes = represented_kernels.to_bytes().unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("kernels.bin"), bytes).unwrap();
    std::fs::write(
        dir.path().join("values.json"),
        serde_json::to_vec(&actual).unwrap(),
    )
    .unwrap();
    drop(exact_kernels);
    drop(represented_kernels);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "artifact_reload_child",
            "--ignored",
            "--nocapture",
        ])
        .env("FASTSECDEC_GRAPH_ARTIFACT_TEST", dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
#[ignore = "invoked as a fresh-process artifact reader without original inputs"]
fn artifact_reload_child() {
    let root =
        std::path::PathBuf::from(std::env::var_os("FASTSECDEC_GRAPH_ARTIFACT_TEST").unwrap());
    let bytes = std::fs::read(root.join("kernels.bin")).unwrap();
    let expected: Vec<Vec<f64>> =
        serde_json::from_slice(&std::fs::read(root.join("values.json")).unwrap()).unwrap();
    let mut kernels = fastsecdec::kernel::KernelSet::from_bytes_with_options(
        &bytes,
        fastsecdec::kernel::KernelLoadOptions { validate: true },
    )
    .unwrap();
    assert_eq!(
        kernels.program_recipe(),
        fastsecdec::kernel::ProgramRecipe::ThresholdV1
    );
    assert_eq!(vectors(&mut kernels), expected);
}
