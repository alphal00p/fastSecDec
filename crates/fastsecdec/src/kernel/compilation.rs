//! Caller-driven compilation of one exact native program per sector.
#[cfg(feature = "native")]
use super::SectorExpressions;
use super::{
    Backend, CompilationProgress, KernelError, KernelSet, PrecisionPolicy, RealKernel,
    SectorKernel, cancellation::Cancellation, complex, evaluator, program,
};
use crate::generation::{GeneratedIntegral, GenerationMetadata};
use std::{collections::HashMap, ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, Symbol},
    domains::float::ErrorPropagatingFloat,
};

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
        let program = program::build_with_parameters(
            self.sector.parameters().to_vec(),
            &self.runtime_parameters,
            self.sector.aliased_coefficients(),
            Cancellation::new(
                self.sector.cancellation_degree(),
                Some(self.sector.cancellation_terms().to_vec()),
                self.sector.dimension(),
            )?,
        )?;
        let sector = SectorKernel::from_program(program, &self.precision, self.use_complex)?;
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
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        precision.validate()?;
        let started = Instant::now();
        let total = self.sectors().len();
        let use_complex = self.sectors().iter().any(|sector| {
            sector
                .aliased_coefficients()
                .iter()
                .any(|value| !program::is_real(value))
        }) || self
            .exact_coefficients()
            .iter()
            .any(super::has_complex_coefficients);
        // Build each exact IR lazily inside the same callback interval as its
        // native host compilation. Initial cancellation precedes symbolic work.
        let mut emit = |completed| emit(&mut progress, started, completed, total);
        emit(0)?;
        let mut sectors = Vec::with_capacity(total);
        for sector in self.sectors() {
            let program = program::build_with_parameters(
                sector.parameters().to_vec(),
                runtime_parameters,
                sector.aliased_coefficients(),
                Cancellation::new(
                    sector.cancellation_degree(),
                    Some(sector.cancellation_terms().to_vec()),
                    sector.dimension(),
                )?,
            )?;
            sectors.push(SectorKernel::from_program(
                program,
                &precision,
                use_complex,
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
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        precision.validate()?;
        let started = Instant::now();
        let total = self.sectors().len();
        emit(&mut progress, started, 0, total)?;
        let use_complex = self.sectors().iter().any(|sector| {
            sector
                .aliased_coefficients()
                .iter()
                .any(|value| !program::is_real(value))
        }) || self
            .exact_coefficients()
            .iter()
            .any(super::has_complex_coefficients);
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
        )?;
        kernels.initialize_artifact()?;
        Ok(kernels)
    }
}

impl SectorKernel {
    pub(super) fn from_program(
        program: program::SectorProgram,
        precision: &PrecisionPolicy,
        use_complex: bool,
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
        // Encode the untouched exact program, never a mapped/evaluated worker's
        // mutable stack. All numeric variants derive from this same native IR.
        let program_bytes: std::sync::Arc<[u8]> = program::encode(&exact)?.into();
        let operations = exact.count_operations().into();
        let backend = if use_complex {
            Backend::Complex(complex::ComplexKernel::from_program(
                exact,
                parameters.len(),
                cancellation.clone(),
                precision.clone(),
                exact_zero.clone(),
                real_coefficients,
            )?)
        } else {
            let evaluator = evaluator::real(&exact)?;
            let requirements =
                evaluator::MappingRequirements::new(&exact).map_err(KernelError::Compilation)?;
            let conditioning = requirements
                .map(
                    &exact,
                    |value| ErrorPropagatingFloat::new(value.re.to_f64(), 15.0),
                    53,
                )
                .ok();
            Backend::Real(RealKernel {
                precision_cache: super::precision_cache::PrecisionCache::new(requirements),
                exact_evaluator: exact,
                evaluator,
                conditioning,
                check_input: vec![ErrorPropagatingFloat::new(0.0, 15.0); inputs],
                check_output: vec![ErrorPropagatingFloat::new(0.0, 15.0); outputs],
            })
        };
        #[cfg(feature = "native")]
        let symjit_ir_bytes = Some(match &backend {
            Backend::Real(kernel) => kernel.evaluator.as_bytes().len(),
            Backend::Complex(kernel) => kernel.symjit_ir_bytes(),
        });
        #[cfg(feature = "portable")]
        let symjit_ir_bytes = None;
        let statistics = super::EvaluatorStatistics {
            version: 1,
            backend: if cfg!(feature = "native") {
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
            parameters_bound: runtime_parameters.is_empty(),
            runtime_parameters,
            parameters,
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
    #[cfg(feature = "native")]
    pub(super) fn from_expressions_for_load(
        orders: Vec<i32>,
        expressions: Vec<SectorExpressions>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
    ) -> Result<Self, KernelError> {
        let use_complex = expressions
            .iter()
            .flat_map(|sector| &sector.coefficients)
            .chain(&exact_expressions)
            .any(super::has_complex_coefficients);
        let programs = expressions
            .into_iter()
            .map(|sector| {
                let coefficients = sector
                    .coefficients
                    .into_iter()
                    .map(AliasedAtom::from)
                    .collect::<Vec<_>>();
                program::build(sector.parameters, &coefficients, sector.cancellation)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_programs_for_load(
            orders,
            programs,
            exact_expressions,
            precision,
            metadata,
            use_complex,
            Vec::new(),
        )
    }

    pub(super) fn from_programs_for_load(
        orders: Vec<i32>,
        programs: Vec<program::SectorProgram>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        runtime_parameters: Vec<Symbol>,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let mut sectors = Vec::with_capacity(programs.len());
        for program in programs {
            if program.exact.get_output_len() != orders.len() {
                return Err(KernelError::Artifact(
                    "native program Laurent output count differs".into(),
                ));
            }
            sectors.push(SectorKernel::from_program(
                program,
                &precision,
                use_complex,
            )?);
        }
        Self::finish(
            orders,
            sectors,
            exact_expressions,
            precision,
            metadata,
            use_complex,
            runtime_parameters,
        )
    }

    fn finish(
        coefficient_orders: Vec<i32>,
        sectors: Vec<SectorKernel>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        runtime_parameters: Vec<Symbol>,
    ) -> Result<Self, KernelError> {
        if sectors
            .iter()
            .any(|sector| sector.runtime_parameters != runtime_parameters)
        {
            return Err(KernelError::Artifact(
                "inconsistent runtime parameter schema".into(),
            ));
        }
        let exact_kernel = if runtime_parameters.is_empty() {
            None
        } else {
            Some(SectorKernel::from_program(
                program::build_with_parameters(
                    Vec::new(),
                    &runtime_parameters,
                    &exact_expressions
                        .iter()
                        .cloned()
                        .map(AliasedAtom::from)
                        .collect::<Vec<_>>(),
                    Cancellation::new(0, Some(Vec::new()), 0)?,
                )?,
                &precision,
                use_complex,
            )?)
        };
        let exact_coefficients = if exact_kernel.is_some() {
            vec![f64::NAN; coefficient_orders.len() * if use_complex { 2 } else { 1 }]
        } else if use_complex {
            complex::exact(&exact_expressions)?
        } else {
            let constants = HashMap::<Atom, f64>::new();
            exact_expressions
                .iter()
                .map(|coefficient| {
                    coefficient
                        .evaluate(&constants)
                        .map_err(|error| KernelError::Compilation(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        if runtime_parameters.is_empty()
            && exact_coefficients.iter().any(|value| !value.is_finite())
        {
            return Err(KernelError::NonFinite);
        }
        use crate::status::CoefficientComponent::{Imag, Real};
        Ok(Self {
            runtime_parameters,
            exact_kernel,
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
