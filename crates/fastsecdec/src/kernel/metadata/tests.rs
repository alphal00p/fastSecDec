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
    let decode = |value| {
        serde_json::from_value::<PortableMetadata>(value)
            .unwrap()
            .into_native(&parameters)
    };
    assert!(decode(original.clone()).is_ok());
    for (path, value) in [
        ("/charts/0/representative", serde_json::json!(999)),
        (
            "/charts/0/representative_permutation",
            serde_json::json!([1]),
        ),
        ("/charts/0/kernel_sector", serde_json::json!(999)),
        ("/charts/0/measure_jacobian", serde_json::json!("2")),
        ("/charts/0/geometry/determinant", serde_json::json!("2")),
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
        assert!(decode(modified).is_err(), "accepted altered {path}");
    }
    // Historical certificate labels remain metadata; native loading performs
    // structural admission only and deliberately does not recertify thresholds.
    let mut historical = original;
    historical["domain"]["branch"] = serde_json::json!("NoThresholdReal");
    historical["domain"]["factors"][0]["certificate"] = serde_json::json!("PositiveCoefficients");
    let historical = decode(historical).unwrap();
    assert_eq!(
        historical.domain_assessment().branch_policy(),
        crate::generation::BranchPolicy::NoThresholdReal
    );
    assert_eq!(
        historical.domain_assessment().factors()[0].certificate(),
        crate::generation::FactorCertificate::PositiveCoefficients
    );
}
