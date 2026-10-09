use super::*;

#[test]
fn durable_sector_without_receipt_is_regenerated_after_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.toml");
    let output = directory.path().join("integral.fsd");
    fs::write(&input, "[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1'\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    let mut journal = Journal::open(&input, &output, false, "first").unwrap();
    let prepare = Request::Prepare {
        input: input.clone(),
        workers: 1,
        overrides: Default::default(),
    };
    journal.request("prepare", prepare.clone()).unwrap();
    jobs::execute(&journal.job_path("prepare"), &mut |_| Ok(())).unwrap();
    let Response::Prepared(prepared) = journal.accept("prepare", &prepare).unwrap() else {
        panic!()
    };
    let discover = Request::Discover {
        preparation: journal.response_path("prepare"),
        program_recipe: prepared.native.program_recipe,
        source_id: prepared.native.source.blake3.clone(),
        dimension: prepared.native.dimension,
        index: 0,
    };
    journal.request("discover-0", discover.clone()).unwrap();
    jobs::execute(&journal.job_path("discover-0"), &mut |_| Ok(())).unwrap();
    let Response::Discovered(chart) = journal.accept("discover-0", &discover).unwrap() else {
        panic!()
    };
    // The same sector number in another recipe or source is different work.
    for change_recipe in [false, true] {
        let mut foreign = chart.clone();
        if change_recipe {
            foreign.program_recipe = fastsecdec::kernel::indexed::ProgramRecipe::FixedV1;
        } else {
            foreign.source_id = "a different source".into();
        }
        assert!(
            journal::validate(&discover, &Response::Discovered(foreign), &journal.root).is_err()
        );
    }
    let assignment = native::SymmetryAssignment {
        program_recipe: chart.program_recipe,
        source_id: chart.source_id.clone(),
        source: 0,
        representative: 0,
        permutation: (0..chart.dimension).collect(),
    };
    let symmetry = Request::Symmetry {
        preparation: journal.response_path("prepare"),
        chart: chart.clone(),
        candidates: vec![],
    };
    journal::validate(
        &symmetry,
        &Response::Symmetry(assignment.clone()),
        &journal.root,
    )
    .unwrap();
    let mut repeated = assignment.clone();
    repeated.permutation.push(0);
    assert!(journal::validate(&symmetry, &Response::Symmetry(repeated), &journal.root).is_err());
    for change_recipe in [false, true] {
        let mut foreign = assignment.clone();
        if change_recipe {
            foreign.program_recipe = fastsecdec::kernel::indexed::ProgramRecipe::FixedV1;
        } else {
            foreign.source_id = "a different source".into();
        }
        assert!(journal::validate(&symmetry, &Response::Symmetry(foreign), &journal.root).is_err());
    }
    let prepared =
        native::finish_preparation(&prepared.native, vec![chart], vec![assignment], vec![])
            .unwrap();
    let data = journal.root.join("sector-0.dat");
    let request = Request::Sector {
        job: prepared.sectors[0].clone(),
        max_order: 0,
        evaluator: fastsecdec::kernel::CompilationSettings {
            backend: fastsecdec::kernel::EvaluatorBackend::Eager,
            ..Default::default()
        },
        output: data.clone(),
    };
    journal.request("sector-0", request.clone()).unwrap();
    jobs::execute(&journal.job_path("sector-0"), &mut |_| Ok(())).unwrap();
    assert!(fs::metadata(&data).unwrap().len() > 0);
    // Inject precisely the filesystem state after the durable data rename but
    // before publishing a receipt. No completion has reached the coordinator.
    fs::remove_file(journal.response_path("sector-0")).unwrap();
    let partial = data.with_extension("writing");
    fs::write(&partial, b"an interrupted earlier write").unwrap();
    drop(journal); // the old caller/worker is no longer executing
    let mut recovered = Journal::open(&input, &output, true, "second").unwrap();
    let (_, accepted) = recovered.request("sector-0", request.clone()).unwrap();
    assert!(
        accepted.is_none(),
        "data alone must never count as completed work"
    );
    jobs::execute(&recovered.job_path("sector-0"), &mut |_| Ok(())).unwrap();
    let Response::Compiled(compiled) = recovered.accept("sector-0", &request).unwrap() else {
        panic!()
    };
    for corruption in 0..3 {
        let mut foreign = compiled.clone();
        if corruption == 2 {
            let mode = foreign.source_chart_modes.remove(&0).unwrap();
            foreign.source_chart_modes.insert(1, mode);
        } else {
            let indices = &mut foreign
                .receipts
                .iter_mut()
                .find(|receipt| !receipt.source_indices.is_empty())
                .unwrap()
                .source_indices;
            if corruption == 0 {
                indices[0] = 1;
            } else {
                indices.push(indices[0]);
            }
        }
        assert!(
            journal::validate(&request, &Response::Compiled(foreign), &recovered.root).is_err()
        );
    }
    for change_recipe in [false, true] {
        let mut foreign = compiled.clone();
        if change_recipe {
            foreign.program_recipe = fastsecdec::kernel::indexed::ProgramRecipe::FixedV1;
        } else {
            foreign.source_id = "a different source".into();
        }
        assert!(
            journal::validate(&request, &Response::Compiled(foreign), &recovered.root).is_err()
        );
    }
    // A forged matching outer receipt cannot relabel the actual native program.
    let mut foreign_request = request.clone();
    let Request::Sector { job, .. } = &mut foreign_request else {
        panic!()
    };
    job.program_recipe = fastsecdec::kernel::indexed::ProgramRecipe::FixedV1;
    let mut foreign = compiled.clone();
    foreign.program_recipe = job.program_recipe;
    assert!(
        journal::validate(
            &foreign_request,
            &Response::Compiled(foreign),
            &recovered.root
        )
        .is_err()
    );
    assert!(!partial.exists());
    let mut writer = IndexedWriter::new(std::io::Cursor::new(Vec::new())).unwrap();
    let mut sector = File::open(compiled.data).unwrap();
    for receipt in compiled.receipts {
        writer.append_record(&mut sector, receipt).unwrap();
    }
    let (bytes, catalogue) = writer.finish().unwrap();
    assert_eq!(catalogue.sector_count(), 1);
    let mut reader = fastsecdec::kernel::indexed::IndexedReader::from_reader(
        bytes,
        fastsecdec::kernel::KernelLoadOptions { validate: true },
    )
    .unwrap();
    let mut kernels = reader.load_sector(0).unwrap();
    let mut value = [0.];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut value)
        .unwrap();
    assert!((value[0] - 2. / 3.).abs() < 1e-14);
}
