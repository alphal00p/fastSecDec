use super::*;
use crate::generation::CoefficientRequestCounts;

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
fn attempt_reset_and_fallback_preserve_completed_sector_semantics() {
    let mut status = snapshot();
    status.observe_generation(1, &event(0, CoefficientExpansionStage::Lowering, 1, 7));
    assert_eq!(status.completed, 0);
    assert!(status.detail.contains("epsilon expansion pass 1"));
    assert_eq!(status.coefficient_expansion.as_ref().unwrap().attempt, 1);
    status.observe_generation(1, &event(0, CoefficientExpansionStage::RegularSeries, 2, 0));
    assert_eq!(status.completed, 0);
    assert_eq!(
        status
            .coefficient_expansion
            .as_ref()
            .unwrap()
            .requests
            .unique_requests,
        0
    );
    status.observe_generation(1, &event(0, CoefficientExpansionStage::Complete, 2, 5));
    assert_eq!(status.completed, 1);
    status.observe_generation(
        1,
        &event(1, CoefficientExpansionStage::PhysicalFallback, 0, 0),
    );
    assert_eq!(status.completed, 1);
    status.observe_generation(1, &event(1, CoefficientExpansionStage::Complete, 0, 0));
    assert_eq!(status.completed, 2);
    let progress = status.coefficient_expansion.as_ref().unwrap();
    assert_eq!(
        progress.requested_method,
        CoefficientExpansionMethod::NativeNamed
    );
    assert_eq!(
        progress.effective_method,
        CoefficientExpansionMethod::Physical
    );
    assert!(!status.detail.contains("formal pieces"));
}

#[test]
fn physical_status_and_legacy_timing_defaults_stay_distinct() {
    let mut status = snapshot();
    status.observe_generation(
        0,
        &GenerationProgress::Subtraction {
            sector: 0,
            total: 1,
            terms: 3,
        },
    );
    assert!(status.coefficient_expansion.is_none());
    status.observe_generation(
        0,
        &GenerationProgress::PhaseTiming {
            phase: GenerationPhase::CoefficientExpansion,
            seconds: 2.5,
        },
    );
    assert_eq!(status.timings.coefficient_expansion_seconds, 2.5);
    assert_eq!(status.timings.subtraction_seconds, 0.0);
    assert_eq!(status.timings.laurent_seconds, 0.0);
    let mut legacy = serde_json::to_value(&status).unwrap();
    assert!(legacy.get("coefficient_expansion").is_none());
    legacy["timings"]
        .as_object_mut()
        .unwrap()
        .remove("coefficient_expansion_seconds");
    let restored: GenerationSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(restored.coefficient_expansion.is_none());
    assert_eq!(restored.timings.coefficient_expansion_seconds, 0.0);
}

#[test]
fn formula_phase_counts_and_timing_are_distinct_from_sector_instantiation() {
    let mut status = snapshot();
    status.observe_generation(1, &event(0, CoefficientExpansionStage::Lowering, 1, 7));
    status.observe_generation(
        1,
        &GenerationProgress::FormulaPreparation {
            completed: 0,
            total: 2,
            sectors: 5,
            reused: 3,
        },
    );
    assert_eq!(status.stage, GenerationStage::FormulaPreparation);
    assert_eq!((status.completed, status.total), (0, Some(2)));
    assert!(status.coefficient_expansion.is_none());
    assert!(status.detail.contains("5 eligible sectors"));
    assert!(status.detail.contains("3 shared uses"));
    for seconds in [0.5, 0.25] {
        status.observe_generation(
            1,
            &GenerationProgress::PhaseTiming {
                phase: GenerationPhase::FormulaPreparation,
                seconds,
            },
        );
    }
    assert_eq!(status.timings.formula_preparation_seconds, Some(0.75));
    assert_eq!(status.timings.coefficient_expansion_seconds, 0.0);
    status.observe_generation(
        1,
        &GenerationProgress::FormulaPreparation {
            completed: 2,
            total: 2,
            sectors: 5,
            reused: 3,
        },
    );
    status.observe_generation(
        1,
        &GenerationProgress::FormulaInstantiation {
            sector: 0,
            total: 5,
        },
    );
    assert_eq!(status.stage, GenerationStage::CoefficientExpansion);
    assert_eq!((status.completed, status.total), (0, Some(5)));
    assert!(status.coefficient_expansion.is_none());
    assert_eq!(status.formula_preparation.unwrap().completed, 2);
    status.observe_generation(
        1,
        &GenerationProgress::FormulaInstantiation {
            sector: 5,
            total: 5,
        },
    );
    assert_eq!((status.completed, status.total), (5, Some(5)));
    assert_eq!(status.timings.formula_preparation_seconds, Some(0.75));
}

#[test]
fn unknown_formula_observations_differ_from_completed_zero_work() {
    let status = snapshot();
    let legacy = serde_json::to_value(&status).unwrap();
    assert!(legacy.get("formula_preparation").is_none());
    assert!(
        legacy["timings"]
            .get("formula_preparation_seconds")
            .is_none()
    );
    let mut restored: GenerationSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(restored.formula_preparation.is_none());
    assert!(restored.timings.formula_preparation_seconds.is_none());
    restored.observe_generation(
        0,
        &GenerationProgress::FormulaPreparation {
            completed: 0,
            total: 0,
            sectors: 0,
            reused: 0,
        },
    );
    restored.observe_generation(
        0,
        &GenerationProgress::PhaseTiming {
            phase: GenerationPhase::FormulaPreparation,
            seconds: 0.0,
        },
    );
    let encoded = serde_json::to_value(&restored).unwrap();
    assert_eq!(encoded["formula_preparation"]["total"], 0);
    assert_eq!(encoded["formula_preparation"]["reused"], 0);
    assert_eq!(encoded["timings"]["formula_preparation_seconds"], 0.0);
    let replay: GenerationSnapshot = serde_json::from_value(encoded).unwrap();
    assert_eq!(replay, restored);
}
