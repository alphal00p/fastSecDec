use super::*;
use std::{
    sync::{
        Barrier,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};
use symbolica::{parse, symbol};

fn scalar(polynomial: &Atom, inputs: &[Symbol]) -> ExactProgram {
    polynomial
        .evaluator(&inputs.iter().copied().map(Atom::var).collect::<Vec<_>>())
        .optimization_settings(CompilationSettings::default().native())
        .build()
        .unwrap()
}

fn evaluate(program: &ExactProgram, inputs: &[f64], count: usize) -> Vec<f64> {
    let mut evaluator = program.clone().map_coeff(&|value| value.re.to_f64());
    let mut output = vec![0.0; count];
    evaluator.evaluate(inputs, &mut output);
    output
}

fn bytes(program: &ExactProgram) -> Vec<u8> {
    bincode::serde::encode_to_vec(program, bincode::config::standard()).unwrap()
}

#[test]
fn compact_native_bodies_are_part_of_the_source_and_jet_cache_key() {
    use crate::contour::ContourDefinitions;
    let x = symbol!("compact_cache::x");
    let polynomial = (1..=24)
        .map(|power| Atom::var(x).pow(Atom::num(power)))
        .sum::<Atom>();
    let (definitions, calls) = ContourDefinitions::coefficients(&[x], &[polynomial]).unwrap();
    assert_eq!(definitions.entries().len(), 1);
    let entry = &definitions.entries()[0];
    // A local native FunctionMap owns its exact body. Even independently
    // admitted definition owners sharing one handle cannot share cached IR.
    let changed = ContourDefinitions::from_parts(vec![(
        entry.function(),
        entry.parameters().to_vec(),
        entry.body() + Atom::one(),
    )])
    .unwrap();
    let cache = SourcePrograms::default();
    let shape = [vec![0], vec![1], vec![2]];
    let first = cache
        .jets_with_definitions(
            &calls[0],
            &[x],
            &shape,
            &[],
            CompilationSettings::default(),
            &Arc::new(definitions),
        )
        .unwrap();
    let second = cache
        .jets_with_definitions(
            &calls[0],
            &[x],
            &shape,
            &[],
            CompilationSettings::default(),
            &Arc::new(changed),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(cache.0.lock().unwrap().len(), 2);
    let a = evaluate(&first, &[0.25, 1., 0.], 3);
    let b = evaluate(&second, &[0.25, 1., 0.], 3);
    assert_eq!(b, vec![a[0] + 1., a[1], a[2]]);
}

#[test]
fn identical_concurrent_jets_share_one_native_owner_and_exact_ir() {
    let cache = SourcePrograms::default();
    let polynomial = parse!("native_cache::x^2+native_cache::x*native_cache::y+native_cache::x^3");
    let inputs = [symbol!("native_cache::x"), symbol!("native_cache::y")];
    let shape = vec![
        vec![0, 0],
        vec![1, 0],
        vec![0, 1],
        vec![2, 0],
        vec![1, 1],
        vec![3, 0],
        vec![2, 1],
        vec![3, 1],
    ];
    let barrier = Barrier::new(8);
    let programs = std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    cache
                        .jets(
                            &polynomial,
                            &inputs,
                            &shape,
                            &[],
                            CompilationSettings::default(),
                        )
                        .unwrap()
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(
        programs
            .iter()
            .all(|program| Arc::ptr_eq(program, &programs[0]))
    );
    let sources = cache.0.lock().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].jets.lock().unwrap().len(), 1);
    drop(sources);
    let serial = SourcePrograms::default()
        .jets(
            &polynomial,
            &inputs,
            &shape,
            &[],
            CompilationSettings::default(),
        )
        .unwrap();
    assert_eq!(bytes(&programs[0]), bytes(&serial));

    // Native map jets x=t, y=t*u at (t,u)=(0,2). Dividing the known t^2
    // monomial means selecting shifted coefficients, before any inversion.
    let arguments = [
        0., 1., 0., 0., 0., 0., 0., 0., 0., 2., 0., 0., 1., 0., 0., 0.,
    ];
    let values = evaluate(&programs[0], &arguments, shape.len());
    assert_eq!(values, [0., 0., 0., 3., 0., 1., 1., 0.]);
    // These are normalized Taylor coefficients: the t^3 coefficient is 1,
    // not the third derivative 6; the shifted residual is 1+u+t.
    assert_eq!([values[3], values[5], values[6]], [3., 1., 1.]);
}

#[test]
fn structural_zero_masks_remain_distinct_scientific_keys() {
    let cache = SourcePrograms::default();
    let polynomial = parse!("native_cache_zero::x+native_cache_zero::y");
    let inputs = [
        symbol!("native_cache_zero::x"),
        symbol!("native_cache_zero::y"),
    ];
    let shape = vec![vec![0], vec![1]];
    let full = cache
        .jets(
            &polynomial,
            &inputs,
            &shape,
            &[],
            CompilationSettings::default(),
        )
        .unwrap();
    let x_constant = cache
        .jets(
            &polynomial,
            &inputs,
            &shape,
            &[(0, 1)],
            CompilationSettings::default(),
        )
        .unwrap();
    let y_constant = cache
        .jets(
            &polynomial,
            &inputs,
            &shape,
            &[(1, 1)],
            CompilationSettings::default(),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&full, &x_constant));
    assert!(!Arc::ptr_eq(&x_constant, &y_constant));
    assert_eq!(evaluate(&full, &[2., 1., 3., 4.], 2), [5., 5.]);
    assert_eq!(evaluate(&x_constant, &[2., 0., 3., 4.], 2), [5., 4.]);
    assert_eq!(evaluate(&y_constant, &[2., 1., 3., 0.], 2), [5., 1.]);
    assert_eq!(evaluate(&full, &[2., 1., 3., 0.], 2), [5., 1.]);
}

#[test]
fn source_keys_preserve_input_order_and_compilation_settings() {
    let cache = SourcePrograms::default();
    let polynomial = parse!("native_cache_order::x+2*native_cache_order::y");
    let inputs = [
        symbol!("native_cache_order::x"),
        symbol!("native_cache_order::y"),
    ];
    let shape = vec![vec![0]];
    let settings = CompilationSettings::default();
    let first = cache
        .jets(&polynomial, &inputs, &shape, &[], settings)
        .unwrap();
    let reversed = cache
        .jets(&polynomial, &[inputs[1], inputs[0]], &shape, &[], settings)
        .unwrap();
    let different_settings = cache
        .jets(
            &polynomial,
            &inputs,
            &shape,
            &[],
            CompilationSettings {
                horner_iterations: 0,
                ..settings
            },
        )
        .unwrap();
    assert_eq!(cache.0.lock().unwrap().len(), 3);
    assert!(!Arc::ptr_eq(&first, &reversed));
    assert!(!Arc::ptr_eq(&first, &different_settings));
    assert_eq!(evaluate(&first, &[2., 3.], 1), [8.]);
    assert_eq!(evaluate(&reversed, &[3., 2.], 1), [8.]);
    assert_eq!(evaluate(&different_settings, &[2., 3.], 1), [8.]);
}

// Hold one initializer open while an unrelated real native cache request must
// finish. Channels bound every wait and always release the held initializer,
// including a failed independence assertion; no sleep-based timing benchmark.
fn while_pending(
    pending: &ProgramCell,
    value: ExactProgram,
    work: impl FnOnce() -> Result<Arc<ExactProgram>, KernelError> + Send,
) {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (completed_tx, completed_rx) = mpsc::channel();
    std::thread::scope(|scope| {
        let held = scope.spawn(move || {
            program(pending, || {
                entered_tx.send(()).unwrap();
                release_rx
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(|error| error.to_string())?;
                Ok(value)
            })
        });
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let other = scope.spawn(move || completed_tx.send(work()).unwrap());
        let completed = completed_rx.recv_timeout(Duration::from_secs(5));
        release_tx.send(()).unwrap();
        held.join().unwrap().unwrap();
        other.join().unwrap();
        completed
            .expect("unrelated key must finish while the first key is pending")
            .unwrap();
    });
}

#[test]
fn unrelated_sources_do_not_wait_for_an_initializing_source() {
    let cache = SourcePrograms::default();
    let x = symbol!("native_cache_parallel::x");
    let polynomial = Atom::var(x);
    let source = cache
        .source(&polynomial, &[x], CompilationSettings::default(), None)
        .unwrap();
    while_pending(&source.exact, scalar(&polynomial, &[x]), || {
        cache.jets(
            &(polynomial.clone() + Atom::one()),
            &[x],
            &[vec![0]],
            &[],
            CompilationSettings::default(),
        )
    });
}

#[test]
fn unrelated_jet_shapes_do_not_wait_for_an_initializing_jet() {
    let cache = SourcePrograms::default();
    let x = symbol!("native_cache_parallel_jet::x");
    let polynomial = Atom::var(x).pow(Atom::num(2));
    let source = cache
        .source(&polynomial, &[x], CompilationSettings::default(), None)
        .unwrap();
    program(&source.exact, || Ok(scalar(&polynomial, &[x]))).unwrap();
    let cell = Arc::new(ProgramCell::new());
    source
        .jets
        .lock()
        .unwrap()
        .insert((vec![vec![0]], vec![]), cell.clone());
    while_pending(&cell, scalar(&polynomial, &[x]), || {
        cache.jets(
            &polynomial,
            &[x],
            &[vec![0], vec![1]],
            &[],
            CompilationSettings::default(),
        )
    });
    assert_eq!(source.jets.lock().unwrap().len(), 2);
}

#[test]
fn failed_initialization_is_shared_without_poisoning_other_keys() {
    let cell = ProgramCell::new();
    let attempts = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let workers = (0..8).map(|_| scope.spawn(|| {
            let error = program(&cell, || {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err("native construction failed".into())
            }).unwrap_err();
            assert!(matches!(error, KernelError::Compilation(message) if message == "native construction failed"));
        })).collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    let cache = SourcePrograms::default();
    let x = symbol!("native_cache_error::x");
    let polynomial = parse!("native_cache_error::undefined_function(native_cache_error::x)");
    let first = cache
        .jets(
            &polynomial,
            &[x],
            &[vec![0]],
            &[],
            CompilationSettings::default(),
        )
        .unwrap_err();
    let second = cache
        .jets(
            &polynomial,
            &[x],
            &[vec![0]],
            &[],
            CompilationSettings::default(),
        )
        .unwrap_err();
    assert_eq!(first.to_string(), second.to_string());
    cache
        .jets(
            &Atom::var(x),
            &[x],
            &[vec![0]],
            &[],
            CompilationSettings::default(),
        )
        .unwrap();
}

#[test]
fn initializer_panic_does_not_poison_or_complete_the_cell() {
    let cell = ProgramCell::new();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        program(&cell, || {
            panic!("intentional native-cache initializer unwind")
        })
    }));
    assert!(panic.is_err());
    assert!(cell.get().is_none());
    let x = symbol!("native_cache_panic::x");
    let result = program(&cell, || Ok(scalar(&Atom::var(x), &[x]))).unwrap();
    assert_eq!(evaluate(&result, &[3.], 1), [3.]);
}
