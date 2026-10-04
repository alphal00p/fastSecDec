use numerica::numerical_integration::ContinuousGrid;
use serde::{Deserialize, Serialize};

use crate::integration::{IntegrationError, Result};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HavanaSettings {
    pub points_per_batch: usize,
    pub batches: u32,
    pub seed: u64,
    pub bins: usize,
    pub minimum_probability_density: f64,
}

impl Default for HavanaSettings {
    fn default() -> Self {
        Self {
            points_per_batch: 4096,
            batches: 64,
            seed: 0,
            bins: 32,
            minimum_probability_density: 0.01,
        }
    }
}

impl HavanaSettings {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.points_per_batch < 2
            || self.batches < 2
            || self.bins == 0
            || !self.minimum_probability_density.is_finite()
            || !(0.0..=1.0).contains(&self.minimum_probability_density)
        {
            return Err(IntegrationError::Invalid("Havana needs at least two points per batch, two batches, positive bins and a probability floor in [0,1]".into()));
        }
        self.planned_points()?;
        self.bins
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("Havana bin count overflow".into()))?;
        Ok(())
    }

    pub(crate) fn planned_points(&self) -> Result<u64> {
        (self.points_per_batch as u64)
            .checked_mul(self.batches as u64)
            .ok_or_else(|| IntegrationError::Invalid("Havana point count overflow".into()))
    }

    pub(crate) fn grid(&self, dimension: usize) -> Result<ContinuousGrid<f64>> {
        self.validate()?;
        ContinuousGrid::new_with_min_probability_density(
            dimension,
            self.bins,
            self.bins,
            None,
            false,
            self.minimum_probability_density,
        )
        .map_err(IntegrationError::Invalid)
    }
}
