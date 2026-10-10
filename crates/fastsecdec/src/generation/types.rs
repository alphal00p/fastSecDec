use crate::kernel::indexed::ProgramRecipe;
use fastsecdec_sectors::{DecompositionOptions, DecompositionProgress, SectorMap};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use symbolica::atom::{AliasedAtom, Atom, Symbol};

#[derive(Clone, Debug)]
pub struct GenerationOptions {
    /// How sector maps and endpoint derivatives enter evaluator construction.
    pub mode: GenerationMode,
    /// Select one mathematical evaluator recipe. Scheduling is independent of
    /// this choice; public program archives can retain several alternatives.
    /// Contour recipes require explicit F/U semantics and never infer them
    /// from polynomial ordering.
    pub program_recipe: crate::kernel::indexed::ProgramRecipe,
    /// Legacy recorded caller assertion; threshold freedom is always the
    /// caller's responsibility when no regularisation is requested.
    pub assume_no_threshold: bool,
    /// Highest included Laurent order. Lower pole orders are found exactly.
    pub max_order: i32,
    pub decomposition: DecompositionOptions,
    pub max_subtractions_per_axis: usize,
    pub max_subtraction_terms: usize,
    pub subtraction: SubtractionStrategy,
    pub coefficient_expansion: CoefficientExpansionOptions,
}

/// Independent implementations of the same exact sector integral.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Materialize mapped expressions and their symbolic endpoint derivatives.
    #[default]
    Symbolic,
    /// Compose sector maps and endpoint jets in native evaluator arithmetic.
    NumericalDual,
}

impl GenerationMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Symbolic => "symbolic",
            Self::NumericalDual => "numerical_dual",
        }
    }
}

/// Representation used during endpoint subtraction and Laurent expansion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoefficientExpansionMethod {
    /// Expand the complete subtracted expression in the regulator.
    #[default]
    #[serde(rename = "full_expression", alias = "physical")]
    Physical,
    /// Compose the Laurent vector from shared regular-coefficient series.
    #[serde(rename = "coefficient_series", alias = "native_named")]
    NativeNamed,
}

impl CoefficientExpansionMethod {
    /// Canonical user-facing configuration and status name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Physical => "full_expression",
            Self::NativeNamed => "coefficient_series",
        }
    }
}

/// Limits for the optional native named-coefficient route. `None` adds no cap;
/// native integer ranges and the ordinary subtraction limits still apply.
/// These limits do not affect `Physical` or the named route's exact physical
/// fallback for unregulated endpoint admission. Width is a native relative
/// request, not a claim about absolute Laurent coverage.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CoefficientExpansionOptions {
    pub method: CoefficientExpansionMethod,
    pub max_series_attempts: Option<usize>,
    pub max_relative_width: Option<i64>,
    pub max_unique_requests: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoefficientExpansionStage {
    Admission,
    RegularSeries,
    Naming,
    Endpoint,
    Composition,
    Coverage,
    Lowering,
    PhysicalFallback,
    Complete,
}

/// Counts within the current native-width attempt; they reset on each retry.
/// Formal requests and aliases are representation counts, not physical sectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoefficientRequestCounts {
    pub source_bodies: usize,
    pub unique_requests: usize,
    pub cached_partials: usize,
    pub aliases: usize,
    pub interleaved_requests: usize,
    pub fallback_requests: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtractionStrategy {
    #[default]
    Taylor,
    IntegrateByParts,
}

impl SubtractionStrategy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Taylor => "taylor",
            Self::IntegrateByParts => "integrate_by_parts",
        }
    }
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            mode: GenerationMode::default(),
            program_recipe: crate::kernel::indexed::ProgramRecipe::UndeformedV1,
            assume_no_threshold: false,
            max_order: 0,
            decomposition: DecompositionOptions::default(),
            max_subtractions_per_axis: 64,
            max_subtraction_terms: 1_000_000,
            subtraction: SubtractionStrategy::Taylor,
            coefficient_expansion: CoefficientExpansionOptions::default(),
        }
    }
}

impl GenerationOptions {
    /// Whether this selected recipe deforms the smooth sector density.
    pub fn contour_enabled(&self) -> bool {
        self.program_recipe != crate::kernel::indexed::ProgramRecipe::UndeformedV1
    }
}

#[derive(Clone, Debug)]
pub enum GenerationProgress {
    Decomposition(DecompositionProgress),
    Factorization {
        sector: usize,
        total: usize,
    },
    NumericalMapping {
        sector: usize,
        total: usize,
    },
    /// Unique complete-chart formulas; `reused` counts planned shared uses.
    FormulaPreparation {
        completed: usize,
        total: usize,
        sectors: usize,
        reused: usize,
    },
    /// Number of charts whose prepared formula or exact fallback is assembled.
    FormulaInstantiation {
        sector: usize,
        total: usize,
    },
    /// Per-chart preparation of a complete density and its native canonical graph.
    SymmetryPreparation {
        sector: usize,
        total: usize,
    },
    /// Deterministic exact comparison and representative admission.
    Symmetry {
        completed: usize,
        total: usize,
    },
    Subtraction {
        sector: usize,
        total: usize,
        terms: usize,
    },
    LaurentExpansion {
        sector: usize,
        total: usize,
    },
    /// Named coefficient work. Zero attempt/width denotes work not yet started,
    /// never a Series cutoff. A `PhysicalFallback` stage records the effective
    /// route; its entire duration remains in `CoefficientExpansion` timing.
    CoefficientExpansion {
        sector: usize,
        total: usize,
        stage: CoefficientExpansionStage,
        attempt: usize,
        relative_width: i64,
        formal_pieces: usize,
        requests: CoefficientRequestCounts,
    },
    PhaseTiming {
        phase: GenerationPhase,
        seconds: f64,
    },
    Complete {
        sectors: usize,
        orders: Vec<i32>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationPhase {
    Domain,
    Geometry,
    Mapping,
    FormulaPreparation,
    Symmetry,
    Subtraction,
    Laurent,
    CoefficientExpansion,
}

/// Provenance of a fresh sector's numerical conditioning profile. Both forms
/// schedule precision checks; neither is a floating-point error certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditioningBasis {
    RetainedRemainders,
    MappedEndpointBound,
}

/// Exact original endpoint provenance for one cancellation loss contribution.
/// The power is a canonical native Rational string, independent of Atom state.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointCancellationSource {
    pub order: usize,
    pub original_power: String,
}

/// A retained remainder has one source per active axis. A mapped endpoint bound
/// retains alternatives from the original terms; each axis takes their maximum
/// only after normalization by the chosen power-specific threshold.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointProfileRow {
    pub axes: Vec<Vec<EndpointCancellationSource>>,
}

/// All Laurent outputs retain the same sector integration support.
#[derive(Clone, Debug)]
pub struct GeneratedSector {
    pub(crate) program_descriptor: Option<std::sync::Arc<crate::kernel::NativeProgramDescriptor>>,
    pub(crate) dynamic_check_sources: Vec<std::sync::Arc<crate::kernel::DynamicCheckSource>>,
    pub(crate) deferred: Option<std::sync::Arc<super::numerical_dual::DualSector>>,
    pub(crate) cancellation_degree: usize,
    pub(crate) cancellation_terms: Vec<Vec<usize>>,
    pub(crate) endpoint_profiles: Vec<EndpointProfileRow>,
    pub(crate) conditioning_basis: ConditioningBasis,
    pub(crate) parameters: Vec<Symbol>,
    pub(crate) coefficients: Vec<AliasedAtom>,
    pub(crate) materialized: OnceLock<Vec<Atom>>,
    pub(crate) map: SectorMap,
}

impl GeneratedSector {
    pub(crate) fn program_descriptor(
        &self,
    ) -> Option<&std::sync::Arc<crate::kernel::NativeProgramDescriptor>> {
        self.program_descriptor.as_ref()
    }
    pub(crate) fn dynamic_check_sources(
        &self,
    ) -> &[std::sync::Arc<crate::kernel::DynamicCheckSource>] {
        &self.dynamic_check_sources
    }
    pub fn generation_mode(&self) -> GenerationMode {
        if self.deferred.is_some() {
            GenerationMode::NumericalDual
        } else {
            GenerationMode::Symbolic
        }
    }
    pub fn cancellation_degree(&self) -> usize {
        self.cancellation_degree
    }
    /// Coordinate conditioning profiles used by the precision heuristic.
    /// Named generation may supply a conservative row rather than actual
    /// retained remainders. The numeric artifact persists rows and degree,
    /// but does not record this fresh result's descriptive basis.
    pub fn cancellation_terms(&self) -> &[Vec<usize>] {
        &self.cancellation_terms
    }
    pub fn endpoint_profiles(&self) -> &[EndpointProfileRow] {
        &self.endpoint_profiles
    }
    pub fn conditioning_basis(&self) -> ConditioningBasis {
        self.conditioning_basis
    }
    pub fn dimension(&self) -> usize {
        self.parameters.len()
    }
    pub fn parameters(&self) -> &[Symbol] {
        &self.parameters
    }
    /// Native coefficient roots and their coordinate-dependent definitions.
    /// This view preserves shared subexpressions without restoring large Atoms.
    pub fn aliased_coefficients(&self) -> &[AliasedAtom] {
        &self.coefficients
    }
    /// Materialize and cache the complete native expressions on explicit request.
    /// Prefer [`Self::aliased_coefficients`] for compact symbolic inspection.
    /// Kernel compilation and portable saving do not call this accessor.
    pub fn coefficients(&self) -> &[Atom] {
        self.materialized.get_or_init(|| {
            if let Some(deferred) = &self.deferred {
                return deferred.materialized_coefficients();
            }
            self.coefficients
                .iter()
                .cloned()
                .map(AliasedAtom::into_inner)
                .collect()
        })
    }
    pub fn map(&self) -> &SectorMap {
        &self.map
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedIntegral {
    pub(crate) program_descriptor: Option<std::sync::Arc<crate::kernel::NativeProgramDescriptor>>,
    pub(crate) dynamic_check_sources: Vec<std::sync::Arc<crate::kernel::DynamicCheckSource>>,
    pub(crate) metadata: super::GenerationMetadata,
    pub(crate) orders: Vec<i32>,
    pub(crate) sectors: Vec<GeneratedSector>,
    pub(crate) exact_coefficients: Vec<Atom>,
}

impl GeneratedIntegral {
    /// An empty result has no chart from which compilation can recover its
    /// requested capability. Preserve that explicit generation choice;
    /// ordinary nonempty records keep their existing descriptor and codec.
    pub(super) fn preserve_empty_recipe(
        mut self,
        recipe: ProgramRecipe,
    ) -> Result<Self, GenerationError> {
        if self.metadata.charts().is_empty()
            && self.program_descriptor.is_none()
            && recipe != ProgramRecipe::UndeformedV1
        {
            let descriptor = if recipe.is_dynamic() {
                // An empty source has neither a radius nor a certificate to
                // construct. This is an explicit exact zero, never a fallback
                // for unresolved singularities or a scalelessness assumption.
                if !self.sectors.is_empty()
                    || self.exact_coefficients.iter().any(|value| !value.is_zero())
                    || !self.dynamic_check_sources.is_empty()
                {
                    return Err(GenerationError::Invariant(
                        "a chart-free dynamic result must be an explicit exact zero".into(),
                    ));
                }
                crate::kernel::NativeProgramDescriptor::dynamic(recipe, vec![], vec![])
            } else {
                crate::kernel::NativeProgramDescriptor::static_recipe(recipe)
            }
            .map_err(|error| GenerationError::Contour(error.to_string()))?;
            self.program_descriptor = Some(std::sync::Arc::new(descriptor));
        }
        Ok(self)
    }

    pub(crate) fn program_descriptor(
        &self,
    ) -> Option<&std::sync::Arc<crate::kernel::NativeProgramDescriptor>> {
        self.program_descriptor.as_ref()
    }
    pub(crate) fn dynamic_check_sources(
        &self,
    ) -> &[std::sync::Arc<crate::kernel::DynamicCheckSource>] {
        &self.dynamic_check_sources
    }
    pub fn metadata(&self) -> &super::GenerationMetadata {
        &self.metadata
    }
    pub fn orders(&self) -> &[i32] {
        &self.orders
    }
    pub fn sectors(&self) -> &[GeneratedSector] {
        &self.sectors
    }
    /// Contributions whose entire support has been integrated analytically.
    /// Their covariance is zero and they must be added exactly once.
    pub fn exact_coefficients(&self) -> &[Atom] {
        &self.exact_coefficients
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("contour deformation: {0}")]
    Contour(String),
    #[error(
        "no-threshold domain could not be certified for {0}; an explicit caller assertion is required"
    )]
    UnknownDomain(Atom),
    #[error("an interior zero or sign change proves a threshold for {0}")]
    Threshold(Atom),
    #[error(
        "mixed-sign cube factor has an unresolved upper-boundary zero requiring affine endpoint charts: {0}"
    )]
    UpperBoundary(Atom),
    #[error(
        "mixed-sign factor has uncertified boundary geometry; the no-threshold assertion covers only the open domain: {0}"
    )]
    UnresolvedBoundary(Atom),
    #[error(
        "a negative polynomial with a noninteger or regulator-dependent exponent requires complex branch handling: {0}"
    )]
    ComplexBranch(Atom),
    #[error(transparent)]
    Parametric(#[from] crate::parametric::ParametricError),
    #[error(transparent)]
    Geometry(#[from] fastsecdec_sectors::SectorError),
    #[error("generation cancelled")]
    Cancelled,
    #[error(
        "unsupported endpoint exponent {0}: expected an affine regulator dependence with rational coefficients"
    )]
    EndpointExponent(Atom),
    #[error(
        "endpoint divergence at parameter {parameter:?} is not regulated by the supplied regulator"
    )]
    UnregulatedEndpoint { parameter: Symbol },
    #[error("generation resource limit: {0}")]
    ResourceLimit(&'static str),
    #[error("Symbolica Laurent expansion failed: {0}")]
    Series(String),
    #[error("expected integer Laurent orders, got exponent {0}")]
    FractionalLaurent(String),
    #[error("internal generation invariant failed: {0}")]
    Invariant(String),
    #[error("reserved coefficient symbol {name} conflicts with native metadata: {reason}")]
    ReservedSymbolConflict { name: String, reason: String },
}
