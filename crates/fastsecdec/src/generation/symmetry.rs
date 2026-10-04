//! Verified integration-variable permutations of complete factored densities.
//!
//! Graphica owns graph canonization. This module only encodes Symbolica's
//! existing expression tree, with shared vertices for integration variables.
//! No polynomial expansion, tensor-index reinterpretation, or independent
//! algebraic canonicalizer is used. A graph match is only a candidate: native
//! simultaneous substitution must reproduce the representative density exactly.

use std::collections::{HashMap, HashSet};

use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    graph::{CanonicalForm, Graph},
    id::{Pattern, Replacement},
};

use super::GenerationError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Vertex {
    Root,
    Parameter,
    Constant(Atom),
    Add,
    Multiply,
    Power,
    Function(Symbol),
}

type DensityGraph = Graph<Vertex, usize>;

struct Representative {
    source_index: usize,
    parameters: Vec<Symbol>,
    canonical_parameters: Vec<usize>,
    density: Atom,
}

/// A verified source-coordinate permutation into an earlier representative.
pub(super) struct SymmetryMatch {
    pub representative: usize,
    /// Source coordinate i becomes representative coordinate permutation[i].
    pub permutation: Vec<usize>,
}

/// Representatives retain full prefactors, regulator exponents and numerators.
/// Structural differences can miss an equivalence, but can never justify an
/// unverified merge. Registration order fixes deterministic representatives.
#[derive(Default)]
pub(super) struct SymmetryRegistry {
    classes: HashMap<DensityGraph, Vec<Representative>>,
}

impl SymmetryRegistry {
    pub fn register(
        &mut self,
        source_index: usize,
        parameters: &[Symbol],
        density: &Atom,
    ) -> Result<SymmetryMatch, GenerationError> {
        let canonical = incidence_graph(density, parameters)?;
        let canonical_parameters = canonical.vertex_map[..parameters.len()].to_vec();
        let candidates = self.classes.entry(canonical.graph).or_default();
        for representative in candidates.iter() {
            let destination = representative
                .canonical_parameters
                .iter()
                .enumerate()
                .map(|(axis, vertex)| (*vertex, axis))
                .collect::<HashMap<_, _>>();
            let permutation = canonical_parameters
                .iter()
                .map(|vertex| destination.get(vertex).copied())
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    GenerationError::Invariant(
                        "canonical density map did not preserve parameter vertices".into(),
                    )
                })?;
            if verified_permutation(
                density,
                parameters,
                &representative.density,
                &representative.parameters,
                &permutation,
            ) {
                return Ok(SymmetryMatch {
                    representative: representative.source_index,
                    permutation,
                });
            }
        }
        candidates.push(Representative {
            source_index,
            parameters: parameters.to_vec(),
            canonical_parameters,
            density: density.clone(),
        });
        Ok(SymmetryMatch {
            representative: source_index,
            permutation: (0..parameters.len()).collect(),
        })
    }
}

fn verified_permutation(
    source: &Atom,
    source_parameters: &[Symbol],
    target: &Atom,
    target_parameters: &[Symbol],
    permutation: &[usize],
) -> bool {
    if source_parameters.len() != target_parameters.len()
        || permutation.len() != source_parameters.len()
        || permutation.iter().copied().collect::<HashSet<_>>().len() != permutation.len()
        || permutation
            .iter()
            .any(|index| *index >= target_parameters.len())
    {
        return false;
    }
    source.replace_multiple(
        source_parameters
            .iter()
            .zip(permutation)
            .map(|(parameter, axis)| {
                // Literal patterns also preserve valid symbols whose names end in '_'.
                Replacement::new(
                    Pattern::Literal(Atom::var(*parameter)),
                    Pattern::Literal(Atom::var(target_parameters[*axis])),
                )
            }),
    ) == *target
}

struct IncidenceBuilder<'a> {
    graph: DensityGraph,
    parameters: HashMap<Symbol, usize>,
    atoms: Vec<Atom>,
    memo: HashMap<AtomView<'a>, usize>,
}

impl<'a> IncidenceBuilder<'a> {
    fn edge(&mut self, parent: usize, child: usize, slot: usize) -> Result<(), GenerationError> {
        self.graph
            .add_edge(parent, child, true, slot)
            .map_err(|error| GenerationError::Invariant(error.into()))?;
        Ok(())
    }

    fn expression(&mut self, expression: AtomView<'a>) -> Result<usize, GenerationError> {
        if let Some(node) = self.memo.get(&expression) {
            return Ok(*node);
        }
        if let AtomView::Var(variable) = expression
            && let Some(node) = self.parameters.get(&variable.get_symbol())
        {
            return Ok(*node);
        }
        let node = if !self
            .atoms
            .iter()
            .any(|parameter| expression.contains(parameter.as_view()))
        {
            // Regulator functions and external scalar coefficients remain exact
            // opaque Atom labels, rather than being evaluated or reconstructed.
            self.graph.add_node(Vertex::Constant(expression.to_owned()))
        } else {
            let (label, children) = match expression {
                AtomView::Add(sum) => (
                    Vertex::Add,
                    sum.iter().map(|child| (0, child)).collect::<Vec<_>>(),
                ),
                AtomView::Mul(product) => (
                    Vertex::Multiply,
                    product.iter().map(|child| (0, child)).collect(),
                ),
                AtomView::Pow(power) => {
                    let (base, exponent) = power.get_base_exp();
                    (Vertex::Power, vec![(0, base), (1, exponent)])
                }
                AtomView::Fun(function) => (
                    Vertex::Function(function.get_symbol()),
                    function
                        .iter()
                        .enumerate()
                        .map(|(slot, child)| {
                            (
                                if function.get_symbol().is_symmetric() {
                                    0
                                } else {
                                    slot
                                },
                                child,
                            )
                        })
                        .collect(),
                ),
                AtomView::Num(_) | AtomView::Var(_) => {
                    return Err(GenerationError::Invariant(
                        "unclassified density leaf".into(),
                    ));
                }
            };
            let parent = self.graph.add_node(label);
            for (slot, child) in children {
                let child = self.expression(child)?;
                self.edge(parent, child, slot)?;
            }
            parent
        };
        self.memo.insert(expression, node);
        Ok(node)
    }
}

fn incidence_graph(
    density: &Atom,
    parameters: &[Symbol],
) -> Result<CanonicalForm<Vertex, usize>, GenerationError> {
    if parameters.iter().collect::<HashSet<_>>().len() != parameters.len() {
        return Err(GenerationError::Invariant(
            "duplicate density parameters".into(),
        ));
    }
    let mut builder = IncidenceBuilder {
        graph: Graph::new(),
        parameters: HashMap::new(),
        atoms: parameters
            .iter()
            .map(|parameter| Atom::var(*parameter))
            .collect(),
        memo: HashMap::new(),
    };
    // Insertion order lets the native canonical vertex map recover coordinates.
    // Unused coordinates are retained because full integration support matters.
    for parameter in parameters {
        let node = builder.graph.add_node(Vertex::Parameter);
        builder.parameters.insert(*parameter, node);
    }
    let root = builder.graph.add_node(Vertex::Root);
    let expression = builder.expression(density.as_view())?;
    builder.edge(root, expression, 0)?;
    Ok(builder.graph.canonize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{parse, symbol};

    #[test]
    fn native_tensor_canonicalizer_is_not_a_scalar_parameter_permutation_api() {
        assert!(
            parse!("x*(x+y)^(-2+eps)")
                .canonize_tensors([(parse!("x"), 0), (parse!("y"), 0)])
                .is_err()
        );
    }

    #[test]
    fn full_factored_density_permutation_retains_regulator_and_prefactor() {
        let mut registry = SymmetryRegistry::default();
        let parameters = [symbol!("x"), symbol!("y")];
        let source = parse!("gamma(eps)*x^(-1+eps)*y^2*(x+2*y)^(-3-eps)*(1+x*y)^5");
        let target = parse!("gamma(eps)*y^(-1+eps)*x^2*(y+2*x)^(-3-eps)*(1+x*y)^5");
        assert_eq!(
            registry
                .register(7, &parameters, &source)
                .unwrap()
                .representative,
            7
        );
        let matched = registry.register(9, &parameters, &target).unwrap();
        assert_eq!(matched.representative, 7);
        assert_eq!(matched.permutation, [1, 0]);
        assert!(verified_permutation(
            &target,
            &parameters,
            &source,
            &parameters,
            &matched.permutation
        ));
    }

    #[test]
    fn identical_uf_never_hides_distinct_numerators_exponents_or_weights() {
        let parameters = [symbol!("x"), symbol!("y")];
        let mut registry = SymmetryRegistry::default();
        for (index, expression) in [
            parse!("x*(1+x+2*y)^(-2+eps)"),
            parse!("y*(1+x+2*y)^(-2+eps)"),
            parse!("x*(1+x+2*y)^(-2+2*eps)"),
            parse!("2*x*(1+x+2*y)^(-2+eps)"),
            parse!("x*(1+x+3*y)^(-2+eps)"),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                registry
                    .register(index, &parameters, expression)
                    .unwrap()
                    .representative,
                index
            );
        }
    }

    #[test]
    fn sums_require_one_consistent_permutation_for_every_term() {
        let parameters = [symbol!("x"), symbol!("y")];
        let mut registry = SymmetryRegistry::default();
        let source = parse!("x/(1+x+2*y)+3*y^2/(2+x+y)");
        let target = parse!("y/(1+y+2*x)+3*x^2/(2+y+x)");
        let inconsistent = parse!("y/(1+y+2*x)+3*y^2/(2+y+x)");
        registry.register(0, &parameters, &source).unwrap();
        assert_eq!(
            registry
                .register(1, &parameters, &target)
                .unwrap()
                .representative,
            0
        );
        assert_eq!(
            registry
                .register(2, &parameters, &inconsistent)
                .unwrap()
                .representative,
            2
        );
    }

    #[test]
    fn unused_coordinates_and_zero_dimensional_support_are_preserved() {
        let mut registry = SymmetryRegistry::default();
        let one = Atom::one();
        assert_eq!(registry.register(0, &[], &one).unwrap().representative, 0);
        assert_eq!(
            registry
                .register(1, &[symbol!("x")], &one)
                .unwrap()
                .representative,
            1
        );
        assert_eq!(
            registry
                .register(2, &[symbol!("x"), symbol!("y")], &one)
                .unwrap()
                .representative,
            2
        );
        assert_eq!(
            registry
                .register(3, &[symbol!("z")], &one)
                .unwrap()
                .representative,
            1
        );
    }

    #[test]
    fn native_verification_is_literal_and_rejects_invalid_or_wrong_permutations() {
        let source_parameters = [symbol!("density_x_"), symbol!("density_y_")];
        let target_parameters = [symbol!("density_u_"), symbol!("density_v_")];
        let source = parse!("density_x_+2*density_y_");
        let target = parse!("density_v_+2*density_u_");
        let mut registry = SymmetryRegistry::default();
        registry.register(0, &target_parameters, &target).unwrap();
        let result = registry.register(1, &source_parameters, &source).unwrap();
        assert_eq!(result.representative, 0);
        assert_eq!(result.permutation, [1, 0]);
        assert!(!verified_permutation(
            &source,
            &source_parameters,
            &target,
            &target_parameters,
            &[0, 1]
        ));
        assert!(!verified_permutation(
            &source,
            &source_parameters,
            &target,
            &target_parameters,
            &[1, 1]
        ));
    }

    #[test]
    fn factored_large_power_stays_compact_and_exact() {
        let parameters = [symbol!("x"), symbol!("y")];
        let expression = parse!("(1+x+2*y)^10000/(1+x*y)^(3+eps)");
        let graph = incidence_graph(&expression, &parameters).unwrap();
        assert!(graph.graph.nodes().len() < 25);
        let mut registry = SymmetryRegistry::default();
        registry.register(0, &parameters, &expression).unwrap();
        assert_eq!(
            registry
                .register(1, &parameters, &parse!("(1+y+2*x)^10000/(1+x*y)^(3+eps)"))
                .unwrap()
                .representative,
            0
        );
    }
}
