use super::super::*;
use super::helpers::*;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct AdaptedOpen {
    pub(super) geometry: Arc<super::graph::GraphCoordinateOpen>,
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
    let functions = incidence
        .iter()
        .map(|i| source.divisors()[*i].equation.clone())
        .collect::<Vec<_>>();
    let extra_units = source
        .divisors()
        .iter()
        .enumerate()
        .filter(|(i, _)| !incidence.contains(i))
        .map(|(_, d)| d.equation.clone())
        .collect::<Vec<_>>();
    let geometry = match super::graph::graph_coordinates(
        source.frame().clone(),
        &functions,
        columns,
        &extra_units,
        namespace,
        b,
    )? {
        Ok(v) => v,
        Err(e) => {
            return Ok(Err(EmptyAdaptedOpen {
                incidence: incidence.to_vec(),
                columns: columns.to_vec(),
                ideal: e.ideal,
                guards: e.guards,
            }));
        }
    };
    Ok(Ok(AdaptedOpen {
        source: source.clone(),
        incidence: incidence.to_vec(),
        columns: columns.to_vec(),
        extension: geometry.extension.clone(),
        graph: geometry.graph.clone(),
        boundary_axes: geometry.coordinate_axes.clone(),
        relative_minor: geometry.relative_minor.clone(),
        source_open: geometry.source_open.clone(),
        forward_coordinate_jacobian: geometry.forward_coordinate_jacobian.clone(),
        geometry,
    }))
}
