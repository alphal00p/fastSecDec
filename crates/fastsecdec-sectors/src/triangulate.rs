//! Pulling triangulations use existing extreme rays and exact face incidences.
use crate::{
    SectorError,
    arithmetic::{IntVector, dot, rank},
    types::Monitor,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn pulling(
    rays: &[IntVector],
    inequalities: &[IntVector],
    monitor: &mut Monitor<'_>,
) -> Result<Vec<Vec<usize>>, SectorError> {
    let face: Vec<_> = (0..rays.len()).collect();
    let dim = rank(rays);
    let incidence: Vec<Vec<usize>> = inequalities
        .iter()
        .map(|h| {
            rays.iter()
                .enumerate()
                .filter_map(|(i, r)| dot(h, r).is_zero().then_some(i))
                .collect()
        })
        .collect();
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
        return Ok(v.clone());
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
        if rank(&rows) == dim - 1 {
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
        for mut simplex in recurse(&facet, dim - 1, rays, incidence, cache, monitor)? {
            simplex.push(apex);
            simplex.sort();
            result.push(simplex);
            if result.len() > monitor.options.max_sectors {
                return Err(SectorError::ResourceLimit {
                    resource: "triangulation simplices",
                    limit: monitor.options.max_sectors,
                });
            }
        }
    }
    result.sort();
    result.dedup();
    cache.insert(face.to_vec(), result.clone());
    Ok(result)
}
