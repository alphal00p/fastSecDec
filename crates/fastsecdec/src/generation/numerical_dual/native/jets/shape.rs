use super::{KernelError, compilation};
use std::collections::BTreeMap;
use symbolica::utils::resources;

pub(super) struct Shape {
    pub(super) components: Vec<Vec<usize>>,
    index: BTreeMap<Vec<usize>, usize>,
}
impl Shape {
    pub(super) fn new(maxima: &[usize]) -> Result<Self, KernelError> {
        resources::checkpoint()?;
        // Each caller requests one maximal multiindex (including any source
        // valuation shift). Its complete box is precisely the minimal ancestor
        // closure required by native HyperDual; maxima are never pooled across
        // requests or sectors.
        let count = maxima
            .iter()
            .try_fold(1usize, |count, max| count.checked_mul(max.checked_add(1)?))
            .ok_or_else(|| compilation("numerical-dual jet shape overflow"))?;
        maxima.iter().try_fold(0usize, |sum, maximum| {
            sum.checked_add(*maximum)
                .ok_or_else(|| compilation("numerical-dual total derivative order overflow"))
        })?;
        let vectors_bytes = count
            .checked_mul(std::mem::size_of::<Vec<usize>>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(|| compilation("numerical-dual shape vector allocation size overflow"))?;
        maxima
            .len()
            .checked_mul(std::mem::size_of::<usize>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(|| compilation("numerical-dual shape row allocation size overflow"))?;
        let component_bytes = Some(vectors_bytes)
            .and_then(|vectors| {
                count
                    .checked_mul(maxima.len())
                    .and_then(|entries| entries.checked_mul(std::mem::size_of::<usize>()))
                    .and_then(|entries| vectors.checked_add(entries))
            })
            .ok_or_else(|| compilation("numerical-dual shape allocation size overflow"))?;
        resources::check_allocation(component_bytes as u64)?;
        let mut components = vec![Vec::new()];
        for max in maxima {
            resources::checkpoint()?;
            let mut next = Vec::new();
            let next_count = components
                .len()
                .checked_mul(max + 1)
                .ok_or_else(|| compilation("numerical-dual jet shape overflow"))?;
            next.try_reserve_exact(next_count).map_err(|error| {
                compilation(format!("numerical-dual shape allocation: {error}"))
            })?;
            for prefix in components {
                for n in 0..=*max {
                    resources::checkpoint()?;
                    let mut p = prefix.clone();
                    p.push(n);
                    next.push(p);
                }
            }
            components = next;
        }
        components.sort_by(|a, b| {
            a.iter()
                .sum::<usize>()
                .cmp(&b.iter().sum())
                .then_with(|| a.cmp(b))
        });
        let index = components
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, p)| (p, i))
            .collect();
        resources::checkpoint()?;
        Ok(Self { components, index })
    }
    pub(super) fn position(&self, powers: &[usize]) -> Result<usize, KernelError> {
        self.index
            .get(powers)
            .copied()
            .ok_or_else(|| compilation("native jet coefficient absent from requested shape"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_construction_checks_planned_bytes_against_the_shared_budget() {
        use symbolica::utils::resources::{ResourceBudget, ResourceLimitError, ResourceLimits};
        let budget = ResourceBudget::with_memory_monitor(
            ResourceLimits {
                max_process_rss_bytes: Some(1),
                ..Default::default()
            },
            || Ok(0),
        );
        assert!(matches!(
            budget.run(|| Shape::new(&[2, 2])),
            Err(KernelError::Resource(ResourceLimitError::Memory {
                additional_bytes,
                ..
            })) if additional_bytes > 1
        ));
        assert_eq!(Shape::new(&[2, 2]).unwrap().components.len(), 9);
    }

    #[test]
    fn individual_request_shapes_are_minimal_native_ancestor_closures() {
        for maxima in [&[0, 0, 0][..], &[2, 0, 1], &[1, 3, 2]] {
            let shape = Shape::new(maxima).unwrap();
            assert_eq!(
                shape.components.len(),
                maxima.iter().map(|value| value + 1).product::<usize>()
            );
            assert!(shape.position(maxima).is_ok());
            for component in &shape.components {
                assert!(
                    component
                        .iter()
                        .zip(maxima)
                        .all(|(value, max)| value <= max)
                );
                for axis in 0..component.len() {
                    if component[axis] != 0 {
                        let mut parent = component.clone();
                        parent[axis] -= 1;
                        assert!(shape.position(&parent).is_ok());
                    }
                }
            }
            // Native construction independently validates the ancestor contract.
            let native = symbolica::domains::dual::HyperDual::<f64>::new(shape.components.clone());
            assert_eq!(native.values.len(), shape.components.len());
        }
        assert!(Shape::new(&[usize::MAX]).is_err());
        // Shape construction has no arbitrary component-count ceiling. Native
        // multiplication is a separate operation and is not invoked here.
        assert_eq!(Shape::new(&[4096]).unwrap().components.len(), 4097);
    }
}
