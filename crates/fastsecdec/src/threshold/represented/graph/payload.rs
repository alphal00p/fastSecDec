//! Opt-in interpretation of native local fragments, never a graph reconstruction.
use super::super::check_bits;
use super::*;
use idenso::tensor::SymbolicTensor;
use idenso::{CookMode, CookSettings};
use spenso::structure::TensorStructure;
use spenso::structure::abstract_index::AbstractIndex;
use spenso::structure::partial::{PartialIndex, PartialStructure};
use symbolica::atom::{AtomView, UserData};
use symbolica::coefficient::{Coefficient, CoefficientView};
use symbolica::id::{AtomTreeIterator, MatchSettings};

/// Numerical input convention for graph payloads. This changes the meaning of
/// a rounded aggregate: each stored local fragment is exactified first, then
/// the native graph recomputes their product. The original aggregate is retained
/// as provenance, not asserted equal to that product or multiplied again.
///
/// Currently only Float coefficients outside function arguments are admitted.
/// This deliberately refuses legitimate scalar `f(0.1)` and shorthand arguments
/// as well as tensor-index/momentum labels. It is not full ggHH payload support.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphPayloadPolicy {
    PrimitiveFragmentsV1,
}

fn label_node(work: &mut Work<GraphLocation>, depth: usize) -> Result<()> {
    work.nodes = work
        .nodes
        .checked_add(1)
        .filter(|n| *n <= work.limits.nodes)
        .ok_or(Error::ResourceIncomplete("represented input nodes"))?;
    if depth > work.limits.depth {
        return Err(Error::ResourceIncomplete("represented input depth"));
    }
    Ok(())
}

/// Inspect native reversible labels without replacing their cooked identities.
/// Charge this preflight, including its ordinary syntax, to the shared budget.
/// The native iterator bounds deep/wide payloads without a recursive visitor;
/// only one byte-bounded symbol is uncooked at a time, never the whole tensor.
pub(super) fn validate_labels(
    work: &mut Work<GraphLocation>,
    original: &Atom,
    depth: usize,
) -> Result<()> {
    work.input_bytes = work
        .input_bytes
        .checked_add(original.as_view().get_byte_size())
        .filter(|n| *n <= work.limits.input_bytes)
        .ok_or(Error::ResourceIncomplete("represented input bytes"))?;
    let cooking = CookSettings::indices().with_mode(CookMode::ReversibleEncoding);
    let mut pending = vec![(
        AtomTreeIterator::new(original.as_view(), MatchSettings::default()),
        depth,
        false,
    )];
    let mut position = Vec::new();
    while let Some((tree, base_depth, in_label)) = pending.last_mut() {
        let Some(node) = tree.next_into(Some(&mut position)) else {
            pending.pop();
            continue;
        };
        let depth = base_depth
            .checked_add(position.len())
            .ok_or(Error::ResourceIncomplete("represented input depth"))?;
        label_node(work, depth)?;
        if *in_label && let AtomView::Num(number) = node {
            if matches!(number.get_coeff_view(), CoefficientView::Float(..)) {
                return Err(Error::Unsupported(
                    "graph payload Float in an encoded index label",
                ));
            }
            match number.get_coeff_view().to_owned() {
                Coefficient::Complex(q) => {
                    check_bits(&q.re, work.limits)?;
                    check_bits(&q.im, work.limits)?;
                }
                Coefficient::Indeterminate | Coefficient::Infinity(_) => {
                    return Err(Error::Unsupported("nonfinite represented coefficient"));
                }
                _ => {
                    return Err(Error::Unsupported(
                        "coefficient field outside exact complex rationals and represented Float",
                    ));
                }
            }
        }
        let AtomView::Var(variable) = node else {
            continue;
        };
        let UserData::Atom(payload) = variable.get_symbol().get_data() else {
            continue;
        };
        // Bound the borrowed original before the native codec can clone it.
        let input_bytes = work
            .input_bytes
            .checked_add(payload.as_view().get_byte_size())
            .filter(|n| *n <= work.limits.input_bytes)
            .ok_or(Error::ResourceIncomplete("represented input bytes"))?;
        let output_bytes = work
            .output_bytes
            .checked_add(payload.as_view().get_byte_size())
            .filter(|n| *n <= work.limits.output_bytes)
            .ok_or(Error::ResourceIncomplete("represented output bytes"))?;
        let depth = depth
            .checked_add(1)
            .filter(|d| *d <= work.limits.depth)
            .ok_or(Error::ResourceIncomplete("represented input depth"))?;
        if work.nodes == work.limits.nodes {
            return Err(Error::ResourceIncomplete("represented input nodes"));
        }
        if cooking.uncook(node).as_view() == node {
            continue;
        }
        work.input_bytes = input_bytes;
        work.output_bytes = output_bytes;
        // Follow nested native payloads too. Depth/nodes also bound cycles;
        // reading the original avoids normalization, exactification or renaming.
        pending.push((
            AtomTreeIterator::new(payload.as_view(), MatchSettings::default()),
            depth,
            true,
        ));
    }
    Ok(())
}

pub(super) fn validate_interface_labels(
    work: &mut Work<GraphLocation>,
    original: &SymbolicTensor<PartialStructure>,
) -> Result<()> {
    // A ranked zero can retain labels absent from its expression. Borrow the
    // canonical ports; neither logical-slot allocation nor reinference is needed.
    for index in original.structure().canonical().external_indices_iter() {
        label_node(work, 0)?;
        let PartialIndex::Explicit(mut index) = index else {
            continue;
        };
        let mut depth = 0;
        while let AbstractIndex::Scoped(scope) = index {
            validate_labels(work, &Atom::var(scope.scope()), depth)?;
            depth = depth
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("represented input depth"))?;
            label_node(work, depth)?;
            index = scope.index();
        }
        if let AbstractIndex::Symbol(symbol) = index {
            validate_labels(work, &Atom::var(symbol.into()), depth)?;
        }
    }
    Ok(())
}

fn atom(work: &mut Work<GraphLocation>, original: &Atom, location: GraphLocation) -> Result<Atom> {
    validate_labels(work, original, 0)?;
    work.atom_checked(original, location, true, || {
        // Native index/interface admission on the ORIGINAL expression. The
        // bounded role fence runs first; inference alone does not distinguish
        // a Float momentum label from a numerical scalar argument.
        SymbolicTensor::infer(original.clone())
            .map(|_| ())
            .map_err(|e| Error::Invalid(format!("original graph payload: {e}")))
    })
}

fn tensor(
    work: &mut Work<GraphLocation>,
    original: &SymbolicTensor<PartialStructure>,
    location: GraphLocation,
) -> Result<SymbolicTensor<PartialStructure>> {
    validate_labels(work, original.expression(), 0)?;
    validate_interface_labels(work, original)?;
    let expression = work.atom_checked(original.expression(), location, true, || {
        SymbolicTensor::validate_interface(original.expression(), original.structure())
            .map_err(|e| Error::Invalid(format!("original graph payload: {e}")))
    })?;
    // Only scalar coefficients are exactified. Keep the native interface,
    // including open ports carried by a zero expression, on every fragment.
    original
        .with_algebra_result(expression)
        .map_err(|e| Error::Invalid(format!("exact graph payload: {e}")))
}

pub(super) fn convert(
    original: &FeynmanDiagram,
    work: &mut Work<GraphLocation>,
    mut observe: impl FnMut(&Work<GraphLocation>) -> Result<()>,
) -> Result<FeynmanDiagram> {
    // Observe stored aggregate independently; this value never defines the
    // replacement density. All original precision remains in the witness.
    let _ = tensor(
        work,
        original.numerator(),
        GraphLocation::OriginalAggregate {},
    )?;
    let mut vertices = Vec::new();
    for (id, vertex) in original.vertices() {
        observe(work)?;
        vertices.push((
            id,
            tensor(
                work,
                &vertex.numerator,
                GraphLocation::Payload {
                    role: format!("vertex_numerator:{}", id.0),
                },
            )?,
        ));
    }
    let mut edges = Vec::new();
    for (id, _, edge) in original.edges() {
        observe(work)?;
        edges.push((
            id,
            tensor(
                work,
                &edge.numerator,
                GraphLocation::Payload {
                    role: format!("edge_numerator:{}", id.0),
                },
            )?,
        ));
    }
    let mut global = |expression, role: &str| {
        observe(work)?;
        tensor(
            work,
            expression,
            GraphLocation::Payload { role: role.into() },
        )
    };
    let prefactor = global(original.numerator_prefactor(), "numerator_prefactor")?;
    let projector = global(original.projector(), "projector")?;
    observe(work)?;
    let overall = atom(
        work,
        original.overall_factor(),
        GraphLocation::Payload {
            role: "overall_factor".into(),
        },
    )?;
    // Every source payload was bounded/admitted before native validation or
    // graph arithmetic. Original and exact diagrams remain distinct owners.
    original
        .validate()
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let exact = original
        .clone()
        .with_numerator_fragments(vertices, edges)
        .map_err(|e| Error::Invalid(e.to_string()))?
        .with_numerator_prefactor(prefactor)
        .with_projector(projector)
        .with_overall_factor(overall);
    exact
        .validate()
        .map_err(|e| Error::Invalid(e.to_string()))?;
    Ok(exact)
}
