use super::*;
use crate::parametric::{ParametricTerm, PolynomialFactor};
use symbolica::{parse, state::State, symbol};

fn input(
    domain: ParametricDomain,
    role: FactorRole,
    semantics: FactorSemantics,
) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![
            symbol!("source_identity_test::x"),
            symbol!("source_identity_test::y"),
        ],
        symbol!("source_identity_test::eps"),
        domain,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!(
                        "source_identity_test::x+source_identity_test::y+source_identity_test::m"
                    ),
                    Atom::one(),
                    role,
                )
                .with_semantics(semantics),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn identity_retains_physical_roles_order_and_constraints() {
    let source = input(
        ParametricDomain::UnitCube,
        FactorRole::Singularity,
        FactorSemantics::Causal,
    );
    let m = symbol!("source_identity_test::m");
    let n = symbol!("source_identity_test::n");
    let identify =
        |input: &ParametricIntegrand, runtime: &[Symbol], constraints: &[RuntimeMassConstraint]| {
            source_identity(input, runtime, constraints).unwrap()
        };
    let baseline = identify(&source, &[m, n], &[]);
    crate::kernel::indexed::ProgramArchiveWriter::new(
        std::io::Cursor::new(Vec::<u8>::new()),
        baseline.clone(),
        [crate::kernel::indexed::ProgramRecipe::FixedV1],
    )
    .unwrap();
    assert_ne!(baseline, identify(&source, &[n, m], &[]));
    assert_ne!(baseline, identify(&source, &[m], &[]));
    for changed in [
        input(
            ParametricDomain::PositiveOrthant,
            FactorRole::Singularity,
            FactorSemantics::Causal,
        ),
        input(
            ParametricDomain::UnitCube,
            FactorRole::Polynomial,
            FactorSemantics::Causal,
        ),
        input(
            ParametricDomain::UnitCube,
            FactorRole::Singularity,
            FactorSemantics::Positive,
        ),
    ] {
        assert_ne!(baseline, identify(&changed, &[m, n], &[]));
    }
    let reversed_parameters = ParametricIntegrand::new(
        source.parameters().iter().copied().rev().collect(),
        source.regulator(),
        source.domain(),
        source.terms().to_vec(),
    )
    .unwrap();
    assert_ne!(baseline, identify(&reversed_parameters, &[m, n], &[]));
    let other_regulator = ParametricIntegrand::new(
        source.parameters().to_vec(),
        symbol!("source_identity_test::different_eps"),
        source.domain(),
        source.terms().to_vec(),
    )
    .unwrap();
    assert_ne!(baseline, identify(&other_regulator, &[m, n], &[]));
    let constraints = [RuntimeMassConstraint {
        name: "mass".into(),
        expression: Atom::var(m),
    }];
    assert_ne!(baseline, identify(&source, &[m, n], &constraints));
    let renamed = [RuntimeMassConstraint {
        name: "different".into(),
        expression: Atom::var(m),
    }];
    assert_ne!(
        identify(&source, &[m, n], &constraints),
        identify(&source, &[m, n], &renamed)
    );
    let changed = [RuntimeMassConstraint {
        name: "mass".into(),
        expression: Atom::var(n),
    }];
    assert_ne!(
        identify(&source, &[m, n], &constraints),
        identify(&source, &[m, n], &changed)
    );
    let with_recipe = [
        crate::contour::lambda_symbol(),
        m,
        crate::contour::dynamic::lambda_cap_symbol(),
        n,
        crate::contour::dynamic::safety_fraction_symbol(),
        crate::contour::dynamic::displacement_cap_symbol(),
    ];
    assert_eq!(baseline, identify(&source, &with_recipe, &[]));
    let user_symbol = symbol!("fastsecdec::contour::user_physical_mass");
    assert_ne!(baseline, identify(&source, &[m, n, user_symbol], &[]));
}

#[test]
fn canonical_source_identity_survives_fresh_process_symbol_and_ring_scrambling() {
    let directory = tempfile::tempdir().unwrap();
    let mut results = Vec::new();
    for count in [0, 19] {
        let path = directory.path().join(format!("identity-{count}.json"));
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "generation::identity::tests::source_identity_worker",
                "--nocapture",
            ])
            .env("FASTSECDEC_IDENTITY_WORKER_OUTPUT", &path)
            .env("FASTSECDEC_IDENTITY_WORKER_NOISE", count.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let result: (String, String) =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        results.push(result);
    }
    assert_eq!(results[0].0, results[1].0);
    // Prove this is an effective regression: the old state-transport digest
    // actually differs despite the identical ordered physical input.
    assert_ne!(results[0].1, results[1].1);
}

#[test]
fn source_identity_worker() {
    let Some(path) = std::env::var_os("FASTSECDEC_IDENTITY_WORKER_OUTPUT") else {
        return;
    };
    let count = std::env::var("FASTSECDEC_IDENTITY_WORKER_NOISE")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    for index in 0..count {
        let noise = symbol!(format!("source_identity_noise::p{index}"));
        let expression = Atom::one() + Atom::var(noise);
        let _registered_ring =
            expression.set_coefficient_ring(std::sync::Arc::new(vec![noise.into()]));
    }
    if count > 0 {
        let _ = symbol!(
            "source_identity_test::m",
            "source_identity_test::eps",
            "source_identity_test::y",
            "source_identity_test::x"
        );
    }
    let source = input(
        ParametricDomain::UnitCube,
        FactorRole::Singularity,
        FactorSemantics::Causal,
    );
    let identity = source_identity(&source, &[symbol!("source_identity_test::m")], &[]).unwrap();
    let mut transport = Vec::new();
    State::export_partial(
        &mut transport,
        source.terms()[0].factors()[0]
            .polynomial()
            .get_all_symbols(true),
    )
    .unwrap();
    let transport_digest = blake3::hash(&transport).to_hex().to_string();
    std::fs::write(
        path,
        serde_json::to_vec(&(identity, transport_digest)).unwrap(),
    )
    .unwrap();
}
