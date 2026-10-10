//! Local-strength controls use a native-verified quadratic root, so they test
//! the production subtraction/jet machinery independently of the root callback.
use super::*;
use crate::{
    contour::{
        SmoothContourMap,
        dynamic::{
            DynamicEnvelope, displacement_cap_symbol, lambda_cap_symbol, safety_fraction_symbol,
        },
    },
    generation::{
        SubtractionStrategy, captured_coefficients, captured_subtraction, mapping::MappedTerm,
    },
    kernel::{CompilationSettings, EvaluatorBackend},
    parametric::FactorSemantics,
};
use symbolica::{
    atom::AtomCore,
    domains::{
        float::{Complex, DoubleFloat, RealLike},
        rational::Rational,
    },
    evaluate::ExpressionEvaluator,
    parse, symbol,
};

type ExactProgram = ExpressionEvaluator<Complex<Rational>>;

fn programs(degree: i32, strategy: SubtractionStrategy) -> (ExactProgram, ExactProgram) {
    let x = symbol!("local_subtraction::x");
    let epsilon = symbol!("local_subtraction::eps");
    let f = parse!("1/4-local_subtraction::x");
    let envelope = DynamicEnvelope::new(&[x], f.clone(), &[]).unwrap();
    let coefficients = envelope.polynomial_coefficients().unwrap();
    assert_eq!(coefficients.len(), 1);
    // The existing exact quadratic identity proves this branch: a2*u^2=1,
    // u>0. Binding the constants here leaves a genuinely local x-dependent map.
    let lambda = (Atom::var(safety_fraction_symbol()) * Atom::var(lambda_cap_symbol())
        / coefficients[0].sqrt())
    .replace(safety_fraction_symbol())
    .with(Atom::num((1, 2)))
    .replace(lambda_cap_symbol())
    .with(Atom::num((1, 2)))
    .replace(displacement_cap_symbol())
    .with(Atom::num((1, 4)));
    assert!(!lambda.derivative(x).derivative(x).is_zero());
    let mut map = SmoothContourMap::new(&[x], f.clone(), lambda).unwrap();
    let powers = vec![Atom::num(-degree) - Atom::var(epsilon)];
    let regular = map.smooth_density(&powers, &[(f, Atom::num(-1), FactorSemantics::Causal)]);
    let options = GenerationOptions {
        subtraction: strategy,
        max_order: 0,
        ..Default::default()
    };
    let terms = vec![MappedTerm {
        powers: powers.clone(),
        prefactor: Atom::one(),
        regular: regular.clone(),
    }];
    let recipe = subtraction::expand(&terms, &[x], epsilon, &options, &[], &mut |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert!(
        recipe
            .requests
            .iter()
            .any(|request| request.derivatives[0] >= (degree - 1) as usize)
    );
    let settings = CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    };
    let dual = native::build(
        &DualSector {
            jacobian: None,
            contour_definitions: Arc::default(),
            programs: Arc::new(native::SourcePrograms::default()),
            source_parameters: vec![x],
            regulator: epsilon,
            parameters: vec![x],
            map: SectorMap {
                fixed_parameter: None,
                exponent_matrix: vec![vec![1.into()]],
                determinant: 1.into(),
                jacobian_powers: vec![0.into()],
                factor_valuations: vec![],
            },
            terms: vec![DualTerm { factors: vec![] }],
            mapped_regular: Some(vec![regular.clone()]),
            recipe: Arc::new(recipe),
            orders: vec![-1, 0],
        },
        &[],
        settings,
    )
    .unwrap();
    let expression = captured_subtraction(
        vec![(Atom::one(), regular, powers)],
        &[x],
        epsilon,
        &options,
    )
    .unwrap()
    .0;
    let coefficients = captured_coefficients(&expression, &[x], epsilon, 0)
        .unwrap()
        .coefficients;
    let roots = [-1, 0].map(|order| {
        coefficients
            .get(&order)
            .cloned()
            .unwrap_or_default()
            .into_inner()
    });
    let symbolic = Atom::evaluator_multiple(&roots, &[Atom::var(x)])
        .optimization_settings(settings.native())
        .build()
        .unwrap();
    (symbolic, dual)
}

#[test]
fn local_strength_higher_jets_preserve_taylor_ibp_and_complete_laurent_integrals() {
    for degree in [1, 2, 3] {
        for strategy in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let (symbolic, dual) = programs(degree, strategy);
            let map = |c: &Complex<Rational>| {
                Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im))
            };
            let mut symbolic = symbolic.map_coeff(&map);
            let mut dual = dual.map_coeff(&map);
            let zero = Complex::new(DoubleFloat::from(0.), DoubleFloat::from(0.));
            let mut a = vec![zero; 2];
            let mut b = vec![zero; 2];
            let mut integral = vec![zero; 2];
            let count = 4096;
            for index in 0..count {
                let x = (index as f64 + 0.5) / count as f64;
                let input = [Complex::new(DoubleFloat::from(x), DoubleFloat::from(0.))];
                symbolic.evaluate(&input, &mut a);
                dual.evaluate(&input, &mut b);
                for ((expected, actual), total) in a.iter().zip(&b).zip(&mut integral) {
                    let delta = *expected - *actual;
                    assert!(
                        delta.re.to_f64().hypot(delta.im.to_f64()) < 1e-12,
                        "degree={degree}, strategy={strategy:?}, x={x}, expected={expected:?}, actual={actual:?}"
                    );
                    *total += *actual;
                }
            }
            let integral = integral
                .iter()
                .map(|value| {
                    Complex::new(
                        value.re.to_f64() / count as f64,
                        value.im.to_f64() / count as f64,
                    )
                })
                .collect::<Vec<_>>();
            // Analytic continuation of x^(-degree-eps)/(1/4-x-i0), retaining
            // the entire pole+finite vector, including its nonzero imaginary part.
            let residue = 4_f64.powi(degree);
            let rational = (1..degree)
                .map(|k| 4_f64.powi(degree - k) / k as f64)
                .sum::<f64>();
            let expected = [
                Complex::new(-residue, 0.),
                Complex::new(
                    -residue * 3_f64.ln() - rational,
                    residue * std::f64::consts::PI,
                ),
            ];
            for (actual, expected) in integral.iter().zip(expected) {
                let delta = *actual - expected;
                assert!(
                    delta.re.hypot(delta.im) < 3e-4,
                    "degree={degree}, strategy={strategy:?}, expected={expected:?}, actual={actual:?}"
                );
            }
        }
    }
}
