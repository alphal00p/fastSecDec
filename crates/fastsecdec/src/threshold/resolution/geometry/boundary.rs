use super::super::{Budget, Error, EtaleFrame, Ideal, Poly};
use std::{collections::BTreeSet, sync::Arc};
use symbolica::{atom::AtomCore, domains::atom::AtomField, tensors::matrix::Matrix};
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BoundaryId(pub u64);
#[derive(Clone, Debug)]
pub struct InitialDivisor {
    pub id: BoundaryId,
    pub equation: Poly,
}
#[derive(Clone, Debug)]
pub struct IntersectionCertificate {
    pub indices: Vec<usize>,
    pub locus: Ideal,
    pub empty: bool,
    /// Full-rank minor opens whose union covers this algebraic locus.
    pub minors: Vec<Poly>,
}
#[derive(Clone, Debug, Default)]
pub struct SncProgress {
    pub intersections: Vec<IntersectionCertificate>,
    pub operations: usize,
    pub ideal_slots: usize,
}
/// Initial ordered divisor geometry only. Initial birth is zero; no unchecked
/// integer history or invented new-exceptional transition can construct this.
#[derive(Clone, Debug)]
pub struct VerifiedRelativeSnc {
    frame: Arc<EtaleFrame>,
    divisors: Arc<Vec<InitialDivisor>>,
    progress: SncProgress,
}
impl VerifiedRelativeSnc {
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn divisors(&self) -> &[InitialDivisor] {
        &self.divisors
    }
    pub fn progress(&self) -> &SncProgress {
        &self.progress
    }
    pub fn initial_old_snapshot(&self) -> Vec<BoundaryId> {
        self.divisors.iter().map(|v| v.id).collect()
    }
    pub fn intersection(&self, indices: &[usize]) -> Option<&IntersectionCertificate> {
        self.progress
            .intersections
            .iter()
            .find(|v| v.indices == indices)
    }
}
#[derive(Clone, Debug)]
pub enum SncProduction {
    Verified(Arc<VerifiedRelativeSnc>),
    Unresolved {
        frame: Arc<EtaleFrame>,
        divisors: Arc<Vec<InitialDivisor>>,
        indices: Vec<usize>,
        obstruction: Ideal,
        progress: SncProgress,
    },
    Incomplete {
        frame: Arc<EtaleFrame>,
        divisors: Arc<Vec<InitialDivisor>>,
        reason: &'static str,
        progress: SncProgress,
    },
}
pub fn verify_initial_relative_snc(
    frame: Arc<EtaleFrame>,
    divisors: Vec<InitialDivisor>,
    budget: &mut Budget,
) -> Result<SncProduction> {
    if divisors.windows(2).any(|d| d[0].id >= d[1].id) {
        return Err(Error::Invalid("boundary order or duplicate ID"));
    }
    for d in &divisors {
        frame.local().supports(&d.equation)?;
    }
    let divisors = Arc::new(divisors);
    let mut progress = SncProgress::default();
    let result = verify(&frame, &divisors, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(None) => Ok(SncProduction::Verified(Arc::new(VerifiedRelativeSnc {
            frame,
            divisors,
            progress,
        }))),
        Ok(Some((indices, obstruction))) => Ok(SncProduction::Unresolved {
            frame,
            divisors,
            indices,
            obstruction,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(SncProduction::Incomplete {
            frame,
            divisors,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn subsets(n: usize, budget: &mut Budget) -> Result<Vec<Vec<usize>>> {
    let shift = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("boundary subset count"))?;
    let count = 1usize
        .checked_shl(shift)
        .and_then(|v| v.checked_sub(1))
        .ok_or(Error::ResourceIncomplete("boundary subset count"))?;
    budget.reserve_slots(count)?;
    Ok((1..=count)
        .map(|mask| (0..n).filter(|i| mask & (1 << i) != 0).collect())
        .collect())
}
fn combinations(n: usize, k: usize, budget: &mut Budget) -> Result<Vec<Vec<usize>>> {
    if k > n {
        return Ok(vec![]);
    }
    let mut count = 1usize;
    for i in 0..k {
        count = count
            .checked_mul(n - i)
            .ok_or(Error::ResourceIncomplete("minor count"))?
            / (i + 1);
    }
    budget.reserve_slots(count)?;
    fn append(
        n: usize,
        k: usize,
        start: usize,
        prefix: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if prefix.len() == k {
            out.push(prefix.clone());
            return;
        }
        for i in start..=n - (k - prefix.len()) {
            prefix.push(i);
            append(n, k, i + 1, prefix, out);
            prefix.pop();
        }
    }
    let mut out = Vec::new();
    append(n, k, 0, &mut Vec::new(), &mut out);
    Ok(out)
}
fn minor(
    frame: &EtaleFrame,
    rows: &[Vec<Poly>],
    columns: &[usize],
    budget: &mut Budget,
) -> Result<Poly> {
    let n = rows.len();
    let count = n
        .checked_mul(n)
        .ok_or(Error::ResourceIncomplete("minor entries"))?;
    budget.reserve_slots(count)?;
    let max_terms = rows
        .iter()
        .flat_map(|r| columns.iter().map(|i| r[*i].nterms()))
        .max()
        .unwrap_or(1);
    let mut terms = 1usize;
    for i in 1..=n {
        terms = terms
            .checked_mul(i)
            .and_then(|v| v.checked_mul(max_terms))
            .ok_or(Error::ResourceIncomplete("minor term bound"))?;
        if terms > budget.limits.max_terms {
            return Err(Error::ResourceIncomplete("minor term bound"));
        }
    }
    for axis in 0..frame.local().ring().len() {
        let degree = rows
            .iter()
            .try_fold(0usize, |s, r| {
                s.checked_add(
                    columns
                        .iter()
                        .map(|i| usize::from(r[*i].degree(axis)))
                        .max()
                        .unwrap_or(0),
                )
            })
            .ok_or(Error::ResourceIncomplete("minor degree bound"))?;
        if degree > usize::from(budget.limits.max_degree_per_axis) {
            return Err(Error::ResourceIncomplete("minor degree bound"));
        }
    }
    let field = AtomField {
        statistical_zero_test: false,
        cancel_check_on_division: true,
        ..AtomField::new()
    };
    let dim = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("minor dimension"))?;
    budget.charge(1)?;
    let matrix = Matrix::from_linear(
        rows.iter()
            .flat_map(|r| columns.iter().map(|i| r[*i].to_expression()))
            .collect(),
        dim,
        dim,
        field,
    )
    .map_err(|_| Error::Invalid("native boundary Jacobian"))?;
    let determinant = frame.local().ring().atom(
        &matrix
            .det()
            .map_err(|_| Error::Invalid("native boundary minor"))?
            .expand(),
    )?;
    budget.poly(&determinant)?;
    Ok(determinant)
}
fn verify(
    frame: &EtaleFrame,
    divisors: &[InitialDivisor],
    budget: &mut Budget,
    progress: &mut SncProgress,
) -> Result<Option<(Vec<usize>, Ideal)>> {
    // No sampled smoothness: all boundary intersections are considered.
    for indices in subsets(divisors.len(), budget)? {
        let generated = Ideal::new(
            frame.local().ring().clone(),
            indices
                .iter()
                .map(|i| divisors[*i].equation.clone())
                .collect(),
            budget,
        )?;
        let locus = frame.local().ideal().sum(&generated, budget)?;
        let empty = locus.contains(
            &frame.local().ring().one(),
            frame.local().unit_relations(),
            budget,
        )?;
        if empty {
            progress.intersections.push(IntersectionCertificate {
                indices,
                locus,
                empty,
                minors: vec![],
            });
            continue;
        }
        let choices = combinations(frame.free_axes().len(), indices.len(), budget)?;
        let count = indices
            .len()
            .checked_mul(frame.free_axes().len())
            .ok_or(Error::ResourceIncomplete("boundary Jacobian entries"))?;
        budget.reserve_slots(count)?;
        let rows = indices
            .iter()
            .map(|i| {
                (0..frame.free_axes().len())
                    .map(|j| frame.derivative(j, &divisors[*i].equation, budget))
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let mut minors = Vec::new();
        let mut unique = BTreeSet::new();
        for columns in choices {
            let p = minor(frame, &rows, &columns, budget)?;
            if !p.is_zero() && unique.insert(p.to_expression()) {
                minors.push(p);
            }
        }
        let rank = Ideal::new(frame.local().ring().clone(), minors.clone(), budget)?;
        let obstruction = locus.sum(&rank, budget)?;
        let covers = obstruction.contains(
            &frame.local().ring().one(),
            frame.local().unit_relations(),
            budget,
        )?;
        if !covers {
            return Ok(Some((indices, obstruction)));
        }
        progress.intersections.push(IntersectionCertificate {
            indices,
            locus,
            empty,
            minors,
        });
    }
    Ok(None)
}
