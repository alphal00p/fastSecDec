use super::{MonicPolynomial, Result};
use crate::threshold::resolution::{Budget, Error, Poly};
use std::{collections::BTreeSet, sync::Arc};
use symbolica::{
    atom::{Atom, AtomView},
    coefficient::CoefficientView,
    domains::{
        RealEmbedding, Ring as _,
        algebraic::{AlgebraicContext, AlgebraicExtension, AlgebraicNumber, Root},
        rational::{Q, Rational},
    },
    poly::{PolyVariable, univariate::UnivariatePolynomial},
};

/// One exact selected real embedding for the entire point and both clusters.
/// Original expressions, native field images and the source owner are retained.
/// This is a germ at this fiber, not a parameter-region or rebind certificate.
#[derive(Clone)]
pub struct FactorSeed {
    pub(super) polynomial: Arc<MonicPolynomial>,
    pub(super) context: AlgebraicContext,
    pub(super) point: Vec<AlgebraicNumber<Q>>,
    pub(super) left: Vec<AlgebraicNumber<Q>>,
    pub(super) right: Vec<AlgebraicNumber<Q>>,
    original_point: Vec<Atom>,
    original_left: Vec<Atom>,
    original_right: Vec<Atom>,
    resultant: AlgebraicNumber<Q>,
}
fn exact_source(
    atom: AtomView<'_>,
    symbols: bool,
    depth: usize,
    nodes: &mut usize,
    algebraic_degree: &mut usize,
    generators: &mut BTreeSet<Atom>,
    budget: &mut Budget,
) -> Result<usize> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(Error::ResourceIncomplete("constant source nodes"))?;
    if *nodes > budget.limits.max_terms || depth > budget.limits.max_mark {
        return Err(Error::ResourceIncomplete("constant source nodes/depth"));
    }
    budget.charge(1)?;
    let degree = match atom {
        AtomView::Num(n) => match n.get_coeff_view() {
            CoefficientView::Natural(..) | CoefficientView::Large(..) => 0,
            _ => {
                return Err(Error::Invalid(
                    "factor constants require exact numeric coefficients",
                ));
            }
        },
        AtomView::Var(_) if symbols => 1,
        AtomView::Var(_) => return Err(Error::Invalid("factor fiber contains a free symbol")),
        AtomView::Add(a) => {
            let mut max = 0;
            for a in a {
                max = max.max(exact_source(
                    a,
                    symbols,
                    depth + 1,
                    nodes,
                    algebraic_degree,
                    generators,
                    budget,
                )?);
            }
            max
        }
        AtomView::Mul(a) => {
            let mut sum = 0usize;
            for a in a {
                sum = sum
                    .checked_add(exact_source(
                        a,
                        symbols,
                        depth + 1,
                        nodes,
                        algebraic_degree,
                        generators,
                        budget,
                    )?)
                    .ok_or(Error::ResourceIncomplete("constant polynomial degree"))?;
            }
            sum
        }
        AtomView::Pow(a) => {
            let (b, e) = a.get_base_exp();
            let d = exact_source(
                b,
                symbols,
                depth + 1,
                nodes,
                algebraic_degree,
                generators,
                budget,
            )?;
            let e = Rational::try_from(e).map_err(|_| Error::Invalid("constant rational power"))?;
            let numerator = e
                .numerator_ref()
                .to_i64()
                .ok_or(Error::ResourceIncomplete("constant exponent size"))?;
            let denominator = e
                .denominator_ref()
                .to_i64()
                .ok_or(Error::ResourceIncomplete("constant exponent size"))?;
            if numerator.unsigned_abs() > budget.limits.max_mark as u64
                || denominator as u64 > budget.limits.max_mark as u64
            {
                return Err(Error::ResourceIncomplete("constant exponent size"));
            }
            if d > 0 && (numerator < 0 || denominator != 1) {
                return Err(Error::Invalid("native root polynomial shape"));
            }
            if denominator > 1 && generators.insert(atom.to_owned()) {
                *algebraic_degree = algebraic_degree
                    .checked_mul(denominator as usize)
                    .filter(|d| *d <= budget.limits.max_mark)
                    .ok_or(Error::ResourceIncomplete("constant field degree bound"))?;
            }
            d.checked_mul(numerator.unsigned_abs() as usize)
                .ok_or(Error::ResourceIncomplete("constant polynomial degree"))?
        }
        AtomView::Fun(f) => {
            // Bounded native Root parser handles its bound variable/index. No
            // other function is admitted or interpreted by this adapter.
            for a in f {
                exact_source(
                    a,
                    true,
                    depth + 1,
                    nodes,
                    algebraic_degree,
                    generators,
                    budget,
                )?;
            }
            let root = Root::<Q>::try_from(atom)
                .map_err(|_| Error::Invalid("unclassified algebraic constant function"))?;
            budget.poly(root.polynomial())?;
            if root.polynomial().degree(0) as usize > budget.limits.max_mark {
                return Err(Error::ResourceIncomplete("constant root degree"));
            }
            if generators.insert(atom.to_owned()) {
                *algebraic_degree = algebraic_degree
                    .checked_mul(root.polynomial().degree(0) as usize)
                    .filter(|d| *d <= budget.limits.max_mark)
                    .ok_or(Error::ResourceIncomplete("constant field degree bound"))?;
            }
            0
        }
    };
    if degree > budget.limits.max_mark {
        return Err(Error::ResourceIncomplete("constant polynomial degree"));
    }
    Ok(degree)
}

impl FactorSeed {
    pub fn verify(
        polynomial: Arc<MonicPolynomial>,
        point: Vec<Atom>,
        left: Vec<Atom>,
        right: Vec<Atom>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        let total = point
            .len()
            .checked_add(left.len())
            .and_then(|v| v.checked_add(right.len()))
            .ok_or(Error::ResourceIncomplete("factor seed count"))?;
        budget.reserve_slots(total)?;
        if point.len() != polynomial.frame().local().ring().len()
            || left.len() < 2
            || right.len() < 2
            || left
                .len()
                .checked_add(right.len())
                .and_then(|v| v.checked_sub(2))
                != Some(polynomial.degree())
        {
            return Err(Error::Invalid("factor seed dimensions"));
        }
        let mut context = AlgebraicContext::new(AlgebraicExtension::trivial(Q));
        let mut nodes = 0;
        let mut algebraic_degree = 1usize;
        let mut generators = BTreeSet::new();
        for a in point.iter().chain(&left).chain(&right) {
            exact_source(
                a.as_view(),
                false,
                0,
                &mut nodes,
                &mut algebraic_degree,
                &mut generators,
                budget,
            )?;
        }
        // Complete the field FIRST: extending after retaining values could
        // otherwise leave older values in a different primitive-element basis.
        for a in point.iter().chain(&left).chain(&right) {
            context
                .extend(a.as_view())
                .map_err(|_| Error::Invalid("native algebraic constant discovery"))?;
            if context.field().poly().degree(0) as usize > budget.limits.max_mark {
                return Err(Error::ResourceIncomplete("native constant field degree"));
            }
        }
        let get = |atoms: &[Atom]| -> Result<Vec<AlgebraicNumber<Q>>> {
            atoms
                .iter()
                .map(|a| {
                    let value = context
                        .image(a)
                        .ok_or(Error::Invalid("native algebraic constant conversion"))?
                        .clone();
                    context
                        .field()
                        .try_sign(&value)
                        .map_err(|_| Error::Invalid("factor fiber is not real"))?;
                    Ok(value)
                })
                .collect()
        };
        let values = get(&point)?;
        let a = get(&left)?;
        let b = get(&right)?;
        let field = context.field();
        if !context.is_trivial() {
            field
                .try_sign(&field.generator())
                .map_err(|_| Error::Invalid("constant primitive element has no real embedding"))?;
        }
        if a.last() != Some(&field.one()) || b.last() != Some(&field.one()) {
            return Err(Error::Invalid("factor seed is not monic"));
        }
        let eval =
            |p: &Poly| p.evaluate_with_coeff_map(|q| field.constant(q.clone()), &values, field);
        for relation in polynomial
            .frame()
            .local()
            .ideal()
            .generators()
            .iter()
            .chain(polynomial.frame().local().unit_relations())
        {
            budget.charge(1)?;
            if !field.is_zero(&eval(relation)) {
                return Err(Error::Invalid(
                    "factor fiber violates base relation or guard",
                ));
            }
        }
        let variable = Arc::new(PolyVariable::from(polynomial.variable()));
        let ap = UnivariatePolynomial::from_coefficients(field, a.clone(), variable.clone());
        let bp = UnivariatePolynomial::from_coefficients(field, b.clone(), variable.clone());
        let source = UnivariatePolynomial::from_coefficients(
            field,
            polynomial.coefficients().iter().map(eval).collect(),
            variable,
        );
        budget.charge(2)?;
        if &ap * &bp != source {
            return Err(Error::Invalid(
                "factor seed product differs from source fiber",
            ));
        }
        let resultant = ap.resultant(&bp);
        if field.is_zero(&resultant) {
            return Err(Error::Invalid("factor seed clusters are not coprime"));
        }
        Ok(Arc::new(Self {
            polynomial,
            context,
            point: values,
            left: a,
            right: b,
            original_point: point,
            original_left: left,
            original_right: right,
            resultant,
        }))
    }
    pub fn polynomial(&self) -> &Arc<MonicPolynomial> {
        &self.polynomial
    }
    pub fn context(&self) -> &AlgebraicContext {
        &self.context
    }
    pub fn original_point(&self) -> &[Atom] {
        &self.original_point
    }
    pub fn original_factors(&self) -> (&[Atom], &[Atom]) {
        (&self.original_left, &self.original_right)
    }
    pub fn resultant(&self) -> &AlgebraicNumber<Q> {
        &self.resultant
    }
}
