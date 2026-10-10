//! Shared coefficient/schema and physical-extent checks for indexed versions.
//! A recipe layout is not an incomplete version-one archive.
use super::{RecordDescriptor, catalogue::validate_layout, failure};
use crate::{kernel::KernelError, status::CoefficientComponent};
use std::collections::BTreeSet;

pub(super) struct RecordLayout {
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub runtime_parameters: Vec<String>,
    pub records: Vec<RecordDescriptor>,
}

pub(super) fn finish_layout(
    mut records: Vec<RecordDescriptor>,
) -> Result<RecordLayout, KernelError> {
    if records.is_empty() {
        return Err(failure(
            "a complete recipe requires an exact or stochastic record",
        ));
    }
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
    for record in &records {
        record.receipt.validate()?;
        if record.receipt.runtime_parameters != runtime_parameters {
            return Err(failure("worker runtime parameter schemas differ"));
        }
    }
    let low = records.iter().map(|r| r.receipt.orders[0]).min().unwrap();
    let high = records
        .iter()
        .map(|r| *r.receipt.orders.last().unwrap())
        .max()
        .unwrap();
    if i64::from(high) - i64::from(low) + 1 > 1_000_000 {
        return Err(failure("unreasonable Laurent layout span"));
    }
    let complex = records
        .iter()
        .any(|r| r.receipt.components.contains(&CoefficientComponent::Imag));
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
            let index = next_sector;
            next_sector += 1;
            index
        });
    }
    Ok(RecordLayout {
        orders,
        components,
        runtime_parameters,
        records,
    })
}

pub(super) fn validate_records(
    orders: &[i32],
    components: &[CoefficientComponent],
    runtime_parameters: &[String],
    records: &[RecordDescriptor],
) -> Result<(), KernelError> {
    validate_layout(orders, components)?;
    if records.is_empty() {
        return Err(failure("empty native recipe"));
    }
    let mut sources = BTreeSet::new();
    let selection = records[0]
        .receipt
        .source_scope
        .as_ref()
        .map(|scope| scope.selection());
    let mut original_sources = std::collections::BTreeMap::new();
    let mut next_sector = 0;
    for record in records {
        record.receipt.validate()?;
        if record
            .receipt
            .source_scope
            .as_ref()
            .map(|scope| scope.selection())
            != selection
        {
            return Err(failure("inconsistent indexed generation source selection"));
        }
        if let Some(scope) = &record.receipt.source_scope {
            for (local, original) in record
                .receipt
                .source_indices
                .iter()
                .zip(scope.chart_source_sectors())
            {
                if original_sources.insert(*local, *original).is_some() {
                    return Err(failure("duplicate indexed source lineage"));
                }
            }
        }
        if record.receipt.runtime_parameters != runtime_parameters
            || record.output_indices.len() != record.receipt.orders.len()
            || record
                .receipt
                .source_indices
                .iter()
                .any(|id| !sources.insert(*id))
        {
            return Err(failure("invalid or duplicate indexed record schema"));
        }
        for ((order, component), index) in record
            .receipt
            .orders
            .iter()
            .zip(&record.receipt.components)
            .zip(&record.output_indices)
        {
            if orders.get(*index) != Some(order) || components.get(*index) != Some(component) {
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
    }
    if sources
        .into_iter()
        .enumerate()
        .any(|(expected, actual)| expected != actual)
    {
        return Err(failure("incomplete original source-chart coverage"));
    }
    if selection.is_some_and(|selection| {
        original_sources.into_values().collect::<Vec<_>>() != selection.source_sectors()
    }) {
        return Err(failure(
            "indexed records do not cover their declared original source subset",
        ));
    }
    Ok(())
}

pub(super) fn validate_ranges<'a>(
    records: impl Iterator<Item = &'a RecordDescriptor>,
    header_length: u64,
    records_end: u64,
) -> Result<(), KernelError> {
    let mut ranges = Vec::new();
    for record in records {
        let end = record
            .offset
            .checked_add(record.receipt.length)
            .ok_or_else(|| failure("record range overflow"))?;
        if record.offset < header_length || end > records_end {
            return Err(failure("invalid indexed record range"));
        }
        ranges.push((record.offset, end));
    }
    ranges.sort_unstable();
    let mut end = header_length;
    for (start, next) in ranges {
        if start != end {
            return Err(failure("overlapping or missing native record bytes"));
        }
        end = next;
    }
    if end != records_end {
        return Err(failure("catalogue extent differs"));
    }
    Ok(())
}
