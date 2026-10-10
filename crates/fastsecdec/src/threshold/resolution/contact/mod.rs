//! Constructive ordinary contact quotients and differential coefficient ideals.
//!
//! The native graph frame, unit guards and full inclusive derivative tower are
//! checked. Normal-only jets remain separate diagnostics. No boundary history,
//! full BM invariant, global center gluing or real branch atlas is inferred.
mod coefficients;
mod embedding;
#[cfg(test)]
mod tests;
use super::{
    Budget, Error, EtaleCertificate, EtaleFrame, Guard, Ideal, LocalizedAlgebra, MarkedIdeal, Poly,
    ProducedContactCover,
};
pub(crate) use embedding::clear_units;
pub use embedding::{RingExtension, UnitClearing};
use std::sync::Arc;
use symbolica::{
    atom::Symbol,
    domains::{integer::gcd_unsigned, rational::Rational},
};
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct ContactJet {
    pub order: usize,
    pub ambient: Ideal,
    pub restricted: MarkedIdeal,
}
#[derive(Clone, Debug, Default)]
pub struct ContactProgress {
    pub stage: &'static str,
    pub verified_graph: Option<Arc<EtaleFrame>>,
    pub verified_contact: Option<Arc<EtaleFrame>>,
    pub completed_jets: Vec<ContactJet>,
    pub completed_differential_layers: Vec<ContactJet>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct ContactQuotient {
    source: Arc<ProducedContactCover>,
    candidate_index: usize,
    extension: RingExtension,
    clearing: UnitClearing,
    graph: Arc<EtaleFrame>,
    contact: Arc<EtaleFrame>,
    contact_axis: usize,
    normal_index: usize,
    normal_coefficient: MarkedIdeal,
    differential_coefficient: MarkedIdeal,
    progress: ContactProgress,
}
impl ContactQuotient {
    pub fn source(&self) -> &Arc<ProducedContactCover> {
        &self.source
    }
    pub fn candidate_index(&self) -> usize {
        self.candidate_index
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn clearing(&self) -> &UnitClearing {
        &self.clearing
    }
    pub fn graph(&self) -> &Arc<EtaleFrame> {
        &self.graph
    }
    pub fn contact(&self) -> &Arc<EtaleFrame> {
        &self.contact
    }
    pub fn contact_axis(&self) -> usize {
        self.contact_axis
    }
    pub fn normal_derivative_index(&self) -> usize {
        self.normal_index
    }
    /// Normal jets only, retained for diagnostics, not BM invariant recursion.
    pub fn normal_coefficient(&self) -> &MarkedIdeal {
        &self.normal_coefficient
    }
    /// Inclusive full relative derivative presentation, restricted to contact.
    /// This is the ordinary E=empty coefficient construction, with no boundary
    /// history or center/gluing theorem inferred by the constructor.
    pub fn differential_coefficient(&self) -> &MarkedIdeal {
        &self.differential_coefficient
    }
    pub fn progress(&self) -> &ContactProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum ContactProduction {
    Constructed(Box<ContactQuotient>),
    /// The graph with z=0 is the unit ideal on its selected derivative open.
    EmptyContactOpen {
        source: Arc<ProducedContactCover>,
        candidate_index: usize,
        graph: Arc<EtaleFrame>,
        contact_axis: usize,
        progress: ContactProgress,
    },
    /// Completed native jet ideals and source evidence survive a resource stop.
    Incomplete {
        source: Arc<ProducedContactCover>,
        candidate_index: usize,
        progress: ContactProgress,
        reason: &'static str,
    },
}

/// Construct one ordinary boundary-free coefficient chart from an actual native
/// contact-open production. New symbols must be globally fresh in this ring.
/// This is not a boundary/history stage, real-sheet selection or a BM algorithm.
pub fn construct_contact_quotient(
    source: Arc<ProducedContactCover>,
    candidate_index: usize,
    fresh: [Symbol; 2],
    budget: &mut Budget,
) -> Result<ContactProduction> {
    if candidate_index >= source.candidates().len() {
        return Err(Error::Invalid("contact candidate index"));
    }
    let mut progress = ContactProgress::default();
    let result = construct(&source, candidate_index, fresh, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(Built::Contact(mut q)) => {
            q.progress = progress;
            Ok(ContactProduction::Constructed(q))
        }
        Ok(Built::Empty(graph, contact_axis)) => Ok(ContactProduction::EmptyContactOpen {
            source,
            candidate_index,
            graph,
            contact_axis,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(ContactProduction::Incomplete {
            source,
            candidate_index,
            progress,
            reason,
        }),
        Err(e) => Err(e),
    }
}
enum Built {
    Contact(Box<ContactQuotient>),
    Empty(Arc<EtaleFrame>, usize),
}
fn indices(ideal: &Ideal, selected: &[Poly]) -> Result<Vec<usize>> {
    selected
        .iter()
        .map(|p| {
            ideal
                .generators()
                .iter()
                .position(|q| q == p)
                .ok_or(Error::Invalid("selected graph equation disappeared"))
        })
        .collect()
}
pub(crate) fn restrict(p: &Poly, z: usize, budget: &mut Budget) -> Result<Poly> {
    budget.poly(p)?;
    budget.charge(1)?;
    let p = p.replace(z, &Rational::zero());
    budget.poly(&p)?;
    Ok(p)
}
fn construct(
    source: &Arc<ProducedContactCover>,
    candidate_index: usize,
    fresh: [Symbol; 2],
    budget: &mut Budget,
    progress: &mut ContactProgress,
) -> Result<Built> {
    progress.stage = "mark preflight";
    let d = source.algebraic_maximum_order();
    if d == 0 || d > budget.limits.max_mark {
        return Err(Error::ResourceIncomplete("contact order bound"));
    }
    let mut common = 1usize;
    for m in 1..=d {
        let gcd = gcd_unsigned(
            u64::try_from(common).map_err(|_| Error::ResourceIncomplete("mark conversion"))?,
            u64::try_from(m).map_err(|_| Error::ResourceIncomplete("mark conversion"))?,
        ) as usize;
        common = (common / gcd)
            .checked_mul(m)
            .ok_or(Error::ResourceIncomplete("coefficient mark LCM"))?;
        if common > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("coefficient mark LCM"));
        }
    }
    budget.reserve_slots(
        d.checked_mul(source.source().generators().len())
            .ok_or(Error::ResourceIncomplete("contact jet count"))?,
    )?;
    progress.stage = "unit clearing";
    let old = source.frame();
    let candidate = &source.candidates()[candidate_index];
    let old_clearing = clear_units(old.local(), &candidate.equation, budget)?;
    let extension = RingExtension::new(old.local().ring().clone(), fresh, budget)?;
    let ring = extension.target().clone();
    let z = extension.source().len();
    let inverse = z + 1;
    let clearing = UnitClearing {
        original: extension.pull(&old_clearing.original, budget)?,
        numerator: extension.pull(&old_clearing.numerator, budget)?,
        denominator: extension.pull(&old_clearing.denominator, budget)?,
    };
    let equation =
        &clearing.numerator - &budget.mul(&ring.coordinate(z)?, &clearing.denominator)?;
    let inherited = extension.ideal(old.local().ideal(), budget)?;
    let mut selected = old
        .selected_equations()
        .iter()
        .map(|i| extension.pull(&old.source().ideal().generators()[*i], budget))
        .collect::<Result<Vec<_>>>()?;
    selected.push(equation.clone());
    let relations = inherited.sum(&Ideal::new(ring.clone(), vec![equation], budget)?, budget)?;
    let selected_indices = indices(&relations, &selected)?;
    let mut axes = old.local().axes().to_vec();
    axes.push(z);
    let guards = extension.guards(old.local().guards(), budget)?;
    let local = LocalizedAlgebra::new(relations, axes, guards, budget)?;
    let chosen = old.free_axes()[candidate.free_derivative_index];
    let mut dependent = old.dependent_axes().to_vec();
    dependent.push(chosen);
    let mut free = old
        .free_axes()
        .iter()
        .copied()
        .filter(|i| *i != chosen)
        .collect::<Vec<_>>();
    let normal = free.len();
    free.push(z);
    progress.stage = "graph frame";
    let graph = Arc::new(
        EtaleCertificate {
            source: local,
            equations: selected_indices,
            dependent_axes: dependent.clone(),
            free_axes: free.clone(),
            determinant_inverse_axis: Some(inverse),
        }
        .verify(budget)?,
    );
    // The block determinant/Schur-complement identity certifies that this is
    // exactly the old chart localized at D_i h, rather than an extra open.
    let old_minor = extension.pull(old.determinant(), budget)?;
    let old_differential = extension.pull(&candidate.differential, budget)?;
    let expected = budget.mul(&old_minor, &clearing.denominator)?;
    let expected = budget.mul(&expected, &old_differential)?;
    if !graph
        .local()
        .zero(&(graph.determinant() - &expected), budget)?
    {
        return Err(Error::Invalid("contact minor/open identity"));
    }
    if !graph.local().zero(
        &(graph.derivative(normal, &clearing.original, budget)? - ring.one()),
        budget,
    )? {
        return Err(Error::Invalid("contact normal derivative"));
    }
    for index in 0..normal {
        if !graph.local().zero(
            &graph.derivative(index, &clearing.original, budget)?,
            budget,
        )? {
            return Err(Error::Invalid("contact tangential derivative"));
        }
    }
    progress.verified_graph = Some(graph.clone());
    progress.stage = "contact restriction";
    // Prove an empty contact open explicitly before constructor admission.
    let with_contact = graph.local().ideal().sum(
        &Ideal::new(ring.clone(), vec![ring.coordinate(z)?], budget)?,
        budget,
    )?;
    if with_contact.contains(&ring.one(), graph.local().unit_relations(), budget)? {
        return Ok(Built::Empty(graph, z));
    }
    let equations = graph
        .source()
        .ideal()
        .generators()
        .iter()
        .map(|p| restrict(p, z, budget))
        .collect::<Result<Vec<_>>>()?;
    let selected = selected
        .iter()
        .map(|p| restrict(p, z, budget))
        .collect::<Result<Vec<_>>>()?;
    let relations = Ideal::new(ring.clone(), equations, budget)?;
    let selected_indices = indices(&relations, &selected)?;
    // Use pre-new-minor guards; verification adds the restricted determinant
    // with the same inverse slot, preventing duplicate inverse roles.
    let guards = graph
        .source()
        .guards()
        .iter()
        .map(|g| {
            Ok(Guard {
                factor: restrict(&g.factor, z, budget)?,
                inverse_axis: g.inverse_axis,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let axes = graph
        .source()
        .axes()
        .iter()
        .copied()
        .filter(|i| *i != z)
        .collect();
    let local = LocalizedAlgebra::new(relations, axes, guards, budget)?;
    free.pop();
    let contact = Arc::new(
        EtaleCertificate {
            source: local,
            equations: selected_indices,
            dependent_axes: dependent,
            free_axes: free,
            determinant_inverse_axis: Some(inverse),
        }
        .verify(budget)?,
    );
    if contact.determinant() != &restrict(graph.determinant(), z, budget)? {
        return Err(Error::Invalid("restricted contact minor"));
    }
    progress.verified_contact = Some(contact.clone());
    progress.stage = "normal coefficient jets";
    let initial = extension.ideal(source.source(), budget)?;
    let normal_coefficient = coefficients::build(
        &graph,
        &contact,
        &initial,
        z,
        d,
        coefficients::Differentiation::Normal(normal),
        &mut progress.completed_jets,
        budget,
    )?;
    progress.stage = "inclusive differential coefficient";
    let differential_coefficient = coefficients::build(
        &graph,
        &contact,
        &initial,
        z,
        d,
        coefficients::Differentiation::FullInclusive,
        &mut progress.completed_differential_layers,
        budget,
    )?;
    progress.stage = "complete";
    Ok(Built::Contact(Box::new(ContactQuotient {
        source: source.clone(),
        candidate_index,
        extension,
        clearing,
        graph,
        contact,
        contact_axis: z,
        normal_index: normal,
        normal_coefficient,
        differential_coefficient,
        progress: ContactProgress::default(),
    })))
}
