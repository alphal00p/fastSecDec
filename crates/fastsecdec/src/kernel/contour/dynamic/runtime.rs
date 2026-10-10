//! Atomic numerical factory remapping of an already saved native program.
//! Policy changes select callback factories; they never rebuild symbolic IR.
use super::validation::{Coverage, Specification, Validation};
use crate::{
    contour::{ContourSettings, functions::dynamic::requested::Mode},
    kernel::{
        Backend, KernelError, KernelSet, NativeProgramDescriptor, PrecisionClass, SectorKernel,
        evaluator::{self, MappingRequirements},
    },
};
use std::{collections::BTreeMap, sync::Arc};
#[cfg(test)]
mod tests;

/// Pilot-only numeric owners. They are bounded by the currently resident
/// sectors, never cloned with a binding and discarded at a work boundary.
#[derive(Default)]
pub(in crate::kernel) struct PilotOwners {
    sectors: BTreeMap<usize, (SectorKernel, Vec<f64>)>,
    exact: Option<Validation>,
}

pub(in crate::kernel) enum Remapping {
    Unchanged,
    Real(evaluator::RealEvaluator, Arc<MappingRequirements>),
    Complex(evaluator::ComplexEvaluator, Arc<MappingRequirements>),
}
impl SectorKernel {
    pub(in crate::kernel) fn has_dynamic_callbacks(&self) -> bool {
        match &self.backend {
            Backend::Real(kernel) => kernel.evaluator.has_dynamic_callbacks(),
            Backend::Complex(kernel) => kernel.has_dynamic_callbacks(),
        }
    }
    pub(in crate::kernel) fn prepare_dynamic_mapping(
        &self,
        specification: Option<Arc<Specification>>,
        owners: &NativeProgramDescriptor,
    ) -> Result<Remapping, KernelError> {
        let already_plain = match &self.backend {
            Backend::Real(kernel) => kernel.evaluator.validation().is_none(),
            Backend::Complex(kernel) => kernel.dynamic_validation().is_none(),
        };
        if specification.is_none() && already_plain {
            return Ok(Remapping::Unchanged);
        }
        let _owners = owners.enter();
        let _plain = Mode::default().enter();
        let _checking = super::validation::enter_optional(specification);
        let exact = self.exact_program();
        let execution = match &self.backend {
            Backend::Real(kernel) => kernel.evaluator.execution_backend(),
            Backend::Complex(kernel) => kernel.execution_backend(),
        };
        let requirements = MappingRequirements::new(exact).map_err(KernelError::Compilation)?;
        match &self.backend {
            Backend::Real(_) => Ok(Remapping::Real(
                evaluator::real(exact, execution)?,
                requirements,
            )),
            Backend::Complex(_) => Ok(Remapping::Complex(
                evaluator::complex(exact, execution)?,
                requirements,
            )),
        }
    }

    pub(in crate::kernel) fn apply_dynamic_mapping(&mut self, remapping: Remapping) {
        if !matches!(remapping, Remapping::Unchanged) {
            self.dynamic_history = self.dynamic_statistics();
        }
        match (&mut self.backend, remapping) {
            (_, Remapping::Unchanged) => {}
            (Backend::Real(kernel), Remapping::Real(evaluator, requirements)) => {
                kernel.evaluator = evaluator;
                let mut double_cache =
                    crate::kernel::precision_cache::PrecisionCache::new(requirements.clone());
                double_cache.timing = kernel.double_cache.timing;
                kernel.double_cache = double_cache;
                let mut precision_cache =
                    crate::kernel::precision_cache::PrecisionCache::new(requirements.clone());
                precision_cache.timing = kernel.precision_cache.timing;
                kernel.precision_cache = precision_cache;
                kernel.conditioning = evaluator::Conditioning::new(requirements);
            }
            (Backend::Complex(kernel), Remapping::Complex(evaluator, requirements)) => {
                kernel.apply_dynamic_mapping(evaluator, requirements)
            }
            _ => unreachable!("prepared numerical mapping must retain the native output domain"),
        }
    }

    pub(in crate::kernel) fn dynamic_statistics(&self) -> (usize, u32) {
        let mut total = self.dynamic_history;
        let mut visit = |validation: &Validation| {
            total.0 += validation.checked_arguments;
            total.1 = total.1.max(validation.maximum_bits);
        };
        match &self.backend {
            Backend::Real(kernel) => {
                for validation in [
                    kernel.evaluator.validation(),
                    kernel.double_cache.validation.as_ref(),
                    kernel.precision_cache.validation.as_ref(),
                    kernel.conditioning.validation(),
                ]
                .into_iter()
                .flatten()
                {
                    visit(validation);
                }
            }
            Backend::Complex(kernel) => kernel.visit_dynamic_validations(&mut visit),
        }
        total
    }
    pub(in crate::kernel) fn reset_dynamic_statistics(&mut self) {
        self.dynamic_history = (0, 0);
        let mut reset = |validation: &mut Validation| {
            validation.checked_arguments = 0;
            validation.maximum_bits = 0;
        };
        match &mut self.backend {
            Backend::Real(kernel) => {
                for validation in [
                    kernel.evaluator.validation_mut(),
                    kernel.double_cache.validation.as_mut(),
                    kernel.precision_cache.validation.as_mut(),
                    kernel.conditioning.validation_mut(),
                ]
                .into_iter()
                .flatten()
                {
                    reset(validation);
                }
            }
            Backend::Complex(kernel) => kernel.visit_dynamic_validations_mut(&mut reset),
        }
    }
    fn accepted_dynamic_coverage(&self, class: PrecisionClass) -> Result<Coverage, KernelError> {
        let coverage = match &self.backend {
            Backend::Real(kernel) => match class {
                PrecisionClass::F64 => kernel.evaluator.validation(),
                PrecisionClass::DoubleFloat => kernel.double_cache.validation.as_ref(),
                PrecisionClass::Arbitrary => kernel.precision_cache.validation.as_ref(),
                PrecisionClass::Unstable => None,
            }
            .and_then(Validation::coverage),
            Backend::Complex(kernel) => kernel.dynamic_coverage(class),
        };
        coverage.cloned().ok_or_else(|| {
            KernelError::Contour(
                "accepted pilot output lacks its actual candidate certificate".into(),
            )
        })
    }
}

impl KernelSet {
    pub(in crate::kernel) fn bind_dynamic(
        &mut self,
        point: &BTreeMap<symbolica::atom::Symbol, f64>,
        settings: &ContourSettings,
    ) -> Result<(), KernelError> {
        let ordered = self.ordered_runtime_parameters(point)?;
        let binding = crate::kernel::contour::ContourBinding::new(self, point, settings.clone())?;
        let dynamic = binding
            .dynamic()
            .expect("dynamic settings construct a dynamic binding");
        let replacements = self
            .sectors
            .iter()
            .enumerate()
            .map(|(index, sector)| {
                sector.prepare_dynamic_mapping(
                    dynamic.sector_specification(index, false)?,
                    dynamic.descriptor(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let failure_contexts = (0..self.sectors.len())
            .map(|index| {
                let (parameters, bundles) = dynamic.sector_requests(index);
                super::failure::FailureContext::new(
                    dynamic.descriptor(),
                    parameters,
                    bundles,
                    dynamic.runtime_values(),
                    self.precision.max_bits,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let exact = self.dynamic_exact_at(dynamic)?;
        // Every fallible operation has completed. Commit the point, numeric
        // factories and readiness together; failed rebinds leave old owners live.
        self.commit_runtime_parameters(&ordered, exact);
        for (((index, sector), replacement), failure_context) in self
            .sectors
            .iter_mut()
            .enumerate()
            .zip(replacements)
            .zip(failure_contexts)
        {
            sector.apply_dynamic_mapping(replacement);
            sector.reset_dynamic_statistics();
            sector.contour_validation = binding.for_sector(index);
            sector.dynamic_failure = Some(failure_context);
        }
        self.dynamic_pilot = PilotOwners::default();
        self.contour_binding = Some(binding);
        Ok(())
    }

    pub(in crate::kernel) fn dynamic_exact_at(
        &self,
        binding: &super::binding::DynamicBinding,
    ) -> Result<Vec<f64>, KernelError> {
        let _owners = binding.descriptor().enter();
        let complex = self
            .components
            .contains(&crate::status::CoefficientComponent::Imag);
        let result = if let Some(specification) = binding.exact_specification(false)? {
            crate::kernel::exact::evaluate_checked(
                &self.exact_expressions,
                binding.bound_point(),
                complex,
                &self.exact_requests,
                &mut Validation::new(specification),
                binding.runtime_values(),
            )
        } else {
            crate::kernel::exact::evaluate(&self.exact_expressions, binding.bound_point(), complex)
        };
        result.map_err(|error| match error {
            KernelError::NonFinite
            | KernelError::PrecisionExhausted { .. }
            | KernelError::PrecisionEvaluation(_) => KernelError::Contour(format!(
                "unresolved deformation in exact contribution: {error}"
            )),
            _ => error,
        })
    }

    pub(in crate::kernel) fn validate_dynamic_point(
        &mut self,
        chart: usize,
        point: &[f64],
        homotopy: bool,
    ) -> Result<crate::kernel::ContourCheckReport, KernelError> {
        use super::binding::PilotWork;
        let binding = self
            .contour_binding
            .as_ref()
            .and_then(|binding| binding.dynamic())
            .ok_or_else(|| KernelError::Contour("no dynamic binding".into()))?;
        let plan = binding.plan(chart, point, homotopy)?;
        let mut accepted_exact = None;
        let receipt = plan.execute(|work| match work {
            PilotWork::Sector(work) => {
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    self.dynamic_pilot.sectors.entry(work.sector_index)
                {
                    let original = self.sectors.get(work.sector_index).ok_or_else(|| {
                        KernelError::Contour("pilot references a nonresident sector".into())
                    })?;
                    let mut sector = original.try_clone()?;
                    let replacement = sector.prepare_dynamic_mapping(
                        Some(work.specification.clone()),
                        binding.descriptor(),
                    )?;
                    sector.apply_dynamic_mapping(replacement);
                    sector.reset_dynamic_statistics();
                    // Only this private pilot owner bypasses pilot readiness;
                    // all point, callback, precision and certificate checks stay.
                    sector.contour_validation = None;
                    let output = vec![0.0; sector.output_count()];
                    entry.insert((sector, output));
                }
                let (sector, output) = self
                    .dynamic_pilot
                    .sectors
                    .get_mut(&work.sector_index)
                    .expect("pilot owner inserted");
                let report = sector.evaluate_with_diagnostics(&work.point, output)?;
                sector.accepted_dynamic_coverage(report.class)
            }
            PilotWork::Exact(work) => {
                let validation = self
                    .dynamic_pilot
                    .exact
                    .get_or_insert_with(|| Validation::new(work.specification.clone()));
                let _owners = binding.descriptor().enter();
                accepted_exact = Some(crate::kernel::exact::evaluate_checked(
                    &self.exact_expressions,
                    &work.point,
                    self.components
                        .contains(&crate::status::CoefficientComponent::Imag),
                    &self.exact_requests,
                    validation,
                    binding.runtime_values(),
                )?);
                validation.coverage().cloned().ok_or_else(|| {
                    KernelError::Contour("exact pilot output lacks candidate coverage".into())
                })
            }
        })?;
        let report = self
            .contour_binding
            .as_mut()
            .and_then(|binding| binding.dynamic_mut())
            .expect("binding unchanged while synchronous pilot executes")
            .accept(plan, receipt)?;
        if let Some(exact) = accepted_exact {
            self.exact_coefficients = exact;
        }
        Ok(report)
    }

    pub(in crate::kernel) fn release_finished_pilot_owners(&mut self) {
        let Some(binding) = self
            .contour_binding
            .as_ref()
            .and_then(|binding| binding.dynamic())
        else {
            return;
        };
        let charts = binding.charts();
        // A shared sector remains cached until every context requiring it has
        // finished. Context-only exact ownership follows the same rule.
        self.dynamic_pilot.sectors.retain(|index, _| {
            charts.iter().any(|chart| {
                chart.kernel_sectors.contains(index) && !binding.chart_ready(chart.chart_index)
            })
        });
        if charts
            .iter()
            .filter(|chart| chart.includes_exact)
            .all(|chart| binding.chart_ready(chart.chart_index))
        {
            self.dynamic_pilot.exact = None;
        }
    }
}
