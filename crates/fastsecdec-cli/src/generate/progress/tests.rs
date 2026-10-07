use super::*;
use fastsecdec::{
    generation::{CoefficientExpansionStage, CoefficientRequestCounts},
    status::GenerationStage,
};

fn snapshot() -> GenerationSnapshot {
    GenerationSnapshot {
        stage: GenerationStage::Input,
        completed: 0,
        total: None,
        sectors: 0,
        kernels: 0,
        elapsed_seconds: 0.0,
        timings: Default::default(),
        coefficient_expansion: None,
        formula_preparation: None,
        detail: String::new(),
    }
}

fn event(
    sector: usize,
    stage: CoefficientExpansionStage,
    attempt: usize,
    requests: usize,
) -> GenerationProgress {
    GenerationProgress::CoefficientExpansion {
        sector,
        total: 2,
        stage,
        attempt,
        relative_width: if attempt == 0 { 0 } else { 3 },
        formal_pieces: 0,
        requests: CoefficientRequestCounts {
            unique_requests: requests,
            ..Default::default()
        },
    }
}

#[test]
fn suppressed_presentation_still_cancels_without_manufacturing_completion() {
    let mut dashboard = Dashboard::with_status_interval(false, true, 60_000).unwrap();
    let mut status = snapshot();
    let mut error = None;
    status.observe_generation(0, &event(0, CoefficientExpansionStage::Lowering, 1, 1));
    assert_eq!(
        publish_generation(&mut dashboard, &status, &mut error),
        ControlFlow::Continue(())
    );
    dashboard.request_cancel();
    assert_eq!(
        publish_generation(&mut dashboard, &status, &mut error),
        ControlFlow::Break(())
    );
    assert!(error.is_none());
    assert_eq!(status.completed, 0);
    assert_ne!(status.stage, GenerationStage::Complete);
}

#[test]
fn running_formula_index_does_not_claim_global_completions() {
    let mut status = snapshot();
    status.observe_generation(
        0,
        &GenerationProgress::FormulaPreparation {
            completed: 3,
            total: 4,
            sectors: 30,
            reused: 26,
        },
    );
    // Formula 3 can run first; the worker-local native poll is not proof that
    // three other formulas have finished. Aggregate admission stays separate.
    let activity = worker_activity(
        SymbolicJobId {
            stage: SymbolicStage::FormulaPreparation,
            index: 3,
        },
        &status.detail,
    );
    assert_eq!(activity, "Building subtraction formula 3");
    assert!(!activity.contains("ready"));
}
