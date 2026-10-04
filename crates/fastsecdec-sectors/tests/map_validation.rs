use fastsecdec_sectors::{
    DecompositionOptions, ParametricDomain, PolynomialSupport, SectorMap, decompose,
};
use std::ops::ControlFlow;

#[test]
fn generated_maps_validate_in_all_domains() {
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::PositiveOrthant,
        ParametricDomain::ProjectiveSimplex,
    ] {
        let supports = if domain == ParametricDomain::ProjectiveSimplex {
            vec![vec![2, 0], vec![1, 1], vec![0, 2]]
        } else {
            vec![vec![0, 0], vec![2, 0], vec![0, 2]]
        };
        let decomposition = decompose(
            domain,
            &[PolynomialSupport::new(supports).unwrap()],
            &DecompositionOptions::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        for sector in decomposition.sectors {
            sector.validate(domain).unwrap();
        }
    }
}

#[test]
fn malformed_maps_are_rejected_without_panics() {
    let valid = SectorMap {
        fixed_parameter: Some(1),
        exponent_matrix: vec![
            vec![2.into(), 1.into()],
            vec![0.into(), 0.into()],
            vec![0.into(), 1.into()],
        ],
        determinant: 2.into(),
        jacobian_powers: vec![1.into(), 1.into()],
        factor_valuations: vec![vec![0.into(), 0.into()]],
    };
    valid.validate(ParametricDomain::ProjectiveSimplex).unwrap();
    let mut variants = Vec::new();
    let mut map = valid.clone();
    map.fixed_parameter = None;
    variants.push(map);
    let mut map = valid.clone();
    map.fixed_parameter = Some(3);
    variants.push(map);
    let mut map = valid.clone();
    map.exponent_matrix[1][0] = 1.into();
    variants.push(map);
    let mut map = valid.clone();
    map.exponent_matrix[0].pop();
    variants.push(map);
    let mut map = valid.clone();
    map.factor_valuations[0].pop();
    variants.push(map);
    let mut map = valid.clone();
    map.determinant = 0.into();
    variants.push(map);
    let mut map = valid.clone();
    map.determinant = (-2).into();
    variants.push(map);
    let mut map = valid.clone();
    map.determinant = 1.into();
    variants.push(map);
    let mut map = valid.clone();
    map.jacobian_powers[0] = 2.into();
    variants.push(map);
    let mut map = valid.clone();
    map.exponent_matrix[2] = map.exponent_matrix[0].clone();
    map.jacobian_powers = vec![3.into(), 1.into()];
    variants.push(map);
    for map in variants {
        assert!(map.validate(ParametricDomain::ProjectiveSimplex).is_err());
    }
    assert!(valid.validate(ParametricDomain::UnitCube).is_err());
}

#[test]
fn signed_orthant_and_zero_dimensional_maps_keep_their_domain_meaning() {
    let inverse = SectorMap {
        fixed_parameter: None,
        exponent_matrix: vec![vec![(-1).into()]],
        determinant: 1.into(),
        jacobian_powers: vec![(-2).into()],
        factor_valuations: vec![vec![(-3).into()]],
    };
    inverse.validate(ParametricDomain::PositiveOrthant).unwrap();
    assert!(inverse.validate(ParametricDomain::UnitCube).is_err());
    let mut scalar = SectorMap {
        fixed_parameter: None,
        exponent_matrix: vec![],
        determinant: 1.into(),
        jacobian_powers: vec![],
        factor_valuations: vec![vec![]],
    };
    scalar.validate(ParametricDomain::UnitCube).unwrap();
    assert!(
        scalar
            .validate(ParametricDomain::ProjectiveSimplex)
            .is_err()
    );
    scalar.exponent_matrix.push(vec![]);
    scalar.fixed_parameter = Some(0);
    scalar
        .validate(ParametricDomain::ProjectiveSimplex)
        .unwrap();
}
