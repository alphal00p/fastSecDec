//! Native directional Hessian series and sufficient polynomial/spectral bounds.
use super::{
    displacement_cap_symbol, lambda_cap_symbol, radius_fraction_symbol, safety_fraction_symbol,
};
use crate::generation::GenerationError;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{atom::AtomField, integer::Integer, rational::Rational},
    id::{Pattern, Replacement},
    symbol,
};

/// Smooth positive envelope of a real argument. The two forms are exactly the
/// same function. Runtime lowering must select the rationalized form for a
/// negative argument, and propagate derivatives of the mathematical function.
/// Keeping these native expressions is not a production callback implementation.
#[derive(Clone, Debug)]
pub struct SmoothPositivePart {
    argument: Atom,
    expression: Atom,
    negative_argument_expression: Atom,
}
impl SmoothPositivePart {
    fn new(argument: Atom, regularity: &Atom) -> Self {
        let square_root = (argument.pow(2) + regularity.pow(2)).sqrt();
        let expression = (&argument + &square_root) / Atom::num(2);
        let negative_argument_expression =
            regularity.pow(2) / (Atom::num(2) * (square_root - &argument));
        Self {
            argument,
            expression,
            negative_argument_expression,
        }
    }
    pub fn argument(&self) -> &Atom {
        &self.argument
    }
    pub fn expression(&self) -> &Atom {
        &self.expression
    }
    pub fn negative_argument_expression(&self) -> &Atom {
        &self.negative_argument_expression
    }
}

/// One odd F ray order, with full-sector (not face-dependent) matrix dimension.
#[derive(Clone, Debug)]
pub struct CausalEnvelopeTerm {
    order: u32,
    hessian_ray: Vec<Atom>,
    squared_bound: Atom,
    spectral_mean: Atom,
    spectral_gap_squared: Atom,
    positive_bound: SmoothPositivePart,
}
impl CausalEnvelopeTerm {
    pub fn order(&self) -> u32 {
        self.order
    }
    /// Row-major T_k = D^k F[v^(k-2), ., .].
    pub fn hessian_ray(&self) -> &[Atom] {
        &self.hessian_ray
    }
    pub fn squared_bound(&self) -> &Atom {
        &self.squared_bound
    }
    pub fn spectral_mean(&self) -> &Atom {
        &self.spectral_mean
    }
    /// Sum-of-squares form of (n-1)/n * (tr(R^2)-tr(R)^2/n).
    pub fn spectral_gap_squared(&self) -> &Atom {
        &self.spectral_gap_squared
    }
    pub fn positive_bound(&self) -> &SmoothPositivePart {
        &self.positive_bound
    }
}

#[derive(Clone, Debug)]
pub struct PositiveEnvelopeTerm {
    order: u32,
    ray_coefficient: Atom,
    squared_bound: Atom,
    positive_bound: SmoothPositivePart,
}
impl PositiveEnvelopeTerm {
    pub fn order(&self) -> u32 {
        self.order
    }
    /// Real coefficient of t^k in U(x-i*t*v), including (-1)^(k/2).
    pub fn ray_coefficient(&self) -> &Atom {
        &self.ray_coefficient
    }
    pub fn squared_bound(&self) -> &Atom {
        &self.squared_bound
    }
    pub fn positive_bound(&self) -> &SmoothPositivePart {
        &self.positive_bound
    }
}

/// The exact positive constant is a sufficient lower bound for U throughout
/// the closed cube, established using its native nonnegative coefficients.
#[derive(Clone, Debug)]
pub struct PositiveEnvelope {
    polynomial: Atom,
    lower_bound: Atom,
    terms: Vec<PositiveEnvelopeTerm>,
}
impl PositiveEnvelope {
    pub fn polynomial(&self) -> &Atom {
        &self.polynomial
    }
    pub fn certified_lower_bound(&self) -> &Atom {
        &self.lower_bound
    }
    pub fn terms(&self) -> &[PositiveEnvelopeTerm] {
        &self.terms
    }
}

/// Factored native expressions for both approved sufficient-radius equations.
/// H(u)=1 determines the dimensionless radius; lambda=S*L*u. The builder never
/// divides by A, never chooses a root, and never changes dimension on a face.
#[derive(Clone, Debug)]
pub struct DynamicEnvelope {
    parameters: Vec<Symbol>,
    causal_polynomial: Atom,
    weights: Vec<Atom>,
    direction: Vec<Atom>,
    leading_causal_magnitude: Atom,
    causal_terms: Vec<CausalEnvelopeTerm>,
    positive_factors: Vec<PositiveEnvelope>,
    polynomial_level: Atom,
    sign_aware_level: Atom,
    regularity: Atom,
}
impl DynamicEnvelope {
    pub fn new(
        parameters: &[Symbol],
        causal_polynomial: Atom,
        positive_polynomials: &[Atom],
    ) -> Result<Self, GenerationError> {
        let auxiliary = Auxiliary::new(parameters.len());
        let reserved = [
            lambda_cap_symbol(),
            displacement_cap_symbol(),
            safety_fraction_symbol(),
            radius_fraction_symbol(),
            super::super::lambda_symbol(),
            auxiliary.ray,
        ];
        let reserved = reserved
            .into_iter()
            .chain(auxiliary.directions.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        if parameters
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != parameters.len()
            || parameters.iter().any(|p| reserved.contains(p))
            || std::iter::once(&causal_polynomial)
                .chain(positive_polynomials)
                .any(|polynomial| reserved.iter().any(|s| polynomial.contains_symbol(*s)))
        {
            return Err(invalid(
                "dynamic contour coordinates/inputs collide with reserved symbols",
            ));
        }
        let degree = polynomial_degree(&causal_polynomial, parameters)?;
        if degree > u32::MAX / 2 {
            return Err(invalid(
                "dynamic F degree exceeds the envelope exponent range",
            ));
        }
        if causal_polynomial.is_zero() || !real(&causal_polynomial) {
            return Err(invalid(
                "dynamic contour F must be nonzero and real for real inputs",
            ));
        }
        let weights = parameters
            .iter()
            .map(|p| Atom::var(*p) * (Atom::one() - Atom::var(*p)))
            .collect::<Vec<_>>();
        let gradients = parameters
            .iter()
            .map(|p| causal_polynomial.derivative(*p))
            .collect::<Vec<_>>();
        let direction = weights
            .iter()
            .zip(&gradients)
            .map(|(w, gradient)| w * gradient)
            .collect::<Vec<_>>();
        let leading_causal_magnitude = weights
            .iter()
            .zip(&gradients)
            .map(|(w, gradient)| w * gradient.pow(2))
            .sum();
        let regularity = Atom::num((1, 1000));
        let cap = Atom::var(lambda_cap_symbol());
        let fraction = Atom::var(radius_fraction_symbol());
        let base = base_level(&direction);
        let mut polynomial_level = base.clone();
        let mut sign_aware_level = base;

        // Keep direction placeholders independent during the auxiliary series.
        // Substitute v afterwards so physical derivatives include derivatives
        // of v, while the ray derivative itself holds v fixed.
        let mut causal_terms = Vec::new();
        let orders = (3..=degree).step_by(2).collect::<Vec<_>>();
        let matrix_size = parameters
            .len()
            .checked_mul(parameters.len())
            .ok_or_else(|| invalid("dynamic Hessian dimension overflows"))?;
        let mut matrices = vec![vec![Atom::Zero; matrix_size]; orders.len()];
        if let Some(maximum) = orders.last() {
            for (i, x) in parameters.iter().enumerate() {
                for (j, y) in parameters.iter().enumerate().skip(i) {
                    let hessian = causal_polynomial.derivative(*x).derivative(*y);
                    if hessian.is_zero() {
                        continue;
                    }
                    let hessian_degree = polynomial_degree(&hessian, parameters)?;
                    if hessian_degree == 0 {
                        continue;
                    }
                    let ray = auxiliary.ray(&hessian, parameters);
                    let series = ray
                        .series(
                            auxiliary.ray,
                            0,
                            (*maximum - 2).min(hessian_degree) as usize,
                        )
                        .map_err(|error| invalid(format!("native Hessian ray series: {error}")))?;
                    for (term, order) in matrices.iter_mut().zip(&orders) {
                        if *order - 2 > hessian_degree {
                            continue;
                        }
                        let coefficient = series
                            .coefficient(i64::from(*order - 2).into())
                            .ok_or_else(|| invalid("missing exact Hessian ray coefficient"))?;
                        let value = auxiliary.bind(
                            &(Atom::num(Integer::factorial(*order - 2)) * coefficient),
                            &direction,
                        );
                        term[i * parameters.len() + j] = value.clone();
                        term[j * parameters.len() + i] = value;
                    }
                }
            }
        }
        for (order, hessian_ray) in orders.into_iter().zip(matrices) {
            let term = causal_term(order, hessian_ray, &weights, &cap, &regularity);
            polynomial_level += Atom::num(((degree - 1) / 2) as i64)
                * &term.squared_bound
                * (&cap * &fraction).pow(i64::from(2 * order - 2));
            causal_terms.push(term);
        }
        let causal_envelope = causal_terms
            .iter()
            .map(|term| term.positive_bound.expression() * fraction.pow(i64::from(term.order - 1)))
            .sum::<Atom>();
        sign_aware_level += causal_envelope.pow(2);

        let mut positive_factors = Vec::with_capacity(positive_polynomials.len());
        for polynomial in positive_polynomials {
            let u_degree = polynomial_degree(polynomial, parameters)?;
            if u_degree > u32::MAX / 2 {
                return Err(invalid(
                    "dynamic U degree exceeds the envelope exponent range",
                ));
            }
            let lower_bound = certify_positive(polynomial, parameters)?;
            let ray = auxiliary.ray(polynomial, parameters);
            let series = ray
                .series(auxiliary.ray, 0, u_degree as usize)
                .map_err(|error| invalid(format!("native positive-factor ray series: {error}")))?;
            let mut terms = Vec::new();
            for order in (2..=u_degree).step_by(2) {
                let coefficient = series
                    .coefficient(i64::from(order).into())
                    .ok_or_else(|| invalid("missing exact positive-factor ray coefficient"))?;
                let sign = if order % 4 == 0 { 1 } else { -1 };
                let ray_coefficient = auxiliary.bind(&(Atom::num(sign) * coefficient), &direction);
                let squared_bound = (&ray_coefficient / polynomial).pow(2);
                let positive_bound = SmoothPositivePart::new(
                    -&ray_coefficient * cap.pow(i64::from(order)) / polynomial,
                    &regularity,
                );
                polynomial_level += Atom::num((u_degree / 2) as i64)
                    * &squared_bound
                    * (&cap * &fraction).pow(i64::from(2 * order));
                terms.push(PositiveEnvelopeTerm {
                    order,
                    ray_coefficient,
                    squared_bound,
                    positive_bound,
                });
            }
            sign_aware_level += terms
                .iter()
                .map(|term| term.positive_bound.expression() * fraction.pow(i64::from(term.order)))
                .sum::<Atom>()
                .pow(2);
            positive_factors.push(PositiveEnvelope {
                polynomial: polynomial.clone(),
                lower_bound,
                terms,
            });
        }
        Ok(Self {
            parameters: parameters.to_vec(),
            causal_polynomial,
            weights,
            direction,
            leading_causal_magnitude,
            causal_terms,
            positive_factors,
            polynomial_level,
            sign_aware_level,
            regularity,
        })
    }

    pub fn parameters(&self) -> &[Symbol] {
        &self.parameters
    }
    pub fn causal_polynomial(&self) -> &Atom {
        &self.causal_polynomial
    }
    pub fn weights(&self) -> &[Atom] {
        &self.weights
    }
    pub fn direction(&self) -> &[Atom] {
        &self.direction
    }
    pub fn leading_causal_magnitude(&self) -> &Atom {
        &self.leading_causal_magnitude
    }
    pub fn causal_terms(&self) -> &[CausalEnvelopeTerm] {
        &self.causal_terms
    }
    pub fn positive_factors(&self) -> &[PositiveEnvelope] {
        &self.positive_factors
    }

    /// Structural even order shared by every restriction of this sector.
    /// A coefficient vanishing on a face never shortens the root schema.
    pub fn maximum_even_order(&self) -> u32 {
        self.causal_terms
            .iter()
            .map(|term| 2 * term.order - 2)
            .chain(
                self.positive_factors
                    .iter()
                    .flat_map(|factor| &factor.terms)
                    .map(|term| 2 * term.order),
            )
            .max()
            .unwrap_or(2)
            .max(2)
    }

    /// Dense [a2, a4, ...] in H(u), including structurally absent zero slots.
    pub fn polynomial_coefficients(&self) -> Result<Vec<Atom>, GenerationError> {
        self.polynomial_coefficients_with_inputs(
            self.direction.iter().map(|v| v.pow(2)).sum(),
            |term| term.squared_bound.clone(),
            |_, term| term.squared_bound.clone(),
        )
    }

    /// Build the same native arithmetic combiner with independently evaluated
    /// primitive inputs. Inputs must be radius-independent and represent the
    /// same bounds; this does not authorize changing their proof or counts.
    /// Certified checkers can supply native ball-enclosed primitive slots
    /// without rebuilding or convolving the coefficient polynomial themselves.
    pub fn polynomial_coefficients_with_inputs(
        &self,
        direction_norm_squared: Atom,
        mut causal_squared: impl FnMut(&CausalEnvelopeTerm) -> Atom,
        mut positive_squared: impl FnMut(&PositiveEnvelope, &PositiveEnvelopeTerm) -> Atom,
    ) -> Result<Vec<Atom>, GenerationError> {
        let radius = Atom::var(lambda_cap_symbol()) * Atom::var(radius_fraction_symbol());
        let mut level = base_level_from_norm(&direction_norm_squared);
        for term in &self.causal_terms {
            level += Atom::num(self.causal_terms.len())
                * causal_squared(term)
                * radius.pow(i64::from(2 * term.order - 2));
        }
        for factor in &self.positive_factors {
            for term in &factor.terms {
                level += Atom::num(factor.terms.len())
                    * positive_squared(factor, term)
                    * radius.pow(i64::from(2 * term.order));
            }
        }
        self.coefficients(&level)
    }

    /// The same schema for the sign-aware level. Positive-part coefficients
    /// retain their symbolic form for cancellation-resistant runtime lowering.
    pub fn sign_aware_coefficients(&self) -> Result<Vec<Atom>, GenerationError> {
        self.coefficients(&self.sign_aware_level)
    }

    /// Insert a native callback/alias for each smooth positive part before
    /// polynomial collection can normalize away its original expression shape.
    /// The caller must lower the same mathematical function and native jets;
    /// this hook does not authorize changing the envelope or its regularity.
    pub fn sign_aware_coefficients_with(
        &self,
        lower: impl FnMut(&SmoothPositivePart) -> Atom,
    ) -> Result<Vec<Atom>, GenerationError> {
        self.sign_aware_coefficients_with_inputs(
            self.direction.iter().map(|v| v.pow(2)).sum(),
            lower,
        )
    }

    /// Native sign-aware coefficient combiner with an independent direction
    /// norm and smooth-positive primitive inputs; see the polynomial contract.
    pub fn sign_aware_coefficients_with_inputs(
        &self,
        direction_norm_squared: Atom,
        mut lower: impl FnMut(&SmoothPositivePart) -> Atom,
    ) -> Result<Vec<Atom>, GenerationError> {
        let variable = Atom::var(radius_fraction_symbol());
        let causal = self
            .causal_terms
            .iter()
            .map(|term| lower(&term.positive_bound) * variable.pow(i64::from(term.order - 1)))
            .sum::<Atom>();
        let mut level = base_level_from_norm(&direction_norm_squared) + causal.pow(2);
        for factor in &self.positive_factors {
            let positive = factor
                .terms
                .iter()
                .map(|term| lower(&term.positive_bound) * variable.pow(i64::from(term.order)))
                .sum::<Atom>();
            level += positive.pow(2);
        }
        self.coefficients(&level)
    }

    fn coefficients(&self, level: &Atom) -> Result<Vec<Atom>, GenerationError> {
        let variable = Atom::var(radius_fraction_symbol());
        let polynomial = level.to_polynomial_in_vars_with_field::<u32>(
            [variable.clone()],
            &AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        );
        let maximum = self.maximum_even_order();
        let mut coefficients = vec![Atom::Zero; (maximum / 2) as usize];
        for term in &polynomial {
            let order = term.exponents[0];
            if order == 0 || !order.is_multiple_of(2) || order > maximum {
                return Err(invalid("dynamic level violates its fixed even root schema"));
            }
            if term.coefficient.contains(variable.as_view()) {
                return Err(invalid("dynamic coefficient retains root dependence"));
            }
            coefficients[(order / 2 - 1) as usize] = term.coefficient.clone();
        }
        Ok(coefficients)
    }
    pub fn polynomial_level(&self) -> &Atom {
        &self.polynomial_level
    }
    /// Exact structural powers that can occur in the reference equation.
    /// The counts in H remain fixed even when an entry vanishes on a face.
    /// [2] and [2,4] admit native closed-form root specializations later.
    pub fn polynomial_exponents(&self) -> Vec<u32> {
        std::iter::once(2)
            .chain(
                self.causal_terms
                    .iter()
                    .filter(|term| !term.squared_bound.is_zero())
                    .map(|term| 2 * term.order - 2),
            )
            .chain(
                self.positive_factors
                    .iter()
                    .flat_map(|factor| &factor.terms)
                    .filter(|term| !term.squared_bound.is_zero())
                    .map(|term| 2 * term.order),
            )
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn sign_aware_level(&self) -> &Atom {
        &self.sign_aware_level
    }
    pub fn regularity(&self) -> &Atom {
        &self.regularity
    }
    /// Substitute the selected positive root for the radius fraction only once
    /// when constructing the full dynamic density and its native derivatives.
    pub fn strength(&self) -> Atom {
        Atom::var(safety_fraction_symbol())
            * Atom::var(lambda_cap_symbol())
            * Atom::var(radius_fraction_symbol())
    }
}

fn causal_term(
    order: u32,
    hessian_ray: Vec<Atom>,
    weights: &[Atom],
    cap: &Atom,
    regularity: &Atom,
) -> CausalEnvelopeTerm {
    let n = weights.len();
    let factorial = Atom::num(Integer::factorial(order));
    let sign = if order % 4 == 3 { 1 } else { -1 };
    let scale = Atom::num(sign) * cap.pow(i64::from(order - 1)) / &factorial;
    let diagonal = weights
        .iter()
        .enumerate()
        .map(|(i, weight)| &scale * weight * &hessian_ray[i * n + i])
        .collect::<Vec<_>>();
    let mut weighted_norm = Atom::Zero;
    let mut gap = Atom::Zero;
    for i in 0..n {
        weighted_norm += weights[i].pow(2) * hessian_ray[i * n + i].pow(2);
        for j in i + 1..n {
            let off_diagonal_square = &weights[i] * &weights[j] * hessian_ray[i * n + j].pow(2);
            weighted_norm += Atom::num(2) * &off_diagonal_square;
            gap += (&diagonal[i] - &diagonal[j]).pow(2)
                + Atom::num(2 * n as i64) * scale.pow(2) * off_diagonal_square;
        }
    }
    let spectral_mean = diagonal.into_iter().sum::<Atom>() / Atom::num(n as i64);
    // (n-1)/n^2 times diagonal differences and 2*n weighted off-diagonal
    // squares. No subtraction of nearly equal traces and no sqrt(weights).
    let spectral_gap_squared = Atom::num((n as i64 - 1, (n * n) as i64)) * gap;
    let positive_bound = SmoothPositivePart::new(
        &spectral_mean + (&spectral_gap_squared + regularity.pow(2)).sqrt(),
        regularity,
    );
    CausalEnvelopeTerm {
        order,
        hessian_ray,
        squared_bound: weighted_norm / factorial.pow(2),
        spectral_mean,
        spectral_gap_squared,
        positive_bound,
    }
}

fn invalid(message: impl Into<String>) -> GenerationError {
    GenerationError::Contour(message.into())
}
fn base_level(direction: &[Atom]) -> Atom {
    base_level_from_norm(&direction.iter().map(|v| v.pow(2)).sum())
}
fn base_level_from_norm(direction_norm_squared: &Atom) -> Atom {
    Atom::var(radius_fraction_symbol()).pow(2)
        * (Atom::one()
            + Atom::var(lambda_cap_symbol()).pow(2) * direction_norm_squared
                / Atom::var(displacement_cap_symbol()).pow(2))
}
fn real(expression: &Atom) -> bool {
    crate::kernel::is_real_expression(
        expression,
        &expression
            .get_all_symbols(true)
            .into_iter()
            .collect::<Vec<_>>(),
    )
}
fn polynomial_degree(expression: &Atom, parameters: &[Symbol]) -> Result<u32, GenerationError> {
    // Reuse native homogeneous scaling degree, as in parametric/homogeneity,
    // rather than expanding the multivariate F solely to count derivatives.
    // The placeholder is already disjoint from all input symbols at admission.
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let indeterminates = expression
        .is_polynomial(true, false)
        .ok_or_else(|| invalid("dynamic factor is not polynomial in its real coordinates"))?;
    if !indeterminates.iter().all(|indeterminate| {
        variables
            .iter()
            .any(|variable| variable.as_view() == *indeterminate)
            || variables
                .iter()
                .all(|variable| !indeterminate.contains(variable.as_view()))
    }) {
        return Err(invalid(
            "dynamic factor hides coordinate dependence in a native polynomial indeterminate",
        ));
    }
    let scale = Atom::var(symbol!("fastsecdec::contour::dynamic::auxiliary_ray"));
    let scaled = expression.replace_multiple(parameters.iter().map(|p| {
        Replacement::new(
            Pattern::Literal(Atom::var(*p)),
            Pattern::Literal(Atom::var(*p) * &scale),
        )
    }));
    let polynomial = scaled.to_polynomial_in_vars_with_field::<u32>(
        [scale.clone()],
        &AtomField {
            statistical_zero_test: false,
            ..AtomField::new()
        },
    );
    if (&polynomial)
        .into_iter()
        .any(|term| term.coefficient.contains(scale.as_view()))
    {
        return Err(invalid(
            "dynamic factor is not polynomial in its real coordinates",
        ));
    }
    Ok(polynomial.degree(0))
}
fn certify_positive(expression: &Atom, parameters: &[Symbol]) -> Result<Atom, GenerationError> {
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let polynomial = expression.to_polynomial_in_vars_with_field::<u32>(
        &variables,
        &AtomField {
            statistical_zero_test: false,
            ..AtomField::new()
        },
    );
    for term in &polynomial {
        let coefficient = &term.coefficient;
        // Native sign predicates prove signs where an expression is defined;
        // is_finite() only rules out explicit infinities. Exact rational
        // coefficients give a sufficient definedness domain without inventing
        // a general constant-expression validator or trusting assumptions.
        if Rational::try_from(coefficient.as_view()).is_err()
            || !coefficient.is_nonnegative().is_true()
        {
            return Err(invalid(
                "unsupported dynamic U positivity proof: native coefficients must be exact rationals with proven nonnegative signs",
            ));
        }
    }
    let constant = polynomial.get_constant();
    if !constant.is_positive().is_true() {
        return Err(invalid(
            "dynamic U requires a strictly positive constant in its nonnegative coefficient certificate",
        ));
    }
    Ok(constant)
}

struct Auxiliary {
    ray: Symbol,
    directions: Vec<Symbol>,
}
impl Auxiliary {
    fn new(dimension: usize) -> Self {
        Self {
            ray: symbol!("fastsecdec::contour::dynamic::auxiliary_ray"),
            directions: (0..dimension)
                .map(|i| {
                    symbol!(&format!(
                        "fastsecdec::contour::dynamic::auxiliary_direction_{i}"
                    ))
                })
                .collect(),
        }
    }
    fn ray(&self, expression: &Atom, parameters: &[Symbol]) -> Atom {
        expression.replace_multiple(parameters.iter().zip(&self.directions).map(|(x, d)| {
            Replacement::new(
                Pattern::Literal(Atom::var(*x)),
                Pattern::Literal(Atom::var(*x) + Atom::var(self.ray) * Atom::var(*d)),
            )
        }))
    }
    fn bind(&self, expression: &Atom, direction: &[Atom]) -> Atom {
        expression.replace_multiple(self.directions.iter().zip(direction).map(|(d, v)| {
            Replacement::new(Pattern::Literal(Atom::var(*d)), Pattern::Literal(v.clone()))
        }))
    }
}
