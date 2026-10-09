use serde::{Deserialize, Serialize};

/// Mathematical prescription; validation is deliberately configured separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ContourMode {
    #[default]
    Off,
    Fixed {
        lambda: f64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContourValidation {
    #[default]
    Always,
    Pilot,
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourValidationOptions {
    pub policy: ContourValidation,
    pub pilot_points: usize,
}
impl Default for ContourValidationOptions {
    fn default() -> Self {
        Self {
            policy: ContourValidation::Always,
            pilot_points: 256,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourSettings {
    pub deformation: ContourMode,
    pub validation: ContourValidationOptions,
}
impl ContourSettings {
    /// Admission shared by native and CLI callers; no implicit strength tuning.
    pub fn validate(&self) -> Result<(), String> {
        if let ContourMode::Fixed { lambda } = self.deformation
            && (!lambda.is_finite() || lambda <= 0.0)
        {
            return Err("fixed contour strength must be finite and positive".into());
        }
        if self.validation.policy != ContourValidation::Off && self.validation.pilot_points == 0 {
            return Err("enabled contour validation requires at least one pilot point".into());
        }
        Ok(())
    }
}
