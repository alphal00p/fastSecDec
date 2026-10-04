use super::*;
use crate::integration::{IntegrationProblem, SectorSpec};

impl KernelResultManifest {
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
}
