use super::{ClearedPolynomial, InputPolynomial, Monicization, Result, polynomialization};
use crate::threshold::resolution::{
    Budget, Error, FactorProduction, FactorProgress, Ideal, MarkedIdeal, Poly, UnitClearing,
    VerifiedRelativeSnc, WholeCartierFactorization, factor_whole_cartier_equations,
};
use std::sync::Arc;
use symbolica::{
    domains::rational::Q,
    poly::{PolyVariable, polynomial::PolynomialRing, univariate::UnivariatePolynomial},
};
#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub stage: &'static str,
    pub clearings: Vec<UnitClearing>,
    pub cleared: Option<Arc<ClearedPolynomial>>,
    pub monicization: Option<Arc<Monicization>>,
    pub discriminant: Option<Poly>,
    pub factorization: Option<Arc<WholeCartierFactorization>>,
    pub pending_factors: Option<FactorProgress>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct QuasiOrdinaryPolynomial {
    prepared: Arc<Monicization>,
    ledger: Arc<VerifiedRelativeSnc>,
    discriminant: Poly,
    factorization: Arc<WholeCartierFactorization>,
    residual_unit: Ideal,
}
impl QuasiOrdinaryPolynomial {
    pub fn prepared(&self) -> &Arc<Monicization> {
        &self.prepared
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    pub fn discriminant(&self) -> &Poly {
        &self.discriminant
    }
    pub fn factorization(&self) -> &Arc<WholeCartierFactorization> {
        &self.factorization
    }
    pub fn residual_unit_test(&self) -> &Ideal {
        &self.residual_unit
    }
}
#[derive(Clone, Debug)]
pub enum Preparation {
    Admitted(Arc<QuasiOrdinaryPolynomial>),
    /// The actual leading zero/degree-drop locus is retained, not discarded.
    LeadingStratum {
        ledger: Arc<VerifiedRelativeSnc>,
        cleared: Arc<ClearedPolynomial>,
        test: Ideal,
    },
    /// Identically zero on this original localization, not a zero density.
    Repeated {
        ledger: Arc<VerifiedRelativeSnc>,
        prepared: Arc<Monicization>,
        discriminant: Poly,
    },
    ResidualNeedsRefinement {
        prepared: Arc<Monicization>,
        discriminant: Poly,
        factors: Arc<WholeCartierFactorization>,
        test: Ideal,
    },
    Incomplete {
        source: Arc<InputPolynomial>,
        ledger: Arc<VerifiedRelativeSnc>,
        reason: &'static str,
        progress: Box<Progress>,
    },
}
/// Requires actual shared frame ownership. A formatting/hash match is not
/// authority. Native algebra calls are synchronous; callers own hard limits.
pub fn prepare(
    source: Arc<InputPolynomial>,
    ledger: Arc<VerifiedRelativeSnc>,
    max_power: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<Preparation> {
    if !Arc::ptr_eq(&source.frame, ledger.frame()) {
        return Err(Error::Invalid("quasiordinary frame owner mismatch"));
    }
    let mut progress = Progress::default();
    let result = run(
        &source,
        &ledger,
        max_power,
        namespace,
        budget,
        &mut progress,
    );
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Err(Error::ResourceIncomplete(reason)) => Ok(Preparation::Incomplete {
            source,
            ledger,
            reason,
            progress: Box::new(progress),
        }),
        result => result,
    }
}
pub(crate) fn discriminant(
    p: &crate::threshold::algebraic_branch::MonicPolynomial,
    budget: &mut Budget,
) -> Result<Poly> {
    let n = p.degree();
    let n32 = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("resultant degree"))?;
    let sum = p.coefficients().iter().try_fold(0usize, |s, c| {
        s.checked_add(c.nterms())
            .ok_or(Error::ResourceIncomplete("resultant coefficient term sum"))
    })?;
    let derivative_sum = p.coefficients().iter().skip(1).try_fold(0usize, |s, c| {
        s.checked_add(c.nterms())
            .ok_or(Error::ResourceIncomplete("resultant derivative term sum"))
    })?;
    sum.checked_pow(n32 - 1)
        .and_then(|x| {
            derivative_sum
                .checked_pow(n32)
                .and_then(|y| x.checked_mul(y))
        })
        .filter(|x| *x <= budget.limits.max_terms)
        .ok_or(Error::ResourceIncomplete("resultant Sylvester term bound"))?;
    let dimension = n
        .checked_mul(2)
        .and_then(|x| x.checked_sub(1))
        .ok_or(Error::ResourceIncomplete("resultant dimension"))?;
    for i in 0..p.frame().local().ring().len() {
        let degree = p
            .coefficients()
            .iter()
            .map(|c| usize::from(c.degree(i)))
            .max()
            .unwrap_or(0);
        if degree
            .checked_mul(dimension)
            .is_none_or(|d| d > usize::from(budget.limits.max_degree_per_axis))
        {
            return Err(Error::ResourceIncomplete(
                "resultant coefficient degree bound",
            ));
        }
    }
    budget.reserve_slots(p.coefficients().len())?;
    // Wider native exponent storage protects native intermediates; final
    // narrowing is checked. This is not an allocator bound for the owner CAS.
    let native = UnivariatePolynomial::from_coefficients(
        &PolynomialRing::new(Q),
        p.coefficients()
            .iter()
            .map(|c| c.map_exp(|e| u32::from(*e)))
            .collect(),
        Arc::new(PolyVariable::from(p.variable())),
    );
    budget.charge(2)?;
    let resultant = native.resultant(&native.derivative());
    let parity = n
        .checked_mul(n - 1)
        .ok_or(Error::ResourceIncomplete("discriminant sign exponent"))?
        / 2;
    let wide = if parity % 2 == 1 {
        -resultant
    } else {
        resultant
    };
    if (0..wide.nvars()).any(|i| wide.degree(i) > u32::from(budget.limits.max_degree_per_axis)) {
        return Err(Error::ResourceIncomplete("discriminant native degree"));
    }
    if wide.nterms() > budget.limits.max_terms {
        return Err(Error::ResourceIncomplete("discriminant native terms"));
    }
    let result = wide.map_exp(|e| u16::try_from(*e).expect("checked discriminant narrowing"));
    budget.poly(&result)?;
    Ok(result)
}
fn run(
    source: &Arc<InputPolynomial>,
    ledger: &Arc<VerifiedRelativeSnc>,
    max_power: usize,
    namespace: &str,
    budget: &mut Budget,
    progress: &mut Progress,
) -> Result<Preparation> {
    progress.stage = "native guard clearing";
    let cleared = polynomialization::clear(source, progress, budget)?;
    progress.cleared = Some(cleared.clone());
    progress.stage = "original leading unit";
    let (leading_test, valid) = polynomialization::unit_test(source, cleared.leading(), budget)?;
    if !valid {
        return Ok(Preparation::LeadingStratum {
            ledger: ledger.clone(),
            cleared,
            test: leading_test,
        });
    }
    let prepared = polynomialization::monic(cleared, leading_test, budget)?;
    progress.monicization = Some(prepared.clone());
    progress.stage = "native discriminant";
    let disc = discriminant(prepared.monic(), budget)?;
    progress.discriminant = Some(disc.clone());
    if source.frame.local().zero(&disc, budget)? {
        return Ok(Preparation::Repeated {
            ledger: ledger.clone(),
            prepared,
            discriminant: disc,
        });
    }
    progress.stage = "actual SNC divisor extraction";
    let ideal = Ideal::new(
        source.frame.local().ring().clone(),
        vec![disc.clone()],
        budget,
    )?;
    let marked = Arc::new(MarkedIdeal::new(ideal, 1, budget)?);
    let factors =
        match factor_whole_cartier_equations(ledger.clone(), marked, max_power, namespace, budget)?
        {
            FactorProduction::WholeEquations(f) => Arc::new(*f),
            FactorProduction::Incomplete {
                reason,
                progress: p,
                ..
            } => {
                progress.pending_factors = Some(p);
                return Err(Error::ResourceIncomplete(reason));
            }
            FactorProduction::ZeroIdeal { .. } => {
                return Err(Error::Invalid("nonzero discriminant became zero ideal"));
            }
        };
    progress.factorization = Some(factors.clone());
    if factors.residual().generators().len() != 1 {
        return Err(Error::Invalid("principal discriminant residual shape"));
    }
    progress.stage = "original residual unit";
    let (test, valid) =
        polynomialization::unit_test(source, &factors.residual().generators()[0], budget)?;
    if !valid {
        return Ok(Preparation::ResidualNeedsRefinement {
            prepared,
            discriminant: disc,
            factors,
            test,
        });
    }
    progress.stage = "complete prerequisite";
    Ok(Preparation::Admitted(Arc::new(QuasiOrdinaryPolynomial {
        prepared,
        ledger: ledger.clone(),
        discriminant: disc,
        factorization: factors,
        residual_unit: test,
    })))
}
