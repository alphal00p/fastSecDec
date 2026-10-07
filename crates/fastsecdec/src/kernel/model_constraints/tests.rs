use super::*;
use crate::kernel::{CompilationSettings, EvaluatorBackend};
use symbolica::{atom::EvaluationInfo, function, parse, symbol};

fn template(p: Symbol, exact: Atom, complex: bool) -> KernelSet {
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![],
        vec![exact],
        Default::default(),
        None,
        complex,
        vec![p],
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap();
    kernels.initialize_artifact().unwrap();
    kernels
}

#[test]
fn saved_exact_offsets_bind_without_compilation_and_keep_failed_rebind_atomic() {
    let p = symbol!("offset_binding::p");
    let initial = template(p, parse!("offset_binding::p^(1/2)"), true);
    let bytes = initial.artifact_bytes().unwrap().to_vec();
    let identity = initial.content_id().to_owned();
    let mut restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.content_id(), identity);
    assert!(!restored.parameters_bound());
    for value in [4.0, -4.0] {
        restored
            .bind_parameters(&BTreeMap::from([(p, value)]))
            .unwrap();
        let expected = if value > 0.0 { [2.0, 0.0] } else { [0.0, 2.0] };
        assert_eq!(restored.exact_coefficients(), expected);
        assert_eq!(restored.artifact_bytes().unwrap(), bytes);
    }
    let mut pole = template(p, parse!("1/offset_binding::p"), false);
    pole.bind_parameters(&BTreeMap::from([(p, 2.0)])).unwrap();
    let id = pole.content_id().to_owned();
    assert!(matches!(
        pole.bind_parameters(&BTreeMap::from([(p, 0.0)])),
        Err(KernelError::NonFinite)
    ));
    assert_eq!(pole.content_id(), id);
    assert_eq!(pole.exact_coefficients(), [0.5]);
    let constants =
        super::super::exact::evaluate(&[parse!("2+3𝑖")], &BTreeMap::new(), true).unwrap();
    assert_eq!(constants, [2.0, 3.0]);
}

#[test]
fn native_mass_indeterminates_keep_tags_and_defer_function_errors_to_binding() {
    let p = symbol!("mass_direct::p");
    assert!(matches!(
        template(p, Atom::Zero, false).with_runtime_mass_constraints(vec![RuntimeMassConstraint {
            name: "m".into(),
            expression: parse!("mass_direct::undeclared")
        }]),
        Err(KernelError::Artifact(_))
    ));
    let tagged = symbol!(
        "mass_direct::tagged",
        eval = EvaluationInfo::new().with_tags(1).register_tagged(|tags| {
            assert_eq!(tags.len(), 1);
            Box::new(|args: &[Complex<Float>]| args[0].clone())
        })
    );
    let tag = Atom::var(symbol!("mass_direct::symbolic_tag"));
    let kernels = template(p, Atom::var(p), false)
        .with_runtime_mass_constraints(vec![RuntimeMassConstraint {
            name: "m".into(),
            expression: function!(tagged, tag, Atom::var(p)),
        }])
        .unwrap();
    let mut loaded = KernelSet::from_bytes(kernels.artifact_bytes().unwrap()).unwrap();
    loaded.bind_parameters(&BTreeMap::from([(p, 2.0)])).unwrap();
    assert_eq!(loaded.exact_coefficients(), [2.0]);
    for expression in [
        parse!("unknown_mass_function(mass_direct::p)"),
        parse!("log(mass_direct::undeclared)"),
        parse!("log(mass_direct::p,mass_direct::p)"),
    ] {
        let template = template(p, Atom::Zero, false)
            .with_runtime_mass_constraints(vec![RuntimeMassConstraint {
                name: "m".into(),
                expression,
            }])
            .unwrap();
        let mut loaded = KernelSet::from_bytes(template.artifact_bytes().unwrap()).unwrap();
        let id = loaded.content_id().to_owned();
        assert!(matches!(
            loaded.bind_parameters(&BTreeMap::from([(p, 2.0)])),
            Err(KernelError::Parameters(_))
        ));
        assert!(!loaded.parameters_bound());
        assert_eq!(loaded.content_id(), id);
    }
}
