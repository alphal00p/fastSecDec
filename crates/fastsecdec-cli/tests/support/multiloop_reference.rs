//! Native transport and comparison only; no reference evaluator or integrator.
use std::{fs, path::Path};

use fastsecdec::{
    integration::VectorEstimate,
    reference::{
        ComparisonContext, Compatibility, Independence, ReferenceComparison, ReferenceResult,
        compare, read_reference,
    },
};

pub const CASES: [(&str, usize, usize); 6] = [
    ("kite_2loop", 2, 5),
    ("self_energy_3loop", 3, 7),
    ("three_point_2loop", 2, 5),
    ("three_point_2loop_6line", 2, 6),
    ("three_point_3loop", 3, 7),
    ("three_point_3loop_8line", 3, 8),
];

pub fn load_reference(repository: &Path, name: &str) -> ReferenceResult {
    let path = repository.join(format!("examples/references/{name}.json"));
    let reference = read_reference(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(reference.provenance.attributes["case"], name);
    // Evidence is tied to the current scientific inputs, not only their labels.
    for source in reference.provenance.attributes["native_sources"]
        .as_array()
        .unwrap()
    {
        let path = source["path"].as_str().unwrap();
        let bytes = fs::read(repository.join(path)).unwrap();
        assert_eq!(
            blake3::hash(&bytes).to_hex().as_str(),
            source["blake3"].as_str().unwrap(),
            "reference input evidence changed: {path}"
        );
    }
    reference
}

pub fn compare_reference(
    repository: &Path,
    name: &str,
    content_id: &str,
    estimate: &VectorEstimate,
) -> ReferenceComparison {
    let reference = load_reference(repository, name);
    compare(estimate, &reference, &comparison_context(name, content_id)).unwrap()
}

pub fn comparison_context(name: &str, content_id: &str) -> ComparisonContext {
    ComparisonContext {
            kernel_content_id: content_id.into(),
            normalization: Compatibility::Confirmed {
                basis: "Both use d^Dk/(i*pi^(D/2)) per loop, unit propagator powers, explicit multiplier 1 and the same (-1)^N Gamma(N-LD/2) factor; only the finite eps^0 coefficient is present.".into(),
            },
            kinematics: Compatibility::Confirmed {
                basis: format!("{name}: independently matched native graph incidence/external attachments, masses 1 and external virtualities -1; current native card/model/graph/card bytes match the recorded evidence."),
            },
            independence: Independence::Independent {
                basis: "The external pySecDec C++ package uses independently generated sectors and hardware-seeded random shifts; its existing bridge does not forward request.seed. Native QMC uses a separate fixed seed and no shared samples.".into(),
            },
    }
}
