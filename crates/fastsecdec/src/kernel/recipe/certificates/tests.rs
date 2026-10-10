use super::*;
use crate::{
    contour::functions::dynamic::{requested, requests::Lookup},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, program},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{domains::float::Complex, symbol};
mod fresh_process;

fn fixture(
    settings: CompilationSettings,
) -> (
    NativeProgramDescriptor,
    DynamicCheckSource,
    crate::generation::GenerationMetadata,
) {
    fixture_with_causal_sign(settings, 1)
}

fn fixture_with_causal_sign(
    settings: CompilationSettings,
    sign: i64,
) -> (
    NativeProgramDescriptor,
    DynamicCheckSource,
    crate::generation::GenerationMetadata,
) {
    let x = symbol!("certificate_transport::x");
    let eps = symbol!("certificate_transport::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    sign * (Atom::num(2) + Atom::var(x) + Atom::var(x).pow(3)),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::one() + Atom::var(x).pow(2),
                    Atom::Zero,
                    FactorRole::Polynomial,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let owner = generated.program_descriptor().unwrap().as_ref().clone();
    let source = generated.dynamic_check_sources()[0].as_ref().clone();
    let metadata = generated.metadata().clone();
    let runtime = crate::kernel::compilation::runtime_inputs(&generated, &[]);
    let certificate = DynamicCheckProgram::build(
        &source,
        &owner.charts[0],
        metadata.charts()[0].contour().unwrap(),
        &runtime,
        settings,
    )
    .unwrap();
    (
        owner.with_certificates(vec![certificate]).unwrap(),
        source,
        metadata,
    )
}

#[test]
fn certificate_runtime_order_is_bound_to_the_receiving_owner() {
    let (owner, source, metadata) = fixture(CompilationSettings::default());
    let first = symbol!("certificate_runtime_order::physical_a");
    let second = symbol!("certificate_runtime_order::physical_b");
    let runtime = [first, second]
        .into_iter()
        .chain(
            owner
                .recipe()
                .recipe_parameters()
                .iter()
                .map(|name| symbol!(*name)),
        )
        .collect::<Vec<_>>();
    let certificate = DynamicCheckProgram::build(
        &source,
        &owner.charts()[0],
        metadata.charts()[0].contour().unwrap(),
        &runtime,
        CompilationSettings::default(),
    )
    .unwrap();
    let owner = owner.with_certificates(vec![certificate]).unwrap();
    owner
        .validate_generation(Some(&metadata), &runtime)
        .unwrap();
    let mut permuted = runtime.clone();
    permuted.swap(0, 1);
    let error = owner
        .validate_generation(Some(&metadata), &permuted)
        .unwrap_err()
        .to_string();
    assert!(error.contains("runtime inputs differ"), "{error}");
    // Native symbol identity, not an alternate serialized namespace spelling,
    // determines input equality. No arbitrary physical value is substituted.
    let mut exact_only = owner.clone();
    exact_only.charts.clear();
    for certificate in std::sync::Arc::make_mut(exact_only.certificates.as_mut().unwrap()) {
        certificate.chart_index = None;
    }
    exact_only.validate_generation(None, &runtime).unwrap();
    assert!(exact_only.validate_generation(None, &permuted).is_err());
}

#[test]
fn untagged_exact_associations_union_contexts_after_native_cancellation() {
    use crate::contour::functions::dynamic::requests::merge_exact_requests;
    let (mut owner, source, _) = fixture_with_causal_sign(CompilationSettings::default(), 1);
    let (mut opposite, other, _) = fixture_with_causal_sign(CompilationSettings::default(), -1);
    assert_ne!(source.namespace, other.namespace);
    let prepare = |source: &DynamicCheckSource, owner: &NativeProgramDescriptor| {
        let mut lookup = Lookup::default();
        lookup
            .insert(
                &source.namespace,
                &source.full_strength,
                &source.parameters,
                &owner.certificates().unwrap()[0].faces,
            )
            .unwrap();
        let exact = source
            .full_strength
            .replace(Atom::var(source.parameters[0]))
            .with(0);
        let requests = lookup.exact_requests(std::slice::from_ref(&exact)).unwrap();
        (exact, requests)
    };
    let (left, mut requests) = prepare(&source, &owner);
    let (right, other_requests) = prepare(&other, &opposite);
    assert_eq!(
        left, right,
        "opposite F directions have the same polynomial radius"
    );
    requests.extend(other_requests);
    opposite.remap_charts(&[1]).unwrap();
    owner.merge(opposite).unwrap();
    let exact = vec![&left - &right, &left + &right];
    assert!(exact[0].is_zero());
    let merged = merge_exact_requests(&exact, requests.clone()).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].bundle.0.len(), 2);
    assert!(
        owner.for_payload(&[], &exact, None).is_err(),
        "untagged surviving roots require explicit associations"
    );
    let retained = owner
        .for_payload_with_requests(&[], &exact, None, &merged)
        .unwrap();
    assert_eq!(retained.helpers.len(), 1);
    assert_eq!(retained.certificates().unwrap().len(), 2);
    assert!(
        retained
            .certificates()
            .unwrap()
            .iter()
            .all(|c| c.chart_index.is_none())
    );
    let cancelled = vec![&left - &right];
    let merged = merge_exact_requests(&cancelled, requests).unwrap();
    assert!(merged.is_empty());
    let empty = owner
        .for_payload_with_requests(&[], &cancelled, None, &merged)
        .unwrap();
    assert!(empty.helpers.is_empty());
    assert!(empty.certificates().unwrap().is_empty());
    // Losing a retained face context cannot be hidden by untagged mathematics.
    let requests = prepare(&source, &owner).1;
    let mut corrupt = requests.clone();
    corrupt[0].bundle.0[0].face = vec![(2, 0)];
    assert!(
        owner
            .for_payload_with_requests(&[], &[left], None, &corrupt)
            .is_err()
    );
}

fn exact_request(source: &DynamicCheckSource, faces: &[Vec<(usize, u8)>]) -> Atom {
    let mut lookup = Lookup::default();
    lookup
        .insert(
            &source.namespace,
            &source.full_strength,
            &source.parameters,
            faces,
        )
        .unwrap();
    let restricted = source
        .full_strength
        .replace(Atom::var(source.parameters[0]))
        .with(Atom::Zero);
    lookup.lower(&restricted, requested::symbol()).unwrap()
}

#[test]
fn saved_v11_descriptor_restores_arithmetic_without_symbolic_sources() {
    let (owner, _, metadata) = fixture(CompilationSettings::default());
    let saved = SavedProgramDescriptorV2::from_native(&owner).unwrap();
    let bytes = bincode::serde::encode_to_vec(saved, bincode::config::standard()).unwrap();
    let original = owner.certificates().unwrap()[0].raw_program.clone();
    drop(owner);
    let (saved, used): (SavedProgramDescriptorV2, _) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    let restored = saved.restore().unwrap();
    let certificate = &restored.certificates().unwrap()[0];
    certificate.validate_metadata(&metadata).unwrap();
    assert_eq!(certificate.raw_program, original);
    let (raw, coefficients) = certificate.restore_arithmetic().unwrap();
    assert_eq!(raw.get_output_len(), certificate.schema.len());
    assert_eq!(
        coefficients.get_output_len(),
        certificate.structure.coefficient_count
    );
    assert!(
        restored
            .to_staging_bytes()
            .unwrap_err()
            .to_string()
            .contains("downgraded")
    );
}

#[test]
fn exact_selection_retains_only_surviving_request_contexts_and_remaps_projections() {
    let (owner, source, _) = fixture(CompilationSettings::default());
    let exact = exact_request(&source, &owner.certificates().unwrap()[0].faces);
    let retained = owner.for_payload(&[], &[exact], None).unwrap();
    assert!(retained.charts.is_empty());
    assert_eq!(retained.helpers.len(), 1);
    assert_eq!(retained.certificates().unwrap().len(), 1);
    assert_eq!(retained.certificates().unwrap()[0].chart_index, None);
    let restored = SavedProgramDescriptorV2::from_native(&retained)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(
        restored.certificates().unwrap()[0].namespace,
        source.namespace
    );
    let empty = owner.for_payload(&[], &[Atom::Zero], None).unwrap();
    assert!(empty.certificates().unwrap().is_empty());
    assert!(empty.helpers.is_empty());
    let mut selected = owner.for_payload(&[0], &[], None).unwrap();
    selected.remap_charts(&[17]).unwrap();
    assert_eq!(selected.charts[0].chart_index, 17);
    assert_eq!(selected.certificates().unwrap()[0].chart_index, Some(17));
    assert!(owner.clone().remap_charts(&[]).is_err());
}

#[test]
fn equal_namespaces_keep_distinct_native_programs_and_chart_projections() {
    let (mut first, source, _) = fixture(CompilationSettings::default());
    let (mut second, _, _) = fixture(CompilationSettings {
        horner_iterations: 0,
        cpe_rounds: Some(0),
        ..Default::default()
    });
    let left = first.certificates().unwrap()[0].clone();
    let right = second.certificates().unwrap()[0].clone();
    assert_eq!(left.namespace, right.namespace);
    assert_ne!(
        left.raw_program, right.raw_program,
        "native optimization controls should exercise distinct encodings"
    );
    let map = |bytes: &[u8]| {
        program::decode(bytes)
            .unwrap()
            .map_coeff(&|value| Complex::new(value.re.to_f64(), value.im.to_f64()))
    };
    let mut a = map(&left.raw_program);
    let mut b = map(&right.raw_program);
    let point = vec![Complex::new(0.2, 0.); a.get_input_len()];
    let mut values_a = vec![Complex::new(0., 0.); a.get_output_len()];
    let mut values_b = values_a.clone();
    a.evaluate(&point, &mut values_a);
    b.evaluate(&point, &mut values_b);
    for (a, b) in values_a.iter().zip(values_b) {
        assert!((a.re - b.re).abs() < 1e-10 * (1. + a.re.abs()));
        assert!((a.im - b.im).abs() < 1e-10 * (1. + a.im.abs()));
    }
    second.remap_charts(&[4]).unwrap();
    first.merge(second).unwrap();
    assert_eq!(first.certificates().unwrap().len(), 2);
    assert_eq!(
        first
            .charts
            .iter()
            .map(|chart| chart.chart_index)
            .collect::<Vec<_>>(),
        vec![0, 4]
    );
    let reversed = first.for_payload(&[4, 0], &[], None).unwrap();
    assert_eq!(
        reversed
            .charts
            .iter()
            .map(|chart| chart.chart_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        reversed
            .certificates()
            .unwrap()
            .iter()
            .map(|certificate| certificate.chart_index)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(0)]
    );
    assert!(
        first
            .for_payload(&[0, 0], &[], None)
            .unwrap_err()
            .to_string()
            .contains("duplicate requested source")
    );
    let mut legacy = first.clone();
    legacy.certificates = None;
    assert!(legacy.for_payload(&[0, 0], &[], None).is_err());
    let exact = exact_request(&source, &left.faces);
    let aggregate = first.for_payload(&[], &[exact], None).unwrap();
    assert_eq!(aggregate.certificates().unwrap().len(), 2);
    assert!(
        aggregate
            .certificates()
            .unwrap()
            .iter()
            .all(|certificate| certificate.chart_index.is_none())
    );
    SavedProgramDescriptorV2::from_native(&aggregate)
        .unwrap()
        .restore()
        .unwrap();
}

#[test]
fn certificate_admission_rejects_missing_proofs_foreign_metadata_and_forged_faces() {
    let (owner, source, metadata) = fixture(CompilationSettings::default());
    let certificate = owner.certificates().unwrap()[0].clone();
    assert!(owner.clone().with_certificates(vec![]).is_err());
    assert!(
        owner
            .clone()
            .with_certificates(vec![certificate.clone(), certificate.clone()])
            .is_err()
    );
    let mut corrupt = certificate.clone();
    corrupt.namespace = "a".repeat(64);
    assert!(owner.clone().with_certificates(vec![corrupt]).is_err());
    let mut corrupt = certificate.clone();
    corrupt.structure.positive_proofs[0] = PositiveFactorProof::NonnegativeCoefficients {
        lower_bound: Rational::from(0),
    };
    assert!(owner.clone().with_certificates(vec![corrupt]).is_err());
    let mut foreign = metadata;
    foreign.charts[0]
        .contour
        .as_mut()
        .unwrap()
        .causal_polynomial += Atom::one();
    assert!(certificate.validate_metadata(&foreign).is_err());
    let mut corrupt = certificate.clone();
    corrupt.raw_program.truncate(corrupt.raw_program.len() / 2);
    assert!(corrupt.restore_arithmetic().is_err());
    let mut corrupt = certificate.clone();
    corrupt.faces.retain(|face| !face.contains(&(0, 0)));
    let owner = owner.with_certificates(vec![corrupt]).unwrap();
    let exact = exact_request(&source, &certificate.faces);
    assert!(owner.for_payload(&[], &[exact], None).is_err());
}
