use super::{FactorSeed, Result};
use crate::threshold::resolution::{
    Budget, Error, EtaleCertificate, EtaleFrame, Ideal, LocalizedAlgebra, Poly, RingExtension,
};
use std::sync::Arc;
use symbolica::{
    atom::Symbol,
    domains::{Field, Ring as _, algebraic::AlgebraicNumber, rational::Q},
    poly::{PolyVariable, polynomial::PolynomialRing, univariate::UnivariatePolynomial},
    tensors::matrix::Matrix,
};

/// Complete implicit factor equations and exact selected fiber. Finite jets are
/// derived views; the bulk factor is never replaced by a Taylor polynomial.
pub struct FactorBranch {
    seed: Arc<FactorSeed>,
    extension: RingExtension,
    frame: Arc<EtaleFrame>,
    left: Vec<Poly>,
    right: Vec<Poly>,
    constant_axis: Option<usize>,
    fiber: Vec<AlgebraicNumber<Q>>,
    product_minor: Poly,
}
pub struct FactorJet {
    owner: Arc<FactorBranch>,
    coefficient: Poly,
    directions: Vec<usize>,
    derivative: Poly,
    at_fiber: AlgebraicNumber<Q>,
}
impl FactorJet {
    pub fn owner(&self) -> &Arc<FactorBranch> {
        &self.owner
    }
    pub fn coefficient(&self) -> &Poly {
        &self.coefficient
    }
    pub fn directions(&self) -> &[usize] {
        &self.directions
    }
    pub fn derivative(&self) -> &Poly {
        &self.derivative
    }
    pub fn at_fiber(&self) -> &AlgebraicNumber<Q> {
        &self.at_fiber
    }
}
impl FactorBranch {
    /// Fresh roles are left nonleading coefficients, right coefficients,
    /// optional primitive constant, then the new determinant inverse.
    pub fn prepare(
        seed: Arc<FactorSeed>,
        fresh: &[Symbol],
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        let base = seed.polynomial.frame();
        let n = base.local().ring().len();
        let d = seed.polynomial.degree();
        let m = seed.left.len() - 1;
        let k = seed.right.len() - 1;
        let algebraic = !seed.context.is_trivial();
        let expected = d
            .checked_add(usize::from(algebraic))
            .and_then(|v| v.checked_add(1))
            .ok_or(Error::ResourceIncomplete("factor extension size"))?;
        if fresh.len() != expected || fresh.contains(&seed.polynomial.variable()) {
            return Err(Error::Invalid("factor fresh role count/namespace"));
        }
        // Product-map Jacobian entries are single coefficient variables or
        // constants. d! is a conservative determinant term bound; all such
        // products have degree at most d before native cancellation.
        let mut terms = 1usize;
        for i in 1..=d {
            terms = terms
                .checked_mul(i)
                .filter(|n| *n <= budget.limits.max_terms)
                .ok_or(Error::ResourceIncomplete("factor determinant term bound"))?;
        }
        if d > usize::from(budget.limits.max_degree_per_axis) || d + 1 > budget.limits.max_terms {
            return Err(Error::ResourceIncomplete("factor product degree/terms"));
        }
        let extension = RingExtension::with_symbols(base.local().ring().clone(), fresh, budget)?;
        let ring = extension.target();
        let left = (0..m)
            .map(|i| ring.coordinate(n + i))
            .chain(std::iter::once(Ok(ring.one())))
            .collect::<Result<Vec<_>>>()?;
        let right = (0..k)
            .map(|i| ring.coordinate(n + m + i))
            .chain(std::iter::once(Ok(ring.one())))
            .collect::<Result<Vec<_>>>()?;
        let variable = Arc::new(PolyVariable::from(seed.polynomial.variable()));
        let coefficient_ring = PolynomialRing::new(Q);
        let lp = UnivariatePolynomial::from_coefficients(
            &coefficient_ring,
            left.clone(),
            variable.clone(),
        );
        let rp =
            UnivariatePolynomial::from_coefficients(&coefficient_ring, right.clone(), variable);
        budget.reserve_slots(
            d.checked_mul(d)
                .ok_or(Error::ResourceIncomplete("factor product matrix size"))?,
        )?;
        budget.charge(
            m.checked_mul(k)
                .ok_or(Error::ResourceIncomplete("factor coefficient products"))?,
        )?;
        let product = &lp * &rp;
        let source = extension.ideal(base.local().ideal(), budget)?;
        let mut equations = source.generators().to_vec();
        let mut selected = base.selected_equations().to_vec();
        let mut dependent = base.dependent_axes().to_vec();
        let mut axes = base.local().axes().to_vec();
        let constant_axis = algebraic.then_some(n + d);
        let minimal_derivative = if let Some(axis) = constant_axis {
            let mut minimal = seed.context.field().poly().clone();
            minimal.rename_variable(
                &seed.context.field().poly().variables()[0],
                &PolyVariable::from(fresh[d]),
            );
            let minimal = ring.atom(&minimal.to_expression())?;
            budget.poly(&minimal)?;
            selected.push(equations.len());
            dependent.push(axis);
            axes.push(axis);
            equations.push(minimal.clone());
            minimal.derivative(axis)
        } else {
            ring.one()
        };
        let mut product_equations = Vec::with_capacity(d);
        for (i, p) in product.coefficients().iter().enumerate().take(d) {
            let equation = p - &extension.pull(&seed.polynomial.coefficients()[i], budget)?;
            budget.poly(&equation)?;
            selected.push(equations.len());
            dependent.push(n + i);
            axes.push(n + i);
            product_equations.push(equation.clone());
            equations.push(equation);
        }
        let dim =
            u32::try_from(d).map_err(|_| Error::ResourceIncomplete("factor matrix dimension"))?;
        let entries = product_equations
            .iter()
            .flat_map(|p| (0..d).map(move |j| p.derivative(n + j)))
            .collect();
        budget.charge(2)?;
        let minor = Matrix::from_linear(entries, dim, dim, coefficient_ring)
            .map_err(|_| Error::Invalid("native product matrix"))?
            .det()
            .map_err(|_| Error::Invalid("native product determinant"))?;
        let resultant = lp.resultant(&rp);
        budget.poly(&minor)?;
        budget.poly(&resultant)?;
        // Ascending outputs and per-factor ascending coefficients fix this sign.
        if minor != resultant {
            return Err(Error::Invalid("native product minor/resultant identity"));
        }
        let old_minor = extension.pull(base.determinant(), budget)?;
        let old_constant = budget.mul(&old_minor, &minimal_derivative)?;
        let expected_minor = budget.mul(&old_constant, &minor)?;
        let source = LocalizedAlgebra::new(
            Ideal::new(ring.clone(), equations, budget)?,
            axes,
            extension.guards(base.local().guards(), budget)?,
            budget,
        )?;
        let inverse = n + expected - 1;
        let frame = Arc::new(
            EtaleCertificate {
                source,
                equations: selected,
                dependent_axes: dependent,
                free_axes: base.free_axes().to_vec(),
                determinant_inverse_axis: Some(inverse),
            }
            .verify(budget)?,
        );
        if frame.determinant() != &expected_minor {
            return Err(Error::Invalid("extended factor block determinant"));
        }
        let field = seed.context.field();
        let mut fiber = seed.point.clone();
        fiber.extend(seed.left[..m].iter().cloned());
        fiber.extend(seed.right[..k].iter().cloned());
        if algebraic {
            fiber.push(field.generator());
        }
        fiber.push(field.zero());
        let determinant = frame.determinant().evaluate_with_coeff_map(
            |q| field.constant(q.clone()),
            &fiber,
            field,
        );
        if field.is_zero(&determinant) {
            return Err(Error::Invalid("factor selected fiber is singular"));
        }
        fiber[inverse] = field.div(&field.one(), &determinant);
        for relation in frame
            .local()
            .ideal()
            .generators()
            .iter()
            .chain(frame.local().unit_relations())
        {
            if !field.is_zero(&relation.evaluate_with_coeff_map(
                |q| field.constant(q.clone()),
                &fiber,
                field,
            )) {
                return Err(Error::Invalid("factor fiber fails extended equations"));
            }
        }
        Ok(Arc::new(Self {
            seed,
            extension,
            frame,
            left,
            right,
            constant_axis,
            fiber,
            product_minor: minor,
        }))
    }
    pub fn seed(&self) -> &Arc<FactorSeed> {
        &self.seed
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn factors(&self) -> (&[Poly], &[Poly]) {
        (&self.left, &self.right)
    }
    pub fn constant_axis(&self) -> Option<usize> {
        self.constant_axis
    }
    pub fn product_minor(&self) -> &Poly {
        &self.product_minor
    }
    pub fn jet(
        self: &Arc<Self>,
        right: bool,
        coefficient: usize,
        directions: &[usize],
        budget: &mut Budget,
    ) -> Result<FactorJet> {
        if directions.len() > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("factor jet order"));
        }
        budget.reserve_slots(directions.len())?;
        let p = if right { &self.right } else { &self.left }
            .get(coefficient)
            .ok_or(Error::Invalid("factor coefficient index"))?;
        let mut derivative = p.clone();
        for &direction in directions {
            derivative = self.frame.derivative(direction, &derivative, budget)?;
        }
        let field = self.seed.context.field();
        let at_fiber =
            derivative.evaluate_with_coeff_map(|q| field.constant(q.clone()), &self.fiber, field);
        Ok(FactorJet {
            owner: self.clone(),
            coefficient: p.clone(),
            directions: directions.to_vec(),
            derivative,
            at_fiber,
        })
    }
}
