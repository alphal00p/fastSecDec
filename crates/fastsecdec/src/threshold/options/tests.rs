use super::*;
use crate::threshold::gcad::*;
use crate::{
    generation::{GenerationMode, GenerationOptions},
    kernel::indexed::ProgramRecipe,
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use symbolica::{atom::Atom, symbol};

fn input(domain: ParametricDomain) -> ParametricIntegrand {
    let (x, y, eps, a) = symbol!(
        "threshold_options_test::x",
        "threshold_options_test::y",
        "threshold_options_test::eps",
        "threshold_options_test::a"
    );
    let coordinates = if domain == ParametricDomain::UnitCube {
        vec![x]
    } else {
        vec![x, y]
    };
    let factor = if domain == ParametricDomain::ProjectiveSimplex {
        Atom::var(x) - Atom::var(a) * (Atom::var(x) + Atom::var(y))
    } else {
        Atom::var(x) - Atom::var(a)
    };
    let mut factors = vec![
        PolynomialFactor::new(
            factor,
            -Atom::one() - Atom::var(eps),
            FactorRole::Singularity,
        )
        .with_semantics(FactorSemantics::Causal),
    ];
    if domain == ParametricDomain::ProjectiveSimplex {
        factors.push(
            PolynomialFactor::new(
                Atom::var(x) + Atom::var(y),
                -Atom::one() + Atom::var(eps),
                FactorRole::Singularity,
            )
            .with_semantics(FactorSemantics::Positive),
        );
    }
    ParametricIntegrand::new(
        coordinates.clone(),
        eps,
        domain,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::zero(); coordinates.len()],
            factors,
        )],
    )
    .unwrap()
}
#[test]
fn options_preserve_source_selection_and_reject_incompatible_recipes() {
    let mut options = ThresholdDecompositionOptions::default();
    let mut generation = GenerationOptions::default();
    assert_eq!(
        options.effective_strategy(&generation).unwrap(),
        ThresholdStrategy::GcadFirst
    );
    generation.source_sectors = Some(vec![3, 1]);
    assert_eq!(
        options.effective_strategy(&generation).unwrap(),
        ThresholdStrategy::SectorFirst
    );
    assert_eq!(generation.source_sectors, Some(vec![3, 1]));
    options.strategy = ThresholdStrategy::GcadFirst;
    assert!(options.effective_strategy(&generation).is_err());
    options.strategy = ThresholdStrategy::Auto;
    for bad in [vec![], vec![2, 2]] {
        generation.source_sectors = Some(bad);
        assert!(options.effective_strategy(&generation).is_err());
    }
    generation.source_sectors = None;
    for recipe in [
        ProgramRecipe::FixedV1,
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        generation.program_recipe = recipe;
        assert!(options.effective_strategy(&generation).is_err());
    }
    generation.program_recipe = ProgramRecipe::UndeformedV1;
    generation.mode = GenerationMode::NumericalDual;
    assert!(options.effective_strategy(&generation).is_err());
}
#[test]
fn options_build_native_geometry_without_changing_exact_bindings_or_parameter_roles() {
    let mut options = ThresholdDecompositionOptions::default();
    let a = symbol!("threshold_options_test::a");
    options.kinematics.exact_values.insert(a, (1, 3).into());
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::ProjectiveSimplex,
    ] {
        let request = options
            .gcad_first_request(&input(domain), &GenerationOptions::default())
            .unwrap();
        assert_eq!(request.domain().coordinates().len(), 1);
        assert_eq!(
            request.kinematics().exact_values[&a],
            symbolica::prelude::Rational::from((1, 3))
        );
        assert_eq!(
            request.projective_preparation().is_some(),
            domain == ParametricDomain::ProjectiveSimplex
        );
        assert_eq!(request.solve_verified().unwrap().cells().len(), 2);
    }
    options.kinematics.exact_values.clear();
    options.kinematics.runtime_parameters.push(a);
    options.kinematics.strict_positive = vec![Atom::var(a), Atom::one() - Atom::var(a)];
    let request = options
        .gcad_first_request(
            &input(ParametricDomain::UnitCube),
            &GenerationOptions::default(),
        )
        .unwrap();
    assert_eq!(request.aliases()[0].role, AliasRole::RuntimeParameter);
    assert_eq!(request.aliases()[1].role, AliasRole::IntegrationCoordinate);
}
#[test]
fn options_delegate_solver_limits_and_never_fake_source_chart_preparation() {
    let mut options = ThresholdDecompositionOptions::default();
    let input = input(ParametricDomain::UnitCube);
    options
        .kinematics
        .exact_values
        .insert(symbol!("threshold_options_test::a"), (1, 3).into());
    options.gcad_limits.workers = 2;
    assert!(
        options
            .gcad_first_request(&input, &GenerationOptions::default())
            .is_err()
    );
    options.gcad_limits.workers = 1;
    options.strategy = ThresholdStrategy::SectorFirst;
    assert!(
        options
            .gcad_first_request(&input, &GenerationOptions::default())
            .is_err()
    );
    options.strategy = ThresholdStrategy::Auto;
    options.threshold_cells = Some(vec![0, 0]);
    assert!(
        options
            .gcad_first_request(&input, &GenerationOptions::default())
            .is_err()
    );
}
