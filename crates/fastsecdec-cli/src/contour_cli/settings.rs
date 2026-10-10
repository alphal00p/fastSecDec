//! CLI adaptation of native mathematical settings; no sampling or CAS work.
use super::*;

pub(super) fn parse_construction(value: &str) -> Result<DynamicConstruction, String> {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .map_err(|_| "construction must be polynomial or sign_aware".into())
}

pub(super) fn parse_diagnostics(
    value: &str,
) -> Result<fastsecdec::contour::ContourDiagnosticsMode, String> {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .map_err(|_| "contour diagnostics must be disabled or aggregate".into())
}

impl ContourArgs {
    pub(crate) fn apply_integration(&self, settings: &mut IntegrationInput) -> CliResult<()> {
        self.apply(&mut settings.contour)?;
        if let Some(mode) = self.contour_diagnostics {
            settings.contour_diagnostics = mode;
        }
        Ok(())
    }

    pub(crate) fn apply(&self, settings: &mut ContourSettings) -> CliResult<()> {
        if let Some(mode) = self.contour.as_deref() {
            settings.deformation = match mode {
                "off" => ContourMode::Off,
                "fixed" => ContourMode::Fixed {
                    lambda: self.lambda.or(match settings.deformation {
                        ContourMode::Fixed { lambda } => Some(lambda),
                        _ => None,
                    }).ok_or("--contour fixed requires --lambda or a fixed strength in the runtime settings")?,
                },
                value if value.starts_with("dynamical=") => {
                    let safety_fraction = value["dynamical=".len()..].parse::<f64>()
                        .map_err(|_| "dynamical contour requires a numerical safety fraction: dynamical=<S>")?;
                    match settings.deformation {
                        ContourMode::Dynamical { lambda_cap, displacement_cap, construction, .. } =>
                            ContourMode::Dynamical { safety_fraction, lambda_cap, displacement_cap, construction },
                        _ => ContourMode::dynamical(safety_fraction),
                    }
                }
                _ => return Err("contour must be fixed, off or dynamical=<S>; dynamic execution requires an admitted dynamic recipe".into()),
            };
        }
        if let Some(lambda) = self.lambda {
            if !matches!(settings.deformation, ContourMode::Fixed { .. }) {
                return Err("--lambda requires a fixed contour prescription".into());
            }
            settings.deformation = ContourMode::Fixed { lambda };
        }
        if self.lambda_cap.is_some()
            || self.displacement_cap.is_some()
            || self.contour_construction.is_some()
        {
            let ContourMode::Dynamical {
                lambda_cap,
                displacement_cap,
                construction,
                ..
            } = &mut settings.deformation
            else {
                return Err(
                    "dynamic caps and construction require a dynamical contour prescription".into(),
                );
            };
            if let Some(value) = self.lambda_cap {
                *lambda_cap = value;
            }
            if let Some(value) = self.displacement_cap {
                *displacement_cap = value;
            }
            if let Some(value) = self.contour_construction {
                *construction = value;
            }
        }
        if let Some(policy) = self.contour_validation {
            settings.validation.policy = match policy {
                ValidationArg::Always => ContourValidation::Always,
                ValidationArg::Pilot => ContourValidation::Pilot,
                ValidationArg::Off => ContourValidation::Off,
            };
        }
        if let Some(points) = self.contour_pilot_points {
            settings.validation.pilot_points = points.get();
        }
        settings.validate()?;
        Ok(())
    }

    /// Resolve steering before generation without registering native symbols.
    /// The requested resident recipe is independent of artifact defaults.
    pub(crate) fn generation_request(
        &self,
        base: &IntegrationInput,
        overlay: Option<&std::path::Path>,
    ) -> CliResult<(
        crate::config::GenerationOverrides,
        fastsecdec::kernel::ProgramRecipe,
    )> {
        let contour = self.resolve(base, overlay)?;
        Ok((
            crate::config::GenerationOverrides {
                contour: !matches!(contour.deformation, ContourMode::Off),
                recipe: None,
            },
            contour.deformation.program_recipe(),
        ))
    }

    pub(crate) fn resolve(
        &self,
        base: &IntegrationInput,
        overlay: Option<&std::path::Path>,
    ) -> CliResult<ContourSettings> {
        let mut effective = serde_json::to_value(base)?;
        if let Some(path) = overlay {
            let overlay = crate::config::read_integration_overlay(path)?;
            crate::config::merge_runtime_values(&mut effective, overlay);
        }
        let effective: IntegrationInput = serde_json::from_value(effective)?;
        let mut contour = effective.contour;
        self.apply(&mut contour)?;
        Ok(contour)
    }
}

#[cfg(test)]
mod tests;
