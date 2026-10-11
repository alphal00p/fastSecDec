//! Native tagged root callback. Immutable programs, clone-local scratch; no
//! sampled point participates in branch selection or later initial guesses.
use super::{Error, RegularSection, Result, native};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{Arc, LazyLock, Mutex},
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, EvalFn, EvaluationInfo, Symbol},
    domains::{
        float::{Complex, DoubleFloat, Float, Real, RealLike},
        rational::Rational,
    },
    evaluate::{EvaluationDomain, ExpressionEvaluator, OptimizationSettings},
    solve::{BracketedRootConvergence, BracketedRootOptions, nsolve_bracketed},
    symbol,
};
#[derive(Debug)]
pub struct RootProgram {
    exact: ExpressionEvaluator<Complex<Rational>>,
    lower: Rational,
    upper: Rational,
    degree: usize,
}
impl RootProgram {
    pub fn prepare(section: &RegularSection, settings: OptimizationSettings) -> Result<Arc<Self>> {
        let degree = section.coefficients().len() - 1;
        let z = symbol!("fastsecdec::threshold::regular_section::root_argument");
        let mut inputs = vec![Atom::var(z)];
        let mut p = Atom::zero();
        for i in 0..=degree {
            let c = Atom::var(symbol!(&format!(
                "fastsecdec::threshold::regular_section::coefficient_{i}"
            )));
            p += &c * Atom::var(z).pow(i as i64);
            inputs.push(c);
        }
        let exact = Atom::evaluator_multiple(&[p.clone(), p.derivative(z)], &inputs)
            .optimization_settings(settings)
            .build()
            .map_err(native)?;
        Ok(Arc::new(Self {
            exact,
            lower: section.bracket().lower.clone(),
            upper: section.bracket().upper.clone(),
            degree,
        }))
    }
    pub fn call(&self, tag: Symbol, coefficients: &[Atom]) -> Result<Atom> {
        if coefficients.len() != self.degree + 1 {
            return Err(Error::Invalid("root coefficient shape".into()));
        }
        Ok(ROOT.call_args(
            [Atom::num(self.degree), Atom::var(tag)]
                .into_iter()
                .chain(coefficients.iter().cloned()),
        ))
    }
}
#[derive(Clone, Default)]
pub struct Scope(BTreeMap<Symbol, Arc<RootProgram>>);
impl Scope {
    pub fn insert(&mut self, tag: Symbol, program: Arc<RootProgram>) -> Result<()> {
        match self.0.entry(tag) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(program);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                Err(Error::Invalid("duplicate section callback tag".into()))
            }
        }
    }
    #[cfg(test)]
    pub fn native_roundtrip(&self) -> Result<Self> {
        let mut out = Self::default();
        for (tag, p) in &self.0 {
            let bytes = bincode::serde::encode_to_vec(
                (&p.exact, &p.lower, &p.upper, p.degree),
                bincode::config::standard(),
            )
            .map_err(native)?;
            let ((exact, lower, upper, degree), read) =
                bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(native)?;
            if read != bytes.len() {
                return Err(Error::Invalid(
                    "native root helper codec trailing bytes".into(),
                ));
            }
            out.insert(
                *tag,
                Arc::new(RootProgram {
                    exact,
                    lower,
                    upper,
                    degree,
                }),
            )?;
        }
        Ok(out)
    }
    pub fn enter<R>(&self, bits: u32, prepare: impl FnOnce() -> R) -> R {
        struct Restore(Option<(Scope, u32)>);
        impl Drop for Restore {
            fn drop(&mut self) {
                PREPARING.replace(self.0.take());
            }
        }
        let _restore = Restore(PREPARING.replace(Some((self.clone(), bits))));
        prepare()
    }
}
thread_local! {static PREPARING:RefCell<Option<(Scope,u32)>>=const{RefCell::new(None)};static FAILURE:RefCell<Option<String>>=const{RefCell::new(None)};}
pub fn attempt<R>(run: impl FnOnce() -> R) -> std::result::Result<R, String> {
    struct Restore(Option<String>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FAILURE.replace(self.0.take());
        }
    }
    let _restore = Restore(FAILURE.take());
    let value = run();
    match FAILURE.take() {
        Some(e) => Err(e),
        None => Ok(value),
    }
}
fn failed(e: String) {
    FAILURE.with_borrow_mut(|f| {
        if f.is_none() {
            *f = Some(e)
        }
    })
}
fn resolve(tags: &[AtomView<'_>]) -> std::result::Result<(Arc<RootProgram>, u32), String> {
    if tags.len() != 2 {
        return Err("regular-section callback requires degree and owner tags".into());
    }
    let tag = tags[1]
        .as_var_view()
        .ok_or("regular-section owner tag must be a symbol")?
        .get_symbol();
    PREPARING.with_borrow(|s| {
        let (scope, bits) = s
            .as_ref()
            .ok_or("regular-section mapping requires its exact owner scope")?;
        let p = scope
            .0
            .get(&tag)
            .ok_or("regular-section tag is not owned by this map")?;
        if tags[0] != Atom::num(p.degree).as_view() {
            return Err("regular-section degree differs from owner".into());
        }
        Ok((p.clone(), *bits))
    })
}
static ROOT: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "fastsecdec::threshold::regular_section::implicit_v1",
        der = |view, index, out| {
            let Some(args) = view.as_fun_view() else {
                return;
            };
            if index < 2 {
                **out = Atom::zero();
                return;
            }
            let Some(count) = args.get_nargs().checked_sub(2) else {
                return;
            };
            if index >= args.get_nargs() || count < 2 {
                return;
            }
            let root = view.to_owned();
            let derivative: Atom = (1..count)
                .map(|i| Atom::num(i) * args.get(i + 2).to_owned() * root.pow((i - 1) as i64))
                .sum();
            **out = -root.pow((index - 2) as i64) / derivative;
        },
        eval = EvaluationInfo::new()
            .with_tags(2)
            .register_tagged(real::<f64>)
            .register_tagged(real::<DoubleFloat>)
            .register_tagged(real::<Float>)
            .register_tagged(complex::<f64>)
            .register_tagged(complex::<DoubleFloat>)
            .register_tagged(complex::<Float>)
    )
});
// Same minimal precision-constructor adapter as existing contour callbacks;
// arithmetic, root iteration, and derivatives remain native owner operations.
pub trait Number: Real + RealLike + PartialOrd + EvaluationDomain + Send + Sync {
    fn one_at(bits: u32) -> Self;
    fn invalid(bits: u32) -> Self;
}
impl Number for f64 {
    fn one_at(_: u32) -> Self {
        1.
    }
    fn invalid(_: u32) -> Self {
        f64::NAN
    }
}
impl Number for DoubleFloat {
    fn one_at(_: u32) -> Self {
        Self::from(1.)
    }
    fn invalid(_: u32) -> Self {
        Self::from(f64::NAN)
    }
}
impl Number for Float {
    fn one_at(bits: u32) -> Self {
        Self::with_val(bits, 1)
    }
    fn invalid(bits: u32) -> Self {
        Self::with_val(bits, f64::NAN)
    }
}
#[derive(Clone)]
struct Scratch<T> {
    program: ExpressionEvaluator<T>,
    input: Vec<T>,
    output: [T; 2],
    options: BracketedRootOptions<T>,
    lower: T,
    upper: T,
}
struct Workspace<T> {
    owner: Arc<RootProgram>,
    scratch: Mutex<Scratch<T>>,
    bits: u32,
    poisoned: bool,
}
impl<T: Clone> Clone for Workspace<T> {
    fn clone(&self) -> Self {
        let (scratch, poisoned) = match self.scratch.lock() {
            Ok(s) => (s.clone(), self.poisoned),
            Err(e) => (e.into_inner().clone(), true),
        };
        Self {
            owner: self.owner.clone(),
            scratch: Mutex::new(scratch),
            bits: self.bits,
            poisoned,
        }
    }
}
impl<T: Number> Workspace<T> {
    fn new(tags: &[AtomView<'_>]) -> std::result::Result<Self, String> {
        let (owner, requested) = resolve(tags)?;
        let bits = T::FIXED_PRECISION.unwrap_or(requested);
        let one = T::one_at(bits);
        let zero = one.zero();
        let program = owner
            .exact
            .clone()
            .map_coeff_with_prec(&|c| one.from_rational(&c.re), bits);
        let tolerance = one
            .from_usize(2)
            .pow(u64::from(bits.saturating_sub(5)))
            .inv();
        let scratch = Scratch {
            program,
            input: vec![zero.clone(); owner.degree + 2],
            output: [zero.clone(), zero.clone()],
            lower: one.from_rational(&owner.lower),
            upper: one.from_rational(&owner.upper),
            options: BracketedRootOptions {
                absolute_tolerance: zero,
                relative_tolerance: tolerance,
                max_iterations: (bits as usize).saturating_mul(4).max(256),
                initial_guess: None,
                convergence: BracketedRootConvergence::Bracket,
            },
        };
        Ok(Self {
            owner,
            scratch: Mutex::new(scratch),
            bits,
            poisoned: false,
        })
    }
    fn evaluate(
        &self,
        count: usize,
        value: impl Fn(usize) -> std::result::Result<T, String>,
    ) -> std::result::Result<T, String> {
        if self.poisoned {
            return Err("regular-section callback scratch poisoned".into());
        }
        if count != self.owner.degree + 1 {
            return Err("regular-section coefficient shape/nonfinite input".into());
        }
        let mut s = self
            .scratch
            .lock()
            .map_err(|_| "regular-section callback scratch poisoned")?;
        for i in 0..count {
            let v = value(i)?;
            if !v.is_finite() {
                return Err("regular-section nonfinite input".into());
            }
            s.input[i + 1] = v;
        }
        if self.owner.degree == 1 {
            // Exact degree-one arithmetic under the SAME section certificate.
            // No solver or sample-state shortcut is needed for constant bounds.
            if s.input[2].is_zero() {
                return Err("regular-section linear leading coefficient vanished".into());
            }
            let root = -s.input[1].clone() / s.input[2].clone();
            if !root.is_finite() || root < s.lower || root > s.upper {
                return Err("regular-section linear root outside bracket".into());
            }
            return Ok(root);
        }
        let Scratch {
            program,
            input,
            output,
            lower,
            upper,
            options,
        } = &mut *s;
        let root = nsolve_bracketed(lower.clone(), upper.clone(), options, |z| {
            input[0] = z.clone();
            if program.try_evaluate(input, output).is_err() {
                return (T::invalid(self.bits), T::invalid(self.bits));
            }
            (output[0].clone(), output[1].clone())
        })
        .map_err(|e| {
            format!(
                "regular-section native root refinement at {} bits: {e}",
                self.bits
            )
        })?;
        if !root.root.is_finite() || root.root < *lower || root.root > *upper {
            return Err("regular-section nonfinite/outside bracket root".into());
        }
        Ok(root.root)
    }
}
fn real<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<T> {
    let workspace = Workspace::<T>::new(tags);
    Box::new(move |values: &[T]| {
        let result = match &workspace {
            Ok(w) => w.evaluate(values.len(), |i| Ok(values[i].clone())),
            Err(e) => Err(e.clone()),
        };
        result.unwrap_or_else(|e| {
            failed(e);
            T::invalid(workspace.as_ref().map_or(53, |w| w.bits))
        })
    })
}
fn complex<T: Number>(tags: &[AtomView<'_>]) -> EvalFn<Complex<T>> {
    let workspace = Workspace::<T>::new(tags);
    Box::new(move |values: &[Complex<T>]| {
        let result = match &workspace {
            Ok(w) => w.evaluate(values.len(), |i| {
                if !values[i].im.is_zero() {
                    Err("regular-section map coefficients must be real".into())
                } else {
                    Ok(values[i].re.clone())
                }
            }),
            Err(e) => Err(e.clone()),
        };
        match result {
            Ok(v) => Complex::new(v.clone(), v.zero()),
            Err(e) => {
                failed(e);
                let v = T::invalid(workspace.as_ref().map_or(53, |w| w.bits));
                Complex::new(v.clone(), v)
            }
        }
    })
}
