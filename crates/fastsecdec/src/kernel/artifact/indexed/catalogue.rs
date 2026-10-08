use super::failure;
use crate::{
    kernel::{EvaluatorStatistics, KernelError},
    status::CoefficientComponent,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Small receipt returned by a completed worker. It contains no native Atoms,
/// evaluator instructions, expression text or executable owner.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordReceipt {
    pub version: u32,
    pub length: u64,
    pub digest: String,
    pub native_content_id: String,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub runtime_parameters: Vec<String>,
    pub dimension: Option<usize>,
    pub statistics: Option<EvaluatorStatistics>,
    /// Local chart ordinal -> original source-chart ordinal.
    pub source_indices: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDescriptor {
    pub offset: u64,
    pub receipt: RecordReceipt,
    /// Local numerical output -> the complete catalogue vector.
    pub output_indices: Vec<usize>,
    pub sector: Option<usize>,
}

/// Complete immutable index. Local record layouts remain unchanged when later
/// records extend the Laurent range or introduce imaginary components.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelCatalogue {
    pub version: u32,
    pub content_id: String,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub runtime_parameters: Vec<String>,
    pub records: Vec<RecordDescriptor>,
    /// Bytes before the compact footer. Record ranges never include the footer.
    pub records_end: u64,
}

impl RecordReceipt {
    pub(crate) fn validate(&self) -> Result<(), KernelError> {
        if self.version != 1 || self.length == 0 {
            return Err(failure("unsupported or empty worker record"));
        }
        super::super::validate_content_id(&self.digest)?;
        super::super::validate_content_id(&self.native_content_id)?;
        validate_layout(&self.orders, &self.components)?;
        if self.dimension.is_some() != self.statistics.is_some()
            || self.dimension == Some(0)
            || self
                .runtime_parameters
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != self.runtime_parameters.len()
            || self
                .source_indices
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(failure("invalid worker record descriptor"));
        }
        Ok(())
    }
}

pub(super) fn validate_layout(
    orders: &[i32],
    components: &[CoefficientComponent],
) -> Result<(), KernelError> {
    use CoefficientComponent::{Imag, Real};
    if orders.is_empty() || orders.len() != components.len() {
        return Err(failure("empty or inconsistent coefficient layout"));
    }
    let complex = components.contains(&Imag);
    let width = if complex { 2 } else { 1 };
    if !orders.len().is_multiple_of(width) {
        return Err(failure("inconsistent complex coefficient layout"));
    }
    let mut previous = None;
    for (orders, components) in orders.chunks(width).zip(components.chunks(width)) {
        if components[0] != Real
            || (complex && (components[1] != Imag || orders[1] != orders[0]))
            || previous.is_some_and(|value: i32| value.checked_add(1) != Some(orders[0]))
        {
            return Err(failure("invalid Laurent/component ordering"));
        }
        previous = Some(orders[0]);
    }
    Ok(())
}

impl KernelCatalogue {
    pub fn sector_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.sector.is_some())
            .count()
    }
    pub fn sector(&self, sector: usize) -> Result<&RecordDescriptor, KernelError> {
        self.records
            .iter()
            .find(|record| record.sector == Some(sector))
            .ok_or_else(|| failure(format!("unknown sector {sector}")))
    }
    pub(crate) fn finish(
        mut records: Vec<RecordDescriptor>,
        records_end: u64,
    ) -> Result<Self, KernelError> {
        if records.is_empty() {
            return Err(failure(
                "a complete archive requires a record, including for an exact-only integral",
            ));
        }
        // Physical storage follows completion order. Scientific order and identity
        // follow stable source-chart identities, with the shared exact row first.
        records.sort_by(|a, b| {
            a.receipt
                .source_indices
                .cmp(&b.receipt.source_indices)
                .then_with(|| {
                    a.receipt
                        .native_content_id
                        .cmp(&b.receipt.native_content_id)
                })
        });
        let runtime_parameters = records[0].receipt.runtime_parameters.clone();
        let low = records.iter().map(|r| r.receipt.orders[0]).min().unwrap();
        let high = records
            .iter()
            .map(|r| *r.receipt.orders.last().unwrap())
            .max()
            .unwrap();
        let complex = records
            .iter()
            .any(|r| r.receipt.components.contains(&CoefficientComponent::Imag));
        let coefficient_count = i64::from(high) - i64::from(low) + 1;
        if coefficient_count > 1_000_000 {
            return Err(failure("unreasonable Laurent layout span"));
        }
        let mut orders = Vec::new();
        let mut components = Vec::new();
        for order in low..=high {
            orders.push(order);
            components.push(CoefficientComponent::Real);
            if complex {
                orders.push(order);
                components.push(CoefficientComponent::Imag);
            }
        }
        let mut next_sector = 0;
        for record in &mut records {
            record.receipt.validate()?;
            if record.receipt.runtime_parameters != runtime_parameters {
                return Err(failure("worker runtime parameter schemas differ"));
            }
            record.output_indices = record
                .receipt
                .orders
                .iter()
                .zip(&record.receipt.components)
                .map(|(order, component)| {
                    orders
                        .iter()
                        .zip(&components)
                        .position(|pair| pair == (order, component))
                        .unwrap()
                })
                .collect();
            record.sector = record.receipt.dimension.map(|_| {
                let id = next_sector;
                next_sector += 1;
                id
            });
        }
        let mut catalogue = Self {
            version: 1,
            content_id: String::new(),
            orders,
            components,
            runtime_parameters,
            records,
            records_end,
        };
        catalogue.content_id = catalogue.identity()?;
        catalogue.validate(true)?;
        Ok(catalogue)
    }

    fn identity(&self) -> Result<String, KernelError> {
        // Byte offsets, context-dependent transport digests and completion order
        // are intentionally absent: native semantic IDs already bind each record.
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-indexed-semantic-v1\0");
        serde_json::to_writer(
            &mut hash,
            &(&self.orders, &self.components, &self.runtime_parameters),
        )?;
        for record in &self.records {
            serde_json::to_writer(
                &mut hash,
                &(
                    &record.receipt.native_content_id,
                    &record.receipt.source_indices,
                    &record.output_indices,
                    record.sector,
                ),
            )?;
        }
        Ok(hash.finalize().to_hex().to_string())
    }

    /// Cheap schema/range checks are mandatory; semantic digest recomputation is
    /// controlled by the caller's existing trusted-cache validation policy.
    pub fn validate(&self, integrity: bool) -> Result<(), KernelError> {
        if self.version != 1 || self.records.is_empty() {
            return Err(failure("unsupported or empty catalogue"));
        }
        super::super::validate_content_id(&self.content_id)?;
        validate_layout(&self.orders, &self.components)?;
        let mut ranges = Vec::new();
        let mut source_indices = BTreeSet::new();
        let mut next_sector = 0;
        for record in &self.records {
            record.receipt.validate()?;
            let end = record
                .offset
                .checked_add(record.receipt.length)
                .ok_or_else(|| failure("record range overflow"))?;
            if record.offset < super::MAGIC.len() as u64
                || end > self.records_end
                || record.receipt.runtime_parameters != self.runtime_parameters
                || record.output_indices.len() != record.receipt.orders.len()
                || record
                    .receipt
                    .source_indices
                    .iter()
                    .any(|id| !source_indices.insert(*id))
            {
                return Err(failure("invalid or duplicate indexed record range/schema"));
            }
            for ((order, component), index) in record
                .receipt
                .orders
                .iter()
                .zip(&record.receipt.components)
                .zip(&record.output_indices)
            {
                if self.orders.get(*index) != Some(order)
                    || self.components.get(*index) != Some(component)
                {
                    return Err(failure("invalid indexed coefficient projection"));
                }
            }
            if record.receipt.dimension.is_some() {
                if record.sector != Some(next_sector) {
                    return Err(failure("noncanonical numerical sector ordering"));
                }
                next_sector += 1;
            } else if record.sector.is_some() {
                return Err(failure("exact-only record has a numerical sector"));
            }
            ranges.push((record.offset, end));
        }
        ranges.sort_unstable();
        let mut end = super::MAGIC.len() as u64;
        for (start, next) in ranges {
            if start != end {
                return Err(failure("overlapping or missing native record bytes"));
            }
            end = next;
        }
        if end != self.records_end || (integrity && self.identity()? != self.content_id) {
            return Err(failure("catalogue content identity or extent differs"));
        }
        Ok(())
    }
}
