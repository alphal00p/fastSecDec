use crate::{
    arithmetic::{IntVector, determinant, dot, rank},
    cone::extreme_rays,
    support::minkowski_candidates,
    triangulate::pulling,
    types::Monitor,
    *,
};
use numerica::domains::integer::Integer;
use std::ops::ControlFlow;

pub fn decompose(
    domain: ParametricDomain,
    supports: &[PolynomialSupport],
    options: &DecompositionOptions,
    mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
) -> Result<Decomposition, SectorError> {
    let Some(first) = supports.first() else {
        return Err(SectorError::InvalidSupport("no factor polynomials".into()));
    };
    let dim = first.dimension();
    if supports.iter().any(|s| s.dimension() != dim) {
        return Err(SectorError::InvalidSupport(
            "factor dimensions differ".into(),
        ));
    }
    if domain == ParametricDomain::ProjectiveSimplex && dim == 0 {
        return Err(SectorError::InvalidSupport(
            "projective input needs a parameter".into(),
        ));
    }
    if domain == ParametricDomain::ProjectiveSimplex {
        for support in supports {
            let degrees: Vec<_> = support
                .exponents()
                .iter()
                .map(|row| {
                    row.iter()
                        .fold(Integer::from(0), |sum, exponent| sum + exponent)
                })
                .collect();
            if degrees.iter().any(|degree| degree != &degrees[0]) {
                return Err(SectorError::InvalidSupport(
                    "projective gauge fixing requires individually homogeneous factor polynomials"
                        .into(),
                ));
            }
        }
    }
    let mut monitor = Monitor {
        callback: &mut progress,
        options,
        status: DecompositionProgress {
            phase: DecompositionPhase::Supports,
            chart: 0,
            completed_constraints: 0,
            total_constraints: 0,
            rays: 0,
            sectors: 0,
        },
    };
    let mut output = Decomposition {
        domain,
        sectors: Vec::new(),
        candidate_vertices: 0,
        geometric_vertices: 0,
    };
    let fixed_axes: Vec<_> = if domain == ParametricDomain::ProjectiveSimplex {
        (0..dim).map(Some).collect()
    } else {
        vec![None]
    };
    for fixed in fixed_axes {
        monitor.status.chart = fixed.unwrap_or(0);
        monitor.status.phase = DecompositionPhase::Supports;
        monitor.status.completed_constraints = 0;
        monitor.status.total_constraints = 0;
        monitor.status.rays = 0;
        monitor.status.sectors = output.sectors.len();
        let chart: Vec<_> = supports
            .iter()
            .map(|s| fixed.map_or_else(|| s.clone(), |i| s.without_axis(i)))
            .collect();
        decompose_chart(
            &chart,
            fixed,
            domain != ParametricDomain::PositiveOrthant,
            &mut output,
            &mut monitor,
        )?;
    }
    monitor.status.phase = DecompositionPhase::Complete;
    monitor.status.sectors = output.sectors.len();
    monitor.emit()?;
    Ok(output)
}

fn decompose_chart(
    supports: &[PolynomialSupport],
    fixed: Option<usize>,
    cube: bool,
    output: &mut Decomposition,
    monitor: &mut Monitor<'_>,
) -> Result<(), SectorError> {
    let dim = supports[0].dimension();
    if dim == 0 {
        monitor.emit()?;
        if output.sectors.len() >= monitor.options.max_sectors {
            return Err(SectorError::ResourceLimit {
                resource: "sectors",
                limit: monitor.options.max_sectors,
            });
        }
        output.sectors.push(SectorMap {
            fixed_parameter: fixed,
            exponent_matrix: vec![vec![]; usize::from(fixed.is_some())],
            determinant: 1.into(),
            jacobian_powers: vec![],
            factor_valuations: vec![vec![]; supports.len()],
        });
        return Ok(());
    }
    let candidates = minkowski_candidates(supports, monitor)?;
    output.candidate_vertices += candidates.len();
    if !cube {
        let differences: Vec<IntVector> = candidates
            .iter()
            .skip(1)
            .map(|p| p.iter().zip(&candidates[0]).map(|(a, b)| a - b).collect())
            .collect();
        let affine_rank = rank(&differences);
        if affine_rank != dim {
            return Err(SectorError::RankDeficient {
                rank: affine_rank,
                dimension: dim,
            });
        }
    }
    let mut generators: Vec<_> = candidates
        .iter()
        .map(|p| {
            let mut h = p.clone();
            h.push(1.into());
            h
        })
        .collect();
    if cube {
        for axis in 0..dim {
            let mut e = vec![Integer::from(0); dim + 1];
            e[axis] = 1.into();
            generators.push(e);
        }
    }
    let facets = extreme_rays(generators, monitor)?;
    for vertex in &candidates {
        monitor.status.phase = DecompositionPhase::Triangulation;
        monitor.emit()?;
        let mut point = vertex.clone();
        point.push(1.into());
        let mut rays: Vec<_> = facets
            .iter()
            .filter(|f| dot(f, &point).is_zero())
            .map(|f| f[..dim].to_vec())
            .collect();
        rays.sort();
        rays.dedup();
        if rank(&rays) != dim {
            continue;
        }
        output.geometric_vertices += 1;
        let mut inequalities: Vec<IntVector> = candidates
            .iter()
            .map(|p| p.iter().zip(vertex).map(|(a, b)| a - b).collect())
            .collect();
        if cube {
            for axis in 0..dim {
                let mut e = vec![Integer::from(0); dim];
                e[axis] = 1.into();
                inequalities.push(e);
            }
        }
        for simplex in pulling(&rays, &inequalities, monitor)? {
            let chart_matrix: Vec<IntVector> = (0..dim)
                .map(|i| simplex.iter().map(|&j| rays[j][i].clone()).collect())
                .collect();
            let det = determinant(&chart_matrix)?;
            if det.is_zero() {
                return Err(SectorError::Geometry(
                    "singular sector transformation".into(),
                ));
            }
            let jacobian_powers = (0..dim)
                .map(|j| {
                    chart_matrix
                        .iter()
                        .fold(Integer::from(-1), |s, row| s + &row[j])
                })
                .collect();
            let factor_valuations: Vec<IntVector> = supports
                .iter()
                .map(|s| {
                    let images: Vec<IntVector> = s
                        .exponents()
                        .iter()
                        .map(|exp| {
                            (0..dim)
                                .map(|j| {
                                    exp.iter()
                                        .enumerate()
                                        .fold(Integer::from(0), |acc, (i, e)| {
                                            acc + e * &chart_matrix[i][j]
                                        })
                                })
                                .collect()
                        })
                        .collect();
                    let valuation: IntVector = (0..dim)
                        .map(|j| images.iter().map(|row| row[j].clone()).min().unwrap())
                        .collect();
                    if !images.contains(&valuation) {
                        return Err(SectorError::Geometry(
                            "a transformed factor has no constant residual".into(),
                        ));
                    }
                    Ok(valuation)
                })
                .collect::<Result<_, SectorError>>()?;
            let mut exponent_matrix = chart_matrix;
            if let Some(axis) = fixed {
                exponent_matrix.insert(axis, vec![Integer::from(0); dim]);
            }
            output.sectors.push(SectorMap {
                fixed_parameter: fixed,
                exponent_matrix,
                determinant: det,
                jacobian_powers,
                factor_valuations,
            });
            if output.sectors.len() > monitor.options.max_sectors {
                return Err(SectorError::ResourceLimit {
                    resource: "sectors",
                    limit: monitor.options.max_sectors,
                });
            }
            monitor.status.sectors = output.sectors.len();
        }
    }
    if output.sectors.is_empty() {
        return Err(SectorError::Geometry(
            "normal fan produced no full-dimensional sectors".into(),
        ));
    }
    Ok(())
}
