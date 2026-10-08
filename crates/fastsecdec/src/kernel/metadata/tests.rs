//! Native semantic admission controls, independent of the cache wire envelope.
use super::PortableMetadata;
use crate::{
    generation::{GenerationOptions, generate},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{atom::Atom, parse, symbol};

#[test]
fn stored_maps_restore_exactly_in_projective_and_signed_orthant_domains() {
    let x = symbol!("metadata_domains::x");
    let y = symbol!("metadata_domains::y");
    for domain in [
        ParametricDomain::ProjectiveSimplex,
        ParametricDomain::PositiveOrthant,
    ] {
        let projective = domain == ParametricDomain::ProjectiveSimplex;
        let polynomial = Atom::var(x) + Atom::var(y) + Atom::num(if projective { 0 } else { 1 });
        let input = ParametricIntegrand::new(
            vec![x, y],
            symbol!("metadata_domains::eps"),
            domain,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::Zero; 2],
                vec![PolynomialFactor::new(
                    polynomial,
                    Atom::num(if projective { -2 } else { -3 }),
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap();
        let generated = generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let parameters = generated
            .sectors()
            .iter()
            .map(|sector| sector.parameters().to_vec())
            .collect::<Vec<_>>();
        let fast = PortableMetadata::from_native(generated.metadata())
            .into_native(&parameters, false)
            .unwrap();
        let validated = PortableMetadata::from_native(generated.metadata())
            .into_native(&parameters, true)
            .unwrap();
        assert!(!fast.charts().is_empty());
        if !projective {
            assert!(fast.charts().iter().any(|chart| {
                chart
                    .geometry()
                    .exponent_matrix
                    .iter()
                    .flatten()
                    .any(|power| power < &symbolica::domains::integer::Integer::from(0))
            }));
        }
        assert_eq!(
            serde_json::to_value(PortableMetadata::from_native(&fast)).unwrap(),
            serde_json::to_value(PortableMetadata::from_native(&validated)).unwrap()
        );
        for (left, right) in fast.charts().iter().zip(validated.charts()) {
            assert_eq!(
                left.coordinates.measure_factor,
                right.coordinates.measure_factor
            );
            assert_eq!(
                left.coordinates.measure_powers,
                right.coordinates.measure_powers
            );
            assert_eq!(
                left.coordinates.projective_fixed_parameter(),
                right.coordinates.projective_fixed_parameter()
            );
        }
    }
}

#[test]
fn altered_chart_semantics_are_rejected_by_native_metadata_admission() {
    let input = ParametricIntegrand::new(
        vec![symbol!("metadata_admission::x")],
        symbol!("metadata_admission::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+metadata_admission::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let parameters = generated
        .sectors()
        .iter()
        .map(|sector| sector.parameters().to_vec())
        .collect::<Vec<_>>();
    let original =
        serde_json::to_value(PortableMetadata::from_native(generated.metadata())).unwrap();
    let decode = |value, validate| {
        serde_json::from_value::<PortableMetadata>(value)
            .unwrap()
            .into_native(&parameters, validate)
    };
    let fast = decode(original.clone(), false).unwrap();
    let validated = decode(original.clone(), true).unwrap();
    assert_eq!(
        serde_json::to_value(PortableMetadata::from_native(&fast)).unwrap(),
        serde_json::to_value(PortableMetadata::from_native(&validated)).unwrap()
    );
    for (a, b) in fast.charts().iter().zip(validated.charts()) {
        assert_eq!(a.coordinates.measure_factor, b.coordinates.measure_factor);
        assert_eq!(a.coordinates.measure_powers, b.coordinates.measure_powers);
    }
    for (path, value) in [
        ("/charts/0/representative", serde_json::json!(999)),
        (
            "/charts/0/representative_permutation",
            serde_json::json!([1]),
        ),
        ("/charts/0/kernel_sector", serde_json::json!(999)),
        ("/charts/0/images", serde_json::json!([])),
        (
            "/charts/0/geometry/exponent_matrix",
            serde_json::json!([[]]),
        ),
        ("/charts/0/pre_subtraction/version", serde_json::json!(2)),
        (
            "/charts/0/pre_subtraction/terms/0/powers",
            serde_json::json!([]),
        ),
    ] {
        let mut modified = original.clone();
        *modified
            .pointer_mut(path)
            .expect("native presentation path") = value;
        for validate in [false, true] {
            assert!(
                decode(modified.clone(), validate).is_err(),
                "accepted altered {path}"
            );
        }
    }
    for (path, value) in [
        ("/charts/0/measure_jacobian", serde_json::json!("2")),
        ("/charts/0/geometry/determinant", serde_json::json!("2")),
        (
            "/domain/factors/0/polynomial",
            serde_json::json!("sin(metadata_admission::x)"),
        ),
    ] {
        let mut modified = original.clone();
        *modified
            .pointer_mut(path)
            .expect("native presentation path") = value;
        assert!(
            decode(modified.clone(), false).is_ok(),
            "fast restore rejected {path}"
        );
        assert!(
            decode(modified, true).is_err(),
            "validation accepted {path}"
        );
    }
    // Historical certificate labels remain metadata; native loading performs
    // structural admission only and deliberately does not recertify thresholds.
    let mut historical = original;
    historical["domain"]["branch"] = serde_json::json!("NoThresholdReal");
    historical["domain"]["factors"][0]["certificate"] = serde_json::json!("PositiveCoefficients");
    let historical = decode(historical, true).unwrap();
    assert_eq!(
        historical.domain_assessment().branch_policy(),
        crate::generation::BranchPolicy::NoThresholdReal
    );
    assert_eq!(
        historical.domain_assessment().factors()[0].certificate(),
        crate::generation::FactorCertificate::PositiveCoefficients
    );
}
