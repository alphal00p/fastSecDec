//! Intended-graph acceptance: native A446 -> signed export -> symGCAD -> verify.
#![cfg(target_os = "linux")]

use fastsecdec::{Atom, AtomCore};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use symbolica::{id::Pattern, parser::ParseSettings};

#[path = "support/external_workflow.rs"]
mod external_workflow;
#[path = "../../../examples/no_deformation/a446/prepare.rs"]
mod fixture;
use external_workflow::run;

fn atom(text: &str) -> Atom {
    Atom::parse(text, "a446_wire_check", ParseSettings::default()).unwrap()
}

fn exact_sparse(terms: &serde_json::Value, names: &[String]) -> Atom {
    terms
        .as_array()
        .unwrap()
        .iter()
        .map(|term| {
            let exponents = term["exponents"].as_array().unwrap();
            assert_eq!(exponents.len(), names.len());
            exponents.iter().zip(names).fold(
                atom(term["coefficient"].as_str().unwrap()),
                |value, (degree, name)| value * atom(name).pow(degree.as_u64().unwrap() as u32),
            )
        })
        .sum::<Atom>()
        .expand()
}

fn assert_corpus_identity(bundle: &serde_json::Value) {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/no_deformation/a446/expected_uf.json"
    ))
    .unwrap();
    let original_names = expected["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    // Corpus [x1,x2,x3,x4,x5,x7,x8] versus native internal edges 4..10.
    // Use simultaneous fresh names before mapping: source and wire names overlap.
    let wire_for_corpus = ["x0", "x1", "x2", "x4", "x3", "x5", "x6"];
    for (upper, lower) in [("U", "u"), ("F", "f")] {
        let mut original = exact_sparse(&expected[upper]["terms"], &original_names);
        for (name, value) in [("mt2", "1"), ("mz2", "5/18"), ("s", "10"), ("t", "-3")] {
            original = original
                .replace(Pattern::Literal(atom(name)))
                .with(Pattern::Literal(atom(value)));
        }
        for (index, name) in original_names.iter().enumerate() {
            original = original
                .replace(Pattern::Literal(atom(name)))
                .with(Pattern::Literal(atom(&format!("corpus_{index}"))));
        }
        for (index, name) in wire_for_corpus.iter().enumerate() {
            original = original
                .replace(Pattern::Literal(atom(&format!("corpus_{index}"))))
                .with(Pattern::Literal(atom(name)));
        }
        let sparse = &bundle["specialized"][lower];
        let names = sparse["variables"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert!(
            (original - exact_sparse(&sparse["terms"], &names))
                .expand()
                .is_zero()
        );
    }
    let bindings = bundle["graph"]["propagators"].as_array().unwrap();
    assert_eq!(bindings.len(), 7);
    for (index, binding) in bindings.iter().enumerate() {
        assert_eq!(binding["edge_id"], index + 4);
        assert_eq!(binding["denominator_index"], index);
        assert_eq!(binding["parameter"], format!("x{index}"));
    }
}

#[test]
#[ignore = "requires an explicit SYMGCAD_BIN and external wall/memory allocation"]
fn native_a446_export_import_solve_and_independent_verify() {
    let symgcad = PathBuf::from(std::env::var_os("SYMGCAD_BIN").expect("set SYMGCAD_BIN"));
    assert!(symgcad.is_file());
    let seconds = std::env::var("FASTSECDEC_INTEROP_SECONDS")
        .ok()
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(180);
    assert!((1..=600).contains(&seconds));
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let directory = if let Some(path) = std::env::var_os("FASTSECDEC_INTEROP_OUTPUT") {
        let path = PathBuf::from(path);
        fs::create_dir(&path).expect("interop output must be fresh");
        path
    } else {
        tempfile::tempdir().unwrap().keep()
    };
    println!("retained A446 evidence: {}", directory.display());
    let prepared = directory.join("native-input");
    let preparation = fixture::prepare(&prepared).unwrap();
    fs::write(
        directory.join("preparation.json"),
        serde_json::to_vec_pretty(&preparation).unwrap(),
    )
    .unwrap();
    let exported = directory.join("export");
    let mut export = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    export
        .args(["--json", "--plain", "export-symanzik"])
        .arg(prepared.join("run.toml"))
        .arg("--point")
        .arg(prepared.join("point.toml"))
        .arg("--output")
        .arg(&exported);
    run(export, &directory, "export", deadline);
    let bundle =
        serde_json::from_slice(&fs::read(exported.join("symanzik.json")).unwrap()).unwrap();
    assert_corpus_identity(&bundle);
    let checked = directory.join("checked.toml");
    let mut import = Command::new(&symgcad);
    import
        .arg("import-fastsecdec")
        .arg(exported.join("symanzik.json"))
        .arg("--problem")
        .arg(exported.join("problem.toml"))
        .arg("--output")
        .arg(&checked);
    assert_eq!(
        run(import, &directory, "import", deadline)["coverage_claimed"],
        false
    );

    let mut problem: toml::Value = toml::from_str(&fs::read_to_string(&checked).unwrap()).unwrap();
    // Only solver configuration and limits change. All imported polynomials, parameters,
    // groups and strict domain constraints remain exactly as checked.
    let scheduling: toml::Value = toml::from_str(
        r#"
        [solver]
        engine = "monotone"
        ordering = "given"
        order = ["x4", "x1", "x6", "x5", "x3", "x2", "x0"]
        homogeneous_block = ["x0", "x1", "x2", "x3", "x4", "x5", "x6"]
        homogeneous_anchor = "x4"
        positivity = true
        input_positivity = true
        merge = "none"
        [limits]
        wall_time_secs = 60.0
        memory_mib = 2048
        workers = 1
        "#,
    )
    .unwrap();
    for key in ["solver", "limits"] {
        problem
            .as_table_mut()
            .unwrap()
            .insert(key.into(), scheduling[key].clone());
    }
    let solve_input = directory.join("solve.toml");
    fs::write(&solve_input, toml::to_string_pretty(&problem).unwrap()).unwrap();
    let proof = directory.join("result.json");
    let mut solve = Command::new(&symgcad);
    solve
        .arg("solve")
        .arg(&solve_input)
        .arg("--output")
        .arg(&proof);
    run(solve, &directory, "solve", deadline);
    let result: serde_json::Value = serde_json::from_slice(&fs::read(&proof).unwrap()).unwrap();
    assert_eq!(result["status"], "complete_generic");
    let mut verify = Command::new(&symgcad);
    verify.arg("verify").arg(&proof);
    let verified = run(verify, &directory, "verify", deadline);
    assert_eq!(verified["verified"], true);
    assert_eq!(verified["cells"], 8);
    let result: serde_json::Value = serde_json::from_slice(&fs::read(&proof).unwrap()).unwrap();
    let cells = result["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 8);
    let signs = cells
        .iter()
        .map(|cell| {
            let signs = cell["signs"].as_array().unwrap();
            assert_eq!(signs.len(), 1);
            signs[0].as_i64().unwrap()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(signs, [-1, 1].into());
    assert!(Path::new(&proof).is_file());
}
