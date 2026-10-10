use super::{KernelCatalogue, RecordDescriptor, RecordReceipt, failure};
use crate::kernel::{KernelError, KernelSet};
use std::io::{Read, Seek, Write};

/// Write one standalone native worker record. Callers own durability and atomic
/// publication of the associated receipt; a failed write never returns one.
pub fn write_record(
    writer: &mut impl Write,
    kernels: &KernelSet,
    source_indices: Vec<usize>,
) -> Result<RecordReceipt, KernelError> {
    if kernels.sectors().len() > 1 {
        return Err(failure("a worker record must contain at most one sector"));
    }
    if !kernels.sectors().is_empty()
        && kernels
            .exact_expressions
            .iter()
            .any(|value| value != &symbolica::atom::Atom::Zero)
    {
        return Err(failure(
            "split stochastic and exact contributions with write_unit",
        ));
    }
    if kernels
        .generation_metadata()
        .map_or(0, |metadata| metadata.charts().len())
        != source_indices.len()
    {
        return Err(failure(
            "source-chart mapping differs from the worker metadata",
        ));
    }
    let bytes = kernels.artifact_bytes()?;
    if !bytes.starts_with(super::super::binary::PREFIX) {
        return Err(failure(
            "worker record must be a native local kernel, not an archive",
        ));
    }
    let receipt = receipt(
        kernels,
        bytes,
        kernels.template_content_id().into(),
        source_indices,
    );
    receipt.validate()?;
    writer.write_all(bytes).map_err(failure)?;
    Ok(receipt)
}

/// Persist a completed generation unit as independently readable exact-only
/// and stochastic records. Receipt order matches their consecutive byte ranges
/// in `writer`; a coordinator can copy them without decoding native objects.
pub fn write_unit(
    writer: &mut (impl Write + Seek),
    kernels: &KernelSet,
    source_indices: Vec<usize>,
) -> Result<Vec<RecordReceipt>, KernelError> {
    let mut receipts = Vec::new();
    for_each_unit_record(kernels, &source_indices, |bytes, receipt| {
        writer.write_all(bytes).map_err(failure)?;
        receipts.push(receipt);
        Ok(())
    })?;
    Ok(receipts)
}

/// One partition/serialization implementation for worker files, archives and
/// selected resident retention. The callback borrows at most one record's bytes.
pub(super) fn for_each_unit_record(
    kernels: &KernelSet,
    source_indices: &[usize],
    mut emit: impl FnMut(&[u8], RecordReceipt) -> Result<(), KernelError>,
) -> Result<(), KernelError> {
    if kernels.sectors().len() > 1
        || kernels
            .metadata
            .as_ref()
            .map_or(0, |metadata| metadata.charts.len())
            != source_indices.len()
    {
        return Err(failure(
            "generation unit must have at most one sector and complete source-chart mapping",
        ));
    }
    for sector in std::iter::once(None).chain((0..kernels.sectors().len()).map(Some)) {
        let (bytes, mut receipt) = partition(kernels, sector)?;
        for source in &mut receipt.source_indices {
            *source = *source_indices
                .get(*source)
                .ok_or_else(|| failure("invalid generation-unit source index"))?;
        }
        receipt.validate()?;
        emit(&bytes, receipt)?;
    }
    Ok(())
}

fn receipt(
    kernels: &KernelSet,
    bytes: &[u8],
    id: String,
    source_indices: Vec<usize>,
) -> RecordReceipt {
    RecordReceipt {
        version: if kernels.program_descriptor().is_some() {
            2
        } else {
            1
        },
        recipe: kernels
            .program_descriptor()
            .map(|descriptor| descriptor.recipe()),
        length: bytes.len() as u64,
        digest: blake3::hash(bytes).to_hex().to_string(),
        native_content_id: id,
        orders: kernels.orders().to_vec(),
        components: kernels.components().to_vec(),
        runtime_parameters: kernels
            .runtime_parameters()
            .iter()
            .map(|p| p.get_name().into())
            .collect(),
        dimension: kernels.sectors().first().map(|sector| sector.dimension()),
        statistics: kernels
            .sectors()
            .first()
            .map(|sector| sector.statistics().clone()),
        source_indices,
    }
}

pub(super) fn partition(
    kernels: &KernelSet,
    sector: Option<usize>,
) -> Result<(Vec<u8>, RecordReceipt), KernelError> {
    let (id, bytes, source_indices) = super::super::binary::partition(kernels, sector)?;
    let mut receipt = receipt(kernels, &bytes, id, source_indices);
    receipt.dimension = sector.map(|i| kernels.sectors()[i].dimension());
    receipt.statistics = sector.map(|i| kernels.sectors()[i].statistics().clone());
    if let Some(projection) = sector.and_then(|i| kernels.sectors()[i].projection.as_ref()) {
        receipt.orders = projection.local_flat_orders();
        receipt.components = projection.local_components.clone();
    }
    receipt.validate()?;
    Ok((bytes, receipt))
}

/// Append-only caller-owned writer. Incoming records are copied in fixed-size
/// blocks with their receipt digest checked before admission. A failed append
/// poisons the writer: callers must discard/recover the staging file rather than
/// publish a catalogue containing a partial record.
pub struct IndexedWriter<W> {
    writer: W,
    records: Vec<RecordDescriptor>,
    failed: bool,
}
impl<W: Write + Seek> IndexedWriter<W> {
    pub fn new(mut writer: W) -> Result<Self, KernelError> {
        if writer.stream_position().map_err(failure)? != 0 {
            return Err(failure("indexed writer must start at byte zero"));
        }
        writer.write_all(super::MAGIC).map_err(failure)?;
        Ok(Self {
            writer,
            records: Vec::new(),
            failed: false,
        })
    }
    pub fn append_record(
        &mut self,
        reader: &mut impl Read,
        receipt: RecordReceipt,
    ) -> Result<(), KernelError> {
        if self.failed {
            return Err(failure("cannot append to a failed archive writer"));
        }
        receipt.validate()?;
        self.failed = true;
        let offset = self.writer.stream_position().map_err(failure)?;
        super::transport::copy_record(&mut self.writer, reader, &receipt)?;
        self.records.push(RecordDescriptor {
            offset,
            receipt,
            output_indices: Vec::new(),
            sector: None,
        });
        self.failed = false;
        Ok(())
    }
    pub fn finish(mut self) -> Result<(W, KernelCatalogue), KernelError> {
        if self.failed {
            return Err(failure("cannot finish a failed archive writer"));
        }
        let end = self.writer.stream_position().map_err(failure)?;
        let catalogue = KernelCatalogue::finish(self.records, end)?;
        super::transport::write_footer(&mut self.writer, &catalogue, super::FOOTER)?;
        Ok((self.writer, catalogue))
    }
}
