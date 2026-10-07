use crate::{
    Atom,
    generation::{GenerationOptions, generate},
    kernel::{KernelSet, cancellation::Cancellation},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

fn native_template() -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("sector_identity::x"), symbol!("sector_identity::y")],
        symbol!("sector_identity::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("sector_identity::x+sector_identity::y"),
                    parse!("-1-sector_identity::eps"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("1+sector_identity::x"),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
            ],
        )],
    )
    .unwrap();
    generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile()
    .unwrap()
}

#[test]
fn native_complex_sector_identity_survives_reload_and_evaluation() {
    let input = ParametricIntegrand::new(
        vec![symbol!("sector_identity::x")],
        symbol!("sector_identity::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("(2+3𝑖)*gamma(1+sector_identity::eps)"),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+sector_identity::x"),
                parse!("sector_identity::eps-1"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut fresh = generated.compile().unwrap();
    assert_eq!(fresh.orders(), [0, 0, 1, 1, 2, 2]);
    let id = fresh.sector_content_id(0).unwrap();
    assert!(id.starts_with("fsd-sector-v1:"));
    assert!(fresh.sector_content_id(fresh.sectors.len()).is_err());
    let bytes = fresh.to_bytes().unwrap();
    let mut loaded = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(loaded.sector_content_id(0).unwrap(), id);
    for point in [0.25, 1e-8] {
        let mut a = vec![0.; 6];
        let mut b = vec![0.; 6];
        fresh.sectors_mut()[0].evaluate(&[point], &mut a).unwrap();
        loaded.sectors_mut()[0].evaluate(&[point], &mut b).unwrap();
        for (a, b) in a.iter().zip(b) {
            assert!((a - b).abs() < 1e-11);
        }
    }
    assert_eq!(fresh.sector_content_id(0).unwrap(), id);
    assert_eq!(loaded.sector_content_id(0).unwrap(), id);
    assert_eq!(fresh.to_bytes().unwrap(), bytes);
    assert_eq!(loaded.to_bytes().unwrap(), bytes);
}

#[test]
fn native_identity_is_additive_and_missing_metadata_stays_distinct() {
    let mut original = native_template();
    let with_metadata = original.to_bytes().unwrap();
    original.metadata = None;
    original.initialize_artifact().unwrap();
    let without_metadata = original.to_bytes().unwrap();
    let mut ids = Vec::new();
    for bytes in [&with_metadata, &without_metadata] {
        let loaded = KernelSet::from_bytes(bytes).unwrap();
        let parent = loaded.content_id().to_owned();
        let id = loaded.sector_content_id(0).unwrap();
        assert_eq!(
            KernelSet::from_bytes(bytes)
                .unwrap()
                .sector_content_id(0)
                .unwrap(),
            id
        );
        assert_eq!(loaded.to_bytes().unwrap(), *bytes);
        assert_eq!(loaded.content_id(), parent);
        ids.push(id);
    }
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn optional_chart_preview_can_be_absent_in_saved_native_ir() {
    let mut kernels = native_template();
    for chart in &mut kernels.metadata.as_mut().unwrap().charts {
        chart.pre_subtraction = None;
    }
    kernels.initialize_artifact().unwrap();
    let bytes = kernels.to_bytes().unwrap();
    let restored = KernelSet::from_bytes(&bytes).unwrap();
    let metadata = restored.generation_metadata().unwrap();
    assert!(
        metadata
            .charts()
            .iter()
            .all(|chart| chart.pre_subtraction().is_none())
    );
    let portable = crate::kernel::PortableMetadata::from_native(metadata);
    assert!(
        !serde_json::to_string(&portable)
            .unwrap()
            .contains("pre_subtraction")
    );
    assert_eq!(restored.content_id(), kernels.content_id());
    assert_eq!(restored.to_bytes().unwrap(), bytes);
}

#[test]
fn selected_identity_ignores_other_kernels_offsets_and_parent_ordinal() {
    let mut kernels = native_template();
    let id = kernels.sector_content_id(0).unwrap();
    let parent = kernels.content_id().to_owned();
    assert!(kernels.sectors.len() > 1);
    // Reordering valid native kernels and updating their retained associations
    // leaves the selected representation and its original charts unchanged.
    kernels.sectors.swap(0, 1);
    for chart in &mut kernels.metadata.as_mut().unwrap().charts {
        chart.kernel_sector = chart.kernel_sector.map(|index| match index {
            0 => 1,
            1 => 0,
            other => other,
        });
    }
    kernels.exact_coefficients[0] += 17.;
    kernels.exact_expressions[0] = &kernels.exact_expressions[0] + Atom::num(17);
    kernels.sectors[0].precision.relative_tolerance *= 2.;
    kernels.initialize_artifact().unwrap();
    assert_ne!(kernels.content_id(), parent);
    assert_eq!(kernels.sector_content_id(1).unwrap(), id);
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
}

#[test]
fn numerical_policy_layout_and_retained_semantics_bind_selected_identity() {
    let mut kernels = native_template();
    let id = kernels.sector_content_id(0).unwrap();
    let precision = kernels.sectors[0].precision.clone();
    kernels.sectors[0].precision.relative_tolerance *= 2.;
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.sectors[0].precision = precision;
    assert_eq!(kernels.sector_content_id(0).unwrap(), id);

    let cancellation = kernels.sectors[0].cancellation.clone();
    kernels.sectors[0].cancellation = Cancellation::new(1, None, 2).unwrap();
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.sectors[0].cancellation = cancellation;

    let program = kernels.sectors[0].program_bytes.clone();
    kernels.sectors[0].program_bytes = kernels.sectors[1].program_bytes.clone();
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.sectors[0].program_bytes = program;

    // These private identity-boundary controls deliberately vary one field;
    // they do not claim to be loadable altered scientific artifacts.
    kernels.orders[0] -= 1;
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.orders[0] += 1;
    let component = kernels.components[0];
    kernels.components[0] = crate::status::CoefficientComponent::Imag;
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.components[0] = component;

    let old_parameter = kernels.sectors[0].parameters[0];
    kernels.sectors[0].parameters[0] = symbol!("sector_identity::renamed");
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.sectors[0].parameters[0] = old_parameter;

    let metadata = kernels.metadata.clone();
    let retained = kernels.metadata.as_mut().unwrap();
    retained.domain.caller_asserted = !retained.domain.caller_asserted;
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.metadata = metadata.clone();
    let chart = kernels
        .metadata
        .as_mut()
        .unwrap()
        .charts
        .iter_mut()
        .find(|chart| chart.kernel_sector == Some(0))
        .unwrap();
    chart.coordinates.measure_jacobian = Atom::num(2);
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.metadata = None;
    assert_ne!(kernels.sector_content_id(0).unwrap(), id);
    kernels.metadata = metadata;
    assert_eq!(kernels.sector_content_id(0).unwrap(), id);
}
