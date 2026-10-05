use super::GenerationInput;
use fastsecdec::generation::{CoefficientExpansionMethod, CoefficientExpansionOptions};

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
