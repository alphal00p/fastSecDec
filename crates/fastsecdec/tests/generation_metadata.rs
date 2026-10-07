use fastsecdec::{
    generation::{BranchPolicy, FactorCertificate, GenerationOptions, generate},
    kernel::{KernelSet, PrecisionPolicy},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, AtomCore},
    id::{Pattern, Replacement},
    parse, symbol,
};

fn generated(
    domain: ParametricDomain,
    polynomial: Atom,
    parameters: Vec<symbolica::atom::Symbol>,
    exponent: Atom,
    asserted: bool,
) -> fastsecdec::generation::GeneratedIntegral {
    let input = ParametricIntegrand::new(
        parameters.clone(),
        symbol!("metadata::eps"),
        domain,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; parameters.len()],
            vec![PolynomialFactor::new(
                polynomial,
                exponent,
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            assume_no_threshold: asserted,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
}

#[test]
fn retained_maps_are_the_actual_pre_subtraction_pullback_and_keep_gauge_meaning() {
    let x = symbol!("metadata::x");
    let y = symbol!("metadata::y");
    for (domain, polynomial, parameters, exponent) in [
        (
            ParametricDomain::UnitCube,
            parse!("metadata::x+metadata::y"),
            vec![x, y],
            Atom::num(-1),
        ),
        (
            ParametricDomain::ProjectiveSimplex,
            parse!("metadata::x+metadata::y"),
            vec![x, y],
            Atom::num(-2),
        ),
        (
            ParametricDomain::PositiveOrthant,
            parse!("1+metadata::x"),
            vec![x],
            Atom::num(-2),
        ),
    ] {
        let value = generated(
            domain,
            polynomial.clone(),
            parameters,
            exponent.clone(),
            false,
        );
        assert_eq!(
            value.metadata().domain_assessment().branch_policy(),
            BranchPolicy::UserResponsible
        );
        assert!(!value.metadata().domain_assessment().relies_on_assertion());
        assert_eq!(value.metadata().charts().len(), 2);
        assert_eq!(value.sectors().len(), 1);
        for chart in value.metadata().charts() {
            assert_eq!(chart.kernel_sector(), Some(0));
            let coordinates = chart.coordinates();
            if domain == ParametricDomain::ProjectiveSimplex {
                let fixed = coordinates.projective_fixed_parameter().unwrap();
                assert_eq!(coordinates.images()[fixed], Atom::one());
                assert_ne!(
                    coordinates.images().iter().cloned().sum::<Atom>(),
                    Atom::one()
                );
            } else {
                assert!(coordinates.projective_fixed_parameter().is_none());
            }
        }
        let chart = &value.metadata().charts()[0];
        let coordinates = chart.coordinates();
        let pulled = polynomial
            .replace_multiple(
                coordinates
                    .source_parameters()
                    .iter()
                    .zip(coordinates.images())
                    .map(|(source, target)| {
                        Replacement::new(
                            Pattern::Literal(Atom::var(*source)),
                            Pattern::Literal(target.clone()),
                        )
                    }),
            )
            .pow(exponent)
            * coordinates.measure_jacobian();
        // These regular mapped examples need no subtraction. The two verified
        // equivalent charts contribute twice the representative pullback.
        assert!(
            (pulled * Atom::num(2) - &value.sectors()[0].coefficients()[0])
                .together()
                .is_zero()
        );
        let kernel = value.compile().unwrap();
        let restored = KernelSet::from_bytes(&kernel.to_bytes().unwrap()).unwrap();
        assert_eq!(restored.content_id(), kernel.content_id());
        assert_eq!(restored.generation_metadata().unwrap().charts().len(), 2);
    }
}

#[test]
fn assessments_and_exact_chart_associations_survive_portable_roundtrip() {
    let asserted = generated(
        ParametricDomain::UnitCube,
        parse!("1-metadata::x+metadata::x^2"),
        vec![symbol!("metadata::x")],
        Atom::num(-1),
        true,
    );
    let metadata = asserted.metadata().domain_assessment();
    assert!(!metadata.relies_on_assertion() && metadata.caller_asserted());
    assert_eq!(
        metadata.factors()[0].certificate(),
        FactorCertificate::UncheckedUserResponsibility
    );
    let restored = KernelSet::from_bytes(
        &asserted
            .to_kernel_bytes(PrecisionPolicy::default())
            .unwrap(),
    )
    .unwrap();
    assert!(
        !restored
            .generation_metadata()
            .unwrap()
            .domain_assessment()
            .relies_on_assertion()
    );
    let ordinary = generated(
        ParametricDomain::UnitCube,
        parse!("1+metadata::x"),
        vec![symbol!("metadata::x")],
        Atom::num(-1),
        false,
    );
    let optional_assertion = generated(
        ParametricDomain::UnitCube,
        parse!("1+metadata::x"),
        vec![symbol!("metadata::x")],
        Atom::num(-1),
        true,
    );
    assert_ne!(
        ordinary.compile().unwrap().content_id(),
        optional_assertion.compile().unwrap().content_id()
    );
    let exact = generated(
        ParametricDomain::UnitCube,
        Atom::one(),
        vec![symbol!("metadata::x")],
        Atom::num(-1),
        false,
    );
    assert!(exact.sectors().is_empty());
    assert!(
        exact
            .metadata()
            .charts()
            .iter()
            .all(|chart| chart.kernel_sector().is_none())
    );
    assert_eq!(
        KernelSet::from_bytes(&exact.to_kernel_bytes(PrecisionPolicy::default()).unwrap())
            .unwrap()
            .exact_coefficients(),
        [1.0]
    );
}

#[test]
fn binary_metadata_validation_rejects_corruption() {
    let generated = generated(
        ParametricDomain::UnitCube,
        parse!("1+metadata::x"),
        vec![symbol!("metadata::x")],
        Atom::num(-1),
        false,
    );
    let bytes = generated
        .to_kernel_bytes(PrecisionPolicy::default())
        .unwrap();
    assert!(bytes.starts_with(b"FastSecDec\0binserde"));
    assert!(KernelSet::from_bytes(&bytes).is_ok());
    // Exact semantic-field mutations are tested beside the native metadata
    // decoder, without duplicating its private context-binserde wire format.
    for offset in [0, bytes.len() / 2, bytes.len() - 1] {
        let mut invalid = bytes.clone();
        invalid[offset] ^= 1;
        assert!(
            KernelSet::from_bytes_with_options(
                &invalid,
                fastsecdec::kernel::KernelLoadOptions { validate: true },
            )
            .is_err()
        );
    }
}

#[test]
fn zero_dimensional_direct_exact_densities_keep_valid_metadata() {
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::PositiveOrthant,
    ] {
        let input = ParametricIntegrand::new(
            vec![],
            symbol!("metadata::eps"),
            domain,
            vec![ParametricTerm::new(Atom::num(7), vec![], vec![])],
        )
        .unwrap();
        let generated = generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        assert_eq!(generated.exact_coefficients(), [Atom::num(7)]);
        let kernels = KernelSet::from_bytes(
            &generated
                .to_kernel_bytes(PrecisionPolicy::default())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(kernels.exact_coefficients(), [7.0]);
        assert!(
            kernels
                .generation_metadata()
                .unwrap()
                .domain_assessment()
                .parameters()
                .is_empty()
        );
    }
    let source = symbol!("fastsecdec::sector_0::t0");
    let input = ParametricIntegrand::new(
        vec![source],
        symbol!("metadata::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::num(7), vec![Atom::Zero], vec![])],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert!(
        !generated.metadata().charts()[0]
            .coordinates()
            .target_parameters()
            .contains(&source)
    );
    assert!(
        KernelSet::from_bytes(
            &generated
                .to_kernel_bytes(PrecisionPolicy::default())
                .unwrap()
        )
        .is_ok()
    );
    // An unused regulator must also stay distinct from generated coordinates;
    // otherwise the Laurent expansion would mistake a coordinate for epsilon.
    let input = ParametricIntegrand::new(
        vec![symbol!("metadata::x")],
        source,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::one(), vec![Atom::one()], vec![])],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let mut kernels = generated.compile().unwrap();
    let mut output = [0.0];
    kernels.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert_eq!(output, [0.25]);
}
