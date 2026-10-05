//! Shared native chart/cone seams. Serial and caller-scheduled paths use these.
use crate::{
    arithmetic::{IntVector, determinant, dot, rank},
    cone::extreme_rays,
    support::minkowski_candidates,
    triangulate::pulling,
    types::Monitor,
    *,
};
use numerica::domains::integer::Integer;

#[cfg(test)]
std::thread_local! {
    pub(crate) static FACET_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn validate_input(
    domain: ParametricDomain,
    supports: &[PolynomialSupport],
) -> Result<usize, SectorError> {
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
    Ok(dim)
}

pub(crate) struct ChartData {
    pub supports: Vec<PolynomialSupport>,
    pub fixed: Option<usize>,
    cube: bool,
    pub candidates: Vec<IntVector>,
    facets: Vec<IntVector>,
}

pub(crate) fn prepare_chart(
    supports: Vec<PolynomialSupport>,
    fixed: Option<usize>,
    cube: bool,
    monitor: &mut Monitor<'_>,
) -> Result<ChartData, SectorError> {
    let dim = supports[0].dimension();
    if dim == 0 {
        monitor.emit()?;
        return Ok(ChartData {
            supports,
            fixed,
            cube,
            candidates: Vec::new(),
            facets: Vec::new(),
        });
    }
    let candidates = minkowski_candidates(&supports, monitor)?;
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
    #[cfg(test)]
    FACET_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let facets = extreme_rays(generators, monitor)?;
    Ok(ChartData {
        supports,
        fixed,
        cube,
        candidates,
        facets,
    })
}

pub(crate) fn trivial_map(chart: &ChartData) -> SectorMap {
    SectorMap {
        fixed_parameter: chart.fixed,
        exponent_matrix: vec![vec![]; usize::from(chart.fixed.is_some())],
        determinant: 1.into(),
        jacobian_powers: vec![],
        factor_valuations: vec![vec![]; chart.supports.len()],
    }
}

/// Append native maps in order. On error the caller retains the successful
/// prefix, so a later conversion error cannot mask an earlier cumulative limit.
pub(crate) fn decompose_candidate(
    chart: &ChartData,
    candidate: usize,
    output: &mut Vec<SectorMap>,
    monitor: &mut Monitor<'_>,
) -> Result<bool, SectorError> {
    let dim = chart.supports[0].dimension();
    let supports = &chart.supports;
    let candidates = &chart.candidates;
    let facets = &chart.facets;
    let fixed = chart.fixed;
    let cube = chart.cube;
    let vertex = &candidates[candidate];
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
        return Ok(false);
    }
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
        output.push(SectorMap {
            fixed_parameter: fixed,
            exponent_matrix,
            determinant: det,
            jacobian_powers,
            factor_valuations,
        });
        if output.len() > monitor.options.max_sectors {
            return Err(SectorError::ResourceLimit {
                resource: "sectors",
                limit: monitor.options.max_sectors,
            });
        }
        monitor.status.sectors = output.len();
    }
    Ok(true)
}
