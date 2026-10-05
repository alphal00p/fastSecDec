use fastsecdec_sectors::{DecompositionOptions, DecompositionProgress, SectorMap};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use symbolica::atom::{AliasedAtom, Atom, Symbol};

#[derive(Clone, Debug)]
pub struct GenerationOptions {
    /// Accept an inconclusive no-threshold check by an explicit caller assertion.
    /// A proven interior zero or sign change is never overridden.
    /// Boundary geometry must still be certified for the available charts.
    pub assume_no_threshold: bool,
    /// Highest included Laurent order. Lower pole orders are found exactly.
    pub max_order: i32,
    pub decomposition: DecompositionOptions,
    pub max_subtractions_per_axis: usize,
    pub max_subtraction_terms: usize,
    pub subtraction: SubtractionStrategy,
    pub coefficient_expansion: CoefficientExpansionOptions,
}

/// Representation used during endpoint subtraction and Laurent expansion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoefficientExpansionMethod {
    #[default]
    Physical,
    NativeNamed,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubtractionStrategy {
    Taylor,
    IntegrateByParts,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
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

#[derive(Clone, Debug)]
pub enum GenerationProgress {
    Decomposition(DecompositionProgress),
    Factorization {
        sector: usize,
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

/// All Laurent outputs retain the same sector integration support.
#[derive(Clone, Debug)]
pub struct GeneratedSector {
    pub(crate) cancellation_degree: usize,
    pub(crate) cancellation_terms: Vec<Vec<usize>>,
    pub(crate) conditioning_basis: ConditioningBasis,
    pub(crate) parameters: Vec<Symbol>,
    pub(crate) coefficients: Vec<AliasedAtom>,
    pub(crate) materialized: OnceLock<Vec<Atom>>,
    pub(crate) map: SectorMap,
}

impl GeneratedSector {
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
    pub(crate) metadata: super::GenerationMetadata,
    pub(crate) orders: Vec<i32>,
    pub(crate) sectors: Vec<GeneratedSector>,
    pub(crate) exact_coefficients: Vec<Atom>,
}

impl GeneratedIntegral {
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
