//! Shared native decoding and selected-record assembly. No alternative recipe
//! or evaluator is decoded by these helpers.
use super::{RecordDescriptor, assembly::ResidentAssembly, failure};
use crate::{
    kernel::{CompilationProgress, KernelError, KernelLoadOptions, KernelLoadProgress, KernelSet},
    status::CoefficientComponent,
};
use std::{
    io::{Read, Seek, SeekFrom},
    ops::ControlFlow,
    time::Instant,
};

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
    read_record_with_retention(reader, descriptor, options, true)
}

fn read_record_with_retention(
    reader: &mut (impl Read + Seek),
    descriptor: &RecordDescriptor,
    options: KernelLoadOptions,
    retain: bool,
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
    let kernels = super::super::owned_record(bytes, options, retain)?;
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
    let mut assembly = ResidentAssembly::default();
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
        let record = &selection.records[index];
        let local = read_record_with_retention(reader, record, options, false)?;
        completed += usize::from(!local.sectors.is_empty());
        assembly.push(local, &record.receipt.source_indices, exact_only)?;
    }
    let combined = assembly.finish(
        selection.content_id.to_owned(),
        selection.orders,
        selection.components,
        exact_only,
    )?;
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
