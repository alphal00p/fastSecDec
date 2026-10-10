//! Caller-owned process boundary. Serialized work is untrusted transport, never
//! a verified continuation or permission to publish a complete integral.
use super::*;
use crate::generation::streaming::{
    RecordRef,
    codec::{self, Atoms},
};
use crate::kernel::indexed::{ProgramArchiveWriter, RecordReceipt};
use std::io::{Read, Seek, Write};

const KIND: &str = "threshold-compilation-work-plan-v1";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdCompilationWork {
    plan: RecordRef,
    index: usize,
}
/// A worker receipt binds the immutable preparation record actually decoded.
/// The live plan validates it before copying the exact hashed program bytes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdWorkReceipt {
    work: ThresholdCompilationWork,
    record: RecordReceipt,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    tables: crate::kernel::threshold_owner::StagedTablePlan,
    specifications: Vec<Specification>,
    precision: PrecisionPolicy,
    settings: CompilationSettings,
}
pub(super) fn write_plan(prepared: &Prepared) -> Result<RecordRef, KernelError> {
    let (tables, symbols) = prepared.tables.descriptor();
    codec::write(
        &prepared.root,
        "threshold-work-plan",
        KIND,
        &Plan {
            tables,
            specifications: prepared.specifications.clone(),
            precision: prepared.precision.clone(),
            settings: prepared.settings,
        },
        Atoms::default(),
        symbols,
    )
    .map_err(failure)
}
impl ThresholdCompilationWork {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn plan_record(&self) -> &RecordRef {
        &self.plan
    }
    /// Run one native job synchronously and write one native record. The limit
    /// covers imported transport bytes, not decoded RSS or compiler allocations.
    /// The caller owns process lifetime, concurrency, RSS and durable publication.
    pub fn compile_record(
        &self,
        root: &Path,
        maximum_bytes: u64,
        writer: &mut impl Write,
    ) -> Result<ThresholdWorkReceipt, KernelError> {
        let remaining = maximum_bytes
            .checked_sub(self.plan.bytes)
            .ok_or_else(|| failure("native plan transport limit"))?;
        let (plan, atoms, symbols): (Plan, _, _) =
            codec::read(root, &self.plan, KIND).map_err(failure)?;
        if !atoms.atoms.is_empty() {
            return Err(failure("native work plan contains nonlocal Atom payload"));
        }
        plan.precision.validate()?;
        plan.settings.validate()?;
        let tables = StagedTables::from_descriptor(plan.tables, symbols)?;
        let specification = plan
            .specifications
            .get(self.index)
            .ok_or_else(|| failure("worker index out of range"))?;
        match (&specification.kind, &specification.vector) {
            (ThresholdJobKind::Setup {}, None) => {}
            (ThresholdJobKind::Exact { .. }, Some(vector))
                if matches!(vector.kind, record::VectorKind::Exact {}) => {}
            (ThresholdJobKind::Stochastic { contribution }, Some(vector))
                if matches!(vector.kind, record::VectorKind::Stochastic {})
                    && contribution.0 == vector.chart => {}
            _ => return Err(failure("worker kind/vector association")),
        }
        let prepared = Arc::new(Prepared {
            token: Arc::new(()),
            root: root.into(),
            tables,
            specifications: plan.specifications,
            precision: plan.precision,
            settings: plan.settings,
            work_record: None,
        });
        let completion = ThresholdCompilationJob {
            prepared,
            index: self.index,
        }
        .run_bounded(remaining)?;
        let record = completion.write_record(writer)?;
        Ok(ThresholdWorkReceipt {
            work: self.clone(),
            record,
        })
    }
}
impl ThresholdWorkReceipt {
    pub fn work(&self) -> &ThresholdCompilationWork {
        &self.work
    }
    pub fn record(&self) -> &RecordReceipt {
        &self.record
    }
}
impl ThresholdCompilationPlan {
    pub fn work(&self, index: usize) -> Result<ThresholdCompilationWork, KernelError> {
        if index >= self.job_count() {
            return Err(failure("job index out of range"));
        }
        Ok(ThresholdCompilationWork {
            plan: self
                .prepared
                .work_record
                .clone()
                .ok_or_else(|| failure("unissued worker plan"))?,
            index,
        })
    }
    /// Accept durable output only against this live, natively authorized plan.
    /// Resume must reconstruct/reverify preparation before calling this method;
    /// neither the receipt nor the work descriptor reconstitutes authority.
    pub fn validate_work_receipt(&self, receipt: &ThresholdWorkReceipt) -> Result<(), KernelError> {
        if receipt.work != self.work(receipt.work.index)? {
            return Err(failure("foreign worker preparation record"));
        }
        receipt.record.validate()?;
        let summary = self
            .prepared
            .tables
            .publication(self.prepared.settings, &self.prepared.precision)?;
        let expected = summary
            .expected
            .get(receipt.work.index)
            .ok_or_else(|| failure("unexpected worker result"))?;
        let actual = receipt
            .record
            .threshold
            .as_ref()
            .ok_or_else(|| failure("worker omitted threshold lineage"))?;
        if actual.parent != summary.parent
            || actual.carrier != expected.carrier
            || actual.lineage != expected.lineage
            || receipt.record.native_content_id != expected.native_content_id
        {
            return Err(failure(
                "worker result differs from issued mathematical job",
            ));
        }
        Ok(())
    }
    pub fn append_work_record<W: Write + Seek>(
        &self,
        writer: &mut ProgramArchiveWriter<W>,
        reader: &mut impl Read,
        receipt: ThresholdWorkReceipt,
    ) -> Result<(), KernelError> {
        self.validate_work_receipt(&receipt)?;
        // Native archive copying checks the digest of exactly the copied bytes.
        writer.append_record(
            crate::kernel::ProgramRecipe::ThresholdV1,
            reader,
            receipt.record,
        )
    }
}
