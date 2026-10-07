use super::{KernelError, compilation};
use std::collections::BTreeMap;

pub(super) struct Shape {
    pub(super) components: Vec<Vec<usize>>,
    index: BTreeMap<Vec<usize>, usize>,
}
impl Shape {
    pub(super) fn new(maxima: &[usize]) -> Result<Self, KernelError> {
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
