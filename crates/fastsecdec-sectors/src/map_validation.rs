use crate::{ParametricDomain, SectorError, SectorMap, arithmetic::determinant};
use numerica::domains::integer::Integer;

impl SectorMap {
    /// Validate the exact monomial map and measure for its input domain.
    /// Factor-valuation row widths are checked; certifying their values requires
    /// the original factor supports and remains the decomposer's responsibility.
    pub fn validate(&self, domain: ParametricDomain) -> Result<(), SectorError> {
        let invalid = |detail: &str| SectorError::Geometry(format!("invalid sector map: {detail}"));
        let source = self.source_dimension();
        let target = self.dimension();
        if self.exponent_matrix.iter().any(|row| row.len() != target)
            || self.factor_valuations.iter().any(|row| row.len() != target)
        {
            return Err(invalid("inconsistent matrix or factor-valuation width"));
        }
        if domain == ParametricDomain::ProjectiveSimplex {
            let Some(pivot) = self.fixed_parameter else {
                return Err(invalid("projective map has no fixed parameter"));
            };
            if source != target.saturating_add(1)
                || pivot >= source
                || self.exponent_matrix[pivot]
                    .iter()
                    .any(|entry| !entry.is_zero())
            {
                return Err(invalid("projective dimension or fixed-parameter row"));
            }
        } else if source != target || self.fixed_parameter.is_some() {
            return Err(invalid("nonprojective dimension or fixed parameter"));
        }
        if domain != ParametricDomain::PositiveOrthant
            && self
                .exponent_matrix
                .iter()
                .flatten()
                .any(|entry| entry < &Integer::from(0))
        {
            return Err(invalid("negative exponent outside positive-orthant domain"));
        }
        for j in 0..target {
            let expected = self
                .exponent_matrix
                .iter()
                .fold(Integer::from(-1), |sum, row| sum + &row[j]);
            if self.jacobian_powers[j] != expected {
                return Err(invalid("Jacobian exponent disagrees with exponent matrix"));
            }
        }
        let square = self
            .exponent_matrix
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != self.fixed_parameter)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        if self.determinant <= 0 || determinant(&square)? != self.determinant {
            return Err(invalid("nonpositive or inconsistent absolute determinant"));
        }
        Ok(())
    }
}
