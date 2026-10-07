use super::*;
use fastsecdec::generation::{CoefficientExpansionMethod, GeneratedIntegral};

fn endpoint_input(complex: bool) -> GeneratedIntegral {
    let input = ParametricIntegrand::new(
        vec![symbol!("inspection::x")],
        symbol!("inspection::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                parse!("(2+3𝑖)*gamma(1+inspection::eps)")
            } else {
                parse!("2*gamma(1+inspection::eps)")
            },
            vec![parse!("-2+inspection::eps")],
            vec![PolynomialFactor::new(
                parse!("1+inspection::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let mut options = GenerationOptions::default();
    options.coefficient_expansion.method = CoefficientExpansionMethod::NativeNamed;
    generate(&input, &options, |_| ControlFlow::Continue(())).unwrap()
}

#[test]
fn regulated_endpoint_record_precedes_subtraction_and_survives_cold_loading() {
    let generated = endpoint_input(false);
    let original = generated.metadata().charts()[0].pre_subtraction().unwrap();
    assert_eq!(original.version(), 1);
    assert_eq!(original.regulator(), symbol!("inspection::eps"));
    assert_eq!(original.terms().len(), 1);
    let term = &original.terms()[0];
    assert_eq!(term.prefactor(), &parse!("2*gamma(1+inspection::eps)"));
    let power = &term.powers()[0];
    assert_eq!(power.exponent(), &parse!("-2+inspection::eps"));
    assert_eq!(*power.constant(), -2);
    assert_eq!(*power.slope(), 1);
    assert_eq!(power.subtraction_count(), 2);
    assert!(term.regular_expression_bytes() > 0);
    let summary = generated.metadata().display(false).to_string();
    assert!(
        summary.contains("1 mapped terms; minimum endpoint constant -2; maximum Taylor count 2")
    );
    assert!(!summary.contains("gamma("));
    assert!(
        generated
            .metadata()
            .display(true)
            .to_string()
            .contains("Taylor count=2")
    );
    let kernels = generated.compile().unwrap();
    let bytes = kernels.to_bytes().unwrap();
    let restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.content_id(), kernels.content_id());
    assert_eq!(restored.artifact_bytes().unwrap(), bytes);
    let cold = restored.generation_metadata().unwrap().charts()[0]
        .pre_subtraction()
        .unwrap();
    assert_eq!(cold.terms()[0].prefactor(), term.prefactor());
    assert_eq!(cold.terms()[0].powers()[0].exponent(), power.exponent());
    assert_eq!(cold.terms()[0].powers()[0].subtraction_count(), 2);
}

#[test]
fn symmetry_retains_original_chart_powers_without_multiplicity() {
    let input = ParametricIntegrand::new(
        vec![symbol!("inspection::x"), symbol!("inspection::y")],
        symbol!("inspection::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![Atom::Zero; 2],
            vec![PolynomialFactor::new(
                parse!("inspection::x+inspection::y"),
                parse!("-1+inspection::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let result = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(result.metadata().charts().len(), 2);
    assert_eq!(result.sectors().len(), 1);
    for chart in result.metadata().charts() {
        assert_eq!(chart.kernel_sector(), Some(0));
        let term = &chart.pre_subtraction().unwrap().terms()[0];
        assert_eq!(term.prefactor(), &Atom::num(3));
        assert_eq!(
            term.powers().len(),
            chart.coordinates().target_parameters().len()
        );
        assert_eq!(
            term.powers()
                .iter()
                .filter(|p| p.exponent() == &parse!("inspection::eps"))
                .count(),
            1
        );
        assert!(term.powers().iter().all(|p| p.subtraction_count() == 0));
        assert_eq!(chart.representative_permutation().len(), 2);
    }
}

#[test]
fn actual_shared_program_size_records_distinguish_backend_and_arithmetic() {
    for complex in [false, true] {
        let generated = endpoint_input(complex);
        let kernels = generated.compile().unwrap();
        assert!(!kernels.sectors().is_empty());
        let bytes = kernels.to_bytes().unwrap();
        for sector in kernels.sectors() {
            let stats = sector.statistics();
            assert_eq!(stats.version, 1);
            assert_eq!(stats.arithmetic, if complex { "complex" } else { "real" });
            assert_eq!(stats.inputs, 1);
            assert_eq!(stats.outputs, generated.orders().len());
            assert!(stats.exact_program_bytes > 0);
            assert!(
                stats.operations.additions
                    + stats.operations.multiplications
                    + stats.operations.inversions
                    + stats.operations.function_calls
                    > 0
            );
            #[cfg(feature = "native")]
            {
                assert_eq!(stats.backend, "symjit_o2");
                assert!(stats.symjit_ir_bytes.unwrap() > 0);
            }
            #[cfg(feature = "portable")]
            {
                assert_eq!(stats.backend, "symbolica_interpreter");
                assert_eq!(stats.symjit_ir_bytes, None);
            }
            let roundtrip: fastsecdec::kernel::EvaluatorStatistics =
                serde_json::from_str(&serde_json::to_string(stats).unwrap()).unwrap();
            assert_eq!(roundtrip, *stats);
            assert_eq!(sector.try_clone().unwrap().statistics(), stats);
        }
        let restored = KernelSet::from_bytes(&bytes).unwrap();
        for (fresh, cold) in kernels.sectors().iter().zip(restored.sectors()) {
            assert_eq!(fresh.statistics(), cold.statistics());
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn old_chart_metadata_remains_explicitly_absent_without_identity_change() {
    let bytes = include_bytes!("../fixtures/kernel-v2-triangle.json");
    let kernels = KernelSet::from_bytes(bytes).unwrap();
    let meta = kernels.generation_metadata().unwrap();
    assert!(
        meta.charts()
            .iter()
            .all(|chart| chart.pre_subtraction().is_none())
    );
    let portable = fastsecdec::kernel::PortableMetadata::from_native(meta);
    assert!(
        !serde_json::to_string(&portable)
            .unwrap()
            .contains("pre_subtraction")
    );
    assert_eq!(kernels.to_bytes().unwrap(), bytes);
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(
        kernels.content_id(),
        original["content_id"].as_str().unwrap()
    );
}
