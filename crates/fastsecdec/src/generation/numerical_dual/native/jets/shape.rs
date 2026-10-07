use super::{KernelError, compilation};
use std::collections::BTreeMap;

pub(super) struct Shape {
    pub(super) components: Vec<Vec<usize>>,
    index: BTreeMap<Vec<usize>, usize>,
}
impl Shape {
    pub(super) fn new(maxima: &[usize]) -> Result<Self, KernelError> {
        // Each caller requests one maximal multiindex (including any source
        // valuation shift). Its complete box is precisely the minimal ancestor
        // closure required by native HyperDual; maxima are never pooled across
        // requests or sectors.
        let count = maxima
            .iter()
            .try_fold(1usize, |count, max| count.checked_mul(max.checked_add(1)?))
            .ok_or_else(|| compilation("numerical-dual jet shape overflow"))?;
        // Native dense HyperDual multiplication owns a quadratic table. Keep
        // excessive requests explicit instead of allowing an allocation abort.
        if count > 4096 {
            return Err(compilation(format!(
                "numerical-dual jet requires {count} components (limit 4096); use symbolic generation or a lower expansion order"
            )));
        }
        let mut components = vec![Vec::new()];
        for max in maxima {
            components = components
                .into_iter()
                .flat_map(|prefix| {
                    (0..=*max).map(move |n| {
                        let mut p = prefix.clone();
                        p.push(n);
                        p
                    })
                })
                .collect();
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
        assert!(Shape::new(&[4096]).is_err());
    }
}
