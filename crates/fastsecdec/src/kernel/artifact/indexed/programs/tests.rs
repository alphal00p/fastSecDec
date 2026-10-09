use super::*;
use crate::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelLoadOptions, KernelSet},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Seek, SeekFrom},
    ops::ControlFlow,
};
use symbolica::{atom::Atom, parse, symbol};

fn kernel(contour: bool, complex: bool, order: i32) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("archive_recipes::x")],
        symbol!("archive_recipes::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                parse!("2+3𝑖")
            } else {
                Atom::one()
            },
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("2+archive_recipes::p*archive_recipes::x"),
                    parse!("-1+archive_recipes::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            program_recipe: if contour {
                ProgramRecipe::FixedV1
            } else {
                ProgramRecipe::UndeformedV1
            },
            max_order: order,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings_parameters_and_progress(
        Default::default(),
        &[symbol!("archive_recipes::p")],
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
}

fn append(
    writer: &mut ProgramArchiveWriter<Cursor<Vec<u8>>>,
    recipe: ProgramRecipe,
    kernels: &KernelSet,
    source: usize,
) {
    let mut bytes = Cursor::new(Vec::new());
    let receipts = super::super::write_unit(&mut bytes, kernels, vec![source]).unwrap();
    let mut remaining = bytes.get_ref().as_slice();
    for receipt in receipts {
        writer
            .append_record(recipe, &mut remaining, receipt)
            .unwrap();
    }
    assert!(remaining.is_empty());
}

#[test]
fn v10_fixed_record_restores_explicit_recipe_and_pilot_associations() {
    let mut fixed = kernel(true, true, 1);
    fixed
        .declare_program_recipe(ProgramRecipe::FixedV1)
        .unwrap();
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        blake3::hash(b"v10-fixed").to_hex().to_string(),
        [ProgramRecipe::FixedV1],
    )
    .unwrap();
    append(&mut writer, ProgramRecipe::FixedV1, &fixed, 0);
    let (writer, catalogue) = writer.finish().unwrap();
    assert!(catalogue.recipes[0].records.iter().all(|record| {
        record.receipt.version == 2 && record.receipt.recipe == Some(ProgramRecipe::FixedV1)
    }));
    let mut reader = ProgramArchiveReader::from_reader(
        Cursor::new(writer.into_inner()),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let mut restored = reader
        .select(ProgramRecipe::FixedV1)
        .unwrap()
        .load_all()
        .unwrap();
    assert_eq!(
        restored.program_descriptor().unwrap().recipe(),
        ProgramRecipe::FixedV1
    );
    let settings = ContourSettings {
        deformation: ContourMode::Fixed { lambda: 0.1 },
        validation: ContourValidationOptions {
            policy: ContourValidation::Pilot,
            pilot_points: 1,
        },
    };
    restored
        .bind_parameters_with_contour(
            &BTreeMap::from([(symbol!("archive_recipes::p"), 1.)]),
            &settings,
        )
        .unwrap();
    let charts = restored.contour_validation_charts();
    for chart in charts {
        restored
            .validate_contour_point(chart.chart_index, &vec![0.4; chart.dimension], true)
            .unwrap();
    }
    restored.finish_contour_pilot().unwrap();
    let portable = restored.to_bytes().unwrap();
    let again = KernelSet::from_bytes_with_options(&portable, KernelLoadOptions { validate: true })
        .unwrap();
    assert_eq!(again.program_recipe(), ProgramRecipe::FixedV1);
    assert!(again.program_descriptor().is_some());
}

#[test]
fn dynamic_directory_cannot_infer_a_recipe_from_runtime_names() {
    let mut fixed = kernel(true, false, 0);
    fixed
        .declare_program_recipe(ProgramRecipe::FixedV1)
        .unwrap();
    let mut records = Cursor::new(Vec::new());
    let mut receipts = super::super::write_unit(&mut records, &fixed, vec![0]).unwrap();
    let receipt = receipts.remove(0);
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        blake3::hash(b"v10-mismatch").to_hex().to_string(),
        [ProgramRecipe::DynamicPolynomialV1],
    )
    .unwrap();
    assert!(
        writer
            .append_record(
                ProgramRecipe::DynamicPolynomialV1,
                &mut records.get_ref().as_slice(),
                receipt
            )
            .is_err()
    );
}

fn archive(reverse: bool) -> (Vec<u8>, ProgramArchiveCatalogue) {
    let plain = kernel(false, false, 0);
    let fixed = kernel(true, true, 1);
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        blake3::hash(b"source-fixture").to_hex().to_string(),
        [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1],
    )
    .unwrap();
    if reverse {
        append(&mut writer, ProgramRecipe::FixedV1, &fixed, 1);
        append(&mut writer, ProgramRecipe::UndeformedV1, &plain, 0);
        append(&mut writer, ProgramRecipe::FixedV1, &fixed, 0);
    } else {
        append(&mut writer, ProgramRecipe::FixedV1, &fixed, 0);
        append(&mut writer, ProgramRecipe::UndeformedV1, &plain, 0);
        append(&mut writer, ProgramRecipe::FixedV1, &fixed, 1);
    }
    let (writer, catalogue) = writer.finish().unwrap();
    (writer.into_inner(), catalogue)
}

#[test]
fn independent_recipe_layouts_parameters_and_completion_order_are_preserved() {
    let (bytes, catalogue) = archive(false);
    assert_eq!(catalogue.version, 2);
    assert_eq!(catalogue.content_id, archive(true).1.content_id);
    let plain = catalogue.recipe(ProgramRecipe::UndeformedV1).unwrap();
    let fixed = catalogue.recipe(ProgramRecipe::FixedV1).unwrap();
    assert_eq!(plain.orders, [0]);
    assert_eq!(fixed.orders, [0, 0, 1, 1]);
    assert_eq!(plain.sector_count(), 1);
    assert_eq!(fixed.sector_count(), 2);
    assert_eq!(plain.physics_parameters, fixed.physics_parameters);
    assert_eq!(plain.runtime_parameters().len(), 1);
    assert_eq!(fixed.runtime_parameters().len(), 2);
    assert_ne!(plain.content_id, fixed.content_id);
    let mut reader =
        ProgramArchiveReader::from_reader(Cursor::new(bytes), KernelLoadOptions { validate: true })
            .unwrap();
    let mut plain_loaded = reader
        .select(ProgramRecipe::UndeformedV1)
        .unwrap()
        .load_all()
        .unwrap();
    plain_loaded
        .bind_parameters(&BTreeMap::from([(symbol!("archive_recipes::p"), 1.)]))
        .unwrap();
    let mut output = [0.];
    plain_loaded.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert!((output[0] - 1. / 2.25).abs() < 1e-14);
    let mut fixed_loaded = reader
        .select(ProgramRecipe::FixedV1)
        .unwrap()
        .load_all()
        .unwrap();
    fixed_loaded
        .bind_parameters_with_contour(
            &BTreeMap::from([(symbol!("archive_recipes::p"), 1.)]),
            &ContourSettings {
                deformation: ContourMode::Fixed { lambda: 0.1 },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Pilot,
                    pilot_points: 1,
                },
            },
        )
        .unwrap();
    let charts = fixed_loaded.contour_validation_charts();
    assert_eq!(
        charts
            .iter()
            .map(|chart| chart.chart_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    for chart in &charts {
        fixed_loaded
            .validate_contour_point(chart.chart_index, &[0.25], true)
            .unwrap();
    }
    assert!(fixed_loaded.finish_contour_pilot().unwrap().pilot_complete);
    let mut a = [0.; 4];
    let mut b = [0.; 4];
    fixed_loaded.sectors_mut()[0]
        .evaluate(&[0.25], &mut a)
        .unwrap();
    fixed_loaded.sectors_mut()[1]
        .evaluate(&[0.25], &mut b)
        .unwrap();
    assert_eq!(a, b);
}

struct ObservedReader {
    bytes: Cursor<Vec<u8>>,
    reads: Vec<(u64, usize)>,
}
impl Read for ObservedReader {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        let start = self.bytes.position();
        let count = self.bytes.read(output)?;
        self.reads.push((start, count));
        Ok(count)
    }
}
impl Seek for ObservedReader {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.bytes.seek(position)
    }
}

#[test]
fn selection_never_reads_or_decodes_other_recipe_payloads() {
    let (mut bytes, catalogue) = archive(false);
    let fixed = catalogue.recipe(ProgramRecipe::FixedV1).unwrap();
    // If any unselected record were decoded, its missing native prefix would
    // fail before backend restoration. Directory admission must not read it.
    for record in &fixed.records {
        bytes[record.offset as usize] ^= 0xff;
    }
    let observed = ObservedReader {
        bytes: Cursor::new(bytes),
        reads: Vec::new(),
    };
    let mut reader =
        ProgramArchiveReader::from_reader(observed, KernelLoadOptions { validate: true }).unwrap();
    reader
        .select(ProgramRecipe::UndeformedV1)
        .unwrap()
        .load_all()
        .unwrap();
    let observed = reader.into_inner();
    for record in &fixed.records {
        assert!(
            observed
                .reads
                .iter()
                .all(|(offset, count)| *offset + *count as u64 <= record.offset
                    || *offset >= record.offset + record.receipt.length)
        );
    }
    let mut reader =
        ProgramArchiveReader::from_reader(observed, KernelLoadOptions { validate: true }).unwrap();
    assert!(
        reader
            .select(ProgramRecipe::FixedV1)
            .unwrap()
            .load_sector(0)
            .err()
            .unwrap()
            .to_string()
            .contains("digest")
    );
}

fn rewrite(bytes: &[u8], catalogue: &ProgramArchiveCatalogue) -> Vec<u8> {
    let mut output = bytes[..catalogue.records_end as usize].to_vec();
    super::super::transport::write_footer(&mut output, catalogue, super::FOOTER).unwrap();
    output
}

#[test]
fn global_ranges_and_recipe_schema_are_checked_even_without_payload_hashes() {
    let (bytes, catalogue) = archive(false);
    let options = KernelLoadOptions { validate: false };
    for mutation in 0..6 {
        let mut bad = catalogue.clone();
        match mutation {
            0 => bad.recipes[1].records[0].offset = bad.recipes[0].records[0].offset,
            1 => bad.recipes[1].records[0].offset = u64::MAX,
            2 => {
                bad.recipes[1].records.pop();
            }
            3 => bad.recipes[1].recipe_parameters.clear(),
            4 => bad.recipes.push(bad.recipes[0].clone()),
            _ => {
                let record = bad.recipes[1]
                    .records
                    .iter_mut()
                    .find(|record| record.sector == Some(1))
                    .unwrap();
                record.receipt.source_indices[0] = 7;
            }
        }
        assert!(
            ProgramArchiveReader::from_reader(Cursor::new(rewrite(&bytes, &bad)), options).is_err(),
            "mutation {mutation}"
        );
    }
    let mut wrong_id = catalogue.clone();
    wrong_id.recipes[1].content_id = blake3::hash(b"wrong").to_hex().to_string();
    assert!(
        ProgramArchiveReader::from_reader(
            Cursor::new(rewrite(&bytes, &wrong_id)),
            KernelLoadOptions { validate: true }
        )
        .is_err()
    );
    let unknown = String::from_utf8(serde_json::to_vec(&catalogue).unwrap())
        .unwrap()
        .replace("fixed-v1", "fixed-v999");
    assert!(serde_json::from_str::<ProgramArchiveCatalogue>(&unknown).is_err());
    assert!(
        ProgramArchiveReader::from_reader(Cursor::new(&bytes[..bytes.len() - 1]), options).is_err()
    );
}

#[test]
fn incomplete_recipe_sets_and_failed_record_writes_cannot_be_published() {
    let source = blake3::hash(b"source-fixture").to_hex().to_string();
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        source.clone(),
        [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1],
    )
    .unwrap();
    append(
        &mut writer,
        ProgramRecipe::UndeformedV1,
        &kernel(false, false, 0),
        0,
    );
    assert!(writer.finish().is_err());
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        source,
        [ProgramRecipe::UndeformedV1],
    )
    .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    let receipts = super::super::write_unit(&mut bytes, &kernel(false, false, 0), vec![0]).unwrap();
    assert!(
        writer
            .append_record(
                ProgramRecipe::UndeformedV1,
                &mut &bytes.get_ref()[..1],
                receipts[0].clone()
            )
            .is_err()
    );
    assert!(writer.finish().is_err());
}

#[test]
fn local_metadata_indices_fail_admission_without_panicking() {
    let (bytes, catalogue) = archive(false);
    let descriptor = catalogue
        .recipe(ProgramRecipe::FixedV1)
        .unwrap()
        .sector(0)
        .unwrap();
    let mut reader = ProgramArchiveReader::from_reader(
        Cursor::new(bytes),
        KernelLoadOptions { validate: false },
    )
    .unwrap();
    let mut kernel = reader
        .select(ProgramRecipe::FixedV1)
        .unwrap()
        .load_sector(0)
        .unwrap();
    super::super::record_reader::check_record(descriptor, &kernel).unwrap();
    kernel.metadata.as_mut().unwrap().charts[0].representative = usize::MAX;
    assert!(super::super::record_reader::check_record(descriptor, &kernel).is_err());
    kernel.metadata.as_mut().unwrap().charts[0].representative = 0;
    kernel.contour_checks[0].chart_index = usize::MAX;
    assert!(super::super::record_reader::check_record(descriptor, &kernel).is_err());
    kernel.contour_checks[0].chart_index = 0;
    kernel.metadata.as_mut().unwrap().charts[0].source_index = usize::MAX;
    assert!(super::super::record_reader::check_record(descriptor, &kernel).is_err());
}

#[test]
fn selected_portable_roundtrip_retains_recipe_identity_and_pilot_mapping() {
    let (bytes, catalogue) = archive(false);
    assert!(
        KernelSet::from_bytes(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("explicit")
    );
    let mut reader = ProgramArchiveReader::from_reader(
        Cursor::new(bytes.clone()),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    for recipe in [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1] {
        let selected = reader.select(recipe).unwrap().load_all().unwrap();
        let saved = selected.to_bytes().unwrap();
        assert!(saved.len() < bytes.len());
        let saved_reader = ProgramArchiveReader::from_reader(
            Cursor::new(&saved),
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(
            saved_reader.catalogue().source_identity,
            catalogue.source_identity
        );
        assert_eq!(saved_reader.catalogue().recipes.len(), 1);
        assert_eq!(saved_reader.catalogue().recipes[0].recipe, recipe);
        assert_eq!(
            saved_reader.catalogue().recipes[0].content_id,
            catalogue.recipe(recipe).unwrap().content_id
        );
        let mut reloaded =
            KernelSet::from_bytes_with_options(&saved, KernelLoadOptions { validate: true })
                .unwrap();
        assert_eq!(
            selected.template_content_id(),
            reloaded.template_content_id()
        );
        let physics = BTreeMap::from([(symbol!("archive_recipes::p"), 1.)]);
        if recipe == ProgramRecipe::FixedV1 {
            reloaded
                .bind_parameters_with_contour(
                    &physics,
                    &ContourSettings {
                        deformation: ContourMode::Fixed { lambda: 0.1 },
                        validation: ContourValidationOptions {
                            policy: ContourValidation::Pilot,
                            pilot_points: 1,
                        },
                    },
                )
                .unwrap();
            let charts = reloaded.contour_validation_charts();
            assert_eq!(charts.len(), 2);
            for chart in charts {
                reloaded
                    .validate_contour_point(chart.chart_index, &[0.25], true)
                    .unwrap();
            }
            assert!(reloaded.finish_contour_pilot().unwrap().pilot_complete);
        } else {
            reloaded.bind_parameters(&physics).unwrap();
        }
        let mut output = vec![0.; reloaded.orders().len()];
        reloaded.sectors_mut()[0]
            .evaluate(&[0.25], &mut output)
            .unwrap();
        assert!(output.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn legacy_indexed_v1_is_a_single_recipe_without_changing_its_identity() {
    for contour in [false, true] {
        let kernels = kernel(contour, false, 0);
        let (bytes, catalogue) = super::super::to_bytes(&kernels).unwrap();
        let mut reader = ProgramArchiveReader::from_reader(
            Cursor::new(bytes),
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(reader.catalogue().version, 1);
        assert!(reader.catalogue().source_identity.is_none());
        assert_eq!(reader.catalogue().content_id, catalogue.content_id);
        let recipe = if contour {
            ProgramRecipe::FixedV1
        } else {
            ProgramRecipe::UndeformedV1
        };
        let loaded = reader.select(recipe).unwrap().load_all().unwrap();
        assert_eq!(loaded.template_content_id(), catalogue.content_id);
        assert_eq!(
            loaded.sectors()[0].program_bytes,
            kernels.sectors()[0].program_bytes
        );
        let absent = if contour {
            ProgramRecipe::UndeformedV1
        } else {
            ProgramRecipe::FixedV1
        };
        assert!(
            reader
                .select(absent)
                .err()
                .unwrap()
                .to_string()
                .contains("regenerate")
        );
    }
}

#[test]
fn exact_only_recipes_keep_their_own_offsets_and_validation_records() {
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        blake3::hash(b"exact-source").to_hex().to_string(),
        [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1],
    )
    .unwrap();
    for contour in [false, true] {
        let input = ParametricIntegrand::new(
            Vec::new(),
            symbol!("archive_exact::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                if contour {
                    parse!("2+3𝑖")
                } else {
                    Atom::one()
                },
                Vec::new(),
                vec![
                    PolynomialFactor::new(
                        parse!("2+archive_exact::p"),
                        parse!("-1+archive_exact::eps"),
                        FactorRole::Singularity,
                    )
                    .with_semantics(FactorSemantics::Causal),
                ],
            )],
        )
        .unwrap();
        let kernels = generate(
            &input,
            &GenerationOptions {
                program_recipe: if contour {
                    ProgramRecipe::FixedV1
                } else {
                    ProgramRecipe::UndeformedV1
                },
                max_order: i32::from(contour),
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[symbol!("archive_exact::p")],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert!(kernels.sectors().is_empty());
        append(
            &mut writer,
            if contour {
                ProgramRecipe::FixedV1
            } else {
                ProgramRecipe::UndeformedV1
            },
            &kernels,
            0,
        );
    }
    let (bytes, _) = writer.finish().unwrap();
    let mut reader =
        ProgramArchiveReader::from_reader(bytes, KernelLoadOptions { validate: true }).unwrap();
    let physics = BTreeMap::from([(symbol!("archive_exact::p"), 1.)]);
    let mut plain = reader
        .select(ProgramRecipe::UndeformedV1)
        .unwrap()
        .load_exact()
        .unwrap();
    plain.bind_parameters(&physics).unwrap();
    assert!((plain.exact_coefficients()[0] - 1. / 3.).abs() < 1e-14);
    let mut selected = reader.select(ProgramRecipe::FixedV1).unwrap();
    let mut record = selected.load_record(0).unwrap();
    let mut settings = ContourSettings {
        deformation: ContourMode::Fixed { lambda: 0.1 },
        validation: ContourValidationOptions {
            policy: ContourValidation::Pilot,
            pilot_points: 1,
        },
    };
    record
        .bind_parameters_with_contour(&physics, &settings)
        .unwrap();
    let chart = record.contour_validation_charts()[0].clone();
    assert_eq!(chart.dimension, 0);
    record
        .validate_contour_point(chart.chart_index, &[], true)
        .unwrap();
    assert!(record.finish_contour_pilot().unwrap().pilot_complete);
    let mut exact = selected.load_exact().unwrap();
    assert!(exact.generation_metadata().is_none());
    settings.validation.policy = ContourValidation::Off;
    exact
        .bind_parameters_with_contour(&physics, &settings)
        .unwrap();
    assert!((exact.exact_coefficients()[0] - 2. / 3.).abs() < 1e-14);
    assert!((exact.exact_coefficients()[1] - 1.).abs() < 1e-14);
    assert_eq!(exact.exact_coefficients(), record.exact_coefficients());
}
