//! One consuming assembly path for selected records and already compiled units.
use super::failure;
use crate::{
    generation::ChartRecord,
    kernel::{KernelError, KernelSet, projection::OutputProjection},
    status::CoefficientComponent,
};
use std::collections::BTreeMap;
use symbolica::atom::Atom;

/// Retains only the explicitly requested numerical owner. Incoming units must
/// contain at most one sector and use local chart indices; no evaluator is
/// cloned, decoded or translated while combining them.
#[derive(Default)]
pub(crate) struct ResidentAssembly {
    combined: Option<KernelSet>,
    charts: Vec<ChartRecord>,
    exact: BTreeMap<i32, Atom>,
    layouts: Vec<(Vec<i32>, Vec<CoefficientComponent>)>,
    source_scope: Option<crate::generation::GenerationSourceScope>,
    original_sources: BTreeMap<usize, usize>,
}

impl ResidentAssembly {
    pub(crate) fn push(
        &mut self,
        mut local: KernelSet,
        sources: &[usize],
        exact_only: bool,
    ) -> Result<(), KernelError> {
        if local.sectors.len() > 1 || local.sectors.iter().any(|s| s.projection.is_some()) {
            return Err(failure(
                "resident assembly requires one unprojected native unit",
            ));
        }
        let scope = local
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.source_scope.clone());
        if self.combined.is_some()
            && self.source_scope.as_ref().map(|scope| scope.selection())
                != scope.as_ref().map(|scope| scope.selection())
        {
            return Err(failure(
                "indexed units have different generation source selections",
            ));
        }
        if let Some(scope) = &scope {
            scope.validate(sources.len()).map_err(failure)?;
            for (local, original) in scope.chart_source_sectors().iter().enumerate() {
                if self
                    .original_sources
                    .insert(sources[local], *original)
                    .is_some()
                {
                    return Err(failure("duplicate source lineage in indexed units"));
                }
            }
        }
        if self.combined.is_none() {
            self.source_scope = scope;
        }
        for (order, value) in local
            .coefficient_orders
            .iter()
            .zip(&local.exact_expressions)
        {
            *self.exact.entry(*order).or_insert(Atom::Zero) += value;
        }
        if exact_only {
            if !local.sectors.is_empty() {
                return Err(failure("stochastic unit passed to exact-only assembly"));
            }
            // Exact setup intentionally does not retain rich chart expressions.
            if let Some(metadata) = &mut local.metadata {
                if let Some(scope) = &metadata.source_scope {
                    metadata.source_scope =
                        Some(scope.with_chart_sources(vec![]).map_err(failure)?);
                    metadata.charts.clear();
                } else {
                    local.metadata = None;
                }
            }
            local.contour_checks.clear();
            local.program_descriptor = local
                .program_descriptor
                .as_ref()
                .map(|descriptor| {
                    descriptor.for_payload_with_requests(
                        &[],
                        &local.exact_expressions,
                        None,
                        &local.exact_requests,
                    )
                })
                .transpose()?;
        } else if let Some(metadata) = &mut local.metadata {
            if metadata.charts.len() != sources.len() {
                return Err(failure(
                    "resident source mapping differs from native metadata",
                ));
            }
            if let Some(descriptor) = &mut local.program_descriptor {
                descriptor.remap_charts(sources)?;
            }
            for check in &mut local.contour_checks {
                check.chart_index = *sources
                    .get(check.chart_index)
                    .ok_or_else(|| failure("invalid local contour check index"))?;
            }
            for mut chart in metadata.charts.drain(..) {
                chart.source_index = *sources
                    .get(chart.source_index)
                    .ok_or_else(|| failure("invalid local source chart index"))?;
                chart.representative = *sources
                    .get(chart.representative)
                    .ok_or_else(|| failure("invalid local representative chart index"))?;
                if let Some(index) = chart.kernel_sector {
                    if index >= local.sectors.len() {
                        return Err(failure("invalid local kernel-sector index"));
                    }
                    chart.kernel_sector = Some(self.layouts.len() + index);
                }
                self.charts.push(chart);
            }
        } else if !sources.is_empty() {
            return Err(failure("resident source mapping lacks native metadata"));
        }
        if !local.sectors.is_empty() {
            self.layouts
                .push((local.orders.clone(), local.components.clone()));
        }
        local.portable_artifact = None;
        if let Some(combined) = &mut self.combined {
            if combined.program_recipe() != local.program_recipe() {
                return Err(failure("inconsistent explicit native recipe"));
            }
            match (
                &mut combined.program_descriptor,
                local.program_descriptor.take(),
            ) {
                (Some(existing), Some(incoming)) => existing.merge(incoming)?,
                (None, Some(incoming)) => combined.program_descriptor = Some(incoming),
                _ => {}
            }
            if combined.compilation_settings != local.compilation_settings
                || serde_json::to_value(&combined.precision)?
                    != serde_json::to_value(&local.precision)?
                || combined.runtime_parameters != local.runtime_parameters
                || combined.runtime_mass_constraints.len() != local.runtime_mass_constraints.len()
                || combined
                    .runtime_mass_constraints
                    .iter()
                    .zip(&local.runtime_mass_constraints)
                    .any(|(a, b)| a.name != b.name || a.expression != b.expression)
            {
                return Err(failure("inconsistent native record numerical policy"));
            }
            combined.sectors.append(&mut local.sectors);
            combined.contour_checks.append(&mut local.contour_checks);
            combined.exact_requests.append(&mut local.exact_requests);
            if combined.metadata.is_none() {
                combined.metadata = local.metadata.take();
            }
        } else {
            self.combined = Some(local);
        }
        Ok(())
    }

    pub(crate) fn finish(
        mut self,
        content_id: String,
        orders: &[i32],
        components: &[CoefficientComponent],
        exact_only: bool,
    ) -> Result<KernelSet, KernelError> {
        let mut combined = self
            .combined
            .take()
            .ok_or_else(|| failure("empty native archive"))?;
        self.charts.sort_by_key(|chart| chart.source_index);
        if !exact_only
            && self
                .charts
                .iter()
                .enumerate()
                .any(|(index, chart)| chart.source_index != index)
        {
            return Err(failure("incomplete original source-chart coverage"));
        }
        if exact_only {
            if let Some(metadata) = &mut combined.metadata {
                if let Some(scope) = &self.source_scope {
                    metadata.source_scope =
                        Some(scope.with_chart_sources(vec![]).map_err(failure)?);
                    metadata.charts.clear();
                } else {
                    combined.metadata = None;
                }
            }
        } else if let Some(metadata) = &mut combined.metadata {
            metadata.charts = self.charts;
            if let Some(scope) = self.source_scope {
                let originals = self.original_sources.into_values().collect::<Vec<_>>();
                if originals != scope.selection().source_sectors() {
                    return Err(failure(
                        "indexed archive lacks declared selected source charts",
                    ));
                }
                metadata.source_scope = Some(scope.with_chart_sources(originals).map_err(failure)?);
            }
        }
        let complex = components.contains(&CoefficientComponent::Imag);
        for (sector, (local_orders, local_components)) in
            combined.sectors.iter_mut().zip(self.layouts)
        {
            if local_orders != orders || local_components != components {
                let indices = local_orders
                    .iter()
                    .zip(&local_components)
                    .map(|(order, component)| {
                        orders
                            .iter()
                            .zip(components)
                            .position(|(o, c)| o == order && c == component)
                            .ok_or_else(|| {
                                failure("local output absent from complete coefficient layout")
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                sector.projection = Some(OutputProjection::new(
                    indices,
                    orders.len(),
                    complex,
                    &local_orders,
                    local_components,
                )?);
            }
        }
        let mut coefficient_orders = orders.to_vec();
        coefficient_orders.dedup();
        if self
            .exact
            .keys()
            .any(|order| coefficient_orders.binary_search(order).is_err())
        {
            return Err(failure(
                "exact output absent from complete coefficient layout",
            ));
        }
        combined.exact_expressions = coefficient_orders
            .iter()
            .map(|order| {
                crate::generation::normalize_exact_coefficient(
                    &self.exact.remove(order).unwrap_or(Atom::Zero),
                )
            })
            .collect();
        combined.exact_requests =
            crate::contour::functions::dynamic::requests::merge_exact_requests(
                &combined.exact_expressions,
                std::mem::take(&mut combined.exact_requests),
            )
            .map_err(failure)?;
        if exact_only {
            combined.program_descriptor = combined
                .program_descriptor
                .as_ref()
                .map(|descriptor| {
                    descriptor.for_payload_with_requests(
                        &[],
                        &combined.exact_expressions,
                        None,
                        &combined.exact_requests,
                    )
                })
                .transpose()?;
        }
        combined.coefficient_orders = coefficient_orders;
        combined.orders = orders.to_vec();
        combined.components = components.to_vec();
        combined.exact_coefficients = if combined.runtime_parameters.is_empty() {
            crate::kernel::exact::evaluate(
                &combined.exact_expressions,
                &Default::default(),
                complex,
            )?
        } else {
            vec![f64::NAN; combined.orders.len()]
        };
        combined.content_id = content_id;
        combined.template_content_id = None;
        Ok(combined)
    }
}
