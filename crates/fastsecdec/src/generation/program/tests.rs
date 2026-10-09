use crate::{
    generation::{GenerationMode, GenerationOptions, SubtractionStrategy, generate},
    kernel::indexed::ProgramRecipe,
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::float::Complex,
    symbol,
};

mod cubic;
mod higher;

fn pole() -> ParametricIntegrand {
    let x = symbol!("dynamic_generation_gate::x");
    let eps = symbol!("dynamic_generation_gate::eps");
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    Atom::num((1, 4)) - Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn selected_polynomial_recipe_retains_native_owners_and_analytic_subtraction() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let generated = generate(
                &pole(),
                &GenerationOptions {
                    program_recipe: ProgramRecipe::DynamicPolynomialV1,
                    mode,
                    subtraction,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            let descriptor = generated.program_descriptor().unwrap();
            assert_eq!(descriptor.recipe(), ProgramRecipe::DynamicPolynomialV1);
            assert_eq!(descriptor.charts().len(), 1);
            assert_eq!(generated.dynamic_check_sources().len(), 1);
            assert_eq!(generated.orders(), &[-1, 0]);
            assert_eq!(generated.sectors().len(), 1);
            let sector = &generated.sectors()[0];
            assert!(sector.program_descriptor().is_some());
            let mut inputs = sector
                .parameters()
                .iter()
                .map(|p| Atom::var(*p))
                .collect::<Vec<_>>();
            inputs.extend([
                Atom::var(crate::contour::dynamic::safety_fraction_symbol()),
                Atom::var(crate::contour::dynamic::lambda_cap_symbol()),
                Atom::var(crate::contour::dynamic::displacement_cap_symbol()),
            ]);
            let mut totals = Vec::new();
            for coefficient in sector.coefficients() {
                let mut evaluator = {
                    let _preparing = descriptor.enter();
                    coefficient
                        .evaluator(&inputs)
                        .build()
                        .unwrap()
                        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
                };
                let points = 8192;
                let mut sum = Complex::new(0., 0.);
                for i in 0..points {
                    let value = evaluator.evaluate_single(&[
                        Complex::new((i as f64 + 0.5) / points as f64, 0.),
                        Complex::new(0.8, 0.),
                        Complex::new(0.2, 0.),
                        Complex::new(1., 0.),
                    ]);
                    sum += value;
                }
                totals.push(sum / Complex::new(points as f64, 0.));
            }
            assert!((totals[0].re + 4.).abs() < 1e-10);
            assert!(totals[0].im.abs() < 1e-10);
            assert!(
                (totals[1].re + 4. * 3f64.ln()).abs() < 2e-5,
                "{mode:?}/{subtraction:?}: {:?}",
                totals[1]
            );
            assert!((totals[1].im - 4. * std::f64::consts::PI).abs() < 2e-5);
        }
    }
}
