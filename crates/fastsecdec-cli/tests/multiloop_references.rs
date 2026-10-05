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
