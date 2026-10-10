use super::ProbeResult;
use serde_json::{Value, json};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    evaluate::{
        EvaluatorComposer, FunctionMap, FunctionRegistrationOptions, InliningPolicy,
        OptimizationSettings, Slot,
    },
    symbol,
};

pub(super) fn run() -> ProbeResult<Value> {
    let (a, x, t, root) = symbol!(
        "threshold_probe_root::a",
        "threshold_probe_root::x",
        "threshold_probe_root::t",
        "threshold_probe_root::root"
    );
    let aa = Atom::var(a);
    let xx = Atom::var(x);
    let tt = Atom::var(t);
    let body = (&aa + xx.pow(2)).pow((1, 2));
    let call = root.call([aa.clone(), xx.clone()].as_slice());
    let calls = [
        call.clone(),
        call.derivative(x),
        call.derivative(a),
        call.derivative(x).derivative(x),
        call.derivative(x).derivative(a),
    ];
    let root_bodies = [
        body.clone(),
        body.derivative(x),
        body.derivative(a),
        body.derivative(x).derivative(x),
        body.derivative(x).derivative(a),
    ];
    let mut functions = FunctionMap::new();
    functions.add_function(root, vec![a, x], body.clone())?;
    for (orders, derivative) in [[0, 1], [1, 0], [0, 2], [1, 1]]
        .into_iter()
        .zip(&root_bodies[1..])
    {
        functions.add_tagged_function(
            Symbol::DERIVATIVE,
            vec![Atom::num(orders[0]), Atom::num(orders[1]), Atom::var(root)],
            vec![a, x],
            derivative.clone(),
        )?;
    }

    // Full pulled-back density: y=t*r(a,x), J=r(a,x). Native symbolic
    // differentiation includes both y and J before evaluator composition.
    let density: Atom =
        &call * (Atom::num(1) + (&tt * &call).pow(2)) / (Atom::num(1) + &tt * &call);
    let expressions = [
        density.clone(),
        density.derivative(x),
        density.derivative(x).derivative(x),
        density.derivative(x).derivative(a),
    ];
    let parameters = [aa.clone(), xx.clone(), tt.clone()];
    let direct = Atom::evaluator_multiple(&expressions, &parameters)
        .function_map(functions.clone())
        .horner_iterations(10)
        .build()?;
    let prefix = Atom::evaluator_multiple(&calls, &[aa.clone(), xx.clone()])
        .function_map(functions)
        .horner_iterations(10)
        .build()?;
    let mut inputs = calls.to_vec();
    inputs.extend([xx.clone(), tt.clone()]);
    let outer = Atom::evaluator_multiple(&expressions, &inputs)
        .horner_iterations(10)
        .build()?;
    let mut composer = EvaluatorComposer::new(3);
    let mut bindings = composer.append(&prefix, &[Slot::Param(0), Slot::Param(1)])?;
    bindings.extend([Slot::Param(1), Slot::Param(2)]);
    let outputs = composer.append(&outer, &bindings)?;
    let composed = composer.finish(&outputs, OptimizationSettings::default())?;
    assert_eq!(composed.get_input_len(), 3);
    assert_eq!(composed.get_output_len(), 4);
    assert!(composed.export_instructions().sub_evaluators.is_empty());

    let explicit: Atom =
        &body * (Atom::num(1) + (&tt * &body).pow(2)) / (Atom::num(1) + &tt * &body);
    let explicit_outputs = [
        explicit.clone(),
        explicit.derivative(x),
        explicit.derivative(x).derivative(x),
        explicit.derivative(x).derivative(a),
    ];
    let explicit = Atom::evaluator_multiple(&explicit_outputs, &parameters)
        .horner_iterations(10)
        .build()?;
    let mut evaluators =
        [direct, composed, explicit].map(|program| program.map_coeff(&|q| q.re.to_f64()));
    let mut roots = prefix.map_coeff(&|q| q.re.to_f64());
    let mut max_error = 0.0_f64;
    for point in [
        [1.0_f64, 0.0, 0.0],
        [0.5, 0.25, 0.375],
        [2.0, 0.75, 1.0],
        [0.0001, 0.03125, 0.625],
    ] {
        let r = (point[0] + point[1] * point[1]).sqrt();
        let expected = [
            r,
            point[1] / r,
            1.0 / (2.0 * r),
            point[0] / r.powi(3),
            -point[1] / (2.0 * r.powi(3)),
        ];
        let mut actual = [0.0; 5];
        roots.evaluate(&point[..2], &mut actual);
        for (left, right) in actual.into_iter().zip(expected) {
            assert!((left - right).abs() <= 2e-12 * right.abs().max(1.0));
        }
        let mut vectors = [[0.0; 4]; 3];
        for (evaluator, vector) in evaluators.iter_mut().zip(&mut vectors) {
            evaluator.evaluate(&point, vector);
        }
        for vector in &vectors[..2] {
            for (actual, expected) in vector.iter().zip(vectors[2]) {
                let error = (actual - expected).abs() / expected.abs().max(1.0);
                assert!(error <= 2e-12, "{point:?}: {actual} != {expected}");
                max_error = max_error.max(error);
            }
        }
    }

    // A simple-root theorem cannot be used at a=x=r=0: P_r vanishes.
    let pr = Atom::num(2) * &call;
    let collision = pr.replace(call.clone()).with(Atom::Zero);
    assert_eq!(collision, Atom::Zero);

    // Exercise the owner's actual non-inlined-body composition refusal,
    // without relying on the unrelated DERIVATIVE/Never upstream patch.
    let mut retained = FunctionMap::new();
    retained.add_function_with_options(
        root,
        vec![a, x],
        body,
        FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
    )?;
    let retained = call
        .evaluator(&[aa, xx])
        .function_map(retained)
        .horner_iterations(10)
        .build()?;
    let error = EvaluatorComposer::new(2)
        .append(&retained, &[Slot::Param(0), Slot::Param(1)])
        .expect_err("retained body must be explicitly inlined");
    Ok(json!({
        "root_equation": "r^2-a-x^2=0; positive root for a>0",
        "symbolic_partial_orders": [[0,0],[0,1],[1,0],[0,2],[1,1]],
        "complete_density_vectors_compared": 4,
        "points_including_parameter_rebinding": 4,
        "maximum_scaled_difference": max_error,
        "public_runtime_inputs": 3,
        "supplied_collision_counterexample_pr_is_zero": true,
        "general_collision_detector_exercised": false,
        "noninlined_composition_refusal": error,
        "generic_implicit_root_solver_or_endpoint_reduction_implemented": false
    }))
}
