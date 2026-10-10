use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelLoadOptions},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{atom::Atom, parse, symbol};

fn unit(index: usize) -> KernelSet {
    let (prefactor, power, factors, max_order) = match index {
        0 => (Atom::num(2), parse!("-1+resident_unit::eps"), vec![], 0),
        1 => (
            parse!("2+3𝑖"),
            Atom::Zero,
            vec![PolynomialFactor::new(
                parse!("1+resident_unit::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
            0,
        ),
        _ => (
            Atom::num(5),
            Atom::Zero,
            vec![PolynomialFactor::new(
                parse!("1+resident_unit::x"),
                parse!("-1+resident_unit::eps"),
                FactorRole::Singularity,
            )],
            1,
        ),
    };
    let input = ParametricIntegrand::new(
        vec![symbol!("resident_unit::x")],
        symbol!("resident_unit::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(prefactor, vec![power], factors)],
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            max_order,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn resident_layout_grows_without_recompiling_prior_units() {
    let recipe = ProgramRecipe::UndeformedV1;
    let mut archive =
        ProgramArchiveWriter::new(Cursor::new(Vec::new()), "e".repeat(64), [recipe]).unwrap();
    let mut resident = ProgramResidentAssembly::new("e".repeat(64), recipe).unwrap();
    for index in 0..3 {
        resident
            .append_unit(&mut archive, unit(index), &[index])
            .unwrap();
    }
    let (bytes, catalogue) = archive.finish().unwrap();
    let mut kernels = resident.finish(&catalogue).unwrap();
    let mut reader = super::super::ProgramArchiveReader::from_reader(
        bytes,
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let mut restored = reader.select(recipe).unwrap().load_all().unwrap();
    assert_eq!(kernels.content_id(), restored.content_id());
    assert_eq!(kernels.orders(), &[-1, -1, 0, 0, 1, 1]);
    assert_eq!(kernels.sectors.len(), 2);
    kernels.bind_parameters(&BTreeMap::new()).unwrap();
    restored.bind_parameters(&BTreeMap::new()).unwrap();
    assert_eq!(kernels.exact_coefficients(), &[2., 0., 0., 0., 0., 0.]);
    assert_eq!(kernels.exact_coefficients(), restored.exact_coefficients());
    for (left, right) in kernels.sectors_mut().iter_mut().zip(restored.sectors_mut()) {
        let mut a = [0.; 6];
        let mut b = a;
        left.evaluate(&[0.37], &mut a).unwrap();
        right.evaluate(&[0.37], &mut b).unwrap();
        assert_eq!(a, b);
        assert_eq!(&a[..2], &[0., 0.]);
    }
    let mut total = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut value = [0.; 6];
        for i in 0..512 {
            sector
                .evaluate(&[(i as f64 + 0.5) / 512.], &mut value)
                .unwrap();
            for (sum, v) in total.iter_mut().zip(value) {
                *sum += v / 512.;
            }
        }
    }
    let logarithm = 2f64.ln();
    for (actual, expected) in total.iter().zip([
        2.,
        0.,
        7. * logarithm,
        3. * logarithm,
        2.5 * logarithm * logarithm,
        0.,
    ]) {
        assert!((actual - expected).abs() < 2e-6, "{actual} vs {expected}");
    }
}

#[test]
fn noncanonical_resident_unit_order_cannot_claim_canonical_identity() {
    let recipe = ProgramRecipe::UndeformedV1;
    let mut archive =
        ProgramArchiveWriter::new(Cursor::new(Vec::new()), "f".repeat(64), [recipe]).unwrap();
    let mut resident = ProgramResidentAssembly::new("f".repeat(64), recipe).unwrap();
    resident.append_unit(&mut archive, unit(2), &[1]).unwrap();
    resident.append_unit(&mut archive, unit(1), &[0]).unwrap();
    let (_, catalogue) = archive.finish().unwrap();
    assert!(
        resident
            .finish(&catalogue)
            .err()
            .unwrap()
            .to_string()
            .contains("sector order")
    );
}

#[test]
fn saved_exact_records_cancel_factored_numeric_signs_before_binding() {
    let p = symbol!("resident_exact_cancellation::p");
    let eps = symbol!("resident_exact_cancellation::eps");
    let left = Atom::num(2) + Atom::one() / Atom::var(p);
    let right = -left.clone();
    assert!(!(&left + &right).is_zero());
    let recipe = ProgramRecipe::UndeformedV1;
    let mut archive =
        ProgramArchiveWriter::new(Cursor::new(Vec::new()), "c".repeat(64), [recipe]).unwrap();
    let mut resident = ProgramResidentAssembly::new("c".repeat(64), recipe).unwrap();
    for (index, exact) in [vec![left.clone(), left], vec![right.clone(), right + 6]]
        .into_iter()
        .enumerate()
    {
        let source = ParametricIntegrand::new(
            vec![symbol!("resident_exact_cancellation::x")],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                &exact[0] + Atom::var(eps) * &exact[1],
                vec![Atom::Zero],
                vec![],
            )],
        )
        .unwrap();
        let mut unit = generate(
            &source,
            &GenerationOptions {
                max_order: 1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[p],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert!(unit.sectors.is_empty());
        assert_eq!(unit.coefficient_orders, [0, 1]);
        // Preserve a valid factored representation in the native encoded
        // record, as older generation could do. Do not normalize the fixture:
        // both consuming assembly and reload must normalize their actual sum.
        unit.exact_expressions = exact;
        resident.append_unit(&mut archive, unit, &[index]).unwrap();
    }
    let (bytes, catalogue) = archive.finish().unwrap();
    let mut live = resident.finish(&catalogue).unwrap();
    let mut reader = super::super::ProgramArchiveReader::from_reader(
        bytes,
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let mut restored = reader.select(recipe).unwrap().load_all().unwrap();
    for kernels in [&mut live, &mut restored] {
        assert_eq!(kernels.exact_expressions, [Atom::Zero, Atom::num(6)]);
        kernels.bind_parameters(&BTreeMap::from([(p, 0.)])).unwrap();
        assert_eq!(kernels.exact_coefficients(), &[0., 6.]);
        assert!(kernels.exact_requests.is_empty());
    }
    assert_eq!(live.content_id(), restored.content_id());
}
