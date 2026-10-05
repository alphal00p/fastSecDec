//! Portable semantic metadata. Canonical Atom serialization remains native;
//! this layer validates current chart/domain associations before compilation.
mod chart;
mod domain;
use super::KernelError;
use crate::generation::GenerationMetadata;
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore, AtomView, Symbol};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// The same canonical semantic record used in portable kernel artifacts.
/// This is a transport/presentation value; native computation continues to use
/// [`GenerationMetadata`]. Loading a kernel revalidates the record natively.
pub struct PortableMetadata {
    domain: domain::PortableAssessment,
    charts: Vec<chart::PortableChart>,
}
impl PortableMetadata {
    pub fn from_native(value: &GenerationMetadata) -> Self {
        Self {
            domain: domain::PortableAssessment::from_native(value.domain_assessment()),
            charts: value
                .charts()
                .iter()
                .map(chart::PortableChart::from_native)
                .collect(),
        }
    }
    pub(super) fn into_native(
        self,
        sectors: &[Vec<Symbol>],
    ) -> Result<GenerationMetadata, KernelError> {
        let domain = self.domain.into_native()?;
        let charts = self
            .charts
            .into_iter()
            .enumerate()
            .map(|(index, chart)| chart.into_native(index, &domain))
            .collect::<Result<Vec<_>, _>>()?;
        let mut covered = vec![false; sectors.len()];
        for chart in &charts {
            let representative = charts
                .get(chart.representative)
                .ok_or_else(|| invalid("unknown chart representative"))?;
            if representative.representative != representative.source_index
                || chart.kernel_sector != representative.kernel_sector
            {
                return Err(invalid("inconsistent representative association"));
            }
            if chart.representative == chart.source_index
                && chart
                    .representative_permutation
                    .iter()
                    .enumerate()
                    .any(|(axis, mapped)| axis != *mapped)
            {
                return Err(invalid(
                    "representative chart permutation is not the identity",
                ));
            }
            if let Some(index) = chart.kernel_sector {
                let sector = sectors
                    .get(index)
                    .ok_or_else(|| invalid("unknown chart kernel sector"))?;
                if representative.coordinates.target_parameters != *sector {
                    return Err(invalid("chart and kernel coordinates differ"));
                }
                covered[index] = true;
            }
        }
        if covered.iter().any(|value| !value) {
            return Err(invalid("kernel sector has no retained chart"));
        }
        Ok(GenerationMetadata { domain, charts })
    }
}
fn invalid(message: &str) -> KernelError {
    KernelError::Artifact(message.into())
}
fn atom(text: String) -> Result<Atom, KernelError> {
    super::artifact::atom(text)
}
fn symbols(values: Vec<String>) -> Result<Vec<Symbol>, KernelError> {
    let symbols = values
        .into_iter()
        .map(|text| {
            let value = atom(text)?;
            match value.as_view() {
                AtomView::Var(variable) => Ok(variable.get_symbol()),
                _ => Err(invalid("metadata parameter is not a symbol")),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if symbols
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != symbols.len()
    {
        return Err(invalid("duplicate metadata parameters"));
    }
    Ok(symbols)
}
fn symbol_strings(values: &[Symbol]) -> Vec<String> {
    values
        .iter()
        .map(|symbol| Atom::var(*symbol).to_canonical_string())
        .collect()
}
