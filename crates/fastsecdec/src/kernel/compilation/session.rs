//! Caller-stepped construction retaining every completed native sector evaluator.
use super::*;
use std::sync::Arc;

pub struct CompilationSession {
    generated: Arc<GeneratedIntegral>,
    runtime: Arc<Vec<Symbol>>,
    precision: PrecisionPolicy,
    settings: CompilationSettings,
    use_complex: bool,
    dynamic: Option<PreparedDynamic>,
    prepared: bool,
    owner: Arc<()>,
    sectors: Vec<SectorKernel>,
    result: Option<KernelSet>,
    complete: bool,
    failed: Option<String>,
    elapsed_seconds: f64,
}

impl CompilationSession {
    pub fn new(
        generated: Arc<GeneratedIntegral>,
        runtime_parameters: Vec<Symbol>,
        precision: PrecisionPolicy,
        settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let settings = settings.resolve_contour_jacobian(generated.contour_jacobian())?;
        settings.validate()?;
        let runtime_parameters = runtime_inputs(&generated, &runtime_parameters);
        validate_descriptor(&generated, &runtime_parameters)?;
        let use_complex = requires_complex(&generated, &runtime_parameters);
        Ok(Self {
            generated,
            runtime: Arc::new(runtime_parameters),
            precision,
            settings,
            use_complex,
            dynamic: None,
            prepared: false,
            owner: Arc::new(()),
            sectors: Vec::new(),
            result: None,
            complete: false,
            failed: None,
            elapsed_seconds: 0.0,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn completed_sectors(&self) -> usize {
        if self.complete {
            self.total_sectors()
        } else {
            self.sectors.len()
        }
    }
    pub fn total_sectors(&self) -> usize {
        self.generated.sectors().len()
    }
    pub fn take_result(&mut self) -> Option<KernelSet> {
        self.result.take()
    }

    /// Execute at most `max_units` sector builds or final assembly units.
    /// Breaking the observer pauses without discarding any completed evaluator.
    /// A native build is indivisible; no thread or background loop is created.
    /// Native errors remain terminal for this owner and never yield a result.
    pub fn step(
        &mut self,
        max_units: usize,
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<bool, KernelError> {
        let started = Instant::now();
        let result = self.step_inner(max_units, &mut progress);
        self.elapsed_seconds += started.elapsed().as_secs_f64();
        result
    }

    fn step_inner(
        &mut self,
        max_units: usize,
        progress: &mut impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<bool, KernelError> {
        let _preparing = crate::kernel::NativeProgramDescriptor::enter_optional(
            self.generated.program_descriptor().map(Arc::as_ref),
        );
        if max_units == 0 {
            return Err(KernelError::Compilation(
                "max_units must be positive".into(),
            ));
        }
        if let Some(reason) = &self.failed {
            return Err(KernelError::Compilation(reason.clone()));
        }
        if self.complete {
            return Ok(true);
        }
        for _ in 0..max_units {
            let event = self.snapshot();
            if progress(&event).is_break() {
                return Ok(false);
            }
            if !self.prepared {
                match PreparedDynamic::build(&self.generated, &self.runtime, self.settings) {
                    Ok(dynamic) => {
                        self.dynamic = dynamic;
                        self.prepared = true;
                    }
                    Err(error) => {
                        self.failed = Some(error.to_string());
                        return Err(error);
                    }
                }
            }
            let result = if self.sectors.len() < self.total_sectors() {
                let index = self.sectors.len();
                CompilationJob {
                    owner: self.owner.clone(),
                    program_descriptor: self.generated.sectors()[index]
                        .program_descriptor()
                        .cloned(),
                    request_lookup: self.dynamic.as_ref().map(|value| value.lookup.clone()),
                    index,
                    input: CompilationInput::Generated(self.generated.sectors()[index].clone()),
                    runtime_parameters: self.runtime.clone(),
                    precision: self.precision.clone(),
                    settings: self.settings,
                    use_complex: self.use_complex,
                }
                .run()
                .map(|completion| self.sectors.push(completion.sector))
            } else {
                KernelSet::finish(
                    self.generated.orders().to_vec(),
                    std::mem::take(&mut self.sectors),
                    self.dynamic.as_ref().map_or_else(
                        || self.generated.exact_coefficients().to_vec(),
                        |value| value.exact.clone(),
                    ),
                    self.precision.clone(),
                    Some(self.generated.metadata().clone()),
                    self.use_complex,
                    self.runtime.as_ref().clone(),
                    self.settings,
                )
                .and_then(|mut kernels| {
                    kernels.exact_requests = self
                        .dynamic
                        .as_ref()
                        .map_or_else(Vec::new, |value| value.exact_requests.clone());
                    kernels.attach_program_descriptor(
                        self.dynamic
                            .as_ref()
                            .map(|value| &value.descriptor)
                            .or(self.generated.program_descriptor()),
                    )?;
                    kernels.initialize_artifact()?;
                    self.result = Some(kernels);
                    self.complete = true;
                    Ok(())
                })
            };
            if let Err(error) = result {
                self.failed = Some(error.to_string());
                return Err(error);
            }
            if progress(&self.snapshot()).is_break() || self.complete {
                break;
            }
        }
        Ok(self.complete)
    }

    pub fn snapshot(&self) -> CompilationProgress {
        CompilationProgress {
            completed: self.completed_sectors(),
            total: self.total_sectors(),
            elapsed_seconds: self.elapsed_seconds,
        }
    }
}
