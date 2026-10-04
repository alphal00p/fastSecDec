//! Independent native one-loop reduction/master checks of Gaussian numerators.
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc, time::Instant};

use fastsecdec::{
    Atom, AtomCore, Kinematics, Model,
    generation::{GenerationOptions, GenerationProgress, generate},
    input::{GraphIntegral, default_algebra_settings},
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec, VectorEstimate},
    parametric::ParametricIntegrand,
    status::CoefficientComponent,
};
use feynkit_graph::symbols;
use oneloop::EvaluationBackend;
use oneloopreduce::OneLoopMasters;
use symbolica::{domains::float::Complex, parse, symbol};

fn graph(dot: &str, triangle: bool, scale: i64, mass: i64) -> GraphIntegral {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let p = (0..3)
        .map(|index| symbols::external_momentum().call(index))
        .collect::<Vec<_>>();
    let mut kin = Kinematics::in_dimension(&parse!("hepkit_reduction::D")).unwrap();
    if triangle {
        // Preserve the actual historical propagators: virtualities s,0,2s.
        kin = kin
            .with_mass_squared(&p[0], Atom::num(-scale))
            .unwrap()
            .with_mass_squared(&p[1], Atom::Zero)
            .unwrap()
            .with_scalar_product(&p[0], &p[1], Atom::num((scale, 2)))
            .unwrap();
    } else {
        for momentum in &p {
            kin = kin.with_mass_squared(momentum, Atom::Zero).unwrap();
        }
        for (left, right, value) in [(0, 1, -scale), (1, 2, -scale), (0, 2, 2 * scale)] {
            kin = kin
                .with_scalar_product(&p[left], &p[right], Atom::num((value, 2)))
                .unwrap();
        }
    }
    GraphIntegral::from_dot(model, dot, &kin)
        .unwrap()
        .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::num(mass))]))
        .unwrap()
}

/// Compose existing native APIs as the HEPKit bridge does, retaining the
/// dimension-dependent coefficients. No reduction or master formula lives here.
fn native_coefficients(graph: &GraphIntegral, mu_squared: i64) -> Result<[f64; 3], String> {
    let numerator = graph
        .scalar_numerator(&default_algebra_settings())
        .map_err(|error| error.to_string())?;
    let powers = graph
        .powers()
        .iter()
        .map(|power| i32::try_from(*power).unwrap())
        .collect::<Vec<_>>();
    let reduction = oneloopreduce::reduce_family(graph.family(), &powers, &numerator)
        .map_err(|error| error.to_string())?;
    let dimension = symbol!("hepkit_reduction::D");
    let mut terms = BTreeMap::<Atom, Atom>::new();
    for (coefficient, master) in reduction.terms {
        let master = OneLoopMasters.symbol_with_scale(&master, &Atom::num(mu_squared));
        if master.contains_symbol(dimension) {
            return Err("dimension-dependent master arguments require another reference".into());
        }
        *terms.entry(master).or_insert(Atom::Zero) += coefficient;
    }
    let mut result = [0.0; 3];
    for (master, coefficient) in terms {
        let coefficient = coefficient.cancel();
        if coefficient.is_zero() {
            continue;
        }
        let series = coefficient
            .series(dimension, 4, 2)
            .map_err(|error| error.to_string())?;
        if series.get_trailing_exponent() < 0 {
            return Err(format!(
                "coefficient of {master} has a D=4 pole; positive master epsilon orders unavailable"
            ));
        }
        if series
            .terms()
            .any(|(power, coefficient)| !power.is_integer() && !coefficient.is_zero())
        {
            return Err("fractional reduction coefficient powers are unsupported".into());
        }
        let mut coefficients = [0.0; 3];
        for (order, multiplier) in [1.0, -2.0, 4.0].into_iter().enumerate() {
            let coefficient = series.coefficient((order as i64).into()).unwrap();
            coefficients[order] = f64::try_from(&coefficient)
                .map_err(|error| format!("non-numeric coefficient {coefficient}: {error}"))?
                * multiplier;
        }
        let (family, arguments) = oneloop::master_arguments(&master)?;
        let arguments = arguments
            .iter()
            .map(|argument| {
                Complex::<f64>::try_from(argument)
                    .map_err(|error| format!("non-numeric master argument {argument}: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut values = [Complex::new(0.0, 0.0); 3];
        oneloop::evaluate_with_backend(
            family,
            &arguments,
            &mut values,
            EvaluationBackend::Expression,
        )?;
        if values
            .iter()
            .any(|value| !value.re.is_finite() || !value.im.is_finite() || value.im.abs() > 1e-9)
        {
            return Err(format!(
                "nonfinite/non-Euclidean master {master}: {values:?}"
            ));
        }
        for output in 0..3 {
            for order in 0..3 - output {
                result[output] += coefficients[order] * values[output + order].re;
            }
        }
    }
    if result.iter().any(|value| !value.is_finite()) {
        return Err("nonfinite sum of native reduction/master coefficients".into());
    }
    Ok(result)
}

fn with_stack(work: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

fn integrate(graph: GraphIntegral, mu_squared: i64) -> VectorEstimate {
    let parameterization_started = Instant::now();
    let epsilon = symbol!("hepkit_reduction::eps");
    let multiplier = Atom::num(mu_squared).pow(Atom::var(epsilon))
        * parse!(
            "gamma(1-2*hepkit_reduction::eps)/(gamma(1+hepkit_reduction::eps)*gamma(1-hepkit_reduction::eps)^2)"
        );
    let graph = graph.with_measure_multiplier(multiplier);
    let input = ParametricIntegrand::from_graph(
        &graph,
        (0..graph.powers().len())
            .map(|index| symbol!(format!("hepkit_reduction::x{index}")))
            .collect(),
        epsilon,
        parse!("4-2*hepkit_reduction::eps"),
    )
    .unwrap();
    println!(
        "native Gaussian parameterization: {:.6}s, {} terms",
        parameterization_started.elapsed().as_secs_f64(),
        input.terms().len()
    );
    let mut timings = BTreeMap::<String, f64>::new();
    let generated = generate(&input, &GenerationOptions::default(), |progress| {
        if let GenerationProgress::PhaseTiming { phase, seconds } = progress {
            *timings.entry(format!("{phase:?}")).or_default() += seconds;
        }
        ControlFlow::Continue(())
    })
    .unwrap();
    println!("generation stages: {timings:?}");
    let compilation_started = Instant::now();
    let mut kernels = generated.compile().unwrap();
    println!(
        "portable O2 compilation: {:.6}s, {} kernels",
        compilation_started.elapsed().as_secs_f64(),
        kernels.sectors().len()
    );
    let integration_started = Instant::now();
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
            points: 2048,
            shifts: 32,
            seed: 78139,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|id| session.worker_context(id as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id() as usize;
        let value = workers[id]
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[id].evaluate(point, output)
            })
            .unwrap();
        session.submit(value).unwrap();
    }
    let estimate = session.estimate().unwrap();
    println!(
        "QMC evaluation: {:.6}s",
        integration_started.elapsed().as_secs_f64()
    );
    estimate
}

fn compare(name: &str, graph: GraphIntegral, mu_squared: i64) {
    let expected = native_coefficients(&graph, mu_squared)
        .unwrap_or_else(|error| panic!("{name}: native reduction/master limitation: {error}"));
    let actual = integrate(graph, mu_squared);
    assert!(actual.production_complete);
    assert_eq!(actual.orders.last(), Some(&0));
    assert!(actual.orders.iter().all(|order| (-2..=0).contains(order)));
    assert!(
        actual
            .components
            .iter()
            .all(|component| *component == CoefficientComponent::Real)
    );
    for (order, expected) in [0, -1, -2].into_iter().zip(expected) {
        let (value, error) = actual
            .orders
            .iter()
            .position(|candidate| *candidate == order)
            .map(|index| (actual.mean[index], actual.standard_error[index]))
            .unwrap_or((0.0, 0.0));
        println!("{name}, eps^{order}: {value} +/- {error}; native {expected}");
        assert!(
            error < 1e-3 * expected.abs().max(1.0),
            "{name}: inconclusive precision"
        );
        assert!(
            (value - expected).abs() <= 8.0 * error + 2e-8 * expected.abs().max(1.0),
            "{name}, eps^{order}: {value} +/- {error}; native reduction/master {expected}"
        );
    }
}

#[test]
fn native_triangle_rank_one_matches_reduction_and_masters() {
    with_stack(|| {
        for (scale, mass, mu_squared) in [(1, 0, 1), (3, 1, 4)] {
            compare(
                &format!("triangle rank1 scale{scale} mass{mass}"),
                graph(
                    include_str!("../../../examples/graphs/triangle_numerator.dot"),
                    true,
                    scale,
                    mass,
                ),
                mu_squared,
            );
        }
    });
}

#[test]
fn native_box_rank_one_matches_reduction_and_masters() {
    with_stack(|| {
        for (scale, mass, mu_squared) in [(1, 0, 1), (2, 1, 4)] {
            compare(
                &format!("box rank1 scale{scale} mass{mass}"),
                graph(
                    include_str!("../../../examples/graphs/box_numerator.dot"),
                    false,
                    scale,
                    mass,
                ),
                mu_squared,
            );
        }
    });
}

#[test]
fn native_box_rank_two_matches_reduction_and_masters() {
    with_stack(|| {
        for (scale, mass, mu_squared) in [(1, 0, 1), (1, 1, 1)] {
            compare(
                &format!("box rank2 scale{scale} mass{mass}"),
                graph(
                    include_str!("../../../examples/graphs/box_rank2_numerator.dot"),
                    false,
                    scale,
                    mass,
                ),
                mu_squared,
            );
        }
    });
}

#[test]
fn native_box_rank_five_matches_reduction_and_masters() {
    with_stack(|| {
        compare(
            "box rank5",
            graph(
                include_str!("../../../examples/graphs/box_high_rank_numerator.dot"),
                false,
                1,
                0,
            ),
            1,
        );
    });
}

#[test]
fn native_gram_degenerate_box_matches_reduction_and_masters() {
    with_stack(|| {
        let graph = graph(
            include_str!("../../../examples/graphs/box_rank2_numerator.dot"),
            false,
            0,
            1,
        );
        compare("zero Gram massive box rank2", graph, 1);
    });
}
