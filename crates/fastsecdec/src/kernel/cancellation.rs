//! Coordinate profiles used by numerical conditioning. A row can describe a
//! retained Taylor remainder or a conservative bound from mapped endpoint powers.
//! Rows schedule precision checks; they are not floating-point error certificates.
//! Historical artifacts have only a total degree and retain a conservative
//! bound using the smallest coordinate.
use super::KernelError;

#[derive(Clone)]
pub(super) struct Cancellation {
    dimension: usize,
    degree: usize,
    terms: Option<Vec<Vec<usize>>>,
    dominant_terms: Vec<Vec<usize>>,
    endpoint_profiles: Option<Vec<crate::generation::EndpointProfileRow>>,
}

impl Cancellation {
    #[cfg(feature = "threshold-decomposition")]
    pub fn from_endpoint_profiles(
        profiles: Vec<crate::generation::EndpointProfileRow>,
        dimension: usize,
    ) -> Result<Self, KernelError> {
        let rows = Self::profile_rows(&profiles, dimension)?;
        let rows = rows.into_iter().collect::<Vec<_>>();
        let degree = rows
            .iter()
            .map(|row| {
                row.iter()
                    .try_fold(0usize, |total, n| total.checked_add(*n))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| KernelError::Artifact("cancellation degree overflow".into()))?
            .into_iter()
            .max()
            .unwrap_or(0);
        Self::new(degree, Some(rows), dimension)?.with_endpoint_profiles(profiles)
    }

    pub fn new(
        degree: usize,
        terms: Option<Vec<Vec<usize>>>,
        dimension: usize,
    ) -> Result<Self, KernelError> {
        if let Some(terms) = &terms {
            let mut maximum = 0;
            for term in terms {
                if term.len() != dimension {
                    return Err(KernelError::Artifact(
                        "cancellation metadata dimension mismatch".into(),
                    ));
                }
                let total = term
                    .iter()
                    .try_fold(0usize, |total, value| total.checked_add(*value))
                    .ok_or_else(|| KernelError::Artifact("cancellation degree overflow".into()))?;
                maximum = maximum.max(total);
            }
            if degree != maximum {
                return Err(KernelError::Artifact(
                    "inconsistent cancellation degree".into(),
                ));
            }
        }
        // A row equal to the componentwise maxima dominates every other row.
        // This common dense-Taylor case reduces the hot-path bound to one dot
        // product without an expensive general dominance search.
        let dominant_terms = terms.as_ref().map_or_else(Vec::new, |terms| {
            let maxima = (0..dimension)
                .map(|axis| terms.iter().map(|row| row[axis]).max().unwrap_or(0))
                .collect::<Vec<_>>();
            if terms.contains(&maxima) {
                vec![maxima]
            } else {
                terms.clone()
            }
        });
        Ok(Self {
            dimension,
            degree,
            terms,
            dominant_terms,
            endpoint_profiles: None,
        })
    }

    pub fn degree(&self) -> usize {
        self.degree
    }
    pub fn terms(&self) -> Option<&[Vec<usize>]> {
        self.terms.as_deref()
    }

    pub fn endpoint_profiles(&self) -> Option<&[crate::generation::EndpointProfileRow]> {
        self.endpoint_profiles.as_deref()
    }

    pub fn with_endpoint_profiles(
        mut self,
        profiles: Vec<crate::generation::EndpointProfileRow>,
    ) -> Result<Self, KernelError> {
        let projected = Self::profile_rows(&profiles, self.dimension)?;
        let expected = self
            .terms
            .as_ref()
            .ok_or_else(|| {
                KernelError::Artifact("endpoint profiles need cancellation rows".into())
            })?
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if projected != expected {
            return Err(KernelError::Artifact(
                "endpoint profile coverage differs from cancellation rows".into(),
            ));
        }
        self.endpoint_profiles = Some(profiles);
        Ok(self)
    }

    fn profile_rows(
        profiles: &[crate::generation::EndpointProfileRow],
        dimension: usize,
    ) -> Result<std::collections::BTreeSet<Vec<usize>>, KernelError> {
        let mut projected = std::collections::BTreeSet::new();
        for row in profiles {
            if row.axes.len() != dimension {
                return Err(KernelError::Artifact(
                    "endpoint profile dimension mismatch".into(),
                ));
            }
            let mut orders = Vec::with_capacity(dimension);
            for axis in &row.axes {
                let mut maximum = 0;
                for source in axis {
                    super::stability::canonical_power(&source.original_power)?;
                    if source.order == 0 {
                        return Err(KernelError::Artifact(
                            "zero-order endpoint cancellation source".into(),
                        ));
                    }
                    maximum = maximum.max(source.order);
                }
                orders.push(maximum);
            }
            projected.insert(orders);
        }
        Ok(projected)
    }
    pub fn routing_rows(&self) -> Vec<Vec<usize>> {
        self.terms.clone().unwrap_or_else(|| {
            (0..self.dimension)
                .map(|axis| {
                    let mut row = vec![0; self.dimension];
                    row[axis] = self.degree;
                    row
                })
                .collect()
        })
    }

    pub fn lost_bits(&self, point: &[f64]) -> f64 {
        let point = &point[..self.dimension];
        if self.terms.is_some() {
            self.dominant_terms
                .iter()
                .map(|term| {
                    term.iter()
                        .zip(point)
                        .filter(|(degree, _)| **degree != 0)
                        .map(|(degree, value)| -value.log2() * *degree as f64)
                        .sum::<f64>()
                })
                .fold(0.0, f64::max)
        } else if self.degree == 0 {
            0.0
        } else {
            -point.iter().copied().fold(1.0, f64::min).log2() * self.degree as f64
        }
    }

    pub fn needs_check(&self, point: &[f64], threshold: f64) -> bool {
        self.degree != 0 && self.lost_bits(point) > -threshold.log2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_coordinates_and_independent_remainders_have_distinct_bounds() {
        let one_axis = Cancellation::new(1, Some(vec![vec![1, 0]]), 2).unwrap();
        assert!(!one_axis.needs_check(&[0.5, 1e-80], 1e-3));
        assert!(one_axis.needs_check(&[1e-5, 0.5], 1e-3));
        let independent = Cancellation::new(2, Some(vec![vec![2, 0], vec![0, 2]]), 2).unwrap();
        assert_eq!(independent.lost_bits(&[0.25, 0.25]), 4.0);
        let mixed = Cancellation::new(4, Some(vec![vec![2, 2]]), 2).unwrap();
        assert_eq!(mixed.lost_bits(&[0.25, 0.25]), 8.0);
    }

    #[test]
    fn high_degree_cancellation_triggers_checks_at_interior_points() {
        let high = Cancellation::new(16, Some(vec![vec![16]]), 1).unwrap();
        assert!(high.needs_check(&[0.1], 1e-3));
        assert!(high.needs_check(&[0.02], 1e-3));
        assert!(!high.needs_check(&[0.9], 1e-3));
        let legacy = Cancellation::new(16, None, 1).unwrap();
        assert_eq!(legacy.lost_bits(&[0.02]), high.lost_bits(&[0.02]));
    }
}
