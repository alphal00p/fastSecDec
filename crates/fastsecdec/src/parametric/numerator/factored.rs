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
//! and the form of the polynomial factors change. The constant part of a form
//! enters the source row exactly like a propagator mass. Its Gamma-function
//! shift is the total source order of the product, as for the monomial route.
//!
//! The derivative itself is taken first in formal "jets", symbols standing for
//! the source derivatives of the kernel factors, so intermediate expressions
//! are small polynomials. The collected result is returned as a polynomial in
//! the jets at zero source ([`JetMoment`]); the caller either expands it in the
//! Feynman parameters when that is small or keeps one term per jet monomial.
//! For a family without external momenta, `F = U * L` with `L` affine; the
//! kernel then uses `U^(a+b) L^b`, whose jets of `L` are constants, and absorbs
//! `U^m` into the `U` exponent.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use feynkit_graph::IntegralFamily;
use serde::{Deserialize, Serialize};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::atom::AtomField,
    id::{Pattern, Replacement},
    symbol,
};

/// A numerator `sum_p coefficient_p * prod_j forms[j]^orders_p[j]`.
pub(super) struct FactoredNumerator {
    /// Distinct affine forms in the family scalar products, in first-seen
    /// order, each expanded and scaled to a unit leading rational coefficient.
    pub forms: Vec<Atom>,
    /// Loop-independent coefficient and per-form source order of each product.
    pub products: Vec<(Atom, Vec<u32>)>,
    /// Whether every form is a bare family scalar product, so that the forms
    /// are rows of the scalar-product route.
    pub bare: bool,
}

/// Why a loop-dependent numerator was not parameterized with factored sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeratorFallback {
    /// Every form is a bare family scalar product; the automatic choice then
    /// keeps the historical route, whose source rows are the same.
    BareScalarProducts,
    /// A loop-dependent subexpression is not a polynomial in affine forms.
    Unstructured,
    /// Distributing the numerator exceeds the bound on products.
    TooManyProducts,
    /// More distinct forms than family scalar products.
    TooManyForms,
}

impl std::fmt::Display for NumeratorFallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BareScalarProducts => "every affine factor is a bare scalar product",
            Self::Unstructured => {
                "a loop-dependent subexpression is not a polynomial in affine scalar-product forms"
            }
            Self::TooManyProducts => "the distributed numerator exceeds 4096 products of forms",
            Self::TooManyForms => {
                "the numerator needs more distinct affine forms than the family has scalar products"
            }
        })
    }
}

/// Bound on the distinct products of forms; larger numerators fall back.
const MAX_PRODUCTS: usize = 4096;

/// Bound on the pairwise products formed while distributing one product.
const MAX_PAIRS: usize = 1 << 20;

/// Recognize a polynomial in affine forms: sums, products with loop-independent
/// coefficients and positive integer powers of affine forms, recursively.
///
/// Terms of a sum whose degree in the scalar products is at most one are
/// grouped and admitted as one exactly expanded affine form, however Symbolica
/// stores the sum, and loop-independent factors of a product become part of
/// its coefficient. Only sums of higher degree are distributed; affine forms
/// are never expanded into products. Parts of an affine sum whose coefficients
/// contain `excluded` symbols (regulator and dimension symbols) are split off
/// as coefficients of bare scalar products or constants, so that every form is
/// a valid regulator-independent source row of the native family.
pub(super) fn factor(
    numerator: &Atom,
    family: &IntegralFamily,
    excluded: &[Symbol],
) -> Result<FactoredNumerator, NumeratorFallback> {
    let mut factorizer = Factorizer {
        basis: family.scalar_products(),
        loop_momenta: family.loop_momenta(),
        excluded,
        forms: Vec::new(),
    };
    let products = factorizer.products(numerator.as_view())?;
    if factorizer.forms.len() > factorizer.basis.len() {
        return Err(NumeratorFallback::TooManyForms);
    }
    let count = factorizer.forms.len();
    let products = products
        .terms
        .into_iter()
        .filter(|(coefficient, _)| !coefficient.expand().is_zero())
        .map(|(coefficient, orders)| {
            let mut dense = vec![0; count];
            for (index, order) in orders {
                dense[index] = order;
            }
            (coefficient, dense)
        })
        .collect();
    let bare = factorizer
        .forms
        .iter()
        .all(|form| factorizer.basis.contains(form));
    Ok(FactoredNumerator {
        forms: factorizer.forms,
        products,
        bare,
    })
}

type Orders = BTreeMap<usize, u32>;

/// Products of forms merged by their orders, in first-seen order.
#[derive(Default)]
struct Products {
    terms: Vec<(Atom, Orders)>,
    positions: HashMap<Orders, usize>,
}

impl Products {
    fn constant(coefficient: Atom) -> Self {
        let mut products = Self::default();
        products.terms.push((coefficient, Orders::new()));
        products.positions.insert(Orders::new(), 0);
        products
    }

    fn add(&mut self, coefficient: Atom, orders: Orders) -> Result<(), NumeratorFallback> {
        if let Some(&position) = self.positions.get(&orders) {
            let sum = &self.terms[position].0 + coefficient;
            self.terms[position].0 = sum;
            return Ok(());
        }
        if self.terms.len() >= MAX_PRODUCTS {
            return Err(NumeratorFallback::TooManyProducts);
        }
        self.positions.insert(orders.clone(), self.terms.len());
        self.terms.push((coefficient, orders));
        Ok(())
    }

    fn extend(&mut self, other: Products) -> Result<(), NumeratorFallback> {
        for (coefficient, orders) in other.terms {
            self.add(coefficient, orders)?;
        }
        Ok(())
    }

    /// Distribute a product of two sums of products.
    fn times(&self, other: &Products) -> Result<Products, NumeratorFallback> {
        if self.terms.len().saturating_mul(other.terms.len()) > MAX_PAIRS {
            return Err(NumeratorFallback::TooManyProducts);
        }
        let mut all = Products::default();
        for (a, orders_a) in &self.terms {
            for (b, orders_b) in &other.terms {
                let mut orders = orders_a.clone();
                for (index, order) in orders_b {
                    let entry = orders.entry(*index).or_insert(0);
                    *entry = entry
                        .checked_add(*order)
                        .ok_or(NumeratorFallback::TooManyProducts)?;
                }
                all.add(a * b, orders)?;
            }
        }
        Ok(all)
    }
}

struct Factorizer<'a> {
    basis: &'a [Atom],
    loop_momenta: &'a [Atom],
    excluded: &'a [Symbol],
    forms: Vec<Atom>,
}

impl Factorizer<'_> {
    fn loop_dependent(&self, view: AtomView<'_>) -> bool {
        self.basis.iter().any(|p| view.contains(p.as_view()))
            || self.loop_momenta.iter().any(|p| view.contains(p.as_view()))
            || view
                .get_all_symbols(true)
                .contains(&feynkit_graph::symbols::loop_momentum())
    }

    fn is_basis(&self, view: AtomView<'_>) -> bool {
        self.basis.iter().any(|p| p.as_view() == view)
    }

    /// An upper bound on the total degree in the scalar products, read from
    /// the structure without expansion; `None` if it is not a polynomial in
    /// them with loop-independent coefficients.
    fn degree(&self, view: AtomView<'_>) -> Option<u32> {
        if !self.loop_dependent(view) {
            return Some(0);
        }
        if self.is_basis(view) {
            return Some(1);
        }
        match view {
            AtomView::Add(add) => add
                .iter()
                .try_fold(0u32, |maximum, term| Some(maximum.max(self.degree(term)?))),
            AtomView::Mul(mul) => mul
                .iter()
                .try_fold(0u32, |sum, factor| sum.checked_add(self.degree(factor)?)),
            AtomView::Pow(pow) => {
                let (base, exponent) = pow.get_base_exp();
                let power = u32::try_from(exponent).ok().filter(|p| *p > 0)?;
                self.degree(base)?.checked_mul(power)
            }
            _ => None,
        }
    }

    fn products(&mut self, view: AtomView<'_>) -> Result<Products, NumeratorFallback> {
        match self.degree(view) {
            None => return Err(NumeratorFallback::Unstructured),
            Some(0) => return Ok(Products::constant(view.to_owned())),
            // A product keeps its loop-independent factors as coefficients.
            Some(1) if !matches!(view, AtomView::Mul(_)) => return self.affine(view),
            Some(_) => {}
        }
        match view {
            AtomView::Add(add) => {
                // Group all terms of degree at most one into a single affine
                // form, independent of how the sum was flattened.
                let (linear, higher): (Vec<_>, Vec<_>) = add
                    .iter()
                    .partition(|term| self.degree(*term).is_some_and(|d| d <= 1));
                let mut all = if linear.is_empty() {
                    Products::default()
                } else {
                    self.affine(Atom::add_many(linear).as_view())?
                };
                for term in higher {
                    all.extend(self.products(term)?)?;
                }
                Ok(all)
            }
            AtomView::Mul(mul) => {
                let mut all = Products::constant(Atom::one());
                for factor in mul.iter() {
                    all = all.times(&self.products(factor)?)?;
                }
                Ok(all)
            }
            AtomView::Pow(pow) => {
                let (base, exponent) = pow.get_base_exp();
                let power = u32::try_from(exponent)
                    .ok()
                    .filter(|p| *p > 0)
                    .ok_or(NumeratorFallback::Unstructured)?;
                let base = self.products(base)?;
                let mut all = Products::constant(Atom::one());
                for _ in 0..power {
                    all = all.times(&base)?;
                }
                Ok(all)
            }
            _ => Err(NumeratorFallback::Unstructured),
        }
    }

    /// Admit an expression of degree at most one in the scalar products after
    /// exact expansion: its regulator-free part is one form (or a constant),
    /// and regulator- or dimension-dependent coefficients multiply bare scalar
    /// products or the constant.
    fn affine(&mut self, view: AtomView<'_>) -> Result<Products, NumeratorFallback> {
        let polynomial = view.to_owned().to_polynomial_in_vars_with_field::<u32>(
            self.basis,
            &AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        );
        let mut linear = BTreeMap::<usize, Atom>::new();
        let mut constant = Atom::Zero;
        let mut split = Vec::new();
        for term in &polynomial {
            let degree = term.exponents.iter().map(|e| u64::from(*e)).sum::<u64>();
            if degree > 1 || self.loop_dependent(term.coefficient.as_view()) {
                return Err(NumeratorFallback::Unstructured);
            }
            let variable = term.exponents.iter().position(|e| *e == 1);
            let (free, dependent) = self.split(term.coefficient);
            match variable {
                Some(index) if !free.is_zero() => {
                    linear.insert(index, free);
                }
                None => constant = free,
                Some(_) => {}
            }
            if !dependent.is_zero() {
                split.push((variable, dependent));
            }
        }
        let mut products = Products::default();
        if let Some(leading) = linear.values().next() {
            // Scale to a unit leading rational coefficient so that multiples
            // of one form share a source row.
            let scale = match leading.as_view() {
                AtomView::Num(_) => leading.clone(),
                _ => Atom::one(),
            };
            let form = (linear
                .iter()
                .map(|(index, coefficient)| coefficient * &self.basis[*index])
                .sum::<Atom>()
                + &constant)
                / &scale;
            let index = self.form(form.expand());
            products.add(scale, Orders::from([(index, 1)]))?;
        } else if !constant.is_zero() {
            products.add(constant, Orders::new())?;
        }
        for (variable, coefficient) in split {
            match variable {
                Some(index) => {
                    let index = self.form(self.basis[index].clone());
                    products.add(coefficient, Orders::from([(index, 1)]))?;
                }
                None => products.add(coefficient, Orders::new())?,
            }
        }
        Ok(products)
    }

    /// Split a loop-independent coefficient into its `excluded`-free part and
    /// the rest, after expansion only when it contains an excluded symbol.
    fn split(&self, coefficient: &Atom) -> (Atom, Atom) {
        let excluded = |view: AtomView<'_>| self.excluded.iter().any(|s| view.contains_symbol(*s));
        if !excluded(coefficient.as_view()) {
            return (coefficient.clone(), Atom::Zero);
        }
        let expanded = coefficient.expand();
        match expanded.as_view() {
            AtomView::Add(add) => {
                let (dependent, free): (Vec<_>, Vec<_>) =
                    add.iter().partition(|term| excluded(*term));
                (Atom::add_many(free), Atom::add_many(dependent))
            }
            _ => (Atom::Zero, expanded),
        }
    }

    fn form(&mut self, form: Atom) -> usize {
        match self.forms.iter().position(|f| *f == form) {
            Some(index) => index,
            None => {
                self.forms.push(form);
                self.forms.len() - 1
            }
        }
    }
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
    /// Whether the kernel `U` at zero source is identical to the native `U`
    /// of the term, so its powers can be absorbed into the `U` exponent.
    native_u: bool,
}

impl SourceKernel {
    /// `source_u`/`source_f` are the native extended-family Symanzik polynomials
    /// in `parameters` followed by `sources`; `constants` are the source-free
    /// constant parts of every extended row, in the same order. `u` is the
    /// native Symanzik `U` of the original family.
    pub fn new(
        source_u: Atom,
        source_f: Atom,
        parameters: &[Atom],
        sources: &[Symbol],
        constants: &[Atom],
        u: &Atom,
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
        let native_u = (source_u.replace_multiple(&zero_sources) - u)
            .expand()
            .is_zero();
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
            native_u,
        }
    }

    #[cfg(test)]
    pub fn is_vacuum(&self) -> bool {
        self.vacuum
    }

    /// `d^orders (U^a F^b)|_{s=0} = U^(a-m+u_shift) F^(b-m) N`, `m = sum(orders)`,
    /// with `N` returned as a polynomial in the nonconstant jets at zero source.
    /// The generic kernel has `u_shift = 0`; the vacuum kernel has `u_shift = m`
    /// and a smaller `N`.
    pub fn moment(&self, orders: &[u32], a: &Atom, b: &Atom, namespace: &str) -> JetMoment {
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
        let (jets, monomials) = jets.collect(&differentiated, self.native_u);
        JetMoment {
            jets,
            monomials,
            u_shift: if self.vacuum { total } else { 0 },
        }
    }
}

/// One nonconstant jet of a kernel factor at zero source.
pub(super) struct Jet {
    /// A polynomial in the Feynman parameters.
    pub value: Atom,
    /// Whether this is the native `U` itself (the zeroth jet of `U`).
    pub native_u: bool,
}

/// A source moment `N = sum_t coefficient_t * prod_j jets[j]^exponents_t[j]`.
pub(super) struct JetMoment {
    pub jets: Vec<Jet>,
    /// Coefficients are free of Feynman parameters and sources; they contain
    /// the symbolic kernel exponents and constant jets.
    pub monomials: Vec<(Atom, Vec<u32>)>,
    /// Extra power of `U` for the vacuum kernel.
    pub u_shift: u32,
}

impl JetMoment {
    /// An upper bound on the number of parameter monomials produced by
    /// expanding `N`, before like terms merge: a power `e` of a jet with `t`
    /// terms has at most `binomial(t+e-1, e)` monomials.
    pub fn expansion_bound(&self) -> u128 {
        let sizes = self
            .jets
            .iter()
            .map(|jet| jet.value.nterms() as u128)
            .collect::<Vec<_>>();
        self.monomials
            .iter()
            .map(|(_, exponents)| {
                exponents.iter().zip(&sizes).filter(|(e, _)| **e > 0).fold(
                    1u128,
                    |bound, (e, t)| {
                        bound.saturating_mul(binomial(t + u128::from(*e) - 1, u128::from(*e)))
                    },
                )
            })
            .fold(0u128, u128::saturating_add)
    }

    /// `N` written out in its jets.
    pub fn polynomial(&self) -> Atom {
        Atom::add_many(
            self.monomials
                .iter()
                .map(|(coefficient, exponents)| {
                    coefficient
                        * self
                            .jets
                            .iter()
                            .zip(exponents)
                            .filter(|(_, e)| **e > 0)
                            .map(|(jet, e)| jet.value.pow(Atom::num(*e)))
                            .product::<Atom>()
                })
                .collect::<Vec<_>>(),
        )
    }
}

fn binomial(n: u128, k: u128) -> u128 {
    let k = k.min(n.saturating_sub(k));
    (0..k).fold(1u128, |value, i| value.saturating_mul(n - i) / (i + 1))
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

    /// Collect in placeholders and evaluate the used jets at zero source.
    fn collect(&mut self, expression: &Atom, native_u: bool) -> (Vec<Jet>, Vec<(Atom, Vec<u32>)>) {
        if expression.is_zero() {
            return (Vec::new(), Vec::new());
        }
        let used = self
            .placeholders
            .iter()
            .filter(|(_, s)| expression.contains_symbol(**s))
            .map(|(key, s)| (key.clone(), *s))
            .collect::<Vec<_>>();
        if used.is_empty() {
            return (Vec::new(), vec![(expression.clone(), Vec::new())]);
        }
        let variables = used.iter().map(|(_, s)| Atom::var(*s)).collect::<Vec<_>>();
        let jets = used
            .iter()
            .map(|((k, beta), _)| Jet {
                value: self
                    .value(*k, beta)
                    .expect("placeholders denote nonzero jets")
                    .replace_multiple(&self.kernel.zero_sources),
                native_u: native_u && *k == 0 && beta.iter().all(|order| *order == 0),
            })
            .collect();
        let polynomial = expression.to_polynomial_in_vars_with_field::<u32>(
            &variables,
            &AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        );
        let monomials = (&polynomial)
            .into_iter()
            .map(|term| (term.coefficient.clone(), term.exponents.to_vec()))
            .collect();
        (jets, monomials)
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
        let u0 = u.replace_multiple(&zero);
        for (generic_f, vacuum) in [(&f + &x * &x, false), (f.clone(), true)] {
            let kernel = SourceKernel::new(
                u.clone(),
                generic_f.clone(),
                &parameters,
                &[s, t],
                &constants,
                &u0,
            );
            assert_eq!(kernel.is_vacuum(), vacuum);
            for orders in [[0u32, 0], [1, 0], [2, 1], [3, 2]] {
                let m = orders.iter().sum::<u32>();
                let moment = kernel.moment(&orders, &a, &b, "factored_test::jet");
                let (numerator, shift) = (moment.polynomial(), moment.u_shift);
                assert!(moment.jets.iter().filter(|jet| jet.native_u).count() <= 1);
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

    fn sunset_family() -> (IntegralFamily, [Atom; 6]) {
        use feynkit_kinematics::Kinematics;
        let (k, l, p) = (
            parse!("factored_test::k"),
            parse!("factored_test::l"),
            parse!("factored_test::p"),
        );
        let kin = Kinematics::in_dimension(&parse!("factored_test::D"))
            .unwrap()
            .with_momenta([k.clone(), l.clone(), p.clone()])
            .unwrap()
            .with_mass_squared(&p, parse!("factored_test::s"))
            .unwrap();
        let sp = |a: &Atom, b: &Atom| kin.scalar_product(a, b).unwrap();
        let products = [
            sp(&k, &k),
            sp(&l, &l),
            sp(&k, &l),
            sp(&k, &p),
            sp(&l, &p),
            sp(&p, &p),
        ];
        let [kk, ll, kl, kp, lp, pp] = products.clone();
        let d3 = &kk + &ll + &pp + Atom::num(2) * (&kl + &kp + &lp) - Atom::num(1);
        let family = IntegralFamily::new(
            vec![k, l],
            vec![p],
            vec![&kk - Atom::num(1), &ll - Atom::num(1), d3],
            &kin,
        )
        .unwrap();
        (family, products)
    }

    /// The decomposition depends on the mathematical sums, not on how
    /// Symbolica flattened them, and keeps regulator symbols out of forms.
    #[test]
    fn factorization_groups_affine_sums_and_splits_regulator_coefficients() {
        let (family, [kk, ll, kl, kp, lp, _]) = sunset_family();
        let eps = symbol!("factored_test::eps");
        let excluded = [eps];
        let a = &kk + Atom::num(2) * &kp - Atom::num(3);
        let b = &lp + Atom::num(2);
        let c = &kl + &ll - Atom::num(5);
        let count = |numerator: Atom| {
            let factored = factor(&numerator, &family, &excluded).unwrap();
            (factored.forms.len(), factored.products.len(), factored.bare)
        };
        // A - C is stored as a flat sum of the terms of A and -1 * C.
        assert_eq!(count(a.pow(2) * (&a - &c) * &a), (2, 1, false));
        // A * B + C flattens the terms of C into the outer sum.
        assert_eq!(
            count(((&a * &b + &c).pow(2) - Atom::num(3)) * &a),
            (3, 4, false)
        );
        // A numeric multiple inside a form shares the row of the form.
        let scaled = Atom::num(2) * (&kl + &kp + &lp) - Atom::num(4);
        assert_eq!(
            count(scaled.pow(3) * (&kl + &kp + &lp - Atom::num(2))),
            (1, 1, false)
        );
        // A regulator in the constant of a form, or as a coefficient, is split.
        let e = Atom::var(eps);
        assert_eq!(count((&a - &e).pow(2) * &b), (2, 3, false));
        // A regulator factor of a form stays its coefficient.
        assert_eq!(count(&e * &a * (Atom::num(3) * &e * &b)), (2, 1, false));
        let mixed = &e * b.pow(2) + (Atom::num(4) - Atom::num(2) * &e) * &kk;
        let factored = factor(&mixed, &family, &excluded).unwrap();
        assert_eq!(factored.forms.len(), 2);
        assert!(factored.forms.contains(&kk));
        // Bare scalar products are recognized as such.
        assert_eq!(count(Atom::num(2) * &kk * kp.pow(2)), (2, 1, true));
        // Not a polynomial in the scalar products.
        assert_eq!(
            factor(&kk.pow(Atom::num(-1)), &family, &excluded).err(),
            Some(NumeratorFallback::Unstructured)
        );
    }

    /// Splitting all monomials of a dense polynomial in many forms exceeds the
    /// product bound instead of growing without limit.
    #[test]
    fn factorization_bounds_products_and_forms() {
        let (family, [kk, ll, kl, kp, lp, _]) = sunset_family();
        let forms = [
            &kk + Atom::num(1),
            &ll + Atom::num(2),
            &kl + Atom::num(3),
            &kp + Atom::num(4),
            &lp + Atom::num(5),
        ];
        let dense = forms
            .iter()
            .map(|form| form * form + Atom::num(1))
            .product::<Atom>()
            .pow(Atom::num(6));
        assert_eq!(
            factor(&dense, &family, &[]).err(),
            Some(NumeratorFallback::TooManyProducts)
        );
        // Five scalar products: six distinct forms exceed the basis.
        let many = forms.iter().product::<Atom>() * (&kk + &ll + Atom::num(7));
        assert_eq!(
            factor(&many, &family, &[]).err(),
            Some(NumeratorFallback::TooManyForms)
        );
    }
}
