//! Exact native real signs, with a cheap rational path and local result reuse.

use std::{cmp::Ordering, collections::HashMap};

use symbolica::{
    atom::Atom,
    domains::{RealEmbedding, algebraic::AlgebraicContext, rational::Rational},
};

use super::rational;

#[derive(Default)]
pub(super) struct CoefficientSigns {
    algebraic: HashMap<Atom, Option<Ordering>>,
}

impl CoefficientSigns {
    pub(super) fn get(&mut self, value: &Atom) -> Option<Ordering> {
        if let Some(value) = rational(value) {
            return Some(value.cmp(&Rational::from(0)));
        }
        *self.algebraic.entry(value.clone()).or_insert_with(|| {
            let mut context = AlgebraicContext::from_atom(value.as_view()).ok()?;
            let element = context.convert_atom(value.as_view()).ok()?;
            context.field().try_sign(&element).ok()
        })
    }

    pub(super) fn uniform<'a>(
        &mut self,
        coefficients: impl IntoIterator<Item = &'a Atom>,
    ) -> Option<Ordering> {
        let mut direction = None;
        for coefficient in coefficients {
            let sign = self.get(coefficient)?;
            if sign == Ordering::Equal || direction.is_some_and(|previous| previous != sign) {
                return None;
            }
            direction = Some(sign);
        }
        direction
    }
}
