//! Generation-side construction from the actual exact section owner. The
//! detached arithmetic module has no dependency on GCAD or source proofs.
use super::{RegularSection, Result, native};
use crate::generation::identity::CanonicalAtom;
pub use crate::kernel::algebraic::{Number, RootProgram, Scope, attempt};
use std::sync::Arc;
use symbolica::evaluate::OptimizationSettings;
impl RootProgram {
    pub fn prepare(section: &RegularSection, settings: OptimizationSettings) -> Result<Arc<Self>> {
        let source = section.source();
        let raw = source.decomposition().native_result();
        let selector_domain = match section.bound().index_domain {
            symgcad::output::RootIndexDomain::Real => 0u8,
            symgcad::output::RootIndexDomain::Positive => 1,
            symgcad::output::RootIndexDomain::RealDescending => 2,
        };
        let contract = serde_json::to_vec(&(
            "fastsecdec-regular-section-branch-v1",
            source
                .decomposition()
                .request()
                .source_identity()
                .map_err(native)?,
            CanonicalAtom(section.equation()),
            section
                .coefficients()
                .iter()
                .map(CanonicalAtom)
                .collect::<Vec<_>>(),
            &raw.order,
            &raw.normalized_constraints,
            selector_domain,
            section.bound().index,
            section.axis(),
            section.bracket().lower.to_string(),
            section.bracket().upper.to_string(),
            section.bracket().derivative_margin.to_string(),
            section.orientation(),
        ))
        .map_err(native)?;
        Self::prepare_from_certificate(
            section.coefficients().len() - 1,
            section.bracket().lower.clone(),
            section.bracket().upper.clone(),
            contract,
            settings,
        )
        .map_err(Into::into)
    }
}
