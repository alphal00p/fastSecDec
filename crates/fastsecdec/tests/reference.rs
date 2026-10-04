use fastsecdec::{
    integration::VectorEstimate,
    reference::*,
    status::CoefficientComponent::{Imag, Real},
};

fn key(order: i32, component: fastsecdec::status::CoefficientComponent) -> CoefficientKey {
    CoefficientKey { order, component }
}
fn estimate(rows: &[(CoefficientKey, f64, f64)]) -> VectorEstimate {
    VectorEstimate {
        orders: rows.iter().map(|(k, _, _)| k.order).collect(),
        components: rows.iter().map(|(k, _, _)| k.component).collect(),
        mean: rows.iter().map(|(_, m, _)| *m).collect(),
        standard_error: rows.iter().map(|(_, _, e)| *e).collect(),
        covariance_of_mean: vec![0.0; rows.len() * rows.len()],
        production_complete: true,
    }
}
fn reference(rows: &[(CoefficientKey, f64, ReferenceUncertainty)]) -> ReferenceResult {
    let mut result = ReferenceResult::new(
        rows.iter()
            .map(|(key, value, uncertainty)| ReferenceCoefficient {
                key: *key,
                value: *value,
                uncertainty: *uncertainty,
            })
            .collect(),
        ReferenceProvenance::new("independent fixture", "unit measure"),
    );
    result.validation = ReferenceValidation::Checked {
        evidence: "analytic fixture".into(),
    };
    result
}
fn context() -> ComparisonContext {
    ComparisonContext {
        kernel_content_id: "kernel-fixture".into(),
        normalization: Compatibility::Confirmed {
            basis: "same unit measure".into(),
        },
        kinematics: Compatibility::Confirmed {
            basis: "same exact invariants".into(),
        },
        independence: Independence::Independent {
            basis: "independent native oracle".into(),
        },
    }
}

#[test]
fn aligns_sparse_complex_keys_without_inventing_missing_zeros() {
    let actual = estimate(&[
        (key(0, Imag), 9.0, 1.0),
        (key(-1, Real), 2.0, 0.5),
        (key(2, Real), 7.0, 1.0),
    ]);
    let expected = reference(&[
        (key(-1, Real), 1.0, ReferenceUncertainty::Exact),
        (key(0, Imag), 6.0, ReferenceUncertainty::StandardError(2.0)),
        (key(0, Real), 0.0, ReferenceUncertainty::Exact),
    ]);
    let report = compare(&actual, &expected, &context()).unwrap();
    assert_eq!(
        report.rows.iter().map(|r| r.key).collect::<Vec<_>>(),
        [key(-1, Real), key(0, Real), key(0, Imag), key(2, Real)]
    );
    assert_eq!(report.rows[0].pull, Pull::Value(2.0));
    assert_eq!(
        report.rows[1].pull,
        Pull::Unavailable(UnavailablePull::MissingEstimate)
    );
    assert_eq!(report.rows[2].combined_standard_error, Some(5f64.sqrt()));
    assert_eq!(report.rows[2].pull, Pull::Value(3.0 / 5f64.sqrt()));
    assert_eq!(
        report.rows[3].pull,
        Pull::Unavailable(UnavailablePull::MissingReference)
    );
    assert!(!report.eligibility.eligible);
    assert!(report.rows[1].difference.is_none());
}

#[test]
fn unknown_errors_and_explicit_exact_zero_have_distinct_meanings() {
    let actual = estimate(&[(key(0, Real), 0.0, 0.0), (key(1, Real), 1.0, 0.0)]);
    let expected = reference(&[
        (key(0, Real), 0.0, ReferenceUncertainty::Exact),
        (key(1, Real), 0.0, ReferenceUncertainty::Exact),
    ]);
    let report = compare(&actual, &expected, &context()).unwrap();
    assert_eq!(report.rows[0].pull, Pull::ZeroCombinedError { equal: true });
    assert_eq!(
        report.rows[1].pull,
        Pull::ZeroCombinedError { equal: false }
    );
    assert!(report.rows.iter().all(|r| r.relative_difference.is_none()));
    assert!(report.eligibility.eligible); // Eligibility is not agreement/truth.
    serde_json::to_string(&report).unwrap();
    let mut unknown = expected;
    unknown.coefficients[0].uncertainty = ReferenceUncertainty::Unknown;
    let report = compare(&actual, &unknown, &context()).unwrap();
    assert_eq!(
        report.rows[0].pull,
        Pull::Unavailable(UnavailablePull::UnknownReferenceUncertainty)
    );
    assert_eq!(report.rows[0].combined_standard_error, None);
    assert!(!report.eligibility.eligible);
    assert!(report.to_string().contains("uncertainty unknown"));
}

#[test]
fn eligibility_records_coverage_evidence_and_independence_without_overriding_identity() {
    let mut actual = estimate(&[(key(0, Real), 2.0, 1.0)]);
    let mut expected = reference(&[(key(0, Real), 1.0, ReferenceUncertainty::StandardError(1.0))]);
    assert!(
        compare(&actual, &expected, &context())
            .unwrap()
            .eligibility
            .eligible
    );
    actual.production_complete = false;
    expected.validation = ReferenceValidation::Unverified;
    let mut assumptions = context();
    assumptions.normalization = Compatibility::Unknown;
    assumptions.kinematics = Compatibility::Mismatch {
        detail: "different invariant".into(),
    };
    assumptions.independence = Independence::Unknown;
    let report = compare(&actual, &expected, &assumptions).unwrap();
    for reason in [
        IneligibilityReason::IncompleteProduction,
        IneligibilityReason::UnverifiedReference,
        IneligibilityReason::NormalizationUnconfirmed,
        IneligibilityReason::KinematicsMismatch,
        IneligibilityReason::IndependenceUnconfirmed,
    ] {
        assert!(report.eligibility.reasons.contains(&reason));
    }
    assert_eq!(
        report.rows[0].pull,
        Pull::Unavailable(UnavailablePull::IndependenceUnconfirmed)
    );
    assumptions.independence = Independence::Correlated {
        detail: "shared samples".into(),
    };
    let report = compare(&actual, &expected, &assumptions).unwrap();
    assert_eq!(
        report.rows[0].pull,
        Pull::Unavailable(UnavailablePull::EstimatesCorrelated)
    );
    assert_eq!(report.rows[0].combined_standard_error, None);
    assumptions.independence = Independence::Independent {
        basis: String::new(),
    };
    assert!(compare(&actual, &expected, &assumptions).is_err());
    expected.kernel_content_id = Some("different-scientific-content".into());
    assert!(matches!(
        compare(&actual, &expected, &context()),
        Err(ReferenceError::KernelIdentityMismatch { .. })
    ));
}

#[test]
fn rejects_duplicate_keys_nonfinite_values_and_invalid_uncertainties() {
    let actual = estimate(&[(key(0, Real), 2.0, 1.0)]);
    let expected = reference(&[(key(0, Real), 1.0, ReferenceUncertainty::Exact)]);
    let duplicate = estimate(&[(key(0, Real), 2.0, 1.0), (key(0, Real), 2.0, 1.0)]);
    assert!(compare(&duplicate, &expected, &context()).is_err());
    let mut duplicate = expected.clone();
    duplicate
        .coefficients
        .push(duplicate.coefficients[0].clone());
    assert!(compare(&actual, &duplicate, &context()).is_err());
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        let mut value = expected.clone();
        value.coefficients[0].uncertainty = ReferenceUncertainty::StandardError(bad);
        assert!(compare(&actual, &value, &context()).is_err());
    }
    let mut value = actual.clone();
    value.covariance_of_mean.clear();
    assert!(compare(&value, &expected, &context()).is_err());
    let mut value = actual;
    value.mean[0] = f64::NAN;
    assert!(compare(&value, &expected, &context()).is_err());
}

#[test]
fn finite_inputs_cannot_emit_nonfinite_derived_values() {
    for (value, target, error, target_error, quantity) in [
        (f64::MAX, -f64::MAX, 1.0, 1.0, "difference"),
        (1.0, f64::from_bits(1), 1.0, 1.0, "relative difference"),
        (1.0, 1.0, f64::MAX, f64::MAX, "combined standard error"),
        (1.0, 0.0, f64::from_bits(1), 0.0, "pull"),
    ] {
        let actual = estimate(&[(key(0, Real), value, error)]);
        let expected = reference(&[(
            key(0, Real),
            target,
            ReferenceUncertainty::StandardError(target_error),
        )]);
        assert!(
            matches!(compare(&actual,&expected,&context()),Err(ReferenceError::NumericRange { quantity:q,.. }) if q==quantity)
        );
    }
    let actual = estimate(&[(key(0, Real), 1.0, 1e308)]);
    let expected = reference(&[(
        key(0, Real),
        0.0,
        ReferenceUncertainty::StandardError(1e308),
    )]);
    assert!(
        compare(&actual, &expected, &context()).unwrap().rows[0]
            .combined_standard_error
            .unwrap()
            .is_finite()
    );
}

#[test]
fn historical_targets_preserve_unknown_errors_and_unverified_provenance() {
    for bytes in [
        include_bytes!("../../../examples/targets/double_box.json").as_slice(),
        include_bytes!("../../../examples/targets/issue_1.json").as_slice(),
    ] {
        let target = read_historical_target(bytes).unwrap();
        assert_eq!(target.validation, ReferenceValidation::Unverified);
        assert!(
            target
                .coefficients
                .iter()
                .all(|c| c.uncertainty == ReferenceUncertainty::Unknown)
        );
        assert!(target.provenance.revision.is_some());
        let actual = estimate(
            &target
                .coefficients
                .iter()
                .map(|c| (c.key, c.value, 1.0))
                .collect::<Vec<_>>(),
        );
        let result = compare(&actual, &target, &context()).unwrap();
        assert!(
            result
                .rows
                .iter()
                .all(|r| r.pull == Pull::Unavailable(UnavailablePull::UnknownReferenceUncertainty))
        );
    }
    for bytes in [
        include_bytes!("../../../examples/targets/box.json").as_slice(),
        include_bytes!("../../../examples/targets/triangle.json").as_slice(),
        include_bytes!("../../../examples/targets/four_loop_hard.json").as_slice(),
    ] {
        let target = read_historical_target(bytes).unwrap();
        assert_eq!(target.validation, ReferenceValidation::Unverified);
        assert!(
            target
                .coefficients
                .iter()
                .all(|c| matches!(c.uncertainty, ReferenceUncertainty::StandardError(_)))
        );
    }
}

#[test]
fn native_versioned_envelope_round_trips_without_changing_reference_meaning() {
    let expected = reference(&[(key(0, Real), 2.0, ReferenceUncertainty::Exact)]);
    let bytes = encode_reference(&expected).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope["format"], "fastsecdec-reference");
    assert_eq!(envelope["version"], 1);
    assert_eq!(read_reference(&bytes).unwrap(), expected);
    assert!(read_reference(&serde_json::to_vec(&expected).unwrap()).is_err());
    let mut invalid = expected;
    invalid.coefficients[0].value = f64::NAN;
    assert!(encode_reference(&invalid).is_err());
    assert!(invalid.validate().is_err());
}

#[test]
fn document_reader_rejects_ambiguous_versions_and_malformed_shapes() {
    let expected = reference(&[(key(0, Real), 2.0, ReferenceUncertainty::Exact)]);
    let envelope: serde_json::Value =
        serde_json::from_slice(&encode_reference(&expected).unwrap()).unwrap();
    let mut variants = Vec::new();
    let mut value = envelope.clone();
    value["version"] = 2.into();
    variants.push(value);
    let mut value = envelope.clone();
    value["format"] = "another-format".into();
    variants.push(value);
    let mut value = envelope.clone();
    value["schema_version"] = 1.into();
    variants.push(value);
    let mut value = envelope.clone();
    value["orders"] = serde_json::json!([0]);
    variants.push(value);
    let mut value = envelope.clone();
    value["reference"] = serde_json::Value::Null;
    variants.push(value);
    let mut value = envelope;
    value["unexpected"] = true.into();
    variants.push(value);
    variants.extend([
        serde_json::json!([]),
        serde_json::json!({}),
        serde_json::json!({"schema_version":2}),
    ]);
    for value in variants {
        assert!(
            read_reference(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn automatic_historical_dispatch_keeps_null_uncertainty() {
    let bytes = include_bytes!("../../../examples/targets/double_box.json");
    assert_eq!(
        read_reference(bytes).unwrap(),
        read_historical_target(bytes).unwrap()
    );
    let mut invalid: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    invalid["standard_errors"] = serde_json::json!([]);
    assert!(read_reference(&serde_json::to_vec(&invalid).unwrap()).is_err());
}
