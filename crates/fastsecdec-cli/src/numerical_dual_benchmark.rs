//! Explicit ignored scientific/performance control for four saved ggHH artifacts.
mod integral;
mod runtime_order;
use crate::{CliResult, artifact::Artifact, input};
use fastsecdec::{
    generation::ChartRecord,
    kernel::{KernelSet, PrecisionClass, ReplayPolicy, StabilitySettings},
    status::CoefficientComponent,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use symbolica::{
    atom::AtomCore,
    numerical_integration::{ContinuousGrid, MonteCarloRng, Sample},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    artifact: PathBuf,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Configuration {
    cases: Vec<Case>,
    pairs: Vec<[String; 2]>,
    non_pointwise_pairs: Vec<[String; 2]>,
    integral_points: u64,
    integral_shifts: u32,
    parameters: PathBuf,
    output: PathBuf,
    seed: u64,
    comparison_points: usize,
    batch_rows: usize,
    timing_batches: usize,
    relative_tolerance: f64,
    absolute_tolerance: f64,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            cases: Vec::new(),
            pairs: Vec::new(),
            non_pointwise_pairs: Vec::new(),
            integral_points: 1024,
            integral_shifts: 8,
            parameters: PathBuf::new(),
            output: PathBuf::new(),
            seed: 4917,
            comparison_points: 16,
            batch_rows: 256,
            timing_batches: 16,
            relative_tolerance: 1e-8,
            absolute_tolerance: 1e-12,
        }
    }
}

struct Loaded {
    name: String,
    kernels: KernelSet,
    charts: Vec<ChartRecord>,
    layout: Vec<(i32, CoefficientComponent)>,
    report: Value,
}

fn point(path: &Path) -> CliResult<BTreeMap<fastsecdec::Symbol, f64>> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Point {
        parameters: BTreeMap<String, toml::Value>,
    }
    let point: Point = toml::from_str(&fs::read_to_string(path)?)?;
    let expressions = crate::config::canonical_parameter_names(point.parameters)?
        .into_iter()
        .map(|(name, value)| Ok((input::symbol(&name)?, input::value_expression(&value)?)))
        .collect::<CliResult<BTreeMap<_, _>>>()?;
    feynkit_model::resolve_scalar_bindings(expressions)?
        .into_iter()
        .map(|(symbol, value)| {
            let value = value.evaluate(&HashMap::<fastsecdec::Atom, f64>::new())?;
            if !value.is_finite() {
                return Err("benchmark point must be finite".into());
            }
            Ok((symbol, value))
        })
        .collect()
}

fn sample_points(
    dimension: usize,
    count: usize,
    seed: u64,
    low: f64,
    high: f64,
) -> CliResult<Vec<Vec<f64>>> {
    if dimension == 0 {
        return Ok(vec![vec![]; count]);
    }
    let mut grid = ContinuousGrid::<f64>::new(dimension, 1, usize::MAX, None, false)?;
    let mut rng = MonteCarloRng::new(seed, 0);
    let mut sample = Sample::new();
    (0..count)
        .map(|_| {
            grid.sample(&mut rng, &mut sample);
            let Sample::Continuous(_, point) = &sample else {
                return Err("expected native continuous sample".into());
            };
            Ok(point
                .iter()
                .map(|value| low + (high - low) * value)
                .collect())
        })
        .collect()
}

fn load(case: Case) -> CliResult<Loaded> {
    let started = Instant::now();
    let (artifact, kernels) = Artifact::load(&case.artifact)?;
    let elapsed = started.elapsed().as_secs_f64();
    let charts = kernels
        .generation_metadata()
        .ok_or("benchmark artifact has no chart metadata")?
        .charts()
        .to_vec();
    let layout = kernels
        .orders()
        .iter()
        .copied()
        .zip(kernels.components().iter().copied())
        .collect();
    let report = json!({"name":case.name,"artifact":crate::artifact::relative_display(&case.artifact),
        "kernel_content_id":kernels.content_id(),"loading_seconds":elapsed,
        "generation":artifact.generation,"generation_timings":artifact.generation_timings,
        "kernel_sectors":kernels.sectors().len(),"source_charts":charts.len(),
        "evaluator_statistics":kernels.sectors().iter().map(|sector| sector.statistics()).collect::<Vec<_>>()});
    Ok(Loaded {
        name: case.name,
        kernels,
        charts,
        layout,
        report,
    })
}

fn bind(loaded: &mut Loaded, parameters: &BTreeMap<fastsecdec::Symbol, f64>) -> CliResult<()> {
    let started = Instant::now();
    loaded.kernels.bind_parameters(parameters)?;
    let elapsed = started.elapsed().as_secs_f64();
    loaded.report["binding_seconds"] = json!(elapsed);
    loaded.report["loading_and_binding_seconds"] =
        json!(loaded.report["loading_seconds"].as_f64().unwrap() + elapsed);
    loaded.report["kernel_content_id"] = json!(loaded.kernels.content_id());
    Ok(())
}

fn chart_sum(loaded: &mut Loaded, points: &[Vec<f64>]) -> CliResult<Vec<f64>> {
    if loaded.charts.len() != points.len() {
        return Err("chart point count mismatch".into());
    }
    let mut multiplicities = BTreeMap::<usize, usize>::new();
    for chart in &loaded.charts {
        if let Some(sector) = chart.kernel_sector() {
            *multiplicities.entry(sector).or_default() += 1;
        }
    }
    let mut total = loaded.kernels.exact_coefficients().to_vec();
    for (chart, point) in loaded.charts.iter().zip(points) {
        let Some(sector) = chart.kernel_sector() else {
            continue;
        };
        let permutation = chart.representative_permutation();
        if permutation.len() != point.len() {
            return Err("chart permutation dimension mismatch".into());
        }
        let mut representative = vec![0.0; point.len()];
        let mut assigned = vec![false; point.len()];
        for (source, target) in permutation.iter().copied().enumerate() {
            if target >= point.len() || assigned[target] {
                return Err("invalid chart permutation".into());
            }
            representative[target] = point[source];
            assigned[target] = true;
        }
        let mut values = vec![0.0; total.len()];
        loaded.kernels.sectors_mut()[sector].evaluate(&representative, &mut values)?;
        for (total, value) in total.iter_mut().zip(values) {
            *total += value / multiplicities[&sector] as f64;
        }
    }
    Ok(total)
}

fn compare(
    left: &mut Loaded,
    right: &mut Loaded,
    configuration: &Configuration,
) -> CliResult<Value> {
    if left.charts.len() != right.charts.len() {
        return Err("source chart count differs between lanes".into());
    }
    for (a, b) in left.charts.iter().zip(&right.charts) {
        if a.source_index() != b.source_index()
            || a.geometry() != b.geometry()
            || a.coordinates().images() != b.coordinates().images()
            || a.coordinates().measure_jacobian() != b.coordinates().measure_jacobian()
        {
            return Err(format!(
                "source chart {} has incompatible native geometry",
                a.source_index()
            )
            .into());
        }
    }
    let layout = left
        .layout
        .iter()
        .chain(&right.layout)
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let points = left
        .charts
        .iter()
        .map(|chart| {
            sample_points(
                chart.coordinates().target_parameters().len(),
                configuration.comparison_points,
                configuration.seed.wrapping_add(chart.source_index() as u64),
                0.15,
                0.85,
            )
        })
        .collect::<CliResult<Vec<_>>>()?;
    let mut rows = Vec::new();
    let mut maximum_relative = 0.0f64;
    let mut maximum_absolute = 0.0f64;
    let mut pointwise_agreement = true;
    for index in 0..configuration.comparison_points + 3 {
        let chart_points = points
            .iter()
            .enumerate()
            .map(|(chart, points)| {
                if index < configuration.comparison_points {
                    points[index].clone()
                } else {
                    vec![
                        [0.01, 0.2, 0.8][index - configuration.comparison_points];
                        left.charts[chart].coordinates().target_parameters().len()
                    ]
                }
            })
            .collect::<Vec<_>>();
        let a = chart_sum(left, &chart_points)?;
        let b = chart_sum(right, &chart_points)?;
        let a = left
            .layout
            .iter()
            .copied()
            .zip(a)
            .collect::<BTreeMap<_, _>>();
        let b = right
            .layout
            .iter()
            .copied()
            .zip(b)
            .collect::<BTreeMap<_, _>>();
        let mut av = Vec::new();
        let mut bv = Vec::new();
        for key in &layout {
            let a = a.get(key).copied().unwrap_or(0.0);
            let b = b.get(key).copied().unwrap_or(0.0);
            let difference = (a - b).abs();
            let scale = a.abs().max(b.abs());
            maximum_absolute = maximum_absolute.max(difference);
            maximum_relative =
                maximum_relative.max(difference / scale.max(configuration.absolute_tolerance));
            if !a.is_finite() || !b.is_finite() {
                return Err(format!(
                    "{} vs {}: nonfinite point{index}, component{key:?}",
                    left.name, right.name
                )
                .into());
            }
            pointwise_agreement &= difference
                <= configuration.absolute_tolerance + configuration.relative_tolerance * scale;
            av.push(a);
            bv.push(b);
        }
        rows.push(json!({"chart_points":chart_points,"reference":av,"candidate":bv}));
    }
    let integral = if configuration
        .non_pointwise_pairs
        .iter()
        .any(|pair| pair[0] == left.name && pair[1] == right.name)
    {
        Some(integral::compare(left, right, configuration)?)
    } else {
        None
    };
    let agreement = integral
        .as_ref()
        .map_or(pointwise_agreement, |value| value["agreement"] == true);
    Ok(
        json!({"reference":left.name,"candidate":right.name,"layout":layout,"geometry_equal":true,
        "pointwise_agreement":pointwise_agreement,"paired_native_qmc":integral,"agreement":agreement,
        "source_chart_normalization":"representative permutation, divide by chart multiplicity; global exact offsets once",
        "maximum_absolute_difference":maximum_absolute,"maximum_relative_difference":maximum_relative,"points":rows}),
    )
}

fn timings(loaded: &mut Loaded, configuration: &Configuration) -> CliResult<Value> {
    let scalar = configuration.batch_rows == 1;
    let entrypoint = if scalar {
        "evaluate_weighted"
    } else {
        "evaluate_weighted_batch"
    };
    let mut stability = StabilitySettings::default();
    stability.levels[0].minimum_effective_distance = 1e-100;
    stability.levels[1].minimum_effective_distance = 1e-200;
    for level in &mut stability.levels {
        level.escalate_for_large_weight_threshold = None;
        level.power_thresholds.clear();
    }
    loaded.kernels.set_stability_settings(&stability)?;
    let mut sectors = Vec::new();
    let mut all_calls = 0u64;
    let mut all_ns = 0u64;
    for sector in 0..loaded.kernels.sectors().len() {
        let mut context = loaded
            .kernels
            .evaluation_context(sector, ReplayPolicy::default())?;
        let points = sample_points(
            context.dimension(),
            configuration.batch_rows,
            configuration.seed.wrapping_add(sector as u64),
            0.2,
            0.8,
        )?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let weights = vec![1.0; configuration.batch_rows];
        let mut output = vec![0.0; configuration.batch_rows * context.output_count()];
        for _ in 0..4 {
            if scalar {
                context.evaluate_weighted(&points, 1.0, &mut output)?;
            } else {
                context
                    .evaluate_weighted_batch(&points, &weights, &mut output)
                    .map_err(|failure| failure.error)?;
            }
        }
        let before = context.evaluation_metrics();
        let wall = Instant::now();
        let mut sample_ns = Vec::new();
        for _ in 0..configuration.timing_batches {
            let before = context.evaluation_metrics();
            let primary_f64 = if scalar {
                let report = context.evaluate_weighted(&points, 1.0, &mut output)?;
                report.precision.class == PrecisionClass::F64 && !report.replayed
            } else {
                let reports = context
                    .evaluate_weighted_batch(&points, &weights, &mut output)
                    .map_err(|failure| failure.error)?;
                reports.len() == configuration.batch_rows
                    && reports.iter().all(|report| {
                        report.precision.class == PrecisionClass::F64 && !report.replayed
                    })
            };
            if !primary_f64 {
                return Err(
                    "benchmark requested pure primary f64 but a sample used another route".into(),
                );
            }
            let delta = context.evaluation_metrics().since(before);
            if delta.f64.calls != configuration.batch_rows as u64
                || delta.double_float.calls != 0
                || delta.arbitrary.calls != 0
                || (scalar && (delta.f64.matrix_invocations != 0 || delta.f64.matrix_points != 0))
            {
                return Err(
                    "benchmark native evaluator call accounting differs from primary rows".into(),
                );
            }
            sample_ns.push(delta.f64.nanoseconds as f64 / delta.f64.calls as f64);
        }
        let wall_seconds = wall.elapsed().as_secs_f64();
        let delta = context.evaluation_metrics().since(before);
        sample_ns.sort_by(f64::total_cmp);
        all_calls += delta.f64.calls;
        all_ns += delta.f64.nanoseconds;
        sectors.push(json!({"sector":sector,"dimension":context.dimension(),"worker_wall_seconds":wall_seconds,
            "native_timings":delta,"mean_f64_ns_per_point":delta.f64.nanoseconds as f64/delta.f64.calls as f64,
            "minimum_invocation_ns_per_point":sample_ns[0],"median_invocation_ns_per_point":sample_ns[sample_ns.len()/2],
            "p95_invocation_ns_per_point":sample_ns[(sample_ns.len()-1)*95/100]}));
    }
    Ok(
        json!({"entrypoint":entrypoint,"benchmark_only_stability":stability,"batch_rows":configuration.batch_rows,
        "timing_batches":configuration.timing_batches,"warmup_batches":4,"coordinate_interval":[0.2,0.8],
        "mean_f64_ns_per_point":(all_calls>0).then_some(all_ns as f64/all_calls as f64),"sectors":sectors}),
    )
}

pub(super) fn run(path: &Path) -> CliResult<Value> {
    let configuration: Configuration = serde_json::from_slice(&fs::read(path)?)?;
    if configuration.cases.len() != 4
        || configuration.batch_rows == 0
        || configuration.timing_batches == 0
        || configuration.comparison_points == 0
        || configuration.integral_shifts < 2
        || configuration.integral_points < 1024
        || !configuration.integral_points.is_power_of_two()
        || configuration.absolute_tolerance <= 0.0
        || configuration.relative_tolerance <= 0.0
    {
        return Err(
            "benchmark requires four cases and positive samples, batches, tolerances".into(),
        );
    }
    // Restore every native symbol and its attributes before parsing any point
    // names. Otherwise a first textual declaration creates an untyped symbol
    // that conflicts with a real runtime input in the saved native context.
    let mut loaded = configuration
        .cases
        .iter()
        .map(|case| {
            load(Case {
                name: case.name.clone(),
                artifact: case.artifact.clone(),
            })
        })
        .collect::<CliResult<Vec<_>>>()?;
    if loaded
        .iter()
        .map(|case| &case.name)
        .collect::<BTreeSet<_>>()
        .len()
        != 4
    {
        return Err("duplicate benchmark case name".into());
    }
    let point_started = Instant::now();
    let parameters = point(&configuration.parameters)?;
    let parameter_point_parsing_seconds = point_started.elapsed().as_secs_f64();
    for case in &mut loaded {
        bind(case, &parameters)?;
    }
    let mut comparisons = Vec::new();
    for pair in &configuration.pairs {
        let a = loaded
            .iter()
            .position(|case| case.name == pair[0])
            .ok_or("unknown benchmark reference")?;
        let b = loaded
            .iter()
            .position(|case| case.name == pair[1])
            .ok_or("unknown benchmark candidate")?;
        if a == b {
            return Err("benchmark cannot compare case with itself".into());
        }
        let report = if a < b {
            let (left, right) = loaded.split_at_mut(b);
            compare(&mut left[a], &mut right[0], &configuration)?
        } else {
            let (left, right) = loaded.split_at_mut(a);
            compare(&mut right[0], &mut left[b], &configuration)?
        };
        comparisons.push(report);
    }
    let mut cases = Vec::new();
    for loaded in &mut loaded {
        let timing = timings(loaded, &configuration)?;
        loaded.report["performance"] = timing;
        cases.push(loaded.report.clone());
    }
    let agreement = comparisons.iter().all(|value| value["agreement"] == true);
    let report = json!({"agreement":agreement,"comparisons":comparisons,"cases":cases,"seed":configuration.seed,
        "parameter_point_parsing_seconds":parameter_point_parsing_seconds,
        "scope":"paired source-chart Laurent vectors within each subtraction strategy; warm native primary f64 scalar or matrix timing, not integration throughput"});
    if !configuration.output.as_os_str().is_empty() {
        crate::artifact::atomic_write(&configuration.output, &serde_json::to_vec_pretty(&report)?)?;
    }
    if !agreement {
        return Err("native comparison failed; complete vectors and covariances were saved in the benchmark report".into());
    }
    Ok(report)
}

#[test]
#[ignore = "requires four explicitly generated artifacts and a runtime point"]
fn compare_saved_numerical_dual_artifacts() {
    let path = std::env::var("FASTSECDEC_DUAL_BENCHMARK_CONFIG")
        .expect("explicit benchmark configuration");
    run(Path::new(&path)).unwrap();
}

#[test]
fn paired_source_charts_retain_symmetry_multiplicity_and_native_covariance() {
    use fastsecdec::{
        generation::{GenerationMode, GenerationOptions, generate},
        kernel::{CompilationSettings, EvaluatorBackend, PrecisionPolicy},
        parametric::{
            FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
        },
    };
    use std::ops::ControlFlow;
    use symbolica::{atom::Atom, parse, symbol};
    let input = ParametricIntegrand::new(
        vec![
            symbol!("dual_bench_probe::x"),
            symbol!("dual_bench_probe::y"),
        ],
        symbol!("dual_bench_probe::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("dual_bench_probe::x+dual_bench_probe::y"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let build = |mode, name: &str| {
        let generated = generate(
            &input,
            &GenerationOptions {
                mode,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let kernels = generated
            .compile_with_settings_parameters_and_progress(
                PrecisionPolicy::default(),
                &[],
                CompilationSettings {
                    backend: EvaluatorBackend::Eager,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        Loaded {
            name: name.into(),
            charts: generated.metadata().charts().to_vec(),
            layout: kernels
                .orders()
                .iter()
                .copied()
                .zip(kernels.components().iter().copied())
                .collect(),
            kernels,
            report: Value::Null,
        }
    };
    let mut a = build(GenerationMode::Symbolic, "symbolic");
    let mut b = build(GenerationMode::NumericalDual, "dual");
    assert_eq!(a.charts.len(), 2);
    assert_eq!(a.kernels.sectors().len(), 1);
    assert_eq!(b.kernels.sectors().len(), 2);
    assert_eq!(a.layout, [(0, CoefficientComponent::Real)]);
    let layout = [
        (0, CoefficientComponent::Real),
        (0, CoefficientComponent::Imag),
    ];
    assert_eq!(b.layout, layout);
    let mut configuration = Configuration {
        comparison_points: 3,
        batch_rows: 8,
        timing_batches: 2,
        non_pointwise_pairs: vec![["symbolic".into(), "dual".into()]],
        ..Default::default()
    };
    let comparison = compare(&mut a, &mut b, &configuration).unwrap();
    assert_eq!(comparison["pointwise_agreement"], true);
    assert_eq!(comparison["agreement"], true);
    let paired = &comparison["paired_native_qmc"];
    assert_eq!(paired["channel_layout"]["reference"], json!(layout));
    assert_eq!(paired["channel_layout"]["candidate"], json!(layout));
    let estimate: fastsecdec::integration::VectorEstimate =
        serde_json::from_value(paired["native_full_vector_estimate"].clone()).unwrap();
    estimate.validate().unwrap();
    // Both lanes occupy the physical (real, imaginary) union: the proven-real
    // symbolic lane supplies an exact zero imaginary channel. Retain every
    // within-lane and cross-lane covariance entry instead of projecting it out.
    let width = layout.len() * 2;
    assert_eq!(estimate.mean.len(), width);
    assert_eq!(estimate.covariance_of_mean.len(), width * width);
    for index in [0, 2] {
        assert!(
            (estimate.mean[index] - 2.0 * std::f64::consts::LN_2).abs()
                <= 1e-10 + 6.0 * estimate.standard_error[index]
        );
    }
    for row in 0..width {
        for column in 0..width {
            let covariance = estimate.covariance_of_mean[row * width + column];
            if row % 2 == 1 || column % 2 == 1 {
                assert_eq!(covariance, 0.0);
            } else {
                let variance = estimate.covariance_of_mean[0];
                assert!(variance > 0.0);
                assert!((covariance - variance).abs() <= 1e-24 + 1e-8 * variance);
            }
        }
    }
    for index in [1, 3] {
        assert_eq!(estimate.mean[index], 0.0);
        assert_eq!(estimate.standard_error[index], 0.0);
    }
    let timing = timings(&mut b, &configuration).unwrap();
    assert_eq!(timing["entrypoint"], "evaluate_weighted_batch");
    assert_eq!(timing["sectors"].as_array().unwrap().len(), 2);
    for sector in timing["sectors"].as_array().unwrap() {
        assert_eq!(sector["native_timings"]["f64"]["calls"], 16);
    }
    configuration.batch_rows = 1;
    let timing = timings(&mut b, &configuration).unwrap();
    assert_eq!(timing["entrypoint"], "evaluate_weighted");
    for sector in timing["sectors"].as_array().unwrap() {
        assert_eq!(sector["native_timings"]["f64"]["calls"], 2);
        assert_eq!(sector["native_timings"]["f64"]["matrix_invocations"], 0);
        assert_eq!(sector["native_timings"]["f64"]["matrix_points"], 0);
    }
}
