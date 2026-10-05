//! Ignored, bounded diagnostics using the actual production generation path.

use super::{GenerationError, GenerationOptions, GenerationProgress, generate, symmetry};
use crate::{
    Kinematics, Model,
    input::GraphIntegral,
    parametric::{FactorRole, ParametricIntegrand, ParametricTerm, PolynomialFactor},
};
use std::{collections::BTreeMap, ops::ControlFlow, path::Path, sync::Arc, time::Instant};
use symbolica::{atom::AtomCore, parse, symbol};

mod gghh;

#[test]
fn native_common_factor_collection_can_strip_chart_monomials() {
    let variables = [parse!("x"), parse!("y")];
    let polynomial_in_coordinates = |expression: &symbolica::atom::Atom| {
        expression
            .is_polynomial(true, false)
            .is_some_and(|indeterminates| {
                indeterminates.iter().all(|indeterminate| {
                    variables
                        .iter()
                        .any(|variable| *indeterminate == variable.as_view())
                        || variables
                            .iter()
                            .all(|variable| !indeterminate.contains(variable.as_view()))
                })
            })
    };
    for (mapped, inverse, expected) in [
        (parse!("(x+x*y)^7"), parse!("x^-7"), parse!("(1+y)^7")),
        (parse!("(1+x^-1)^4"), parse!("x^4"), parse!("(1+x)^4")),
        (
            parse!("(x^-2*y+x^-1*y)^3"),
            parse!("x^6*y^-3"),
            parse!("(1+x)^3"),
        ),
        (
            parse!("(x+x*y)^10000"),
            parse!("x^-10000"),
            parse!("(1+y)^10000"),
        ),
    ] {
        let residual = mapped.collect_factors() * inverse;
        assert_eq!(residual, expected);
        assert!(polynomial_in_coordinates(&residual));
        assert!(residual.as_view().get_byte_size() < 100);
    }
    // Syntactic collection need not discover cancellation between powers of
    // sums. The existing native signed polynomial fallback remains necessary.
    let hidden = parse!("(1+x)^2-1-2*x");
    let candidate = hidden.collect_factors() * parse!("x^-2");
    assert!(!polynomial_in_coordinates(&candidate));
    assert_eq!(
        hidden
            .to_polynomial_in_vars::<i32>(&variables)
            .mul_exp(&[-2, 0])
            .flatten(false),
        parse!("1")
    );
    // Native polynomial recognition may choose a compound indeterminate;
    // accepting it requires the coordinate-independence check above.
    assert!(!polynomial_in_coordinates(&parse!("x^-1")));
    assert!(!polynomial_in_coordinates(&parse!("log(x)")));
}

#[test]
#[ignore = "bounded rank-five native factor collection and generation profiling"]
fn rank_five_native_factor_collection() {
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    let limit_seconds = std::env::var("FASTSECDEC_RANK_FIVE_SECONDS")
        .map(|value| value.parse::<f64>().unwrap())
        .unwrap_or(120.0);
    let chart_limit = std::env::var("FASTSECDEC_RANK_FIVE_CHARTS")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(64);
    let variant = std::env::var("FASTSECDEC_RANK_FIVE_VARIANT").unwrap_or_else(|_| "both".into());
    let stripping =
        std::env::var("FASTSECDEC_RANK_FIVE_STRIPPING").unwrap_or_else(|_| "factored".into());
    let support_cache =
        std::env::var("FASTSECDEC_RANK_FIVE_SUPPORT_CACHE").unwrap_or_else(|_| "enabled".into());
    let build = if cfg!(debug_assertions) {
        "development"
    } else {
        "release"
    };
    assert!(limit_seconds.is_finite() && limit_seconds > 0.0 && chart_limit > 0);
    assert!(["both", "unchanged", "collected"].contains(&variant.as_str()));
    assert!(["sparse", "factored"].contains(&stripping.as_str()));
    assert!(["enabled", "disabled"].contains(&support_cache.as_str()));
    super::mapping::profile_sparse_only(stripping == "sparse");
    let _support_profile = super::support::profile_bypass(support_cache == "disabled");
    let parameterization_started = Instant::now();
    let original = input();
    let parameterization_seconds = parameterization_started.elapsed().as_secs_f64();
    let factors_before = factor_sizes(&original);
    // Bound the explicit fixture experiment before any additional CAS work.
    assert!(factors_before.iter().sum::<usize>() <= 16 * 1024 * 1024);
    eprintln!(
        "rank-five Gaussian: {parameterization_seconds:.3}s, factor bytes {factors_before:?}"
    );

    for collected in [false, true] {
        let name = if collected { "collected" } else { "unchanged" };
        if variant != "both" && variant != name {
            continue;
        }
        let collection_started = Instant::now();
        let terms = original
            .terms()
            .iter()
            .map(|term| {
                ParametricTerm::new(
                    term.prefactor().clone(),
                    term.monomial_powers().to_vec(),
                    term.factors()
                        .iter()
                        .map(|factor| {
                            let polynomial = if collected && factor.role() == FactorRole::Polynomial
                            {
                                factor.polynomial().collect_factors()
                            } else {
                                factor.polynomial().clone()
                            };
                            PolynomialFactor::new(
                                polynomial,
                                factor.exponent().clone(),
                                factor.role(),
                            )
                        })
                        .collect(),
                )
            })
            .collect();
        let collection_seconds = collection_started.elapsed().as_secs_f64();
        let validation_started = Instant::now();
        let input = ParametricIntegrand::new(
            original.parameters().to_vec(),
            original.regulator(),
            original.domain(),
            terms,
        )
        .unwrap();
        // Independent native sparse-polynomial equality is confined to the
        // bounded final numerator factors; the primary density stays factored.
        let mut variables = original
            .parameters()
            .iter()
            .map(|p| symbolica::atom::Atom::var(*p))
            .collect::<Vec<_>>();
        variables.push(symbolica::atom::Atom::var(original.regulator()));
        for (before, after) in original.terms().iter().zip(input.terms()) {
            for (before, after) in before.factors().iter().zip(after.factors()) {
                assert_eq!(
                    before.polynomial().to_polynomial_in_vars::<u32>(&variables),
                    after.polynomial().to_polynomial_in_vars::<u32>(&variables),
                );
            }
        }
        let validation_seconds = validation_started.elapsed().as_secs_f64();
        let factors_after = factor_sizes(&input);
        eprintln!(
            "{name}: collection {collection_seconds:.3}s, validation {validation_seconds:.3}s, factor bytes {factors_after:?}"
        );

        symmetry::begin_profile();
        super::mapping::profile::begin();
        let started = Instant::now();
        let mut timings = BTreeMap::<String, f64>::new();
        let mut charts_started = 0;
        let mut total_charts = None;
        let result = generate(&input, &GenerationOptions::default(), |status| {
            if let GenerationProgress::PhaseTiming { phase, seconds } = status {
                *timings.entry(format!("{phase:?}")).or_default() += seconds;
            }
            if let GenerationProgress::Factorization { sector, total } = status {
                total_charts = Some(*total);
                if *sector >= chart_limit {
                    return ControlFlow::Break(());
                }
                charts_started += 1;
            }
            if started.elapsed().as_secs_f64() >= limit_seconds {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
        let generation_seconds = started.elapsed().as_secs_f64();
        let graphs = symmetry::take_profile();
        let mapping = super::mapping::profile::take();
        let (completed, numerical_sectors, vector_fingerprint) = match result {
            Ok(generated) => {
                // Native canonical printing and hashing are outside all timed
                // stages; paired cache modes must preserve the full vector.
                let vector = serde_json::json!({
                    "orders": generated.orders(),
                    "exact": generated.exact_coefficients().iter().map(AtomCore::to_canonical_string).collect::<Vec<_>>(),
                    "sectors": generated.sectors().iter().map(|sector| serde_json::json!({
                        "parameters": sector.parameters().iter().map(|p| symbolica::atom::Atom::var(*p).to_canonical_string()).collect::<Vec<_>>(),
                        "coefficients": sector.coefficients().iter().map(AtomCore::to_canonical_string).collect::<Vec<_>>(),
                        "cancellation_terms": sector.cancellation_terms(),
                    })).collect::<Vec<_>>(),
                });
                (
                    true,
                    Some(generated.sectors().len()),
                    Some(
                        blake3::hash(&serde_json::to_vec(&vector).unwrap())
                            .to_hex()
                            .to_string(),
                    ),
                )
            }
            Err(GenerationError::Cancelled) => (false, None, None),
            Err(error) => panic!("rank-five {name}: {error}"),
        };
        eprintln!(
            "{name}: completed {completed}, {charts_started}/{total_charts:?} charts, {generation_seconds:.3}s; stages {timings:?}"
        );
        let report = serde_json::json!({
            "fixture": "examples/runs/box_high_rank_numerator.toml; explicit measure multiplier one",
            "variant": name, "monomial_stripping": stripping, "parameterization_seconds": parameterization_seconds,
            "collection_seconds": collection_seconds, "validation_seconds": validation_seconds,
            "factor_bytes_before": factors_before, "factor_bytes_after": factors_after,
            "generation_seconds": generation_seconds, "stage_seconds": timings,
            "completed": completed, "numerical_sectors": numerical_sectors,
            "laurent_vector_fingerprint": vector_fingerprint,
            "charts_started": charts_started, "total_charts": total_charts,
            "graphs": graphs, "chart_limit": chart_limit, "cooperative_seconds_limit": limit_seconds,
            "mapping_factors": mapping.factors, "support_cache": mapping.support_cache,
            "limit_semantics": "checked at existing generation progress boundaries; native CAS/canonization calls are not interrupted",
            "build": build,
            "measurement_scope": "native generation diagnostic; no reference performance acceptance claim",
        });
        std::fs::write(
            evidence.join(format!(
                "rank-five-generation-{build}-{stripping}-{name}-support-{support_cache}.json"
            )),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    super::mapping::profile_sparse_only(false);
}

fn factor_sizes(input: &ParametricIntegrand) -> Vec<usize> {
    input
        .terms()
        .iter()
        .flat_map(|term| term.factors())
        .map(|factor| factor.polynomial().as_view().get_byte_size())
        .collect()
}

pub(crate) fn input() -> ParametricIntegrand {
    use symbolica::atom::Atom;
    let model = Arc::new(
        Model::from_json(include_str!("../../../../examples/models/scalar.json")).unwrap(),
    );
    let p = (0..3)
        .map(|index| feynkit_graph::symbols::external_momentum().call(index))
        .collect::<Vec<_>>();
    let mut kinematics = Kinematics::in_dimension(&parse!("rank_five_profile::D")).unwrap();
    for momentum in &p {
        kinematics = kinematics.with_mass_squared(momentum, Atom::Zero).unwrap();
    }
    for (left, right, value) in [(0, 1, -1), (1, 2, -1), (0, 2, 2)] {
        kinematics = kinematics
            .with_scalar_product(&p[left], &p[right], Atom::num((value, 2)))
            .unwrap();
    }
    let graph = GraphIntegral::from_dot(
        model,
        include_str!("../../../../examples/graphs/box_high_rank_numerator.dot"),
        &kinematics,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::Zero)]))
    .unwrap();
    ParametricIntegrand::from_graph(
        &graph,
        (0..graph.powers().len())
            .map(|index| symbol!(format!("rank_five_profile::x{index}")))
            .collect(),
        symbol!("rank_five_profile::eps"),
        parse!("4-2*rank_five_profile::eps"),
    )
    .unwrap()
}
