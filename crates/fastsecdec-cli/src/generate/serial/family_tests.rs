//! Actual child-process family generation and both native restoration paths.
use super::*;
use fastsecdec::{
    contour::{ContourMode, ContourSettings, ContourValidation},
    kernel::{KernelLoadOptions, KernelSet},
};
use std::{ops::ControlFlow, sync::atomic::Ordering, time::Duration};

#[test]
#[ignore = "native process fixture; requires an issued socket lease"]
fn native_child_entry() {
    if std::env::var_os("FASTSECDEC_WORKER_CONTROL").is_none() {
        return;
    }
    let run = std::env::var("FASTSECDEC_GENERATION_TEST_RUN").unwrap();
    let lease = std::env::var("FASTSECDEC_GENERATION_TEST_LEASE")
        .unwrap()
        .parse()
        .unwrap();
    crate::process::child::serve(run, lease, |path: std::path::PathBuf, emit| {
        jobs::execute(&path, emit).map_err(|e| std::io::Error::other(e.to_string()))
    })
    .unwrap();
}

fn card(path: &Path, mode: &str) {
    fs::write(path, format!("[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nprefactor='2+3𝑖'\nmonomial_powers=['-1+eps']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1'\nsemantics='causal'\n[generation]\norder=0\nmode='{mode}'\n[generation.evaluator]\nbackend='eager'\n")).unwrap();
}
fn execute(
    path: &Path,
    output: &Path,
    recipes: &[ProgramRecipe],
    default: ProgramRecipe,
    workers: usize,
    resume: bool,
    dashboard: &mut Dashboard,
) -> CliResult<Artifact> {
    generate_family(
        GenerationRun {
            path,
            output,
            reference: None,
            workers,
            resume,
            overrides: Default::default(),
        },
        recipes,
        default,
        dashboard,
    )
}
fn bind(kernels: &mut KernelSet, recipe: ProgramRecipe) {
    if recipe == ProgramRecipe::FixedV1 {
        let mut contour = ContourSettings {
            deformation: ContourMode::Fixed { lambda: 0.2 },
            ..Default::default()
        };
        contour.validation.policy = ContourValidation::Off;
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &contour)
            .unwrap();
    } else {
        kernels.bind_parameters(&BTreeMap::new()).unwrap();
    }
}
fn integrate(kernels: &mut KernelSet) -> Vec<f64> {
    let mut total = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut contribution = vec![0.; total.len()];
        for point in 0..512 {
            sector
                .evaluate(&[(point as f64 + 0.5) / 512.], &mut contribution)
                .unwrap();
            for (sum, value) in total.iter_mut().zip(&contribution) {
                *sum += value / 512.;
            }
        }
    }
    total
}

#[test]
fn family_processes_publish_one_universal_archive_with_independent_recipe_layouts() {
    for mode in ["symbolic", "numerical_dual"] {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.toml");
        let output = directory.path().join("family.fsd");
        card(&input, mode);
        let recipes = [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1];
        let artifact = execute(
            &input,
            &output,
            &recipes,
            recipes[0],
            2,
            false,
            &mut Dashboard::new(false, false).unwrap(),
        )
        .unwrap();
        assert!(artifact.indexed.is_none());
        let stored = artifact.programs.as_ref().unwrap();
        assert_eq!(stored.catalogue.recipes.len(), 2);
        assert_eq!(stored.generation.len(), 2);
        for recipe in recipes {
            let (_, mut ordinary) = Artifact::load_recipe_observed(
                &output,
                KernelLoadOptions { validate: true },
                Some(recipe),
                |_| Ok(()),
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            bind(&mut ordinary, recipe);
            let values = integrate(&mut ordinary);
            for ((order, component), actual) in ordinary
                .orders()
                .iter()
                .zip(ordinary.components())
                .zip(&values)
            {
                let scale = match component {
                    fastsecdec::status::CoefficientComponent::Real => 2.,
                    fastsecdec::status::CoefficientComponent::Imag => 3.,
                };
                let expected = if *order == -1 {
                    scale
                } else {
                    assert_eq!(*order, 0);
                    -scale * 2f64.ln()
                };
                assert!(
                    (actual - expected).abs() < 2e-5,
                    "{mode} {recipe:?} eps^{order}: {actual} vs {expected}"
                );
            }
            let mut reader = artifact
                .open_program_archive(&output, KernelLoadOptions { validate: true })
                .unwrap();
            let mut selected = reader.select(recipe).unwrap();
            let catalogue = selected.catalogue().clone();
            let mut selective = vec![0.; catalogue.orders.len()];
            for (index, record) in catalogue.records.iter().enumerate() {
                let mut local = selected.load_record(index).unwrap();
                bind(&mut local, recipe);
                for (index, value) in record.output_indices.iter().zip(integrate(&mut local)) {
                    selective[*index] += value;
                }
            }
            assert_eq!(values.len(), selective.len());
            for (left, right) in values.iter().zip(&selective) {
                assert!((left - right).abs() < 2e-12);
            }
        }
        // The requested order is immaterial, but changing the default is not.
        let manifest = fs::read(output.with_extension("fsd.json")).unwrap();
        let resumed = execute(
            &input,
            &output,
            &[recipes[1], recipes[0]],
            recipes[0],
            1,
            true,
            &mut Dashboard::new(false, false).unwrap(),
        )
        .unwrap();
        assert_eq!(artifact.kernel_content_id, resumed.kernel_content_id);
        assert_eq!(
            fs::read(output.with_extension("fsd.json")).unwrap(),
            manifest
        );
        assert!(
            execute(
                &input,
                &output,
                &recipes,
                recipes[1],
                1,
                true,
                &mut Dashboard::new(false, false).unwrap()
            )
            .err()
            .unwrap()
            .to_string()
            .contains("family or default changed")
        );
    }
}

#[test]
fn cancelled_family_preserves_previous_publication_and_resumes_durable_sources() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.toml");
    let output = directory.path().join("family.fsd");
    card(&input, "numerical_dual");
    let recipes = [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1];
    execute(
        &input,
        &output,
        &recipes,
        recipes[0],
        1,
        false,
        &mut Dashboard::new(false, false).unwrap(),
    )
    .unwrap();
    let manifest = fs::read(output.with_extension("fsd.json")).unwrap();
    let mut dashboard = Dashboard::new(false, false).unwrap();
    let cancel = dashboard.cancellation_handle();
    let journal = output
        .with_file_name("family.fsd.generation")
        .join("journal.json");
    let observer = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if let Ok(bytes) = fs::read(&journal)
                && let Ok(state) = serde_json::from_slice::<serde_json::Value>(&bytes)
                && state["completed"] == false
                && let Some(run) = state["run_directory"].as_str()
            {
                let receipt = journal
                    .parent()
                    .unwrap()
                    .join(run)
                    .join("receipts/source-0.json");
                if let Ok(bytes) = fs::read(&receipt) {
                    cancel.store(true, Ordering::Relaxed);
                    return (receipt, bytes);
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        cancel.store(true, Ordering::Relaxed);
        panic!("native source receipt was not observed");
    });
    let result = execute(
        &input,
        &output,
        &recipes,
        recipes[0],
        1,
        false,
        &mut dashboard,
    );
    let (receipt, source_bytes) = observer.join().unwrap();
    assert!(result.err().unwrap().to_string().contains("cancelled"));
    assert_eq!(
        fs::read(output.with_extension("fsd.json")).unwrap(),
        manifest
    );
    assert_eq!(fs::read(&receipt).unwrap(), source_bytes);
    // Family mismatch is refused before any completed shortcut or journal rewrite.
    let state_path = output
        .with_file_name("family.fsd.generation")
        .join("journal.json");
    let state = fs::read(&state_path).unwrap();
    assert!(
        execute(
            &input,
            &output,
            &recipes[..1],
            recipes[0],
            2,
            true,
            &mut Dashboard::new(false, false).unwrap()
        )
        .is_err()
    );
    assert_eq!(fs::read(&state_path).unwrap(), state);
    let resumed = execute(
        &input,
        &output,
        &recipes,
        recipes[0],
        2,
        true,
        &mut Dashboard::new(false, false).unwrap(),
    )
    .unwrap();
    assert_eq!(
        resumed.programs.as_ref().unwrap().catalogue.recipes.len(),
        2
    );
    assert!(!receipt.exists(), "completed staging is released");
    assert!(Artifact::load_with_options(&output, KernelLoadOptions { validate: true }).is_ok());
}
