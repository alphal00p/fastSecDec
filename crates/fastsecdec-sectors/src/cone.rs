//! Exact double description for full-dimensional pointed cones H r >= 0.
use crate::{
    SectorError,
    arithmetic::{IntVector, dot, integer_ray, matrix, primitive, rank},
    types::{DecompositionPhase, Monitor},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Ray {
    vector: IntVector,
    zeros: BTreeSet<usize>,
}

pub(crate) fn extreme_rays(
    mut constraints: Vec<IntVector>,
    monitor: &mut Monitor<'_>,
) -> Result<Vec<IntVector>, SectorError> {
    if constraints.is_empty() {
        return Err(SectorError::Geometry("empty cone constraints".into()));
    }
    let dim = constraints[0].len();
    if dim > monitor.options.max_rays {
        return Err(SectorError::ResourceLimit {
            resource: "cone rays",
            limit: monitor.options.max_rays,
        });
    }
    constraints = constraints
        .into_iter()
        .map(primitive)
        .filter(|r| r.iter().any(|v| !v.is_zero()))
        .collect();
    constraints.sort();
    constraints.dedup();
    let mut basis = Vec::new();
    let mut basis_indices = BTreeSet::new();
    for (i, row) in constraints.iter().enumerate() {
        let mut candidate = basis.clone();
        candidate.push(row.clone());
        if rank(&candidate) > basis.len() {
            basis.push(row.clone());
            basis_indices.insert(i);
        }
        if basis.len() == dim {
            break;
        }
    }
    if basis.len() != dim {
        return Err(SectorError::Geometry("cone is not pointed".into()));
    }
    let inverse = matrix(&basis)?
        .inv()
        .map_err(|e| SectorError::Geometry(e.to_string()))?;
    let mut rays: Vec<Ray> = (0..dim)
        .map(|j| {
            let vector = integer_ray(
                (0..dim)
                    .map(|i| inverse[(i as u32, j as u32)].clone())
                    .collect(),
            );
            let zeros = basis_indices
                .iter()
                .copied()
                .filter(|&i| dot(&constraints[i], &vector).is_zero())
                .collect();
            Ray { vector, zeros }
        })
        .collect();
    let mut pending: BTreeSet<_> = (0..constraints.len())
        .filter(|i| !basis_indices.contains(i))
        .collect();
    monitor.status.phase = DecompositionPhase::Facets;
    monitor.status.total_constraints = constraints.len();
    while !pending.is_empty() {
        monitor.status.completed_constraints = constraints.len() - pending.len();
        monitor.status.rays = rays.len();
        monitor.emit()?;
        // Dynamic minimum-pair insertion reduces intermediate ray growth.
        // The index tie-break makes output independent of hash/thread order.
        let next = pending
            .iter()
            .copied()
            .min_by_key(|&i| {
                let (p, n) = rays.iter().fold((0usize, 0usize), |(p, n), r| {
                    let v = dot(&constraints[i], &r.vector);
                    (p + usize::from(v > 0), n + usize::from(v < 0))
                });
                (p.saturating_mul(n), n, i)
            })
            .unwrap();
        pending.remove(&next);
        let values: Vec<_> = rays
            .iter()
            .map(|r| dot(&constraints[next], &r.vector))
            .collect();
        let positive: Vec<_> = (0..rays.len()).filter(|&i| values[i] > 0).collect();
        let negative: Vec<_> = (0..rays.len()).filter(|&i| values[i] < 0).collect();
        let mut out = BTreeMap::<IntVector, BTreeSet<usize>>::new();
        for (i, ray) in rays.iter().enumerate() {
            if values[i] < 0 {
                continue;
            }
            let mut zeros = ray.zeros.clone();
            if values[i].is_zero() {
                zeros.insert(next);
            }
            out.insert(ray.vector.clone(), zeros);
        }
        for &p in &positive {
            monitor.emit()?;
            for &n in &negative {
                let shared = rays[p].zeros.intersection(&rays[n].zeros);
                // A two-dimensional face has codimension dim-2 in this cone.
                // Its common active constraints must have at least that rank,
                // hence at least that cardinality. Reject smaller intersections
                // before allocating a set or scanning every other ray. This is
                // only a necessary condition; the exact adjacency test follows.
                let minimum_active = dim.saturating_sub(2);
                if shared.clone().take(minimum_active).count() < minimum_active {
                    continue;
                }
                let mut common: BTreeSet<_> = shared.copied().collect();
                // Two rays span an edge iff no third ray contains their common
                // active facets. This is the exact combinatorial DD test.
                if rays
                    .iter()
                    .enumerate()
                    .any(|(k, r)| k != p && k != n && common.is_subset(&r.zeros))
                {
                    continue;
                }
                let vector = primitive(
                    rays[n]
                        .vector
                        .iter()
                        .zip(&rays[p].vector)
                        .map(|(a, b)| &values[p] * a - &values[n] * b)
                        .collect(),
                );
                common.insert(next);
                out.insert(vector, common);
                if out.len() > monitor.options.max_rays {
                    return Err(SectorError::ResourceLimit {
                        resource: "cone rays",
                        limit: monitor.options.max_rays,
                    });
                }
            }
        }
        rays = out
            .into_iter()
            .map(|(vector, zeros)| Ray { vector, zeros })
            .collect();
    }
    monitor.status.completed_constraints = constraints.len();
    monitor.status.rays = rays.len();
    monitor.emit()?;
    Ok(rays.into_iter().map(|r| r.vector).collect())
}
