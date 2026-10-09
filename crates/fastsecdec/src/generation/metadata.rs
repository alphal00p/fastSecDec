//! Retained chart semantics before endpoint subtraction. Complex contour maps
//! remain separate from positive-real sector measures and their geometry.
use fastsecdec_sectors::{ParametricDomain, SectorMap};
use symbolica::atom::{Atom, Symbol};
mod pre_subtraction;
pub use pre_subtraction::{EndpointPower, PreSubtractionMetadata, PreSubtractionTerm};

/// Branch convention and optional causal-contour capability of generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BranchPolicy {
    /// Historical artifacts whose generation performed a domain assessment.
    NoThresholdReal,
    /// No threshold certification or regularization was requested or performed.
    UserResponsible,
    /// Generated with an explicit F-directed contour; strength is runtime input.
    CausalContour,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FactorCertificate {
    /// No sign, boundary or interior threshold checks were performed.
    UncheckedUserResponsibility,
    PositiveCoefficients,
    NegativeCoefficientsIntegerPower,
    /// The caller asserted absence of interior thresholds. Boundary resolution
    /// was checked independently and is never supplied by this assertion.
    ExplicitInteriorAssertion,
}

#[derive(Clone, Debug)]
pub struct FactorAssessment {
    pub(crate) term_index: usize,
    pub(crate) factor_index: usize,
    pub(crate) polynomial: Atom,
    pub(crate) exponent: Atom,
    pub(crate) certificate: FactorCertificate,
}
impl FactorAssessment {
    pub fn term_index(&self) -> usize {
        self.term_index
    }
    pub fn factor_index(&self) -> usize {
        self.factor_index
    }
    pub fn polynomial(&self) -> &Atom {
        &self.polynomial
    }
    pub fn exponent(&self) -> &Atom {
        &self.exponent
    }
    pub fn certificate(&self) -> FactorCertificate {
        self.certificate
    }
}

#[derive(Clone, Debug)]
pub struct DomainAssessment {
    pub(crate) parameters: Vec<Symbol>,
    pub(crate) domain: ParametricDomain,
    pub(crate) branch: BranchPolicy,
    pub(crate) caller_asserted: bool,
    pub(crate) factors: Vec<FactorAssessment>,
}
impl DomainAssessment {
    pub fn parameters(&self) -> &[Symbol] {
        &self.parameters
    }
    pub fn domain(&self) -> ParametricDomain {
        self.domain
    }
    pub fn branch_policy(&self) -> BranchPolicy {
        self.branch
    }
    pub fn caller_asserted(&self) -> bool {
        self.caller_asserted
    }
    pub fn factors(&self) -> &[FactorAssessment] {
        &self.factors
    }
    pub fn relies_on_assertion(&self) -> bool {
        self.factors
            .iter()
            .any(|factor| factor.certificate == FactorCertificate::ExplicitInteriorAssertion)
    }
}

/// General expression boundary for the actual phase-one coordinate pullback.
/// `images[i]` replaces `source_parameters[i]` using unit-cube target symbols.
#[derive(Clone, Debug)]
pub struct CoordinateMap {
    pub(crate) source_parameters: Vec<Symbol>,
    pub(crate) target_parameters: Vec<Symbol>,
    pub(crate) images: Vec<Atom>,
    pub(crate) measure_jacobian: Atom,
    pub(crate) measure_factor: Atom,
    pub(crate) measure_powers: Vec<Atom>,
    pub(crate) source_domain: ParametricDomain,
    pub(crate) projective_fixed_parameter: Option<usize>,
}
impl CoordinateMap {
    pub fn source_parameters(&self) -> &[Symbol] {
        &self.source_parameters
    }
    pub fn target_parameters(&self) -> &[Symbol] {
        &self.target_parameters
    }
    pub fn images(&self) -> &[Atom] {
        &self.images
    }
    /// Positive real measure factor, not an oriented/complex determinant. For a
    /// projective chart it is the gauge-fixed measure after homogeneity removes
    /// the common scale; the images are not normalized simplex coordinates.
    pub fn measure_jacobian(&self) -> &Atom {
        &self.measure_jacobian
    }
    pub fn source_domain(&self) -> ParametricDomain {
        self.source_domain
    }
    pub fn projective_fixed_parameter(&self) -> Option<usize> {
        self.projective_fixed_parameter
    }
}

#[derive(Clone, Debug)]
pub struct ChartRecord {
    pub(crate) source_index: usize,
    pub(crate) representative: usize,
    pub(crate) representative_permutation: Vec<usize>,
    pub(crate) kernel_sector: Option<usize>,
    pub(crate) coordinates: CoordinateMap,
    pub(crate) geometry: SectorMap,
    pub(crate) pre_subtraction: Option<PreSubtractionMetadata>,
    pub(crate) contour: Option<crate::contour::ContourMetadata>,
}
impl ChartRecord {
    pub fn contour(&self) -> Option<&crate::contour::ContourMetadata> {
        self.contour.as_ref()
    }
    pub fn source_index(&self) -> usize {
        self.source_index
    }
    pub fn representative(&self) -> usize {
        self.representative
    }
    /// This chart's target parameter `i` maps to the representative chart's
    /// target parameter `permutation[i]` (not to an original input parameter).
    pub fn representative_permutation(&self) -> &[usize] {
        &self.representative_permutation
    }
    /// None denotes no numerical kernel at the requested orders: an exact,
    /// cancelled, or truncated-to-zero contribution. This does not retain
    /// separate exact coefficients for each chart.
    pub fn kernel_sector(&self) -> Option<usize> {
        self.kernel_sector
    }
    pub fn coordinates(&self) -> &CoordinateMap {
        &self.coordinates
    }
    pub fn geometry(&self) -> &SectorMap {
        &self.geometry
    }
    /// Original mapped terms before symmetry multiplicity, endpoint subtraction
    /// or Laurent expansion. Older artifacts explicitly lack this record.
    pub fn pre_subtraction(&self) -> Option<&PreSubtractionMetadata> {
        self.pre_subtraction.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct GenerationMetadata {
    pub(crate) domain: DomainAssessment,
    pub(crate) charts: Vec<ChartRecord>,
}
impl GenerationMetadata {
    pub fn domain_assessment(&self) -> &DomainAssessment {
        &self.domain
    }
    pub fn charts(&self) -> &[ChartRecord] {
        &self.charts
    }
}
