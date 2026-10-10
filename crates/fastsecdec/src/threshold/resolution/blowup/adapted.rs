use super::super::{contact::clear_units, *};
use super::helpers::*;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct AdaptedOpen {
    pub(crate) source: Arc<VerifiedRelativeSnc>,
    pub(crate) incidence: Vec<usize>,
    pub(crate) columns: Vec<usize>,
    pub(crate) extension: Extension,
    pub(crate) graph: Arc<EtaleFrame>,
    pub(crate) boundary_axes: Vec<usize>,
    pub(crate) relative_minor: Poly,
    /// Source-local principal open, including every absent divisor.
    source_open: Poly,
    /// det(d adapted free coordinates / d original free coordinates), before
    /// the standard blowup. Its inverse is required for density pullbacks.
    forward_coordinate_jacobian: Poly,
}
impl AdaptedOpen {
    pub fn source(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.source
    }
    pub fn incidence(&self) -> &[usize] {
        &self.incidence
    }
    pub fn columns(&self) -> &[usize] {
        &self.columns
    }
    pub fn graph(&self) -> &Arc<EtaleFrame> {
        &self.graph
    }
    pub fn boundary_axes(&self) -> &[usize] {
        &self.boundary_axes
    }
    pub fn relative_minor(&self) -> &Poly {
        &self.relative_minor
    }
    pub fn source_open(&self) -> &Poly {
        &self.source_open
    }
    pub fn forward_coordinate_jacobian(&self) -> &Poly {
        &self.forward_coordinate_jacobian
    }
}
#[derive(Clone, Debug)]
pub struct EmptyAdaptedOpen {
    pub incidence: Vec<usize>,
    pub columns: Vec<usize>,
    pub ideal: Ideal,
    pub guards: Vec<Guard>,
}
#[derive(Clone, Debug, Default)]
pub struct AdaptedProgress {
    pub completed: Vec<Arc<AdaptedOpen>>,
    pub empty: Vec<EmptyAdaptedOpen>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct AdaptedCover {
    source: Arc<VerifiedRelativeSnc>,
    proof: VerifiedOpenCover,
    progress: AdaptedProgress,
}
impl AdaptedCover {
    pub fn source(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.source
    }
    pub fn opens(&self) -> &[Arc<AdaptedOpen>] {
        &self.progress.completed
    }
    pub fn progress(&self) -> &AdaptedProgress {
        &self.progress
    }
    pub fn proof(&self) -> &VerifiedOpenCover {
        &self.proof
    }
}
#[derive(Clone, Debug)]
pub enum AdaptedProduction {
    Complete(Arc<AdaptedCover>),
    Incomplete {
        source: Arc<VerifiedRelativeSnc>,
        reason: &'static str,
        progress: AdaptedProgress,
    },
}
pub fn produce_adapted_cover(
    source: Arc<VerifiedRelativeSnc>,
    namespace: &str,
    b: &mut Budget,
) -> Result<AdaptedProduction> {
    let mut progress = AdaptedProgress::default();
    let result = build(&source, namespace, b, &mut progress);
    progress.operations = b.operations();
    progress.ideal_slots = b.ideal_slots();
    match result {
        Ok(proof) => Ok(AdaptedProduction::Complete(Arc::new(AdaptedCover {
            source,
            proof,
            progress,
        }))),
        Err(Error::ResourceIncomplete(reason)) => Ok(AdaptedProduction::Incomplete {
            source,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn build(
    source: &Arc<VerifiedRelativeSnc>,
    namespace: &str,
    b: &mut Budget,
    progress: &mut AdaptedProgress,
) -> Result<VerifiedOpenCover> {
    let n = source.divisors().len();
    let shift = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("incidence count"))?;
    let count = 1usize
        .checked_shl(shift)
        .ok_or(Error::ResourceIncomplete("incidence count"))?;
    b.reserve_slots(count)?;
    for mask in 0..count {
        let incidence = (0..n).filter(|i| mask & (1 << i) != 0).collect::<Vec<_>>();
        if incidence.len() > source.frame().free_axes().len() {
            continue;
        }
        // The SNC certificate explicitly certifies every omitted over-rank
        // intersection empty; no real-point heuristic is used.
        if !incidence.is_empty()
            && source
                .intersection(&incidence)
                .ok_or(Error::Invalid("missing SNC intersection"))?
                .empty
        {
            continue;
        }
        for columns in combinations(source.frame().free_axes().len(), incidence.len(), b)? {
            let index = progress.completed.len() + progress.empty.len();
            let outcome = one(
                source,
                &incidence,
                &columns,
                &format!("{namespace}_open{index}"),
                b,
            )?;
            b.reserve_slots(1)?;
            match outcome {
                Ok(open) => progress.completed.push(Arc::new(open)),
                Err(empty) => progress.empty.push(empty),
            }
        }
    }
    if progress.completed.is_empty() {
        return Err(Error::Invalid("SNC cover unexpectedly empty"));
    }
    OpenCoverCertificate {
        algebra: source.frame().local().clone(),
        support: Ideal::new(source.frame().local().ring().clone(), vec![], b)?,
        opens: progress
            .completed
            .iter()
            .map(|v| v.source_open.clone())
            .collect(),
    }
    .verify(b)
}
fn one(
    source: &Arc<VerifiedRelativeSnc>,
    incidence: &[usize],
    columns: &[usize],
    namespace: &str,
    b: &mut Budget,
) -> Result<std::result::Result<AdaptedOpen, EmptyAdaptedOpen>> {
    let old = source.frame();
    let k = incidence.len();
    let n = source.divisors().len();
    // k graph slots, n-k complement inverses, one graph minor inverse.
    let extension = Extension::new(
        old.local().ring().clone(),
        n.checked_add(1)
            .ok_or(Error::ResourceIncomplete("graph slots"))?,
        namespace,
        b,
    )?;
    let ring = extension.target();
    let start = extension.source().len();
    let boundary_axes = (start..start + k).collect::<Vec<_>>();
    let entries = incidence
        .iter()
        .flat_map(|i| columns.iter().map(move |j| (*i, *j)))
        .map(|(i, j)| old.derivative(j, &source.divisors()[i].equation, b))
        .collect::<Result<Vec<_>>>()?;
    let relative_minor = determinant(old.local().ring(), entries, k, b)?;
    let mut source_open = relative_minor.clone();
    for (i, divisor) in source.divisors().iter().enumerate() {
        if !incidence.contains(&i) {
            source_open = b.mul(&source_open, &divisor.equation)?;
        }
    }
    let mut forward_entries = Vec::new();
    for (j, _) in old.free_axes().iter().enumerate() {
        if !columns.contains(&j) {
            for l in 0..old.free_axes().len() {
                forward_entries.push(if l == j {
                    old.local().ring().one()
                } else {
                    old.local().ring().one().zero()
                });
            }
        }
    }
    for i in incidence {
        for j in 0..old.free_axes().len() {
            forward_entries.push(old.derivative(j, &source.divisors()[*i].equation, b)?);
        }
    }
    let forward_coordinate_jacobian = determinant(
        old.local().ring(),
        forward_entries,
        old.free_axes().len(),
        b,
    )?;
    let clearing = incidence
        .iter()
        .map(|i| clear_units(old.local(), &source.divisors()[*i].equation, b))
        .collect::<Result<Vec<_>>>()?;
    let mut selected = old
        .selected_equations()
        .iter()
        .map(|i| extension.pull(&old.source().ideal().generators()[*i], b))
        .collect::<Result<Vec<_>>>()?;
    let mut equations = Vec::new();
    let mut denominator = ring.one();
    for (i, c) in clearing.iter().enumerate() {
        let u = extension.pull(&c.denominator, b)?;
        denominator = b.mul(&denominator, &u)?;
        equations.push(
            &extension.pull(&c.numerator, b)? - &b.mul(&ring.coordinate(boundary_axes[i])?, &u)?,
        );
    }
    selected.extend(equations.iter().cloned());
    let relations = extension
        .ideal(old.local().ideal(), b)?
        .sum(&Ideal::new(ring.clone(), equations, b)?, b)?;
    let mut guards = extension.guards(old.local().guards(), b)?;
    let mut slot = start + k;
    for i in 0..n {
        if !incidence.contains(&i) {
            let cleared = clear_units(old.local(), &source.divisors()[i].equation, b)?;
            guards.push(Guard {
                factor: extension.pull(&cleared.numerator, b)?,
                inverse_axis: slot,
            });
            slot += 1;
        }
    }
    let old_minor = extension.pull(old.determinant(), b)?;
    let pulled_minor = extension.pull(&relative_minor, b)?;
    let expected = b.mul(&old_minor, &denominator)?;
    let expected = b.mul(&expected, &pulled_minor)?;
    // Preflight the exact open including its future minor before invoking
    // LocalizedAlgebra's nonempty constructor.
    let minor_clearing = clear_units(old.local(), &relative_minor, b)?;
    let mut probe_guards = guards.clone();
    probe_guards.push(Guard {
        factor: extension.pull(&minor_clearing.numerator, b)?,
        inverse_axis: slot,
    });
    if empty(&relations, &probe_guards, b)? {
        return Ok(Err(EmptyAdaptedOpen {
            incidence: incidence.to_vec(),
            columns: columns.to_vec(),
            ideal: relations,
            guards: probe_guards,
        }));
    }
    let mut axes = old.local().axes().to_vec();
    axes.extend(&boundary_axes);
    let local = LocalizedAlgebra::new(relations.clone(), axes, guards, b)?;
    let mut dependent = old.dependent_axes().to_vec();
    dependent.extend(columns.iter().map(|j| old.free_axes()[*j]));
    let mut free = old
        .free_axes()
        .iter()
        .enumerate()
        .filter(|(j, _)| !columns.contains(j))
        .map(|(_, a)| *a)
        .collect::<Vec<_>>();
    free.extend(&boundary_axes);
    let graph = Arc::new(
        EtaleCertificate {
            source: local,
            equations: indices(&relations, &selected)?,
            dependent_axes: dependent,
            free_axes: free,
            determinant_inverse_axis: if selected.is_empty() {
                None
            } else {
                Some(slot)
            },
        }
        .verify(b)?,
    );
    if !graph.local().zero(&(graph.determinant() - &expected), b)? {
        return Err(Error::Invalid("adapted graph Schur minor"));
    }
    if !unit(
        graph.local(),
        &extension.pull(&forward_coordinate_jacobian, b)?,
        b,
    )? {
        return Err(Error::Invalid("adapted coordinate Jacobian not unit"));
    }
    for (i, c) in clearing.iter().enumerate() {
        let h = extension.pull(&c.original, b)?;
        if !graph
            .local()
            .zero(&(&h - &ring.coordinate(boundary_axes[i])?), b)?
        {
            return Err(Error::Invalid("adapted coordinate identity"));
        }
        for (j, axis) in graph.free_axes().iter().enumerate() {
            let want = if *axis == boundary_axes[i] {
                ring.one()
            } else {
                ring.one().zero()
            };
            if !graph
                .local()
                .zero(&(graph.derivative(j, &h, b)? - want), b)?
            {
                return Err(Error::Invalid("adapted derivative identity"));
            }
        }
    }
    for i in 0..n {
        if !incidence.contains(&i)
            && !unit(
                graph.local(),
                &extension.pull(&source.divisors()[i].equation, b)?,
                b,
            )?
        {
            return Err(Error::Invalid("complement divisor not unit"));
        }
    }
    Ok(Ok(AdaptedOpen {
        source: source.clone(),
        incidence: incidence.to_vec(),
        columns: columns.to_vec(),
        extension,
        graph,
        boundary_axes,
        relative_minor,
        source_open,
        forward_coordinate_jacobian,
    }))
}
