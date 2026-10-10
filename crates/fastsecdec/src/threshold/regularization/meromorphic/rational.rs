//! Exact Q(i)(epsilon) admission and one shared regular witness.
//! Native Symbolica owns all field/polynomial arithmetic. This certificate is
//! local in epsilon; it does not declare an entire convergence strip regular.
use std::{ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::{atom::AtomField, float::Complex, rational::Rational},
    poly::PolyVariable,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Unsupported(&'static str),
    Invalid(&'static str),
    ResourceIncomplete(&'static str),
    Cancelled,
    Native(String),
}
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub source_nodes: usize,
    pub source_depth: usize,
    pub degree: u16,
    pub terms: usize,
    pub integer_power: i64,
    pub witness_candidates: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            source_nodes: 4096,
            source_depth: 64,
            degree: 256,
            terms: 4096,
            integer_power: 256,
            witness_candidates: 1024,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RationalPrefactor {
    original: Atom,
    numerator: Atom,
    denominator: Atom,
    regulator: Symbol,
    degree: u16,
}
impl RationalPrefactor {
    pub fn original(&self) -> &Atom {
        &self.original
    }
    pub fn numerator(&self) -> &Atom {
        &self.numerator
    }
    pub fn denominator(&self) -> &Atom {
        &self.denominator
    }
    pub fn regulator(&self) -> Symbol {
        self.regulator
    }
    pub fn denominator_degree(&self) -> u16 {
        self.degree
    }

    /// A bounded native syntax check prevents algebraic conversion from
    /// rationalizing a Float or expanding the accepted coefficient field.
    pub fn admit(original: Atom, regulator: Symbol, limits: Limits) -> Result<Self> {
        if limits.integer_power < 0 {
            return Err(Error::Invalid("negative prefactor integer-power limit"));
        }
        // Native recombination can add the numerator/denominator degrees.
        // Keep even caller-enlarged limits inside the native exponent width.
        if limits.degree > u16::MAX / 2 {
            return Err(Error::Invalid(
                "prefactor degree limit exceeds safe native exponent range",
            ));
        }
        expression_bound(original.as_view(), limits, 0, &mut 0)?;
        let mut failure = None;
        let mut count = 0usize;
        original.visitor(&mut |node| {
            if failure.is_some() {
                return false;
            }
            count += 1;
            if count > limits.source_nodes {
                failure = Some(Error::ResourceIncomplete("prefactor source nodes"));
                return false;
            }
            failure = match node {
                AtomView::Num(_) if Complex::<Rational>::try_from(node).is_err() => Some(
                    Error::Unsupported("prefactor numeric coefficient is not exact Q(i)"),
                ),
                AtomView::Var(v) if v.get_symbol() != regulator => Some(Error::Unsupported(
                    "prefactor contains an undeclared variable",
                )),
                AtomView::Fun(_) => Some(Error::Unsupported(
                    "prefactor function is outside the rational class",
                )),
                AtomView::Pow(p) => match Rational::try_from(p.get_exp()) {
                    Ok(exponent) if exponent.is_integer() => {
                        match exponent.numerator_ref().to_i64() {
                            Some(value)
                                if value >= -limits.integer_power
                                    && value <= limits.integer_power =>
                            {
                                None
                            }
                            _ => Some(Error::ResourceIncomplete("prefactor integer power")),
                        }
                    }
                    _ => Some(Error::Unsupported(
                        "prefactor power is not an exact integer",
                    )),
                },
                _ => None,
            };
            failure.is_none()
        });
        if let Some(error) = failure {
            return Err(error);
        }
        // The single preflight above bounded both numerator and denominator
        // before invoking native conversion; no cancellation is assumed.

        let variables = Arc::new(vec![PolyVariable::from(regulator)]);
        let (mut context, rational) = original
            .to_rational_polynomial_in_algebraic_extension::<u16>(variables.clone())
            .map_err(Error::Native)?;
        if rational.numerator.variables() != &variables
            || rational.denominator.variables() != &variables
        {
            return Err(Error::Unsupported("native prefactor variable map grew"));
        }
        if rational.denominator.is_zero() {
            return Err(Error::Invalid("zero native prefactor denominator"));
        }
        for polynomial in [&rational.numerator, &rational.denominator] {
            if polynomial.degree(0) > limits.degree || polynomial.nterms() > limits.terms {
                return Err(Error::ResourceIncomplete("prefactor polynomial size"));
            }
        }
        // A trivial Q extension needs no preferred algebraic representative.
        // The context's preferred zero-generator representation currently
        // evaluates 0^0 when converting a constant; use the native polynomial
        // conversion for this exact trivial-field case. Retain the context for
        // Q(i), where the embedding of the generator fixes its complex sign.
        let (numerator, denominator) = if context.is_trivial() {
            (
                rational.numerator.to_expression(),
                rational.denominator.to_expression(),
            )
        } else {
            (
                context
                    .polynomial_to_expression(&rational.numerator)
                    .map_err(Error::Native)?,
                context
                    .polynomial_to_expression(&rational.denominator)
                    .map_err(Error::Native)?,
            )
        };
        // Independently fence the output coefficients; conversion into an
        // algebraic extension alone never proves they belong to Q(i).
        let field = AtomField {
            statistical_zero_test: false,
            ..AtomField::new()
        };
        for expression in [&numerator, &denominator] {
            let polynomial = expression.to_polynomial_in_vars_with_field::<u16>(regulator, &field);
            if (&polynomial)
                .into_iter()
                .any(|term| Complex::<Rational>::try_from(term.coefficient.as_view()).is_err())
            {
                return Err(Error::Unsupported(
                    "canonical prefactor coefficients are not exact Q(i)",
                ));
            }
        }
        let recombination = &original * &denominator - &numerator;
        expression_bound(recombination.as_view(), limits, 0, &mut 0)?;
        let identity = context
            .to_rational_polynomial::<u16>(recombination.as_view(), variables.clone())
            .map_err(Error::Native)?;
        if identity.numerator.variables() != &variables || !identity.numerator.is_zero() {
            return Err(Error::Invalid("native prefactor recombination failed"));
        }
        Ok(Self {
            original,
            numerator,
            denominator,
            regulator,
            degree: rational.denominator.degree(0),
        })
    }
    fn denominator_at(&self, epsilon: &Rational) -> Result<Complex<Rational>> {
        let value = self
            .denominator
            .replace(Atom::var(self.regulator))
            .with(Atom::num(epsilon.clone()));
        Complex::<Rational>::try_from(value.as_view())
            .map_err(|_| Error::Invalid("non-exact prefactor witness value"))
    }
}

/// Conservative native-syntax bookkeeping before polynomial conversion. No
/// coefficients, cancellations, polynomial factorization or CAS are implemented.
pub(crate) fn preflight_expression(expression: &Atom, limits: Limits) -> Result<()> {
    expression_bound(expression.as_view(), limits, 0, &mut 0).map(|_| ())
}

#[derive(Clone, Copy)]
struct ExpressionBound {
    numerator_degree: usize,
    denominator_degree: usize,
    numerator_terms: usize,
    denominator_terms: usize,
}
fn expression_bound(
    node: AtomView<'_>,
    limits: Limits,
    depth: usize,
    nodes: &mut usize,
) -> Result<ExpressionBound> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(Error::ResourceIncomplete("prefactor preflight node count"))?;
    if depth > limits.source_depth || *nodes > limits.source_nodes {
        return Err(Error::ResourceIncomplete("prefactor preflight depth/nodes"));
    }
    let add = |a: usize, b: usize| {
        a.checked_add(b)
            .ok_or(Error::ResourceIncomplete("prefactor expansion bound"))
    };
    let mul = |a: usize, b: usize| {
        a.checked_mul(b)
            .ok_or(Error::ResourceIncomplete("prefactor expansion bound"))
    };
    let constant = ExpressionBound {
        numerator_degree: 0,
        denominator_degree: 0,
        numerator_terms: 1,
        denominator_terms: 1,
    };
    let checked = |b: ExpressionBound| {
        if b.numerator_degree > usize::from(limits.degree)
            || b.denominator_degree > usize::from(limits.degree)
            || b.numerator_terms > limits.terms
            || b.denominator_terms > limits.terms
        {
            Err(Error::ResourceIncomplete(
                "prefactor preconversion degree/term bound",
            ))
        } else {
            Ok(b)
        }
    };
    let value = match node {
        AtomView::Num(_) => constant,
        AtomView::Var(_) => ExpressionBound {
            numerator_degree: 1,
            ..constant
        },
        AtomView::Add(sum) => {
            let mut acc = None;
            for term in sum.iter() {
                let next = expression_bound(term, limits, depth + 1, nodes)?;
                acc = Some(if let Some(old) = acc {
                    let old: ExpressionBound = old;
                    checked(ExpressionBound {
                        numerator_degree: add(old.numerator_degree, next.denominator_degree)?
                            .max(add(next.numerator_degree, old.denominator_degree)?),
                        denominator_degree: add(old.denominator_degree, next.denominator_degree)?,
                        numerator_terms: add(
                            mul(old.numerator_terms, next.denominator_terms)?,
                            mul(next.numerator_terms, old.denominator_terms)?,
                        )?,
                        denominator_terms: mul(old.denominator_terms, next.denominator_terms)?,
                    })?
                } else {
                    next
                });
            }
            acc.unwrap_or(constant)
        }
        AtomView::Mul(product) => {
            let mut acc = constant;
            for factor in product.iter() {
                let next = expression_bound(factor, limits, depth + 1, nodes)?;
                acc = checked(ExpressionBound {
                    numerator_degree: add(acc.numerator_degree, next.numerator_degree)?,
                    denominator_degree: add(acc.denominator_degree, next.denominator_degree)?,
                    numerator_terms: mul(acc.numerator_terms, next.numerator_terms)?,
                    denominator_terms: mul(acc.denominator_terms, next.denominator_terms)?,
                })?;
            }
            acc
        }
        AtomView::Pow(power) => {
            let mut base = expression_bound(power.get_base(), limits, depth + 1, nodes)?;
            let exponent = Rational::try_from(power.get_exp())
                .ok()
                .filter(|r| r.is_integer())
                .and_then(|r| r.numerator_ref().to_i64())
                .ok_or(Error::Unsupported("noninteger preflight power"))?;
            if exponent < 0 {
                std::mem::swap(&mut base.numerator_degree, &mut base.denominator_degree);
                std::mem::swap(&mut base.numerator_terms, &mut base.denominator_terms);
            }
            let count = u32::try_from(exponent.unsigned_abs())
                .map_err(|_| Error::ResourceIncomplete("prefactor power preflight"))?;
            ExpressionBound {
                numerator_degree: mul(base.numerator_degree, count as usize)?,
                denominator_degree: mul(base.denominator_degree, count as usize)?,
                numerator_terms: base
                    .numerator_terms
                    .checked_pow(count)
                    .ok_or(Error::ResourceIncomplete("prefactor term power bound"))?,
                denominator_terms: base
                    .denominator_terms
                    .checked_pow(count)
                    .ok_or(Error::ResourceIncomplete("prefactor term power bound"))?,
            }
        }
        _ => return Err(Error::Unsupported("nonrational preflight syntax")),
    };
    checked(value)
}

/// Immutable proof of one common regular point; continuity gives a nonempty
/// complex neighborhood. No numerical radius or whole-strip claim is made.
#[derive(Clone, Debug)]
pub struct SharedWitness {
    prefactors: Vec<RationalPrefactor>,
    strip: (Option<Rational>, Option<Rational>),
    epsilon: Rational,
    denominator_values: Vec<Complex<Rational>>,
    tested: usize,
}
impl SharedWitness {
    pub fn prefactors(&self) -> &[RationalPrefactor] {
        &self.prefactors
    }
    pub fn strip(&self) -> (&Option<Rational>, &Option<Rational>) {
        (&self.strip.0, &self.strip.1)
    }
    pub fn epsilon(&self) -> &Rational {
        &self.epsilon
    }
    pub fn denominator_values(&self) -> &[Complex<Rational>] {
        &self.denominator_values
    }
    pub fn tested(&self) -> usize {
        self.tested
    }
    pub fn construct(
        prefactors: &[RationalPrefactor],
        lower: Option<&Rational>,
        upper: Option<&Rational>,
        limits: Limits,
        observer: impl FnMut(usize) -> ControlFlow<()>,
    ) -> Result<Self> {
        Self::construct_with(prefactors, lower, upper, limits, 0, |_| true, observer)
    }
    pub(super) fn construct_with(
        prefactors: &[RationalPrefactor],
        lower: Option<&Rational>,
        upper: Option<&Rational>,
        limits: Limits,
        additional_excluded_points: usize,
        mut allowed: impl FnMut(&Rational) -> bool,
        mut observer: impl FnMut(usize) -> ControlFlow<()>,
    ) -> Result<Self> {
        if let (Some(l), Some(u)) = (lower, upper)
            && l >= u
        {
            return Err(Error::Invalid("empty open endpoint strip"));
        }
        if prefactors
            .windows(2)
            .any(|pair| pair[0].regulator != pair[1].regulator)
        {
            return Err(Error::Invalid("prefactor regulator mismatch"));
        }
        let (a, b) = inner_interval(lower, upper)?;
        let required = prefactors
            .iter()
            .try_fold(
                additional_excluded_points
                    .checked_add(1)
                    .ok_or(Error::ResourceIncomplete("pole-family count"))?,
                |count, p| count.checked_add(usize::from(p.degree)),
            )
            .ok_or(Error::ResourceIncomplete("prefactor degree sum"))?;
        if required > limits.witness_candidates {
            return Err(Error::ResourceIncomplete(
                "shared epsilon witness candidates",
            ));
        }
        let denominator = required
            .checked_add(1)
            .and_then(|n| i64::try_from(n).ok())
            .ok_or(Error::ResourceIncomplete(
                "shared epsilon candidate denominator",
            ))?;
        for index in 0..required {
            if observer(index).is_break() {
                return Err(Error::Cancelled);
            }
            let numerator = i64::try_from(index + 1)
                .map_err(|_| Error::ResourceIncomplete("shared epsilon candidate numerator"))?;
            let epsilon = &a + &((&b - &a) * Rational::from((numerator, denominator)));
            let values = prefactors
                .iter()
                .map(|p| p.denominator_at(&epsilon))
                .collect::<Result<Vec<_>>>()?;
            if values.iter().all(|v| !v.re.is_zero() || !v.im.is_zero()) && allowed(&epsilon) {
                return Ok(Self {
                    prefactors: prefactors.to_vec(),
                    strip: (lower.cloned(), upper.cloned()),
                    epsilon,
                    denominator_values: values,
                    tested: index + 1,
                });
            }
        }
        Err(Error::Invalid(
            "native nonzero polynomials exceeded the exact root-count bound",
        ))
    }
}

pub(super) fn inner_interval(
    lower: Option<&Rational>,
    upper: Option<&Rational>,
) -> Result<(Rational, Rational)> {
    if let (Some(l), Some(u)) = (lower, upper)
        && l >= u
    {
        return Err(Error::Invalid("empty open endpoint strip"));
    }
    let (a, b) = match (lower, upper) {
        (Some(l), Some(u)) => {
            let third = (u - l) / Rational::from(3);
            (l + &third, u - &third)
        }
        (Some(l), None) => (l + &Rational::one(), l + &Rational::from(2)),
        (None, Some(u)) => (u - &Rational::from(2), u - &Rational::one()),
        (None, None) => (Rational::from(-1), Rational::one()),
    };
    Ok((a, b))
}
