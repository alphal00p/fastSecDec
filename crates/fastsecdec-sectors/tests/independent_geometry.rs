//! Independent review probes: conservation and fan coverage, rather than an
//! assertion that a particular decomposition has an expected sector count.
use fastsecdec_sectors::{
    Decomposition, DecompositionOptions, ParametricDomain, PolynomialSupport, SectorError,
    SectorMap, decompose,
};
use numerica::{
    domains::{
        integer::Integer,
        rational::{Q, Rational},
    },
    tensors::matrix::Matrix,
};
use std::ops::ControlFlow;

fn support(rows: &[&[i64]]) -> PolynomialSupport {
    PolynomialSupport::new(rows.iter().map(|row| row.to_vec()).collect()).unwrap()
}

fn run(domain: ParametricDomain, supports: &[PolynomialSupport]) -> Decomposition {
    decompose(domain, supports, &DecompositionOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
}

fn cube_moment(sectors: &[SectorMap], powers: &[i64]) -> Rational {
    sectors.iter().fold(Rational::from(0), |sum, sector| {
        assert_eq!(sector.source_dimension(), powers.len());
        assert!(
            sector
                .exponent_matrix
                .iter()
                .flatten()
                .all(|x| x >= &Integer::from(0))
        );
        let denominator = (0..sector.dimension()).fold(Integer::from(1), |product, j| {
            let exponent = sector
                .exponent_matrix
                .iter()
                .zip(powers)
                .fold(Integer::from(0), |sum, (row, power)| {
                    sum + &row[j] * Integer::from(power + 1)
                });
            assert!(exponent > 0);
            product * exponent
        });
        sum + Q.to_element(sector.determinant.clone(), denominator, true)
    })
}

fn expected_moment(powers: &[i64]) -> Rational {
    Q.to_element(
        1.into(),
        powers
            .iter()
            .fold(Integer::from(1), |p, q| p * Integer::from(q + 1)),
        true,
    )
}

fn check_cube_moments(result: &Decomposition, dimension: usize) {
    for powers in [
        vec![0; dimension],
        vec![1; dimension],
        (0..dimension as i64).collect(),
    ] {
        assert_eq!(
            cube_moment(&result.sectors, &powers),
            expected_moment(&powers)
        );
    }
    for axis in 0..dimension {
        let mut powers = vec![0; dimension];
        powers[axis] = 5;
        assert_eq!(
            cube_moment(&result.sectors, &powers),
            expected_moment(&powers)
        );
    }
}

/// Test logarithmic cone membership by an independent exact linear solve.
/// Samples on a fan boundary are excluded, since boundaries have measure zero.
fn check_fan_interiors(result: &Decomposition, cube: bool) {
    let dimension = result.sectors[0].dimension();
    let inverses: Vec<_> = result
        .sectors
        .iter()
        .map(|s| {
            assert_eq!(s.dimension(), s.source_dimension());
            Matrix::from_nested_vec(
                s.exponent_matrix
                    .iter()
                    .map(|row| row.iter().cloned().map(Rational::from).collect())
                    .collect(),
                Q,
            )
            .unwrap()
            .inv()
            .unwrap()
        })
        .collect();
    let mut checked = 0;
    for sample in 0..97 {
        let point: Vec<_> = (0..dimension)
            .map(|axis| {
                let value = ((sample * (axis * 16 + 13) + axis * 11 + 3) % 103 + 1) as i64;
                Rational::from(if cube || (sample >> axis) & 1 == 0 {
                    value
                } else {
                    -value
                })
            })
            .collect();
        let mut strict = 0;
        let mut boundary = false;
        for inverse in &inverses {
            let coordinates: Vec<_> = (0..dimension)
                .map(|i| {
                    (0..dimension).fold(Rational::from(0), |sum, j| {
                        sum + &inverse[(i as u32, j as u32)] * &point[j]
                    })
                })
                .collect();
            if coordinates.iter().all(|x| x > &Rational::from(0)) {
                strict += 1;
            } else if coordinates.iter().all(|x| x >= &Rational::from(0)) {
                boundary = true;
            }
        }
        if !boundary {
            assert_eq!(strict, 1, "log point {point:?} has wrong multiplicity");
            checked += 1;
        }
    }
    assert!(
        checked >= 60,
        "too many samples coincided with fan boundaries"
    );
}

#[test]
fn repeated_minkowski_summands_preserve_extreme_vertices() {
    // Every mixed sum has several representations and is discarded. The four
    // exposed extreme sums must survive all three prefilter stages.
    let square = support(&[&[0, 0], &[2, 0], &[0, 2], &[2, 2], &[1, 1]]);
    let result = run(
        ParametricDomain::PositiveOrthant,
        &[square.clone(), square.clone(), square],
    );
    assert_eq!(result.geometric_vertices, 4);
    check_fan_interiors(&result, false);
}

#[test]
fn rank_deficiency_uses_combined_support_and_cube_recession() {
    let x = support(&[&[0, 0, 0], &[1, 0, 0]]);
    let yz = support(&[&[0, 0, 0], &[0, 1, 0], &[0, 0, 1]]);
    let combined = run(ParametricDomain::PositiveOrthant, &[x.clone(), yz]);
    check_fan_interiors(&combined, false);
    assert!(matches!(
        decompose(
            ParametricDomain::PositiveOrthant,
            std::slice::from_ref(&x),
            &DecompositionOptions::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(SectorError::RankDeficient {
            rank: 1,
            dimension: 3
        })
    ));
    // A cube needs no full affine rank: its recession orthant supplies the axes.
    check_cube_moments(&run(ParametricDomain::UnitCube, &[x]), 3);
    check_cube_moments(
        &run(ParametricDomain::UnitCube, &[support(&[&[4, 2, 1]])]),
        3,
    );
}

#[test]
fn four_dimensional_non_simplicial_fan_has_no_gaps_or_overlaps() {
    let mut rows = Vec::new();
    for axis in 0..4 {
        for offset in [-1, 1] {
            let mut row = vec![1; 4];
            row[axis] += offset;
            rows.push(row);
        }
    }
    let polytope = PolynomialSupport::new(rows).unwrap();
    let orthant = run(
        ParametricDomain::PositiveOrthant,
        std::slice::from_ref(&polytope),
    );
    assert_eq!(orthant.geometric_vertices, 8);
    check_fan_interiors(&orthant, false);
    let cube = run(ParametricDomain::UnitCube, &[polytope]);
    check_fan_interiors(&cube, true);
    check_cube_moments(&cube, 4);
}

#[test]
fn varied_supports_preserve_exact_cube_polynomial_moments() {
    for fixture in 0..12 {
        let first = (0..7)
            .map(|term| {
                (0..3)
                    .map(|axis| {
                        ((term * (axis * 4 + 3) + fixture * (axis + 2) + term * term) % 7) as i64
                    })
                    .collect()
            })
            .collect();
        let second = (0..4)
            .map(|term| {
                (0..3)
                    .map(|axis| ((term * (axis + 3) + fixture + axis * axis) % 5) as i64)
                    .collect()
            })
            .collect();
        let result = run(
            ParametricDomain::UnitCube,
            &[
                PolynomialSupport::new(first).unwrap(),
                PolynomialSupport::new(second).unwrap(),
            ],
        );
        check_cube_moments(&result, 3);
        check_fan_interiors(&result, true);
    }
}

#[test]
fn translation_does_not_change_fan_but_shifts_factor_valuations() {
    let rows = vec![vec![0, 0], vec![2, 0], vec![0, 3], vec![1, 1]];
    let offset = [3, 5];
    let shifted: Vec<Vec<i64>> = rows
        .iter()
        .map(|row| row.iter().zip(offset).map(|(x, a)| x + a).collect())
        .collect();
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::PositiveOrthant,
    ] {
        let a = run(domain, &[PolynomialSupport::new(rows.clone()).unwrap()]);
        let b = run(domain, &[PolynomialSupport::new(shifted.clone()).unwrap()]);
        assert_eq!(a.sectors.len(), b.sectors.len());
        for (a, b) in a.sectors.iter().zip(&b.sectors) {
            assert_eq!(a.exponent_matrix, b.exponent_matrix);
            assert_eq!(a.determinant, b.determinant);
            assert_eq!(a.jacobian_powers, b.jacobian_powers);
            for j in 0..2 {
                let correction = (0..2).fold(Integer::from(0), |sum, i| {
                    sum + &a.exponent_matrix[i][j] * Integer::from(offset[i])
                });
                assert_eq!(
                    b.factor_valuations[0][j],
                    &a.factor_valuations[0][j] + correction
                );
            }
        }
    }
}

// Positive half of the standard 16-point Gauss-Legendre rule. This independent
// quadrature checks the geometric map/Jacobian against analytic integrals.
const GAUSS: [(f64, f64); 8] = [
    (0.09501250983763744, 0.189_450_610_455_068_5),
    (0.2816035507792589, 0.1826034150449236),
    (0.4580167776572274, 0.16915651939500254),
    (0.6178762444026438, 0.14959598881657673),
    (0.755404408355003, 0.12462897125553387),
    (0.8656312023878318, 0.09515851168249279),
    (0.9445750230732326, 0.06225352393864789),
    (0.9894009349916499, 0.027152459411754095),
];

fn integrate_two_dimensions(result: &Decomposition, f: impl Fn(&[f64]) -> f64) -> f64 {
    let nodes: Vec<_> = GAUSS
        .iter()
        .flat_map(|&(x, w)| [((1.0 - x) / 2.0, w / 2.0), ((1.0 + x) / 2.0, w / 2.0)])
        .collect();
    let mut total = 0.0;
    for sector in &result.sectors {
        assert_eq!(sector.dimension(), 2);
        for &(u, wu) in &nodes {
            for &(v, wv) in &nodes {
                let point = [u, v];
                let source: Vec<_> = sector
                    .exponent_matrix
                    .iter()
                    .map(|row| {
                        row.iter()
                            .zip(point)
                            .map(|(power, t)| t.powi(power.to_i64().unwrap() as i32))
                            .product()
                    })
                    .collect();
                let jacobian = sector.determinant.to_i64().unwrap() as f64
                    * sector
                        .jacobian_powers
                        .iter()
                        .zip(point)
                        .map(|(power, t)| t.powi(power.to_i64().unwrap() as i32))
                        .product::<f64>();
                total += wu * wv * jacobian * f(&source);
            }
        }
    }
    total
}

#[test]
fn projective_gauge_maps_preserve_normalized_simplex_measure_and_moment() {
    let result = run(
        ParametricDomain::ProjectiveSimplex,
        &[
            support(&[&[2, 0, 0], &[0, 2, 0], &[0, 0, 2]]),
            support(&[&[1, 1, 0], &[1, 0, 1], &[0, 1, 1]]),
        ],
    );
    assert!(result.sectors.len() > 3, "probe needs secondary sectors");
    // For three parameters, gauge fixing a degree -3 density gives the
    // normalized simplex measure. x0/(sum x)^4 has the same total degree.
    let volume = integrate_two_dimensions(&result, |x| 1.0 / x.iter().sum::<f64>().powi(3));
    let moment = integrate_two_dimensions(&result, |x| x[0] / x.iter().sum::<f64>().powi(4));
    assert!((volume - 0.5).abs() < 2e-12, "{volume}");
    assert!((moment - 1.0 / 6.0).abs() < 2e-12, "{moment}");
}

#[test]
fn mixed_infinity_maps_preserve_integral_on_positive_orthant() {
    let result = run(
        ParametricDomain::PositiveOrthant,
        &[support(&[&[0, 0], &[1, 0], &[0, 1]])],
    );
    assert!(result.sectors.iter().any(|s| {
        s.exponent_matrix
            .iter()
            .flatten()
            .any(|p| p < &Integer::from(0))
    }));
    let integral = integrate_two_dimensions(&result, |x| 1.0 / (1.0 + x[0] + x[1]).powi(3));
    assert!((integral - 0.5).abs() < 2e-12, "{integral}");
    check_fan_interiors(&result, false);
}

#[test]
fn resource_errors_and_late_cancellation_remain_distinct_from_empty_results() {
    let supports = [support(&[&[1, 0], &[0, 1]])];
    for options in [
        DecompositionOptions {
            max_support_pairs: 1,
            ..Default::default()
        },
        DecompositionOptions {
            max_rays: 1,
            ..Default::default()
        },
        DecompositionOptions {
            max_sectors: 1,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            decompose(ParametricDomain::UnitCube, &supports, &options, |_| {
                ControlFlow::Continue(())
            }),
            Err(SectorError::ResourceLimit { .. })
        ));
    }
    let mut calls = 0;
    assert!(matches!(
        decompose(
            ParametricDomain::UnitCube,
            &supports,
            &DecompositionOptions::default(),
            |_| {
                calls += 1;
                if calls == 5 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        ),
        Err(SectorError::Cancelled)
    ));
    assert_eq!(calls, 5);
}
