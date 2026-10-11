use super::*;
use symbolica::atom::Symbol;

/// A native implicit simple-root presentation. The entire source polynomial and
/// exact complementary factor remain owned; no Taylor polynomial is substituted.
pub struct SimpleRootBranch {
    seed: Arc<FactorSeed>,
    extension: RingExtension,
    point: Arc<GermPoint>,
    left: Vec<Poly>,
    right: Vec<Poly>,
    equation: Poly,
    row_scale: Poly,
    derivative: Poly,
    clearings: Vec<UnitClearing>,
}
impl SimpleRootBranch {
    pub fn seed(&self) -> &Arc<FactorSeed> {
        &self.seed
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn point(&self) -> &Arc<GermPoint> {
        &self.point
    }
    pub fn factors(&self) -> (&[Poly], &[Poly]) {
        (&self.left, &self.right)
    }
    pub fn equation(&self) -> &Poly {
        &self.equation
    }
    pub fn row_scale(&self) -> &Poly {
        &self.row_scale
    }
    pub fn derivative(&self) -> &Poly {
        &self.derivative
    }
    pub fn clearings(&self) -> &[UnitClearing] {
        &self.clearings
    }
    pub(super) fn prepare(
        seed: Arc<FactorSeed>,
        fresh: &[Symbol],
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if seed.left.len() != 2 || fresh.len() != 2 || fresh.contains(&seed.polynomial().variable())
        {
            return Err(Error::Invalid("simple-root factor roles"));
        }
        let base = seed.polynomial().frame();
        let n = base.local().ring().len();
        let extension = RingExtension::with_symbols(base.local().ring().clone(), fresh, b)?;
        let ring = extension.target();
        let a = ring.coordinate(n)?;
        let left = vec![a.clone(), ring.one()];
        let variable = Arc::new(PolyVariable::from(seed.polynomial().variable()));
        let coefficient_ring = PolynomialRing::new(Q);
        let coefficients = seed
            .polynomial()
            .coefficients()
            .iter()
            .map(|c| extension.pull(c, b))
            .collect::<Result<Vec<_>>>()?;
        let terms = coefficients
            .iter()
            .try_fold(0usize, |sum, p| sum.checked_add(p.nterms()))
            .ok_or(Error::ResourceIncomplete("simple-root quotient term bound"))?;
        if terms > b.limits.max_terms {
            return Err(Error::ResourceIncomplete("simple-root quotient term bound"));
        }
        let source = UnivariatePolynomial::from_coefficients(
            &coefficient_ring,
            coefficients,
            variable.clone(),
        );
        let linear =
            UnivariatePolynomial::from_coefficients(&coefficient_ring, left.clone(), variable);
        b.charge(2)?;
        let (quotient, remainder) = source.quot_rem(&linear);
        if &(&linear * &quotient) + &remainder != source || remainder.degree() > 0 {
            return Err(Error::Invalid("native simple-root monic division"));
        }
        for c in quotient
            .coefficients()
            .iter()
            .chain(remainder.coefficients())
        {
            b.poly(c)?;
        }
        let right = quotient.coefficients().to_vec();
        let derivative = linear.resultant(&quotient);
        b.poly(&derivative)?;
        let clearings = seed
            .polynomial()
            .coefficients()
            .iter()
            .map(|c| clear_units(base.local(), c, b))
            .collect::<Result<Vec<_>>>()?;
        let mut denominator = base.local().ring().one();
        for c in &clearings {
            denominator = b.mul(&denominator, &c.denominator)?;
        }
        let row_scale = extension.pull(&denominator, b)?;
        let mut equation = ring.one().zero();
        for c in clearings.iter().rev() {
            equation = b.mul(&equation, &(-a.clone()))?;
            let (unit, rest) = denominator.quot_rem(&c.denominator, false);
            if !rest.is_zero() || b.mul(&unit, &c.denominator)? != denominator {
                return Err(Error::Invalid("simple-root denominator identity"));
            }
            let coefficient = b.mul(&unit, &c.numerator)?;
            equation = equation + extension.pull(&coefficient, b)?;
            b.poly(&equation)?;
        }
        equation = -equation;
        // d[-D*P(-a)]/da = D*P'(-a) = D*Res(z+a,Q).
        let expected = b.mul(&row_scale, &derivative)?;
        let old_ideal = extension.ideal(base.local().ideal(), b)?;
        let old_units = base
            .local()
            .unit_relations()
            .iter()
            .map(|p| extension.pull(p, b))
            .collect::<Result<Vec<_>>>()?;
        // Check BEFORE adjoining P: guarded source coefficients and cleared
        // coefficients agree only in the original extended base algebra.
        if !old_ideal.contains(&(&equation.derivative(n) - &expected), &old_units, b)? {
            return Err(Error::Invalid("simple-root derivative/resultant identity"));
        }
        let mut equations = extension
            .ideal(base.local().ideal(), b)?
            .generators()
            .to_vec();
        let mut selected = base.selected_equations().to_vec();
        selected.push(equations.len());
        equations.push(equation.clone());
        let mut axes = base.local().axes().to_vec();
        axes.push(n);
        let mut dependent = base.dependent_axes().to_vec();
        dependent.push(n);
        let local = LocalizedAlgebra::new(
            Ideal::new(ring.clone(), equations, b)?,
            axes,
            extension.guards(base.local().guards(), b)?,
            b,
        )?;
        let frame = Arc::new(
            EtaleCertificate {
                source: local,
                equations: selected,
                dependent_axes: dependent,
                free_axes: base.free_axes().to_vec(),
                determinant_inverse_axis: Some(n + 1),
            }
            .verify(b)?,
        );
        let old = extension.pull(base.determinant(), b)?;
        if !old_ideal.contains(
            &(frame.determinant() - &b.mul(&old, &expected)?),
            &old_units,
            b,
        )? {
            return Err(Error::Invalid("simple-root full minor identity"));
        }
        for c in (&(&linear * &quotient) - &source).coefficients() {
            if !frame.local().zero(c, b)? {
                return Err(Error::Invalid("simple-root full factor identity"));
            }
        }
        let field = seed.context().field();
        let mut values = seed.point.clone();
        values.push(seed.left[0].clone());
        values.push(field.zero());
        let det = frame.determinant().evaluate_with_coeff_map(
            |q| field.constant(q.clone()),
            &values,
            field,
        );
        if field.is_zero(&det) {
            return Err(Error::Invalid("simple-root selected minor vanishes"));
        }
        values[n + 1] = field.div(&field.one(), &det);
        let original = values.iter().map(|v| field.element_to_atom(v)).collect();
        let point = GermPoint::from_context(frame, seed.context().clone(), original, values, b)?;
        Ok(Arc::new(Self {
            seed,
            extension,
            point,
            left,
            right,
            equation,
            row_scale,
            derivative,
            clearings,
        }))
    }
}
/// Distinct exact owners; a simple-root equation is never advertised as a
/// coefficient-product FactorBranch.
pub enum Factorization {
    Coprime(Arc<FactorBranch>),
    Simple(Arc<SimpleRootBranch>),
}
impl Factorization {
    pub fn extension(&self) -> &RingExtension {
        match self {
            Self::Coprime(f) => f.extension(),
            Self::Simple(f) => f.extension(),
        }
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        match self {
            Self::Coprime(f) => f.frame(),
            Self::Simple(f) => f.point().frame(),
        }
    }
    pub fn factors(&self) -> (&[Poly], &[Poly]) {
        match self {
            Self::Coprime(f) => f.factors(),
            Self::Simple(f) => f.factors(),
        }
    }
    pub fn seed(&self) -> &Arc<FactorSeed> {
        match self {
            Self::Coprime(f) => f.seed(),
            Self::Simple(f) => f.seed(),
        }
    }
}
