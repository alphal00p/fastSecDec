//! Original geometry scope is independent of compact kernel and chart IDs.
#[cfg(feature = "native")]
use fastsecdec::kernel::PrecisionPolicy;
use fastsecdec::{
    generation::{
        GeneratedIntegral, GenerationMode, GenerationOptions, GenerationSession, generate,
    },
    kernel::{CompilationSettings, EvaluatorBackend, KernelLoadOptions, KernelSet, indexed},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
    results::{ExactContributionPolicy, KernelResultManifest, ResultScope},
};
use std::{io::Cursor, ops::ControlFlow};
use symbolica::{atom::Atom, parse, symbol};

fn input(exact: bool) -> ParametricIntegrand {
    let factors = vec![PolynomialFactor::new(
        parse!("source_selection::x+source_selection::y"),
        if exact {
            parse!("-source_selection::eps")
        } else {
            Atom::num(-1)
        },
        FactorRole::Singularity,
    )];
    ParametricIntegrand::new(
        vec![
            symbol!("source_selection::x"),
            symbol!("source_selection::y"),
        ],
        symbol!("source_selection::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("2+3𝑖"),
            if exact {
                vec![parse!("-1+source_selection::eps"); 2]
            } else {
                vec![Atom::Zero; 2]
            },
            factors,
        )],
    )
    .unwrap()
}
fn options(mode: GenerationMode, selection: Option<Vec<usize>>) -> GenerationOptions {
    GenerationOptions {
        mode,
        source_sectors: selection,
        ..Default::default()
    }
}
fn generated(
    mode: GenerationMode,
    selection: Option<Vec<usize>>,
    exact: bool,
) -> GeneratedIntegral {
    let mut options = options(mode, selection);
    if exact {
        options.max_order = -2;
    }
    generate(&input(exact), &options, |_| ControlFlow::Continue(())).unwrap()
}
fn compile(generated: &GeneratedIntegral) -> KernelSet {
    generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap()
}
fn values(mut kernels: KernelSet) -> Vec<f64> {
    let mut total = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut value = vec![0.; total.len()];
        sector
            .evaluate(&vec![0.37; sector.dimension()], &mut value)
            .unwrap();
        total.iter_mut().zip(value).for_each(|(sum, x)| *sum += x);
    }
    total
}

#[test]
fn selected_symmetry_scope_and_cooperative_geometry_are_native_and_identity_bound() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let full = generated(mode, None, false);
        assert_eq!(full.metadata().charts().len(), 2);
        let all = generated(mode, Some(vec![1, 0]), false);
        assert!(all.metadata().source_scope().is_none());
        assert_eq!(
            compile(&all).to_bytes().unwrap(),
            compile(&full).to_bytes().unwrap()
        );
        let mut subsets = Vec::new();
        for original in [0, 1] {
            let selected = generated(mode, Some(vec![original]), false);
            let metadata = selected.metadata();
            let scope = metadata.source_scope().unwrap();
            assert_eq!(scope.selection().original_source_count(), 2);
            assert_eq!(scope.selection().source_sectors(), [original]);
            assert_eq!(scope.chart_source_sectors(), [original]);
            assert_eq!(metadata.charts().len(), 1);
            assert_eq!(metadata.charts()[0].source_index(), 0);
            assert_eq!(metadata.charts()[0].representative(), 0);
            assert_eq!(
                metadata.charts()[0].coordinates().images(),
                full.metadata().charts()[original].coordinates().images()
            );
            let mut stepped =
                GenerationSession::new(input(false), options(mode, Some(vec![original])));
            while !stepped.is_complete() {
                stepped.step(1, |_| ControlFlow::Continue(())).unwrap();
            }
            assert_eq!(
                compile(&stepped.take_result().unwrap()).to_bytes().unwrap(),
                compile(&selected).to_bytes().unwrap()
            );
            subsets.push(compile(&selected));
        }
        // Equivalent chart integrands still have different original-source identities.
        assert_ne!(subsets[0].content_id(), subsets[1].content_id());
        let left = values(subsets.remove(0));
        let right = values(subsets.remove(0));
        for ((left, right), full) in left.into_iter().zip(right).zip(values(compile(&full))) {
            assert!(
                (left + right - full).abs() < 1e-12,
                "selected symmetry multiplicity changed"
            );
        }
    }
}

#[test]
fn exact_only_selection_survives_raw_indexed_and_result_projection() {
    let selected = generated(GenerationMode::Symbolic, Some(vec![1]), true);
    assert!(selected.sectors().is_empty());
    let kernels = compile(&selected);
    assert_eq!(kernels.orders(), [-2, -2]);
    assert_eq!(kernels.exact_coefficients(), [2., 3.]);
    let raw = kernels.to_bytes().unwrap();
    assert!(raw.starts_with(b"FastSecDec\0binserde\x0e"));
    let restored =
        KernelSet::from_bytes_with_options(&raw, KernelLoadOptions { validate: true }).unwrap();
    assert_eq!(restored.content_id(), kernels.content_id());
    let (bytes, catalogue) = indexed::to_bytes(&restored).unwrap();
    assert_eq!(catalogue.source_selection().unwrap().source_sectors(), [1]);
    let mut reader = indexed::IndexedReader::from_reader(
        Cursor::new(bytes),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    for owner in [reader.load_all().unwrap(), reader.load_exact().unwrap()] {
        assert_eq!(owner.exact_coefficients(), [2., 3.]);
        assert_eq!(
            owner
                .generation_metadata()
                .unwrap()
                .source_scope()
                .unwrap()
                .selection()
                .source_sectors(),
            [1]
        );
        let again = KernelSet::from_bytes_with_options(
            &owner.to_bytes().unwrap(),
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        let manifest = KernelResultManifest::from_kernels(&again);
        let qualified = manifest
            .canonical_scope(&ResultScope::FullIntegral)
            .unwrap();
        assert_eq!(
            qualified,
            ResultScope::SelectedSectors {
                sector_ids: vec![],
                exact_policy: ExactContributionPolicy::IncludeAll
            }
        );
        let omitted = manifest
            .integration_problem(
                &ResultScope::SelectedSectors {
                    sector_ids: vec![],
                    exact_policy: ExactContributionPolicy::ExcludeAll,
                },
                "selected-again",
            )
            .unwrap();
        assert_eq!(omitted.exact_coefficients, [0., 0.]);
        assert!(
            format!("{}", again.generation_metadata().unwrap().display(false))
                .contains("Partial original integral")
        );
        let saved = again.to_bytes().unwrap();
        let repeated =
            KernelSet::from_bytes_with_options(&saved, KernelLoadOptions { validate: true })
                .unwrap();
        assert_eq!(repeated.to_bytes().unwrap(), saved);
        assert_eq!(repeated.content_id(), again.content_id());
    }
    // Cancellation does not erase the declaration that original chart 1 was selected.
    let original = input(true);
    let term = &original.terms()[0];
    let cancelled = ParametricIntegrand::new(
        original.parameters().to_vec(),
        original.regulator(),
        original.domain(),
        vec![
            term.clone(),
            ParametricTerm::new(
                -term.prefactor(),
                term.monomial_powers().to_vec(),
                term.factors().to_vec(),
            ),
        ],
    )
    .unwrap();
    let mut options = options(GenerationMode::Symbolic, Some(vec![1]));
    options.max_order = -2;
    let cancelled = generate(&cancelled, &options, |_| ControlFlow::Continue(())).unwrap();
    let restored = KernelSet::from_bytes(&compile(&cancelled).to_bytes().unwrap()).unwrap();
    assert!(restored.sectors().is_empty());
    assert!(
        restored
            .exact_coefficients()
            .iter()
            .all(|value| *value == 0.)
    );
    assert_eq!(
        restored
            .generation_metadata()
            .unwrap()
            .source_scope()
            .unwrap()
            .selection()
            .source_sectors(),
        [1]
    );
}

#[test]
fn source_selection_rejects_invalid_requests_before_mapping() {
    for selection in [vec![], vec![0, 0], vec![2], vec![0, 2]] {
        let error = generate(
            &input(false),
            &options(GenerationMode::Symbolic, Some(selection)),
            |_| ControlFlow::Continue(()),
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("source"));
    }
    let empty = ParametricIntegrand::new(
        vec![symbol!("source_selection::x")],
        symbol!("source_selection::eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    assert!(
        generate(
            &empty,
            &options(GenerationMode::Symbolic, Some(vec![0])),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
}

#[test]
fn indexed_lineage_cannot_be_relabelled_or_dropped_even_without_semantic_hash_checks() {
    let mut left = compile(&generated(GenerationMode::Symbolic, Some(vec![0]), false));
    let right = compile(&generated(GenerationMode::Symbolic, Some(vec![1]), false));
    let (bytes, catalogue) = indexed::to_bytes(&right).unwrap();
    assert!(left.adopt_indexed_catalogue(&catalogue).is_err());
    let mut wrong = serde_json::to_value(&catalogue).unwrap();
    for record in wrong["records"].as_array_mut().unwrap() {
        let scope = &mut record["receipt"]["source_scope"];
        scope["selection"]["source_sectors"] = serde_json::json!([0]);
        if !scope["chart_source_sectors"].as_array().unwrap().is_empty() {
            scope["chart_source_sectors"] = serde_json::json!([0]);
        }
    }
    let wrong = serde_json::from_value(wrong).unwrap();
    let mut reader = indexed::IndexedReader::new(
        Cursor::new(&bytes),
        wrong,
        KernelLoadOptions { validate: false },
    )
    .unwrap();
    assert!(reader.load_sector(0).is_err());
    let mut omitted = catalogue.clone();
    omitted.records[0].receipt.source_scope = None;
    assert!(
        indexed::IndexedReader::new(
            Cursor::new(&bytes),
            omitted,
            KernelLoadOptions { validate: false }
        )
        .is_err()
    );
}

#[test]
fn subset_contour_definitions_and_recipe_directory_keep_identical_original_scope() {
    use fastsecdec::{
        contour::ContourJacobian,
        kernel::indexed::{ProgramArchiveReader, ProgramArchiveWriter, ProgramRecipe},
        parametric::FactorSemantics,
    };
    let original = input(false);
    let term = &original.terms()[0];
    let contour_input = ParametricIntegrand::new(
        original.parameters().to_vec(),
        original.regulator(),
        original.domain(),
        vec![ParametricTerm::new(
            term.prefactor().clone(),
            term.monomial_powers().to_vec(),
            term.factors()
                .iter()
                .cloned()
                .map(|factor| factor.with_semantics(FactorSemantics::Causal))
                .collect(),
        )],
    )
    .unwrap();
    let recipes = [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1];
    for mixed in [false, true] {
        let mut writer =
            ProgramArchiveWriter::new(Cursor::new(vec![]), "e".repeat(64), recipes).unwrap();
        for (index, recipe) in recipes.into_iter().enumerate() {
            let options = GenerationOptions {
                source_sectors: Some(vec![if mixed { index } else { 1 }]),
                program_recipe: recipe,
                contour_jacobian: if recipe == ProgramRecipe::FixedV1 {
                    ContourJacobian::Dual
                } else {
                    ContourJacobian::Symbolic
                },
                ..Default::default()
            };
            let generated =
                generate(&contour_input, &options, |_| ControlFlow::Continue(())).unwrap();
            let kernels = compile(&generated);
            let restored = KernelSet::from_bytes_with_options(
                &kernels.to_bytes().unwrap(),
                KernelLoadOptions { validate: true },
            )
            .unwrap();
            assert_eq!(
                restored.generation_metadata().unwrap().source_scope(),
                generated.metadata().source_scope()
            );
            writer.append_kernels(recipe, &restored).unwrap();
        }
        if mixed {
            assert!(writer.finish().is_err());
        } else {
            let (bytes, catalogue) = writer.finish().unwrap();
            let mut reader =
                ProgramArchiveReader::from_reader(bytes, KernelLoadOptions { validate: true })
                    .unwrap();
            for recipe in recipes {
                assert_eq!(
                    catalogue
                        .recipe(recipe)
                        .unwrap()
                        .source_selection()
                        .unwrap()
                        .source_sectors(),
                    [1]
                );
                let loaded = reader.select(recipe).unwrap().load_all().unwrap();
                assert_eq!(
                    loaded
                        .generation_metadata()
                        .unwrap()
                        .source_scope()
                        .unwrap()
                        .chart_source_sectors(),
                    [1]
                );
                assert_eq!(loaded.program_recipe(), recipe);
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn subset_scope_survives_jit_cache_refresh_and_selected_resident_roundtrip() {
    let generated = generated(GenerationMode::Symbolic, Some(vec![1]), false);
    let bytes = generated
        .to_kernel_bytes_with_settings(
            PrecisionPolicy::default(),
            CompilationSettings {
                backend: EvaluatorBackend::Symjit,
                ..Default::default()
            },
        )
        .unwrap();
    let kernels =
        KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: true }).unwrap();
    let (indexed_bytes, catalogue) = indexed::to_bytes(&kernels).unwrap();
    let mut reader = indexed::IndexedReader::from_reader(
        Cursor::new(indexed_bytes),
        KernelLoadOptions { validate: true },
    )
    .unwrap();
    let (refreshed, cached_catalogue) = reader
        .write_with_native_cache(Cursor::new(vec![]), |_| ControlFlow::Continue(()))
        .unwrap();
    assert_eq!(catalogue.content_id, cached_catalogue.content_id);
    assert_eq!(
        catalogue.source_selection(),
        cached_catalogue.source_selection()
    );
    let mut reader =
        indexed::IndexedReader::from_reader(refreshed, KernelLoadOptions { validate: true })
            .unwrap();
    let local = reader.load_sector(0).unwrap();
    let raw = local.to_bytes().unwrap();
    assert!(raw.starts_with(b"FastSecDec\0binserde\x0d"));
    let restored =
        KernelSet::from_bytes_with_options(&raw, KernelLoadOptions { validate: true }).unwrap();
    assert_eq!(
        restored.generation_metadata().unwrap().source_scope(),
        local.generation_metadata().unwrap().source_scope()
    );
    assert_eq!(restored.content_id(), local.content_id());
    assert_eq!(values(restored), values(local));
}
