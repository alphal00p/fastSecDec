use super::*;
use crate::{
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    threshold::{
        gcad::{GcadKinematics, Limits, PreparedDomain, SolverOptions},
        projective::AffineProjectivePreparation,
    },
};
use std::{collections::BTreeMap, fs, process::Command};
use symbolica::{
    atom::{Atom, AtomCore},
    parse,
    prelude::Rational,
    symbol,
};

const MAXIMUM: u64 = 16 * 1024 * 1024;
fn limits() -> Limits {
    Limits {
        workers: 1,
        wall_time_secs: 10.0,
        memory_mib: 1024,
        ..GcadRequest::default_limits()
    }
}
fn cube(numerator: i64) -> GcadRequest {
    let input = ParametricIntegrand::new(
        vec![symbol!("staging_gcad::x")],
        symbol!("staging_gcad::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("(1+I)*staging_gcad::b") * Atom::num(numerator),
            vec![Atom::num(0)],
            vec![
                PolynomialFactor::new(
                    parse!("staging_gcad::x-staging_gcad::a"),
                    parse!("-1+staging_gcad::eps+staging_gcad::eta"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    parse!("1+I*staging_gcad::x"),
                    Atom::num(1),
                    FactorRole::Polynomial,
                ),
            ],
        )],
    )
    .unwrap();
    GcadRequest::unit_cube(
        &input,
        GcadKinematics {
            exact_values: BTreeMap::from([(symbol!("staging_gcad::b"), Rational::from((7, 3)))]),
            runtime_parameters: vec![symbol!("staging_gcad::a")],
            strict_positive: vec![parse!("staging_gcad::a"), parse!("1-staging_gcad::a")],
        },
        SolverOptions::default(),
        limits(),
    )
    .unwrap()
}
fn complete(root: &Path) -> (StagedRequest, EvidenceRecord) {
    let request = Arc::new(cube(1));
    let evidence = request.solve().unwrap();
    let staged = StagedRequest::write(root, request).unwrap();
    let receipt = staged.write_evidence(root, &evidence).unwrap();
    (staged, receipt)
}

#[test]
fn complete_native_request_and_raw_proof_roundtrip_without_owner_copy() {
    let directory = tempfile::tempdir().unwrap();
    let request = Arc::new(cube(1));
    let staged = StagedRequest::write(directory.path(), request.clone()).unwrap();
    assert!(Arc::ptr_eq(staged.request_owner(), &request));
    let again = StagedRequest::write(directory.path(), request.clone()).unwrap();
    assert_eq!(staged.receipt(), again.receipt());
    let evidence = request.solve().unwrap();
    let original = serde_json::to_value(evidence.result()).unwrap();
    let receipt = staged.write_evidence(directory.path(), &evidence).unwrap();
    let restored = StagedRequest::read(directory.path(), staged.receipt(), MAXIMUM).unwrap();
    assert_eq!(restored.request().identity(), request.identity());
    let raw = restored
        .read_evidence(directory.path(), &receipt, MAXIMUM)
        .unwrap();
    assert_eq!(serde_json::to_value(raw.result()).unwrap(), original);
    assert_eq!(
        restored.request().input().terms()[0].prefactor(),
        &parse!("(1+I)*staging_gcad::b")
    );
    assert_eq!(
        restored.request().signed_factors()[0].exponent,
        parse!("-1+staging_gcad::eps+staging_gcad::eta")
    );
    let attempt = restored
        .verify_evidence(directory.path(), &receipt, MAXIMUM)
        .unwrap();
    let verified = attempt.result.unwrap();
    assert_eq!(
        verified
            .cells()
            .map(|c| c.native().signs.clone())
            .collect::<Vec<_>>(),
        vec![vec![-1], vec![1]]
    );
    assert!(matches!(
        attempt.observation.read(directory.path(), MAXIMUM).unwrap(),
        VerificationObservation::Verified { cells: 2, .. }
    ));
    assert!(
        fs::read_dir(directory.path()).unwrap().all(|p| !p
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}

#[test]
fn projective_and_explicit_domains_reconstruct_their_exact_native_preparation() {
    let directory = tempfile::tempdir().unwrap();
    let input = ParametricIntegrand::new(
        vec![symbol!("staging_proj::x"), symbol!("staging_proj::y")],
        symbol!("staging_proj::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![parse!("staging_proj::eps"), Atom::num(0)],
            vec![
                PolynomialFactor::new(
                    parse!("staging_proj::x^2+staging_proj::y^2"),
                    Atom::num(1),
                    FactorRole::Polynomial,
                ),
                PolynomialFactor::new(
                    parse!("staging_proj::x-staging_proj::y"),
                    parse!("-4-staging_proj::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let preparation = AffineProjectivePreparation::eliminate(&input, 0).unwrap();
    let request = GcadRequest::projective(
        preparation,
        GcadKinematics::default(),
        SolverOptions::default(),
        limits(),
    )
    .unwrap();
    let staged = StagedRequest::write(directory.path(), Arc::new(request)).unwrap();
    let restored = StagedRequest::read(directory.path(), staged.receipt(), MAXIMUM).unwrap();
    assert_eq!(restored.request().identity(), staged.request().identity());
    let p = restored.request().projective_preparation().unwrap();
    assert_eq!(p.eliminated_index(), 0);
    assert_eq!(
        p.images().iter().cloned().sum::<Atom>().expand(),
        Atom::num(1)
    );
    assert_eq!(p.measure_jacobian(), &Atom::num(1));
    let raw = restored.request().solve().unwrap();
    let record = restored.write_evidence(directory.path(), &raw).unwrap();
    restored
        .verify_evidence(directory.path(), &record, MAXIMUM)
        .unwrap()
        .result
        .unwrap();

    let input = ParametricIntegrand::new(
        vec![symbol!("staging_explicit::x")],
        symbol!("staging_explicit::eps"),
        ParametricDomain::PositiveOrthant,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0)],
            vec![
                PolynomialFactor::new(
                    parse!("2-staging_explicit::x"),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let domain = PreparedDomain::explicit(
        input.parameters().to_vec(),
        vec![
            parse!("staging_explicit::x"),
            parse!("3-staging_explicit::x"),
        ],
        "bounded original-positive-orthant chart, coverage external".into(),
    )
    .unwrap();
    let request = GcadRequest::prepared(
        &input,
        domain,
        GcadKinematics::default(),
        SolverOptions::default(),
        limits(),
    )
    .unwrap();
    let staged = StagedRequest::write(directory.path(), Arc::new(request)).unwrap();
    let restored = StagedRequest::read(directory.path(), staged.receipt(), MAXIMUM).unwrap();
    assert_eq!(restored.request().identity(), staged.request().identity());
    assert_eq!(
        restored.request().input().domain(),
        ParametricDomain::PositiveOrthant
    );
}

#[test]
fn incomplete_and_rejected_evidence_remain_distinct_and_durable() {
    let directory = tempfile::tempdir().unwrap();
    let (staged, complete) = complete(directory.path());
    let accepted = staged
        .verify_evidence(directory.path(), &complete, MAXIMUM)
        .unwrap();
    accepted.result.unwrap();
    let mut raw = SolveResult::failure(
        staged.request().problem().clone(),
        Status::Incomplete,
        "caller worker checkpoint: unvisited stack retained",
    );
    raw.trace = Some(symgcad::output::LiftTrace::Unfinished {
        reason: "resource budget".into(),
    });
    let expected = serde_json::to_value(&raw).unwrap();
    let raw = NativeDecomposition::from_parts(staged.request().identity().clone(), raw);
    let receipt = staged.write_evidence(directory.path(), &raw).unwrap();
    assert_eq!(
        serde_json::to_value(
            staged
                .read_evidence(directory.path(), &receipt, MAXIMUM)
                .unwrap()
                .result()
        )
        .unwrap(),
        expected
    );
    let attempt = staged
        .verify_evidence(directory.path(), &receipt, MAXIMUM)
        .unwrap();
    assert!(matches!(
        attempt.result,
        Err(GcadError::Incomplete(Status::Incomplete))
    ));
    assert!(matches!(
        attempt.observation.read(directory.path(), MAXIMUM).unwrap(),
        VerificationObservation::Incomplete {
            status: Status::Incomplete
        }
    ));
    let (_, mut corrupted) = staged
        .read_evidence(directory.path(), &complete, MAXIMUM)
        .unwrap()
        .into_parts();
    corrupted.cells[0].signs[0] *= -1;
    let corrupted = NativeDecomposition::from_parts(staged.request().identity().clone(), corrupted);
    let receipt = staged.write_evidence(directory.path(), &corrupted).unwrap();
    let attempt = staged
        .verify_evidence(directory.path(), &receipt, MAXIMUM)
        .unwrap();
    assert!(matches!(attempt.result, Err(GcadError::Verification(_))));
    assert!(matches!(
        attempt.observation.read(directory.path(), MAXIMUM).unwrap(),
        VerificationObservation::Rejected { .. }
    ));
    assert!(matches!(
        accepted
            .observation
            .read(directory.path(), MAXIMUM)
            .unwrap(),
        VerificationObservation::Verified { .. }
    ));
    complete.record.verify(directory.path()).unwrap();
}

#[test]
fn changed_density_and_options_cannot_reassociate_the_same_geometry_proof() {
    let directory = tempfile::tempdir().unwrap();
    let (staged, evidence) = complete(directory.path());
    let foreign = Arc::new(cube(2));
    assert!(staged.request().matches_native_problem(foreign.problem()));
    let foreign = StagedRequest::write(directory.path(), foreign).unwrap();
    assert!(matches!(
        foreign.read_evidence(directory.path(), &evidence, MAXIMUM),
        Err(Error::Association)
    ));
    let raw = staged
        .read_evidence(directory.path(), &evidence, MAXIMUM)
        .unwrap();
    assert!(matches!(
        foreign.write_evidence(directory.path(), &raw),
        Err(Error::Association)
    ));
    let mut false_receipt = evidence.clone();
    false_receipt.request = foreign.receipt().clone();
    assert!(matches!(
        foreign.read_evidence(directory.path(), &false_receipt, MAXIMUM),
        Err(Error::Association)
    ));
    let request = staged.request();
    let mut solver = request.problem().solver.clone();
    solver.seed += 1;
    let request = GcadRequest::unit_cube(
        request.input(),
        request.kinematics().clone(),
        solver,
        request.problem().limits.clone(),
    )
    .unwrap();
    let changed = StagedRequest::write(directory.path(), Arc::new(request)).unwrap();
    assert_ne!(changed.receipt().record, staged.receipt().record);
    assert!(matches!(
        changed.write_evidence(directory.path(), &raw),
        Err(Error::Association)
    ));
}

#[test]
fn corrupted_oversized_wrong_kind_and_foreign_receipts_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let (staged, receipt) = complete(directory.path());
    assert!(matches!(
        StagedRequest::read(directory.path(), staged.receipt(), 0),
        Err(Error::RecordLimit)
    ));
    assert!(matches!(
        staged.read_evidence(directory.path(), &receipt, staged.receipt().record.bytes),
        Err(Error::RecordLimit)
    ));
    let mut wrong = staged.receipt().clone();
    wrong.source_identity.push('x');
    assert!(matches!(
        StagedRequest::read(directory.path(), &wrong, MAXIMUM),
        Err(Error::Association)
    ));
    let (mut metadata, atoms, symbols): (serde_json::Value, _, _) = codec::read(
        directory.path(),
        &staged.receipt().record,
        "threshold-gcad-request-v1",
    )
    .unwrap();
    metadata["witness"] = serde_json::json!("wrong-preparation-witness");
    wrong = staged.receipt().clone();
    wrong.record = codec::write(
        directory.path(),
        "gcad-request",
        "threshold-gcad-request-v1",
        &metadata,
        atoms,
        symbols,
    )
    .unwrap();
    assert!(matches!(
        StagedRequest::read(directory.path(), &wrong, MAXIMUM),
        Err(Error::Association)
    ));
    wrong = staged.receipt().clone();
    wrong.record.path = "../escape".into();
    assert!(StagedRequest::read(directory.path(), &wrong, MAXIMUM).is_err());
    wrong = staged.receipt().clone();
    wrong.record = receipt.record.clone();
    assert!(StagedRequest::read(directory.path(), &wrong, MAXIMUM).is_err());
    let path = receipt.record.resolve(directory.path()).unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes.pop();
    fs::write(&path, &bytes).unwrap();
    assert!(
        staged
            .read_evidence(directory.path(), &receipt, MAXIMUM)
            .is_err()
    );
    let mut wrong = receipt.clone();
    wrong.record.bytes = bytes.len() as u64;
    wrong.record.blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert!(
        staged
            .read_evidence(directory.path(), &wrong, MAXIMUM)
            .is_err()
    );
}

#[test]
fn fresh_process_restores_request_then_verifies_without_solving() {
    const CHILD: &str = "FASTSECDEC_GCAD_STAGING_FRESH_CHILD";
    if let Some(directory) = std::env::var_os(CHILD) {
        let directory = std::path::PathBuf::from(directory);
        // Deliberately change registration order before native State import.
        let _ = symbol!("staging_gcad::b");
        for i in 0..32 {
            let _ = symbolica::atom::Symbol::parse(format!("fresh_noise_{i}"), "staging_noise")
                .unwrap();
        }
        let receipt: EvidenceRecord =
            serde_json::from_slice(&fs::read(directory.join("receipt.json")).unwrap()).unwrap();
        let staged = StagedRequest::read(&directory, &receipt.request, MAXIMUM).unwrap();
        // No fixture constructor and no solve call in this child branch.
        let verified = staged
            .verify_evidence(&directory, &receipt, MAXIMUM)
            .unwrap()
            .result
            .unwrap();
        let values = serde_json::json!({"source_identity": staged.receipt().source_identity, "signs": verified.cells().map(|c| c.native().signs.clone()).collect::<Vec<_>>(), "exponent": verified.request().signed_factors()[0].exponent.to_canonical_string()});
        fs::write(
            directory.join("restored.json"),
            serde_json::to_vec(&values).unwrap(),
        )
        .unwrap();
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let (staged, receipt) = complete(directory.path());
    let expected_exponent = staged.request().signed_factors()[0]
        .exponent
        .to_canonical_string();
    let expected_source = staged.receipt().source_identity.clone();
    fs::write(
        directory.path().join("receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    drop(staged);
    let output = Command::new(std::env::current_exe().unwrap()).args(["--exact", "threshold::gcad::staging::tests::fresh_process_restores_request_then_verifies_without_solving", "--test-threads=1"]).env(CHILD, directory.path()).current_dir(directory.path()).output().unwrap();
    assert!(
        output.status.success(),
        "fresh worker failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let restored: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.path().join("restored.json")).unwrap()).unwrap();
    assert_eq!(restored["source_identity"], expected_source);
    assert_eq!(restored["exponent"], expected_exponent);
    assert_eq!(restored["signs"], serde_json::json!([[-1], [1]]));
}
