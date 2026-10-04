use crate::{
    SectorError,
    arithmetic::IntVector,
    types::{DecompositionPhase, Monitor},
};
use numerica::domains::integer::Integer;
use std::collections::BTreeMap;

/// Nonzero monomial exponents, stripped of coefficients by the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolynomialSupport {
    exponents: Vec<IntVector>,
    dimension: usize,
}

impl PolynomialSupport {
    pub fn new(exponents: Vec<Vec<i64>>) -> Result<Self, SectorError> {
        Self::from_exact(
            exponents
                .into_iter()
                .map(|v| v.into_iter().map(Integer::from).collect())
                .collect(),
        )
    }

    pub fn from_exact(mut exponents: Vec<IntVector>) -> Result<Self, SectorError> {
        let Some(first) = exponents.first() else {
            return Err(SectorError::InvalidSupport("zero polynomial".into()));
        };
        let dimension = first.len();
        if exponents
            .iter()
            .any(|r| r.len() != dimension || r.iter().any(|x| x < &Integer::from(0)))
        {
            return Err(SectorError::InvalidSupport(
                "inconsistent dimension or negative input exponent".into(),
            ));
        }
        exponents.sort();
        exponents.dedup();
        Ok(Self {
            exponents,
            dimension,
        })
    }

    pub fn exponents(&self) -> &[IntVector] {
        &self.exponents
    }
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub(crate) fn without_axis(&self, axis: usize) -> Self {
        let mut exponents: Vec<_> = self
            .exponents
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != axis)
                    .map(|(_, x)| x.clone())
                    .collect()
            })
            .collect();
        exponents.sort();
        exponents.dedup();
        Self {
            exponents,
            dimension: self.dimension - 1,
        }
    }
}

/// Vertices of a Minkowski sum have unique decompositions. Discarding repeated
/// sums is therefore an exact prefilter, not a floating-point hull heuristic.
pub(crate) fn minkowski_candidates(
    supports: &[PolynomialSupport],
    monitor: &mut Monitor<'_>,
) -> Result<Vec<IntVector>, SectorError> {
    monitor.status.phase = DecompositionPhase::Supports;
    let mut candidates = vec![vec![Integer::from(0); supports[0].dimension]];
    for support in supports {
        let pairs = candidates
            .len()
            .checked_mul(support.exponents.len())
            .ok_or(SectorError::ResourceLimit {
                resource: "support pairs",
                limit: monitor.options.max_support_pairs,
            })?;
        if pairs > monitor.options.max_support_pairs {
            return Err(SectorError::ResourceLimit {
                resource: "support pairs",
                limit: monitor.options.max_support_pairs,
            });
        }
        let mut counts = BTreeMap::<IntVector, u8>::new();
        for (i, a) in candidates.iter().enumerate() {
            if i % 128 == 0 {
                monitor.emit()?;
            }
            for b in &support.exponents {
                let sum = a.iter().zip(b).map(|(x, y)| x + y).collect();
                counts.entry(sum).and_modify(|c| *c = 2).or_insert(1);
            }
        }
        candidates = counts
            .into_iter()
            .filter_map(|(x, n)| (n == 1).then_some(x))
            .collect();
    }
    Ok(candidates)
}
