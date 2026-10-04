//! Support-only benchmark fixture extracted from Pathfinder's hard four-loop
//! input. Coefficients stay with the main integral fixture; no CAS parser here.
use fastsecdec_sectors::{DecompositionOptions, ParametricDomain, PolynomialSupport, decompose};
use std::{ops::ControlFlow, time::Instant};

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
    let start = Instant::now();
    let mut last = Instant::now();
    let decomposition = decompose(
        ParametricDomain::PositiveOrthant,
        &supports,
        &DecompositionOptions::default(),
        |status| {
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
    assert_eq!(decomposition.candidate_vertices, 266);
    assert!(!decomposition.sectors.is_empty());
    eprintln!(
        "hard fan: {} vertices, {} sectors, {:.3}s",
        decomposition.geometric_vertices,
        decomposition.sectors.len(),
        start.elapsed().as_secs_f64()
    );
}
