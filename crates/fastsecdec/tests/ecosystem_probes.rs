//! Executable third checks for capabilities found in API and source audits.
//! These prevent introducing parallel CAS or evaluator implementations.

use symbolica::{
    atom::{Atom, AtomCore},
    evaluate::{BatchEvaluator, JITCompilationSettings},
    parse, symbol,
};

#[test]
fn exact_polynomial_operations_stay_in_symbolica() {
    let x = symbol!("x");
    let polynomial = parse!("(x+y)^3-(x-y)^3");
    assert_eq!(polynomial.expand(), parse!("6*x^2*y+2*y^3"));
    assert_eq!(polynomial.derivative(x).expand(), parse!("12*x*y"));
    assert!((polynomial.factor() - polynomial).expand().is_zero());
    let gamma = parse!("gamma(eps)").series(symbol!("eps"), 0, 0).unwrap();
    assert_eq!(gamma.to_atom(), parse!("1/eps-euler_gamma"));
}

#[test]
fn native_polynomial_collection_preserves_factored_input_and_signed_valuations() {
    let variables = [parse!("x"), parse!("y")];
    for expression in [parse!("(x+y)^3-(x-y)^3"), parse!("(x+y)*(x+2*y)^2")] {
        assert_eq!(
            expression.to_polynomial_in_vars::<u32>(&variables),
            expression.expand().to_polynomial_in_vars::<u32>(&variables),
        );
    }
    let mapped = parse!("x^(-3)*(1+y)^2+x^(-2)*y");
    let residual = mapped
        .to_polynomial_in_vars::<i32>(&variables)
        .mul_exp(&[3, 0]);
    assert_eq!(
        residual,
        parse!("(1+y)^2+x*y").to_polynomial_in_vars::<i32>(&variables),
    );
}

#[test]
fn symjit_o2_runs_and_roundtrips_portable_ir() {
    let evaluator = parse!("x^2+2*y")
        .evaluator(&[parse!("x"), parse!("y")])
        .build()
        .unwrap();
    let mut compiled = evaluator
        .jit_compile::<f64>(JITCompilationSettings::default().optimization_level(2))
        .unwrap();
    let mut output = [0.0];
    compiled.evaluate(&[3.0, 4.0], &mut output);
    assert_eq!(output, [17.0]);
    let serialized = serde_json::to_vec(&compiled).unwrap();
    let mut restored: symbolica::evaluate::JITCompiledEvaluator<f64> =
        serde_json::from_slice(&serialized).unwrap();
    restored.evaluate(&[5.0, 1.0], &mut output);
    assert_eq!(output, [27.0]);
    let mut batch_output = [0.0; 2];
    restored
        .evaluate_batch(2, &[3.0, 4.0, 5.0, 1.0], &mut batch_output)
        .unwrap();
    assert_eq!(batch_output, [17.0, 27.0]);
    assert!(!restored.as_bytes().is_empty());
}

#[test]
fn family_completeness_and_isp_rewriting_are_native() {
    use feynkit_graph::IntegralFamily;
    use feynkit_kinematics::Kinematics;
    let (k, p) = (parse!("k"), parse!("p"));
    let kin = Kinematics::new()
        .with_momenta([k.clone(), p.clone()])
        .unwrap()
        .with_mass_squared(&p, parse!("s"))
        .unwrap();
    let kk = kin.scalar_product(&k, &k).unwrap();
    let kp = kin.scalar_product(&k, &p).unwrap();
    let family = IntegralFamily::new(vec![k], vec![p], vec![kk], &kin).unwrap();
    let complete = family.complete(&[]).unwrap();
    assert!(complete.is_complete());
    let labels = [parse!("d0"), parse!("d1")];
    let transformed = complete.rewrite_numerator(&kp, &labels).unwrap();
    assert!(!transformed.is_zero());
    let restored = transformed.replace_multiple(labels.iter().zip(complete.denominators()).map(
        |(left, right)| symbolica::id::Replacement::new(left.to_pattern(), right.to_pattern()),
    ));
    assert_eq!((restored - kp).expand(), Atom::Zero);
}
