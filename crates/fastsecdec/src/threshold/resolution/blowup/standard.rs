//! One shared standard blowup of checked relative graph coordinates.
use super::super::*;
use super::{graph::GraphCoordinateOpen, helpers::*};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Debug)]
pub(super) struct PolynomialPullback {
    extension: Extension,
    changes: Vec<(usize, Poly)>,
}
impl PolynomialPullback {
    pub fn pull(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        let mut p = self.extension.pull(p, b)?;
        for (axis, image) in &self.changes {
            p = b.substitute(&p, *axis, image)?;
        }
        Ok(p)
    }
}
#[derive(Clone, Debug)]
pub(super) struct StandardGeometry {
    pub open: Arc<GraphCoordinateOpen>,
    pub center_axes: Vec<usize>,
    pub pivot: Option<usize>,
    pub frame: Arc<EtaleFrame>,
    pub pullback: PolynomialPullback,
    pub swaps: Vec<(usize, usize)>,
    pub exceptional: Option<Poly>,
    pub jacobian: Poly,
    pub original_coordinate_jacobian: Poly,
}
impl StandardGeometry {
    pub fn pull_from_source(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        self.pullback.pull(&self.open.extension.pull(p, b)?, b)
    }
}
pub(super) fn standard_chart(
    open: Arc<GraphCoordinateOpen>,
    center_axes: &[usize],
    pivot: Option<usize>,
    namespace: &str,
    b: &mut Budget,
) -> Result<Arc<StandardGeometry>> {
    let old = &open.graph;
    if (center_axes.is_empty() && pivot.is_some())
        || center_axes.iter().copied().collect::<BTreeSet<_>>().len() != center_axes.len()
        || center_axes
            .iter()
            .any(|a| !open.coordinate_axes.contains(a))
        || pivot.is_some_and(|p| !center_axes.contains(&p))
    {
        return Err(Error::Invalid("standard graph center or pivot"));
    }
    let count = if pivot.is_some() {
        center_axes.len() - 1
    } else {
        0
    };
    let extension = Extension::new(old.local().ring().clone(), count + 1, namespace, b)?;
    let ring = extension.target().clone();
    let start = extension.source().len();
    let exceptional = pivot.map(|a| ring.coordinate(a)).transpose()?;
    let mut changes = Vec::new();
    let mut swaps = Vec::new();
    let mut next = start;
    if let Some(pivot) = pivot {
        for axis in center_axes {
            if *axis != pivot {
                let image = b.mul(
                    exceptional
                        .as_ref()
                        .ok_or(Error::Invalid("missing exceptional"))?,
                    &ring.coordinate(next)?,
                )?;
                changes.push((*axis, image));
                swaps.push((*axis, next));
                next += 1;
            }
        }
    }
    for (_, image) in &changes {
        if changes.iter().any(|(axis, _)| image.degree(*axis) > 0) {
            return Err(Error::Invalid("recursive coordinate substitution"));
        }
    }
    let pullback = PolynomialPullback { extension, changes };
    let equations = old
        .source()
        .ideal()
        .generators()
        .iter()
        .map(|p| pullback.pull(p, b))
        .collect::<Result<Vec<_>>>()?;
    let selected = old
        .selected_equations()
        .iter()
        .map(|i| pullback.pull(&old.source().ideal().generators()[*i], b))
        .collect::<Result<Vec<_>>>()?;
    let relations = Ideal::new(ring.clone(), equations, b)?;
    let guards = old
        .local()
        .guards()
        .iter()
        .map(|g| {
            Ok(Guard {
                factor: pullback.pull(&g.factor, b)?,
                inverse_axis: g.inverse_axis,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let remap = |a: &usize| {
        swaps
            .iter()
            .find(|(old, _)| old == a)
            .map_or(*a, |(_, new)| *new)
    };
    let axes = old.source().axes().iter().map(remap).collect::<Vec<_>>();
    let free = old.free_axes().iter().map(remap).collect::<Vec<_>>();
    let local = LocalizedAlgebra::new(relations.clone(), axes, guards, b)?;
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: indices(&relations, &selected)?,
            dependent_axes: old.dependent_axes().to_vec(),
            free_axes: free.clone(),
            determinant_inverse_axis: if selected.is_empty() {
                None
            } else {
                Some(next)
            },
        }
        .verify(b)?,
    );
    if !frame.local().zero(
        &(frame.determinant() - &pullback.pull(old.determinant(), b)?),
        b,
    )? {
        return Err(Error::Invalid("pulled relative etale determinant"));
    }
    // The checked Etale base change over ordinary affine blowup coordinates is
    // flat. Thus this exact graph fast path has no exceptional torsion; a raw
    // arbitrary incidence presentation is not admitted through this function.
    let mut entries = Vec::new();
    for source_axis in old.free_axes() {
        let image = pullback.pull(&old.local().ring().coordinate(*source_axis)?, b)?;
        for target_axis in &free {
            entries.push(image.derivative(*target_axis));
        }
    }
    let jacobian = determinant(&ring, entries, free.len(), b)?;
    let expected = match &exceptional {
        Some(e) => b.power(e, center_axes.len() - 1)?,
        None => ring.one(),
    };
    if jacobian != expected {
        return Err(Error::Invalid("standard blowup determinant"));
    }
    let mut original_entries = Vec::new();
    for axis in open.source.free_axes() {
        let original = open.source.local().ring().coordinate(*axis)?;
        let image = pullback.pull(&open.extension.pull(&original, b)?, b)?;
        for j in 0..frame.free_axes().len() {
            original_entries.push(frame.derivative(j, &image, b)?);
        }
    }
    let original_coordinate_jacobian = determinant(&ring, original_entries, free.len(), b)?;
    let forward = pullback.pull(
        &open.extension.pull(&open.forward_coordinate_jacobian, b)?,
        b,
    )?;
    let composed = b.mul(&original_coordinate_jacobian, &forward)?;
    if !frame.local().zero(&(&composed - &jacobian), b)? {
        return Err(Error::Invalid(
            "full relative coordinate Jacobian composition",
        ));
    }
    Ok(Arc::new(StandardGeometry {
        open,
        center_axes: center_axes.to_vec(),
        pivot,
        frame,
        pullback,
        swaps,
        exceptional,
        jacobian,
        original_coordinate_jacobian,
    }))
}
