//! Actual native primary transport and unchanged exact-IR compatibility.
use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, PrimaryEvaluatorRestoration as Restoration},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{io::Cursor, ops::ControlFlow};
use symbolica::{atom::Atom, parse, symbol};

fn kernel(complex: bool) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("primary_wire::x")],
        symbol!("primary_wire::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                parse!("2+3𝑖")
            } else {
                Atom::one()
            },
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+primary_wire::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Symjit,
        horner_iterations: 0,
        ..Default::default()
    })
    .unwrap()
}

fn values(kernels: &mut KernelSet) -> Vec<f64> {
    let mut result = Vec::new();
    for point in [0.125, 0.37, 0.875] {
        let mut output = vec![0.; kernels.orders().len()];
        kernels.sectors_mut()[0]
            .evaluate(&[point], &mut output)
            .unwrap();
        result.extend(output);
    }
    result
}

#[test]
fn native_primary_real_complex_roundtrip_retains_exact_program_and_loaded_bytes() {
    for complex in [false, true] {
        let mut original = kernel(complex);
        let expected = values(&mut original);
        let bytes = original.to_bytes().unwrap();
        assert!(bytes.starts_with(MAGIC));
        let record = decode(&bytes).unwrap();
        assert!(record.primaries[0].is_some());
        let id = original.content_id().to_owned();
        let exact = original.sectors[0].program_bytes.clone();
        for validate in [false, true] {
            let mut restored =
                KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate }).unwrap();
            assert_eq!(restored.content_id(), id);
            assert_eq!(restored.to_bytes().unwrap(), bytes);
            assert_eq!(restored.sectors[0].program_bytes, exact);
            assert_eq!(
                restored.sectors[0].primary_evaluator_restoration(),
                Restoration::CacheRestored
            );
            assert_eq!(values(&mut restored), expected);
            let mut clone = restored.try_clone().unwrap();
            assert!(std::sync::Arc::ptr_eq(
                restored.portable_artifact.as_ref().unwrap(),
                clone.portable_artifact.as_ref().unwrap()
            ));
            assert!(std::ptr::eq(
                restored.sectors[0].exact_program(),
                clone.sectors[0].exact_program()
            ));
            drop(restored);
            assert_eq!(
                std::thread::spawn(move || values(&mut clone))
                    .join()
                    .unwrap(),
                expected
            );
        }
        // An older exact-only record is still valid, recompiles only its backend,
        // and to_bytes preserves those original bytes instead of upgrading it.
        let mut legacy = KernelSet::from_bytes(record.base).unwrap();
        assert_eq!(legacy.content_id(), id);
        assert_eq!(legacy.to_bytes().unwrap(), record.base);
        assert_eq!(
            legacy.sectors[0].primary_evaluator_restoration(),
            Restoration::CacheMissing
        );
        assert_eq!(values(&mut legacy), expected);
        let moved = super::super::owned_record_with_parent(
            bytes.clone(),
            KernelLoadOptions::default(),
            false,
            None,
        )
        .unwrap();
        assert!(moved.portable_artifact.is_none());
        assert_eq!(moved.content_id(), id);
    }
}

#[test]
fn native_primary_shape_identity_count_and_decode_fail_without_optional_hashes() {
    let kernels = kernel(false);
    let bytes = kernels.to_bytes().unwrap();
    let record = decode(&bytes).unwrap();
    for invalid in [
        encode(&"f".repeat(64), record.base, &record.primaries).unwrap(),
        encode(record.content_id, record.base, &[]).unwrap(),
    ] {
        assert!(KernelSet::from_bytes(&invalid).is_err());
    }
    let exact = &kernels.sectors[0].program_bytes;
    for primary in [
        SavedPrimary::new(vec![0xff], exact, false, 1, 1),
        SavedPrimary::new(vec![0xff], exact, true, 1, 1),
        SavedPrimary::new(vec![0xff], exact, false, 2, 1),
    ] {
        let invalid = encode(record.content_id, record.base, &[Some(primary)]).unwrap();
        assert!(KernelSet::from_bytes(&invalid).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(KernelSet::from_bytes(&trailing).is_err());
    assert!(KernelSet::from_bytes(&bytes[..bytes.len() - 1]).is_err());

    // Incompatible owner bytes must never enter its native decoder. The exact
    // program and all its ordinary backend admission still apply.
    let primary = SavedPrimary::new(vec![0xff], exact, false, 1, 1).with_incompatible_owner();
    let incompatible = encode(record.content_id, record.base, &[Some(primary)]).unwrap();
    for validate in [false, true] {
        let mut restored =
            KernelSet::from_bytes_with_options(&incompatible, KernelLoadOptions { validate })
                .unwrap();
        assert_eq!(
            restored.sectors[0].primary_evaluator_restoration(),
            Restoration::CacheIncompatible
        );
        let expected: Vec<f64> = [0.125, 0.37, 0.875].map(|x| 1. / (1. + x)).into();
        assert_eq!(values(&mut restored), expected);
    }
}

#[test]
fn indexed_refresh_preserves_native_identity_and_moves_or_discards_record_bytes() {
    use crate::kernel::indexed::{IndexedReader, IndexedWriter, write_unit};
    let mut original = kernel(true);
    let expected = values(&mut original);
    let mut pieces = Cursor::new(Vec::new());
    let receipts = write_unit(&mut pieces, &original, vec![0]).unwrap();
    let mut archive = IndexedWriter::new(Cursor::new(Vec::new())).unwrap();
    let mut offset = 0;
    // Create a real old exact-only indexed archive using the same native receipts.
    for mut receipt in receipts {
        let end = offset + receipt.length as usize;
        let piece = base(&pieces.get_ref()[offset..end]);
        receipt.length = piece.len() as u64;
        receipt.digest = blake3::hash(piece).to_hex().to_string();
        archive.append_record(&mut &piece[..], receipt).unwrap();
        offset = end;
    }
    let (bytes, catalogue) = archive.finish().unwrap();
    let mut reader =
        IndexedReader::from_reader(bytes, KernelLoadOptions { validate: true }).unwrap();
    let (refreshed, new_catalogue) = reader
        .write_with_native_cache(Cursor::new(Vec::new()), |_| ControlFlow::Continue(()))
        .unwrap();
    assert_eq!(catalogue.content_id, new_catalogue.content_id);
    assert!(
        catalogue
            .records
            .iter()
            .zip(&new_catalogue.records)
            .any(|(a, b)| a.receipt.digest != b.receipt.digest)
    );
    for (a, b) in catalogue.records.iter().zip(&new_catalogue.records) {
        assert_eq!(a.receipt.native_content_id, b.receipt.native_content_id);
        assert_eq!(a.receipt.source_indices, b.receipt.source_indices);
    }
    let mut reader =
        IndexedReader::from_reader(refreshed, KernelLoadOptions { validate: true }).unwrap();
    let mut sector = reader.load_sector(0).unwrap();
    assert!(sector.artifact_bytes().unwrap().starts_with(MAGIC));
    assert_eq!(
        sector.sectors[0].primary_evaluator_restoration(),
        Restoration::CacheRestored
    );
    assert_eq!(values(&mut sector), expected);
    let mut all = reader.load_all().unwrap();
    assert_eq!(
        all.sectors[0].primary_evaluator_restoration(),
        Restoration::CacheRestored
    );
    assert_eq!(values(&mut all), expected);
    // The native selected loader does not retain discarded local record copies.
    let descriptor = reader.catalogue().sector(0).unwrap();
    assert_eq!(
        descriptor.receipt.native_content_id,
        sector.template_content_id()
    );
    assert!(matches!(
        reader.write_with_native_cache(Cursor::new(Vec::new()), |_| ControlFlow::Break(())),
        Err(KernelError::Cancelled)
    ));
}

mod dynamic;

#[test]
fn native_family_refresh_preserves_each_recipe_and_only_changes_transport() {
    use crate::kernel::indexed::{
        ProgramArchiveReader, ProgramArchiveWriter, ProgramRecipe, write_unit,
    };
    let recipes = [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ];
    let mut writer =
        ProgramArchiveWriter::new(Cursor::new(Vec::new()), "a".repeat(64), recipes).unwrap();
    for recipe in recipes {
        let kernels = dynamic::fixture(recipe);
        let mut pieces = Cursor::new(Vec::new());
        let receipts = write_unit(&mut pieces, &kernels, vec![0]).unwrap();
        let mut offset = 0;
        for mut receipt in receipts {
            let end = offset + receipt.length as usize;
            let piece = base(&pieces.get_ref()[offset..end]);
            receipt.length = piece.len() as u64;
            receipt.digest = blake3::hash(piece).to_hex().to_string();
            writer
                .append_record(recipe, &mut &piece[..], receipt)
                .unwrap();
            offset = end;
        }
    }
    let (bytes, old) = writer.finish().unwrap();
    let mut reader =
        ProgramArchiveReader::from_reader(bytes, KernelLoadOptions { validate: true }).unwrap();
    let mut steps = Vec::new();
    let (bytes, new) = reader
        .write_with_native_cache(Cursor::new(Vec::new()), |p| {
            steps.push(p.completed);
            ControlFlow::Continue(())
        })
        .unwrap();
    assert_eq!(
        steps,
        (0..=old.recipes.iter().map(|r| r.records.len()).sum()).collect::<Vec<usize>>()
    );
    assert_eq!(new.source_identity, old.source_identity);
    assert_eq!(new.content_id, old.content_id);
    for (a, b) in old.recipes.iter().zip(&new.recipes) {
        assert_eq!(a.recipe, b.recipe);
        assert_eq!(a.content_id, b.content_id);
        assert!(
            a.records
                .iter()
                .zip(&b.records)
                .any(|(a, b)| a.receipt.digest != b.receipt.digest)
        );
        for (a, b) in a.records.iter().zip(&b.records) {
            assert_eq!(a.receipt.native_content_id, b.receipt.native_content_id);
            assert_eq!(a.output_indices, b.output_indices);
            assert_eq!(a.receipt.source_indices, b.receipt.source_indices);
            if a.sector.is_some() {
                assert_ne!(a.receipt.length, b.receipt.length);
            }
        }
    }
    let mut restored =
        ProgramArchiveReader::from_reader(bytes, KernelLoadOptions { validate: true }).unwrap();
    for recipe in recipes {
        let mut selected = restored.select(recipe).unwrap();
        let kernel = selected.load_sector(0).unwrap();
        assert_eq!(
            kernel.sectors[0].primary_evaluator_restoration(),
            Restoration::CacheRestored
        );
    }
    assert!(matches!(
        restored.write_with_native_cache(Cursor::new(Vec::new()), |_| ControlFlow::Break(())),
        Err(KernelError::Cancelled)
    ));
}
