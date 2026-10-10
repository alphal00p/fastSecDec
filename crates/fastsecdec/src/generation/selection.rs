//! Original geometry selection is separate from compact numerical chart IDs.
use super::GenerationError;
use serde::{Deserialize, Serialize};

/// The source charts retained from one complete native decomposition.
/// Indices refer to geometry charts before symmetry and endpoint subtraction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields)]
pub struct SourceSectorSelection {
    original_source_count: usize,
    source_sectors: Vec<usize>,
}

impl SourceSectorSelection {
    pub fn original_source_count(&self) -> usize {
        self.original_source_count
    }
    pub fn source_sectors(&self) -> &[usize] {
        &self.source_sectors
    }
    pub fn validate(&self) -> Result<(), GenerationError> {
        if self.original_source_count == 0
            || self.source_sectors.is_empty()
            || self.source_sectors.len() >= self.original_source_count
            || self.source_sectors.last().copied().unwrap() >= self.original_source_count
            || self
                .source_sectors
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid("invalid proper source-sector subset"));
        }
        Ok(())
    }
    pub(super) fn resolve(
        requested: Option<&[usize]>,
        total: usize,
    ) -> Result<Option<Self>, GenerationError> {
        let Some(requested) = requested else {
            return Ok(None);
        };
        validate_request(requested)?;
        let mut source_sectors = requested.to_vec();
        source_sectors.sort_unstable();
        if source_sectors.last().copied().unwrap() >= total {
            return Err(invalid(format!(
                "source_sectors contains an index outside 0..{total}"
            )));
        }
        if source_sectors.len() == total {
            return Ok(None);
        }
        let selection = Self {
            original_source_count: total,
            source_sectors,
        };
        selection.validate()?;
        Ok(Some(selection))
    }
    pub(crate) fn full_scope(&self) -> GenerationSourceScope {
        GenerationSourceScope {
            selection: self.clone(),
            chart_source_sectors: self.source_sectors.clone(),
        }
    }
}

/// Identity-bound generation extent, including record-local chart lineage.
///
/// `selection` describes the entire generated partial integral. The chart map
/// describes only this native metadata owner, and can be empty for its exact
/// offset. Neither field is a selection of post-subtraction numerical kernels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields)]
pub struct GenerationSourceScope {
    selection: SourceSectorSelection,
    chart_source_sectors: Vec<usize>,
}

impl GenerationSourceScope {
    pub fn selection(&self) -> &SourceSectorSelection {
        &self.selection
    }
    pub fn chart_source_sectors(&self) -> &[usize] {
        &self.chart_source_sectors
    }
    pub fn validate(&self, chart_count: usize) -> Result<(), GenerationError> {
        self.selection.validate()?;
        if self.chart_source_sectors.len() != chart_count
            || self
                .chart_source_sectors
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .chart_source_sectors
                .iter()
                .any(|index| self.selection.source_sectors.binary_search(index).is_err())
        {
            return Err(invalid(
                "source-sector scope differs from retained chart metadata",
            ));
        }
        Ok(())
    }
    pub(crate) fn for_charts(&self, local_indices: &[usize]) -> Result<Self, GenerationError> {
        let chart_source_sectors = local_indices
            .iter()
            .map(|index| {
                self.chart_source_sectors
                    .get(*index)
                    .copied()
                    .ok_or_else(|| invalid("unknown local chart in source-sector scope"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.with_chart_sources(chart_source_sectors)
    }
    pub(crate) fn with_chart_sources(
        &self,
        chart_source_sectors: Vec<usize>,
    ) -> Result<Self, GenerationError> {
        let scope = Self {
            selection: self.selection.clone(),
            chart_source_sectors,
        };
        scope.validate(scope.chart_source_sectors.len())?;
        Ok(scope)
    }
}

pub(super) fn validate_request(requested: &[usize]) -> Result<(), GenerationError> {
    if requested.is_empty() {
        return Err(invalid(
            "source_sectors must be nonempty; omit it to generate all charts",
        ));
    }
    let mut sorted = requested.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("source_sectors must not contain duplicate indices"));
    }
    Ok(())
}

pub(super) fn select_maps<T>(
    maps: Vec<T>,
    requested: Option<&[usize]>,
) -> Result<(Vec<T>, Option<GenerationSourceScope>), GenerationError> {
    let Some(selection) = SourceSectorSelection::resolve(requested, maps.len())? else {
        return Ok((maps, None));
    };
    let scope = selection.full_scope();
    let maps = maps
        .into_iter()
        .enumerate()
        .filter_map(|(index, map)| {
            selection
                .source_sectors
                .binary_search(&index)
                .is_ok()
                .then_some(map)
        })
        .collect();
    Ok((maps, Some(scope)))
}

fn invalid(message: impl Into<String>) -> GenerationError {
    GenerationError::SourceSelection(message.into())
}
