//! Shared native decoding and selected-record assembly. No alternative recipe
//! or evaluator is decoded by these helpers.
use super::{RecordDescriptor, failure};
use crate::{
    generation::GenerationMetadata,
    kernel::{
        CompilationProgress, KernelError, KernelLoadOptions, KernelLoadProgress, KernelSet,
        projection::OutputProjection,
    },
    status::CoefficientComponent,
};
use std::{
    io::{Read, Seek, SeekFrom},
    ops::ControlFlow,
    time::Instant,
};
use symbolica::atom::Atom;

pub(super) struct RecordSelection<'a> {
    pub content_id: &'a str,
    pub orders: &'a [i32],
    pub components: &'a [CoefficientComponent],
    pub records: &'a [RecordDescriptor],
}

pub(super) fn read_record(
    reader: &mut (impl Read + Seek),
    descriptor: &RecordDescriptor,
    options: KernelLoadOptions,
) -> Result<KernelSet, KernelError> {
    let count = usize::try_from(descriptor.receipt.length).map_err(failure)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(count).map_err(failure)?;
    bytes.resize(count, 0);
    reader
        .seek(SeekFrom::Start(descriptor.offset))
        .map_err(failure)?;
    reader.read_exact(&mut bytes).map_err(failure)?;
    if options.validate && blake3::hash(&bytes).to_hex().as_str() != descriptor.receipt.digest {
        return Err(failure("selected record digest differs"));
    }
    if !bytes.starts_with(super::super::binary::PREFIX) {
        return Err(failure("nested archive or unsupported record codec"));
    }
    let kernels = KernelSet::from_bytes_with_options(&bytes, options)?;
    check_record(descriptor, &kernels)?;
    Ok(kernels)
}
pub(super) fn check_record(
    record: &RecordDescriptor,
    kernels: &KernelSet,
) -> Result<(), KernelError> {
    if kernels.template_content_id() != record.receipt.native_content_id
        || kernels.program_recipe() != record.receipt.program_recipe()
        || kernels.program_descriptor().is_some() != record.receipt.recipe.is_some()
        || kernels.orders() != record.receipt.orders
        || kernels.components() != record.receipt.components
        || kernels.sectors().len() != usize::from(record.receipt.dimension.is_some())
        || kernels.sectors().first().map(|s| s.dimension()) != record.receipt.dimension
        || kernels
            .runtime_parameters()
            .iter()
            .map(|p| p.get_name())
            .ne(record.receipt.runtime_parameters.iter().map(String::as_str))
        || kernels
            .generation_metadata()
            .map_or(0, |m| m.charts().len())
            != record.receipt.source_indices.len()
    {
        return Err(failure(
            "native record differs from its catalogue descriptor",
        ));
    }
    let chart_count = record.receipt.source_indices.len();
    if kernels.generation_metadata().is_some_and(|metadata| {
        metadata.charts().iter().enumerate().any(|(index, chart)| {
            chart.source_index != index || chart.representative >= chart_count
        })
    }) || kernels
        .contour_checks
        .iter()
        .any(|check| check.chart_index >= chart_count)
    {
        return Err(failure(
            "native record has invalid local chart/check indices",
        ));
    }
    Ok(())
}

pub(super) fn load_selected(
    reader: &mut (impl Read + Seek),
    selection: RecordSelection<'_>,
    options: KernelLoadOptions,
    exact_only: bool,
    progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
) -> Result<KernelSet, KernelError> {
    let started = Instant::now();
    let mut combined: Option<KernelSet> = None;
    let mut charts = Vec::new();
    let mut coefficient_orders = selection.orders.to_vec();
    coefficient_orders.dedup();
    let mut exact = vec![Atom::Zero; coefficient_orders.len()];
    let complex = selection.components.contains(&CoefficientComponent::Imag);
    let total = selection
        .records
        .iter()
        .filter(|r| r.sector.is_some())
        .count();
    let mut completed = 0;
    for index in 0..selection.records.len() {
        if exact_only && selection.records[index].sector.is_some() {
            continue;
        }
        if progress(&KernelLoadProgress::Restoring(CompilationProgress {
            completed,
            total,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        }))
        .is_break()
        {
            return Err(KernelError::Cancelled);
        }
        let mut local = read_record(reader, &selection.records[index], options)?;
        let record = &selection.records[index];
        for (order, value) in local
            .coefficient_orders
            .iter()
            .zip(&local.exact_expressions)
        {
            exact[coefficient_orders
                .binary_search(order)
                .expect("validated output map")] += value;
        }
        if exact_only {
            // Exact setup must not retain all analytically integrated chart
            // expressions while walking the archive. Keep offsets and native
            // parameter constraints only; rich inspection is record-local.
            local.metadata = None;
            local.contour_checks.clear();
            local.program_descriptor = local
                .program_descriptor
                .as_ref()
                .map(|descriptor| descriptor.for_payload(&[], &local.exact_expressions, None))
                .transpose()?;
        } else if let Some(mut metadata) = local.metadata.take() {
            if let Some(descriptor) = &mut local.program_descriptor {
                descriptor.remap_charts(&record.receipt.source_indices)?;
            }
            for check in &mut local.contour_checks {
                check.chart_index = *record
                    .receipt
                    .source_indices
                    .get(check.chart_index)
                    .ok_or_else(|| failure("invalid local contour check index"))?;
            }
            for mut chart in metadata.charts.drain(..) {
                chart.source_index = *record
                    .receipt
                    .source_indices
                    .get(chart.source_index)
                    .ok_or_else(|| failure("invalid local source chart index"))?;
                chart.representative = *record
                    .receipt
                    .source_indices
                    .get(chart.representative)
                    .ok_or_else(|| failure("invalid local representative chart index"))?;
                chart.kernel_sector = record.sector;
                charts.push(chart);
            }
            local.metadata = Some(GenerationMetadata {
                domain: metadata.domain,
                charts: Vec::new(),
            });
        }
        if let Some(sector) = local.sectors.first_mut() {
            if record.receipt.orders != selection.orders
                || record.receipt.components != selection.components
            {
                sector.projection = Some(OutputProjection::new(
                    record.output_indices.clone(),
                    selection.orders.len(),
                    complex,
                    &record.receipt.orders,
                    record.receipt.components.clone(),
                )?);
            }
            completed += 1;
        }
        local.portable_artifact = None;
        if let Some(combined) = &mut combined {
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
            if combined.metadata.is_none() {
                combined.metadata = local.metadata.take();
            }
        } else {
            combined = Some(local);
        }
    }
    let mut combined = combined.ok_or_else(|| failure("empty native archive"))?;
    charts.sort_by_key(|chart| chart.source_index);
    if !exact_only
        && charts
            .iter()
            .enumerate()
            .any(|(index, chart)| chart.source_index != index)
    {
        return Err(failure("incomplete original source-chart coverage"));
    }
    if exact_only {
        combined.metadata = None;
    } else if let Some(metadata) = &mut combined.metadata {
        metadata.charts = charts;
    }
    combined.coefficient_orders = coefficient_orders;
    combined.orders = selection.orders.to_vec();
    combined.components = selection.components.to_vec();
    combined.exact_expressions = exact;
    combined.exact_coefficients = if combined.runtime_parameters.is_empty() {
        crate::kernel::exact::evaluate(&combined.exact_expressions, &Default::default(), complex)?
    } else {
        vec![f64::NAN; combined.orders.len()]
    };
    combined.content_id = selection.content_id.to_owned();
    combined.template_content_id = None;
    if progress(&KernelLoadProgress::Restoring(CompilationProgress {
        completed,
        total,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    }))
    .is_break()
    {
        return Err(KernelError::Cancelled);
    }
    Ok(combined)
}
