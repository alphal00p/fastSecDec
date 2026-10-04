//! Caller-steered native integration shared by the coupled-sunset checks.
use std::{ops::ControlFlow, time::Instant};

use fastsecdec::{
    generation::{GenerationOptions, generate},
    integration::{
        IntegrationProblem, PublishedLattice, QmcSession, QmcSettings, RuleSource, SectorSpec,
        VectorEstimate,
    },
    kernel::ReplayPolicy,
    parametric::ParametricIntegrand,
    status::CoefficientComponent,
};

pub fn integrate(
    input: &ParametricIntegrand,
    max_order: i32,
    seed: u64,
    points: u64,
) -> VectorEstimate {
    let started = Instant::now();
    let generated = generate(
        input,
        &GenerationOptions {
            max_order,
            ..Default::default()
        },
        |_| {
            if started.elapsed().as_secs() >= 180 {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    )
    .unwrap();
    let kernels = generated.compile().unwrap();
    let problem = IntegrationProblem::new_with_components(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(id, kernel)| SectorSpec {
                id: id as u64,
                dimension: kernel.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )
    .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points,
            shifts: 16,
            seed,
            rule: RuleSource::Published(PublishedLattice::Kuo38005),
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|id| session.worker_context(id as u64).unwrap())
        .collect::<Vec<_>>();
    let mut contexts = (0..kernels.sectors().len())
        .map(|id| {
            kernels
                .evaluation_context(id, ReplayPolicy::default())
                .unwrap()
        })
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        assert!(
            started.elapsed().as_secs() < 180,
            "sunset diagnostic exceeded its work budget"
        );
        let id = task.sector_id() as usize;
        let result = workers[id]
            .evaluate_weighted(task, |point, weight, output| {
                contexts[id]
                    .evaluate_weighted(point, weight, output)
                    .map(|_| ())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert!(estimate.production_complete);
    assert!(
        estimate
            .components
            .iter()
            .all(|component| *component == CoefficientComponent::Real)
    );
    println!(
        "sunset complete estimate: {}",
        serde_json::to_string(&estimate).unwrap()
    );
    println!(
        "sunset generation/O2/weighted QMC total: {:.3}s",
        started.elapsed().as_secs_f64()
    );
    estimate
}

pub fn compare(order: i32, value: f64, error: f64, expected: f64) {
    println!("eps^{order}: {value:.14e} +/- {error:.3e}; analytic {expected:.14e}");
    assert!(
        error < 2e-5 * expected.abs().max(1.0),
        "inconclusive sunset precision"
    );
    assert!((value - expected).abs() < 8.0 * error + 2e-9 * expected.abs().max(1.0));
}
