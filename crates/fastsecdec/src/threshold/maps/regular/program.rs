//! Native composed map instructions; exact section proof and numerical execution
//! are separate owners. This does not authorize endpoint subtraction.
use super::{
    BracketProposal, Budget, Error, RegularSection, Result, SectionSide,
    callback::{Number, RootProgram, Scope, attempt},
    native,
};
use crate::threshold::{gcad::AliasRole, maps::CellMap};
use std::{collections::BTreeMap, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{
        EvaluationDomain, EvaluatorComposer, ExpressionEvaluator, OptimizationSettings, Slot,
    },
    symbol,
};

pub struct CompiledMap {
    source: Arc<CellMap>,
    sections: Vec<Arc<RegularSection>>,
    scope: Scope,
    exact: ExpressionEvaluator<Complex<Rational>>,
    inputs: Vec<Symbol>,
    dimensions: usize,
}
impl CompiledMap {
    pub fn prepare(
        source: Arc<CellMap>,
        proposals: &[[BracketProposal; 2]],
        settings: OptimizationSettings,
        b: &mut Budget,
    ) -> Result<Self> {
        if source
            .axes()
            .iter()
            .any(|a| a.role() != AliasRole::IntegrationCoordinate)
        {
            return Err(Error::Unsupported("parameter chamber mapping awaits typed parameter admission; no silent specialization".into()));
        }
        if proposals.len() != source.axes().len() {
            return Err(Error::Invalid(
                "one lower/upper proposal per integration axis required".into(),
            ));
        }
        let inputs = source.unit_coordinates().to_vec();
        let mut composer = EvaluatorComposer::new(inputs.len());
        let mut prefix_atoms = Vec::new();
        let mut prefix_slots = Vec::new();
        let mut coordinate_slots = BTreeMap::new();
        let mut widths = Vec::new();
        let mut sections = Vec::new();
        let mut scope = Scope::default();
        let intermediate = settings.clone().cpe_iterations(Some(0));
        let build = |outputs: &[Atom], inputs: &[Atom]| {
            Atom::evaluator_multiple(outputs, inputs)
                .optimization_settings(intermediate.clone())
                .build()
                .map_err(native)
        };
        for (axis, pair) in proposals.iter().enumerate() {
            let mut roots = Vec::new();
            for (which, side) in [SectionSide::Lower, SectionSide::Upper]
                .into_iter()
                .enumerate()
            {
                let section =
                    RegularSection::prepare(source.clone(), axis, side, pair[which].clone(), b)?;
                let call = if let [constant, slope] = section.coefficients() {
                    // The same section certificate proves the slope nonzero.
                    // Keep linear bounds in native arithmetic rather than paying
                    // for a callback and its scratch lock at every sample.
                    -constant.clone() / slope
                } else {
                    let program = RootProgram::prepare(&section, intermediate.clone())?;
                    let tag = symbol!(&format!(
                        "fastsecdec::threshold::regular_section::owner_slot_{}",
                        sections.len()
                    ));
                    let call = program.call(tag, section.coefficients())?;
                    scope.insert(tag, program)?;
                    call
                };
                // Native coefficient arithmetic in this stage sees prior coordinate slots;
                // no previous root expression is recursively substituted into it.
                let body = build(&[call], &prefix_atoms)?;
                roots.push(composer.append(&body, &prefix_slots).map_err(native)?[0]);
                sections.push(section);
            }
            let (lo, hi, t) = symbol!(
                "fastsecdec::threshold::regular_section::lower_slot",
                "fastsecdec::threshold::regular_section::upper_slot",
                "fastsecdec::threshold::regular_section::unit_slot"
            );
            let width = Atom::var(hi) - Atom::var(lo);
            let body = build(
                &[Atom::var(lo) + &width * Atom::var(t), width],
                &[Atom::var(lo), Atom::var(hi), Atom::var(t)],
            )?;
            let unit_index = source.axes()[axis]
                .unit_index()
                .ok_or_else(|| Error::Invalid("integration unit role".into()))?;
            let slots = composer
                .append(&body, &[roots[0], roots[1], Slot::Param(unit_index)])
                .map_err(native)?;
            coordinate_slots.insert(source.axes()[axis].symbol(), slots[0]);
            prefix_atoms.push(Atom::var(source.axes()[axis].symbol()));
            prefix_slots.push(slots[0]);
            widths.push(slots[1]);
        }
        let request = source.decomposition().request();
        let prepared = request
            .domain()
            .coordinates()
            .iter()
            .map(|s| coordinate_slots[s])
            .collect::<Vec<_>>();
        let original = if let Some(p) = request.projective_preparation() {
            let ins = p
                .coordinates()
                .iter()
                .copied()
                .map(Atom::var)
                .collect::<Vec<_>>();
            composer
                .append(&build(p.images(), &ins)?, &prepared)
                .map_err(native)?
        } else {
            prepared.clone()
        };
        let factors = inputs.iter().copied().map(Atom::var).collect::<Vec<_>>();
        let product: Atom = factors.iter().cloned().product();
        let measure = composer
            .append(&build(&[product], &factors)?, &widths)
            .map_err(native)?[0];
        let dimensions = prepared.len() + original.len() + 1;
        let mut outputs = prepared;
        outputs.extend(original);
        outputs.push(measure);
        // Preserve each width for checked execution; their product alone can
        // hide two reversed intervals. These are internal diagnostic outputs.
        outputs.extend(widths);
        let exact = composer.finish(&outputs, settings).map_err(native)?;
        b.check()?;
        Ok(Self {
            source,
            sections,
            scope,
            exact,
            inputs,
            dimensions,
        })
    }
    pub fn source(&self) -> &Arc<CellMap> {
        &self.source
    }
    pub fn sections(&self) -> &[Arc<RegularSection>] {
        &self.sections
    }
    pub fn exact(&self) -> &ExpressionEvaluator<Complex<Rational>> {
        &self.exact
    }
    pub fn inputs(&self) -> &[Symbol] {
        &self.inputs
    }
    pub fn map<T: Number>(&self, bits: u32) -> NumericMap<T> {
        let one = T::one_at(bits);
        let program = self.scope.enter(bits, || {
            self.exact
                .clone()
                .map_coeff_with_prec(&|c| one.from_rational(&c.re), bits)
        });
        NumericMap {
            program,
            owners: self.sections.clone(),
            dimension: self.dimensions,
            inputs: self.inputs.len(),
        }
    }
    #[cfg(test)]
    pub fn native_roundtrip(&self) -> Result<Self> {
        let bytes = bincode::serde::encode_to_vec(&self.exact, bincode::config::standard())
            .map_err(native)?;
        let (exact, read) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
            .map_err(native)?;
        if read != bytes.len() {
            return Err(Error::Invalid("native codec trailing bytes".into()));
        }
        // A test of saved instruction restoration, not an untrusted proof importer.
        Ok(Self {
            source: self.source.clone(),
            sections: self.sections.clone(),
            scope: self.scope.native_roundtrip()?,
            exact,
            inputs: self.inputs.clone(),
            dimensions: self.dimensions,
        })
    }
    /// Unchecked native complex evaluator for composition. Its caller must
    /// retain this owner and use callback::attempt to collect native failures.
    /// Inputs are real unit coordinates; nonreal coefficients are refused.
    pub fn map_complex<T: Number>(&self, bits: u32) -> ExpressionEvaluator<Complex<T>>
    where
        Complex<T>: EvaluationDomain,
    {
        let one = T::one_at(bits);
        self.scope.enter(bits, || {
            self.exact.clone().map_coeff_with_prec(
                &|c| Complex::new(one.from_rational(&c.re), one.from_rational(&c.im)),
                bits,
            )
        })
    }
}
#[derive(Clone)]
pub struct NumericMap<T> {
    program: ExpressionEvaluator<T>,
    owners: Vec<Arc<RegularSection>>,
    dimension: usize,
    inputs: usize,
}
impl<T: Number> NumericMap<T> {
    pub fn sections(&self) -> &[Arc<RegularSection>] {
        &self.owners
    }
    pub fn evaluate(&mut self, unit: &[T]) -> Result<Vec<T>> {
        if unit.len() != self.inputs
            || unit
                .iter()
                .any(|x| !x.is_finite() || *x < x.zero() || *x > x.one())
        {
            return Err(Error::Invalid(
                "map input must lie in its closed unit cube".into(),
            ));
        }
        let one = unit.first().cloned().unwrap_or_else(|| T::one_at(53));
        let mut out = vec![one.zero(); self.dimension + self.inputs];
        attempt(|| self.program.try_evaluate(unit, &mut out))
            .map_err(Error::Native)?
            .map_err(native)?;
        let interior = unit.iter().all(|x| *x > x.zero() && *x < x.one());
        if out.iter().any(|x| !x.is_finite())
            || out[self.dimension..].iter().any(|x| {
                if interior {
                    *x <= x.zero()
                } else {
                    *x < x.zero()
                }
            })
        {
            return Err(Error::Native(
                "map nonfinite output or negative triangular measure".into(),
            ));
        }
        out.truncate(self.dimension);
        Ok(out)
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;
    #[test]
    fn separate_width_checks_prevent_sign_masking_and_allow_closed_collapse() {
        let (s, t) = symbol!("regular_width_guard::s", "regular_width_guard::t");
        let make = |values: &[i64]| NumericMap {
            program: Atom::evaluator_multiple(
                &values.iter().map(|n| Atom::num(*n)).collect::<Vec<_>>(),
                &[Atom::var(s), Atom::var(t)],
            )
            .build()
            .unwrap()
            .map_coeff(&|c| c.re.to_f64()),
            owners: vec![],
            dimension: 1,
            inputs: 2,
        };
        // These deliberately malformed numeric outputs are not certificate owners.
        assert!(make(&[1, -1, -1]).evaluate(&[0.5, 0.5]).is_err());
        assert!(make(&[0, 0, 1]).evaluate(&[0.5, 0.5]).is_err());
        assert_eq!(make(&[0, 0, 1]).evaluate(&[0., 0.5]).unwrap(), vec![0.]);
        assert!(make(&[1, -1, -1]).evaluate(&[0., 0.5]).is_err());
    }
}
