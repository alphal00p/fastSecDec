//! Bounded native pullback evaluator compatibility diagnostic.
mod rank_five;

use super::{
    PrecisionPolicy, cancellation::Cancellation, precision, precision_cache::PrecisionCache,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{
        float::{Complex, ErrorPropagatingFloat, RealLike},
        rational::Rational,
    },
    evaluate::{
        ExpressionEvaluator, FunctionMap, FunctionRegistrationOptions, InliningPolicy,
        JITCompilationSettings,
    },
    id::{Pattern, Replacement},
    parse, symbol,
};

type Exact = ExpressionEvaluator<Complex<Rational>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Representation {
    Substituted,
    Aliases,
    InlineFunctions,
    RetainedFunctions,
}

#[derive(Serialize, Deserialize)]
struct Case {
    representation: Representation,
    builder_direct: bool,
    jit_direct: bool,
    complex: bool,
    file: String,
    source_laurent_seconds: f64,
    representation_seconds: f64,
    build_seconds: f64,
    jit_seconds: f64,
    bytes: usize,
    operations: String,
    interior: Vec<Vec<f64>>,
    weighted_boundary: Vec<f64>,
    precision_bits: u32,
}

#[test]
#[ignore = "native FunctionMap O2/MPFR/weighted/worker/cold-process diagnostic"]
fn native_function_map_pullback_probe() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes/function-map-2.26.4");
    match std::env::var("FASTSECDEC_FUNCTION_MAP_CHILD").as_deref() {
        Ok("write") => write_cases(&directory),
        Ok("read") => read_cases(&directory),
        _ => {
            std::fs::create_dir_all(&directory).unwrap();
            // The parent constructs no symbolic state. Writer and reader are
            // distinct fresh processes, avoiding inherited definitions/state.
            for mode in ["write", "read"] {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "kernel::function_map_probe::native_function_map_pullback_probe",
                        "--exact",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("FASTSECDEC_FUNCTION_MAP_CHILD", mode)
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

fn source(complex: bool) -> (Vec<Atom>, Vec<Symbol>, Vec<(Atom, Atom)>) {
    let parameters = vec![symbol!("alias_probe::t0"), symbol!("alias_probe::t1")];
    let x = parse!("alias_probe::x");
    let y = parse!("alias_probe::y");
    let scale = parse!("alias_probe::scale");
    let expression = parse!(
        "gamma(alias_probe::eps)*(log(1+alias_probe::x)/alias_probe::x + alias_probe::eps*alias_probe::x^2 + alias_probe::eps^2*alias_probe::y^3)"
    );
    let series = expression
        .series(symbol!("alias_probe::eps"), 0, 2)
        .unwrap();
    let coefficients = (-1..=2)
        .map(|order| {
            series.coefficient(Rational::from(order)).unwrap() * Atom::var(parameters[0]) * &scale
        })
        .collect();
    let scalar = if complex {
        Atom::num(Complex::new(Rational::from(2), Rational::from(3)))
    } else {
        Atom::one()
    };
    // Genuine sector-coordinate pullback x=t0, y=t0*t1, |Jacobian|=t0.
    let aliases = vec![
        (x, Atom::var(parameters[0])),
        (y, Atom::var(parameters[0]) * Atom::var(parameters[1])),
        (scale, scalar),
    ];
    (coefficients, parameters, aliases)
}

fn exact(
    source: &(Vec<Atom>, Vec<Symbol>, Vec<(Atom, Atom)>),
    representation: Representation,
    builder_direct: bool,
) -> (Exact, f64) {
    let (mut coefficients, parameters, aliases) = (*source).clone();
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let mut functions = FunctionMap::new();
    match representation {
        Representation::Substituted => {
            coefficients = coefficients
                .iter()
                .map(|coefficient| {
                    coefficient.replace_multiple(aliases.iter().map(|(from, to)| {
                        Replacement::new(
                            Pattern::Literal(from.clone()),
                            Pattern::Literal(to.clone()),
                        )
                    }))
                })
                .collect();
        }
        Representation::Aliases => functions.add_aliases(aliases).unwrap(),
        Representation::InlineFunctions | Representation::RetainedFunctions => {
            let args = [symbol!("alias_probe::a"), symbol!("alias_probe::b")];
            let definitions = [
                parse!("alias_probe::a"),
                parse!("alias_probe::a*alias_probe::b"),
                aliases[2].1.clone(),
            ];
            let policy = if matches!(representation, Representation::RetainedFunctions) {
                InliningPolicy::Never
            } else {
                InliningPolicy::Always
            };
            let mut replacements = Vec::new();
            for (index, ((name, _), body)) in aliases.into_iter().zip(definitions).enumerate() {
                let function = symbol!(format!("alias_probe::coordinate_{index}"));
                functions
                    .add_function_with_options(
                        function,
                        args.to_vec(),
                        body,
                        FunctionRegistrationOptions::new().inlining(policy),
                    )
                    .unwrap();
                replacements.push(Replacement::new(
                    Pattern::Literal(name),
                    Pattern::Literal(function.call(&variables)),
                ));
            }
            coefficients = coefficients
                .iter()
                .map(|coefficient| coefficient.replace_multiple(replacements.clone()))
                .collect();
        }
    }
    let started = Instant::now();
    let evaluator = Atom::evaluator_multiple(&coefficients, &variables)
        .function_map(functions)
        .direct_translation(builder_direct)
        .build()
        .unwrap();
    (evaluator, started.elapsed().as_secs_f64())
}

fn settings(direct: bool) -> JITCompilationSettings {
    JITCompilationSettings::default()
        .optimization_level(2)
        .direct_translation(direct)
}

fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 2e-12 * expected.abs().max(1.0),
            "{actual} != {expected}"
        );
    }
}

fn inspect(exact: &Exact, complex: bool, jit_direct: bool) -> (Vec<Vec<f64>>, Vec<f64>, u32, f64) {
    let points = [[0.2, 0.3], [0.9, 0.1], [0.01, 0.99]];
    let n = exact.get_output_len();
    let started = Instant::now();
    let (interior, compile_seconds) = if complex {
        let mut compiled = exact
            .jit_compile::<Complex<f64>>(settings(jit_direct))
            .unwrap();
        let seconds = started.elapsed().as_secs_f64();
        let mut eager = exact.clone().map_coeff(&|coefficient| {
            Complex::new(coefficient.re.to_f64(), coefficient.im.to_f64())
        });
        let mut conditioning = exact.clone().map_coeff(&|coefficient| {
            Complex::new(
                ErrorPropagatingFloat::new(coefficient.re.to_f64(), 15.0),
                ErrorPropagatingFloat::new(coefficient.im.to_f64(), 15.0),
            )
        });
        let mut result = Vec::new();
        for point in points {
            let point = point.map(|value| Complex::new(value, 0.0));
            let mut actual = vec![Complex::new(0.0, 0.0); n];
            let mut expected = actual.clone();
            compiled.evaluate(&point, &mut actual);
            eager.evaluate(&point, &mut expected);
            close(&flatten(&actual), &flatten(&expected));
            let check_point = point.map(|value| {
                Complex::new(
                    ErrorPropagatingFloat::new(value.re, 15.0),
                    ErrorPropagatingFloat::new(value.im, 15.0),
                )
            });
            let mut checked = vec![
                Complex::new(
                    ErrorPropagatingFloat::new(0.0, 15.0),
                    ErrorPropagatingFloat::new(0.0, 15.0)
                );
                n
            ];
            conditioning.evaluate(&check_point, &mut checked);
            close(
                &checked
                    .iter()
                    .flat_map(|value| [value.re.to_f64(), value.im.to_f64()])
                    .collect::<Vec<_>>(),
                &flatten(&expected),
            );
            result.push(flatten(&actual));
        }
        // Native compiled clones and retained bodies work on caller-owned workers.
        let mut cloned = compiled.clone();
        let worker = std::thread::spawn(move || {
            let mut out = vec![Complex::new(0.0, 0.0); n];
            cloned.evaluate(&[Complex::new(0.2, 0.0), Complex::new(0.3, 0.0)], &mut out);
            flatten(&out)
        })
        .join()
        .unwrap();
        close(&worker, &result[0]);
        (result, seconds)
    } else {
        let mut compiled = exact.jit_compile::<f64>(settings(jit_direct)).unwrap();
        let seconds = started.elapsed().as_secs_f64();
        let mut eager = exact
            .clone()
            .map_coeff(&|coefficient| coefficient.re.to_f64());
        let mut conditioning = exact
            .clone()
            .map_coeff(&|coefficient| ErrorPropagatingFloat::new(coefficient.re.to_f64(), 15.0));
        let mut result = Vec::new();
        for point in points {
            let mut actual = vec![0.0; n];
            let mut expected = actual.clone();
            compiled.evaluate(&point, &mut actual);
            eager.evaluate(&point, &mut expected);
            close(&actual, &expected);
            let mut checked = vec![ErrorPropagatingFloat::new(0.0, 15.0); n];
            conditioning.evaluate(
                &point.map(|value| ErrorPropagatingFloat::new(value, 15.0)),
                &mut checked,
            );
            close(
                &checked
                    .iter()
                    .map(|value| value.to_f64())
                    .collect::<Vec<_>>(),
                &expected,
            );
            result.push(actual);
        }
        let mut cloned = compiled.clone();
        let worker = std::thread::spawn(move || {
            let mut out = vec![0.0; n];
            cloned.evaluate(&[0.2, 0.3], &mut out);
            out
        })
        .join()
        .unwrap();
        close(&worker, &result[0]);
        (result, seconds)
    };
    let cancellation = Cancellation::new(2, Some(vec![vec![1, 1]]), 2).unwrap();
    let worker_exact = exact.clone();
    let (weighted, report) = std::thread::spawn(move || {
        let mut weighted = vec![0.0; n * if complex { 2 } else { 1 }];
        let report = if complex {
            precision::rescue_complex(
                &worker_exact,
                &mut PrecisionCache::default(),
                &[1e-80, 0.25],
                &mut weighted,
                &cancellation,
                &PrecisionPolicy::default(),
                1e80,
            )
            .unwrap()
        } else {
            precision::rescue(
                &worker_exact,
                &mut PrecisionCache::default(),
                &[1e-80, 0.25],
                &mut weighted,
                &cancellation,
                &PrecisionPolicy::default(),
                1e80,
            )
            .unwrap()
        };
        (weighted, report)
    })
    .join()
    .unwrap();
    let gamma = 0.5772156649015329_f64;
    let zeta3 = 1.202056903159594_f64;
    let pi = std::f64::consts::PI;
    let expected = [
        1.0,
        -gamma,
        gamma * gamma / 2.0 + pi * pi / 12.0,
        -gamma.powi(3) / 6.0 - gamma * pi * pi / 12.0 - zeta3 / 3.0,
    ];
    let expected = if complex {
        expected
            .into_iter()
            .flat_map(|value| [2.0 * value, 3.0 * value])
            .collect()
    } else {
        expected.to_vec()
    };
    close(&weighted, &expected);
    assert!(report.rescued && report.bits > 256);
    (interior, weighted, report.bits, compile_seconds)
}

fn flatten(values: &[Complex<f64>]) -> Vec<f64> {
    values
        .iter()
        .flat_map(|value| [value.re, value.im])
        .collect()
}

fn write_cases(directory: &Path) {
    let mut cases = Vec::new();
    for complex in [false, true] {
        let source_started = Instant::now();
        let source = source(complex);
        let source_laurent_seconds = source_started.elapsed().as_secs_f64();
        let mut baseline: Option<Vec<Vec<f64>>> = None;
        for representation in [
            Representation::Substituted,
            Representation::Aliases,
            Representation::InlineFunctions,
            Representation::RetainedFunctions,
        ] {
            for builder_direct in [false, true] {
                for jit_direct in [false, true] {
                    let started = Instant::now();
                    let (evaluator, build_seconds) = exact(&source, representation, builder_direct);
                    let representation_seconds = started.elapsed().as_secs_f64() - build_seconds;
                    let (interior, weighted_boundary, precision_bits, jit_seconds) =
                        inspect(&evaluator, complex, jit_direct);
                    if let Some(expected) = &baseline {
                        for (actual, expected) in interior.iter().zip(expected) {
                            close(actual, expected);
                        }
                    } else {
                        baseline = Some(interior.clone());
                    }
                    let bytes =
                        bincode::encode_to_vec(&evaluator, bincode::config::standard()).unwrap();
                    let file = format!("case-{}.bin", cases.len());
                    std::fs::write(directory.join(&file), &bytes).unwrap();
                    cases.push(Case {
                        representation,
                        builder_direct,
                        jit_direct,
                        complex,
                        file,
                        source_laurent_seconds,
                        representation_seconds,
                        build_seconds,
                        jit_seconds,
                        bytes: bytes.len(),
                        operations: format!("{:?}", evaluator.count_operations()),
                        interior,
                        weighted_boundary,
                        precision_bits,
                    });
                }
            }
        }
    }
    std::fs::write(
        directory.join("cases.json"),
        serde_json::to_vec_pretty(&cases).unwrap(),
    )
    .unwrap();
}

fn read_cases(directory: &Path) {
    // Trigger registered native builtins before decoding callback-bearing IR.
    let _ = symbol!("alias_probe::cold_state");
    let cases: Vec<Case> =
        serde_json::from_slice(&std::fs::read(directory.join("cases.json")).unwrap()).unwrap();
    for case in &cases {
        let bytes = std::fs::read(directory.join(&case.file)).unwrap();
        let (exact, consumed): (Exact, usize) =
            bincode::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(consumed, bytes.len());
        let (interior, weighted, bits, _) = inspect(&exact, case.complex, case.jit_direct);
        for (actual, expected) in interior.iter().zip(&case.interior) {
            close(actual, expected);
        }
        close(&weighted, &case.weighted_boundary);
        assert_eq!(bits, case.precision_bits);
    }
    println!(
        "{} native evaluator cases survived fresh-process O2/MPFR/weighted reconstruction",
        cases.len()
    );
}
