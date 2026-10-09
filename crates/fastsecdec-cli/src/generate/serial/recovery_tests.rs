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
        index: 0,
    };
    journal.request("discover-0", discover.clone()).unwrap();
    jobs::execute(&journal.job_path("discover-0"), &mut |_| Ok(())).unwrap();
    let Response::Discovered(chart) = journal.accept("discover-0", &discover).unwrap() else {
        panic!()
    };
    let assignment = native::SymmetryAssignment {
        source: 0,
        representative: 0,
        permutation: (0..chart.dimension).collect(),
    };
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
