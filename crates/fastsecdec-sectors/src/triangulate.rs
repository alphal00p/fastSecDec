//! Pulling triangulations use existing extreme rays and exact face incidences.
use crate::{
    SectorError,
    arithmetic::{IntVector, dot, rank},
    types::Monitor,
};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::utils::resources;

pub(crate) fn pulling(
    rays: &[IntVector],
    inequalities: &[IntVector],
    monitor: &mut Monitor<'_>,
) -> Result<Vec<Vec<usize>>, SectorError> {
    resources::checkpoint()?;
    let face: Vec<_> = (0..rays.len()).collect();
    let dim = rank(rays)?;
    let incidence: Vec<Vec<usize>> = inequalities
        .iter()
        .map(|h| {
            resources::checkpoint()?;
            let mut active = Vec::new();
            for (i, ray) in rays.iter().enumerate() {
                resources::checkpoint()?;
                if dot(h, ray).is_zero() {
                    active.push(i);
                }
            }
            Ok(active)
        })
        .collect::<Result<_, SectorError>>()?;
    recurse(&face, dim, rays, &incidence, &mut BTreeMap::new(), monitor)
}

fn recurse(
    face: &[usize],
    dim: usize,
    rays: &[IntVector],
    incidence: &[Vec<usize>],
    cache: &mut BTreeMap<Vec<usize>, Vec<Vec<usize>>>,
    monitor: &mut Monitor<'_>,
) -> Result<Vec<Vec<usize>>, SectorError> {
    monitor.emit()?;
    if let Some(v) = cache.get(face) {
        let cached = v.clone();
        resources::checkpoint()?;
        return Ok(cached);
    }
    if face.len() == dim {
        return Ok(vec![face.to_vec()]);
    }
    if dim == 0 || face.len() < dim {
        return Err(SectorError::Geometry("invalid pulling face".into()));
    }
    let apex = face[0];
    let mut facets = BTreeSet::<Vec<usize>>::new();
    for active in incidence {
        resources::checkpoint()?;
        let intersection: Vec<_> = face
            .iter()
            .copied()
            .filter(|i| active.binary_search(i).is_ok())
            .collect();
        if intersection.contains(&apex)
            || intersection.len() < dim - 1
            || intersection.len() == face.len()
        {
            continue;
        }
        let rows: Vec<_> = intersection.iter().map(|&i| rays[i].clone()).collect();
        if rank(&rows)? == dim - 1 {
            facets.insert(intersection);
        }
    }
    if facets.is_empty() {
        return Err(SectorError::Geometry(
            "pulling triangulation found no opposite facets".into(),
        ));
    }
    let mut result = Vec::new();
    for facet in facets {
        resources::checkpoint()?;
        for mut simplex in recurse(&facet, dim - 1, rays, incidence, cache, monitor)? {
            simplex.push(apex);
            simplex.sort();
            result.push(simplex);
            if let Some(limit) = monitor.options.max_sectors
                && result.len() > limit
            {
                return Err(SectorError::ResourceLimit {
                    resource: "triangulation simplices",
                    limit,
                });
            }
        }
    }
    result.sort();
    result.dedup();
    resources::checkpoint()?;
    cache.insert(face.to_vec(), result.clone());
    resources::checkpoint()?;
    Ok(result)
}
