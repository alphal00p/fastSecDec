//! Gaussian moments for numerators that are products of affine source forms.
//!
//! A contracted numerator such as `c * A * B^4 * C^4`, where every loop-dependent
//! factor is affine in the family's loop scalar products, is parameterized by
//! appending the distinct forms themselves (here `A`, `B`, `C`) as Gaussian
//! source rows. One mixed source derivative of order `(1, 4, 4)` then replaces
//! the expansion into scalar-product monomials and one derivative per monomial.
//!
//! Both routes differentiate the same Schwinger-space generating function, so
//! the resulting projective densities agree exactly; only the number of terms
//! and the form of the polynomial factor change. The constant part of a form
//! enters the source row exactly like a propagator mass. Its Gamma-function
//! shift is the total source order of the product, as for the monomial route.
//!
//! The derivative itself is taken first in formal "jets", symbols standing for
//! the source derivatives of the kernel factors, so intermediate expressions
//! are small polynomials. Only the collected result is written in terms of the
//! native jet polynomials at zero source. For a family without external
//! momenta, `F = U * L` with `L` affine; the kernel then uses `U^(a+b) L^b`,
//! whose jets of `L` are constants, and absorbs `U^m` into the `U` exponent.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use feynkit_graph::IntegralFamily;
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::atom::AtomField,
    id::{Pattern, Replacement},
    symbol,
};

/// A numerator `sum_p coefficient_p * prod_j forms[j]^orders_p[j]`.
pub(super) struct FactoredNumerator {
    /// Distinct affine forms in the family scalar products, in first-seen order.
    pub forms: Vec<Atom>,
    /// Loop-independent coefficient and per-form source order of each product.
    pub products: Vec<(Atom, Vec<u32>)>,
}

/// Bound on the number of products before merging; larger expansions keep the
/// scalar-product route.
const MAX_PRODUCTS: usize = 4096;

/// Recognize a polynomial in affine forms: sums, products with loop-independent
/// coefficients and positive integer powers of affine forms, recursively. Only
/// sums of such products are distributed; the affine forms are never expanded.
///
/// Returns `None`, selecting the scalar-product monomial route, unless at least
/// one form differs from a bare family scalar product. A form must be free of
/// `excluded` symbols (regulator and dimension symbols) in its coefficients, so
/// that it is a valid regulator-independent source row of the native family.
pub(super) fn factor(
    numerator: &Atom,
    family: &IntegralFamily,
    excluded: &[Symbol],
) -> Option<FactoredNumerator> {
    let mut factorizer = Factorizer {
        basis: family.scalar_products(),
        loop_momenta: family.loop_momenta(),
        excluded,
        forms: Vec::new(),
        expanded_forms: Vec::new(),
    };
    let products = factorizer.products(numerator.as_view())?;
    if factorizer
        .expanded_forms
        .iter()
        .all(|form| factorizer.basis.contains(form))
    {
        return None;
    }
    // Merge products with equal source orders, keeping first-seen order.
    let count = factorizer.forms.len();
    let mut merged: Vec<(Atom, Vec<u32>)> = Vec::new();
    let mut positions = HashMap::<Vec<u32>, usize>::new();
    for (coefficient, orders) in products {
        let mut dense = vec![0; count];
        for (index, order) in orders {
            dense[index] = order;
        }
        match positions.get(&dense) {
            Some(&position) => merged[position].0 = &merged[position].0 + coefficient,
            None => {
                positions.insert(dense.clone(), merged.len());
                merged.push((coefficient, dense));
            }
        }
    }
    merged.retain(|(coefficient, _)| !coefficient.is_zero());
    Some(FactoredNumerator {
        forms: factorizer.forms,
        products: merged,
    })
}

type Product = (Atom, BTreeMap<usize, u32>);

struct Factorizer<'a> {
    basis: &'a [Atom],
    loop_momenta: &'a [Atom],
    excluded: &'a [Symbol],
    forms: Vec<Atom>,
    expanded_forms: Vec<Atom>,
}

impl Factorizer<'_> {
    fn loop_dependent(&self, view: AtomView<'_>) -> bool {
        self.basis.iter().any(|p| view.contains(p.as_view()))
            || self.loop_momenta.iter().any(|p| view.contains(p.as_view()))
            || view
                .get_all_symbols(true)
                .contains(&feynkit_graph::symbols::loop_momentum())
    }

    /// A cheap syntactic test before the exact affine admission: a sum of
    /// loop-independent terms and coefficient multiples of scalar products.
    fn structurally_affine(&self, view: AtomView<'_>) -> bool {
        let linear = |term: AtomView<'_>| {
            !self.loop_dependent(term)
                || self.basis.iter().any(|p| p.as_view() == term)
                || matches!(term, AtomView::Mul(mul) if {
                    let mut dependent = mul.iter().filter(|f| self.loop_dependent(*f));
                    matches!(
                        (dependent.next(), dependent.next()),
                        (Some(factor), None) if self.basis.iter().any(|p| p.as_view() == factor)
                    )
                })
        };
        match view {
            AtomView::Add(add) => add.iter().all(linear),
            view => linear(view),
        }
    }

    fn form(&mut self, view: AtomView<'_>) -> Option<usize> {
        if !affine(view, self.basis, self.excluded) {
            return None;
        }
        let expanded = view.to_owned().expand();
        Some(
            match self.expanded_forms.iter().position(|f| *f == expanded) {
                Some(index) => index,
                None => {
                    self.forms.push(view.to_owned());
                    self.expanded_forms.push(expanded);
                    self.forms.len() - 1
                }
            },
        )
    }

    fn products(&mut self, view: AtomView<'_>) -> Option<Vec<Product>> {
        if !self.loop_dependent(view) {
            return Some(vec![(view.to_owned(), BTreeMap::new())]);
        }
        if self.structurally_affine(view) {
            let index = self.form(view)?;
            return Some(vec![(Atom::one(), BTreeMap::from([(index, 1)]))]);
        }
        match view {
            AtomView::Add(add) => {
                let mut all = Vec::new();
                for term in add.iter() {
                    all.extend(self.products(term)?);
                    if all.len() > MAX_PRODUCTS {
                        return None;
                    }
                }
                Some(all)
            }
            AtomView::Mul(mul) => {
                let mut all = vec![(Atom::one(), BTreeMap::new())];
                for factor in mul.iter() {
                    let factor = self.products(factor)?;
                    all = combine(&all, &factor)?;
                }
                Some(all)
            }
            AtomView::Pow(pow) => {
                let (base, exponent) = pow.get_base_exp();
                let power = u32::try_from(exponent).ok().filter(|p| *p > 0)?;
                if self.structurally_affine(base) {
                    let index = self.form(base)?;
                    return Some(vec![(Atom::one(), BTreeMap::from([(index, power)]))]);
                }
                let base = self.products(base)?;
                let mut all = vec![(Atom::one(), BTreeMap::new())];
                for _ in 0..power {
                    all = combine(&all, &base)?;
                }
                Some(all)
            }
            _ => None,
        }
    }
}

/// Distribute a product of two sums of products, within [`MAX_PRODUCTS`].
fn combine(left: &[Product], right: &[Product]) -> Option<Vec<Product>> {
    if left.len().checked_mul(right.len())? > MAX_PRODUCTS {
        return None;
    }
    let mut all = Vec::with_capacity(left.len() * right.len());
    for (a, orders_a) in left {
        for (b, orders_b) in right {
            let mut orders = orders_a.clone();
            for (index, order) in orders_b {
                let entry = orders.entry(*index).or_insert(0);
                *entry = entry.checked_add(*order)?;
            }
            all.push((a * b, orders));
        }
    }
    Some(all)
}

/// Exactly affine, with a nonconstant part and loop-independent coefficients
/// free of `excluded` symbols (scalar-product labels may contain them).
fn affine(form: AtomView<'_>, basis: &[Atom], excluded: &[Symbol]) -> bool {
    let polynomial = form.to_owned().to_polynomial_in_vars_with_field::<u32>(
        basis,
        &AtomField {
            statistical_zero_test: false,
            ..AtomField::new()
        },
    );
    let mut linear = false;
    for term in &polynomial {
        let degree = term.exponents.iter().map(|e| u64::from(*e)).sum::<u64>();
        if degree > 1
            || basis.iter().any(|p| term.coefficient.contains(p.as_view()))
            || excluded
                .iter()
                .any(|s| term.coefficient.contains_symbol(*s))
        {
            return false;
        }
        linear |= degree == 1;
    }
    linear
}

/// The kernel `prod_k P_k(x, s)^(e_k)` whose source derivatives are required,
/// together with the fixed rules for jets (source derivatives) of each `P_k`.
pub(super) struct SourceKernel {
    factors: Vec<Atom>,
    sources: Vec<Symbol>,
    parameters: Vec<Atom>,
    zero_sources: Vec<Replacement>,
    /// Whether the kernel is `U^(a+b) L^b` with `F = U L`.
    vacuum: bool,
}

impl SourceKernel {
    /// `source_u`/`source_f` are the native extended-family Symanzik polynomials
    /// in `parameters` followed by `sources`; `constants` are the source-free
    /// constant parts of every extended row, in the same order.
    pub fn new(
        source_u: Atom,
        source_f: Atom,
        parameters: &[Atom],
        sources: &[Symbol],
        constants: &[Atom],
    ) -> Self {
        let zero_sources = sources
            .iter()
            .map(|s| {
                Replacement::new(
                    Pattern::Literal(Atom::var(*s)),
                    Pattern::Literal(Atom::Zero),
                )
            })
            .collect::<Vec<_>>();
        // F = Q.adj(M).Q - U*J. Without a linear term, L = -J is affine. Accept
        // the factorization only after an exact expanded identity check.
        let variables = parameters
            .iter()
            .cloned()
            .chain(sources.iter().map(|s| Atom::var(*s)))
            .collect::<Vec<_>>();
        let l = -variables
            .iter()
            .zip(constants)
            .map(|(variable, constant)| variable * constant)
            .sum::<Atom>();
        let l = l.expand();
        let vacuum = !l.is_zero() && (&source_f - &source_u * &l).expand().is_zero();
        let factors = if vacuum {
            vec![source_u, l]
        } else {
            vec![source_u, source_f]
        };
        Self {
            factors,
            sources: sources.to_vec(),
            parameters: parameters.to_vec(),
            zero_sources,
            vacuum,
        }
    }

    #[cfg(test)]
    pub fn is_vacuum(&self) -> bool {
        self.vacuum
    }

    /// Return `(N, u_shift)` with `d^orders (U^a F^b)|_{s=0} =
    /// U^(a-m+u_shift) F^(b-m) N`, `m = sum(orders)`. The generic kernel has
    /// `u_shift = 0`; the vacuum kernel has `u_shift = m` and a smaller `N`.
    pub fn moment(&self, orders: &[u32], a: &Atom, b: &Atom, namespace: &str) -> (Atom, u32) {
        debug_assert_eq!(orders.len(), self.sources.len());
        let total = orders.iter().sum::<u32>();
        let exponents = if self.vacuum {
            vec![(a + b).expand(), b.clone()]
        } else {
            vec![a.clone(), b.clone()]
        };
        let mut jets = Jets::new(self, namespace);
        let base = (0..self.factors.len())
            .map(|k| jets.atom(k, &vec![0; orders.len()]))
            .product::<Atom>();
        let mut differentiated = Atom::one();
        let mut completed = 0u32;
        for (source, &count) in orders.iter().enumerate() {
            for _ in 0..count {
                let first = (0..self.factors.len())
                    .map(|k| {
                        let mut beta = vec![0; orders.len()];
                        beta[source] = 1;
                        jets.atom(k, &beta)
                    })
                    .collect::<Vec<_>>();
                // d[prod P_k^(e_k-r) D] = prod P_k^(e_k-r-1)
                //   * [prod P_k dD + sum_k (e_k-r) dP_k prod_{l!=k} P_l D].
                let mut next = &base * jets.differentiate(&differentiated, source);
                for (k, first) in first.iter().enumerate() {
                    if first.is_zero() {
                        continue;
                    }
                    let others = (0..self.factors.len())
                        .filter(|l| *l != k)
                        .map(|l| jets.atom(l, &vec![0; orders.len()]))
                        .product::<Atom>();
                    next +=
                        (&exponents[k] - Atom::num(completed)) * first * others * &differentiated;
                }
                differentiated = next.expand();
                completed += 1;
            }
        }
        let numerator = jets.substitute(&differentiated);
        (numerator, if self.vacuum { total } else { 0 })
    }
}

/// Lazily differentiated kernel factors and their formal placeholder symbols.
struct Jets<'a> {
    kernel: &'a SourceKernel,
    namespace: String,
    /// Jets as functions of parameters and sources; `None` is identically zero.
    values: HashMap<(usize, Vec<u32>), Option<Atom>>,
    /// Nonconstant jets are represented by placeholders until collection.
    placeholders: BTreeMap<(usize, Vec<u32>), Symbol>,
    reverse: HashMap<Symbol, (usize, Vec<u32>)>,
}

impl<'a> Jets<'a> {
    fn new(kernel: &'a SourceKernel, namespace: &str) -> Self {
        Self {
            kernel,
            namespace: namespace.into(),
            values: HashMap::new(),
            placeholders: BTreeMap::new(),
            reverse: HashMap::new(),
        }
    }

    fn value(&mut self, k: usize, beta: &[u32]) -> Option<Atom> {
        let key = (k, beta.to_vec());
        if let Some(value) = self.values.get(&key) {
            return value.clone();
        }
        let value = match beta.iter().position(|order| *order > 0) {
            None => Some(self.kernel.factors[k].clone()),
            Some(source) => {
                let mut parent = beta.to_vec();
                parent[source] -= 1;
                self.value(k, &parent).and_then(|parent| {
                    let derivative = parent.derivative(self.kernel.sources[source]);
                    (!derivative.is_zero()).then_some(derivative)
                })
            }
        };
        self.values.insert(key, value.clone());
        value
    }

    /// The jet as it appears in the formal derivative: zero, an inlined
    /// constant (whose own derivatives vanish), or a placeholder symbol.
    fn atom(&mut self, k: usize, beta: &[u32]) -> Atom {
        let Some(value) = self.value(k, beta) else {
            return Atom::Zero;
        };
        let constant = !self
            .kernel
            .parameters
            .iter()
            .any(|p| value.contains(p.as_view()))
            && !self
                .kernel
                .sources
                .iter()
                .any(|s| value.contains_symbol(*s));
        if constant {
            return value;
        }
        let key = (k, beta.to_vec());
        if let Some(symbol) = self.placeholders.get(&key) {
            return Atom::var(*symbol);
        }
        let name = format!(
            "{}_{}_{}",
            self.namespace,
            k,
            beta.iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join("_")
        );
        let placeholder = symbol!(&name);
        self.placeholders.insert(key.clone(), placeholder);
        self.reverse.insert(placeholder, key);
        Atom::var(placeholder)
    }

    /// Source derivative of a polynomial in placeholders: each placeholder is
    /// replaced by its next jet through the chain rule.
    fn differentiate(&mut self, expression: &Atom, source: usize) -> Atom {
        let present = expression
            .get_all_symbols(false)
            .into_iter()
            .filter_map(|s| self.reverse.get(&s).cloned().map(|key| (s, key)))
            .collect::<BTreeMap<_, _>>();
        let mut result = Atom::Zero;
        for (placeholder, (k, mut beta)) in present {
            beta[source] += 1;
            let next = self.atom(k, &beta);
            if next.is_zero() {
                continue;
            }
            result += expression.derivative(placeholder) * next;
        }
        result
    }

    /// Collect in placeholders, then substitute the jets at zero source.
    fn substitute(&mut self, expression: &Atom) -> Atom {
        if expression.is_zero() {
            return Atom::Zero;
        }
        let used = self
            .placeholders
            .iter()
            .filter(|(_, s)| expression.contains_symbol(**s))
            .map(|(key, s)| (key.clone(), *s))
            .collect::<Vec<_>>();
        if used.is_empty() {
            return expression.clone();
        }
        let variables = used.iter().map(|(_, s)| Atom::var(*s)).collect::<Vec<_>>();
        let values = used
            .iter()
            .map(|((k, beta), _)| {
                self.value(*k, beta)
                    .expect("placeholders denote nonzero jets")
                    .replace_multiple(&self.kernel.zero_sources)
            })
            .collect::<Vec<_>>();
        let polynomial = expression.to_polynomial_in_vars_with_field::<u32>(
            &variables,
            &AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        );
        let mut result = Atom::Zero;
        for term in &polynomial {
            let monomial = values
                .iter()
                .zip(term.exponents)
                .filter(|(_, e)| **e > 0)
                .map(|(value, e)| value.pow(Atom::num(*e)))
                .product::<Atom>();
            result += term.coefficient * monomial;
        }
        result
    }
}

/// Collision-free placeholder namespace for one parameterization.
pub(super) fn placeholder_namespace(occupied: &BTreeSet<Symbol>) -> String {
    let mut next = 0usize;
    loop {
        let namespace = format!("fastsecdec::numerator_jet{next}");
        if !occupied
            .iter()
            .any(|s| s.get_name().starts_with(&namespace))
        {
            return namespace;
        }
        next += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::parse;

    #[test]
    fn jets_reproduce_native_mixed_derivatives_for_generic_and_vacuum_kernels() {
        let s = symbol!("factored_test::s");
        let t = symbol!("factored_test::t");
        let [x, y] = [parse!("factored_test::x"), parse!("factored_test::y")];
        let (sa, ta) = (Atom::var(s), Atom::var(t));
        let u = &x + &y + &sa * &y + &ta * &x + &sa * &ta;
        // Vacuum: F = U*L, L affine with constants -J.
        let l = &x * 2 + &y * 3 + &sa * 5 + &ta * 7;
        let f = (&u * &l).expand();
        let parameters = [x.clone(), y.clone()];
        let constants = [Atom::num(-2), Atom::num(-3), Atom::num(-5), Atom::num(-7)];
        let a = parse!("factored_test::a");
        let b = parse!("factored_test::b");
        let zero = [s, t].map(|source| {
            Replacement::new(
                Pattern::Literal(Atom::var(source)),
                Pattern::Literal(Atom::Zero),
            )
        });
        for (generic_f, vacuum) in [(&f + &x * &x, false), (f.clone(), true)] {
            let kernel = SourceKernel::new(
                u.clone(),
                generic_f.clone(),
                &parameters,
                &[s, t],
                &constants,
            );
            assert_eq!(kernel.is_vacuum(), vacuum);
            for orders in [[0u32, 0], [1, 0], [2, 1], [3, 2]] {
                let m = orders.iter().sum::<u32>();
                let (numerator, shift) = kernel.moment(&orders, &a, &b, "factored_test::jet");
                // Compare against direct differentiation with integer powers.
                for (av, bv) in [(9i64, 7i64), (8, 6)] {
                    let mut expected = u.pow(Atom::num(av)) * generic_f.pow(Atom::num(bv));
                    for (source, count) in [s, t].iter().zip(orders) {
                        for _ in 0..count {
                            expected = expected.derivative(*source);
                        }
                    }
                    let expected = expected.replace_multiple(&zero);
                    let bind = |e: &Atom| {
                        e.replace(a.clone())
                            .with(Atom::num(av))
                            .replace(b.clone())
                            .with(Atom::num(bv))
                    };
                    let u0 = u.replace_multiple(&zero);
                    let f0 = generic_f.replace_multiple(&zero);
                    let actual = bind(&numerator)
                        * u0.pow(Atom::num(av - i64::from(m) + i64::from(shift)))
                        * f0.pow(Atom::num(bv - i64::from(m)));
                    assert!(
                        (actual - expected).expand().is_zero(),
                        "orders={orders:?} vacuum={vacuum}"
                    );
                }
            }
        }
    }
}
