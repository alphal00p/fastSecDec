use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelLoadProgress},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

fn template() -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("load_progress::x"), symbol!("load_progress::y")],
        symbol!("load_progress::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("load_progress::x+load_progress::y"),
                parse!("-1-load_progress::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn observed_restore_preserves_native_identity_and_reports_ordered_units() {
    let template = template();
    let mut events = Vec::new();
    let mut loaded =
        KernelSet::from_bytes_with_progress(template.artifact_bytes().unwrap(), |event| {
            events.push(*event);
            ControlFlow::Continue(())
        })
        .unwrap();
    assert!(matches!(events.first(), Some(KernelLoadProgress::Decoding)));
    assert!(matches!(events.last(), Some(KernelLoadProgress::Complete)));
    let steps = events
        .iter()
        .filter_map(|event| match event {
            KernelLoadProgress::Restoring(step) => Some(step),
            _ => None,
        })
        .collect::<Vec<_>>();
    let total = template.sectors().len();
    assert!(total > 0);
    assert_eq!(steps.len(), total + 1);
    for (index, step) in steps.iter().enumerate() {
        assert_eq!((step.completed, step.total), (index, total));
        assert!(step.elapsed_seconds.is_finite() && step.elapsed_seconds >= 0.0);
    }
    assert!(
        steps
            .windows(2)
            .all(|pair| pair[0].elapsed_seconds <= pair[1].elapsed_seconds)
    );
    assert_eq!(loaded.content_id(), template.content_id());
    assert_eq!(
        loaded.artifact_bytes().unwrap(),
        template.artifact_bytes().unwrap()
    );
    let mut original = template.try_clone().unwrap();
    for (a, b) in loaded.sectors_mut().iter_mut().zip(original.sectors_mut()) {
        let mut left = vec![0.0; a.output_count()];
        let mut right = left.clone();
        let point = vec![0.37; a.dimension()];
        a.evaluate(&point, &mut left).unwrap();
        b.evaluate(&point, &mut right).unwrap();
        assert_eq!(left, right);
    }
}

#[test]
fn observed_restore_cancels_at_boundaries_without_partial_success() {
    let template = template();
    let bytes = template.artifact_bytes().unwrap();
    for stop in 0..=template.sectors().len() + 2 {
        let mut count = 0;
        let result = KernelSet::from_bytes_with_progress(bytes, |_| {
            let cancel = count == stop;
            count += 1;
            if cancel {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
        assert!(
            matches!(result, Err(KernelError::Cancelled)),
            "boundary {stop}"
        );
        assert_eq!(count, stop + 1);
    }
    let mut events = Vec::new();
    assert!(
        KernelSet::from_bytes_with_progress(b"invalid", |event| {
            events.push(*event);
            ControlFlow::Continue(())
        })
        .is_err()
    );
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], KernelLoadProgress::Decoding));
}
