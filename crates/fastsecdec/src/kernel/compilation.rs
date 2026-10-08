//! Caller-driven compilation of one exact native program per sector.
use super::{
    Backend, CompilationProgress, CompilationSettings, KernelError, KernelSet, PrecisionPolicy,
    RealKernel, SectorKernel, complex, evaluator, program,
};
use crate::generation::{GeneratedIntegral, GenerationMetadata};
use std::{ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{Atom, Symbol},
    domains::float::ErrorPropagatingFloat,
};
mod session;
pub use session::CompilationSession;

pub(super) fn requires_complex(generated: &GeneratedIntegral, runtime: &[Symbol]) -> bool {
    generated.sectors().iter().any(|sector| {
        // Formal request placeholders do not prove the realness of deferred
        // factors or their derivatives. Retain native complex intermediates
        // without materializing the mapped expressions merely for this proof.
        sector.deferred.is_some()
            || sector.aliased_coefficients().iter().any(|coefficient| {
                !program::is_real_with_parameters(coefficient, sector.parameters(), runtime)
            })
    }) || generated
        .exact_coefficients()
        .iter()
        .any(|coefficient| !program::is_real_expression(coefficient, runtime))
}

/// Caller-owned native sector compilation; jobs are tied to one compilation call.
pub type CompilationDispatch<'a> = dyn FnMut(
        &mut dyn ExactSizeIterator<Item = CompilationJob>,
    ) -> Result<Vec<CompilationCompletion>, KernelError>
    + 'a;

pub struct CompilationJob {
    owner: std::sync::Arc<()>,
    index: usize,
    sector: crate::generation::GeneratedSector,
    runtime_parameters: std::sync::Arc<Vec<Symbol>>,
    precision: PrecisionPolicy,
    settings: CompilationSettings,
    use_complex: bool,
}
pub struct CompilationCompletion {
    owner: std::sync::Arc<()>,
    index: usize,
    sector: SectorKernel,
}
impl CompilationJob {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn run(self) -> Result<CompilationCompletion, KernelError> {
        let program = program::build_sector(&self.sector, &self.runtime_parameters, self.settings)?;
        let sector = SectorKernel::from_program_with_backend(
            program,
            &self.precision,
            self.use_complex,
            self.settings.backend,
        )?;
        Ok(CompilationCompletion {
            owner: self.owner,
            index: self.index,
            sector,
        })
    }
}

impl GeneratedIntegral {
    pub fn compile(&self) -> Result<KernelSet, KernelError> {
        self.compile_with_precision(PrecisionPolicy::default())
    }

    pub fn compile_with_settings(
        &self,
        settings: CompilationSettings,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_settings_parameters_and_progress(
            PrecisionPolicy::default(),
            &[],
            settings,
            |_| ControlFlow::Continue(()),
        )
    }

    pub fn compile_with_precision(
        &self,
        precision: PrecisionPolicy,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_and_progress(precision, |_| ControlFlow::Continue(()))
    }

    pub fn compile_with_progress(
        &self,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_and_progress(PrecisionPolicy::default(), progress)
    }

    pub fn compile_with_precision_and_progress(
        &self,
        precision: PrecisionPolicy,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_parameters_and_progress(precision, &[], progress)
    }

    pub fn compile_with_parameters_and_progress(
        &self,
        runtime_parameters: &[Symbol],
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_parameters_and_progress(
            PrecisionPolicy::default(),
            runtime_parameters,
            progress,
        )
    }

    pub fn compile_with_precision_parameters_and_progress(
        &self,
        precision: PrecisionPolicy,
        runtime_parameters: &[Symbol],
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_settings_parameters_and_progress(
            precision,
            runtime_parameters,
            CompilationSettings::default(),
            progress,
        )
    }

    pub fn compile_with_settings_parameters_and_progress(
        &self,
        precision: PrecisionPolicy,
        runtime_parameters: &[Symbol],
        settings: CompilationSettings,
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        precision.validate()?;
        settings.validate()?;
        let started = Instant::now();
        let total = self.sectors().len();
        let use_complex = requires_complex(self, runtime_parameters);
        // Build each exact IR lazily inside the same callback interval as its
        // native host compilation. Initial cancellation precedes symbolic work.
        let mut emit = |completed| emit(&mut progress, started, completed, total);
        emit(0)?;
        let mut sectors = Vec::with_capacity(total);
        for sector in self.sectors() {
            let program = program::build_sector(sector, runtime_parameters, settings)?;
            sectors.push(SectorKernel::from_program_with_backend(
                program,
                &precision,
                use_complex,
                settings.backend,
            )?);
            emit(sectors.len())?;
        }
        let mut kernels = KernelSet::finish(
            self.orders().to_vec(),
            sectors,
            self.exact_coefficients().to_vec(),
            precision,
            Some(self.metadata().clone()),
            use_complex,
            runtime_parameters.to_vec(),
            settings,
        )?;
        kernels.initialize_artifact()?;
        Ok(kernels)
    }
    /// Compile independent sectors on the caller's executor and assemble in native order.
    /// The library owns no threads. Cancellation around individual compilation
    /// calls is caller driven; native symbolic/JIT calls are not preemptible.
    pub fn compile_with_parameters_and_dispatch(
        &self,
        runtime_parameters: &[Symbol],
        dispatch: &mut CompilationDispatch<'_>,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_parameters_and_dispatch(
            PrecisionPolicy::default(),
            runtime_parameters,
            dispatch,
            progress,
        )
    }

    pub fn compile_with_precision_parameters_and_dispatch(
        &self,
        precision: PrecisionPolicy,
        runtime_parameters: &[Symbol],
        dispatch: &mut CompilationDispatch<'_>,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_settings_parameters_and_dispatch(
            precision,
            runtime_parameters,
            CompilationSettings::default(),
            dispatch,
            progress,
        )
    }

    pub fn compile_with_settings_parameters_and_dispatch(
        &self,
        precision: PrecisionPolicy,
        runtime_parameters: &[Symbol],
        settings: CompilationSettings,
        dispatch: &mut CompilationDispatch<'_>,
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        precision.validate()?;
        settings.validate()?;
        let started = Instant::now();
        let total = self.sectors().len();
        emit(&mut progress, started, 0, total)?;
        let use_complex = requires_complex(self, runtime_parameters);
        let owner = std::sync::Arc::new(());
        let runtime = std::sync::Arc::new(runtime_parameters.to_vec());
        let mut jobs = self
            .sectors()
            .iter()
            .enumerate()
            .map(|(index, sector)| CompilationJob {
                owner: std::sync::Arc::clone(&owner),
                index,
                sector: sector.clone(),
                runtime_parameters: std::sync::Arc::clone(&runtime),
                precision: precision.clone(),
                settings,
                use_complex,
            });
        let mut completions = dispatch(&mut jobs)?;
        if completions.len() != total
            || completions
                .iter()
                .any(|value| !std::sync::Arc::ptr_eq(&owner, &value.owner))
        {
            return Err(KernelError::Artifact(
                "compilation dispatcher returned foreign or incomplete work".into(),
            ));
        }
        completions.sort_by_key(|value| value.index);
        let mut sectors = Vec::with_capacity(total);
        for (index, completion) in completions.into_iter().enumerate() {
            if completion.index != index {
                return Err(KernelError::Artifact(
                    "compilation dispatcher duplicated or omitted work".into(),
                ));
            }
            sectors.push(completion.sector);
        }
        emit(&mut progress, started, total, total)?;
        let mut kernels = KernelSet::finish(
            self.orders().to_vec(),
            sectors,
            self.exact_coefficients().to_vec(),
            precision,
            Some(self.metadata().clone()),
            use_complex,
            runtime_parameters.to_vec(),
            settings,
        )?;
        kernels.initialize_artifact()?;
        Ok(kernels)
    }
}

impl SectorKernel {
    #[cfg(test)]
    pub(super) fn from_program(
        program: program::SectorProgram,
        precision: &PrecisionPolicy,
        use_complex: bool,
    ) -> Result<Self, KernelError> {
        Self::from_program_with_backend(
            program,
            precision,
            use_complex,
            super::EvaluatorBackend::Auto,
        )
    }

    pub(super) fn from_program_with_backend(
        program: program::SectorProgram,
        precision: &PrecisionPolicy,
        use_complex: bool,
        execution: super::EvaluatorBackend,
    ) -> Result<Self, KernelError> {
        Self::from_program_with_bytes(program, precision, use_complex, execution, None)
    }

    fn from_program_with_bytes(
        program: program::SectorProgram,
        precision: &PrecisionPolicy,
        use_complex: bool,
        execution: super::EvaluatorBackend,
        encoded: Option<std::sync::Arc<[u8]>>,
    ) -> Result<Self, KernelError> {
        let program::SectorProgram {
            parameters,
            runtime_parameters,
            exact,
            cancellation,
            exact_zero,
            real_coefficients,
        } = program;
        let inputs = parameters.len() + runtime_parameters.len();
        let outputs = exact.get_output_len();
        if exact.get_input_len() != inputs
            || exact_zero.len() != outputs
            || real_coefficients.len() != outputs
        {
            return Err(KernelError::Artifact(
                "native program dimensions and facts differ".into(),
            ));
        }
        if !use_complex
            && exact
                .get_constants()
                .iter()
                .any(|value| !value.im.is_zero())
        {
            return Err(KernelError::Artifact(
                "complex native coefficient in real output layout".into(),
            ));
        }
        // A loader already owns the untouched native program bytes. Retain them
        // directly instead of serializing the entire decoded program again.
        // Generation encodes before any numeric mapping mutates a workspace.
        let program_bytes = match encoded {
            Some(bytes) => bytes,
            None => program::encode(&exact)?.into(),
        };
        let operations = exact.count_operations().into();
        let backend = if use_complex {
            Backend::Complex(complex::ComplexKernel::from_program(
                exact,
                parameters.len(),
                cancellation.clone(),
                precision.clone(),
                exact_zero.clone(),
                real_coefficients,
                execution,
            )?)
        } else {
            let evaluator = evaluator::real(&exact, execution)?;
            let requirements =
                evaluator::MappingRequirements::new(&exact).map_err(KernelError::Compilation)?;
            let conditioning = evaluator::Conditioning::new(requirements.clone());
            Backend::Real(RealKernel {
                double_cache: super::precision_cache::PrecisionCache::new(requirements.clone()),
                f64_timing: Default::default(),
                conditioning_timing: Default::default(),
                precision_cache: super::precision_cache::PrecisionCache::new(requirements),
                exact_evaluator: exact,
                evaluator,
                conditioning,
                check_input: vec![ErrorPropagatingFloat::new(0.0, 15.0); inputs],
                check_output: vec![ErrorPropagatingFloat::new(0.0, 15.0); outputs],
            })
        };
        let symjit_ir_bytes = match &backend {
            Backend::Real(kernel) => kernel.evaluator.symjit_ir_bytes(),
            Backend::Complex(kernel) => kernel.symjit_ir_bytes(),
        };
        let statistics = super::EvaluatorStatistics {
            version: 1,
            backend: if symjit_ir_bytes.is_some() {
                "symjit_o2"
            } else {
                "symbolica_interpreter"
            }
            .into(),
            arithmetic: if use_complex { "complex" } else { "real" }.into(),
            inputs,
            outputs,
            exact_program_bytes: program_bytes.len(),
            operations,
            symjit_ir_bytes,
        };
        Ok(Self {
            input: vec![0.0; inputs],
            projection: None,
            parameters_bound: runtime_parameters.is_empty(),
            runtime_parameters,
            parameters,
            routing: super::stability::Routing::new(
                &cancellation,
                &super::StabilitySettings::default(),
                None,
            )?,
            stability: super::StabilitySettings::default(),
            cancellation,
            precision: precision.clone(),
            exact_zero,
            program_bytes,
            statistics,
            backend,
        })
    }
}

impl KernelSet {
    // Codec loaders validate and retain their own original artifact after these
    // constructors finish. Do not encode a replacement envelope only to discard it.
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    pub(super) fn from_programs_for_load(
        orders: Vec<i32>,
        programs: Vec<program::SectorProgram>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        runtime_parameters: Vec<Symbol>,
        settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        Self::from_programs_for_load_with_progress(
            orders,
            programs,
            exact_expressions,
            precision,
            metadata,
            use_complex,
            runtime_parameters,
            settings,
            None,
            &mut |_| ControlFlow::Continue(()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_programs_for_load_with_progress(
        orders: Vec<i32>,
        programs: Vec<program::SectorProgram>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        runtime_parameters: Vec<Symbol>,
        settings: CompilationSettings,
        encoded_programs: Option<Vec<std::sync::Arc<[u8]>>>,
        progress: &mut impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let started = Instant::now();
        let total = programs.len();
        if encoded_programs
            .as_ref()
            .is_some_and(|bytes| bytes.len() != total)
        {
            return Err(KernelError::Artifact(
                "native program byte count differs from sector count".into(),
            ));
        }
        let mut encoded_programs = encoded_programs.map(Vec::into_iter);
        if progress(&CompilationProgress {
            completed: 0,
            total,
            elapsed_seconds: 0.0,
        })
        .is_break()
        {
            return Err(KernelError::Cancelled);
        }
        let mut sectors = Vec::with_capacity(programs.len());
        for program in programs {
            if program.exact.get_output_len() != orders.len() {
                return Err(KernelError::Artifact(
                    "native program Laurent output count differs".into(),
                ));
            }
            sectors.push(SectorKernel::from_program_with_bytes(
                program,
                &precision,
                use_complex,
                settings.backend,
                encoded_programs.as_mut().and_then(Iterator::next),
            )?);
            if progress(&CompilationProgress {
                completed: sectors.len(),
                total,
                elapsed_seconds: started.elapsed().as_secs_f64(),
            })
            .is_break()
            {
                return Err(KernelError::Cancelled);
            }
        }
        Self::finish(
            orders,
            sectors,
            exact_expressions,
            precision,
            metadata,
            use_complex,
            runtime_parameters,
            settings,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn finish(
        coefficient_orders: Vec<i32>,
        sectors: Vec<SectorKernel>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        runtime_parameters: Vec<Symbol>,
        compilation_settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        if sectors
            .iter()
            .any(|sector| sector.runtime_parameters != runtime_parameters)
        {
            return Err(KernelError::Artifact(
                "inconsistent runtime parameter schema".into(),
            ));
        }
        let exact_coefficients = if !runtime_parameters.is_empty() {
            vec![f64::NAN; coefficient_orders.len() * if use_complex { 2 } else { 1 }]
        } else {
            super::exact::evaluate(&exact_expressions, &Default::default(), use_complex)?
        };
        if runtime_parameters.is_empty()
            && exact_coefficients.iter().any(|value| !value.is_finite())
        {
            return Err(KernelError::NonFinite);
        }
        use crate::status::CoefficientComponent::{Imag, Real};
        Ok(Self {
            compilation_settings,
            runtime_parameters,
            stability: super::StabilitySettings::default(),
            runtime_mass_constraints: Vec::new(),
            template_content_id: None,
            portable_artifact: None,
            metadata,
            orders: coefficient_orders
                .iter()
                .flat_map(|order| {
                    if use_complex {
                        vec![*order, *order]
                    } else {
                        vec![*order]
                    }
                })
                .collect(),
            components: coefficient_orders
                .iter()
                .flat_map(|_| {
                    if use_complex {
                        vec![Real, Imag]
                    } else {
                        vec![Real]
                    }
                })
                .collect(),
            coefficient_orders,
            precision,
            exact_expressions,
            exact_coefficients,
            content_id: String::new(),
            sectors,
        })
    }
}

fn emit(
    progress: &mut impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    started: Instant,
    completed: usize,
    total: usize,
) -> Result<(), KernelError> {
    if progress(&CompilationProgress {
        completed,
        total,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    })
    .is_break()
    {
        Err(KernelError::Cancelled)
    } else {
        Ok(())
    }
}
