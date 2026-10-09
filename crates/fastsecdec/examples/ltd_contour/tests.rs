use super::*;
use fastsecdec::{Atom, AtomCore, IntegralFamily, Kinematics, parametric::ParametricIntegrand};
use symbolica::{domains::float::Complex, parse, symbol};

#[test]
fn exported_native_measure_preserves_the_declared_regulator_identity() {
    let epsilon = symbol!("ltd::eps");
    let parsed_regulator = Atom::parse(
        "ltd::eps",
        "feynkit_graph",
        symbolica::parser::ParseSettings::default(),
    )
    .unwrap();
    assert_eq!(parsed_regulator, Atom::var(epsilon));
    for loops in [1, 2, 3] {
        let native = import::measure(loops, Atom::var(epsilon));
        let restored = Atom::parse(
            &native.to_canonical_string(),
            "feynkit_graph",
            symbolica::parser::ParseSettings::default(),
        )
        .unwrap();
        assert_eq!(restored, native);
        assert!(
            restored
                .get_all_symbols(false)
                .into_iter()
                .all(|symbol| { symbol == epsilon || symbol == symbolica::atom::Symbol::PI })
        );
        assert_eq!(
            restored.replace(epsilon).with(0),
            import::measure(loops, Atom::Zero)
        );
    }
}

#[test]
fn native_graphs_match_every_archived_denominator_and_symanzik_polynomial() {
    let records = records::records().unwrap();
    assert_eq!(records.len(), 4);
    assert!(records.iter().all(|r| r.n_loops <= 3));
    for record in records {
        let imported = import::import(&record).unwrap();
        assert_eq!(
            imported.summary.denominator_matches,
            match record.name.as_str() {
                "2L6P.a.I" => 9,
                "3L4P.K1" => 10,
                _ => 7,
            }
        );
        assert!(imported.summary.native_u_terms > 0 && imported.summary.native_f_terms > 0);
        assert!(!imported.summary.generation_performed);
        let directory = tempfile::tempdir().unwrap();
        export::write(directory.path(), &record, &imported).unwrap();
    }
}

#[test]
fn archive_perturbations_are_rejected_without_fabricating_a_reference() {
    let mut records = records::records().unwrap();
    let record = records.iter_mut().find(|r| r.name == "2L4P.b.K1*").unwrap();
    assert_eq!(record.analytical_result_real, 0.);
    assert_eq!(record.analytical_result_imag, 0.);
    assert!(
        record
            .references()
            .iter()
            .all(|reference| reference.real != 0. && reference.imaginary != 0.)
    );
    record.loop_lines[0].propagators[0].q[0] += 0.01;
    assert!(import::import(record).is_err());
}

#[test]
fn native_one_loop_master_fixes_the_measure_phase_and_scale() {
    // A zero-momentum equal-mass triangle is finite: in FastSecDec's normalized
    // measure its finite coefficient is -1/2. OneLOop independently supplies it.
    let k = parse!("ltd_measure::k");
    let kin = Kinematics::in_dimension(&parse!("ltd_measure::D"))
        .unwrap()
        .with_momenta([k.clone()])
        .unwrap();
    let denominator = kin.scalar_product(&k, &k).unwrap() - 1;
    let family = IntegralFamily::new(vec![k], vec![], vec![denominator], &kin).unwrap();
    let epsilon = symbol!("ltd_measure::eps");
    let input = ParametricIntegrand::from_family(
        &family,
        &[3],
        Atom::one(),
        vec![symbol!("ltd_measure::x")],
        epsilon,
        parse!("4-2*ltd_measure::eps"),
    )
    .unwrap();
    let generated = fastsecdec::generation::generate(
        &input,
        &fastsecdec::generation::GenerationOptions::default(),
        |_| std::ops::ControlFlow::Continue(()),
    )
    .unwrap();
    let kernels = generated.compile().unwrap();
    assert!(kernels.sectors().is_empty());
    assert_eq!(kernels.orders(), &[0]);
    assert_eq!(kernels.exact_coefficients(), &[-0.5]);
    let normalized = Atom::num(
        symbolica::domains::rational::Rational::try_from(kernels.exact_coefficients()[0]).unwrap(),
    );
    let multiplier = import::measure(1, Atom::Zero);
    let value = (&multiplier * normalized)
        .evaluate(&std::collections::HashMap::<Atom, Complex<f64>>::new())
        .unwrap();
    let expected = Complex::new(0., -1. / (32. * std::f64::consts::PI.powi(2)));
    assert!((value - expected).im.abs() < 1e-16 && value.re.abs() < 1e-16);
    // OneLOop documents a larger stack for lazy triangle construction.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let arguments = [0., 0., 0., 1., 1., 1., 1.].map(|x| Complex::new(x, 0.));
            let mut output = [Complex::new(0., 0.); 3];
            oneloop::evaluate_with_backend(
                oneloop::ScalarIntegral::C0,
                &arguments,
                &mut output,
                oneloop::EvaluationBackend::Expression,
            )
            .unwrap();
            assert!((output[0].re + 0.5).abs() < 1e-12 && output[0].im.abs() < 1e-12);
            assert!(output[1].re.abs() < 1e-12 && output[2].re.abs() < 1e-12);
        })
        .unwrap()
        .join()
        .unwrap();
}
