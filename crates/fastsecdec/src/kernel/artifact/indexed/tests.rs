use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::io::{Cursor, Read, Seek, SeekFrom};
use symbolica::{atom::Atom, parse, symbol};

fn kernel(complex: bool, order: i32) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("indexed_checks::x")],
        symbol!("indexed_checks::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                parse!("2+3𝑖")
            } else {
                Atom::one()
            },
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+indexed_checks::x"),
                parse!("-1+indexed_checks::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            max_order: order,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    })
    .unwrap()
}

fn unit(kernels: &KernelSet, original: usize) -> (Vec<u8>, Vec<RecordReceipt>) {
    assert_eq!(kernels.sectors().len(), 1);
    let mut bytes = Cursor::new(Vec::new());
    let sources =
        (original..original + kernels.generation_metadata().unwrap().charts().len()).collect();
    let receipt = write_unit(&mut bytes, kernels, sources).unwrap();
    (bytes.into_inner(), receipt)
}
fn append(writer: &mut IndexedWriter<Cursor<Vec<u8>>>, bytes: &[u8], receipts: &[RecordReceipt]) {
    let mut source = bytes;
    for receipt in receipts {
        writer.append_record(&mut source, receipt.clone()).unwrap();
    }
    assert!(source.is_empty());
}

#[test]
fn indexed_native_roundtrip_and_heterogeneous_projection_do_not_change_programs() {
    let mut real = kernel(false, 0);
    let mut complex = kernel(true, 1);
    let (a, ar) = unit(&real, 0);
    let (b, br) = unit(&complex, 1);
    let mut writer = IndexedWriter::new(Cursor::new(Vec::new())).unwrap();
    append(&mut writer, &a, &ar);
    append(&mut writer, &b, &br);
    let (bytes, catalogue) = writer.finish().unwrap();
    assert_eq!(catalogue.orders, [0, 0, 1, 1]);
    assert_eq!(catalogue.sector_count(), 2);
    let mut reader = IndexedReader::from_reader(
        Cursor::new(bytes.get_ref()),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let exact = reader.load_exact().unwrap();
    assert!(exact.sectors().is_empty());
    assert_eq!(exact.exact_coefficients(), [0.0; 4]);
    let local = reader.load_sector(0).unwrap();
    assert_eq!(
        local.sectors()[0].program_bytes,
        real.sectors()[0].program_bytes
    );
    let mut all = reader.load_all().unwrap();
    let mut expected_real = [0.0];
    let mut expected_complex = [0.0; 4];
    real.sectors_mut()[0]
        .evaluate(&[0.25], &mut expected_real)
        .unwrap();
    complex.sectors_mut()[0]
        .evaluate(&[0.25], &mut expected_complex)
        .unwrap();
    let mut actual = [0.0; 4];
    all.sectors_mut()[0].evaluate(&[0.25], &mut actual).unwrap();
    assert_eq!(actual, [expected_real[0], 0.0, 0.0, 0.0]);
    all.sectors_mut()[1].evaluate(&[0.25], &mut actual).unwrap();
    assert_eq!(actual, expected_complex);
    let mut context = all.evaluation_context(0, Default::default()).unwrap();
    let mut batch = [0.0; 8];
    context
        .evaluate_weighted_batch(&[0.25, 0.25], &[2.0, 3.0], &mut batch)
        .unwrap();
    let expected_batch = [
        expected_real[0] * 2.0,
        0.0,
        0.0,
        0.0,
        expected_real[0] * 3.0,
        0.0,
        0.0,
        0.0,
    ];
    assert!(
        batch
            .iter()
            .zip(expected_batch)
            .all(|(actual, expected)| (actual - expected).abs() <= 1e-14 * expected.abs().max(1.0))
    );
    assert_eq!(
        all.sectors()[0].program_bytes,
        real.sectors()[0].program_bytes
    );
    let mut reloaded = KernelSet::from_bytes_with_options(
        &all.to_bytes().unwrap(),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    reloaded.sectors_mut()[0]
        .evaluate(&[0.25], &mut actual)
        .unwrap();
    assert_eq!(actual, [expected_real[0], 0.0, 0.0, 0.0]);
    // A reordered completion stream changes disk offsets, never its scientific ID.
    let mut reverse = IndexedWriter::new(Cursor::new(Vec::new())).unwrap();
    append(&mut reverse, &b, &br);
    append(&mut reverse, &a, &ar);
    assert_eq!(reverse.finish().unwrap().1.content_id, catalogue.content_id);
}

struct ObservedReader {
    bytes: Cursor<Vec<u8>>,
    reads: Vec<(u64, usize)>,
}
impl Read for ObservedReader {
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        let position = self.bytes.position();
        let count = self.bytes.read(into)?;
        self.reads.push((position, count));
        Ok(count)
    }
}
impl Seek for ObservedReader {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.bytes.seek(position)
    }
}

#[test]
fn selected_loading_and_exact_setup_never_read_unselected_sector_bytes() {
    let kernels = kernel(false, 0);
    let (bytes, catalogue) = to_bytes(&kernels).unwrap();
    let source = ObservedReader {
        bytes: Cursor::new(bytes),
        reads: Vec::new(),
    };
    let numerical = catalogue.sector(0).unwrap().clone();
    let mut reader =
        IndexedReader::new(source, catalogue, KernelLoadOptions { validate: true }).unwrap();
    reader.load_exact().unwrap();
    let source = reader.into_inner();
    assert!(source.reads.iter().all(
        |(offset, count)| *offset + *count as u64 <= numerical.offset
            || *offset >= numerical.offset + numerical.receipt.length
    ));
}

#[test]
fn indexed_corruption_ranges_and_interrupted_writes_are_rejected() {
    let kernels = kernel(false, 0);
    let (bytes, catalogue) = to_bytes(&kernels).unwrap();
    let mut corrupt = bytes.clone();
    let sector = catalogue.sector(0).unwrap();
    corrupt[(sector.offset + sector.receipt.length - 1) as usize] ^= 1;
    let mut reader = IndexedReader::new(
        Cursor::new(corrupt),
        catalogue.clone(),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    assert!(reader.load_exact().is_ok());
    assert!(
        reader
            .load_sector(0)
            .err()
            .unwrap()
            .to_string()
            .contains("digest")
    );
    let mut invalid = catalogue.clone();
    invalid.records[0].offset = u64::MAX;
    assert!(IndexedReader::new(Cursor::new(bytes.clone()), invalid, Default::default()).is_err());
    assert!(
        IndexedReader::from_reader(Cursor::new(&bytes[..bytes.len() - 1]), Default::default())
            .is_err()
    );
    let (record, receipts) = unit(&kernels, 0);
    let mut writer = IndexedWriter::new(Cursor::new(Vec::new())).unwrap();
    assert!(
        writer
            .append_record(&mut &record[..1], receipts[0].clone())
            .is_err()
    );
    assert!(writer.finish().is_err());
}
