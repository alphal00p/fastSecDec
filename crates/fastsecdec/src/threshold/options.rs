//! Native request settings. Enabling generation is a separate caller decision.
use super::{
    gcad::{GcadError, GcadKinematics, GcadRequest, Limits, Result, SolverOptions},
    projective::AffineProjectivePreparation,
    resolution,
};
use crate::{
    generation::{GenerationMode, GenerationOptions},
    parametric::{ParametricDomain, ParametricIntegrand},
};
use std::collections::BTreeSet;

/// Where verified threshold geometry starts. Source ordinals always refer to
/// the existing unsubtracted sector charts, never newly created GCAD cells.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdStrategy {
    #[default]
    Auto,
    GcadFirst,
    SectorFirst,
}

/// Configuration for opt-in native threshold generation. The outer generation
/// request controls whether this configuration is used; its mere presence does
/// not enable threshold decomposition in an ordinary generation recipe.
#[derive(Clone, Debug)]
pub struct ThresholdDecompositionOptions {
    pub kinematics: GcadKinematics,
    pub strategy: ThresholdStrategy,
    /// Existing owner settings, including exact variable-order constraints.
    pub solver: SolverOptions,
    /// One synchronous solve per caller-owned generation worker.
    pub gcad_limits: Limits,
    pub resolution_limits: resolution::Limits,
    /// Raw verified cell ordinals within one geometry problem. Multiple source
    /// problems require a source/cell lineage selector rather than applying this
    /// list independently to each problem.
    pub threshold_cells: Option<Vec<usize>>,
}
impl Default for ThresholdDecompositionOptions {
    fn default() -> Self {
        Self {
            kinematics: GcadKinematics::default(),
            strategy: ThresholdStrategy::Auto,
            solver: SolverOptions::default(),
            gcad_limits: GcadRequest::default_limits(),
            resolution_limits: resolution::Limits::default(),
            threshold_cells: None,
        }
    }
}
impl ThresholdDecompositionOptions {
    /// Validate mathematical compatibility without generating or solving.
    /// This does not admit a source chart, prove endpoint regularity, or make
    /// an integration artifact complete.
    pub fn effective_strategy(&self, generation: &GenerationOptions) -> Result<ThresholdStrategy> {
        if generation.contour_enabled() {
            return Err(GcadError::Invalid(
                "threshold decomposition and contour deformation are mutually exclusive".into(),
            ));
        }
        if generation.mode != GenerationMode::Symbolic {
            return Err(GcadError::Invalid(
                "threshold decomposition requires symbolic endpoint reduction".into(),
            ));
        }
        if let Some(sources) = &generation.source_sectors {
            distinct_nonempty(sources, "source_sectors")?;
            if self.strategy == ThresholdStrategy::GcadFirst {
                return Err(GcadError::Invalid(
                    "source_sectors requires sector_first or auto to preserve original source identities".into(),
                ));
            }
        }
        if let Some(cells) = &self.threshold_cells {
            distinct_nonempty(cells, "threshold_cells")?;
        }
        Ok(match self.strategy {
            ThresholdStrategy::Auto if generation.source_sectors.is_some() => {
                ThresholdStrategy::SectorFirst
            }
            ThresholdStrategy::Auto => ThresholdStrategy::GcadFirst,
            strategy => strategy,
        })
    }

    /// Build only the full-input geometry request. Actual solving, verification,
    /// cell selection and regularization remain caller-owned separate stages.
    /// Source-chart-first requests require the native unsubtracted chart owner
    /// and cannot be fabricated by treating its original input as a unit cube.
    pub fn gcad_first_request(
        &self,
        input: &ParametricIntegrand,
        generation: &GenerationOptions,
    ) -> Result<GcadRequest> {
        if self.effective_strategy(generation)? != ThresholdStrategy::GcadFirst {
            return Err(GcadError::Unsupported(
                "sector_first geometry requires an admitted unsubtracted source chart".into(),
            ));
        }
        match input.domain() {
            ParametricDomain::UnitCube => GcadRequest::unit_cube(
                input,
                self.kinematics.clone(),
                self.solver.clone(),
                self.gcad_limits.clone(),
            ),
            ParametricDomain::ProjectiveSimplex => GcadRequest::projective(
                AffineProjectivePreparation::last_coordinate(input)
                    .map_err(|error| GcadError::Invalid(error.to_string()))?,
                self.kinematics.clone(),
                self.solver.clone(),
                self.gcad_limits.clone(),
            ),
            ParametricDomain::PositiveOrthant => Err(GcadError::Unsupported(
                "positive-orthant threshold preparation requires a certified compactification"
                    .into(),
            )),
        }
    }
}

fn distinct_nonempty(values: &[usize], name: &str) -> Result<()> {
    if values.is_empty() || values.iter().copied().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(GcadError::Invalid(format!(
            "{name} must be nonempty and contain distinct indices"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
