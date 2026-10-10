use super::{AliasRole, CellMap, GcadError, Result};
use std::{collections::BTreeMap, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
    prelude::Rational,
};
use symgcad::{
    algebra::{Algebra, Poly},
    output::{RootBound, RootIndexDomain},
};

#[derive(Debug)]
pub(super) struct LinearRoot {
    pub polynomial: usize,
    pub selector: RootIndexDomain,
    pub numerator: Poly,
    pub denominator: Poly,
    pub expression: Atom,
}
#[derive(Debug)]
pub struct LinearAxis {
    pub(super) lower: Option<Arc<LinearRoot>>,
    pub(super) upper: Option<Arc<LinearRoot>>,
}

/// Exact rational-function triangular map, with its general cell description.
/// The construction admits only finite linear integration bounds. Parameter
/// axes may be unbounded, but every finite parameter bound must also be linear.
#[derive(Debug)]
pub struct LinearCellMap {
    pub(super) source: Arc<CellMap>,
    pub(super) axes: Vec<LinearAxis>,
    parameter_count: usize,
    parameter_positive: Vec<Poly>,
    parameter_nonzero: Vec<Poly>,
}
impl LinearAxis {
    /// Finite root expression in preceding source coordinates and parameters.
    /// None denotes an unbounded parameter side, never a sampled enclosure.
    pub fn lower(&self) -> Option<&Atom> {
        self.lower.as_ref().map(|r| &r.expression)
    }
    pub fn upper(&self) -> Option<&Atom> {
        self.upper.as_ref().map(|r| &r.expression)
    }
}
impl LinearCellMap {
    pub(super) fn new(source: Arc<CellMap>) -> Result<Arc<Self>> {
        let raw = source.owner.native_result();
        let algebra = Algebra::new(&raw.order).map_err(|e| GcadError::Invalid(e.to_string()))?;
        let parameter_count = source
            .axes
            .iter()
            .take_while(|a| a.role == AliasRole::RuntimeParameter)
            .count();
        let mut roots = BTreeMap::<(usize, usize), Arc<LinearRoot>>::new();
        let mut make_root = |axis: usize, bound: &RootBound| -> Result<Arc<LinearRoot>> {
            if bound.index != 0 {
                return Err(GcadError::Unsupported(
                    "finite linear roots require native selector ordinal zero".into(),
                ));
            }
            let key = (axis, bound.polynomial);
            // A root can be referenced with different domains; keep that exact
            // choice attached, even though the algebraic expression is shared.
            if let Some(root) = roots.get(&key)
                && root.selector == bound.index_domain
            {
                return Ok(root.clone());
            }
            let text = raw.polynomials.get(bound.polynomial).ok_or_else(|| {
                GcadError::Invalid("root references a missing native polynomial".into())
            })?;
            let polynomial = algebra
                .parse(text)
                .map_err(|e| GcadError::Invalid(e.to_string()))?;
            if polynomial.degree(axis) != 1
                || (axis + 1..raw.order.len()).any(|i| polynomial.degree(i) > 0)
            {
                return Err(GcadError::Unsupported(format!(
                    "cell {} axis {} needs a nonlinear or nontriangular root section",
                    source.cell_index, axis
                )));
            }
            let coefficients = polynomial.to_univariate_polynomial_list(axis);
            let numerator = -coefficients
                .iter()
                .find(|(_, d)| *d == 0)
                .map_or_else(|| polynomial.zero(), |(p, _)| p.clone());
            let denominator = coefficients
                .iter()
                .find(|(_, d)| *d == 1)
                .ok_or_else(|| {
                    GcadError::Invalid("linear root lost its leading coefficient".into())
                })?
                .0
                .clone();
            let expression = numerator.to_expression() / denominator.to_expression();
            let expression =
                expression.replace_multiple(raw.order.iter().enumerate().map(|(i, name)| {
                    let alias = Symbol::parse(name, "symgcad")
                        .expect("native Algebra already validated alias");
                    Replacement::new(
                        Pattern::Literal(Atom::var(alias)),
                        Pattern::Literal(Atom::var(source.axes[i].symbol)),
                    )
                }));
            let root = Arc::new(LinearRoot {
                polynomial: bound.polynomial,
                selector: bound.index_domain,
                numerator,
                denominator,
                expression,
            });
            roots.insert(key, root.clone());
            Ok(root)
        };
        let mut axes = Vec::with_capacity(source.axes.len());
        for (index, (axis, role)) in source.native_axes().iter().zip(&source.axes).enumerate() {
            if role.role == AliasRole::IntegrationCoordinate
                && (axis.lower.is_none() || axis.upper.is_none())
            {
                return Err(GcadError::Unsupported(
                    "unbounded integration cells need a separately proved compactification".into(),
                ));
            }
            axes.push(LinearAxis {
                lower: axis
                    .lower
                    .as_ref()
                    .map(|b| make_root(index, b))
                    .transpose()?,
                upper: axis
                    .upper
                    .as_ref()
                    .map(|b| make_root(index, b))
                    .transpose()?,
            });
        }
        let parameter_only =
            |p: &Poly| (parameter_count..raw.order.len()).all(|i| p.degree(i) == 0);
        let parameter_positive = raw
            .normalized_constraints
            .iter()
            .map(|s| {
                algebra
                    .parse(s)
                    .map_err(|e| GcadError::Invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(parameter_only)
            .collect();
        let parameter_nonzero = raw
            .exceptional_polynomials
            .iter()
            .map(|index| {
                raw.polynomials
                    .get(*index)
                    .ok_or_else(|| GcadError::Invalid("missing exceptional polynomial".into()))
                    .and_then(|s| {
                        algebra
                            .parse(s)
                            .map_err(|e| GcadError::Invalid(e.to_string()))
                    })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(parameter_only)
            .collect();
        Ok(Arc::new(Self {
            source,
            axes,
            parameter_count,
            parameter_positive,
            parameter_nonzero,
        }))
    }
    pub fn axes(&self) -> &[LinearAxis] {
        &self.axes
    }
    pub fn source(&self) -> &Arc<CellMap> {
        &self.source
    }
    /// Bind an exact rational point in the actual parameter chamber. This is
    /// not a general symbolic/algebraic-fiber or exceptional-stratum admission.
    pub fn admit_parameters(
        self: &Arc<Self>,
        values: &BTreeMap<Symbol, Rational>,
    ) -> Result<ParameterAdmission> {
        let requested = &self.source.owner.request().kinematics().runtime_parameters;
        if values.len() != requested.len() || requested.iter().any(|s| !values.contains_key(s)) {
            return Err(GcadError::Invalid(
                "parameter admission needs exactly the request's runtime parameter values".into(),
            ));
        }
        let mut prefix = vec![Rational::zero(); self.source.axes.len()];
        for index in 0..self.parameter_count {
            let value = values[&self.source.axes[index].symbol].clone();
            let lower = self.axes[index]
                .lower
                .as_ref()
                .map(|r| r.evaluate(&prefix))
                .transpose()?;
            let upper = self.axes[index]
                .upper
                .as_ref()
                .map(|r| r.evaluate(&prefix))
                .transpose()?;
            if lower.as_ref().is_some_and(|l| &value <= l)
                || upper.as_ref().is_some_and(|u| &value >= u)
            {
                return Err(GcadError::Invalid(format!(
                    "parameter {} lies outside the cell's open chamber",
                    self.source.axes[index].symbol
                )));
            }
            prefix[index] = value;
        }
        if self
            .parameter_positive
            .iter()
            .any(|p| p.replace_all(&prefix) <= Rational::zero())
            || self
                .parameter_nonzero
                .iter()
                .any(|p| p.replace_all(&prefix).is_zero())
        {
            return Err(GcadError::Invalid(
                "parameter point violates a strict constraint or excluded exceptional polynomial"
                    .into(),
            ));
        }
        prefix.truncate(self.parameter_count);
        Ok(ParameterAdmission {
            map: self.clone(),
            values: values.clone(),
            ordered: prefix,
        })
    }
}
impl LinearRoot {
    pub(super) fn evaluate(&self, prefix: &[Rational]) -> Result<Rational> {
        let leading = self.denominator.replace_all(prefix);
        if leading.is_zero() {
            return Err(GcadError::Invalid(format!(
                "linear root polynomial {} changes degree at this point",
                self.polynomial
            )));
        }
        let root = self.numerator.replace_all(prefix) / leading;
        if self.selector == RootIndexDomain::Positive && root <= Rational::zero() {
            return Err(GcadError::Invalid(
                "positive-root selector has no strictly positive linear root".into(),
            ));
        }
        Ok(root)
    }
}

/// A parameter point tied to the immutable map/proof that admitted it.
#[derive(Clone, Debug)]
pub struct ParameterAdmission {
    pub(super) map: Arc<LinearCellMap>,
    pub(super) values: BTreeMap<Symbol, Rational>,
    ordered: Vec<Rational>,
}
#[derive(Debug)]
pub struct RationalMappedPoint {
    pub prepared_coordinates: Vec<Rational>,
    pub original_coordinates: Vec<Rational>,
    pub measure: Rational,
}
impl ParameterAdmission {
    pub fn map(&self) -> &Arc<LinearCellMap> {
        &self.map
    }
    pub fn values(&self) -> &BTreeMap<Symbol, Rational> {
        &self.values
    }
    /// Exact interior control using native polynomial evaluation. This checks
    /// actual leading coefficients, selector domains and positive widths.
    pub fn map_rational(&self, unit: &[Rational]) -> Result<RationalMappedPoint> {
        if unit.len() != self.map.source.unit_coordinates.len()
            || unit
                .iter()
                .any(|v| v <= &Rational::zero() || v >= &Rational::one())
        {
            return Err(GcadError::Invalid(
                "cell map requires a point in its open unit cube".into(),
            ));
        }
        let mut point = self.ordered.clone();
        point.resize(self.map.source.axes.len(), Rational::zero());
        let mut measure = Rational::one();
        for index in self.map.parameter_count..self.map.axes.len() {
            let axis = &self.map.axes[index];
            let lower = axis
                .lower
                .as_ref()
                .expect("finite linear lowering")
                .evaluate(&point)?;
            let upper = axis
                .upper
                .as_ref()
                .expect("finite linear lowering")
                .evaluate(&point)?;
            let width = upper - &lower;
            if width <= Rational::zero() {
                return Err(GcadError::Invalid(
                    "cell map has a nonpositive interval width".into(),
                ));
            }
            point[index] = lower + &width * &unit[index - self.map.parameter_count];
            measure *= width;
        }
        let mapped = self
            .map
            .source
            .axes
            .iter()
            .zip(&point)
            .map(|(axis, value)| (axis.symbol, value.clone()))
            .collect::<BTreeMap<_, _>>();
        let request = self.map.source.owner.request();
        let prepared_coordinates = request
            .domain()
            .coordinates()
            .iter()
            .map(|s| mapped[s].clone())
            .collect::<Vec<_>>();
        let original_coordinates = if let Some(preparation) = request.projective_preparation() {
            preparation
                .images()
                .iter()
                .map(|image| {
                    let value = image.replace_multiple(preparation.coordinates().iter().map(|s| {
                        Replacement::new(
                            Pattern::Literal(Atom::var(*s)),
                            Pattern::Literal(Atom::num(mapped[s].clone())),
                        )
                    }));
                    Rational::try_from(value.as_view())
                        .map_err(|e| GcadError::Invalid(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            prepared_coordinates.clone()
        };
        Ok(RationalMappedPoint {
            prepared_coordinates,
            original_coordinates,
            measure,
        })
    }
}
