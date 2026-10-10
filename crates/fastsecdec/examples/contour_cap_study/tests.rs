use super::*;
use fastsecdec::{
    generation::{GenerationOptions, generate},
    kernel::{
        CompilationSettings, EvaluatorBackend,
        indexed::{IndexedReader, IndexedWriter, write_unit},
    },
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{io::Cursor, ops::ControlFlow};
use symbolica::symbol;

#[test]
fn indexed_runtime_parameter_is_bound_before_selected_scope_admission() {
    let x = symbol!("cap_study_scope::x");
    let eps = symbol!("cap_study_scope::eps");
    let parameter = symbol!("cap_study_scope::p");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::var(parameter),
            vec![-Atom::one() + Atom::var(eps)],
            vec![PolynomialFactor::new(
                Atom::one() + Atom::var(x),
                -Atom::one(),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let kernels = generated
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[parameter],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                horner_iterations: 0,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert_eq!(kernels.sectors().len(), 1);
    let mut unit = Cursor::new(Vec::new());
    let receipts = write_unit(&mut unit, &kernels, vec![0]).unwrap();
    let mut writer = IndexedWriter::new(Cursor::new(Vec::new())).unwrap();
    let mut source = unit.get_ref().as_slice();
    for receipt in receipts {
        writer.append_record(&mut source, receipt).unwrap();
    }
    let (bytes, _) = writer.finish().unwrap();
    let mut reader = IndexedReader::from_reader(
        Cursor::new(bytes.into_inner()),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let mut loaded = reader.load_all().unwrap();
    let plan:Plan=serde_json::from_value(json!({
      "input":{"storage":"flat","saved":{"path":"unused","blake3":"unused"}},
      "expected_recipe":"undeformed-v1","expected_jacobian":"symbolic","expected_sectors":1,"parameters":{},
      "arms":[{"id":"control","settings":{"deformation":{"mode":"off"},"validation":{"policy":"pilot","pilot_points":16}}}],
      "sector_ids":[0],"seed":1,"pilot_seed":2,"points":1024,"shifts":8,"top_k_per_sector":1,"setup_seconds":60.,"sampling_seconds":30.
    })).unwrap();
    assert!(
        scope(&loaded, &plan).is_err(),
        "unbound exact coefficients must not be admitted"
    );
    let scope = bound_scope(&mut loaded, &plan, &BTreeMap::from([(parameter, 2.)])).unwrap();
    assert_eq!(
        scope,
        ResultScope::SelectedSectors {
            sector_ids: vec![0],
            exact_policy: ExactContributionPolicy::ExcludeAll
        }
    );
    let manifest = KernelResultManifest::from_kernels(&loaded);
    let problem = manifest
        .integration_problem(&scope, loaded.content_id())
        .unwrap();
    assert!(problem.exact_coefficients.iter().all(|value| *value == 0.));
    assert!(
        loaded
            .exact_coefficients()
            .iter()
            .all(|value| value.is_finite())
    );
}
