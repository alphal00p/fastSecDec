use std::collections::BTreeSet;

use numerica::numerical_integration::qmc::{QmcPlan, Rank1Rule};
use serde::{Deserialize, Serialize};

use super::{CoefficientComponent, IntegrationError, PublishedLattice, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectorSpec {
    pub id: u64,
    pub dimension: usize,
}

/// Immutable scientific identity and complete common Laurent-vector layout.
///
/// `content_id` must identify all generated expressions and evaluation settings,
/// not merely dimensions or coefficient orders. Whole zero-dimensional sectors
/// belong in `exact_coefficients`; positive-dimensional sectors retain every
/// original integration axis and every output, including constant components.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegrationProblem {
    pub content_id: String,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub sectors: Vec<SectorSpec>,
    pub exact_coefficients: Vec<f64>,
}

impl IntegrationProblem {
    pub fn new(
        content_id: String,
        orders: Vec<i32>,
        sectors: Vec<SectorSpec>,
        exact_coefficients: Vec<f64>,
    ) -> Result<Self> {
        let components = vec![CoefficientComponent::Real; orders.len()];
        Self::new_with_components(content_id, orders, components, sectors, exact_coefficients)
    }

    pub fn new_with_components(
        content_id: String,
        orders: Vec<i32>,
        components: Vec<CoefficientComponent>,
        sectors: Vec<SectorSpec>,
        exact_coefficients: Vec<f64>,
    ) -> Result<Self> {
        let problem = Self {
            content_id,
            orders,
            components,
            sectors,
            exact_coefficients,
        };
        problem.validate()?;
        Ok(problem)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let invalid = |s: &str| IntegrationError::Invalid(s.into());
        if self.content_id.is_empty() {
            return Err(invalid("a complete integrand content identity is required"));
        }
        if self.orders.is_empty()
            || self.components.len() != self.orders.len()
            || self
                .orders
                .iter()
                .zip(&self.components)
                .zip(self.orders.iter().zip(&self.components).skip(1))
                .any(|(a, b)| a >= b)
        {
            return Err(invalid(
                "Laurent (order,component) pairs must be nonempty, aligned and strictly increasing",
            ));
        }
        if self.exact_coefficients.len() != self.orders.len()
            || self.exact_coefficients.iter().any(|v| !v.is_finite())
        {
            return Err(invalid(
                "exact coefficient vector has the wrong shape or nonfinite values",
            ));
        }
        if self.sectors.len() > u32::MAX as usize || self.sectors.iter().any(|s| s.dimension == 0) {
            return Err(invalid(
                "stochastic sectors require positive dimensions and at most u32::MAX sectors",
            ));
        }
        if self
            .sectors
            .iter()
            .map(|s| s.id)
            .collect::<BTreeSet<_>>()
            .len()
            != self.sectors.len()
        {
            return Err(invalid("sector identities must be unique"));
        }
        self.orders
            .len()
            .checked_mul(self.orders.len())
            .ok_or_else(|| invalid("covariance dimensions overflow"))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Periodization {
    None,
    #[default]
    Korobov3,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSource {
    #[default]
    Kuo,
    /// A caller-supplied generating vector, truncated only to each sector's
    /// declared full dimension. No generating-vector search is performed.
    Supplied(Vec<u64>),
    /// Explicit attributed catalogue; unsupported counts or full sector
    /// dimensions are errors rather than a fallback to another vector.
    Published(PublishedLattice),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QmcSettings {
    pub points: u64,
    pub shifts: u32,
    pub seed: u64,
    pub package_points: u64,
    pub periodization: Periodization,
    pub rule: RuleSource,
}

impl Default for QmcSettings {
    fn default() -> Self {
        Self {
            points: 4096,
            shifts: 64,
            seed: 0,
            package_points: 1024,
            periodization: Periodization::Korobov3,
            rule: RuleSource::Kuo,
        }
    }
}

impl QmcSettings {
    /// Validate configuration before constructing kernels or scheduling work.
    ///
    /// Published count bounds are checked even for an all-exact problem.
    /// Full sector dimensions and supplied-vector coprimality are checked when
    /// the session constructs each concrete native rule.
    pub fn validate(&self) -> Result<()> {
        if self.shifts < 2 || self.package_points == 0 || !(2..=1u64 << 53).contains(&self.points) {
            return Err(IntegrationError::Invalid(
                "at least two shifts, a positive package size and a lattice size in 2..=2^53 are required".into(),
            ));
        }
        if let Some(catalogue) = match self.rule {
            RuleSource::Kuo => Some(PublishedLattice::Kuo33002),
            RuleSource::Published(catalogue) => Some(catalogue),
            RuleSource::Supplied(_) => None,
        } {
            // Reuse the catalog constructor for supported-count validation,
            // including an all-exact problem that does not need worker plans.
            Rank1Rule::published(catalogue, self.points, 1)?;
        }
        if matches!(&self.rule, RuleSource::Supplied(vector) if vector.is_empty()) {
            return Err(IntegrationError::Invalid(
                "a supplied generating vector cannot be empty".into(),
            ));
        }
        self.points
            .checked_mul(self.shifts as u64)
            .ok_or_else(|| IntegrationError::Invalid("QMC total point count overflow".into()))?;
        Ok(())
    }

    pub(crate) fn plan(
        &self,
        dimension: usize,
        points: u64,
        shifts: u32,
        stream: u64,
    ) -> Result<QmcPlan> {
        self.validate()?;
        if shifts < 2 {
            return Err(IntegrationError::Invalid(
                "production allocations require at least two shifts".into(),
            ));
        }
        let rule = match &self.rule {
            RuleSource::Kuo => Rank1Rule::kuo(points, dimension)?,
            RuleSource::Published(catalogue) => {
                Rank1Rule::published(*catalogue, points, dimension)?
            }
            RuleSource::Supplied(vector) => {
                let prefix = vector.get(..dimension).ok_or_else(|| {
                    IntegrationError::Invalid(
                        "supplied lattice vector is shorter than a sector's full dimension".into(),
                    )
                })?;
                Rank1Rule::new(points, prefix.to_vec())?
            }
        };
        Ok(QmcPlan::new(rule, shifts, self.seed, stream)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
}

impl Tolerance {
    pub fn new(absolute: f64, relative: f64) -> Result<Self> {
        let value = Self { absolute, relative };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(self) -> Result<()> {
        if !self.absolute.is_finite()
            || !self.relative.is_finite()
            || self.absolute < 0.0
            || self.relative < 0.0
        {
            return Err(IntegrationError::Invalid(
                "absolute and relative tolerances must be finite and nonnegative".into(),
            ));
        }
        Ok(())
    }
}
