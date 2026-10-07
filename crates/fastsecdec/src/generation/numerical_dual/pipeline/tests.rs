use super::*;
use crate::{
    generation::{GenerationMode, SubtractionStrategy, generate},
    parametric::{FactorRole, ParametricDomain, ParametricTerm, PolynomialFactor},
};
use symbolica::{parse, symbol};

fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![
            symbol!("formula_pipeline::x"),
            symbol!("formula_pipeline::y"),
        ],
        symbol!("formula_pipeline::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(
                    parse!("formula_pipeline::x+formula_pipeline::y"),
                    parse!("-2+formula_pipeline::eps"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("1+2*formula_pipeline::x+3*formula_pipeline::y"),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn whole_chart_formulas_are_shared_without_sharing_distinct_native_maps() {
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let options = GenerationOptions {
            mode: GenerationMode::NumericalDual,
            subtraction,
            ..Default::default()
        };
        let mut stats = None;
        let mut formula_finished = false;
        let mut assembly_started = false;
        let mut last_phase = None;
        let result = generate(&input(), &options, |event| {
            match event {
                GenerationProgress::FormulaPreparation {
                    completed,
                    total,
                    sectors,
                    reused,
                } => {
                    assert!(!assembly_started);
                    stats = Some((*completed, *total, *sectors, *reused));
                    formula_finished = completed == total;
                }
                GenerationProgress::FormulaInstantiation { .. } => {
                    assert!(formula_finished);
                    assembly_started = true;
                }
                GenerationProgress::PhaseTiming { phase, .. } => {
                    last_phase = Some(*phase);
                }
                _ => (),
            }
            ControlFlow::Continue(())
        })
        .unwrap();
        assert_eq!(stats, Some((1, 1, 2, 1)));
        assert_eq!(last_phase, Some(GenerationPhase::CoefficientExpansion));
        assert_eq!(result.sectors().len(), 2);
        let left = result.sectors()[0].deferred.as_ref().unwrap();
        let right = result.sectors()[1].deferred.as_ref().unwrap();
        assert!(Arc::ptr_eq(&left.recipe, &right.recipe));
        assert!(Arc::ptr_eq(&left.programs, &right.programs));
        assert_ne!(left.map, right.map);
    }
}

#[test]
fn cooperative_pause_keeps_prepared_recipe_owner_and_completed_discoveries() {
    let input = input();
    let options = GenerationOptions {
        mode: GenerationMode::NumericalDual,
        ..Default::default()
    };
    let prepared = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let maps = prepared
        .metadata()
        .charts
        .iter()
        .map(|chart| chart.geometry.clone())
        .collect();
    let parameters = prepared.sectors()[0].parameters().to_vec();
    let context = Arc::new(Context::new(&input, &options, parameters));
    let mut pipeline = Pipeline::new(context, maps);
    let mut supports = SupportCache::new(input.parameters());
    let mut original_recipe = None;
    let mut last_discoveries = 0;
    while !pipeline.is_complete() {
        pipeline
            .step(&mut supports, &mut |_| ControlFlow::Continue(()))
            .unwrap();
        match &pipeline.stage {
            Stage::Discover { charts, .. } => {
                assert!(charts.len() >= last_discoveries);
                last_discoveries = charts.len();
            }
            Stage::Formulas { recipes, .. } => {
                if let Some(recipe) = recipes.first() {
                    original_recipe = Some(recipe.clone());
                }
            }
            Stage::Instantiate { recipes, .. } => {
                assert!(Arc::ptr_eq(original_recipe.as_ref().unwrap(), &recipes[0]));
                assert_eq!(last_discoveries, 2);
            }
            Stage::Complete => (),
        }
    }
    let completed = pipeline.take_result();
    assert_eq!(completed.len(), 2);
    for chart in completed {
        assert!(Arc::ptr_eq(
            original_recipe.as_ref().unwrap(),
            &chart.sector.deferred.unwrap().recipe
        ));
    }
}
