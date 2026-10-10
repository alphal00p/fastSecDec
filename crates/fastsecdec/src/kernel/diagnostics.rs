//! Operational factory selection and atomic report drains. Saved expressions,
//! mathematical identities and caller-owned sampling state never change here.
use super::{Backend, KernelError, KernelSet, SectorKernel};
use crate::{
    contour::{
        ContourDiagnosticsMode, ContourMode, ContourRuntimeReport, ContourSettings,
        functions::dynamic::diagnostics::{Accumulator, Configuration, Phase},
    },
    status::DiagnosticsOverflow,
};

pub(super) fn configuration(
    mode: ContourDiagnosticsMode,
    settings: &ContourSettings,
) -> Configuration {
    Configuration {
        mode,
        displacement_cap: match settings.deformation {
            ContourMode::Dynamical {
                displacement_cap, ..
            } => displacement_cap,
            _ => 1.,
        },
    }
}
impl SectorKernel {
    pub(super) fn runtime_configuration(&self) -> Configuration {
        match &self.backend {
            Backend::Real(kernel) => kernel.evaluator.diagnostics_configuration(),
            Backend::Complex(kernel) => kernel.diagnostics_configuration(),
        }
    }
    pub(super) fn aggregate_runtime_diagnostics(&self) -> Accumulator {
        let mut total = Accumulator::default();
        total.absorb(&self.runtime_diagnostics);
        let mut visit = |owner: &Accumulator| total.absorb(owner);
        match &self.backend {
            Backend::Real(kernel) => {
                if let Some(owner) = kernel.evaluator.runtime_diagnostics() {
                    visit(owner);
                }
                visit(&kernel.double_cache.runtime_diagnostics);
                visit(&kernel.precision_cache.runtime_diagnostics);
                kernel.conditioning.visit_runtime_diagnostics(&mut visit);
            }
            Backend::Complex(kernel) => kernel.visit_runtime_diagnostics(&mut visit),
        }
        total
    }
    pub(super) fn clear_runtime_diagnostics(&mut self) {
        self.runtime_diagnostics.clear();
        match &mut self.backend {
            Backend::Real(kernel) => {
                kernel.evaluator.clear_runtime_diagnostics();
                kernel.double_cache.runtime_diagnostics.clear();
                kernel.precision_cache.runtime_diagnostics.clear();
                kernel.conditioning.clear_runtime_diagnostics();
            }
            Backend::Complex(kernel) => kernel.clear_runtime_diagnostics(),
        }
    }
    /// Snapshot actual callback work without consuming counters. None means no
    /// aggregate observation was made, rather than an inferred zero count.
    pub fn contour_runtime_report(
        &self,
    ) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        self.aggregate_runtime_diagnostics().snapshot()
    }
    /// Drain only after a successful checked aggregate merge. Overflow leaves
    /// all counters intact and never changes numerical acceptance.
    pub fn take_contour_runtime_report(
        &mut self,
    ) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        let report = self.contour_runtime_report()?;
        self.clear_runtime_diagnostics();
        Ok(report)
    }
}
impl KernelSet {
    pub fn contour_diagnostics_mode(&self) -> ContourDiagnosticsMode {
        self.contour_diagnostics
    }
    /// Select optional observations independently of validation. Numeric owners
    /// are replaced atomically from saved IR; no generation or optimization is
    /// repeated. Unbound owners retain the choice for their next binding.
    pub fn set_contour_diagnostics(
        &mut self,
        mode: ContourDiagnosticsMode,
    ) -> Result<(), KernelError> {
        if mode == self.contour_diagnostics {
            return Ok(());
        }
        if let Some(binding) = self
            .contour_binding
            .as_ref()
            .filter(|b| b.dynamic().is_some())
        {
            let dynamic = binding.dynamic().unwrap();
            let config = configuration(mode, binding.settings());
            let mut preparation = Accumulator::default();
            let replacements = preparation.measure(config, Phase::Preparation, || {
                self.sectors
                    .iter()
                    .enumerate()
                    .map(|(index, sector)| {
                        sector.prepare_dynamic_mapping(
                            dynamic.sector_specification(index, false)?,
                            dynamic.descriptor(),
                            config,
                        )
                    })
                    .collect::<Result<Vec<_>, KernelError>>()
            });
            self.runtime_diagnostics.absorb(&preparation);
            let replacements = replacements?;
            for (sector, replacement) in self.sectors.iter_mut().zip(replacements) {
                sector.apply_dynamic_mapping(replacement);
            }
            self.collect_pilot_runtime_diagnostics();
            self.dynamic_pilot = Default::default();
        }
        self.contour_diagnostics = mode;
        Ok(())
    }
    fn aggregate_runtime_diagnostics(&self) -> Accumulator {
        let mut total = Accumulator::default();
        total.absorb(&self.runtime_diagnostics);
        for sector in &self.sectors {
            total.absorb(&sector.aggregate_runtime_diagnostics());
        }
        for (sector, _) in self.dynamic_pilot.sectors.values() {
            total.absorb_as_pilot(&sector.aggregate_runtime_diagnostics());
        }
        total
    }
    pub(super) fn collect_pilot_runtime_diagnostics(&mut self) {
        for (sector, _) in self.dynamic_pilot.sectors.values_mut() {
            self.runtime_diagnostics
                .absorb_as_pilot(&sector.aggregate_runtime_diagnostics());
            sector.clear_runtime_diagnostics();
        }
    }
    pub fn contour_runtime_report(
        &self,
    ) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        self.aggregate_runtime_diagnostics().snapshot()
    }
    pub fn take_contour_runtime_report(
        &mut self,
    ) -> Result<Option<ContourRuntimeReport>, DiagnosticsOverflow> {
        let report = self.contour_runtime_report()?;
        self.runtime_diagnostics.clear();
        for sector in &mut self.sectors {
            sector.clear_runtime_diagnostics();
        }
        for (sector, _) in self.dynamic_pilot.sectors.values_mut() {
            sector.clear_runtime_diagnostics();
        }
        Ok(report)
    }
}
