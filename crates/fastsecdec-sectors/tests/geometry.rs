use fastsecdec_sectors::{
    DecompositionOptions, ParametricDomain, PolynomialSupport, SectorError, decompose,
};
use numerica::domains::{
    integer::Integer,
    rational::{Q, Rational},
};
use std::ops::ControlFlow;

fn run(domain: ParametricDomain, terms: Vec<Vec<i64>>) -> fastsecdec_sectors::Decomposition {
    decompose(
        domain,
        &[PolynomialSupport::new(terms).unwrap()],
        &DecompositionOptions::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
}

#[test]
fn cube_xy_partitions_volume_and_factors_polynomial() {
    let result = run(ParametricDomain::UnitCube, vec![vec![1, 0], vec![0, 1]]);
    assert_eq!(result.sectors.len(), 2);
    let mut volume = Rational::from(0);
    for sector in result.sectors {
        let denominator = sector
            .jacobian_powers
            .iter()
            .fold(Integer::from(1), |a, p| a * (p + Integer::from(1)));
        volume += Q.to_element(sector.determinant, denominator, true);
        assert!(
            sector
                .exponent_matrix
                .iter()
                .flatten()
                .all(|e| e >= &Integer::from(0))
        );
        // One endpoint variable factors from x+y in both charts.
        assert_eq!(
            sector.factor_valuations[0]
                .iter()
                .filter(|v| !v.is_zero())
                .count(),
            1
        );
    }
    assert_eq!(volume, Rational::from(1));
}

#[test]
fn orthant_one_plus_x_has_inversion_chart() {
    let result = run(ParametricDomain::PositiveOrthant, vec![vec![0], vec![1]]);
    assert_eq!(result.sectors.len(), 2);
    let mut maps: Vec<_> = result
        .sectors
        .iter()
        .map(|s| {
            (
                s.exponent_matrix[0][0].clone(),
                s.jacobian_powers[0].clone(),
                s.factor_valuations[0][0].clone(),
            )
        })
        .collect();
    maps.sort();
    assert_eq!(
        maps,
        vec![
            ((-1).into(), (-2).into(), (-1).into()),
            (1.into(), 0.into(), 0.into())
        ]
    );
    // For f(x)=1/(1+x)^2 both transformed integrands are 1/(1+t)^2.
    for s in &result.sectors {
        let m = s.exponent_matrix[0][0].to_i64().unwrap();
        let j = s.jacobian_powers[0].to_i64().unwrap();
        let t = 0.375_f64;
        let f = t.powi(j as i32) / (1.0 + t.powi(m as i32)).powi(2);
        assert!((f - 1.0 / (1.0 + t).powi(2)).abs() < 1e-14);
    }
}

#[test]
fn projective_simplex_has_primary_gauge_charts() {
    let result = run(
        ParametricDomain::ProjectiveSimplex,
        vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1]],
    );
    assert_eq!(result.sectors.len(), 3);
    for (axis, s) in result.sectors.iter().enumerate() {
        assert_eq!(s.fixed_parameter, Some(axis));
        assert_eq!(s.dimension(), 2);
        assert_eq!(s.exponent_matrix[axis], vec![Integer::from(0); 2]);
        assert_eq!(s.factor_valuations, vec![vec![Integer::from(0); 2]]);
    }
}

#[test]
fn rank_deficient_orthant_is_not_silently_zero() {
    let support = PolynomialSupport::new(vec![vec![1, 0], vec![0, 1]]).unwrap();
    assert!(matches!(
        decompose(
            ParametricDomain::PositiveOrthant,
            &[support],
            &DecompositionOptions::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(SectorError::RankDeficient {
            rank: 1,
            dimension: 2
        })
    ));
}

#[test]
fn cancellation_is_a_typed_error() {
    let support = PolynomialSupport::new(vec![vec![0], vec![1]]).unwrap();
    assert!(matches!(
        decompose(
            ParametricDomain::UnitCube,
            &[support],
            &DecompositionOptions::default(),
            |_| ControlFlow::Break(())
        ),
        Err(SectorError::Cancelled)
    ));
}

#[test]
fn projective_one_parameter_is_zero_dimensional() {
    let result = run(ParametricDomain::ProjectiveSimplex, vec![vec![2]]);
    assert_eq!(result.sectors.len(), 1);
    assert_eq!(result.sectors[0].dimension(), 0);
    assert_eq!(result.sectors[0].determinant, Integer::from(1));
}

#[test]
fn support_validation_rejects_negative_and_mismatched_exponents() {
    assert!(PolynomialSupport::new(vec![vec![-1]]).is_err());
    assert!(PolynomialSupport::new(vec![vec![1], vec![1, 2]]).is_err());
    assert!(PolynomialSupport::new(vec![]).is_err());
}

#[test]
fn non_simplicial_normal_cone_is_triangulated() {
    // Octahedron translated to nonnegative exponents; a vertex has a square
    // normal cone and requires two simplicial sectors.
    let result = run(
        ParametricDomain::PositiveOrthant,
        vec![
            vec![0, 1, 1],
            vec![2, 1, 1],
            vec![1, 0, 1],
            vec![1, 2, 1],
            vec![1, 1, 0],
            vec![1, 1, 2],
        ],
    );
    assert_eq!(result.geometric_vertices, 6);
    assert_eq!(result.sectors.len(), 12);
    assert!(
        result
            .sectors
            .iter()
            .all(|s| s.determinant > Integer::from(0))
    );
}

#[test]
fn nonhomogeneous_projective_factors_cannot_use_gauge_shortcut() {
    let support = PolynomialSupport::new(vec![vec![0, 0], vec![1, 0]]).unwrap();
    assert!(matches!(
        decompose(
            ParametricDomain::ProjectiveSimplex,
            &[support],
            &DecompositionOptions::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(SectorError::InvalidSupport(_))
    ));
}

#[test]
fn zero_dimensional_work_obeys_cancellation_and_limits() {
    let support = PolynomialSupport::new(vec![vec![0]]).unwrap();
    assert!(matches!(
        decompose(
            ParametricDomain::ProjectiveSimplex,
            &[support.clone()],
            &DecompositionOptions::default(),
            |_| ControlFlow::Break(())
        ),
        Err(SectorError::Cancelled)
    ));
    let options = DecompositionOptions {
        max_sectors: 0,
        ..DecompositionOptions::default()
    };
    assert!(matches!(
        decompose(
            ParametricDomain::ProjectiveSimplex,
            &[support],
            &options,
            |_| ControlFlow::Continue(())
        ),
        Err(SectorError::ResourceLimit { .. })
    ));
}

#[test]
fn simultaneous_factors_have_constant_residuals_and_preserve_cube_volume() {
    let supports = [
        PolynomialSupport::new(vec![
            vec![2, 0, 0],
            vec![0, 2, 0],
            vec![0, 0, 2],
            vec![1, 1, 1],
        ])
        .unwrap(),
        PolynomialSupport::new(vec![vec![1, 1, 0], vec![0, 1, 1], vec![1, 0, 1]]).unwrap(),
    ];
    let result = decompose(
        ParametricDomain::UnitCube,
        &supports,
        &DecompositionOptions::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut volume = Rational::from(0);
    for sector in result.sectors {
        let denominator = sector
            .jacobian_powers
            .iter()
            .fold(Integer::from(1), |a, p| a * (p + Integer::from(1)));
        volume += Q.to_element(sector.determinant, denominator, true);
        for (support, valuation) in supports.iter().zip(&sector.factor_valuations) {
            let transformed: Vec<Vec<Integer>> = support
                .exponents()
                .iter()
                .map(|e| {
                    (0..3)
                        .map(|j| {
                            (0..3).fold(Integer::from(0), |sum, i| {
                                sum + &e[i] * &sector.exponent_matrix[i][j]
                            })
                        })
                        .collect()
                })
                .collect();
            assert!(
                transformed.iter().any(|e| e == valuation),
                "each factor needs a nonzero constant residual"
            );
            assert!(
                transformed
                    .iter()
                    .all(|e| e.iter().zip(valuation).all(|(a, b)| a >= b))
            );
        }
    }
    assert_eq!(volume, Rational::from(1));
}
