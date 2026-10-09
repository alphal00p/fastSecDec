//! Runtime contour steering, separate from sampling and mathematical generation.
use clap::{Args, ValueEnum};
use fastsecdec::contour::{ContourMode, ContourSettings, ContourValidation};

use crate::{CliResult, config::IntegrationInput};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ValidationArg {
    Always,
    Pilot,
    Off,
}

#[derive(Args, Default)]
pub(crate) struct ContourArgs {
    /// Contour prescription: fixed or off. Requires contour-capable kernels.
    #[arg(long, value_name = "MODE")]
    contour: Option<String>,
    /// Positive fixed contour strength; never automatically reduced.
    #[arg(long, allow_hyphen_values = true)]
    lambda: Option<f64>,
    /// Check every production point, a pilot only, or disable causal checks.
    #[arg(long, value_enum)]
    contour_validation: Option<ValidationArg>,
    /// Number of independently assigned pilot points per sector.
    #[arg(long)]
    contour_pilot_points: Option<std::num::NonZeroUsize>,
}

impl ContourArgs {
    pub(crate) fn load_diagnostics(
        &self,
        path: &std::path::Path,
    ) -> CliResult<(crate::artifact::Artifact, fastsecdec::kernel::KernelSet)> {
        let mut artifact = crate::artifact::Artifact::load_metadata(path)?;
        if artifact.programs.is_none() {
            return crate::artifact::Artifact::load(path);
        }
        let settings: IntegrationInput =
            serde_json::from_value(artifact.provenance.integration.clone())?;
        select_program(&mut artifact, &self.resolve(&settings, None)?)?;
        crate::artifact::Artifact::load_recipe_observed(
            path,
            fastsecdec::kernel::KernelLoadOptions::default(),
            artifact.selected_recipe(),
            |_| Ok(()),
            |_| std::ops::ControlFlow::Continue(()),
        )
    }
    /// Diagnostic commands use the artifact's recorded physical point and the
    /// same explicit contour preflight as integration. This creates no sampling
    /// session and does not contribute any observation to an integral estimate.
    pub(crate) fn prepare_diagnostics(
        &self,
        artifact: &crate::artifact::Artifact,
        kernels: &mut fastsecdec::kernel::KernelSet,
        dashboard: &mut crate::display::Dashboard,
    ) -> CliResult<Option<DiagnosticContour>> {
        let mut settings: IntegrationInput =
            serde_json::from_value(artifact.provenance.integration.clone())?;
        self.apply(&mut settings.contour)?;
        crate::bind_parameters(kernels, &settings)?;
        if kernels.contour_capable() && settings.contour.validation.policy != ContourValidation::Off
        {
            dashboard.begin_loading();
        }
        let started = std::time::Instant::now();
        let pilot =
            crate::contour_pilot::run(kernels, &settings.contour, settings.seed, None, |pilot| {
                dashboard.loading(&crate::loading::Snapshot {
                    phase: crate::loading::Phase::ContourValidation,
                    completed: Some(pilot.completed),
                    total: Some(pilot.total),
                    elapsed_seconds: started.elapsed().as_secs_f64(),
                })?;
                if dashboard.cancelled() {
                    return Err("contour preflight cancelled".into());
                }
                Ok(())
            })?;
        Ok(
            (settings.contour.deformation != ContourMode::Off).then(|| DiagnosticContour {
                settings: settings.contour,
                pilot: pilot
                    .as_ref()
                    .map(|report| crate::contour_pilot::provenance(kernels, settings.seed, report)),
            }),
        )
    }

    pub(crate) fn apply(&self, settings: &mut ContourSettings) -> CliResult<()> {
        if let Some(mode) = self.contour.as_deref() {
            settings.deformation = match mode {
                "off" => ContourMode::Off,
                "fixed" => ContourMode::Fixed {
                    lambda: self.lambda.or(match settings.deformation {
                        ContourMode::Fixed { lambda } => Some(lambda),
                        ContourMode::Off => None,
                    }).ok_or("--contour fixed requires --lambda or a fixed strength in the runtime settings")?,
                },
                _ => return Err("contour must be fixed or off; dynamic deformation follows the fixed-mode validation milestone".into()),
            };
        }
        if let Some(lambda) = self.lambda {
            if !matches!(settings.deformation, ContourMode::Fixed { .. }) {
                return Err("--lambda requires a fixed contour prescription".into());
            }
            settings.deformation = ContourMode::Fixed { lambda };
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

    /// Resolve only contour steering before generation, without registering
    /// native parameter symbols in a bounded-memory coordinator process.
    pub(crate) fn generation_overrides(
        &self,
        base: &IntegrationInput,
        overlay: Option<&std::path::Path>,
    ) -> CliResult<crate::config::GenerationOverrides> {
        let contour = self.resolve(base, overlay)?;
        Ok(crate::config::GenerationOverrides {
            contour: !matches!(contour.deformation, ContourMode::Off),
        })
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

/// Choose the numerical program without binding parameters or importing any
/// Symbolica state. Legacy single-program files retain their usual admission.
pub(crate) fn select_program(
    artifact: &mut crate::artifact::Artifact,
    settings: &ContourSettings,
) -> CliResult<()> {
    if artifact.programs.is_some() {
        use fastsecdec::kernel::indexed::ProgramRecipe;
        artifact.select_recipe(match settings.deformation {
            ContourMode::Off => ProgramRecipe::UndeformedV1,
            ContourMode::Fixed { .. } => ProgramRecipe::FixedV1,
        })?;
    }
    Ok(())
}

/// CLI presentation joins existing native settings and evidence without changing
/// the stable native benchmark/boundary report schemas.
pub(crate) struct DiagnosticContour {
    settings: ContourSettings,
    pilot: Option<fastsecdec::status::ContourPilotProvenance>,
}

impl DiagnosticContour {
    pub(crate) fn attach(
        self,
        kernels: &mut fastsecdec::kernel::KernelSet,
        report: &mut serde_json::Value,
        checked_work: &str,
    ) -> CliResult<()> {
        let mut checks = fastsecdec::status::ContourCheckCounters::default();
        for kernel in kernels.sectors_mut() {
            if let Some(delta) = kernel.take_contour_validation_report() {
                checks.merge(fastsecdec::status::ContourCheckCounters {
                    checked_arguments: delta.checked_arguments.try_into()?,
                    maximum_bits: delta.maximum_bits,
                })?;
            }
        }
        report["contour"] = serde_json::json!({
            "settings": self.settings,
            "pilot": self.pilot,
            "diagnostic_checks": checks,
            "checked_work": checked_work,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct ParserProbe {
        #[command(flatten)]
        contour: ContourArgs,
    }

    #[test]
    fn positive_strength_has_no_upper_bound_and_validation_is_independent() {
        let args = ParserProbe::try_parse_from([
            "test",
            "--contour",
            "fixed",
            "--lambda",
            "4",
            "--contour-validation",
            "off",
        ])
        .unwrap();
        let mut settings = ContourSettings::default();
        args.contour.apply(&mut settings).unwrap();
        assert_eq!(settings.deformation, ContourMode::Fixed { lambda: 4. });
        assert_eq!(settings.validation.policy, ContourValidation::Off);
        for strength in ["0", "-1", "NaN", "inf"] {
            let args =
                ParserProbe::try_parse_from(["test", "--contour", "fixed", "--lambda", strength])
                    .unwrap();
            assert!(args.contour.apply(&mut ContourSettings::default()).is_err());
        }
    }

    #[test]
    fn pilot_override_preserves_the_configured_strength() {
        let args = ParserProbe::try_parse_from([
            "test",
            "--contour-validation",
            "pilot",
            "--contour-pilot-points",
            "31",
        ])
        .unwrap();
        let mut settings = ContourSettings {
            deformation: ContourMode::Fixed { lambda: 0.2 },
            ..Default::default()
        };
        args.contour.apply(&mut settings).unwrap();
        assert_eq!(settings.deformation, ContourMode::Fixed { lambda: 0.2 });
        assert_eq!(settings.validation.policy, ContourValidation::Pilot);
        assert_eq!(settings.validation.pilot_points, 31);
    }
}
