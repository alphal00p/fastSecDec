use super::GenerationInput;
use fastsecdec::generation::{CoefficientExpansionMethod, CoefficientExpansionOptions};
use fastsecdec::parametric::FamilyPreparationPolicy;

#[test]
fn periodization_reuses_native_variants_and_preserves_the_default() {
    use super::IntegrationInput;
    use fastsecdec::integration::Periodization;

    assert_eq!(
        IntegrationInput::default()
            .qmc_settings()
            .unwrap()
            .periodization,
        Periodization::Korobov3
    );
    for (name, expected) in [
        ("none", Periodization::None),
        ("korobov3", Periodization::Korobov3),
        ("korobov2", Periodization::Korobov2),
    ] {
        let input: IntegrationInput = toml::from_str(&format!("periodization='{name}'")).unwrap();
        assert_eq!(input.qmc_settings().unwrap().periodization, expected);
        let restored: IntegrationInput =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
        assert_eq!(
            restored.qmc_settings().unwrap(),
            input.qmc_settings().unwrap()
        );
    }
    for name in ["unknown", "Korobov2"] {
        let input: IntegrationInput = toml::from_str(&format!("periodization='{name}'")).unwrap();
        assert_eq!(
            input.qmc_settings().unwrap_err().to_string(),
            "periodization must be none, korobov2 or korobov3"
        );
    }
}

#[test]
fn family_policy_uses_native_type_with_explicit_original_cli_default() {
    let historical: GenerationInput = toml::from_str("order=1").unwrap();
    assert_eq!(
        historical.family_preparation,
        FamilyPreparationPolicy::Original
    );
    let original: GenerationInput = toml::from_str("family_preparation='Original'").unwrap();
    assert_eq!(original.family_preparation, historical.family_preparation);
    let prepared: GenerationInput =
        toml::from_str("[family_preparation.SingleUnitTerm]\nmax_states=32").unwrap();
    assert_eq!(
        prepared.family_preparation,
        FamilyPreparationPolicy::SingleUnitTerm {
            max_states: Some(32)
        }
    );
    let unlimited: GenerationInput = toml::from_str("[family_preparation.SingleUnitTerm]").unwrap();
    assert_eq!(
        unlimited.family_preparation,
        FamilyPreparationPolicy::default()
    );
    for invalid in [
        "family_preparation='Unknown'",
        "[family_preparation.SingleUnitTerm]\nmax_states=-1",
    ] {
        assert!(toml::from_str::<GenerationInput>(invalid).is_err());
    }
}

#[test]
fn coefficient_options_reuse_native_defaults_and_reject_unknown_steering() {
    let historical: GenerationInput = toml::from_str("order=1").unwrap();
    assert_eq!(
        historical.coefficient_expansion,
        CoefficientExpansionOptions::default()
    );
    let opted_in: GenerationInput = toml::from_str(
        "[coefficient_expansion]\nmethod='coefficient_series'\nmax_series_attempts=4\nmax_relative_width=12\nmax_unique_requests=32",
    ).unwrap();
    assert_eq!(
        opted_in.coefficient_expansion,
        CoefficientExpansionOptions {
            method: CoefficientExpansionMethod::NativeNamed,
            max_series_attempts: Some(4),
            max_relative_width: Some(12),
            max_unique_requests: Some(32),
        }
    );
    for invalid in [
        "[coefficient_expansion]\nmethod='unknown'",
        "[coefficient_expansion]\nresolver='interleaved'",
        "[coefficient_expansion]\nmax_unique_requests=-1",
    ] {
        assert!(toml::from_str::<GenerationInput>(invalid).is_err());
    }
}

#[test]
fn generation_mode_and_subtraction_use_native_defaults_and_strict_names() {
    use fastsecdec::generation::{GenerationMode, GenerationOptions, SubtractionStrategy};
    let historical: GenerationInput = toml::from_str("order=1").unwrap();
    let native = GenerationOptions::default();
    assert_eq!(historical.mode, native.mode);
    assert_eq!(historical.mode, GenerationMode::Symbolic);
    assert_eq!(historical.subtraction, native.subtraction);
    assert_eq!(historical.subtraction, SubtractionStrategy::Taylor);
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let input: GenerationInput = toml::from_str(&format!(
                "mode='{}'\nsubtraction='{}'",
                mode.name(),
                subtraction.name(),
            ))
            .unwrap();
            assert_eq!(input.mode, mode);
            assert_eq!(input.subtraction, subtraction);
        }
    }
    for invalid in [
        "mode='numeric'",
        "mode='Symbolic'",
        "mode=1",
        "subtraction='ibp'",
        "subtraction='Taylor'",
    ] {
        assert!(
            toml::from_str::<GenerationInput>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn generation_diagnostics_default_off_and_resource_settings_are_explicit() {
    let default = GenerationInput::default();
    assert_eq!(default.max_sectors, None);
    assert_eq!(default.max_support_pairs, None);
    assert_eq!(default.max_rays, None);
    assert_eq!(default.max_subtractions_per_axis, None);
    assert_eq!(default.max_subtraction_terms, None);
    assert_eq!(default.resource_limits().unwrap(), Default::default());
    let explicit: GenerationInput = toml::from_str(
        "max_sectors=2\nmax_support_pairs=3\nmax_rays=4\nmax_subtractions_per_axis=5\nmax_subtraction_terms=6\nmax_process_rss_bytes=1024\nmax_elapsed_seconds=0.0"
    ).unwrap();
    assert_eq!(explicit.max_sectors, Some(2));
    assert_eq!(explicit.max_support_pairs, Some(3));
    assert_eq!(explicit.max_rays, Some(4));
    assert_eq!(explicit.max_subtractions_per_axis, Some(5));
    assert_eq!(explicit.max_subtraction_terms, Some(6));
    assert_eq!(
        explicit.resource_limits().unwrap().max_process_rss_bytes,
        Some(1024)
    );
    assert_eq!(
        explicit.resource_limits().unwrap().max_elapsed,
        Some(std::time::Duration::ZERO)
    );
    for seconds in [f64::NAN, f64::INFINITY, -1.0] {
        let input = GenerationInput {
            max_elapsed_seconds: Some(seconds),
            ..Default::default()
        };
        assert!(input.resource_limits().is_err());
    }
}
