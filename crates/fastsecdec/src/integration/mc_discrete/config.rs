use crate::integration::{IntegrationError, IntegrationProblem, Result, mc::HavanaSettings};
use numerica::numerical_integration::{DiscreteGrid, Grid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HavanaDiscreteSettings {
    pub batch: HavanaSettings,
    pub maximum_sector_probability_ratio: f64,
}
impl Default for HavanaDiscreteSettings {
    fn default() -> Self {
        Self {
            batch: HavanaSettings::default(),
            maximum_sector_probability_ratio: 100.0,
        }
    }
}
impl HavanaDiscreteSettings {
    pub fn validate(&self) -> Result<()> {
        self.batch.validate()?;
        if !self.maximum_sector_probability_ratio.is_finite()
            || self.maximum_sector_probability_ratio < 1.0
        {
            return Err(IntegrationError::Invalid(
                "maximum sector probability ratio must be finite and at least one".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn grid(&self, problem: &IntegrationProblem) -> Result<DiscreteGrid<f64>> {
        self.validate()?;
        DiscreteGrid::new(
            problem
                .sectors
                .iter()
                .map(|sector| {
                    self.batch
                        .grid(sector.dimension)
                        .map(|g| Some(Grid::Continuous(g)))
                })
                .collect::<Result<_>>()?,
            self.maximum_sector_probability_ratio,
            false,
        )
        .map_err(IntegrationError::Invalid)
    }
}
