//! Bounded centered vector moments over independent, identically designed replicas.
//! Numerica's public scalar accumulator does not retain cross-component covariance.
//! Reuse its DoubleFloat arithmetic for the narrow missing vector operation.
use crate::integration::{IntegrationError, IntegrationProblem, Result, VectorEstimate};
use numerica::domains::float::{DoubleFloat, RealLike};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplicaMoments {
    count: u64,
    origin: Vec<f64>,
    offset: Vec<DoubleFloat>,
    m2: Vec<DoubleFloat>,
}
impl ReplicaMoments {
    pub fn new(outputs: usize) -> Result<Self> {
        let size = outputs
            .checked_mul(outputs)
            .filter(|_| outputs > 0)
            .ok_or_else(|| IntegrationError::Invalid("replica vector shape overflow".into()))?;
        Ok(Self {
            count: 0,
            origin: vec![0.; outputs],
            offset: vec![DoubleFloat::from(0.); outputs],
            m2: vec![DoubleFloat::from(0.); size],
        })
    }
    pub fn count(&self) -> u64 {
        self.count
    }
    pub(super) fn centered_mean(&self) -> Vec<DoubleFloat> {
        self.origin
            .iter()
            .zip(&self.offset)
            .map(|(&a, &m)| DoubleFloat::from(a) + m)
            .collect()
    }
    pub(super) fn centered_covariance(&self) -> Result<Vec<DoubleFloat>> {
        if self.count < 2 {
            return Err(IntegrationError::Unavailable(
                "at least two independent complete replicas are required".into(),
            ));
        }
        let denominator =
            DoubleFloat::from(self.count as f64) * DoubleFloat::from((self.count - 1) as f64);
        let covariance: Vec<_> = self.m2.iter().map(|&m| m / denominator).collect();
        for i in 0..self.origin.len() {
            let diagonal = i * self.origin.len() + i;
            if self.m2[diagonal] != DoubleFloat::from(0.) && covariance[diagonal].to_f64() == 0. {
                return Err(IntegrationError::NumericRange);
            }
        }
        Ok(covariance)
    }
    pub fn add(&mut self, values: &[f64]) -> Result<()> {
        if values.len() != self.origin.len() || values.iter().any(|v| !v.is_finite()) {
            return Err(IntegrationError::InvalidReturn(
                "invalid replica vector".into(),
            ));
        }
        let count = self
            .count
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("replica count overflow".into()))?;
        let mut next = self.clone();
        if self.count == 0 {
            next.origin.copy_from_slice(values);
        } else {
            let n = DoubleFloat::from(count as f64);
            let delta: Vec<_> = values
                .iter()
                .zip(&self.origin)
                .zip(&self.offset)
                .map(|((&v, &a), &m)| (DoubleFloat::from(v) - DoubleFloat::from(a)) - m)
                .collect();
            for (mean, d) in next.offset.iter_mut().zip(&delta) {
                *mean += *d / n;
            }
            let factor = DoubleFloat::from(self.count as f64) / n;
            let width = values.len();
            for i in 0..width {
                for j in 0..=i {
                    let increment = delta[i] * delta[j] * factor;
                    if i == j && delta[i] != DoubleFloat::from(0.) && increment.to_f64() == 0. {
                        return Err(IntegrationError::NumericRange);
                    }
                    next.m2[i * width + j] += increment;
                    next.m2[j * width + i] = next.m2[i * width + j];
                }
            }
        }
        next.count = count;
        next.validate(values.len())?;
        *self = next;
        Ok(())
    }
    pub(super) fn validate(&self, outputs: usize) -> Result<()> {
        if self.origin.len() != outputs
            || self.offset.len() != outputs
            || outputs.checked_mul(outputs) != Some(self.m2.len())
            || self.origin.iter().any(|v| !v.is_finite())
            || self
                .offset
                .iter()
                .chain(&self.m2)
                .any(|v| !v.to_f64().is_finite())
        {
            return Err(IntegrationError::Invalid(
                "invalid centered replica moments".into(),
            ));
        }
        for i in 0..outputs {
            if self.m2[i * outputs + i].to_f64() < 0. {
                return Err(IntegrationError::NumericRange);
            }
        }
        Ok(())
    }
    pub fn estimate(&self, problem: &IntegrationProblem, complete: bool) -> Result<VectorEstimate> {
        self.validate(problem.orders.len())?;
        if self.count < 2 {
            return Err(IntegrationError::Unavailable(
                "at least two independent complete replicas are required".into(),
            ));
        }
        let mean = self
            .origin
            .iter()
            .zip(&self.offset)
            .map(|(&a, &m)| (DoubleFloat::from(a) + m).to_f64())
            .collect();
        let covariance_of_mean: Vec<_> = self
            .centered_covariance()?
            .into_iter()
            .map(|m| m.to_f64())
            .collect();
        let standard_error = (0..self.origin.len())
            .map(|i| covariance_of_mean[i * self.origin.len() + i].sqrt())
            .collect();
        let value = VectorEstimate {
            orders: problem.orders.clone(),
            components: problem.components.clone(),
            mean,
            standard_error,
            covariance_of_mean,
            production_complete: complete,
        };
        value.validate()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covariance_normalization_cannot_erase_nonzero_variance() {
        let mut moments = ReplicaMoments::new(1).unwrap();
        // The sufficient statistics of many mostly equal observations can
        // retain a representable M2 whose variance of the mean is too small.
        moments.count = 1_000_000;
        moments.m2[0] = DoubleFloat::from(1e-320);
        assert!(matches!(
            moments.centered_covariance(),
            Err(IntegrationError::NumericRange)
        ));
        let problem = IntegrationProblem::new(
            "tiny-normalized-variance".into(),
            vec![0],
            Vec::new(),
            vec![0.],
        )
        .unwrap();
        assert!(moments.estimate(&problem, true).is_err());
    }
}
