use numerica::domains::integer::Integer;
use std::ops::ControlFlow;

/// Domain of the *input* parameters, before sector transformations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParametricDomain {
    /// Homogeneous projective integrand with delta(1 - sum x). Each supplied
    /// factor polynomial must itself be homogeneous; the caller additionally
    /// verifies total integrand homogeneity including measure and exponents.
    ProjectiveSimplex,
    UnitCube,
    PositiveOrthant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecompositionPhase {
    Supports,
    Facets,
    Triangulation,
    Complete,
}

#[derive(Clone, Debug)]
pub struct DecompositionProgress {
    pub phase: DecompositionPhase,
    pub chart: usize,
    pub completed_constraints: usize,
    pub total_constraints: usize,
    pub rays: usize,
    pub sectors: usize,
}

#[derive(Clone, Debug)]
pub struct DecompositionOptions {
    pub max_support_pairs: usize,
    pub max_rays: usize,
    pub max_sectors: usize,
}

impl Default for DecompositionOptions {
    fn default() -> Self {
        Self {
            max_support_pairs: 10_000_000,
            max_rays: 1_000_000,
            max_sectors: 1_000_000,
        }
    }
}

/// x_i = product_j t_j^M_ij, with t in the open unit cube.
///
/// In projective charts the fixed parameter is one (its matrix row is zero).
/// The caller must use projective homogeneity to cancel the common scale; this
/// is a gauge-fixed map, not the normalized simplex coordinates themselves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorMap {
    pub fixed_parameter: Option<usize>,
    pub exponent_matrix: Vec<Vec<Integer>>,
    pub determinant: Integer,
    pub jacobian_powers: Vec<Integer>,
    /// Minimum exponent of each transformed polynomial along each coordinate.
    pub factor_valuations: Vec<Vec<Integer>>,
}

impl SectorMap {
    pub fn source_dimension(&self) -> usize {
        self.exponent_matrix.len()
    }
    pub fn dimension(&self) -> usize {
        self.jacobian_powers.len()
    }
}

#[derive(Clone, Debug)]
pub struct Decomposition {
    pub domain: ParametricDomain,
    pub sectors: Vec<SectorMap>,
    pub candidate_vertices: usize,
    pub geometric_vertices: usize,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum SectorError {
    #[error(transparent)]
    Work(#[from] crate::GeometryWorkError),
    #[error("invalid polynomial support: {0}")]
    InvalidSupport(String),
    #[error(
        "positive-orthant Newton polytope has affine rank {rank}, expected {dimension}; a scalelessness certificate or corrected input domain is required"
    )]
    RankDeficient { rank: usize, dimension: usize },
    #[error("sector decomposition cancelled")]
    Cancelled,
    #[error("sector decomposition resource limit: {resource} exceeded {limit}")]
    ResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    #[error("exact geometry invariant failed: {0}")]
    Geometry(String),
}

pub(crate) struct Monitor<'a> {
    pub callback: &'a mut dyn FnMut(&DecompositionProgress) -> ControlFlow<()>,
    pub status: DecompositionProgress,
    pub options: &'a DecompositionOptions,
}

impl Monitor<'_> {
    pub fn emit(&mut self) -> Result<(), SectorError> {
        if (self.callback)(&self.status).is_break() {
            Err(SectorError::Cancelled)
        } else {
            Ok(())
        }
    }
}
