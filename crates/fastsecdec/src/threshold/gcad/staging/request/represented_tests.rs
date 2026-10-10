use super::*;
use crate::{
    generation::GenerationOptions,
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    threshold::{
        gcad::{NativeDecomposition, SolverOptions, staging::StagedRequest},
        options::ThresholdDecompositionOptions,
        represented::{self, NumericalMeaning},
    },
};
use std::{ops::ControlFlow, sync::Arc};
use symbolica::{atom::Symbol, symbol};
fn input(projective: bool) -> Arc<ParametricIntegrand> {
    let (x, y, e) = symbol!(
        "represented_owner::x",
        "represented_owner::y",
        "represented_owner::eps"
    );
    let (vars, domain, factors) = if projective {
        (
            vec![x, y],
            ParametricDomain::ProjectiveSimplex,
            vec![
                PolynomialFactor::new(
                    Atom::num(0.75f64) * Atom::var(y) - Atom::num(0.25f64) * Atom::var(x),
                    -Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::var(x) + Atom::var(y),
                    Atom::num(-2) + Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )
    } else {
        (
            vec![x],
            ParametricDomain::UnitCube,
            vec![
                PolynomialFactor::new(
                    Atom::var(x) - Atom::num(0.1f64),
                    -Atom::one() - Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::one() + Atom::num(0.3f64) * Atom::var(x),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
            ],
        )
    };
    Arc::new(
        ParametricIntegrand::new(
            vars.clone(),
            e,
            domain,
            vec![ParametricTerm::new(
                Atom::num(0.125f64),
                vec![Atom::Zero; vars.len()],
                factors,
            )],
        )
        .unwrap(),
    )
}
fn request(projective: bool) -> GcadRequest {
    ThresholdDecompositionOptions::default()
        .gcad_first_represented_request(
            input(projective),
            &GenerationOptions::default(),
            NumericalMeaning::RepresentedValues,
            represented::Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
}
fn dir(label: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "represented-owner-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&p).unwrap();
    p
}
#[test]
fn represented_source_identity_and_exact_legacy_separate() {
    let request = request(false);
    let original = input(false);
    let conversion = request.represented_input().unwrap();
    assert_eq!(request.input(), original.as_ref());
    assert_eq!(request.prepared_terms(), conversion.exact().terms());
    assert_eq!(
        request.signed_factors()[0].original_polynomial,
        *original.terms()[0].factors()[0].polynomial()
    );
    assert_ne!(
        request.signed_factors()[0].original_polynomial,
        request.signed_factors()[0].prepared_polynomial
    );
    let exact = GcadRequest::unit_cube(
        request.exact_input(),
        GcadKinematics::default(),
        SolverOptions::default(),
        GcadRequest::default_limits(),
    )
    .unwrap();
    assert_eq!(request.problem().split, exact.problem().split);
    assert_ne!(request.identity(), exact.identity());
    assert_ne!(
        request.source_identity().unwrap(),
        exact.source_identity().unwrap()
    );
    assert_eq!(
        exact.source_identity().unwrap(),
        crate::generation::source_identity(exact.input(), &[], &[]).unwrap()
    );
    let benign = ThresholdDecompositionOptions::default()
        .gcad_first_represented_request(
            Arc::new(exact.input().clone()),
            &GenerationOptions::default(),
            NumericalMeaning::RepresentedValues,
            represented::Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert!(benign.represented_input().is_none());
    assert_eq!(benign.identity(), exact.identity());
    assert_eq!(
        benign.source_identity().unwrap(),
        exact.source_identity().unwrap()
    );
    let raw = request.solve().unwrap();
    assert!(
        exact
            .verify(NativeDecomposition::from_parts(
                request.identity().clone(),
                raw.result().clone()
            ))
            .is_err()
    );
    assert_eq!(request.verify(raw).unwrap().cells().len(), 2);
}
#[test]
fn projective_preparation_keeps_original_and_exact_associations() {
    let request = request(true);
    let original = input(true);
    let prep = request.projective_preparation().unwrap();
    assert_eq!(request.input(), original.as_ref());
    assert_eq!(prep.input_owner().as_ref(), request.exact_input());
    assert_eq!(request.prepared_terms(), prep.terms());
    assert_eq!(
        request.signed_factors()[0].original_polynomial,
        *original.terms()[0].factors()[0].polynomial()
    );
    let raw = request.solve().unwrap();
    assert_eq!(request.verify(raw).unwrap().cells().len(), 2);
    let p = dir("projective");
    let staged = StagedRequest::write(&p, Arc::new(request.clone())).unwrap();
    let restored = StagedRequest::read(&p, staged.receipt(), 10_000_000).unwrap();
    assert_eq!(restored.request().identity(), request.identity());
    assert_eq!(
        restored.request().projective_preparation(),
        request.projective_preparation()
    );
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn staged_conversion_recomputation_rejects_alteration_and_limits() {
    let p = dir("tamper");
    let request = Arc::new(request(false));
    let staged = StagedRequest::write(&p, request.clone()).unwrap();
    for change in 0..7 {
        let (mut stored, mut atoms, symbols): (StoredRequest, _, _) =
            codec::read(&p, &staged.receipt().record, KIND).unwrap();
        let r = stored.represented.as_mut().unwrap();
        match change {
            0 => r.literals[0].precision_bits[0] += 1,
            1 => r.literals[0].exact = atoms.push(&Atom::num(19)),
            2 => r.version = 2,
            3 => r.meaning = NumericalMeaning::UncertaintyBounds,
            4 => r.limits.rational_bits += 1,
            5 => {
                r.literals[0].location = represented::Location::FactorExponent {
                    term: 99,
                    factor: 99,
                }
            }
            _ => r.exact_input.domain = 2,
        }
        let record = codec::write(
            &p,
            &format!("tampered-{change}"),
            KIND,
            &stored,
            atoms,
            symbols,
        )
        .unwrap();
        let receipt = RequestRecord {
            record,
            source_identity: staged.receipt().source_identity.clone(),
        };
        assert!(
            StagedRequest::read(&p, &receipt, 10_000_000).is_err(),
            "mutation {change}"
        );
    }
    let cap = represented::Limits {
        rational_bits: 16,
        ..represented::Limits::default()
    };
    assert!(
        StagedRequest::read_with_representation_limits(&p, staged.receipt(), 10_000_000, cap)
            .is_err()
    );
    let mut wrong = staged.receipt().clone();
    wrong.source_identity.push('x');
    assert!(StagedRequest::read(&p, &wrong, 10_000_000).is_err());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn fresh_process_staging_replays_conversion_and_native_verification() {
    for projective in [false, true] {
        let p = dir("fresh");
        let staged = StagedRequest::write(&p, Arc::new(request(projective))).unwrap();
        let evidence = staged
            .write_evidence(&p, &staged.request().solve().unwrap())
            .unwrap();
        std::fs::write(
            p.join("receipt.json"),
            serde_json::to_vec(&evidence).unwrap(),
        )
        .unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "threshold::gcad::staging::request::represented_tests::fresh_read_child",
                "--ignored",
                "--nocapture",
            ])
            .env("FSD_REPRESENTED_CHILD_ROOT", &p)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let result: serde_json::Value =
            serde_json::from_slice(&std::fs::read(p.join("child.json")).unwrap()).unwrap();
        assert_eq!(
            result["source"],
            staged.request().source_identity().unwrap()
        );
        assert_eq!(result["cells"], 2);
        std::fs::remove_dir_all(p).unwrap();
    }
}
#[test]
#[ignore = "called only by fresh_process_staging_replays_conversion_and_native_verification"]
fn fresh_read_child() {
    let p = std::path::PathBuf::from(std::env::var("FSD_REPRESENTED_CHILD_ROOT").unwrap());
    let _: Symbol = symbol!("represented_owner::y");
    let _: Symbol = symbol!("represented_owner::eps");
    let _: Symbol = symbol!("represented_owner::x");
    let _: Symbol = symbol!("different_process_registration_order::before_input");
    let receipt = serde_json::from_slice::<crate::threshold::gcad::staging::EvidenceRecord>(
        &std::fs::read(p.join("receipt.json")).unwrap(),
    )
    .unwrap();
    let staged = StagedRequest::read(&p, &receipt.request, 10_000_000).unwrap();
    assert!(staged.request().represented_input().is_some());
    let result = staged
        .verify_evidence(&p, &receipt, 10_000_000)
        .unwrap()
        .result
        .unwrap();
    std::fs::write(p.join("child.json"),serde_json::to_vec(&serde_json::json!({"source":staged.request().source_identity().unwrap(),"cells":result.cells().len()})).unwrap()).unwrap();
}

#[test]
fn numeric_route_preserves_semantics_and_refuses_nonfinite_or_wrong_mode() {
    let template = input(false);
    let x = template.parameters()[0];
    let eps = template.regulator();
    let phase = (Atom::num(0.125f64) * Atom::var(eps)).exp();
    let phase_input = Arc::new(
        ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                phase,
                template.terms()[0].monomial_powers().to_vec(),
                template.terms()[0].factors().to_vec(),
            )],
        )
        .unwrap(),
    );
    let o = ThresholdDecompositionOptions::default();
    let request = o
        .gcad_first_represented_request(
            phase_input,
            &GenerationOptions::default(),
            NumericalMeaning::RepresentedValues,
            represented::Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert_eq!(
        request.prepared_terms()[0].prefactor(),
        &(Atom::num(Rational::from((1, 8))) * Atom::var(eps)).exp()
    );
    assert_eq!(
        request.prepared_terms()[0].factors()[1].role(),
        FactorRole::Polynomial
    );
    assert_eq!(
        request.signed_factors()[0].semantics,
        FactorSemantics::Causal
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let input = Arc::new(
            ParametricIntegrand::new(
                vec![x],
                eps,
                ParametricDomain::UnitCube,
                vec![ParametricTerm::new(
                    Atom::num(value),
                    vec![Atom::Zero],
                    vec![],
                )],
            )
            .unwrap(),
        );
        assert!(
            o.gcad_first_represented_request(
                input,
                &GenerationOptions::default(),
                NumericalMeaning::RepresentedValues,
                represented::Limits::default(),
                |_| ControlFlow::Continue(())
            )
            .is_err()
        );
    }
    let generation = GenerationOptions {
        mode: crate::generation::GenerationMode::NumericalDual,
        ..GenerationOptions::default()
    };
    assert!(
        o.gcad_first_represented_request(
            template,
            &generation,
            NumericalMeaning::RepresentedValues,
            represented::Limits::default(),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
}

#[test]
fn compiled_artifact_keeps_original_numeric_provenance_with_equal_native_vectors() {
    use crate::kernel::{CompilationSettings, EvaluatorBackend, KernelSet, PrecisionPolicy};
    let represented = request(false);
    let exact = GcadRequest::unit_cube(
        represented.exact_input(),
        GcadKinematics::default(),
        SolverOptions::default(),
        GcadRequest::default_limits(),
    )
    .unwrap();
    let mut owners = Vec::new();
    for request in [represented, exact] {
        let expected_source = request.source_identity().unwrap();
        let owner = Arc::new(request.solve_verified().unwrap());
        let fiber = crate::threshold::regularization::RegularizedFiber::admit(
            owner,
            BTreeMap::new(),
            symbol!("represented_owner::unit"),
            Default::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let continued = fiber
            .continue_symbolically(
                &GenerationOptions {
                    max_order: 0,
                    ..GenerationOptions::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
        let p = dir("compiled");
        let kernels = KernelSet::compile_threshold_fiber(
            &bound,
            &p,
            0,
            PrecisionPolicy::default(),
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..CompilationSettings::default()
            },
        )
        .unwrap();
        assert_eq!(
            kernels
                .threshold_metadata()
                .unwrap()
                .lineage()
                .preparation
                .source_identity
                .0,
            expected_source
        );
        let restored = KernelSet::from_bytes(&kernels.to_bytes().unwrap()).unwrap();
        assert_eq!(restored.content_id(), kernels.content_id());
        owners.push(restored);
        std::fs::remove_dir_all(p).unwrap();
    }
    assert_ne!(owners[0].content_id(), owners[1].content_id());
    assert_eq!(owners[0].orders(), owners[1].orders());
    assert_eq!(
        owners[0].exact_coefficients(),
        owners[1].exact_coefficients()
    );
    let mut vectors = Vec::new();
    for owner in &mut owners {
        let mut rows = Vec::new();
        let outputs = owner.orders().len();
        for point in [0.125, 0.375, 0.75] {
            for sector in owner.sectors_mut() {
                let mut v = vec![0.; outputs];
                sector.evaluate(&[point], &mut v).unwrap();
                rows.push(v);
            }
        }
        vectors.push(rows);
    }
    assert_eq!(vectors[0], vectors[1]);
}
