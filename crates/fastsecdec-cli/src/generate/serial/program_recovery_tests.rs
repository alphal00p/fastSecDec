//! Recovery and cross-recipe fences for shared native chart preparation.
use super::*;
use fastsecdec::kernel::indexed::ProgramRecipe;

#[test]
fn shared_chart_receipts_survive_restart_and_reject_foreign_work() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.toml");
    let output = directory.path().join("integral.fsd");
    fs::write(&input, "[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['-1+eps']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1+eps'\nsemantics='causal'\n[generation]\norder=0\nmode='numerical_dual'\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    let recipes = vec![
        ProgramRecipe::UndeformedV1,
        ProgramRecipe::FixedV1,
        ProgramRecipe::DynamicPolynomialV1,
    ];
    let request = Request::PreparePrograms {
        input: input.clone(),
        workers: 1,
        overrides: Default::default(),
        recipes: recipes.clone(),
    };
    let family = family::RecipeFamily::new(recipes, ProgramRecipe::UndeformedV1).unwrap();
    let mut journal = Journal::open_with_family(
        &input,
        &output,
        false,
        "first",
        Default::default(),
        family.clone(),
    )
    .unwrap();
    journal.request("prepare", request.clone()).unwrap();
    jobs::execute(&journal.job_path("prepare"), &mut |_| Ok(())).unwrap();
    // The child published durably, but the coordinator died before accepting it.
    let receipt = fs::read(journal.response_path("prepare")).unwrap();
    drop(journal);
    let mut journal =
        Journal::open_with_family(&input, &output, true, "restart", Default::default(), family)
            .unwrap();
    let mut resumed = request.clone();
    let Request::PreparePrograms { workers, .. } = &mut resumed else {
        panic!()
    };
    *workers = 3;
    let (_, recovered) = journal.request("prepare", resumed).unwrap();
    let Some(Response::PreparedPrograms(prepared)) = recovered else {
        panic!()
    };
    assert_eq!(fs::read(journal.response_path("prepare")).unwrap(), receipt);
    for corruption in 0..5 {
        let mut foreign = prepared.clone();
        match corruption {
            0 => {
                foreign.native.recipes.pop();
            }
            1 => {
                foreign.native.recipes[1] = foreign.native.recipes[0].clone();
            }
            2 => {
                foreign.native.recipes[1].source_identity = "f".repeat(64);
            }
            3 => {
                foreign.native.recipes[1].dimension += 1;
            }
            _ => {
                foreign.native.recipes[1].charts[0].index += 1;
            }
        }
        assert!(
            journal::validate(
                &request,
                &Response::PreparedPrograms(foreign),
                &journal.root
            )
            .is_err()
        );
    }
    let source_request = Request::PrepareChartSource {
        preparation: journal.response_path("prepare"),
        source_identity: prepared.native.source_identity.clone(),
        dimension: prepared.native.recipes[0].dimension,
        map: prepared.native.recipes[0].charts[0].clone(),
    };
    journal.request("source-0", source_request.clone()).unwrap();
    jobs::execute(&journal.job_path("source-0"), &mut |_| Ok(())).unwrap();
    let Response::ChartSource(source) = journal.accept("source-0", &source_request).unwrap() else {
        panic!()
    };
    for corruption in 0..4 {
        let mut foreign = source.clone();
        match corruption {
            0 => {
                foreign.index += 1;
            }
            1 => {
                foreign.source_identity = "f".repeat(64);
            }
            2 => {
                foreign.dimension += 1;
            }
            _ => {
                foreign.map.blake3 = "f".repeat(64);
            }
        }
        assert!(
            journal::validate(
                &source_request,
                &Response::ChartSource(foreign),
                &journal.root
            )
            .is_err()
        );
    }
    let stored_source = fs::read(source.record.resolve(&journal.root).unwrap()).unwrap();
    let mut formulas = Vec::new();
    for recipe in &prepared.native.recipes {
        let discover = Request::DiscoverPrepared {
            preparation: journal.response_path("prepare"),
            program_recipe: recipe.program_recipe,
            source_id: recipe.source.blake3.clone(),
            source: source.clone(),
        };
        let key = format!("discover-{}-0", recipe.program_recipe.name());
        journal.request(&key, discover.clone()).unwrap();
        jobs::execute(&journal.job_path(&key), &mut |_| Ok(())).unwrap();
        let Response::Discovered(chart) = journal.accept(&key, &discover).unwrap() else {
            panic!()
        };
        for corruption in 0..4 {
            let mut foreign = chart.clone();
            match corruption {
                0 => {
                    foreign.index += 1;
                }
                1 => {
                    foreign.source_id = "f".repeat(64);
                }
                2 => {
                    foreign.dimension += 1;
                }
                _ => {
                    foreign.program_recipe = ProgramRecipe::DynamicSignAwareV1;
                }
            }
            assert!(
                journal::validate(&discover, &Response::Discovered(foreign), &journal.root)
                    .is_err()
            );
        }
        let formula = Request::Formula {
            preparation: journal.response_path("prepare"),
            chart,
        };
        let key = format!("formula-{}-0", recipe.program_recipe.name());
        journal.request(&key, formula.clone()).unwrap();
        jobs::execute(&journal.job_path(&key), &mut |_| Ok(())).unwrap();
        let Response::Formula(result) = journal.accept(&key, &formula).unwrap() else {
            panic!()
        };
        formulas.push((formula, result));
    }
    // The same endpoint formula key cannot transfer ownership between recipes.
    for (request, own) in &formulas {
        for (_, foreign) in &formulas {
            assert_eq!(
                journal::validate(request, &Response::Formula(foreign.clone()), &journal.root)
                    .is_ok(),
                own.program_recipe == foreign.program_recipe,
            );
        }
    }
    assert_eq!(
        fs::read(source.record.resolve(&journal.root).unwrap()).unwrap(),
        stored_source
    );
    // A committed compact receipt cannot hide a truncated heavyweight record.
    fs::write(
        source.record.resolve(&journal.root).unwrap(),
        b"interrupted write",
    )
    .unwrap();
    assert!(journal.accept("source-0", &source_request).is_err());
}
