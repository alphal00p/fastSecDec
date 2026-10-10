//! Strong ownership of one selected recipe during symbolic preparation.
use super::GenerationError;
use crate::kernel::{DynamicCheckSource, NativeProgramDescriptor};
use std::sync::Arc;

/// Source expressions stay with their descriptor until compilation replaces
/// them with optimized native checking programs. No weak registry entry is an
/// ownership substitute when jobs cross caller-controlled worker boundaries.
#[derive(Clone, Debug, Default)]
pub(crate) struct ProgramData {
    pub descriptor: Option<Arc<NativeProgramDescriptor>>,
    pub checks: Vec<Arc<DynamicCheckSource>>,
    pub definitions: Vec<(usize, Arc<crate::contour::ContourDefinitions>)>,
    pub source_witnesses: Vec<(usize, super::symmetry::SourceWitness)>,
}

impl ProgramData {
    /// Keep prepared chart identities and renumber them record-locally, before
    /// Laurent assembly creates exact offsets. Do not use for completed kernel
    /// payloads: their projection must also retain exact-expression helpers.
    pub fn select(&self, sources: &[usize]) -> Result<Self, GenerationError> {
        let descriptor = self
            .descriptor
            .as_ref()
            .map(|descriptor| {
                descriptor
                    .for_payload(sources, &[], None)
                    .map(Arc::new)
                    .map_err(|e| GenerationError::Contour(e.to_string()))
            })
            .transpose()?;
        let checks = sources
            .iter()
            .enumerate()
            .flat_map(|(local, source)| {
                self.checks
                    .iter()
                    .filter(move |check| check.chart_index == *source)
                    .map(move |check| {
                        let mut check = check.as_ref().clone();
                        check.chart_index = local;
                        Arc::new(check)
                    })
            })
            .collect();
        let definitions = sources
            .iter()
            .enumerate()
            .flat_map(|(local, source)| {
                self.definitions
                    .iter()
                    .filter(move |(index, _)| index == source)
                    .map(move |(_, definitions)| (local, definitions.clone()))
            })
            .collect();
        let source_witnesses = sources
            .iter()
            .enumerate()
            .flat_map(|(local, source)| {
                self.source_witnesses
                    .iter()
                    .filter(move |(index, _)| index == source)
                    .map(move |(_, witness)| (local, witness.clone()))
            })
            .collect();
        Ok(Self {
            descriptor,
            checks,
            definitions,
            source_witnesses,
        })
    }
    pub fn remap(&mut self, sources: &[usize]) -> Result<(), GenerationError> {
        if let Some(descriptor) = &mut self.descriptor {
            Arc::make_mut(descriptor)
                .remap_charts(sources)
                .map_err(|e| GenerationError::Contour(e.to_string()))?;
        }
        for check in &mut self.checks {
            let check = Arc::make_mut(check);
            check.chart_index = *sources.get(check.chart_index).ok_or_else(|| {
                GenerationError::Invariant("dynamic check source index mismatch".into())
            })?;
        }
        for index in self
            .definitions
            .iter_mut()
            .map(|(index, _)| index)
            .chain(self.source_witnesses.iter_mut().map(|(index, _)| index))
        {
            *index = *sources.get(*index).ok_or_else(|| {
                GenerationError::Invariant("compact contour source index mismatch".into())
            })?;
        }
        Ok(())
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), GenerationError> {
        if let Some(incoming) = &other.descriptor {
            if let Some(existing) = &mut self.descriptor {
                Arc::make_mut(existing)
                    .merge(incoming.as_ref().clone())
                    .map_err(|e| GenerationError::Contour(e.to_string()))?;
            } else {
                self.descriptor = Some(incoming.clone());
            }
        }
        self.checks.extend(other.checks.iter().cloned());
        self.definitions.extend(other.definitions.iter().cloned());
        self.source_witnesses
            .extend(other.source_witnesses.iter().cloned());
        self.checks.sort_by_key(|check| check.chart_index);
        if self
            .checks
            .windows(2)
            .any(|pair| pair[0].chart_index == pair[1].chart_index)
        {
            return Err(GenerationError::Invariant(
                "duplicate dynamic check source".into(),
            ));
        }
        Ok(())
    }

    pub fn contour_definitions(
        &self,
    ) -> Result<Arc<crate::contour::ContourDefinitions>, GenerationError> {
        if let [(_, definitions)] = self.definitions.as_slice() {
            return Ok(definitions.clone());
        }
        let mut combined = crate::contour::ContourDefinitions::default();
        for (_, definitions) in &self.definitions {
            combined
                .merge(definitions)
                .map_err(GenerationError::Contour)?;
        }
        Ok(Arc::new(combined))
    }
}

#[cfg(test)]
mod tests;
