//! Pilot coverage and readiness for saved dynamic source contexts. This module
//! schedules no sampling and performs no numerical solve or evaluator build.
use super::validation::{Coverage, Specification};
use crate::{
    contour::{
        ContourSettings, ContourValidation, ContourValidationOptions,
        functions::dynamic::{requested, requests::Bundle},
    },
    kernel::{
        KernelError, KernelSet, NativeProgramDescriptor, PrecisionPolicy,
        contour::{
            ContourCheckReport, ContourValidationChart, ContourValidationReport, SectorValidation,
        },
        program,
    },
    results::ResultScope,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::atom::Symbol;

#[derive(Clone)]
struct Sector {
    parameters: Vec<Symbol>,
    bundles: BTreeSet<Bundle>,
    specification: OnceLock<Result<Arc<Specification>, String>>,
}
struct Context {
    certificate_index: usize,
    namespace: String,
    chart: ContourValidationChart,
    coordinates: Vec<Symbol>,
    ready: Arc<AtomicBool>,
    pilot_points: usize,
    checked_arguments: usize,
    maximum_bits: u32,
}
impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            certificate_index: self.certificate_index,
            namespace: self.namespace.clone(),
            chart: self.chart.clone(),
            coordinates: self.coordinates.clone(),
            ready: Arc::new(AtomicBool::new(self.ready.load(Ordering::Acquire))),
            pilot_points: self.pilot_points,
            checked_arguments: self.checked_arguments,
            maximum_bits: self.maximum_bits,
        }
    }
}

pub(in crate::kernel) struct DynamicBinding {
    settings: ContourSettings,
    point: Arc<BTreeMap<Symbol, f64>>,
    descriptor: Arc<NativeProgramDescriptor>,
    sectors: Vec<Sector>,
    exact_bundles: BTreeSet<Bundle>,
    exact_specification: OnceLock<Result<Option<Arc<Specification>>, String>>,
    runtime: Vec<f64>,
    precision: PrecisionPolicy,
    contexts: Vec<Context>,
    epoch: Arc<()>,
}
impl Clone for DynamicBinding {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            point: self.point.clone(),
            descriptor: self.descriptor.clone(),
            sectors: self.sectors.clone(),
            exact_bundles: self.exact_bundles.clone(),
            exact_specification: self.exact_specification.clone(),
            runtime: self.runtime.clone(),
            precision: self.precision.clone(),
            contexts: self.contexts.clone(),
            epoch: Arc::new(()),
        }
    }
}

mod pilot;
use pilot::{PilotExact, PilotSector};
pub(in crate::kernel) use pilot::{PilotPlan, PilotReceipt, PilotWork};

fn invalid(reason: impl Into<String>) -> KernelError {
    KernelError::Contour(reason.into())
}

fn sector_requests(callbacks: Vec<program::Callback>) -> Result<BTreeSet<Bundle>, KernelError> {
    let mut bundles = BTreeSet::new();
    for callback in callbacks {
        if callback.symbol == crate::contour::functions::dynamic::symbol() {
            return Err(invalid(
                "dynamic stochastic program contains an unassociated mathematical root",
            ));
        }
        if callback.symbol == requested::symbol() {
            if callback.fixed_args.is_some() || callback.tags.len() != 3 {
                return Err(invalid(
                    "dynamic stochastic request needs three tags and runtime arguments",
                ));
            }
            bundles.insert(Bundle::from_atom(callback.tags[2].as_view()).map_err(invalid)?);
        }
    }
    Ok(bundles)
}

impl DynamicBinding {
    pub fn new(
        kernels: &KernelSet,
        point: &BTreeMap<Symbol, f64>,
        settings: ContourSettings,
    ) -> Result<Self, KernelError> {
        settings.validate().map_err(KernelError::Parameters)?;
        let descriptor = kernels
            .program_descriptor
            .as_ref()
            .ok_or_else(|| invalid("dynamic binding lacks its native descriptor"))?;
        if !descriptor.recipe().is_dynamic()
            || descriptor.recipe() != settings.deformation.program_recipe()
        {
            return Err(invalid(
                "dynamic binding and saved mathematical recipe differ",
            ));
        }
        let certificates = descriptor
            .certificates()
            .ok_or_else(|| invalid("dynamic artifact lacks saved certificates; regenerate"))?;
        descriptor.validate_certificate_runtime(&kernels.runtime_parameters)?;
        let runtime = kernels
            .runtime_parameters
            .iter()
            .map(|symbol| {
                point
                    .get(symbol)
                    .copied()
                    .ok_or_else(|| KernelError::Parameters(format!("missing value for {symbol}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sectors = kernels
            .sectors
            .iter()
            .map(|sector| {
                let bundles =
                    sector_requests(program::callbacks(sector.exact_program()).map_err(invalid)?)?;
                Ok(Sector {
                    parameters: sector.parameters.clone(),
                    bundles,
                    specification: OnceLock::new(),
                })
            })
            .collect::<Result<Vec<_>, KernelError>>()?;
        let exact_bundles = kernels
            .exact_requests
            .iter()
            .map(|request| request.bundle.clone())
            .collect::<BTreeSet<_>>();
        descriptor.validate_request_coverage(
            sectors
                .iter()
                .flat_map(|sector| &sector.bundles)
                .chain(&exact_bundles),
        )?;
        let exact_namespaces = exact_bundles
            .iter()
            .flat_map(|bundle| bundle.0.iter().map(|request| request.namespace.clone()))
            .collect::<BTreeSet<_>>();
        let mut next = certificates
            .iter()
            .filter_map(|c| c.chart_index)
            .max()
            .map_or(Ok(0), |index| {
                index
                    .checked_add(1)
                    .ok_or_else(|| invalid("dynamic reporting chart index overflow"))
            })?;
        let mut contexts = Vec::new();
        for (certificate_index, certificate) in certificates.iter().enumerate() {
            let kernel_sectors = sectors
                .iter()
                .enumerate()
                .filter_map(|(index, sector)| {
                    sector
                        .bundles
                        .iter()
                        .any(|bundle| {
                            bundle
                                .0
                                .iter()
                                .any(|request| request.namespace == certificate.namespace)
                        })
                        .then_some(index)
                })
                .collect::<Vec<_>>();
            let includes_exact = exact_namespaces.contains(&certificate.namespace);
            if kernel_sectors.is_empty() && !includes_exact {
                continue;
            }
            let chart_index = match certificate.chart_index {
                Some(index) => index,
                None => {
                    let index = next;
                    next = next
                        .checked_add(1)
                        .ok_or_else(|| invalid("dynamic reporting chart index overflow"))?;
                    index
                }
            };
            let coordinates = certificate
                .coordinates
                .iter()
                .map(|name| {
                    Symbol::parse(name, "fastsecdec::artifact").map_err(KernelError::Artifact)
                })
                .collect::<Result<Vec<_>, _>>()?;
            contexts.push(Context {
                certificate_index,
                namespace: certificate.namespace.clone(),
                chart: ContourValidationChart {
                    chart_index,
                    kernel_sector: (kernel_sectors.len() == 1).then(|| kernel_sectors[0]),
                    kernel_sectors,
                    includes_exact,
                    dimension: coordinates.len(),
                },
                coordinates,
                ready: Arc::new(AtomicBool::new(false)),
                pilot_points: 0,
                checked_arguments: 0,
                maximum_bits: 0,
            });
        }
        contexts.sort_by_key(|context| context.chart.chart_index);
        if contexts
            .windows(2)
            .any(|pair| pair[0].chart.chart_index == pair[1].chart.chart_index)
        {
            return Err(invalid("dynamic source reporting IDs are not unique"));
        }
        for namespace in sectors
            .iter()
            .flat_map(|sector| sector.bundles.iter())
            .flat_map(|bundle| bundle.0.iter().map(|request| &request.namespace))
            .chain(&exact_namespaces)
        {
            if !contexts
                .iter()
                .any(|context| &context.namespace == namespace)
            {
                return Err(invalid(
                    "surviving dynamic request lacks its source context",
                ));
            }
        }
        Ok(Self {
            settings,
            point: Arc::new(point.clone()),
            descriptor: Arc::new(descriptor.clone()),
            sectors,
            exact_bundles,
            exact_specification: OnceLock::new(),
            runtime,
            precision: kernels.precision.clone(),
            contexts,
            epoch: Arc::new(()),
        })
    }
    pub fn settings(&self) -> &ContourSettings {
        &self.settings
    }
    pub fn bound_point(&self) -> &BTreeMap<Symbol, f64> {
        &self.point
    }
    pub fn runtime_values(&self) -> &[f64] {
        &self.runtime
    }
    pub fn descriptor(&self) -> &NativeProgramDescriptor {
        &self.descriptor
    }
    pub fn sector_requests(&self, index: usize) -> (&[Symbol], &BTreeSet<Bundle>) {
        let sector = &self.sectors[index];
        (&sector.parameters, &sector.bundles)
    }
    pub fn charts(&self) -> Vec<ContourValidationChart> {
        self.contexts
            .iter()
            .map(|context| context.chart.clone())
            .collect()
    }
    pub fn chart_ready(&self, chart_index: usize) -> bool {
        self.contexts.iter().any(|context| {
            context.chart.chart_index == chart_index && context.ready.load(Ordering::Acquire)
        })
    }
    pub fn require_ready(&self, scope: &ResultScope) -> Result<(), KernelError> {
        if self.settings.validation.policy == ContourValidation::Off {
            return Ok(());
        }
        for context in &self.contexts {
            if context.chart.required_by_scope(scope) && !context.ready.load(Ordering::Acquire) {
                return Err(invalid(format!(
                    "complete the configured contour pilot for chart {} before integration",
                    context.chart.chart_index
                )));
            }
        }
        Ok(())
    }
    pub fn for_sector(&self, index: usize) -> Option<SectorValidation> {
        (self.settings.validation.policy != ContourValidation::Off).then(|| {
            SectorValidation::Dynamic {
                policy: self.settings.validation.policy,
                ready: self
                    .contexts
                    .iter()
                    .filter(|context| context.chart.kernel_sectors.contains(&index))
                    .map(|context| context.ready.clone())
                    .collect(),
            }
        })
    }
    pub fn sector_specification(
        &self,
        index: usize,
        force_checked: bool,
    ) -> Result<Option<Arc<Specification>>, KernelError> {
        if !force_checked && self.settings.validation.policy != ContourValidation::Always {
            return Ok(None);
        }
        let sector = self
            .sectors
            .get(index)
            .ok_or_else(|| invalid("unknown dynamic pilot sector"))?;
        if sector.bundles.is_empty() {
            return Ok(None);
        }
        sector
            .specification
            .get_or_init(|| {
                Specification::new(
                    &self.descriptor,
                    &sector.parameters,
                    self.runtime.clone(),
                    self.settings.deformation,
                    sector.bundles.clone(),
                    &self.precision,
                )
                .map_err(|error| error.to_string())
            })
            .clone()
            .map(Some)
            .map_err(invalid)
    }
    pub fn exact_specification(
        &self,
        force_checked: bool,
    ) -> Result<Option<Arc<Specification>>, KernelError> {
        if !force_checked && self.settings.validation.policy != ContourValidation::Always {
            return Ok(None);
        }
        self.exact_specification
            .get_or_init(|| {
                if self.exact_bundles.is_empty() {
                    Ok(None)
                } else {
                    Specification::new(
                        &self.descriptor,
                        &[],
                        self.runtime.clone(),
                        self.settings.deformation,
                        self.exact_bundles.clone(),
                        &self.precision,
                    )
                    .map(Some)
                    .map_err(|error| error.to_string())
                }
            })
            .clone()
            .map_err(invalid)
    }
    pub fn plan(
        &self,
        chart_index: usize,
        point: &[f64],
        homotopy: bool,
    ) -> Result<PilotPlan, KernelError> {
        if self.settings.validation.policy == ContourValidation::Off {
            return Err(invalid(
                "validation is disabled; no new pilot evidence is collected",
            ));
        }
        let context = self
            .contexts
            .iter()
            .find(|context| context.chart.chart_index == chart_index)
            .ok_or_else(|| invalid(format!("unknown validation chart {chart_index}")))?;
        if point.len() != context.coordinates.len() {
            return Err(KernelError::Dimension {
                expected: context.coordinates.len(),
                actual: point.len(),
            });
        }
        if point
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(KernelError::InvalidPoint);
        }
        let sectors = context
            .chart
            .kernel_sectors
            .iter()
            .map(|&sector_index| {
                let point = self.sectors[sector_index]
                    .parameters
                    .iter()
                    .map(|symbol| {
                        context
                            .coordinates
                            .iter()
                            .position(|coordinate| coordinate == symbol)
                            .map(|index| point[index])
                            .ok_or_else(|| {
                                invalid(
                                    "pilot source cannot project every native integration input",
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(PilotSector {
                    sector_index,
                    point,
                    specification: self
                        .sector_specification(sector_index, true)?
                        .ok_or_else(|| invalid("pilot sector lacks its checked specification"))?,
                })
            })
            .collect::<Result<Vec<_>, KernelError>>()?;
        Ok(PilotPlan {
            chart_index,
            sectors,
            exact: if context.chart.includes_exact {
                Some(PilotExact {
                    point: self.point.clone(),
                    specification: self.exact_specification(true)?.ok_or_else(|| {
                        invalid("pilot exact work lacks its checked specification")
                    })?,
                })
            } else {
                None
            },
            homotopy,
            epoch: self.epoch.clone(),
            nonce: Arc::new(()),
        })
    }
    pub fn accept(
        &mut self,
        plan: PilotPlan,
        receipt: PilotReceipt,
    ) -> Result<ContourCheckReport, KernelError> {
        if !Arc::ptr_eq(&plan.epoch, &self.epoch) {
            return Err(invalid("stale or foreign dynamic pilot plan"));
        }
        if !Arc::ptr_eq(&plan.nonce, &receipt.nonce) {
            return Err(invalid(
                "dynamic pilot receipt belongs to another point or plan",
            ));
        }
        let expected = plan
            .sectors
            .iter()
            .map(|sector| sector.sector_index)
            .collect::<BTreeSet<_>>();
        let actual = receipt
            .sectors
            .iter()
            .map(|(sector, _)| *sector)
            .collect::<BTreeSet<_>>();
        if expected != actual
            || actual.len() != receipt.sectors.len()
            || plan.exact.is_some() != receipt.exact.is_some()
        {
            return Err(invalid(
                "dynamic pilot receipt omits or duplicates required numerical work",
            ));
        }
        for (index, coverage) in &receipt.sectors {
            if coverage.bundles != self.sectors[*index].bundles {
                return Err(invalid(
                    "dynamic pilot did not certify every required sector request",
                ));
            }
        }
        let mut updates = BTreeMap::<usize, (usize, u32)>::new();
        for coverage in receipt
            .sectors
            .iter()
            .map(|(_, coverage)| coverage)
            .chain(receipt.exact.iter())
        {
            let arguments = coverage.contexts.iter().try_fold(0usize, |sum, context| {
                sum.checked_add(context.checked_arguments)
                    .ok_or_else(|| invalid("pilot check count overflow"))
            })?;
            let bits = coverage
                .contexts
                .iter()
                .map(|context| context.maximum_bits)
                .max()
                .unwrap_or(0);
            if arguments != coverage.checked_arguments || bits != coverage.maximum_bits {
                return Err(invalid(
                    "pilot coverage totals differ from its source contexts",
                ));
            }
            let mut seen = BTreeSet::new();
            for checked in &coverage.contexts {
                if !seen.insert(checked.certificate_index) {
                    return Err(invalid("pilot coverage repeats a source context"));
                }
                let index = self
                    .contexts
                    .iter()
                    .position(|context| {
                        context.certificate_index == checked.certificate_index
                            && context.namespace == checked.namespace
                    })
                    .ok_or_else(|| invalid("pilot coverage refers to a foreign source context"))?;
                let certificate =
                    &self.descriptor.certificates().unwrap()[checked.certificate_index];
                if checked.chart_index != certificate.chart_index {
                    return Err(invalid(
                        "pilot coverage has a different source chart projection",
                    ));
                }
                let entry = updates.entry(index).or_default();
                entry.0 = entry
                    .0
                    .checked_add(checked.checked_arguments)
                    .ok_or_else(|| invalid("pilot check count overflow"))?;
                entry.1 = entry.1.max(checked.maximum_bits);
            }
        }
        let selected = self
            .contexts
            .iter()
            .position(|context| context.chart.chart_index == plan.chart_index)
            .ok_or_else(|| invalid("pilot reporting context disappeared"))?;
        if plan.homotopy && self.contexts[selected].pilot_points == usize::MAX {
            return Err(invalid("pilot point count overflow"));
        }
        let mut report = ContourCheckReport {
            chart_index: plan.chart_index,
            ..Default::default()
        };
        for (&index, &(count, bits)) in &updates {
            self.contexts[index]
                .checked_arguments
                .checked_add(count)
                .ok_or_else(|| invalid("pilot check count overflow"))?;
            report.checked_arguments = report
                .checked_arguments
                .checked_add(count)
                .ok_or_else(|| invalid("pilot check count overflow"))?;
            report.maximum_bits = report.maximum_bits.max(bits);
        }
        for (index, (count, bits)) in updates {
            self.contexts[index].checked_arguments += count;
            self.contexts[index].maximum_bits = self.contexts[index].maximum_bits.max(bits);
        }
        if plan.homotopy {
            self.contexts[selected].pilot_points += 1;
        }
        Ok(report)
    }
    pub fn finish(&mut self) -> Result<ContourValidationReport, KernelError> {
        self.finish_for_charts(
            &self
                .contexts
                .iter()
                .map(|context| context.chart.chart_index)
                .collect::<Vec<_>>(),
        )
    }
    pub fn finish_for_charts(
        &mut self,
        charts: &[usize],
    ) -> Result<ContourValidationReport, KernelError> {
        if self.settings.validation.policy == ContourValidation::Off {
            return Err(invalid(
                "validation is disabled; no new pilot evidence was collected",
            ));
        }
        for chart in charts {
            let context = self
                .contexts
                .iter()
                .find(|context| context.chart.chart_index == *chart)
                .ok_or_else(|| invalid(format!("unknown validation chart {chart}")))?;
            if context.pilot_points < self.settings.validation.pilot_points {
                return Err(invalid(format!(
                    "pilot lacks the requested number of validated points on chart {chart}"
                )));
            }
        }
        for context in &self.contexts {
            if charts.contains(&context.chart.chart_index) {
                context.ready.store(true, Ordering::Release);
            }
        }
        Ok(self.report_for(charts))
    }
    pub fn report(&self) -> ContourValidationReport {
        self.report_for(
            &self
                .contexts
                .iter()
                .map(|context| context.chart.chart_index)
                .collect::<Vec<_>>(),
        )
    }
    fn report_for(&self, charts: &[usize]) -> ContourValidationReport {
        let contexts = self
            .contexts
            .iter()
            .filter(|context| charts.contains(&context.chart.chart_index))
            .collect::<Vec<_>>();
        let required_charts = contexts
            .iter()
            .map(|context| context.chart.chart_index)
            .collect::<Vec<_>>();
        let validated_charts = contexts
            .iter()
            .filter(|context| context.ready.load(Ordering::Acquire))
            .map(|context| context.chart.chart_index)
            .collect::<Vec<_>>();
        ContourValidationReport {
            policy: self.settings.validation.policy,
            pilot_complete: (self.settings.validation.policy != ContourValidation::Off
                || !required_charts.is_empty())
                && required_charts.len() == validated_charts.len(),
            required_charts,
            validated_charts,
            accepted_pilot_points: contexts.iter().map(|context| context.pilot_points).sum(),
            checked_arguments: contexts
                .iter()
                .map(|context| context.checked_arguments)
                .sum(),
            maximum_bits: contexts
                .iter()
                .map(|context| context.maximum_bits)
                .max()
                .unwrap_or(0),
            production_checked_arguments: 0,
            production_maximum_bits: 0,
        }
    }
    pub fn with_policy(
        &self,
        _kernels: &KernelSet,
        validation: ContourValidationOptions,
    ) -> Result<Self, KernelError> {
        let mut result = self.clone();
        result.settings.validation = validation;
        result
            .settings
            .validate()
            .map_err(KernelError::Parameters)?;
        for context in &result.contexts {
            if context.pilot_points < result.settings.validation.pilot_points {
                context.ready.store(false, Ordering::Release);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
