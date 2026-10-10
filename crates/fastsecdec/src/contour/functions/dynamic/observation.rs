//! Bounded observations of the candidates actually used by a checked native
//! evaluation. Plain factories never call this module. No solver runs here.
use super::requests::Bundle;
use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData, rc::Rc, sync::Arc};
use symbolica::domains::rational::Rational;

#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    pub lambda: Rational,
    pub bits: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Requests(Arc<BTreeMap<Bundle, usize>>);

impl Requests {
    pub(crate) fn new(bundles: impl IntoIterator<Item = Bundle>) -> Result<Self, String> {
        let mut unique = BTreeMap::new();
        for bundle in bundles {
            if bundle.0.is_empty() {
                return Err("empty candidate request bundle".into());
            }
            let next = unique.len();
            if unique.insert(bundle, next).is_some() {
                return Err("duplicate candidate request identity".into());
            }
        }
        Ok(Self(Arc::new(unique)))
    }
    pub(crate) fn bundles(&self) -> impl Iterator<Item = (&Bundle, usize)> {
        self.0.iter().map(|(bundle, index)| (bundle, *index))
    }
    pub(crate) fn begin(&self) -> Attempt {
        Attempt {
            previous: ACTIVE.replace(Some(Active {
                requests: self.clone(),
                values: vec![None; self.0.len()],
                error: None,
            })),
            _thread: PhantomData,
        }
    }
}

struct Active {
    requests: Requests,
    values: Vec<Option<Candidate>>,
    error: Option<String>,
}

thread_local! {
    static ACTIVE:RefCell<Option<Active>>=const {RefCell::new(None)};
}

/// Dropping or unwinding an attempt always restores its surrounding context;
/// neither stale samples nor a failed lower-precision retry can leak outward.
pub(crate) struct Attempt {
    previous: Option<Active>,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for Attempt {
    fn drop(&mut self) {
        ACTIVE.replace(self.previous.take());
    }
}
impl Attempt {
    pub(crate) fn finish(self) -> Result<Vec<Candidate>, String> {
        let active = ACTIVE.take().ok_or("missing dynamic candidate attempt")?;
        if let Some(error) = active.error {
            return Err(error);
        }
        active
            .values
            .into_iter()
            .enumerate()
            .map(|(index, candidate)| {
                candidate
                    .ok_or_else(|| format!("dynamic candidate request {index} was not evaluated"))
            })
            .collect()
    }
}

pub(crate) fn record(bundle: &Bundle, lambda: Rational, bits: u32) -> Result<(), String> {
    ACTIVE.with_borrow_mut(|state| {
        let active = state
            .as_mut()
            .ok_or("checked dynamic callback requires an active candidate attempt")?;
        let result = (|| {
            if lambda <= 0 || bits < 2 {
                return Err("invalid observed dynamic candidate".to_owned());
            }
            let index = *active
                .requests
                .0
                .get(bundle)
                .ok_or("unrecognized dynamic candidate request")?;
            if let Some(previous) = &active.values[index] {
                // Native direct translation can retain repeated pure calls
                // across aliased vector outputs. Each call still computes and
                // returns its own value (and tracked uncertainty). One proof
                // certifies their identical exact centre at the same precision;
                // this is observation coalescing, never numeric memoization.
                return if previous.lambda == lambda && previous.bits == bits {
                    Ok(())
                } else {
                    Err("conflicting dynamic candidates for one request".into())
                };
            }
            active.values[index] = Some(Candidate { lambda, bits });
            Ok(())
        })();
        if let Err(error) = &result {
            active.error.get_or_insert_with(|| error.clone());
        }
        result
    })
}

#[cfg(test)]
mod tests {
    use super::super::requests::Request;
    use super::*;
    fn bundle(namespace: char) -> Bundle {
        Bundle(vec![Request {
            namespace: namespace.to_string().repeat(64),
            face: vec![],
        }])
    }
    #[test]
    fn bounded_candidate_attempts_coalesce_identical_and_reject_conflicting_work() {
        let a = bundle('a');
        let b = bundle('b');
        let requests = Requests::new([a.clone(), b.clone()]).unwrap();
        let attempt = requests.begin();
        record(&b, Rational::from((1, 4)), 106).unwrap();
        record(&a, Rational::from((1, 2)), 53).unwrap();
        record(&a, Rational::from((1, 2)), 53).unwrap();
        let accepted = attempt.finish().unwrap();
        assert_eq!(accepted.len(), 2);
        assert_eq!(accepted[0].lambda, Rational::from((1, 2)));
        assert_eq!(accepted[1].bits, 106);
        let attempt = requests.begin();
        record(&a, Rational::from(1), 53).unwrap();
        record(&a, Rational::from(1), 53).unwrap();
        assert!(
            record(&a, Rational::from((1, 2)), 53)
                .unwrap_err()
                .contains("conflicting")
        );
        assert!(attempt.finish().unwrap_err().contains("conflicting"));
        let attempt = requests.begin();
        record(&a, Rational::from(1), 53).unwrap();
        assert!(
            record(&a, Rational::from(1), 106)
                .unwrap_err()
                .contains("conflicting")
        );
        assert!(attempt.finish().unwrap_err().contains("conflicting"));
        let attempt = requests.begin();
        assert!(record(&bundle('c'), Rational::from(1), 53).is_err());
        assert!(attempt.finish().unwrap_err().contains("unrecognized"));
        assert!(
            requests
                .begin()
                .finish()
                .unwrap_err()
                .contains("not evaluated")
        );
        assert!(record(&a, Rational::from(1), 53).is_err());
    }
    #[test]
    fn nested_retry_and_unwind_preserve_the_outer_candidate() {
        let a = bundle('a');
        let requests = Requests::new([a.clone()]).unwrap();
        let outer = requests.begin();
        record(&a, Rational::from((1, 3)), 53).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _inner = requests.begin();
                record(&a, Rational::from((1, 5)), 192).unwrap();
                panic!("injected callback unwind");
            }))
            .is_err()
        );
        let retry = requests.begin();
        record(&a, Rational::from((1, 7)), 384).unwrap();
        assert_eq!(retry.finish().unwrap()[0].lambda, Rational::from((1, 7)));
        assert_eq!(outer.finish().unwrap()[0].lambda, Rational::from((1, 3)));
    }
}
