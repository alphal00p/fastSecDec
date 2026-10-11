use super::*;
use std::collections::BTreeSet;

/// One exact selected real point, with all entries in the same native field.
/// Existence of a local germ is separate from any interval/parameter box.
#[derive(Clone)]
pub struct GermPoint {
    frame: Arc<EtaleFrame>,
    context: AlgebraicContext,
    original: Vec<Atom>,
    values: Vec<AlgebraicNumber<Q>>,
}
impl GermPoint {
    pub fn verify(
        frame: Arc<EtaleFrame>,
        original: Vec<Atom>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if original.len() != frame.local().ring().len() {
            return Err(Error::Invalid("germ point dimensions"));
        }
        budget.reserve_slots(original.len())?;
        let mut nodes = 0;
        let mut degree = 1;
        let mut generators = BTreeSet::new();
        for a in &original {
            super::super::constants::exact_source(
                a.as_view(),
                false,
                0,
                &mut nodes,
                &mut degree,
                &mut generators,
                budget,
            )?;
        }
        let mut context = AlgebraicContext::new(AlgebraicExtension::trivial(Q));
        for a in &original {
            context
                .extend(a.as_view())
                .map_err(|_| Error::Invalid("native germ point field"))?;
            if context.field().poly().degree(0) as usize > budget.limits.max_mark {
                return Err(Error::ResourceIncomplete("germ point field degree"));
            }
        }
        let values = original
            .iter()
            .map(|a| {
                context
                    .image(a)
                    .cloned()
                    .ok_or(Error::Invalid("germ point native image"))
            })
            .collect::<Result<Vec<_>>>()?;
        Self::from_context(frame, context, original, values, budget)
    }
    pub(super) fn from_context(
        frame: Arc<EtaleFrame>,
        context: AlgebraicContext,
        original: Vec<Atom>,
        values: Vec<AlgebraicNumber<Q>>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if original.len() != values.len() || values.len() != frame.local().ring().len() {
            return Err(Error::Invalid("germ point field dimensions"));
        }
        let field = context.field();
        for v in &values {
            field
                .try_sign(v)
                .map_err(|_| Error::Invalid("nonreal selected germ point"))?;
        }
        if !context.is_trivial() {
            field
                .try_sign(&field.generator())
                .map_err(|_| Error::Invalid("germ primitive element not real"))?;
        }
        for relation in frame
            .local()
            .ideal()
            .generators()
            .iter()
            .chain(frame.local().unit_relations())
        {
            budget.charge(1)?;
            if !field.is_zero(&relation.evaluate_with_coeff_map(
                |q| field.constant(q.clone()),
                &values,
                field,
            )) {
                return Err(Error::Invalid("germ point violates relation or guard"));
            }
        }
        Ok(Arc::new(Self {
            frame,
            context,
            original,
            values,
        }))
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn context(&self) -> &AlgebraicContext {
        &self.context
    }
    pub fn original(&self) -> &[Atom] {
        &self.original
    }
    pub fn values(&self) -> &[AlgebraicNumber<Q>] {
        &self.values
    }
    pub fn evaluate(&self, p: &Poly) -> Result<AlgebraicNumber<Q>> {
        self.frame.local().supports(p)?;
        let field = self.context.field();
        Ok(p.evaluate_with_coeff_map(|q| field.constant(q.clone()), &self.values, field))
    }
    pub fn is_zero(&self, p: &Poly) -> Result<bool> {
        Ok(self.context.field().is_zero(&self.evaluate(p)?))
    }
    pub(super) fn after_factor(branch: &Arc<FactorBranch>, b: &mut Budget) -> Result<Arc<Self>> {
        let context = branch.seed().context().clone();
        let original = branch
            .fiber()
            .iter()
            .map(|v| context.field().element_to_atom(v))
            .collect();
        Self::from_context(
            branch.frame().clone(),
            context,
            original,
            branch.fiber().to_vec(),
            b,
        )
    }
    /// Preserve the existing complete field through a native extension. Added
    /// inverse values are computed from the actual target guard, never assumed.
    pub(super) fn extend(
        &self,
        frame: Arc<EtaleFrame>,
        mut values: Vec<AlgebraicNumber<Q>>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        let n = frame.local().ring().len();
        let field = self.context.field();
        if values.len() > n {
            return Err(Error::Invalid("germ extension dimensions"));
        }
        values.resize(n, field.zero());
        for g in frame.local().guards() {
            if g.inverse_axis >= self.values.len() {
                let h =
                    g.factor
                        .evaluate_with_coeff_map(|q| field.constant(q.clone()), &values, field);
                if field.is_zero(&h) {
                    return Err(Error::Invalid("germ extension guard vanishes at point"));
                }
                values[g.inverse_axis] = field.div(&field.one(), &h);
            }
        }
        let original = values.iter().map(|v| field.element_to_atom(v)).collect();
        Self::from_context(frame, self.context.clone(), original, values, budget)
    }
}
