use super::{KernelCatalogue, RecordDescriptor, failure};
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

/// A catalogue and a seekable source, with no retained evaluator or rich chart
/// metadata. Loading a record reads and restores only its own native payload.
pub struct IndexedReader<R> {
    reader: R,
    catalogue: KernelCatalogue,
    options: KernelLoadOptions,
}
impl<R: Read + Seek> IndexedReader<R> {
    pub fn new(
        mut reader: R,
        catalogue: KernelCatalogue,
        options: KernelLoadOptions,
    ) -> Result<Self, KernelError> {
        catalogue.validate(options.validate)?;
        reader.seek(SeekFrom::Start(0)).map_err(failure)?;
        let mut header = vec![0u8; super::MAGIC.len()];
        reader.read_exact(&mut header).map_err(failure)?;
        let length = reader.seek(SeekFrom::End(0)).map_err(failure)?;
        let trailer = (super::FOOTER.len() + 8) as u64;
        if header != super::MAGIC || length < catalogue.records_end.saturating_add(trailer) {
            return Err(failure("missing header or truncated indexed data"));
        }
        reader
            .seek(SeekFrom::Start(length - trailer))
            .map_err(failure)?;
        let mut count = [0u8; 8];
        reader.read_exact(&mut count).map_err(failure)?;
        let mut footer = vec![0u8; super::FOOTER.len()];
        reader.read_exact(&mut footer).map_err(failure)?;
        if footer != super::FOOTER
            || u64::from_le_bytes(count) != length - trailer - catalogue.records_end
        {
            return Err(failure("incomplete indexed catalogue commit"));
        }
        Ok(Self {
            reader,
            catalogue,
            options,
        })
    }
    /// Read just the bounded footer index, without decoding any native record.
    pub fn from_reader(mut reader: R, options: KernelLoadOptions) -> Result<Self, KernelError> {
        let length = reader.seek(SeekFrom::End(0)).map_err(failure)?;
        let trailer = (super::FOOTER.len() + 8) as u64;
        let footer_start = length
            .checked_sub(trailer)
            .ok_or_else(|| failure("truncated indexed footer"))?;
        reader
            .seek(SeekFrom::Start(footer_start))
            .map_err(failure)?;
        let mut count = [0u8; 8];
        reader.read_exact(&mut count).map_err(failure)?;
        let mut magic = vec![0; super::FOOTER.len()];
        reader.read_exact(&mut magic).map_err(failure)?;
        let count = u64::from_le_bytes(count);
        if magic != super::FOOTER || count > 128 * 1024 * 1024 {
            return Err(failure("invalid or excessive catalogue footer"));
        }
        let start = footer_start
            .checked_sub(count)
            .ok_or_else(|| failure("catalogue range underflow"))?;
        reader.seek(SeekFrom::Start(start)).map_err(failure)?;
        let mut bytes = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).map_err(failure)?;
        let catalogue: KernelCatalogue = serde_json::from_slice(&bytes)?;
        if catalogue.records_end != start {
            return Err(failure("catalogue extent differs from trailer"));
        }
        Self::new(reader, catalogue, options)
    }
    pub fn catalogue(&self) -> &KernelCatalogue {
        &self.catalogue
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
    pub fn load_record(&mut self, index: usize) -> Result<KernelSet, KernelError> {
        let descriptor = self
            .catalogue
            .records
            .get(index)
            .ok_or_else(|| failure(format!("unknown record {index}")))?;
        let count = usize::try_from(descriptor.receipt.length).map_err(failure)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(count).map_err(failure)?;
        bytes.resize(count, 0);
        self.reader
            .seek(SeekFrom::Start(descriptor.offset))
            .map_err(failure)?;
        self.reader.read_exact(&mut bytes).map_err(failure)?;
        if self.options.validate
            && blake3::hash(&bytes).to_hex().as_str() != descriptor.receipt.digest
        {
            return Err(failure("selected record digest differs"));
        }
        if !bytes.starts_with(super::super::binary::PREFIX) {
            return Err(failure("nested archive or unsupported record codec"));
        }
        let kernels = KernelSet::from_bytes_with_options(&bytes, self.options)?;
        Self::check_record(descriptor, &kernels)?;
        Ok(kernels)
    }
    fn check_record(record: &RecordDescriptor, kernels: &KernelSet) -> Result<(), KernelError> {
        if kernels.template_content_id() != record.receipt.native_content_id
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
        Ok(())
    }
    /// Return the original local output layout. `catalogue().sector(id)` supplies
    /// the map into the complete Laurent vector; no parent evaluator is retained.
    pub fn load_sector(&mut self, sector: usize) -> Result<KernelSet, KernelError> {
        let index = self
            .catalogue
            .records
            .iter()
            .position(|r| r.sector == Some(sector))
            .ok_or_else(|| failure(format!("unknown sector {sector}")))?;
        self.load_record(index)
    }
    pub fn load_all(&mut self) -> Result<KernelSet, KernelError> {
        self.load_all_with_progress(&mut |_| ControlFlow::Continue(()))
    }
    /// Restore only exact-offset/constraint records. No numerical-sector record
    /// is read or compiled. Call `bind_parameters` on this small native owner in
    /// the caller's setup process, then return its compact numerical manifest.
    pub fn load_exact(&mut self) -> Result<KernelSet, KernelError> {
        self.load_with_progress(true, &mut |_| ControlFlow::Continue(()))
    }
    /// Ordinary resident loading, with only numerical output scattering when
    /// records have different local layouts. Never rewrites or optimizes IR.
    pub fn load_all_with_progress(
        &mut self,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.load_with_progress(false, progress)
    }
    fn load_with_progress(
        &mut self,
        exact_only: bool,
        progress: &mut impl FnMut(&KernelLoadProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        let started = Instant::now();
        let mut combined: Option<KernelSet> = None;
        let mut charts = Vec::new();
        let mut coefficient_orders = self.catalogue.orders.clone();
        coefficient_orders.dedup();
        let mut exact = vec![Atom::Zero; coefficient_orders.len()];
        let complex = self
            .catalogue
            .components
            .contains(&CoefficientComponent::Imag);
        let total = self.catalogue.sector_count();
        let mut completed = 0;
        for index in 0..self.catalogue.records.len() {
            if exact_only && self.catalogue.records[index].sector.is_some() {
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
            let mut local = self.load_record(index)?;
            let record = &self.catalogue.records[index];
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
            } else if let Some(mut metadata) = local.metadata.take() {
                for mut chart in metadata.charts.drain(..) {
                    chart.source_index = record.receipt.source_indices[chart.source_index];
                    chart.representative = record.receipt.source_indices[chart.representative];
                    chart.kernel_sector = record.sector;
                    charts.push(chart);
                }
                local.metadata = Some(GenerationMetadata {
                    domain: metadata.domain,
                    charts: Vec::new(),
                });
            }
            if let Some(sector) = local.sectors.first_mut() {
                if record.receipt.orders != self.catalogue.orders
                    || record.receipt.components != self.catalogue.components
                {
                    sector.projection = Some(OutputProjection::new(
                        record.output_indices.clone(),
                        self.catalogue.orders.len(),
                        complex,
                        &record.receipt.orders,
                        record.receipt.components.clone(),
                    )?);
                }
                completed += 1;
            }
            local.portable_artifact = None;
            if let Some(combined) = &mut combined {
                if combined.compilation_settings != local.compilation_settings
                    || serde_json::to_value(&combined.precision)?
                        != serde_json::to_value(&local.precision)?
                    || combined.runtime_parameters != local.runtime_parameters
                    || combined.runtime_mass_constraints.len()
                        != local.runtime_mass_constraints.len()
                    || combined
                        .runtime_mass_constraints
                        .iter()
                        .zip(&local.runtime_mass_constraints)
                        .any(|(a, b)| a.name != b.name || a.expression != b.expression)
                {
                    return Err(failure("inconsistent native record numerical policy"));
                }
                combined.sectors.append(&mut local.sectors);
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
        combined.orders = self.catalogue.orders.clone();
        combined.components = self.catalogue.components.clone();
        combined.exact_expressions = exact;
        combined.exact_coefficients = if combined.runtime_parameters.is_empty() {
            crate::kernel::exact::evaluate(
                &combined.exact_expressions,
                &Default::default(),
                complex,
            )?
        } else {
            vec![f64::NAN; combined.orders.len()]
        };
        combined.content_id = self.catalogue.content_id.clone();
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
}
