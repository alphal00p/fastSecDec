//! An additional, explicit cube pullback of actual generated coefficients.
//! This measures evaluator representation, not alias-aware sector generation.
use super::*;
use crate::generation::{GeneratedSector, GenerationOptions, GenerationProgress, generate};
use std::{hint::black_box, ops::ControlFlow};

const FILTER: &str = "kernel::function_map_probe::rank_five::actual_rank_five_pullback";
const POINTS: usize = 16_384;
const REPEATS: usize = 7;
const TIMING_PASSES: usize = 64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Mode {
    Production,
    Pullback(Representation),
}

#[derive(Serialize, Deserialize)]
struct Measurement {
    repetition: usize,
    mode: Mode,
    builder_direct: bool,
    jit_direct: bool,
    representation_seconds: f64,
    builder_seconds: f64,
    jit_seconds: f64,
    scalar_seconds: f64,
    batch_seconds: f64,
    expression_bytes: usize,
    definition_bytes: usize,
    native_ir_bytes: usize,
    operations: String,
    max_scaled_difference: f64,
    precision_bits: u32,
    file: String,
    boundary_values: Vec<f64>,
    cancellation_degree: usize,
    cancellation_terms: Vec<Vec<usize>>,
}

#[derive(Serialize, Deserialize)]
struct Report {
    profile: String,
    symbolic_parameterization_seconds: f64,
    symbolic_generation_seconds: f64,
    symbolic_stage_seconds: std::collections::BTreeMap<String, f64>,
    all_sector_coefficient_bytes: Vec<usize>,
    selected_sector: usize,
    dimension: usize,
    orders: Vec<i32>,
    points: usize,
    repeats: usize,
    timing_passes: usize,
    evaluations_per_measurement: usize,
    rows: Vec<Measurement>,
}

#[test]
#[ignore = "release paired actual rank-five coefficient pullback benchmark"]
fn actual_rank_five_pullback() {
    let profile = if cfg!(debug_assertions) {
        "development"
    } else {
        "release"
    };
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../output/probes/function-map-rank-five-2.26.4-{profile}-r{REPEATS}-n{TIMING_PASSES}"
    ));
    match std::env::var("FASTSECDEC_RANK_FIVE_FUNCTION_MAP_CHILD").as_deref() {
        Ok("write") => write(&directory, profile),
        Ok("read") => read(&directory),
        _ => {
            std::fs::create_dir_all(&directory).unwrap();
            for mode in ["write", "read"] {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        FILTER,
                        "--exact",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("FASTSECDEC_RANK_FIVE_FUNCTION_MAP_CHILD", mode)
                    .output()
                    .unwrap();
                std::fs::write(directory.join(format!("{mode}.log")), &output.stdout).unwrap();
                std::fs::write(directory.join(format!("{mode}.stderr.log")), &output.stderr)
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{mode}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

fn build(sector: &GeneratedSector, mode: Mode, direct: bool) -> (Exact, usize, usize, f64, f64) {
    let started = Instant::now();
    let old = sector
        .parameters()
        .iter()
        .map(|p| Atom::var(*p))
        .collect::<Vec<_>>();
    let variables = if matches!(mode, Mode::Production) {
        old.clone()
    } else {
        (0..old.len())
            .map(|i| Atom::var(symbol!(format!("rank_five_alias::u{i}"))))
            .collect()
    };
    let mut coefficients = sector.coefficients().to_vec();
    let mut map = FunctionMap::new();
    let mut definitions = Vec::new();
    if let Mode::Pullback(representation) = mode {
        let aliases = old
            .iter()
            .zip(&variables)
            .map(|(from, to)| (from.clone(), to.pow(2)))
            .collect::<Vec<_>>();
        match representation {
            Representation::Substituted => {
                coefficients = coefficients
                    .iter()
                    .map(|c| {
                        c.replace_multiple(aliases.iter().map(|(from, to)| {
                            Replacement::new(
                                Pattern::Literal(from.clone()),
                                Pattern::Literal(to.clone()),
                            )
                        }))
                    })
                    .collect();
            }
            Representation::Aliases => {
                definitions = aliases.iter().map(|(_, body)| body.clone()).collect();
                map.add_aliases(aliases).unwrap();
            }
            Representation::InlineFunctions | Representation::RetainedFunctions => {
                let argument = symbol!("rank_five_alias::a");
                let policy = if matches!(representation, Representation::RetainedFunctions) {
                    InliningPolicy::Never
                } else {
                    InliningPolicy::Always
                };
                let mut replacements = Vec::new();
                for (axis, (from, variable)) in old.iter().zip(&variables).enumerate() {
                    let name = symbol!(format!("rank_five_alias::coordinate{axis}"));
                    let body = Atom::var(argument).pow(2);
                    definitions.push(body.clone());
                    map.add_function_with_options(
                        name,
                        vec![argument],
                        body,
                        FunctionRegistrationOptions::new().inlining(policy),
                    )
                    .unwrap();
                    replacements.push(Replacement::new(
                        Pattern::Literal(from.clone()),
                        Pattern::Literal(name.call(&[variable][..])),
                    ));
                }
                coefficients = coefficients
                    .iter()
                    .map(|c| c.replace_multiple(replacements.clone()))
                    .collect();
            }
        }
        let jacobian = variables.iter().fold(Atom::one(), |product, variable| {
            product * variable * Atom::num(2)
        });
        coefficients = coefficients.into_iter().map(|c| c * &jacobian).collect();
    }
    let expression_bytes = coefficients
        .iter()
        .map(|a| a.as_view().get_byte_size())
        .sum();
    let definition_bytes = definitions
        .iter()
        .map(|a| a.as_view().get_byte_size())
        .sum();
    let representation_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let exact = Atom::evaluator_multiple(&coefficients, &variables)
        .function_map(map)
        .direct_translation(direct)
        .build()
        .unwrap();
    (
        exact,
        expression_bytes,
        definition_bytes,
        representation_seconds,
        started.elapsed().as_secs_f64(),
    )
}

fn points(dimension: usize) -> Vec<f64> {
    (0..POINTS)
        .flat_map(|row| {
            (0..dimension).map(move |axis| {
                // Fixed interior benchmark inputs, unrelated to any integration rule.
                0.5 + 0.4 * (((row + 1) * (axis * 2 + 1) * 104729) % 65521) as f64 / 65521.0
            })
        })
        .collect()
}

fn jacobian(point: &[f64]) -> f64 {
    point.iter().map(|x| 2.0 * x).product()
}

fn difference(actual: &[f64], expected: &[f64]) -> f64 {
    assert_eq!(actual.len(), expected.len());
    let maximum = actual
        .iter()
        .zip(expected)
        .map(|(a, b)| {
            assert!(a.is_finite() && b.is_finite());
            (a - b).abs() / b.abs().max(1.0)
        })
        .fold(0.0, f64::max);
    assert!(maximum < 2e-10, "raw O2 evaluator disagreement {maximum}");
    maximum
}

fn boundary(
    exact: &Exact,
    mode: Mode,
    dimension: usize,
    degree: usize,
    terms: Vec<Vec<usize>>,
) -> (Vec<f64>, u32) {
    let mut point = vec![0.7; dimension];
    point[0] = 1e-20;
    let weight = if matches!(mode, Mode::Production) {
        point.iter_mut().for_each(|x| *x *= *x);
        1.0
    } else {
        jacobian(&point).recip()
    };
    let cancellation = Cancellation::new(degree, Some(terms), dimension).unwrap();
    let exact = exact.clone();
    let requirements = crate::kernel::evaluator::MappingRequirements::new(&exact).unwrap();
    std::thread::spawn(move || {
        let mut output = vec![0.0; exact.get_output_len()];
        let report = precision::rescue(
            &exact,
            &mut PrecisionCache::new(requirements),
            &point,
            &mut output,
            &cancellation,
            &PrecisionPolicy::default(),
            weight,
        )
        .unwrap();
        (output, report.bits)
    })
    .join()
    .unwrap()
}

fn check_numeric(exact: &Exact, params: &[f64], dimension: usize, expected: &[f64]) {
    let n = exact.get_output_len();
    let mut eager = exact.clone().map_coeff(&|c| c.re.to_f64());
    let mut conditioning = exact
        .clone()
        .map_coeff(&|c| ErrorPropagatingFloat::new(c.re.to_f64(), 15.0));
    for row in [0, POINTS / 3, POINTS / 2, POINTS - 1] {
        let point = &params[row * dimension..(row + 1) * dimension];
        let target = &expected[row * n..(row + 1) * n];
        let mut values = vec![0.0; n];
        eager.evaluate(point, &mut values);
        difference(&values, target);
        let mut checked = vec![ErrorPropagatingFloat::new(0.0, 15.0); n];
        conditioning.evaluate(
            &point
                .iter()
                .map(|x| ErrorPropagatingFloat::new(*x, 15.0))
                .collect::<Vec<_>>(),
            &mut checked,
        );
        difference(
            &checked.iter().map(|x| x.to_f64()).collect::<Vec<_>>(),
            target,
        );
    }
}

fn write(directory: &Path, profile: &str) {
    let start = Instant::now();
    let input = crate::generation::profiling::input();
    let parameterization_seconds = start.elapsed().as_secs_f64();
    let mut stages = std::collections::BTreeMap::<String, f64>::new();
    let start = Instant::now();
    let generated = generate(&input, &GenerationOptions::default(), |event| {
        if let GenerationProgress::PhaseTiming { phase, seconds } = event {
            *stages.entry(format!("{phase:?}")).or_default() += seconds;
        }
        assert!(
            start.elapsed().as_secs() < 180,
            "bounded native generation exceeded 180 seconds"
        );
        ControlFlow::Continue(())
    })
    .unwrap();
    let generation_seconds = start.elapsed().as_secs_f64();
    let sizes = generated
        .sectors()
        .iter()
        .map(|s| {
            s.coefficients()
                .iter()
                .map(|a| a.as_view().get_byte_size())
                .sum::<usize>()
        })
        .collect::<Vec<_>>();
    let selected = sizes
        .iter()
        .enumerate()
        .max_by_key(|(_, size)| *size)
        .unwrap()
        .0;
    let sector = &generated.sectors()[selected];
    let dimension = sector.dimension();
    assert!(dimension > 0 && sizes[selected] < 16 * 1024 * 1024);
    let params = points(dimension);
    let original_points = params.iter().map(|x| x * x).collect::<Vec<_>>();
    let (original, _, _, _, _) = build(sector, Mode::Production, true);
    let mut eager = original.clone().map_coeff(&|c| c.re.to_f64());
    let n = original.get_output_len();
    let mut expected_original = vec![0.0; POINTS * n];
    for (point, out) in original_points
        .chunks_exact(dimension)
        .zip(expected_original.chunks_exact_mut(n))
    {
        eager.evaluate(point, out);
    }
    let expected_mapped = expected_original
        .chunks_exact(n)
        .zip(params.chunks_exact(dimension))
        .flat_map(|(out, point)| out.iter().map(|x| x * jacobian(point)))
        .collect::<Vec<_>>();
    let original_terms = sector.cancellation_terms().to_vec();
    let (expected_boundary, _) = boundary(
        &original,
        Mode::Production,
        dimension,
        sector.cancellation_degree(),
        original_terms.clone(),
    );
    let mut modes = vec![(Mode::Production, true, false)];
    for representation in [
        Representation::Substituted,
        Representation::Aliases,
        Representation::InlineFunctions,
        Representation::RetainedFunctions,
    ] {
        for direct in [false, true] {
            for jit in [false, true] {
                modes.push((Mode::Pullback(representation), direct, jit));
            }
        }
    }
    let mut rows = Vec::new();
    for repetition in 0..REPEATS {
        // Rotate execution order so every representation is not always timed
        // after the same predecessor; all rows retain identical points/work.
        for index in 0..modes.len() {
            let (mode, direct, jit) = modes[(index + repetition * 5) % modes.len()];
            let (
                exact,
                expression_bytes,
                definition_bytes,
                representation_seconds,
                builder_seconds,
            ) = build(sector, mode, direct);
            let started = Instant::now();
            let mut compiled = exact.jit_compile::<f64>(settings(jit)).unwrap();
            let jit_seconds = started.elapsed().as_secs_f64();
            let (inputs, target) = if matches!(mode, Mode::Production) {
                (&original_points, &expected_original)
            } else {
                (&params, &expected_mapped)
            };
            let mut output = vec![0.0; POINTS * n];
            compiled.batch_evaluate(inputs, &mut output, POINTS); // warm native buffers
            let started = Instant::now();
            for _ in 0..TIMING_PASSES {
                for (point, out) in inputs
                    .chunks_exact(dimension)
                    .zip(output.chunks_exact_mut(n))
                {
                    compiled.evaluate(black_box(point), out);
                }
            }
            let scalar_seconds = started.elapsed().as_secs_f64();
            let mut maximum = difference(black_box(&output), target);
            let started = Instant::now();
            for _ in 0..TIMING_PASSES {
                compiled.batch_evaluate(black_box(inputs), &mut output, POINTS);
            }
            let batch_seconds = started.elapsed().as_secs_f64();
            maximum = maximum.max(difference(black_box(&output), target));
            check_numeric(&exact, inputs, dimension, target);
            let terms = original_terms
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|v| {
                            v * if matches!(mode, Mode::Production) {
                                1
                            } else {
                                2
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let degree = terms
                .iter()
                .map(|row| row.iter().sum::<usize>())
                .max()
                .unwrap_or(0);
            let (boundary_values, precision_bits) =
                boundary(&exact, mode, dimension, degree, terms.clone());
            close(&boundary_values, &expected_boundary);
            let bytes = bincode::encode_to_vec(&exact, bincode::config::standard()).unwrap();
            let file = format!("case-{}.bin", rows.len());
            std::fs::write(directory.join(&file), &bytes).unwrap();
            rows.push(Measurement {
                repetition,
                mode,
                builder_direct: direct,
                jit_direct: jit,
                representation_seconds,
                builder_seconds,
                jit_seconds,
                scalar_seconds,
                batch_seconds,
                expression_bytes,
                definition_bytes,
                native_ir_bytes: bytes.len(),
                operations: format!("{:?}", exact.count_operations()),
                max_scaled_difference: maximum,
                precision_bits,
                file,
                boundary_values,
                cancellation_degree: degree,
                cancellation_terms: terms,
            });
        }
    }
    let report = Report {
        profile: profile.into(),
        symbolic_parameterization_seconds: parameterization_seconds,
        symbolic_generation_seconds: generation_seconds,
        symbolic_stage_seconds: stages,
        all_sector_coefficient_bytes: sizes,
        selected_sector: selected,
        dimension,
        orders: generated.orders().to_vec(),
        points: POINTS,
        repeats: REPEATS,
        timing_passes: TIMING_PASSES,
        evaluations_per_measurement: POINTS * TIMING_PASSES,
        rows,
    };
    std::fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

fn read(directory: &Path) {
    let _ = symbol!("rank_five_alias::cold_state");
    let report: Report =
        serde_json::from_slice(&std::fs::read(directory.join("report.json")).unwrap()).unwrap();
    let params = points(report.dimension);
    let original = params.iter().map(|x| x * x).collect::<Vec<_>>();
    for row in &report.rows {
        let bytes = std::fs::read(directory.join(&row.file)).unwrap();
        let (exact, consumed): (Exact, usize) =
            bincode::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(consumed, bytes.len());
        let mut compiled = exact.jit_compile::<f64>(settings(row.jit_direct)).unwrap();
        let inputs = if matches!(row.mode, Mode::Production) {
            &original
        } else {
            &params
        };
        let mut expected = vec![0.0; POINTS * report.orders.len()];
        let mut actual = expected.clone();
        let mut eager = exact.clone().map_coeff(&|c| c.re.to_f64());
        for (point, output) in inputs
            .chunks_exact(report.dimension)
            .zip(expected.chunks_exact_mut(report.orders.len()))
        {
            eager.evaluate(point, output);
        }
        compiled.batch_evaluate(inputs, &mut actual, POINTS);
        difference(&actual, &expected);
        check_numeric(&exact, inputs, report.dimension, &expected);
        let (values, bits) = boundary(
            &exact,
            row.mode,
            report.dimension,
            row.cancellation_degree,
            row.cancellation_terms.clone(),
        );
        close(&values, &row.boundary_values);
        assert_eq!(bits, row.precision_bits);
    }
    println!(
        "{} real rank-five paired cases survived fresh-process reconstruction",
        report.rows.len()
    );
}
