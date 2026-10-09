//! Immutable numerical records and explicit, case-specific provenance maps.
use super::Result;
use serde::{Deserialize, Serialize};
use symbolica::atom::Atom;

pub const ARCHIVE_URL: &str = "https://arxiv.org/src/1912.09291v2";
pub const ARCHIVE_SHA256: &str = "23add23135a2431284d5db0af4083a4198f22615cd2e5a554abba9599dcd10cf";
pub const RAR_SHA256: &str = "97efb556021a988b999e60c86854f6591942cfe3d674a0b9b228b4cdfedd700d";
pub const YAML_SHA256: &str = "6ad6216da1051a0ff860f80f14cf9b4534cb85aa0f38d48bac06a0e968dd0b68";
pub const MODEL: &str = include_str!("../../../../examples/models/scalar.json");

#[derive(Debug, Deserialize)]
pub struct Record {
    pub name: String,
    pub n_loops: usize,
    pub external_kinematics: Vec<[f64; 4]>,
    pub loop_lines: Vec<LoopLine>,
    pub analytical_result_real: f64,
    pub analytical_result_imag: f64,
    pub raw_yaml_sha256: String,
}
#[derive(Debug, Deserialize)]
pub struct LoopLine {
    pub signature: Vec<i32>,
    pub propagators: Vec<Propagator>,
}
#[derive(Debug, Deserialize)]
pub struct Propagator {
    pub m_squared: f64,
    pub q: [f64; 4],
}
#[derive(Clone, Debug, Serialize)]
pub struct Reference {
    pub method: &'static str,
    pub real: f64,
    pub imaginary: f64,
    pub absolute_errors: [Option<f64>; 2],
    pub uncertainty_note: &'static str,
}
pub fn records() -> Result<Vec<Record>> {
    Ok(serde_json::from_str(include_str!(
        "../../../../examples/contour/ltd/ancillary-records.json"
    ))?)
}
impl Record {
    pub fn slug(&self) -> &'static str {
        match self.name.as_str() {
            "2L4P.b.K1" => "2l4p_k1",
            "2L4P.b.K1*" => "2l4p_k1_massive",
            "2L6P.a.I" => "2l6p_a_i",
            "3L4P.K1" => "3l4p_k1",
            _ => unreachable!("only explicitly admitted records are bundled"),
        }
    }
    pub fn dot(&self) -> &'static str {
        match self.name.as_str() {
            "2L4P.b.K1" | "2L4P.b.K1*" => {
                include_str!("../../../../examples/contour/ltd/topologies/double_ladder.dot")
            }
            "2L6P.a.I" => include_str!("../../../../examples/contour/ltd/topologies/six_point.dot"),
            "3L4P.K1" => {
                include_str!("../../../../examples/contour/ltd/topologies/triple_ladder.dot")
            }
            _ => unreachable!("only explicitly admitted records are bundled"),
        }
    }
    pub fn mass(&self) -> Atom {
        if self.name == "2L4P.b.K1*" {
            Atom::num((2, 5))
        } else {
            Atom::Zero
        }
    }
    /// Archive q shifts written in its original external-vector order. These
    /// finite fixture tables are provenance, not a momentum-routing algorithm.
    pub fn shifts(&self) -> Vec<Vec<i32>> {
        match self.name.as_str() {
            "2L4P.b.K1" | "2L4P.b.K1*" => vec![
                vec![0, 0, 0, 0],
                vec![-1, -1, 0, 0],
                vec![0, -1, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 1, 0],
                vec![-1, -1, 0, 0],
            ],
            "3L4P.K1" => vec![
                vec![0, 0, 0, 0],
                vec![-1, -1, 0, 0],
                vec![0, -1, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![-1, -1, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 1, 0],
                vec![-1, -1, 0, 0],
            ],
            "2L6P.a.I" => vec![
                vec![0, 0, 0, 0, 0, 0],
                vec![-1, 0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0, 0],
                vec![0, 1, 0, 0, 0, 0],
                vec![0, 1, 1, 0, 0, 0],
                vec![0, 1, 1, 1, 0, 0],
                vec![0, 1, 1, 1, 1, 0],
                vec![-1, 0, 0, 0, 0, 0],
            ],
            _ => unreachable!("only explicitly admitted records are bundled"),
        }
    }
    pub fn references(&self) -> Vec<Reference> {
        match self.name.as_str() {
            "2L4P.b.K1" | "3L4P.K1" => vec![Reference {
                method: "ancillary analytic ladder value",
                real: self.analytical_result_real,
                imaginary: self.analytical_result_imag,
                absolute_errors: [None, None],
                uncertainty_note: "Rounded stored analytic value; no numerical uncertainty was supplied.",
            }],
            "2L4P.b.K1*" => vec![Reference {
                method: "Table 6 pySecDec",
                real: 2.8020e-6,
                imaginary: 3.3450e-6,
                absolute_errors: [Some(0.0080e-6), Some(0.0080e-6)],
                uncertainty_note: "Published numerical uncertainties.",
            }],
            "2L6P.a.I" => vec![
                Reference {
                    method: "Table 2 pySecDec",
                    real: -86.080,
                    imaginary: 0.,
                    absolute_errors: [Some(0.090), None],
                    uncertainty_note: "The table reports a real value and its uncertainty.",
                },
                Reference {
                    method: "Table 2 independent momentum-space result",
                    real: -86.600,
                    imaginary: 0.,
                    absolute_errors: [Some(0.800), None],
                    uncertainty_note: "Retained separately; no tuning toward either central value.",
                },
            ],
            _ => unreachable!("only explicitly admitted records are bundled"),
        }
    }
}
