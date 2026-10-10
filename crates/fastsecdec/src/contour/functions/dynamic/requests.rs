//! Diagnostic associations are attached only at executable lowering, after
//! symbolic cancellations and exact/stochastic classification are complete.
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    id::{Pattern, Replacement},
    symbol,
};

const NAMESPACE_PREFIX: &str = "fastsecdec::contour::dynamic::request_namespace_";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) struct Request {
    pub namespace: String,
    /// Restrictions of the unchanged full-sector coordinate list.
    pub face: Vec<(usize, u8)>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) struct Bundle(pub Vec<Request>);

/// Diagnostic context is stored beside an exact mathematical root, never in
/// its arguments. Cross-record native Atom cancellations therefore remain
/// possible before the caller binds the summed exact contribution.
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub(crate) struct ExactRequest {
    pub root: Atom,
    #[bincode(with_serde)]
    pub bundle: Bundle,
}

/// Find mathematical radius calls surviving native aggregate simplification.
/// This only walks the owner's Atoms and does not rewrite them.
pub(crate) fn exact_roots<'a>(expressions: impl IntoIterator<Item = &'a Atom>) -> BTreeSet<Atom> {
    let mut roots = BTreeSet::new();
    for expression in expressions {
        let _ = expression.replace_map(|term, _, _| {
            if term
                .as_fun_view()
                .is_some_and(|function| function.get_symbol() == *super::STRENGTH)
            {
                roots.insert(term.to_owned());
            }
        });
    }
    roots
}

/// Merge contexts only for roots that still occur in the summed mathematical
/// expression. Do not infer that an absent/cancelled factor is a physical pole.
pub(crate) fn merge_exact_requests(
    expressions: &[Atom],
    requests: impl IntoIterator<Item = ExactRequest>,
) -> Result<Vec<ExactRequest>, String> {
    let mut needed = exact_roots(expressions)
        .into_iter()
        .map(|root| (root, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for request in requests {
        // Reuse the canonical native literal parser for complete bundle
        // ordering/namespace/face admission, also for decoded serde values.
        Bundle::from_atom(request.bundle.atom().as_view())?;
        let Some(contexts) = needed.get_mut(&request.root) else {
            continue;
        };
        contexts.extend(request.bundle.0);
    }
    needed
        .into_iter()
        .map(|(root, contexts)| {
            if contexts.is_empty() {
                return Err("surviving exact radius lacks its saved diagnostic context".into());
            }
            Ok(ExactRequest {
                root,
                bundle: Bundle(contexts.into_iter().collect()),
            })
        })
        .collect()
}

impl Bundle {
    pub(crate) fn atom(&self) -> Atom {
        symbol!("fastsecdec::contour::dynamic::request_bundle_v1").call_args(self.0.iter().map(
            |request| {
                symbol!("fastsecdec::contour::dynamic::request_face_v1").call_args(
                    std::iter::once(Atom::var(symbol!(format!(
                        "{NAMESPACE_PREFIX}{}",
                        request.namespace
                    ))))
                    .chain(
                        request
                            .face
                            .iter()
                            .flat_map(|(axis, value)| [Atom::num(*axis), Atom::num(*value)]),
                    ),
                )
            },
        ))
    }

    /// Parse the native literal metadata once at preparation, never per sample.
    /// Association with actual full-sector dimensions is checked by the owner.
    pub(crate) fn from_atom(atom: AtomView<'_>) -> Result<Self, String> {
        let bundle = atom
            .as_fun_view()
            .ok_or("dynamic request bundle must be a literal native function")?;
        if bundle.get_symbol() != symbol!("fastsecdec::contour::dynamic::request_bundle_v1")
            || bundle.get_nargs() == 0
        {
            return Err("invalid or empty dynamic request bundle".into());
        }
        let mut requests = Vec::with_capacity(bundle.get_nargs());
        for index in 0..bundle.get_nargs() {
            let atom = bundle.get(index);
            let face = atom
                .as_fun_view()
                .ok_or("dynamic request face must be a literal native function")?;
            if face.get_symbol() != symbol!("fastsecdec::contour::dynamic::request_face_v1")
                || face.get_nargs() == 0
                || face.get_nargs().is_multiple_of(2)
            {
                return Err("invalid dynamic request face schema".into());
            }
            let namespace = face
                .get(0)
                .as_var_view()
                .ok_or("dynamic request namespace must be a native symbol")?
                .get_symbol();
            let namespace = namespace
                .get_name()
                .strip_prefix(NAMESPACE_PREFIX)
                .ok_or("invalid dynamic request namespace")?
                .to_owned();
            if !valid_namespace(&namespace) {
                return Err("invalid dynamic request namespace digest".into());
            }
            let mut coordinates = Vec::new();
            for index in (1..face.get_nargs()).step_by(2) {
                let axis =
                    u64::try_from(face.get(index)).map_err(|_| "invalid dynamic face axis")?;
                let axis = usize::try_from(axis)
                    .map_err(|_| "dynamic face axis exceeds platform range")?;
                let value =
                    u64::try_from(face.get(index + 1)).map_err(|_| "invalid dynamic face value")?;
                if value > 1
                    || coordinates
                        .last()
                        .is_some_and(|(previous, _)| *previous >= axis)
                {
                    return Err(
                        "unordered, repeated, or non-boundary dynamic face coordinate".into(),
                    );
                }
                coordinates.push((axis, value as u8));
            }
            let request = Request {
                namespace,
                face: coordinates,
            };
            if requests.last().is_some_and(|previous| previous >= &request) {
                return Err("duplicate or unordered dynamic request bundle".into());
            }
            requests.push(request);
        }
        Ok(Self(requests))
    }
}

fn valid_namespace(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Read only late executable metadata. A physical symbol that happens to use
/// a reserved-looking name is not accepted as a diagnostic request.
pub(crate) fn referenced_bundles<'a>(
    expressions: impl IntoIterator<Item = &'a Atom>,
) -> Result<BTreeSet<Bundle>, String> {
    let mut result = BTreeSet::new();
    let mut error = None;
    for expression in expressions {
        let _ = expression.replace_map(|term, _, _| {
            let Some(function) = term.as_fun_view() else {
                return;
            };
            if function.get_symbol() != super::requested::symbol() {
                return;
            }
            if function.get_nargs() < 6 {
                error.get_or_insert_with(|| "invalid surviving dynamic request arity".to_owned());
                return;
            }
            match Bundle::from_atom(function.get(2)) {
                Ok(bundle) => {
                    result.insert(bundle);
                }
                Err(reason) => {
                    error.get_or_insert(reason);
                }
            }
        });
    }
    error.map_or(Ok(result), Err)
}

pub(crate) fn referenced_namespaces(expressions: &[Atom]) -> Result<BTreeSet<String>, String> {
    Ok(referenced_bundles(expressions)?
        .into_iter()
        .flat_map(|bundle| bundle.0.into_iter().map(|request| request.namespace))
        .collect())
}

#[derive(Default)]
pub(crate) struct Lookup(BTreeMap<Atom, BTreeSet<Request>>);

impl Lookup {
    pub(crate) fn exact_requests(&self, expressions: &[Atom]) -> Result<Vec<ExactRequest>, String> {
        exact_roots(expressions)
            .into_iter()
            .map(|root| {
                let bundle = self.bundle(&root)?;
                Ok(ExactRequest { root, bundle })
            })
            .collect()
    }
    /// Restrict the existing full-strength Atom through native substitution.
    /// Equal restricted roots share the complete set of associated faces.
    pub(crate) fn insert(
        &mut self,
        namespace: &str,
        full_strength: &Atom,
        parameters: &[Symbol],
        faces: &[Vec<(usize, u8)>],
    ) -> Result<(), String> {
        if !valid_namespace(namespace) || faces.is_empty() {
            return Err("dynamic request source lacks a namespace or face coverage".into());
        }
        let source = full_strength
            .as_fun_view()
            .ok_or("dynamic request source must retain its full-strength callback")?;
        if source.get_symbol() != *super::STRENGTH {
            return Err("dynamic request source is not the mathematical strength callback".into());
        }
        let mut seen = BTreeSet::new();
        for face in faces {
            if face
                .iter()
                .any(|(axis, value)| *axis >= parameters.len() || *value > 1)
                || face.windows(2).any(|pair| pair[0].0 >= pair[1].0)
                || !seen.insert(face)
            {
                return Err("invalid or duplicate full-sector dynamic face restriction".into());
            }
            let restricted = full_strength.replace_multiple(face.iter().map(|(axis, value)| {
                Replacement::new(
                    Pattern::Literal(Atom::var(parameters[*axis])),
                    Pattern::Literal(Atom::num(*value)),
                )
            }));
            self.0.entry(restricted).or_default().insert(Request {
                namespace: namespace.to_owned(),
                face: face.clone(),
            });
        }
        Ok(())
    }

    pub(crate) fn bundle(&self, strength: &Atom) -> Result<Bundle, String> {
        self.0
            .get(strength)
            .map(|requests| Bundle(requests.iter().cloned().collect()))
            .ok_or_else(|| {
                "surviving dynamic strength does not match a retained native face restriction"
                    .into()
            })
    }

    /// Attach static metadata after symbolic processing. `callback` is the
    /// registered numerical function with the same IFT hooks and one extra tag.
    pub(crate) fn lower(&self, expression: &Atom, callback: Symbol) -> Result<Atom, String> {
        self.lower_with(expression, callback, &[], None)
    }

    /// Numerical-dual seeds apply a face after differentiation. Restrict only
    /// lookup metadata here; the callback's mathematical arguments stay full.
    pub(crate) fn lower_on_face(
        &self,
        expression: &Atom,
        callback: Symbol,
        parameters: &[Symbol],
        face: &[(usize, u8)],
    ) -> Result<Atom, String> {
        if face
            .iter()
            .any(|(axis, value)| *axis >= parameters.len() || *value > 1)
            || face.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err("invalid dynamic dual request face".into());
        }
        let rules = face
            .iter()
            .map(|(axis, value)| {
                Replacement::new(
                    Pattern::Literal(Atom::var(parameters[*axis])),
                    Pattern::Literal(Atom::num(*value)),
                )
            })
            .collect::<Vec<_>>();
        self.lower_with(expression, callback, &rules, Some(face))
    }

    fn lower_with(
        &self,
        expression: &Atom,
        callback: Symbol,
        rules: &[Replacement],
        face: Option<&[(usize, u8)]>,
    ) -> Result<Atom, String> {
        let mut error = None;
        let result = expression.replace_map(|term, _, out| {
            let Some(function) = term.as_fun_view() else {
                return;
            };
            if function.get_symbol() != *super::STRENGTH {
                return;
            }
            match self.bundle(&term.replace_multiple(rules)) {
                Ok(mut bundle) => {
                    // Distinct native jets evaluate distinct physical faces,
                    // even if exact symbolic restriction made their radius
                    // expressions equal. Retain that actual request here;
                    // symbolic lowering above keeps the union when algebra
                    // has already merged equal restricted roots.
                    if let Some(face) = face {
                        bundle.0.retain(|request| request.face == face);
                        if bundle.0.is_empty() {
                            error.get_or_insert_with(|| {
                                "dynamic radius lacks its actual dual request face".to_owned()
                            });
                            return;
                        }
                    }
                    **out = callback.call_args(
                        [
                            function.get(0).to_owned(),
                            function.get(1).to_owned(),
                            bundle.atom(),
                        ]
                        .into_iter()
                        .chain(
                            (2..function.get_nargs()).map(|index| function.get(index).to_owned()),
                        ),
                    );
                }
                Err(reason) => {
                    error.get_or_insert(reason);
                }
            }
        });
        if let Some(reason) = error {
            Err(reason)
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contour::functions::dynamic::{RootProgram, requested, strength};

    #[test]
    fn equal_boundary_radii_keep_distinct_dual_faces_and_merged_symbolic_contexts() {
        let helper = RootProgram::build(1).unwrap();
        let x = symbol!("requested_face_control::x");
        let variable = Atom::var(x);
        let original = strength(
            &helper,
            &[Atom::one() + &variable * (Atom::one() - &variable)],
            &Atom::var(symbol!("requested_face_control::s")),
            &Atom::var(symbol!("requested_face_control::l")),
        )
        .unwrap();
        let mut lookup = Lookup::default();
        let namespace = "c".repeat(64);
        lookup
            .insert(
                &namespace,
                &original,
                &[x],
                &[vec![], vec![(0, 0)], vec![(0, 1)]],
            )
            .unwrap();
        let at_zero = original.replace(variable.clone()).with(Atom::Zero);
        let at_one = original.replace(variable.clone()).with(Atom::one());
        assert_eq!(at_zero, at_one);
        let symbolic = lookup.lower(&at_zero, requested::symbol()).unwrap();
        let symbolic = referenced_bundles([&symbolic]).unwrap();
        assert_eq!(symbolic.len(), 1);
        assert_eq!(symbolic.first().unwrap().0.len(), 2);
        for endpoint in [0, 1] {
            let lowered = lookup
                .lower_on_face(&original, requested::symbol(), &[x], &[(0, endpoint)])
                .unwrap();
            let requests = referenced_bundles([&lowered]).unwrap();
            assert_eq!(
                requests.into_iter().next().unwrap().0,
                vec![Request {
                    namespace: namespace.clone(),
                    face: vec![(0, endpoint)]
                }]
            );
            // The native jet differentiates the full mathematical arguments;
            // only static diagnostic metadata is restricted before its seeds.
            let function = lowered.as_fun_view().unwrap();
            assert_eq!(function.get(3), original.as_fun_view().unwrap().get(2));
            assert!(function.get(3).contains_symbol(x));
        }
    }
}
