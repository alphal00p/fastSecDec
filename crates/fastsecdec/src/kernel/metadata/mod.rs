//! Portable semantic metadata. Canonical Atom serialization remains native;
//! this layer validates current chart/domain associations before compilation.
mod chart;
mod domain;
use super::KernelError;
use crate::generation::GenerationMetadata;
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore, AtomView, Symbol};
use symbolica::state::StateMap;

/// JSON inspection stays readable while binary caches retain native Atom bytes.
#[derive(Clone, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
struct StoredAtom(Atom);
impl Serialize for StoredAtom {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_canonical_string())
    }
}
impl<'de> Deserialize<'de> for StoredAtom {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        super::artifact::atom(text)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
impl From<&Atom> for StoredAtom {
    fn from(atom: &Atom) -> Self {
        Self(atom.clone())
    }
}

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
/// The same canonical semantic record used in portable kernel artifacts.
/// This is a transport/presentation value; native computation continues to use
/// [`GenerationMetadata`]. Loading always checks layout and associations;
/// optional validation also reconstructs polynomial and geometry proofs.
pub struct PortableMetadata {
    domain: domain::PortableAssessment,
    charts: Vec<chart::PortableChart>,
}
#[derive(bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
pub(in crate::kernel) struct LegacyMetadata {
    domain: domain::PortableAssessment,
    charts: Vec<chart::LegacyChart>,
}
impl From<LegacyMetadata> for PortableMetadata {
    fn from(value: LegacyMetadata) -> Self {
        Self {
            domain: value.domain,
            charts: value.charts.into_iter().map(Into::into).collect(),
        }
    }
}
#[cfg(test)]
impl From<PortableMetadata> for LegacyMetadata {
    fn from(value: PortableMetadata) -> Self {
        Self {
            domain: value.domain,
            charts: value.charts.into_iter().map(Into::into).collect(),
        }
    }
}
impl PortableMetadata {
    /// Record-local chart indices, never physical source identifiers. Definitions
    /// remain native StateMap-coded data in the explicitly versioned sidecar.
    pub(in crate::kernel) fn take_contour_definitions(
        &mut self,
    ) -> Vec<(usize, crate::contour::ContourDefinitions)> {
        self.charts
            .iter_mut()
            .enumerate()
            .filter_map(|(index, chart)| {
                let definitions = chart.take_contour_definitions();
                (!definitions.is_empty()).then_some((index, definitions))
            })
            .collect()
    }

    pub(in crate::kernel) fn attach_contour_definitions(
        &mut self,
        definitions: Vec<(usize, crate::contour::ContourDefinitions)>,
    ) -> Result<(), KernelError> {
        let mut previous = None;
        for (index, definitions) in definitions {
            if previous.is_some_and(|previous| index <= previous) {
                return Err(invalid(
                    "compact definition chart indices are not strictly ordered",
                ));
            }
            previous = Some(index);
            self.charts
                .get_mut(index)
                .ok_or_else(|| invalid("unknown compact definition chart"))?
                .attach_contour_definitions(definitions)?;
        }
        Ok(())
    }
    pub(in crate::kernel) fn visit_atoms(&self, visit: &mut impl FnMut(&Atom)) {
        self.domain.visit_atoms(visit);
        for chart in &self.charts {
            chart.visit_atoms(visit);
        }
    }

    /// Hash-boundary view of one kernel's retained semantics. Original chart
    /// ordinals stay intact; the selected kernel has local ordinal zero.
    pub(in crate::kernel) fn for_sector(value: &GenerationMetadata, index: usize) -> Self {
        Self {
            domain: domain::PortableAssessment::from_native(value.domain_assessment()),
            charts: value
                .charts()
                .iter()
                .filter(|chart| chart.kernel_sector() == Some(index))
                .map(chart::PortableChart::for_sector)
                .collect(),
        }
    }

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

    /// Serialize retained source charts associated with one numerical kernel.
    /// Original chart, representative and kernel indices remain unchanged.
    /// This presentation view excludes unrelated charts and global factors;
    /// unlike the sector identity view, it never renumbers the kernel to zero.
    pub fn charts_for_sector(value: &GenerationMetadata, index: usize) -> impl Serialize {
        value
            .charts()
            .iter()
            .filter(|chart| chart.kernel_sector() == Some(index))
            .map(chart::PortableChart::from_native)
            .collect::<Vec<_>>()
    }

    pub(super) fn into_native(
        self,
        sectors: &[Vec<Symbol>],
        validate: bool,
    ) -> Result<GenerationMetadata, KernelError> {
        let domain = self.domain.into_native(validate)?;
        let charts = self
            .charts
            .into_iter()
            .enumerate()
            .map(|(index, chart)| chart.into_native(index, &domain, validate))
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
fn atom(value: StoredAtom) -> Result<Atom, KernelError> {
    Ok(value.0)
}
fn symbols(values: Vec<String>) -> Result<Vec<Symbol>, KernelError> {
    let symbols = values
        .into_iter()
        .map(|text| {
            let value = super::artifact::atom(text)?;
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

#[cfg(test)]
mod tests;
