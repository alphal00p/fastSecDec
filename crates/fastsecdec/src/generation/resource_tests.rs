use super::*;
use crate::Error as NativeError;
use crate::kernel::KernelError;
use crate::parametric::{ParametricDomain, ParametricTerm};
use fastsecdec_sectors::SectorError;
use std::error::Error as StdError;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use symbolica::utils::resources::{ResourceLimitError, ResourceLimits};
use symbolica::{atom::Atom, symbol};

fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("generation_resources::x")],
        symbol!("generation_resources::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::one(), vec![Atom::zero()], vec![])],
    )
    .unwrap()
}

#[test]
fn completed_generation_is_not_returned_after_resource_exhaustion() {
    let rss = Arc::new(AtomicU64::new(0));
    let measured = Arc::clone(&rss);
    let budget = ResourceBudget::with_memory_monitor(
        ResourceLimits {
            max_process_rss_bytes: Some(10),
            ..Default::default()
        },
        move || Ok(measured.load(Ordering::Relaxed)),
    );
    let input = input();
    let result = budget.run(|| {
        generate(&input, &GenerationOptions::default(), |status| {
            if matches!(status, GenerationProgress::Complete { .. }) {
                rss.store(11, Ordering::Relaxed);
            }
            ControlFlow::Continue(())
        })
    });
    assert!(matches!(
        result,
        Err(GenerationError::Resource(ResourceLimitError::Memory { .. }))
    ));
    rss.store(0, Ordering::Relaxed);
    assert!(
        generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .is_ok()
    );
}

#[test]
fn retained_generation_keeps_the_original_active_budget() {
    let rss = Arc::new(AtomicU64::new(0));
    let measured = Arc::clone(&rss);
    let budget = ResourceBudget::with_memory_monitor(
        ResourceLimits {
            max_process_rss_bytes: Some(10),
            ..Default::default()
        },
        move || Ok(measured.load(Ordering::Relaxed)),
    );
    let mut session = budget.run(|| GenerationSession::new(input(), GenerationOptions::default()));
    rss.store(11, Ordering::Relaxed);
    assert!(matches!(
        session.step(1, |_| ControlFlow::Continue(())),
        Err(GenerationError::Resource(ResourceLimitError::Memory { .. }))
    ));
    assert!(session.take_result().is_none());
}

#[test]
fn resource_leaf_is_visible_through_all_public_native_error_sources() {
    let cause = ResourceLimitError::Memory {
        rss_bytes: 11,
        additional_bytes: 3,
        limit_bytes: 10,
    };
    let errors: Vec<Box<dyn StdError>> = vec![
        Box::new(NativeError::Resource(cause.clone())),
        Box::new(GenerationError::Resource(cause.clone())),
        Box::new(KernelError::Resource(cause.clone())),
        Box::new(SectorError::Resource(cause.clone())),
    ];
    for error in errors {
        let source = error
            .source()
            .expect("resource wrapper retains its native cause");
        assert_eq!(source.downcast_ref::<ResourceLimitError>(), Some(&cause));
        assert_eq!(error.to_string(), cause.to_string());
    }
}
