use super::*;
use fastsecdec::{
    generation::{GenerationMode, SubtractionStrategy},
    parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
};
use symbolica::{atom::Atom, symbol};

#[test]
fn source_chart_modes_retain_folded_exact_charts() {
    let input = ParametricIntegrand::new(
        vec![symbol!("record_exact_chart::x")],
        symbol!("record_exact_chart::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![Atom::num(-1)],
            vec![],
        )],
    )
    .unwrap();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let generated = generation::generate(
            &input,
            &GenerationOptions {
                mode,
                subtraction: SubtractionStrategy::Taylor,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert!(generated.sectors().is_empty());
        assert!(!generated.metadata().charts().is_empty());
        let recorded = source_chart_modes(&generated);
        assert_eq!(recorded.len(), generated.metadata().charts().len());
        for chart in generated.metadata().charts() {
            assert!(chart.kernel_sector().is_none());
            assert_eq!(recorded[&chart.source_index()], GenerationMode::Symbolic);
        }
    }
}
