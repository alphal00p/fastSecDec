//! Authoritative initial rational-fiber factory. All native jobs are caller-run;
//! this module creates no worker pool or numerical integration loop.
use super::*;
use crate::{
    generation,
    threshold::{records as record, regularization::BoundContinuation},
};
use std::{path::Path, sync::Arc};
use symbolica::atom::{AliasedAtom, Atom};

impl KernelSet {
    /// Compile the complete currently certified rational interval family.
    /// The caller supplies a fresh native staging directory and executes each
    /// ordinary native CompilationJob synchronously. Global preparation still
    /// retains the certificate; this does not claim bounded global CAD memory.
    pub fn compile_threshold_fiber(
        bound: &BoundContinuation<'_>,
        staging: &Path,
        maximum: i32,
        precision: PrecisionPolicy,
        settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        settings.validate()?;
        if bound.generation_options().mode != generation::GenerationMode::Symbolic {
            return Err(KernelError::Artifact(
                "threshold factory requires certified symbolic continuation".into(),
            ));
        }
        if maximum > bound.generation_options().max_order {
            return Err(KernelError::Artifact(
                "requested Laurent range exceeds the continued request".into(),
            ));
        }
        if staging
            .read_dir()
            .map_err(|e| KernelError::Artifact(e.to_string()))?
            .next()
            .is_some()
        {
            return Err(KernelError::Artifact(
                "threshold staging directory must be empty".into(),
            ));
        }
        let request = bound.certificate().decomposition().request();
        let parent = generation::source_identity(
            request.input(),
            &request.kinematics().runtime_parameters,
            &[],
        )
        .map_err(|e| KernelError::Artifact(e.to_string()))?;
        let records = (0..bound.chart_expressions().len())
            .map(|chart| {
                record::write(staging, &parent, bound, chart, maximum)
                    .map_err(|e| KernelError::Artifact(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let minimum = records
            .iter()
            .map(|r| r.minimum)
            .min()
            .unwrap_or(maximum)
            .min(maximum.min(0));
        let orders = (minimum..=maximum).collect::<Vec<_>>();
        let metadata = super::super::threshold_owner::ThresholdMetadata::from_bound(
            bound,
            &records,
            orders.clone(),
        )?;
        let mut sectors = Vec::new();
        let mut exact = vec![Atom::Zero; orders.len()];
        let owner = Arc::new(());
        for record in &records {
            let local = record::read(staging, record, &parent, u64::MAX)
                .map_err(|e| KernelError::Artifact(e.to_string()))?;
            let coefficients = orders
                .iter()
                .map(|o| {
                    local
                        .coefficients
                        .get(o)
                        .cloned()
                        .unwrap_or_else(|| AliasedAtom::from(Atom::Zero))
                })
                .collect::<Vec<_>>();
            match local.kind {
                record::VectorKind::ZeroInLayout {} => {}
                record::VectorKind::Exact {} => {
                    for (sum, value) in exact.iter_mut().zip(coefficients) {
                        // Record production already admitted/materialized exact
                        // coefficients. It stores no unresolved alias graph here.
                        if !value.get_aliases().is_empty() {
                            return Err(KernelError::Artifact(
                                "exact threshold record has aliases".into(),
                            ));
                        }
                        *sum += value.get_root();
                    }
                }
                record::VectorKind::Stochastic {} => {
                    let input = program::PreparedCoefficientVector {
                        coordinates: local.coordinates,
                        coefficients,
                        functions: Arc::new(local.functions),
                        endpoint_profiles: local.profiles,
                    };
                    let job = CompilationJob {
                        owner: owner.clone(),
                        program_descriptor: None,
                        request_lookup: None,
                        index: sectors.len(),
                        input: CompilationInput::Prepared(Arc::new(input)),
                        runtime_parameters: Arc::new(Vec::new()),
                        precision: precision.clone(),
                        settings,
                        use_complex: true,
                    };
                    let completion = job.run()?;
                    if !Arc::ptr_eq(&completion.owner, &owner) || completion.index != sectors.len()
                    {
                        return Err(KernelError::Artifact(
                            "threshold compilation job association".into(),
                        ));
                    }
                    sectors.push(completion.sector);
                }
            }
        }
        let exact = exact
            .iter()
            .map(generation::normalize_exact_coefficient)
            .collect();
        let mut kernels = Self::finish(
            orders,
            sectors,
            exact,
            precision,
            None,
            true,
            Vec::new(),
            settings,
        )?;
        kernels.attach_threshold(metadata)?;
        kernels.initialize_artifact()?;
        Ok(kernels)
    }
}
