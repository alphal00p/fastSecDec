use super::*;
use clap::Parser;
use fastsecdec::kernel::ProgramRecipe;

#[derive(Parser)]
struct Probe {
    #[command(flatten)]
    contour: ContourArgs,
}

fn arguments(args: &[&str]) -> ContourArgs {
    Probe::try_parse_from(std::iter::once("test").chain(args.iter().copied()))
        .unwrap()
        .contour
}

#[test]
fn dynamic_steering_uses_native_defaults_and_requests_its_actual_recipe() {
    let args = arguments(&["--contour", "dynamical=0.8"]);
    let resolved = args.resolve(&IntegrationInput::default(), None).unwrap();
    assert_eq!(resolved.deformation, ContourMode::dynamical(0.8));
    assert_eq!(
        resolved.deformation.program_recipe(),
        ProgramRecipe::DynamicSignAwareV1
    );
    let (overrides, resident) = args
        .generation_request(&IntegrationInput::default(), None)
        .unwrap();
    assert_eq!(resident, ProgramRecipe::DynamicSignAwareV1);
    assert_eq!(overrides.recipe, Some(resident));

    let args = arguments(&[
        "--contour",
        "dynamical=0.6",
        "--lambda-cap",
        "2",
        "--displacement-cap",
        "3",
        "--contour-construction",
        "polynomial",
        "--contour-validation",
        "off",
    ]);
    let resolved = args.resolve(&IntegrationInput::default(), None).unwrap();
    assert_eq!(
        resolved.deformation,
        ContourMode::Dynamical {
            safety_fraction: 0.6,
            lambda_cap: 2.,
            displacement_cap: 3.,
            construction: DynamicConstruction::Polynomial,
        }
    );
    assert_eq!(
        resolved.deformation.program_recipe(),
        ProgramRecipe::DynamicPolynomialV1
    );
    assert_eq!(resolved.validation.policy, ContourValidation::Off);
    let (overrides, resident) = args
        .generation_request(&IntegrationInput::default(), None)
        .unwrap();
    assert_eq!(resident, ProgramRecipe::DynamicPolynomialV1);
    assert_eq!(overrides.recipe, Some(resident));
}

#[test]
fn native_ranges_and_cross_mode_flags_never_silently_replace_a_prescription() {
    for value in [
        "dynamical=0",
        "dynamical=1",
        "dynamical=-0.1",
        "dynamical=NaN",
        "dynamical=inf",
        "dynamical=",
        "dynamical=no",
    ] {
        assert!(
            arguments(&["--contour", value])
                .resolve(&IntegrationInput::default(), None)
                .is_err()
        );
    }
    for value in ["0", "-1", "NaN", "inf"] {
        for flag in ["--lambda-cap", "--displacement-cap"] {
            assert!(
                arguments(&["--contour", "dynamical=0.8", flag, value])
                    .resolve(&IntegrationInput::default(), None)
                    .is_err()
            );
        }
    }
    for args in [
        vec!["--contour", "fixed", "--lambda", ".2", "--lambda-cap", "1"],
        vec!["--contour", "off", "--contour-construction", "polynomial"],
        vec!["--contour", "dynamical=.8", "--lambda", ".2"],
    ] {
        assert!(
            arguments(&args)
                .resolve(&IntegrationInput::default(), None)
                .is_err()
        );
    }
    assert!(Probe::try_parse_from(["test", "--lambda-cpa", "2"]).is_err());
    assert!(Probe::try_parse_from(["test", "--contour-construction", "polynomal"]).is_err());
}

#[test]
fn dynamic_overlays_preserve_caps_and_can_explicitly_select_undeformed() {
    let directory = tempfile::tempdir().unwrap();
    let overlay = directory.path().join("runtime.toml");
    std::fs::write(&overlay, "[contour.deformation]\nmode='dynamical'\nsafety_fraction=0.7\nlambda_cap=4\ndisplacement_cap=5\nconstruction='polynomial'\n").unwrap();
    let resolved = arguments(&[
        "--contour",
        "dynamical=0.9",
        "--contour-validation",
        "pilot",
    ])
    .resolve(&IntegrationInput::default(), Some(&overlay))
    .unwrap();
    assert_eq!(
        resolved.deformation,
        ContourMode::Dynamical {
            safety_fraction: 0.9,
            lambda_cap: 4.,
            displacement_cap: 5.,
            construction: DynamicConstruction::Polynomial,
        }
    );
    assert_eq!(resolved.validation.policy, ContourValidation::Pilot);
    let (overrides, resident) = arguments(&["--contour", "off"])
        .generation_request(&IntegrationInput::default(), Some(&overlay))
        .unwrap();
    assert!(!overrides.contour);
    assert_eq!(resident, ProgramRecipe::UndeformedV1);
    std::fs::write(
        &overlay,
        "[contour.deformation]\nmode='dynamical'\nsafety_fraction=0.7\nlambda_cpa=4\n",
    )
    .unwrap();
    assert!(
        arguments(&[])
            .resolve(&IntegrationInput::default(), Some(&overlay))
            .is_err()
    );
}

#[test]
fn explicit_mode_changes_drop_only_previous_variant_fields() {
    let directory = tempfile::tempdir().unwrap();
    let overlay = directory.path().join("runtime.toml");
    let mut base = IntegrationInput::default();
    base.contour.deformation = ContourMode::Fixed { lambda: 0.4 };
    base.contour.validation.policy = ContourValidation::Pilot;
    std::fs::write(
        &overlay,
        "[contour.deformation]\nmode='dynamical'\nsafety_fraction=0.7\n",
    )
    .unwrap();
    let dynamic = arguments(&[]).resolve(&base, Some(&overlay)).unwrap();
    assert_eq!(dynamic.deformation, ContourMode::dynamical(0.7));
    assert_eq!(dynamic.validation.policy, ContourValidation::Pilot);
    base.contour = dynamic;
    std::fs::write(
        &overlay,
        "[contour.deformation]\nmode='fixed'\nlambda=0.3\n",
    )
    .unwrap();
    assert_eq!(
        arguments(&[])
            .resolve(&base, Some(&overlay))
            .unwrap()
            .deformation,
        ContourMode::Fixed { lambda: 0.3 }
    );
    std::fs::write(&overlay, "[contour.deformation]\nmode='off'\n").unwrap();
    assert_eq!(
        arguments(&[])
            .resolve(&base, Some(&overlay))
            .unwrap()
            .deformation,
        ContourMode::Off
    );
}

#[test]
fn generation_recipe_flags_use_native_ids_and_reject_conflicting_switches() {
    for recipe in [
        ProgramRecipe::UndeformedV1,
        ProgramRecipe::FixedV1,
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        let cli = crate::Cli::try_parse_from([
            "fastsecdec",
            "generate",
            "unused.toml",
            "--recipe",
            recipe.name(),
        ])
        .unwrap();
        assert!(
            matches!(cli.command, crate::Action::Generate { recipe: Some(actual), contour: false, .. } if actual == recipe)
        );
        assert!(
            crate::Cli::try_parse_from([
                "fastsecdec",
                "generate",
                "unused.toml",
                "--contour",
                "--recipe",
                recipe.name()
            ])
            .is_err()
        );
    }
    assert!(
        crate::Cli::try_parse_from([
            "fastsecdec",
            "generate",
            "unused.toml",
            "--recipe",
            "dynamic-sign-aware-v2"
        ])
        .is_err()
    );
}
