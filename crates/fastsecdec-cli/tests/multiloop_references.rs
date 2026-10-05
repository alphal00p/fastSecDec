//! Frozen external evidence uses the public native reference transport.
#[path = "support/multiloop_reference.rs"]
mod support;

use std::{fs, path::Path};

use fastsecdec::{
    integration::VectorEstimate,
    reference::{
        CoefficientKey, Pull, ReferenceCoefficient, ReferenceProvenance, ReferenceResult,
        ReferenceUncertainty, ReferenceValidation, compare, encode_reference, read_reference,
    },
    status::CoefficientComponent,
};
use serde_json::{Value, json};

fn repository() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn saved_estimate(name: &str) -> (String, VectorEstimate) {
    let report: Value = serde_json::from_slice(
        &fs::read(repository().join(format!(
            "output/diagnostics/massive-multiloop/{name}.integrate.json"
        )))
        .unwrap(),
    )
    .unwrap();
    (
        report["content_id"].as_str().unwrap().into(),
        serde_json::from_value(report["estimate"].clone()).unwrap(),
    )
}

#[test]
fn frozen_references_preserve_uncertainty_projection_and_input_evidence() {
    for (name, loops, propagators) in support::CASES {
        let reference = support::load_reference(repository(), name);
        assert!(matches!(
            reference.validation,
            ReferenceValidation::Checked { .. }
        ));
        assert_eq!(reference.coefficients.len(), 1);
        let coefficient = &reference.coefficients[0];
        assert_eq!(coefficient.key.order, 0);
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert!(
            matches!(coefficient.uncertainty, ReferenceUncertainty::StandardError(error) if error.is_finite() && error > 0.0)
        );
        let attributes = &reference.provenance.attributes;
        assert_eq!(attributes["loops"], loops);
        assert_eq!(attributes["propagators"], propagators);
        assert_eq!(attributes["external_imaginary_value"], 0.0);
        assert_eq!(attributes["external_imaginary_standard_error"], 0.0);
        assert!(attributes["actual_evaluations"].is_null());
        assert!(attributes["actual_lattice_size"].is_null());
        assert_eq!(attributes["raw_report_blake3"].as_str().unwrap().len(), 64);
        assert_eq!(
            read_reference(&encode_reference(&reference).unwrap()).unwrap(),
            reference
        );
    }
}

#[test]
fn double_box_reference_preserves_the_complete_audited_laurent_vector() {
    let reference = read_reference(
        &fs::read(repository().join("examples/references/double_box.json")).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        reference.validation,
        ReferenceValidation::Checked { .. }
    ));
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        vec![-4, -3, -2, -1, 0]
    );
    for coefficient in &reference.coefficients {
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert!(matches!(coefficient.uncertainty,
            ReferenceUncertainty::StandardError(error) if error.is_finite() && error > 0.0));
    }
    // Preserve a tiny measured pole and its uncertainty; the independent exact
    // zero proof must not rewrite this numerical observation.
    assert_ne!(reference.coefficients[0].value, 0.0);
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["physical_tuple_index"], 2);
    assert_eq!(attributes["outer_prefactor"], "1");
    assert_eq!(attributes["constituent_prefactor"], "-Gamma(3+2*eps)");
    assert_eq!(
        attributes["external_imaginary_values"],
        json!([0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        attributes["external_imaginary_standard_errors"],
        json!([0.0, 0.0, 0.0, 0.0, 0.0])
    );
    assert!(attributes["actual_evaluations"].is_null());
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["requested_settings"]["seed"], 20261203);
    assert_eq!(attributes["kinematics"]["s12"], "-1");
    assert_eq!(attributes["kinematics"]["s23"], "-1");
    for source in attributes["native_sources"].as_array().unwrap() {
        let path = source["path"].as_str().unwrap();
        assert_eq!(
            blake3::hash(&fs::read(repository().join(path)).unwrap())
                .to_hex()
                .as_str(),
            source["blake3"].as_str().unwrap(),
            "source {path} changed; reassess reference compatibility"
        );
    }
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}

#[test]
fn issue_one_reference_preserves_full_orthant_data_without_certifying_omitted_covariance() {
    let reference =
        read_reference(&fs::read(repository().join("examples/references/issue_1.json")).unwrap())
            .unwrap();
    // The provider shares shifts between kernels but omits their covariance.
    // Source/normalization/transport review must not make this statistically eligible.
    assert_eq!(reference.validation, ReferenceValidation::Unverified);
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    let means = [10.353244236120734, 99.57228105084454, 760.7522873022788];
    let errors = [
        0.0009814070449116756,
        0.011963742555179839,
        0.09586600861575804,
    ];
    for (i, coefficient) in reference.coefficients.iter().enumerate() {
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert_eq!(coefficient.value, means[i]);
        assert_eq!(
            coefficient.uncertainty,
            ReferenceUncertainty::StandardError(errors[i])
        );
    }
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["reference_domain"], "positive_orthant");
    assert_eq!(attributes["dimension"], 7);
    assert_eq!(attributes["density"], "F^(eps-2)");
    assert_eq!(attributes["prefactor"], "1");
    assert_eq!(attributes["sum_coefficient"], "1");
    assert!(attributes["sector_filter"].is_null());
    assert_eq!(
        attributes["external_imaginary_values"],
        json!([0.0, 0.0, 0.0])
    );
    assert_eq!(
        attributes["external_imaginary_standard_errors"],
        json!([0.0, 0.0, 0.0])
    );
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["actual_evaluations"], 491_479_296_u64);
    assert_eq!(attributes["actual_lattice_size"], 8311);
    assert_eq!(attributes["shifts"], 32);
    assert_eq!(attributes["coefficient_kernels"], 1848);
    assert!(
        attributes["actual_evaluation_unit"]
            .as_str()
            .unwrap()
            .contains("scalar coefficient-kernel")
    );
    assert!(
        attributes["external_variance_policy"]
            .as_str()
            .unwrap()
            .contains("calibration unverified")
    );
    for source in attributes["native_sources"].as_array().unwrap() {
        assert_eq!(
            blake3::hash(&fs::read(repository().join(source["path"].as_str().unwrap())).unwrap())
                .to_hex()
                .as_str(),
            source["blake3"].as_str().unwrap()
        );
    }
    // Exercise the public eligibility boundary with an explicitly independent
    // comparison; unverified provider covariance must remain visible to callers.
    let estimate = VectorEstimate {
        orders: vec![0, 1, 2],
        components: vec![CoefficientComponent::Real; 3],
        mean: means.to_vec(),
        standard_error: vec![1.0; 3],
        covariance_of_mean: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        production_complete: true,
    };
    let context = fastsecdec::reference::ComparisonContext {
        kernel_content_id: "fixture-transport-control".into(),
        normalization: fastsecdec::reference::Compatibility::Confirmed {
            basis: "test control".into(),
        },
        kinematics: fastsecdec::reference::Compatibility::Confirmed {
            basis: "test control".into(),
        },
        independence: fastsecdec::reference::Independence::Independent {
            basis: "test control".into(),
        },
    };
    let comparison = compare(&estimate, &reference, &context).unwrap();
    assert!(!comparison.eligibility.eligible);
    assert_eq!(
        comparison.eligibility.reasons,
        vec![fastsecdec::reference::IneligibilityReason::UnverifiedReference]
    );
    assert!(
        comparison
            .rows
            .iter()
            .all(|row| matches!(row.pull, Pull::Value(0.0)))
    );
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}

#[test]
fn issue_one_together_reference_preserves_native_sector_sum_uncertainty() {
    let reference = read_reference(
        &fs::read(repository().join("examples/references/issue_1_together.json")).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        reference.validation,
        ReferenceValidation::Checked { .. }
    ));
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    let means = [10.34539579586942, 99.81872959411426, 760.9167768416243];
    let errors = [0.01622526223753092, 0.15313872419854435, 0.9781050881960318];
    for (i, coefficient) in reference.coefficients.iter().enumerate() {
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert_eq!(coefficient.value, means[i]);
        assert_eq!(
            coefficient.uncertainty,
            ReferenceUncertainty::StandardError(errors[i])
        );
    }
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["reference_domain"], "positive_orthant");
    assert_eq!(attributes["dimension"], 7);
    assert_eq!(attributes["density"], "F^(eps-2)");
    assert_eq!(attributes["prefactor"], "1");
    assert_eq!(attributes["sum_coefficient"], "1");
    assert_eq!(attributes["together"], true);
    assert_eq!(attributes["geometric_sectors"], 616);
    assert!(attributes["sector_filter"].is_null());
    // The ordinary real interface did not measure an imaginary component or
    // joint cross-order covariance. Neither may turn into invented zeros.
    assert!(attributes["external_imaginary_values"].is_null());
    assert!(attributes["external_imaginary_standard_errors"].is_null());
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["actual_evaluations"], 797856);
    assert_eq!(attributes["actual_lattice_size"], 8311);
    assert_eq!(attributes["shifts"], 32);
    assert_eq!(attributes["scalar_summed_coefficient_calls"], 3);
    assert!(
        attributes["actual_evaluation_unit"]
            .as_str()
            .unwrap()
            .contains("scalar summed-coefficient")
    );
    assert_eq!(attributes["calibration_certified"], false);
    assert_eq!(attributes["exactness"], false);
    assert_eq!(attributes["highest_order_target"], 2);
    assert_eq!(attributes["highest_order_target_met"], false);
    assert!(errors[2] / means[2].abs() > 0.001);
    for source in attributes["native_sources"].as_array().unwrap() {
        assert_eq!(
            blake3::hash(&fs::read(repository().join(source["path"].as_str().unwrap())).unwrap())
                .to_hex()
                .as_str(),
            source["blake3"].as_str().unwrap()
        );
    }
    let prior = &attributes["earlier_unverified_reference"];
    let bytes = fs::read(repository().join(prior["path"].as_str().unwrap())).unwrap();
    assert_eq!(
        blake3::hash(&bytes).to_hex().as_str(),
        prior["blake3"].as_str().unwrap()
    );
    assert_eq!(
        read_reference(&bytes).unwrap().validation,
        ReferenceValidation::Unverified
    );
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}

#[test]
fn offshell_scalar_triple_box_reference_preserves_all_orders_and_native_measure() {
    let reference = read_reference(
        &fs::read(repository().join("examples/references/triple_box_offshell_scalar.json"))
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        reference.validation,
        ReferenceValidation::Checked { .. }
    ));
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        vec![-3, -2, -1, 0]
    );
    // Provider IEEE values are independent transport oracles. Preserve the tiny
    // measured leading pole and positive errors; do not round it to exact zero.
    let mean_bits = [
        13_725_411_892_556_917_872,
        13_821_433_552_745_504_933,
        4_605_142_885_197_471_199,
        4_612_324_208_867_764_023,
    ];
    let error_bits = [
        4_504_790_491_703_934_771,
        4_569_642_537_828_582_985,
        4_580_356_015_104_913_395,
        4_588_402_102_740_449_436,
    ];
    for (i, coefficient) in reference.coefficients.iter().enumerate() {
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert_eq!(coefficient.value.to_bits(), mean_bits[i]);
        let ReferenceUncertainty::StandardError(error) = coefficient.uncertainty else {
            panic!("reported uncertainty must stay statistical")
        };
        assert_eq!(error.to_bits(), error_bits[i]);
        assert!(error > 0.0 && error.is_finite());
    }
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["loops"], 3);
    assert_eq!(attributes["original_propagators"], 10);
    assert_eq!(attributes["active_propagators"], 8);
    assert_eq!(
        attributes["original_order_projected_powers"],
        json!([1, 1, 0, 2, 0, 2, 1, 1, 1, 1])
    );
    assert_eq!(attributes["active_powers"], json!([1, 1, 2, 2, 1, 1, 1, 1]));
    assert_eq!(attributes["constituent_prefactor"], "Gamma(4+3*eps)");
    assert_eq!(attributes["additional_prefactor"], "1");
    assert_eq!(attributes["numerator"], "1");
    assert_eq!(attributes["physical_tuple_index"], 2);
    assert_eq!(attributes["geometric_sectors"], 1182);
    assert_eq!(attributes["together"], true);
    assert!(attributes["sector_filter"].is_null());
    assert_eq!(
        attributes["kinematics"]["external_virtualities"],
        json!(["-1", "-1", "-1", "-1"])
    );
    assert_eq!(attributes["kinematics"]["s12"], "-2");
    assert_eq!(attributes["kinematics"]["s23"], "-2");
    assert_eq!(
        attributes["external_imaginary_values"],
        json!([0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        attributes["external_imaginary_standard_errors"],
        json!([0.0, 0.0, 0.0, 0.0])
    );
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["actual_evaluations"], 1_063_808);
    assert_eq!(attributes["scalar_summed_coefficient_calls"], 4);
    assert!(
        attributes["actual_evaluation_unit"]
            .as_str()
            .unwrap()
            .contains("scalar summed-sector coefficient")
    );
    assert_eq!(attributes["calibration_certified"], false);
    assert_eq!(attributes["exactness"], false);
    assert_eq!(attributes["highest_order_target"], 0);
    assert_eq!(attributes["highest_order_target_met"], false);
    assert_ne!(
        attributes["native_kernel_content_ids"]["original"],
        attributes["native_kernel_content_ids"]["projected"]
    );
    for source in attributes["native_sources"].as_array().unwrap() {
        assert_eq!(
            blake3::hash(&fs::read(repository().join(source["path"].as_str().unwrap())).unwrap())
                .to_hex()
                .as_str(),
            source["blake3"].as_str().unwrap()
        );
    }
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}

#[test]
fn offshell_rank_two_triple_box_reference_preserves_numerator_and_physical_tuple() {
    let reference = read_reference(
        &fs::read(repository().join("examples/references/triple_box_offshell_rank2.json")).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        reference.validation,
        ReferenceValidation::Checked { .. }
    ));
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        vec![-3, -2, -1, 0]
    );
    // Independent provider IEEE rows bind the physical tuple after the native
    // tensor/Gamma prefactor. Do not confuse them with unmultiplied tuple0.
    let mean_bits = [
        13_809_537_659_953_531_147,
        4_599_875_189_574_476_642,
        13_824_758_583_365_437_386,
        4_603_817_135_207_285_032,
    ];
    let error_bits = [
        4_492_763_168_831_924_300,
        4_561_873_223_135_966_907,
        4_571_283_621_660_478_155,
        4_577_172_024_838_135_089,
    ];
    for (i, coefficient) in reference.coefficients.iter().enumerate() {
        assert_eq!(coefficient.key.component, CoefficientComponent::Real);
        assert_eq!(coefficient.value.to_bits(), mean_bits[i]);
        let ReferenceUncertainty::StandardError(error) = coefficient.uncertainty else {
            panic!("reported uncertainty must stay statistical")
        };
        assert_eq!(error.to_bits(), error_bits[i]);
        assert!(error > 0.0 && error.is_finite());
    }
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["loops"], 3);
    assert_eq!(attributes["original_propagators"], 10);
    assert_eq!(attributes["active_propagators"], 8);
    assert_eq!(
        attributes["original_order_projected_powers"],
        json!([1, 1, 0, 2, 0, 2, 1, 1, 1, 1])
    );
    assert_eq!(attributes["active_powers"], json!([1, 1, 2, 2, 1, 1, 1, 1]));
    assert_eq!(attributes["constituent_prefactor"], "Gamma(3+3*eps)");
    assert_eq!(attributes["additional_prefactor"], "1");
    assert_eq!(
        attributes["numerator"],
        "k1(mu)*k3(mu)+2*k2(nu)*p1(nu)*k2(rho)*p2(rho)"
    );
    assert_eq!(attributes["physical_tuple_index"], 2);
    assert_eq!(attributes["geometric_sectors"], 1182);
    assert_eq!(attributes["together"], true);
    assert_eq!(attributes["seed"], 20261202);
    assert!(attributes["sector_filter"].is_null());
    assert_eq!(
        attributes["kinematics"]["external_virtualities"],
        json!(["-1", "-1", "-1", "-1"])
    );
    assert_eq!(attributes["kinematics"]["s12"], "-2");
    assert_eq!(attributes["kinematics"]["s23"], "-2");
    assert_eq!(
        attributes["external_imaginary_values"],
        json!([0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(
        attributes["external_imaginary_standard_errors"],
        json!([0.0, 0.0, 0.0, 0.0])
    );
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["actual_evaluations"], 1_063_808);
    assert_eq!(attributes["scalar_summed_coefficient_calls"], 4);
    assert!(
        attributes["actual_evaluation_unit"]
            .as_str()
            .unwrap()
            .contains("scalar summed-sector coefficient")
    );
    assert_eq!(attributes["calibration_certified"], false);
    assert_eq!(attributes["exactness"], false);
    assert_eq!(attributes["highest_order_target"], 0);
    assert_eq!(attributes["highest_order_target_met"], false);
    assert_ne!(
        attributes["native_kernel_content_ids"]["original"],
        attributes["native_kernel_content_ids"]["projected"]
    );
    for source in attributes["native_sources"].as_array().unwrap() {
        assert_eq!(
            blake3::hash(&fs::read(repository().join(source["path"].as_str().unwrap())).unwrap())
                .to_hex()
                .as_str(),
            source["blake3"].as_str().unwrap()
        );
    }
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}

#[test]
#[ignore = "requires the audited frozen inputs and six ignored external reports; records candidates under output/reference-fixtures only"]
fn record_six_external_reference_candidates() {
    let repository = repository();
    let output = repository.join("output/reference-fixtures");
    fs::create_dir_all(&output).unwrap();
    for (name, loops, propagators) in support::CASES {
        let suffix = if name == "self_energy_3loop" {
            ".serial"
        } else {
            ""
        };
        let raw_path = format!("output/diagnostics/multiloop-pysecdec/{name}{suffix}.raw.json");
        let raw_bytes = fs::read(repository.join(&raw_path)).unwrap();
        let audited = support::load_reference(repository, name);
        assert_eq!(
            blake3::hash(&raw_bytes).to_hex().as_str(),
            audited.provenance.attributes["raw_report_blake3"]
                .as_str()
                .unwrap(),
            "{name}: a changed raw report requires a new independent review"
        );
        let raw: Value = serde_json::from_slice(&raw_bytes).unwrap();
        assert_eq!(raw["schema_version"], 1);
        assert_eq!(raw["request"]["dot_engine"], "pysecdec");
        assert_eq!(raw["mode"], "massive");
        assert_eq!(raw["request"]["m"], 1.0);
        assert_eq!(raw["prefactor_convention"], "pysecdec");
        assert_eq!(raw["request"]["pysecdec_maxeval"], 4096);
        assert_eq!(raw["request"]["pysecdec_epsrel"], 0.05);
        assert_eq!(raw["pysecdec"]["orders"], json!([0]));
        assert_eq!(raw["pysecdec"]["coeffs"].as_array().unwrap().len(), 1);
        assert_eq!(raw["pysecdec"]["errors"].as_array().unwrap().len(), 1);
        let value = raw["pysecdec"]["coeffs"][0]["re"].as_f64().unwrap();
        let error = raw["pysecdec"]["errors"][0]["re"].as_f64().unwrap();
        let imaginary = raw["pysecdec"]["coeffs"][0]["im"].as_f64().unwrap();
        let imaginary_error = raw["pysecdec"]["errors"][0]["im"].as_f64().unwrap();
        assert!(value.is_finite() && error.is_finite() && error > 0.0);
        assert!(imaginary.is_finite() && imaginary_error.is_finite());
        assert_eq!((imaginary, imaginary_error), (0.0, 0.0));

        let mut provenance = ReferenceProvenance::new(
            "Independent pySecDec C++ package through the existing reference CLI",
            "Per-loop d^Dk/(i*pi^(D/2)); D=4-2*eps; unit powers; multiplier 1; finite eps^0",
        );
        provenance.revision = Some("582d8c7f6dde9bf750750d4c2a2d85a94ce940cd".into());
        provenance.location = Some(raw_path);
        provenance.recorded_utc = Some(raw["created_utc"].as_str().unwrap().into());
        provenance.engine = Some("pySecDec 1.6.6; generated C++ compiled with GCC 15.3.0".into());
        provenance.notes = vec![
            "Independent reviewers matched graph incidence, external attachments, masses and virtualities, and checked native/pySecDec normalization. See docs/reviews/massive-multiloop-diagnostics.md.".into(),
            "The real-only projection is explicit: the external imaginary value and its reported error are both zero and remain recorded below. No absent component or uncertainty was invented.".into(),
            "The reported error is retained as a standard error. Initial independent correctness evidence is not convergence or matched-performance certification.".into(),
            "maxeval=4096 is requested steering, not measured work. The bridge does not report actual counts/lattice sizes; its QMC defaults are minn=10000 and 32 minimum shifts, so work may exceed that request.".into(),
            "QMC seed zero preserves the native random_device seed; request.seed=1 is not forwarded by this reference bridge. Native campaigns use independent fixed seeds.".into(),
        ];
        let native_paths = [
            format!("examples/runs/{name}.toml"),
            format!("examples/graphs/{name}.dot"),
            "examples/models/scalar.json".into(),
            "examples/models/massive.json".into(),
        ];
        let native_sources = native_paths
            .into_iter()
            .map(|path| {
                let bytes = fs::read(repository.join(&path)).unwrap();
                json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string()})
            })
            .collect::<Vec<_>>();
        let external_sources = ["dot_file", "kinematics_file"].map(|field| {
            let path = raw["request"][field].as_str().unwrap();
            let bytes = fs::read(
                repository
                    .join("DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder")
                    .join(path),
            )
            .unwrap();
            json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string()})
        });
        let attributes = json!({
            "case":name,"loops":loops,"propagators":propagators,
            "raw_report_blake3":blake3::hash(&raw_bytes).to_hex().to_string(),
            "command":raw["command"],"environment":raw["environment"],
            "reference_symbolica_version":"2.1.0","reference_pysecdec_version":"1.6.6",
            "masses":1,"external_virtualities":-1,
            "external_imaginary_value":imaginary,"external_imaginary_standard_error":imaginary_error,
            "requested_maxeval":raw["request"]["pysecdec_maxeval"],
            "requested_epsrel":raw["request"]["pysecdec_epsrel"],
            "actual_evaluations":null,"actual_lattice_size":null,
            "native_sources":native_sources,"external_sources":external_sources,
            "cpu_affinity":if name=="kite_2loop" {vec![0,1]} else {vec![0]},
            "reference_domain":"Euclidean; no contour deformation",
            "qmc_defaults":{"minn":10000,"minimum_shifts":32,"periodization":"Korobov3","generating_vectors":{"selection":"dimension-dependent merged CBC/PT tables","tables":["cbcpt_dn2_6","cbcpt_cfftw1_6","cbcpt_cfftw2_10","cbcpt_dn1_100"],"selected_rule":null},"lattice_candidates":0,"fitfunction":"default","seed":"native random_device"}
        });
        provenance.attributes = serde_json::from_value(attributes).unwrap();
        let mut reference = ReferenceResult::new(
            vec![ReferenceCoefficient {
                key: CoefficientKey {
                    order: 0,
                    component: CoefficientComponent::Real,
                },
                value,
                uncertainty: ReferenceUncertainty::StandardError(error),
            }],
            provenance,
        );
        reference.validation = ReferenceValidation::Checked {
            evidence:"Independent source/graph/normalization/projection audit and bounded complete-vector native comparison; source errors retained. This label records implementation and input evidence, not convergence certification.".into(),
        };
        let (content_id, estimate) = saved_estimate(name);
        let comparison = compare(
            &estimate,
            &reference,
            &support::comparison_context(name, &content_id),
        )
        .unwrap();
        assert!(comparison.eligibility.eligible, "{name}: {comparison:?}");
        assert_eq!(comparison.rows.len(), 1);
        assert!(
            matches!(comparison.rows[0].pull, Pull::Value(value) if value.abs() < 5.0),
            "{name}: {comparison:?}"
        );
        let encoded = encode_reference(&reference).unwrap();
        assert_eq!(read_reference(&encoded).unwrap(), reference);
        let pretty: Value = serde_json::from_slice(&encoded).unwrap();
        fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&pretty).unwrap(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires the saved complete native diagnostic reports; no new sampling is performed"]
fn saved_native_vectors_compare_with_all_six_independent_references() {
    for (name, _, _) in support::CASES {
        let (content_id, estimate) = saved_estimate(name);
        let comparison = support::compare_reference(repository(), name, &content_id, &estimate);
        assert!(comparison.eligibility.eligible, "{name}: {comparison:?}");
        assert_eq!(comparison.rows.len(), 1);
        assert!(
            matches!(comparison.rows[0].pull,Pull::Value(value) if value.abs()<5.0),
            "{name}: {comparison:?}"
        );
        println!("{name}: {comparison}");
    }
}

#[test]
fn hard_orthant_reference_preserves_reported_zero_and_missing_native_order() {
    use fastsecdec::reference::{
        ComparisonContext, Compatibility, Independence, IneligibilityReason, UnavailablePull,
    };
    let reference = support::load_reference(repository(), "four_loop_hard");
    assert!(matches!(
        reference.validation,
        ReferenceValidation::Checked { .. }
    ));
    assert_eq!(
        reference
            .coefficients
            .iter()
            .map(|c| c.key.order)
            .collect::<Vec<_>>(),
        [-3, -2, -1, 0]
    );
    let value_bits = [
        0,
        13_838_658_250_071_177_118,
        13_848_756_176_037_023_360,
        13_862_885_006_821_585_130,
    ];
    let error_bits = [
        0,
        4_576_763_468_005_019_371,
        4_591_079_869_049_241_685,
        4_605_908_752_028_019_518,
    ];
    for (i, row) in reference.coefficients.iter().enumerate() {
        assert_eq!(row.key.component, CoefficientComponent::Real);
        assert_eq!(row.value.to_bits(), value_bits[i]);
        let ReferenceUncertainty::StandardError(error) = row.uncertainty else {
            panic!("reported zero error must remain statistical, not Exact");
        };
        assert_eq!(error.to_bits(), error_bits[i]);
    }
    let attributes = &reference.provenance.attributes;
    assert_eq!(attributes["reference_domain"], "positive_orthant");
    assert_eq!(attributes["density"], "U^1*F^(eps-3)");
    assert_eq!(attributes["dimension"], 9);
    assert_eq!(attributes["measure"], "prod(dx)");
    assert_eq!(attributes["constituent_prefactor"], "1");
    assert_eq!(attributes["physical_tuple_index"], 2);
    assert_eq!(attributes["geometric_sectors"], 2676);
    assert_eq!(attributes["together"], true);
    assert!(attributes["sector_filter"].is_null());
    assert!(attributes["external_imaginary_values"].is_null());
    assert!(attributes["external_imaginary_standard_errors"].is_null());
    assert!(attributes["external_covariance"].is_null());
    assert_eq!(attributes["actual_evaluations"], 1_063_808);
    assert_eq!(attributes["scalar_summed_coefficient_calls"], 4);
    assert_eq!(attributes["calibration_certified"], false);
    assert_eq!(attributes["highest_order_target"], 0);
    assert_eq!(attributes["highest_order_target_met"], false);
    assert_eq!(attributes["native_estimate_padded"], false);
    let certificate = &attributes["separate_lower_order_zero_certificate"];
    assert_eq!(certificate["complete"], true);
    assert_eq!(certificate["zero_through_order"], -3);
    assert_eq!(certificate["expected_charts"], 2760);
    assert_eq!(certificate["completed_native_laurent_extractions"], 699);
    assert_eq!(certificate["exact_coefficients"], json!(["0"]));
    assert_eq!(certificate["numerical_kernels"], 0);
    assert_eq!(certificate["retained_estimate_modified"], false);
    // The complete, unchanged historical observation has only three orders.
    // The independent symbolic certificate must not silently pad its covariance.
    let estimate = VectorEstimate {
        orders: vec![-2, -1, 0],
        components: vec![CoefficientComponent::Real; 3],
        mean: vec![-4.067405136814634, -18.169409299577108, -162.1952374736587],
        standard_error: vec![0.28758494646617144, 1.0692199476005346, 8.959016118757578],
        covariance_of_mean: vec![
            0.0827051014339507,
            0.1844876701397855,
            0.6094001846855066,
            0.1844876701397855,
            1.1432312963468898,
            7.339009447423726,
            0.6094001846855066,
            7.339009447423726,
            80.26396981615811,
        ],
        production_complete: true,
    };
    let original = estimate.clone();
    let comparison = compare(&estimate,&reference,&ComparisonContext {
        kernel_content_id: attributes["native_kernel_content_id"].as_str().unwrap().into(),
        normalization: Compatibility::Confirmed {basis:"Audited identical explicit U/F density and full positive-orthant unit measure; ordinary physical tuple2, no graph Gamma.".into()},
        kinematics: Compatibility::Confirmed {basis:"Frozen native input hashes verified above, same complete nine-variable polynomial density.".into()},
        independence: Independence::Independent {basis:"External seed20261219 and native seed20261004; separate implementations/observations, no reused samples.".into()},
    }).unwrap();
    assert_eq!(estimate, original);
    assert!(!comparison.eligibility.eligible);
    assert_eq!(
        comparison.eligibility.reasons,
        vec![IneligibilityReason::MissingEstimate]
    );
    assert_eq!(comparison.rows.len(), 4);
    assert_eq!(comparison.rows[0].key.order, -3);
    assert!(comparison.rows[0].estimate.is_none());
    assert!(matches!(
        comparison.rows[0].pull,
        Pull::Unavailable(UnavailablePull::MissingEstimate)
    ));
    assert!(
        comparison.rows[1..]
            .iter()
            .all(|row| matches!(row.pull,Pull::Value(value) if value.abs()<5.0))
    );
    assert_eq!(
        read_reference(&encode_reference(&reference).unwrap()).unwrap(),
        reference
    );
}
