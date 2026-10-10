//! Generation-time preparation of late request metadata and independent saved
//! certificate programs. Loading and sampling never call this module.
use super::*;
use crate::contour::functions::dynamic::requests::{ExactRequest, Lookup};
use crate::kernel::{NativeProgramDescriptor, recipe::DynamicCheckProgram};
use std::sync::Arc;

pub(in crate::kernel) struct PreparedDynamic {
    pub descriptor: Arc<NativeProgramDescriptor>,
    pub lookup: Arc<Lookup>,
    pub exact: Vec<Atom>,
    pub exact_requests: Vec<ExactRequest>,
}

impl PreparedDynamic {
    pub(in crate::kernel) fn build(
        generated: &GeneratedIntegral,
        runtime: &[Symbol],
        settings: CompilationSettings,
    ) -> Result<Option<Self>, KernelError> {
        let Some(descriptor) = generated
            .program_descriptor()
            .filter(|descriptor| descriptor.recipe().is_dynamic())
        else {
            return Ok(None);
        };
        validate_descriptor(generated, runtime)?;
        let mut lookup = Lookup::default();
        let mut certificates = Vec::new();
        for source in generated.dynamic_check_sources() {
            let chart = descriptor
                .charts()
                .iter()
                .find(|chart| chart.chart_index == source.chart_index)
                .ok_or_else(|| {
                    KernelError::Compilation("dynamic source lacks chart proof".into())
                })?;
            let contour = generated
                .metadata()
                .charts()
                .iter()
                .find(|chart| chart.source_index() == source.chart_index)
                .and_then(|chart| chart.contour())
                .ok_or_else(|| {
                    KernelError::Compilation("dynamic source lacks retained map".into())
                })?;
            lookup
                .add_definitions(contour.function_definitions())
                .map_err(KernelError::Compilation)?;
            lookup
                .insert(
                    &source.namespace,
                    &source.full_strength,
                    &source.parameters,
                    contour.validation_faces(),
                )
                .map_err(KernelError::Compilation)?;
            certificates.push(DynamicCheckProgram::build(
                source, chart, contour, runtime, settings,
            )?);
        }
        // Retain native mathematical equality across later record sums. Exact
        // callbacks are cached and observed through separate associations at
        // binding, so diagnostic namespaces cannot inhibit cancellation.
        let exact = generated.exact_coefficients().to_vec();
        let exact_requests = lookup
            .exact_requests(&exact)
            .map_err(KernelError::Compilation)?;
        let descriptor = descriptor
            .as_ref()
            .clone()
            .with_certificates(certificates)?;
        descriptor.validate_generation(Some(generated.metadata()), runtime)?;
        Ok(Some(Self {
            descriptor: Arc::new(descriptor),
            lookup: Arc::new(lookup),
            exact,
            exact_requests,
        }))
    }
}
