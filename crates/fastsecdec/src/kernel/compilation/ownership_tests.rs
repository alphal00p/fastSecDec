use super::*;
use crate::{
    contour::{
        dynamic::DynamicEnvelope,
        functions::dynamic::{RootProgram, strength},
    },
    generation::{GenerationOptions, generate},
    kernel::{DynamicChartRecipe, NativeProgramDescriptor, ProgramRecipe},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::sync::Arc;
use symbolica::{
    atom::AtomCore,
    domains::float::{Float, RealLike},
    symbol,
};

#[test]
fn detached_compilation_job_retains_its_native_helper_owner() {
    let x = symbol!("detached_recipe_job::x");
    let source = ParametricIntegrand::new(
        vec![x],
        symbol!("detached_recipe_job::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                Atom::one() + Atom::var(x),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&source, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let mut sector = generated.sectors()[0].clone();
    let variable = sector.parameters()[0];
    let envelope =
        DynamicEnvelope::new(&[variable], Atom::one() + Atom::var(variable), &[]).unwrap();
    let helper = RootProgram::build(1).unwrap();
    sector.dynamic_check_sources = vec![Arc::new(
        crate::kernel::DynamicCheckSource::from_envelope(0, &envelope),
    )];
    let chart = DynamicChartRecipe::from_envelope(0, &envelope, &helper).unwrap();
    let expression = strength(
        &helper,
        &[Atom::one() + Atom::var(variable).pow(2)],
        &Atom::num((4, 5)),
        &Atom::one(),
    )
    .unwrap();
    sector.coefficients = vec![expression.into()];
    sector.materialized = Default::default();
    let descriptor = Arc::new(
        NativeProgramDescriptor::dynamic(
            ProgramRecipe::DynamicPolynomialV1,
            vec![chart],
            vec![helper],
        )
        .unwrap(),
    );
    let weak = Arc::downgrade(&descriptor);
    let job = CompilationJob {
        owner: Arc::new(()),
        program_descriptor: Some(descriptor),
        index: 0,
        sector,
        runtime_parameters: Arc::new(Vec::new()),
        precision: PrecisionPolicy::default(),
        settings: CompilationSettings::default(),
        use_complex: false,
    };
    drop(generated);
    assert!(weak.upgrade().is_some());
    let mut completion = job.run().unwrap();
    assert!(weak.upgrade().is_some());
    let mut values = [0.0];
    completion.sector.evaluate(&[0.5], &mut values).unwrap();
    assert!((values[0] - 0.8 / 1.25_f64.sqrt()).abs() < 1e-14);
    let mut detached = completion.sector.try_clone().unwrap();
    drop(completion);
    assert!(weak.upgrade().is_none());
    // A detached clone owns its preparation routing even after the complete
    // descriptor disappears. Precision mapping is lazy and happens now.
    let Backend::Real(backend) = &mut detached.backend else {
        panic!()
    };
    let values = backend
        .precision_cache
        .evaluate(
            &backend.exact_evaluator,
            &[0.5],
            192,
            |value| value.re.to_multi_prec_float(192),
            |value| Float::with_val(192, value),
        )
        .unwrap();
    assert!((values[0].to_f64() - 0.8 / 1.25_f64.sqrt()).abs() < 1e-14);
}
