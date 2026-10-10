//! Common lifecycle dispatch. Numerical work remains with the native owners.
use super::{
    ContourCheckReport, ContourValidationChart, ContourValidationReport,
    checks::{FixedBinding, SectorValidation},
    dynamic::binding::DynamicBinding,
};
use crate::{
    contour::{ContourSettings, ContourValidationOptions},
    kernel::{KernelError, KernelSet},
    results::ResultScope,
};
use std::collections::BTreeMap;
use symbolica::atom::Symbol;

#[derive(Clone)]
pub(in crate::kernel) enum ContourBinding {
    Fixed(FixedBinding),
    Dynamic(DynamicBinding),
}

impl ContourBinding {
    pub fn new(
        kernels: &KernelSet,
        point: &BTreeMap<Symbol, f64>,
        settings: ContourSettings,
    ) -> Result<Self, KernelError> {
        if settings.deformation.program_recipe().is_dynamic() {
            DynamicBinding::new(kernels, point, settings).map(Self::Dynamic)
        } else {
            FixedBinding::new(kernels, point, settings).map(Self::Fixed)
        }
    }
    pub fn settings(&self) -> &ContourSettings {
        match self {
            Self::Fixed(value) => value.settings(),
            Self::Dynamic(value) => value.settings(),
        }
    }
    pub fn require_ready(&self, scope: &ResultScope) -> Result<(), KernelError> {
        match self {
            Self::Fixed(value) => value.require_ready(scope),
            Self::Dynamic(value) => value.require_ready(scope),
        }
    }
    pub fn for_sector(&self, index: usize) -> Option<SectorValidation> {
        match self {
            Self::Fixed(value) => value.for_sector(index),
            Self::Dynamic(value) => value.for_sector(index),
        }
    }
    pub fn charts(&self) -> Vec<ContourValidationChart> {
        match self {
            Self::Fixed(value) => value.charts(),
            Self::Dynamic(value) => value.charts(),
        }
    }
    pub fn validate_point(
        &mut self,
        chart: usize,
        point: &[f64],
        homotopy: bool,
    ) -> Result<ContourCheckReport, KernelError> {
        match self {
            Self::Fixed(value) => value.validate_point(chart, point, homotopy),
            Self::Dynamic(_) => Err(KernelError::Contour(
                "dynamic pilot requires execution of its native numerical plan".into(),
            )),
        }
    }
    pub fn finish(&mut self) -> Result<ContourValidationReport, KernelError> {
        match self {
            Self::Fixed(value) => value.finish(),
            Self::Dynamic(value) => value.finish(),
        }
    }
    pub fn finish_for_charts(
        &mut self,
        charts: &[usize],
    ) -> Result<ContourValidationReport, KernelError> {
        match self {
            Self::Fixed(value) => value.finish_for_charts(charts),
            Self::Dynamic(value) => value.finish_for_charts(charts),
        }
    }
    pub fn report(&self) -> ContourValidationReport {
        match self {
            Self::Fixed(value) => value.report(),
            Self::Dynamic(value) => value.report(),
        }
    }
    pub fn with_policy(
        &self,
        kernels: &KernelSet,
        validation: ContourValidationOptions,
    ) -> Result<Self, KernelError> {
        match self {
            Self::Fixed(value) => value.with_policy(kernels, validation).map(Self::Fixed),
            Self::Dynamic(value) => value.with_policy(kernels, validation).map(Self::Dynamic),
        }
    }
    pub fn dynamic(&self) -> Option<&DynamicBinding> {
        match self {
            Self::Dynamic(value) => Some(value),
            Self::Fixed(_) => None,
        }
    }
    pub fn dynamic_mut(&mut self) -> Option<&mut DynamicBinding> {
        match self {
            Self::Dynamic(value) => Some(value),
            Self::Fixed(_) => None,
        }
    }
}
