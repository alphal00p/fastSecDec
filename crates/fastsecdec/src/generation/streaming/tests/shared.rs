use super::*;
use crate::{
    generation::{GenerationPhase, mapping::profile},
    kernel::indexed::ProgramRecipe,
    parametric::FactorSemantics,
};
use symbolica::atom::AtomCore;

fn source(zero_causal_power: bool) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("shared_recipe_gate::x")],
        symbol!("shared_recipe_gate::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-1-shared_recipe_gate::eps")],
            vec![
                PolynomialFactor::new(
                    parse!("1/4-shared_recipe_gate::x"),
                    Atom::num(if zero_causal_power { 0 } else { -1 }),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    parse!("1+shared_recipe_gate::x^2"),
                    Atom::Zero,
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn shared_geometry_and_residuals_are_built_once_with_all_zero_power_declarations() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(true);
    let mut geometry_events = 0;
    let recipes = prepare_recipes_with_runtime(
        &input,
        &GenerationOptions::default(),
        &[
            ProgramRecipe::UndeformedV1,
            ProgramRecipe::FixedV1,
            ProgramRecipe::DynamicPolynomialV1,
        ],
        &[],
        &[],
        directory.path(),
        |status| {
            geometry_events += usize::from(matches!(
                status,
                crate::generation::GenerationProgress::PhaseTiming {
                    phase: GenerationPhase::Geometry,
                    ..
                }
            ));
            ControlFlow::Continue(())
        },
    )
    .unwrap();
    assert_eq!(geometry_events, 1);
    assert_eq!(recipes.recipes.len(), 3);
    assert!(
        recipes
            .recipes
            .iter()
            .all(|recipe| recipe.charts[0].map == recipes.recipes[0].charts[0].map)
    );
    profile::begin();
    let prepared = prepare_chart_source(
        directory.path(),
        &recipes,
        &recipes.recipes[0].charts[0],
        keep,
    )
    .unwrap();
    let data = records::read_prepared(directory.path(), &prepared.record).unwrap();
    assert!(data.retains_declarations);
    let factors = &data.terms.as_ref().unwrap()[0].residuals;
    assert_eq!(factors.len(), 2);
    assert_eq!(
        factors
            .iter()
            .map(|(_, exponent, role)| (exponent.is_zero(), *role))
            .collect::<Vec<_>>(),
        vec![
            (true, FactorSemantics::Causal),
            (true, FactorSemantics::Positive)
        ]
    );
    drop(data);
    for recipe in &recipes.recipes {
        let chart = discover_prepared(directory.path(), recipe, &prepared, keep).unwrap();
        let data = records::read_chart(directory.path(), &chart.record).unwrap();
        if recipe.program_recipe != ProgramRecipe::UndeformedV1 {
            assert_eq!(data.contour.unwrap().positive_polynomials().len(), 1);
        }
    }
    let measured = profile::take();
    assert_eq!(
        measured.factors.len(),
        2,
        "recipe application repeated source extraction"
    );
}

#[test]
fn cancelled_density_keeps_its_shared_branch_declarations_until_recipe_application() {
    let input = source(true);
    let first = input.terms()[0].clone();
    let opposite = ParametricTerm::new(
        -Atom::one(),
        first.monomial_powers().to_vec(),
        first.factors().to_vec(),
    );
    let input = ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![first, opposite],
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let recipes = prepare_recipes_with_runtime(
        &input,
        &GenerationOptions::default(),
        &[
            ProgramRecipe::UndeformedV1,
            ProgramRecipe::FixedV1,
            ProgramRecipe::DynamicPolynomialV1,
        ],
        &[],
        &[],
        directory.path(),
        keep,
    )
    .unwrap();
    let source = prepare_chart_source(
        directory.path(),
        &recipes,
        &recipes.recipes[0].charts[0],
        keep,
    )
    .unwrap();
    let prepared = records::read_prepared(directory.path(), &source.record).unwrap();
    assert_eq!(prepared.terms.as_ref().unwrap().len(), 2);
    assert!(
        prepared
            .terms
            .as_ref()
            .unwrap()
            .iter()
            .all(|term| term.residuals.len() == 2)
    );
    for recipe in &recipes.recipes {
        let receipt = discover_prepared(directory.path(), recipe, &source, keep).unwrap();
        let chart = records::read_chart(directory.path(), &receipt.record).unwrap();
        assert!(chart.mapped.is_empty());
        assert_eq!(
            chart.contour.is_some(),
            recipe.program_recipe != ProgramRecipe::UndeformedV1
        );
    }
}

fn evaluated(generated: &crate::generation::GeneratedIntegral) -> Vec<Complex<f64>> {
    let recipe = generated.program_descriptor().map_or_else(
        || {
            if generated
                .metadata()
                .charts()
                .iter()
                .any(|chart| chart.contour().is_some())
            {
                ProgramRecipe::FixedV1
            } else {
                ProgramRecipe::UndeformedV1
            }
        },
        |descriptor| descriptor.recipe(),
    );
    let runtime = recipe
        .recipe_parameters()
        .iter()
        .map(|name| symbol!(*name))
        .collect::<Vec<_>>();
    let runtime_values = runtime
        .iter()
        .map(|symbol| {
            let value = if *symbol == crate::contour::dynamic::safety_fraction_symbol() {
                0.8
            } else if *symbol == crate::contour::dynamic::displacement_cap_symbol() {
                1.
            } else {
                0.2
            };
            Complex::new(value, 0.)
        })
        .collect::<Vec<_>>();
    let mut result = vec![Complex::new(0., 0.); generated.orders().len()];
    let _preparing = generated
        .program_descriptor()
        .map(|descriptor| descriptor.enter());
    for (index, expression) in generated.exact_coefficients().iter().enumerate() {
        let mut evaluator = expression
            .evaluator(
                &runtime
                    .iter()
                    .map(|symbol| Atom::var(*symbol))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap()
            .map_coeff(&|number| Complex::new(number.re.to_f64(), number.im.to_f64()));
        result[index] += evaluator.evaluate_single(&runtime_values);
    }
    for sector in generated.sectors() {
        let inputs = sector
            .parameters()
            .iter()
            .chain(&runtime)
            .map(|symbol| Atom::var(*symbol))
            .collect::<Vec<_>>();
        let values = sector
            .parameters()
            .iter()
            .map(|_| Complex::new(0.37, 0.))
            .chain(runtime_values.iter().cloned())
            .collect::<Vec<_>>();
        for (index, expression) in sector.coefficients().iter().enumerate() {
            let mut evaluator = expression
                .evaluator(&inputs)
                .build()
                .unwrap()
                .map_coeff(&|number| Complex::new(number.re.to_f64(), number.im.to_f64()));
            result[index] += evaluator.evaluate_single(&values);
        }
    }
    result
}

#[test]
fn shared_sources_match_selected_generation_in_both_modes_and_subtractions() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let directory = tempfile::tempdir().unwrap();
            let options = GenerationOptions {
                mode,
                subtraction,
                ..Default::default()
            };
            let recipes = prepare_recipes_with_runtime(
                &source(false),
                &options,
                &[
                    ProgramRecipe::UndeformedV1,
                    ProgramRecipe::FixedV1,
                    ProgramRecipe::DynamicPolynomialV1,
                ],
                &[],
                &[],
                directory.path(),
                keep,
            )
            .unwrap();
            let shared = recipes.recipes[0]
                .charts
                .iter()
                .map(|job| prepare_chart_source(directory.path(), &recipes, job, keep).unwrap())
                .collect::<Vec<_>>();
            for recipe in &recipes.recipes {
                let shared_charts: Vec<_> = shared
                    .iter()
                    .map(|chart| discover_prepared(directory.path(), recipe, chart, keep).unwrap())
                    .collect();
                if mode == GenerationMode::NumericalDual {
                    let source_data =
                        records::read_prepared(directory.path(), &shared[0].record).unwrap();
                    assert!(source_data.opaque.is_some());
                    assert!(source_data.terms.is_some());
                    let chart =
                        records::read_chart(directory.path(), &shared_charts[0].record).unwrap();
                    let opaque = chart.deferred.unwrap();
                    assert_eq!(
                        opaque.iter().any(|term| !term.factors.is_empty()),
                        recipe.program_recipe == ProgramRecipe::UndeformedV1
                    );
                }
                let direct_charts = recipe
                    .charts
                    .iter()
                    .map(|job| discover(directory.path(), recipe, job, keep).unwrap())
                    .collect();
                let shared_plan = finish_charts(directory.path(), recipe, shared_charts);
                let direct_plan = finish_charts(directory.path(), recipe, direct_charts);
                assert_eq!(shared_plan.sectors.len(), direct_plan.sectors.len());
                assert_eq!(shared_plan.unique_formulas, direct_plan.unique_formulas);
                for (shared, direct) in shared_plan.sectors.iter().zip(&direct_plan.sectors) {
                    let shared = generate_sector(directory.path(), shared, keep).unwrap();
                    let direct = generate_sector(directory.path(), direct, keep).unwrap();
                    assert_eq!(shared.generated.orders(), direct.generated.orders());
                    let shared = evaluated(&shared.generated);
                    let direct = evaluated(&direct.generated);
                    for (left, right) in shared.iter().zip(direct) {
                        assert!(left.re.is_finite() && left.im.is_finite());
                        assert!(
                            (left.re - right.re).abs() < 1e-11
                                && (left.im - right.im).abs() < 1e-11,
                            "{mode:?}/{subtraction:?}/{:?}: {left:?} != {right:?}",
                            recipe.program_recipe
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn shared_receipts_reject_foreign_sources_dimensions_maps_and_recipe_directories() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(false);
    let recipes = prepare_recipes_with_runtime(
        &input,
        &GenerationOptions::default(),
        &[ProgramRecipe::FixedV1, ProgramRecipe::DynamicPolynomialV1],
        &[],
        &[],
        directory.path(),
        keep,
    )
    .unwrap();
    let prepared = prepare_chart_source(
        directory.path(),
        &recipes,
        &recipes.recipes[0].charts[0],
        keep,
    )
    .unwrap();
    let recipe = &recipes.recipes[0];
    let mut foreign = prepared.clone();
    foreign.source_identity = "0".repeat(64);
    assert!(discover_prepared(directory.path(), recipe, &foreign, keep).is_err());
    foreign = prepared.clone();
    foreign.dimension += 1;
    assert!(discover_prepared(directory.path(), recipe, &foreign, keep).is_err());
    foreign = prepared.clone();
    foreign.map = recipe.source.clone();
    assert!(discover_prepared(directory.path(), recipe, &foreign, keep).is_err());
    foreign = prepared.clone();
    foreign.index += 1;
    assert!(discover_prepared(directory.path(), recipe, &foreign, keep).is_err());
    let mut data = records::read_prepared(directory.path(), &prepared.record).unwrap();
    data.retains_declarations = false;
    foreign = prepared.clone();
    foreign.record = records::write_prepared(directory.path(), &data).unwrap();
    assert!(discover_prepared(directory.path(), recipe, &foreign, keep).is_err());
    let mut forged = recipes.clone();
    forged.recipes[1].source_identity = "0".repeat(64);
    assert!(prepare_chart_source(directory.path(), &forged, &recipe.charts[0], keep).is_err());
    let mut forged = recipes.clone();
    forged.recipes[1].charts[0].map = recipe.source.clone();
    assert!(prepare_chart_source(directory.path(), &forged, &recipe.charts[0], keep).is_err());
    assert!(
        prepare_recipes_with_runtime(
            &input,
            &GenerationOptions::default(),
            &[],
            &[],
            &[],
            directory.path(),
            keep
        )
        .is_err()
    );
    assert!(
        prepare_recipes_with_runtime(
            &input,
            &GenerationOptions::default(),
            &[ProgramRecipe::FixedV1, ProgramRecipe::FixedV1],
            &[],
            &[],
            directory.path(),
            keep
        )
        .is_err()
    );
}
