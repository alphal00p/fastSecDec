//! Conservative exact-fold admission for native aliased Laurent coefficients.
use symbolica::atom::{AliasedAtom, AtomCore, AtomView, Symbol};

/// This is a sufficient test only. A root without alias references ignores its
/// unused alias table. Otherwise nested aliases keep a vector stochastic.
/// Native evaluator construction owns alias graph validation; never recursively
/// materialize an unchecked cycle.
pub(crate) fn coordinate_independent(value: &AliasedAtom, coordinates: &[Symbol]) -> bool {
    if coordinates
        .iter()
        .any(|s| value.get_root().contains_symbol(*s))
    {
        return false;
    }
    let mut handles = Vec::with_capacity(value.get_aliases().len());
    for handle in value.get_aliases().keys() {
        let AtomView::Var(handle) = handle.as_view() else {
            return false;
        };
        handles.push(handle.get_symbol());
    }
    if handles
        .iter()
        .all(|symbol| !value.get_root().contains_symbol(*symbol))
    {
        return true;
    }
    value.get_aliases().values().all(|body| {
        coordinates
            .iter()
            .chain(&handles)
            .all(|symbol| !body.contains_symbol(*symbol))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{atom::Atom, symbol};
    #[test]
    fn exact_fold_is_conservative_for_nested_and_unused_aliases() {
        let (a, b, t) = symbol!(
            "exact_classifier::a",
            "exact_classifier::b",
            "exact_classifier::t"
        );
        let mut nested = AliasedAtom::from(Atom::var(a));
        nested.register_alias(Atom::var(a), Atom::var(b));
        nested.register_alias(Atom::var(b), Atom::var(t));
        assert!(!coordinate_independent(&nested, &[t]));
        let mut flat = AliasedAtom::from(Atom::var(a));
        flat.register_alias(Atom::var(a), Atom::num(3));
        assert!(coordinate_independent(&flat, &[t]));
        assert_eq!(flat.into_inner(), Atom::num(3));
        let mut unused = AliasedAtom::from(Atom::one());
        unused.register_alias(Atom::var(a), Atom::var(t));
        assert!(coordinate_independent(&unused, &[t]));
        assert_eq!(unused.into_inner(), Atom::one());
        let mut nested_constant = AliasedAtom::from(Atom::var(a));
        nested_constant.register_alias(Atom::var(a), Atom::var(b));
        nested_constant.register_alias(Atom::var(b), Atom::num(3));
        assert!(!coordinate_independent(&nested_constant, &[t]));
        let mut cycle = AliasedAtom::from(Atom::var(a));
        cycle.register_alias(Atom::var(a), Atom::var(b));
        cycle.register_alias(Atom::var(b), Atom::var(a));
        assert!(!coordinate_independent(&cycle, &[t]));
        // Do not call into_inner on this unchecked cyclic object.
        let unused_cycle = cycle.map_root(|_| Atom::num(7));
        assert!(coordinate_independent(&unused_cycle, &[t]));
        assert_eq!(unused_cycle.into_inner(), Atom::num(7));
    }
}
