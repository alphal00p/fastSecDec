//! Contour settings bind separately from physical inputs; the numerical
//! identity includes strength but never the optional validation policy.
use super::{KernelError, KernelSet};
use crate::contour::{ContourMode, ContourSettings, ContourValidationOptions};
use std::collections::BTreeMap;
use symbolica::atom::Symbol;
mod binding;
mod checks;
pub(in crate::kernel) mod dynamic;
pub(super) use binding::ContourBinding;
pub(super) use checks::{CheckProgram, SectorValidation};
pub use checks::{
    ContourCheckReport, ContourProductionReport, ContourValidationChart, ContourValidationReport,
};
pub(super) use checks::{admit_certified_arithmetic, build_checks};
#[cfg(test)]
mod tests;

impl KernelSet {
    /// Admit a bound numerical problem after the caller completes any required
    /// contour pilot. Exact contributions need this gate too: they have no
    /// stochastic evaluator whose first sample could otherwise enforce it.
    /// Metadata and exact-coefficient inspection remain available before it.
    pub fn validate_integration_readiness(
        &self,
        scope: &crate::results::ResultScope,
    ) -> Result<(), KernelError> {
        if !self.parameters_bound() {
            return Err(KernelError::UnboundParameters);
        }
        if let Some(binding) = &self.contour_binding {
            binding.require_ready(scope)?;
        } else if self.contour_capable() {
            return Err(KernelError::Contour(
                "bind a contour prescription before integration".into(),
            ));
        }
        Ok(())
    }

    /// Bound mathematical prescription and current checking policy, if present.
    pub fn contour_settings(&self) -> Option<&ContourSettings> {
        self.contour_binding.as_ref().map(ContourBinding::settings)
    }
    /// Capability is retained in the native evaluator schema, including exact
    /// records without stochastic sectors, and survives selective loading.
    pub fn contour_capable(&self) -> bool {
        self.program_recipe() != super::ProgramRecipe::UndeformedV1
    }

    /// Atomically bind the physical point and a requested contour prescription.
    /// No symbolic regeneration or evaluator optimization occurs here.
    pub fn bind_parameters_with_contour(
        &mut self,
        physics: &BTreeMap<Symbol, f64>,
        settings: &ContourSettings,
    ) -> Result<(), KernelError> {
        settings.validate().map_err(KernelError::Parameters)?;
        let mut point = physics.clone();
        let lambda = crate::contour::lambda_symbol();
        if point
            .keys()
            .copied()
            .any(crate::contour::is_contour_parameter)
        {
            return Err(KernelError::Parameters(
                "contour strength belongs in contour settings, not the physical parameter map"
                    .into(),
            ));
        }
        if (self.program_recipe().is_dynamic()
            || settings.deformation.program_recipe().is_dynamic())
            && self.program_recipe() != settings.deformation.program_recipe()
        {
            return Err(KernelError::Parameters(
                "contour settings do not match the selected artifact recipe".into(),
            ));
        }
        match (self.contour_capable(), settings.deformation) {
            (false, ContourMode::Off) => self.bind_parameters_raw(&point),
            (false, _) => Err(KernelError::Parameters(
                "artifact has no contour capability; regenerate with contour deformation enabled"
                    .into(),
            )),
            (true, ContourMode::Off) => Err(KernelError::Parameters(
                "contour artifact requires an explicit positive deformation strength".into(),
            )),
            (true, ContourMode::Fixed { lambda: strength }) => {
                point.insert(lambda, strength);
                let binding = ContourBinding::new(self, &point, settings.clone())?;
                self.bind_parameters_raw(&point)?;
                for (index, sector) in self.sectors.iter_mut().enumerate() {
                    sector.contour_validation = binding.for_sector(index);
                }
                self.contour_binding = Some(binding);
                Ok(())
            }
            (
                true,
                ContourMode::Dynamical {
                    safety_fraction,
                    lambda_cap,
                    displacement_cap,
                    ..
                },
            ) => {
                point.insert(
                    crate::contour::dynamic::safety_fraction_symbol(),
                    safety_fraction,
                );
                point.insert(crate::contour::dynamic::lambda_cap_symbol(), lambda_cap);
                point.insert(
                    crate::contour::dynamic::displacement_cap_symbol(),
                    displacement_cap,
                );
                self.bind_dynamic(&point, settings)
            }
        }
    }

    pub(super) fn prepare_contour_checks(&mut self) -> Result<(), KernelError> {
        if self.program_recipe() == super::ProgramRecipe::FixedV1 && self.contour_checks.is_empty()
        {
            let metadata = self.metadata.as_ref().ok_or_else(|| {
                KernelError::Artifact("contour capability requires retained chart metadata".into())
            })?;
            self.contour_checks = build_checks(
                metadata,
                &self.runtime_parameters,
                self.compilation_settings,
            )?;
        }
        Ok(())
    }

    pub fn contour_validation_charts(&self) -> Vec<ContourValidationChart> {
        self.contour_binding
            .as_ref()
            .map_or_else(Vec::new, ContourBinding::charts)
    }

    /// Caller supplies pilot coordinates using its separate validation stream.
    /// Accepted checks never contribute to production integration statistics.
    /// With `homotopy`, the fixed-strength map is also checked at strengths
    /// lambda/8, lambda/4 and lambda/2. These are numerical diagnostics at
    /// finitely many points, not a certificate for the unsampled homotopy.
    /// Only calls with `homotopy=true` count toward completing the preflight
    /// pilot; endpoint-only calls still contribute their observed check counts.
    /// Each arithmetic sign decision uses a certified native ball enclosure.
    /// It encloses the symbolic map at the supplied binary floating inputs;
    /// it does not certify each rounded intermediate of the compiled kernel.
    /// Restrictions use the same full map on the retained subtraction faces.
    pub fn validate_contour_point(
        &mut self,
        chart_index: usize,
        point: &[f64],
        homotopy: bool,
    ) -> Result<ContourCheckReport, KernelError> {
        if self
            .contour_binding
            .as_ref()
            .is_some_and(|binding| binding.dynamic().is_some())
        {
            return self.validate_dynamic_point(chart_index, point, homotopy);
        }
        self.contour_binding
            .as_mut()
            .ok_or_else(|| {
                KernelError::Contour("bind a contour prescription before validation".into())
            })?
            .validate_point(chart_index, point, homotopy)
    }

    pub fn finish_contour_pilot(&mut self) -> Result<ContourValidationReport, KernelError> {
        let report = self
            .contour_binding
            .as_mut()
            .ok_or_else(|| KernelError::Contour("no bound contour prescription".into()))?
            .finish()?;
        self.release_finished_pilot_owners();
        Ok(report)
    }

    /// Accept pilot evidence only for the selected source charts. Other
    /// sectors remain locked; the returned completion describes this scope.
    pub fn finish_contour_pilot_for_charts(
        &mut self,
        charts: &[usize],
    ) -> Result<ContourValidationReport, KernelError> {
        let report = self
            .contour_binding
            .as_mut()
            .ok_or_else(|| KernelError::Contour("no bound contour prescription".into()))?
            .finish_for_charts(charts)?;
        self.release_finished_pilot_owners();
        Ok(report)
    }

    pub fn contour_validation_report(&self) -> Option<ContourValidationReport> {
        self.contour_binding.as_ref().map(|binding| {
            let mut report = binding.report();
            for sector in &self.sectors {
                if let Some(validation) = &sector.contour_validation {
                    let (arguments, bits) = validation.statistics();
                    report.production_checked_arguments += arguments;
                    report.production_maximum_bits = report.production_maximum_bits.max(bits);
                }
                let (arguments, bits) = sector.dynamic_statistics();
                report.production_checked_arguments += arguments;
                report.production_maximum_bits = report.production_maximum_bits.max(bits);
            }
            report
        })
    }

    /// Change checking cost without changing the bound integrand, its content
    /// identity, accumulated statistics or valid pilot evidence.
    pub fn set_contour_validation(
        &mut self,
        validation: ContourValidationOptions,
    ) -> Result<(), KernelError> {
        let binding = self
            .contour_binding
            .as_ref()
            .ok_or_else(|| {
                KernelError::Contour(
                    "bind contour settings before selecting validation policy".into(),
                )
            })?
            .with_policy(self, validation)?;
        if let Some(dynamic) = binding.dynamic() {
            use crate::contour::functions::dynamic::diagnostics::{Accumulator, Phase};
            let configuration =
                super::diagnostics::configuration(self.contour_diagnostics, binding.settings());
            let mut diagnostics = Accumulator::default();
            let prepared = (|| {
                let replacements =
                    diagnostics.measure(configuration, Phase::Preparation, || {
                        self.sectors
                            .iter()
                            .enumerate()
                            .map(|(index, sector)| {
                                sector.prepare_dynamic_mapping(
                                    dynamic.sector_specification(index, false)?,
                                    dynamic.descriptor(),
                                    configuration,
                                )
                            })
                            .collect::<Result<Vec<_>, KernelError>>()
                    })?;
                let exact = if binding.settings().validation.policy
                    == crate::contour::ContourValidation::Always
                {
                    Some(self.dynamic_exact_at(dynamic, configuration, &mut diagnostics)?)
                } else {
                    None
                };
                Ok::<_, KernelError>((replacements, exact))
            })();
            self.runtime_diagnostics.absorb(&diagnostics);
            let (replacements, exact) = prepared?;
            // Preparation and certification must succeed for every owner before
            // replacing any part of the existing usable bound problem.
            for (sector, replacement) in self.sectors.iter_mut().zip(replacements) {
                sector.apply_dynamic_mapping(replacement);
            }
            if let Some(exact) = exact {
                self.exact_coefficients = exact;
            }
            self.collect_pilot_runtime_diagnostics();
            self.dynamic_pilot = Default::default();
        }
        for (index, sector) in self.sectors.iter_mut().enumerate() {
            sector.contour_validation = binding.for_sector(index);
        }
        self.contour_binding = Some(binding);
        Ok(())
    }
}

impl super::SectorKernel {
    /// Drain observational validation counters at a caller-owned work boundary.
    /// Pilot readiness, the mathematical integrand and sampling state are unchanged.
    pub fn take_contour_validation_report(&mut self) -> Option<ContourProductionReport> {
        let report = self.contour_validation_report();
        if let Some(validation) = &mut self.contour_validation {
            validation.take_report();
        }
        self.reset_dynamic_statistics();
        report
    }
    pub fn contour_validation_report(&self) -> Option<ContourProductionReport> {
        let mut report = self
            .contour_validation
            .as_ref()
            .map(SectorValidation::report)
            .or_else(|| {
                self.has_dynamic_callbacks()
                    .then_some(ContourProductionReport {
                        policy: crate::contour::ContourValidation::Off,
                        checked_arguments: 0,
                        maximum_bits: 0,
                    })
            })?;
        let (arguments, bits) = self.dynamic_statistics();
        report.checked_arguments += arguments;
        report.maximum_bits = report.maximum_bits.max(bits);
        Some(report)
    }
}
