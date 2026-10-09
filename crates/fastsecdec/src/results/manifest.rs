use super::*;
use crate::integration::{IntegrationProblem, SectorSpec};

impl KernelResultManifest {
    /// Build an integration problem from a live native owner, requiring bound
    /// parameters and completed contour pilots for the requested stochastic
    /// and exact contributions. This also applies when no sampling is needed.
    /// The plain manifest constructor below is an inspection/caller-declaration
    /// boundary and deliberately does not confer this runtime readiness.
    pub fn integration_problem_from_kernels(
        kernels: &crate::kernel::KernelSet,
        scope: &ResultScope,
        accumulation_identity: impl Into<String>,
    ) -> Result<IntegrationProblem> {
        let manifest = Self::from_kernels(kernels);
        let scope = manifest.canonical_scope(scope)?;
        kernels
            .validate_integration_readiness(&scope)
            .map_err(|error| ResultError::Invalid(error.to_string()))?;
        manifest.integration_problem(&scope, accumulation_identity)
    }

    /// Capture metadata without compiling or evaluating. Sector IDs follow the
    /// public kernel slice's zero-based indices; alternate callers may construct
    /// the native manifest directly with their own stable sector identities.
    pub fn from_kernels(kernels: &crate::kernel::KernelSet) -> Self {
        Self {
            kernel_content_id: kernels.content_id().into(),
            orders: kernels.orders().to_vec(),
            components: kernels.components().to_vec(),
            sectors: kernels
                .sectors()
                .iter()
                .enumerate()
                .map(|(id, s)| SectorSpec {
                    id: id as u64,
                    dimension: s.dimension(),
                })
                .collect(),
            exact_coefficients: kernels.exact_coefficients().to_vec(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        IntegrationProblem::new_with_components(
            self.kernel_content_id.clone(),
            self.orders.clone(),
            self.components.clone(),
            self.sectors.clone(),
            self.exact_coefficients.clone(),
        )?;
        Ok(())
    }

    /// Validate a scope against this complete parent and canonicalize its ID
    /// order. Explicit selections, including empty/all-sector selections, keep
    /// their qualification; they are never promoted to the full integral.
    pub fn canonical_scope(&self, scope: &ResultScope) -> Result<ResultScope> {
        self.validate()?;
        match scope {
            ResultScope::FullIntegral => Ok(ResultScope::FullIntegral),
            ResultScope::SelectedSectors {
                sector_ids,
                exact_policy,
            } => {
                let mut ids = sector_ids.clone();
                ids.sort_unstable();
                let all: std::collections::BTreeSet<_> =
                    self.sectors.iter().map(|sector| sector.id).collect();
                if ids.windows(2).any(|pair| pair[0] == pair[1])
                    || ids.iter().any(|id| !all.contains(id))
                {
                    return Err(ResultError::Invalid(
                        "selected IDs must be a unique subset of the full manifest".into(),
                    ));
                }
                Ok(ResultScope::SelectedSectors {
                    sector_ids: ids,
                    exact_policy: *exact_policy,
                })
            }
        }
    }

    /// Project native sector records and the declared exact offset without
    /// evaluating kernels or reconstructing statistics. The caller's distinct
    /// accumulation identity may include artifact/environment information.
    /// This accepts a caller declaration; for a live kernel owner use
    /// [`Self::integration_problem_from_kernels`] to enforce runtime readiness.
    pub fn integration_problem(
        &self,
        scope: &ResultScope,
        accumulation_identity: impl Into<String>,
    ) -> Result<IntegrationProblem> {
        let scope = self.canonical_scope(scope)?;
        let (sectors, exact) = match scope {
            ResultScope::FullIntegral => (self.sectors.clone(), self.exact_coefficients.clone()),
            ResultScope::SelectedSectors {
                sector_ids,
                exact_policy,
            } => (
                self.sectors
                    .iter()
                    .filter(|sector| sector_ids.binary_search(&sector.id).is_ok())
                    .cloned()
                    .collect(),
                match exact_policy {
                    ExactContributionPolicy::IncludeAll => self.exact_coefficients.clone(),
                    ExactContributionPolicy::ExcludeAll => vec![0.0; self.orders.len()],
                },
            ),
        };
        Ok(IntegrationProblem::new_with_components(
            accumulation_identity.into(),
            self.orders.clone(),
            self.components.clone(),
            sectors,
            exact,
        )?)
    }
}
