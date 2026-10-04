//! Support-only benchmark fixture extracted from Pathfinder's hard four-loop
//! input. Coefficients stay with the main integral fixture; no CAS parser here.
use fastsecdec_sectors::{DecompositionOptions, ParametricDomain, PolynomialSupport, decompose};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::ControlFlow,
    time::Instant,
};

fn support(text: &str) -> PolynomialSupport {
    PolynomialSupport::new(
        text.lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|v| v.parse().unwrap())
                    .collect()
            })
            .collect(),
    )
    .unwrap()
}

#[test]
#[ignore = "nine-dimensional exact geometry performance probe"]
fn hard_four_loop_fan() {
    let supports = [
        support(include_str!("fixtures/hard_u.support")),
        support(include_str!("fixtures/hard_f.support")),
    ];
    assert_eq!(supports[0].exponents().len(), 70);
    assert_eq!(supports[1].exponents().len(), 105);
    probe(&supports, 266);
}

#[test]
#[ignore = "nine-dimensional exact geometry performance probe"]
fn hard_four_loop_singular_f_only() {
    // U has fixed exponent +1 in the physical fixture: it is a polynomial
    // numerator, whereas F^(eps-3) determines singularities. Keep the U/F
    // probe above as a separate stress test of simultaneous support geometry.
    let support = support(include_str!("fixtures/hard_f.support"));
    assert_eq!(support.exponents().len(), 105);
    probe(&[support], 105);
}

fn probe(supports: &[PolynomialSupport], candidates: usize) {
    let start = Instant::now();
    let mut last = Instant::now();
    let mut previous_phase = None;
    let mut phase_start = Instant::now();
    let decomposition = decompose(
        ParametricDomain::PositiveOrthant,
        supports,
        &DecompositionOptions::default(),
        |status| {
            if previous_phase != Some(status.phase) {
                if let Some(phase) = previous_phase {
                    eprintln!(
                        "phase {phase:?}: {:.6}s",
                        phase_start.elapsed().as_secs_f64()
                    );
                }
                previous_phase = Some(status.phase);
                phase_start = Instant::now();
            }
            if last.elapsed().as_secs() >= 5 {
                eprintln!(
                    "{:?}: constraints {}/{}, rays {}, sectors {}, {:.1}s",
                    status.phase,
                    status.completed_constraints,
                    status.total_constraints,
                    status.rays,
                    status.sectors,
                    start.elapsed().as_secs_f64()
                );
                last = Instant::now();
            }
            ControlFlow::Continue(())
        },
    )
    .unwrap();
    assert_eq!(decomposition.candidate_vertices, candidates);
    assert!(!decomposition.sectors.is_empty());
    eprintln!(
        "hard fan: {} vertices, {} sectors, {:.3}s",
        decomposition.geometric_vertices,
        decomposition.sectors.len(),
        start.elapsed().as_secs_f64()
    );
    // A same-toolchain fingerprint checks every ordered exact map, Jacobian,
    // determinant and valuation during performance experiments. It is not a
    // portable artifact format or a substitute for the independent moment tests.
    let mut fingerprint = DefaultHasher::new();
    format!("{:?}", decomposition.sectors).hash(&mut fingerprint);
    eprintln!("ordered exact maps: {:016x}", fingerprint.finish());
}
