//! Mathematical request identity, separate from native-program byte digests and
//! mutable record-local chart indices. Native canonical printing owns atom identity.
use crate::{
    contour::dynamic::DynamicEnvelope,
    generation::identity::{CanonicalAtom, CanonicalSymbol},
    kernel::ProgramRecipe,
};
use serde::Serialize;
use symbolica::atom::{Atom, Symbol};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub(crate) struct FactorIdentity {
    pub causal: String,
    pub positive: Vec<String>,
}

fn digest(domain: &[u8], value: &impl Serialize) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    serde_json::to_writer(&mut hasher, value).map_err(|error| error.to_string())?;
    Ok(hasher.finalize().to_hex().to_string())
}

impl FactorIdentity {
    pub(crate) fn new(causal: &Atom, positive: &[Atom]) -> Result<Self, String> {
        let factor = |atom| digest(b"fastsecdec-contour-factor-v1\0", &CanonicalAtom(atom));
        Ok(Self {
            causal: factor(causal)?,
            positive: positive.iter().map(factor).collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn from_envelope(envelope: &DynamicEnvelope) -> Result<Self, String> {
        Self::new(
            envelope.causal_polynomial(),
            &envelope
                .positive_factors()
                .iter()
                .map(|factor| factor.polynomial().clone())
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate(&self, positive_count: usize) -> Result<(), String> {
        if self.positive.len() != positive_count
            || std::iter::once(&self.causal)
                .chain(&self.positive)
                .any(|digest| !valid_digest(digest))
        {
            return Err("invalid causal/positive-factor mathematical identity".into());
        }
        Ok(())
    }
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Equal full radius recipes may share this key. Chart and sector projections
/// remain separate so reporting never treats equal recipes as equal integrals.
pub(crate) fn namespace(
    recipe: ProgramRecipe,
    envelope: &DynamicEnvelope,
    factors: &FactorIdentity,
) -> Result<String, String> {
    namespace_parts(
        recipe,
        envelope.parameters(),
        factors,
        envelope
            .causal_terms()
            .iter()
            .map(|term| term.order())
            .collect(),
        envelope
            .positive_factors()
            .iter()
            .map(|factor| factor.terms().iter().map(|term| term.order()).collect())
            .collect(),
        &envelope
            .positive_factors()
            .iter()
            .map(|factor| factor.certified_lower_bound().clone())
            .collect::<Vec<_>>(),
        envelope.maximum_even_order(),
        envelope.regularity(),
    )
}

pub(crate) fn namespace_for_chart(
    recipe: ProgramRecipe,
    parameters: &[Symbol],
    factors: &FactorIdentity,
    chart: &super::super::DynamicChartRecipe,
) -> Result<String, String> {
    namespace_parts(
        recipe,
        parameters,
        factors,
        chart.causal_orders.clone(),
        chart.positive_orders.clone(),
        &chart
            .positive_proofs
            .iter()
            .map(|proof| match proof {
                super::super::PositiveFactorProof::NonnegativeCoefficients { lower_bound } => {
                    Atom::num(lower_bound.clone())
                }
            })
            .collect::<Vec<_>>(),
        chart.maximum_even_order,
        &Atom::num(chart.regularity.clone()),
    )
}

#[allow(clippy::too_many_arguments)]
fn namespace_parts(
    recipe: ProgramRecipe,
    parameters: &[Symbol],
    factors: &FactorIdentity,
    causal_orders: Vec<u32>,
    positive_orders: Vec<Vec<u32>>,
    lower_bounds: &[Atom],
    maximum_even_order: u32,
    regularity: &Atom,
) -> Result<String, String> {
    #[derive(Serialize)]
    struct Request<'a> {
        recipe: ProgramRecipe,
        coordinates: Vec<CanonicalSymbol>,
        factors: &'a FactorIdentity,
        causal_orders: Vec<u32>,
        positive_orders: Vec<Vec<u32>>,
        positive_lower_bounds: Vec<CanonicalAtom<'a>>,
        maximum_even_order: u32,
        regularity: CanonicalAtom<'a>,
    }
    if !recipe.is_dynamic() {
        return Err("dynamic request namespace requires a dynamic recipe".into());
    }
    factors.validate(positive_orders.len())?;
    digest(
        b"fastsecdec-contour-request-v1\0",
        &Request {
            recipe,
            coordinates: parameters.iter().copied().map(CanonicalSymbol).collect(),
            factors,
            causal_orders,
            positive_orders,
            positive_lower_bounds: lower_bounds.iter().map(CanonicalAtom).collect(),
            maximum_even_order,
            regularity: CanonicalAtom(regularity),
        },
    )
}
