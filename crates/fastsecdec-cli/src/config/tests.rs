use super::GenerationInput;
use fastsecdec::generation::{CoefficientExpansionMethod, CoefficientExpansionOptions};
use fastsecdec::parametric::FamilyPreparationPolicy;

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
        FamilyPreparationPolicy::SingleUnitTerm { max_states: 32 }
    );
    for invalid in [
        "family_preparation='Unknown'",
        "[family_preparation.SingleUnitTerm]\nmax_states=-1",
        "[family_preparation.SingleUnitTerm]",
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
        "[coefficient_expansion]\nmethod='native_named'\nmax_series_attempts=4\nmax_relative_width=12\nmax_unique_requests=32",
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
