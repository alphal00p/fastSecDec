//! Caller-driven compilation of one exact native program per sector.
use super::{
    Backend, CompilationProgress, KernelError, KernelSet, PrecisionPolicy, RealKernel,
    SectorExpressions, SectorKernel, cancellation::Cancellation, complex, program,
};
use crate::generation::{GeneratedIntegral, GenerationMetadata};
use std::{collections::HashMap, ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore},
    domains::float::ErrorPropagatingFloat,
    evaluate::JITCompilationSettings,
};

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
            let program = program::build(
                sector.parameters().to_vec(),
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
        KernelSet::finish(
            self.orders().to_vec(),
            sectors,
            self.exact_coefficients().to_vec(),
            precision,
            Some(self.metadata().clone()),
            use_complex,
        )
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
            exact,
            cancellation,
            exact_zero,
            real_coefficients,
        } = program;
        let inputs = parameters.len();
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
        let program_bytes = program::encode(&exact)?.into();
        let backend = if use_complex {
            Backend::Complex(complex::ComplexKernel::from_program(
                exact,
                cancellation.clone(),
                precision.clone(),
                exact_zero.clone(),
                real_coefficients,
            )?)
        } else {
            let evaluator = exact
                .jit_compile::<f64>(
                    JITCompilationSettings::default()
                        .optimization_level(2)
                        .direct_translation(true),
                )
                .map_err(KernelError::Compilation)?;
            // Fallible native admission resolves external constants before the
            // conditioning program's infallible coefficient mapping.
            let conditioning = exact
                .clone()
                .map_coeff(&|value| ErrorPropagatingFloat::new(value.re.to_f64(), 15.0));
            Backend::Real(RealKernel {
                precision_cache: Default::default(),
                exact_evaluator: exact,
                evaluator,
                conditioning,
                check_input: vec![ErrorPropagatingFloat::new(0.0, 15.0); inputs],
                check_output: vec![ErrorPropagatingFloat::new(0.0, 15.0); outputs],
            })
        };
        Ok(Self {
            parameters,
            cancellation,
            precision: precision.clone(),
            exact_zero,
            program_bytes,
            backend,
        })
    }
}

impl KernelSet {
    pub(super) fn from_expressions(
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
        Self::from_programs_with_progress(
            orders,
            programs,
            exact_expressions,
            precision,
            metadata,
            use_complex,
            |_| ControlFlow::Continue(()),
        )
    }

    pub(super) fn from_programs_with_progress(
        orders: Vec<i32>,
        programs: Vec<program::SectorProgram>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let started = Instant::now();
        let total = programs.len();
        emit(&mut progress, started, 0, total)?;
        let mut sectors = Vec::with_capacity(total);
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
            emit(&mut progress, started, sectors.len(), total)?;
        }
        Self::finish(
            orders,
            sectors,
            exact_expressions,
            precision,
            metadata,
            use_complex,
        )
    }

    fn finish(
        coefficient_orders: Vec<i32>,
        sectors: Vec<SectorKernel>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        metadata: Option<GenerationMetadata>,
        use_complex: bool,
    ) -> Result<Self, KernelError> {
        let exact_coefficients = if use_complex {
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
        if exact_coefficients.iter().any(|value| !value.is_finite()) {
            return Err(KernelError::NonFinite);
        }
        use crate::status::CoefficientComponent::{Imag, Real};
        let mut result = Self {
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
        };
        result.initialize_artifact()?;
        Ok(result)
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
