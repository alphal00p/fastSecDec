use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

#[test]
fn compact_coefficients_materialize_only_on_explicit_request() {
    let x = symbol!("native_alias_lazy::x");
    let eps = symbol!("native_alias_lazy::eps");
    let polynomial = (Atom::one() + Atom::var(x)).pow(10_000);
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                polynomial,
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let sector = &generated.sectors()[0];
    assert!(sector.materialized.get().is_none());
    assert!(!sector.aliased_coefficients()[0].get_aliases().is_empty());
    assert!(sector.aliased_coefficients()[0].get_byte_size() < 256);
    let kernels = generated.compile().unwrap();
    kernels.to_bytes().unwrap();
    generated.to_kernel_bytes(Default::default()).unwrap();
    assert!(sector.materialized.get().is_none());
    let expected = (Atom::one() + Atom::var(sector.parameters()[0])).pow(10_000);
    assert_eq!(sector.coefficients(), &[expected]);
    assert!(sector.materialized.get().is_some());
}

#[test]
fn alias_images_remain_flat_literal_and_collision_free() {
    let x = symbol!("fastsecdec::laurent_template::c0");
    let eps = symbol!("native_alias_literal::eps_");
    let expression = (Atom::one() + Atom::var(x)).pow(Atom::var(eps)) / Atom::var(eps);
    let result = expand(&expression, &[x], eps, 1, &mut TemplateCache::default()).unwrap();
    assert_eq!(result.keys().copied().collect::<Vec<_>>(), [-1, 0, 1]);
    let log = (Atom::one() + Atom::var(x)).log();
    assert_eq!(result[&-1].clone().into_inner(), Atom::one());
    assert_eq!(result[&0].clone().into_inner(), log.clone());
    assert_eq!(result[&1].clone().into_inner(), log.pow(2) / Atom::num(2));
    for coefficient in result.values() {
        assert!(!coefficient.get_aliases().contains_key(&Atom::var(x)));
        for body in coefficient.get_aliases().values() {
            for handle in coefficient.get_aliases().keys() {
                assert!(!body.contains(handle.as_view()));
            }
        }
    }
    assert!(matches!(
        expand_template(&parse!("exp(1/native_alias_literal::eps_)"), eps, 0),
        Err(GenerationError::Series(_))
    ));
    let unused = symbol!("fastsecdec::laurent_template::c0");
    let active = symbol!("native_alias_literal::active");
    let result = expand(
        &Atom::var(active).pow(2),
        &[unused, active],
        eps,
        0,
        &mut TemplateCache::default(),
    )
    .unwrap();
    assert!(!result[&0].get_aliases().contains_key(&Atom::var(unused)));
    assert_eq!(result[&0].clone().into_inner(), Atom::var(active).pow(2));
}
