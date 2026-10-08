//! Zero-padding/scattering of already compiled local Laurent vectors. This is
//! numerical layout adaptation, never native evaluator composition or optimizer
//! invocation. The original exact program and precision rescue remain local.
use super::{Backend, KernelError, PrecisionReport, SectorKernel};
use crate::status::CoefficientComponent;

#[derive(Clone)]
pub(super) struct OutputProjection {
    pub indices: Vec<usize>,
    pub output_count: usize,
    pub complex: bool,
    pub local_orders: Vec<i32>,
    pub local_components: Vec<CoefficientComponent>,
    values: Vec<f64>,
    primary: Vec<f64>,
}
impl OutputProjection {
    pub fn new(
        indices: Vec<usize>,
        output_count: usize,
        complex: bool,
        local_flat_orders: &[i32],
        local_components: Vec<CoefficientComponent>,
    ) -> Result<Self, KernelError> {
        if indices.len() != local_flat_orders.len()
            || indices.len() != local_components.len()
            || indices.iter().any(|index| *index >= output_count)
            || indices.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(KernelError::Artifact(
                "invalid local output projection".into(),
            ));
        }
        let mut local_orders = local_flat_orders.to_vec();
        local_orders.dedup();
        Ok(Self {
            values: vec![0.0; indices.len()],
            primary: vec![0.0; indices.len()],
            indices,
            output_count,
            complex,
            local_orders,
            local_components,
        })
    }
    pub fn local_flat_orders(&self) -> Vec<i32> {
        let complex = self.local_components.contains(&CoefficientComponent::Imag);
        self.local_orders
            .iter()
            .flat_map(|order| std::iter::repeat_n(*order, if complex { 2 } else { 1 }))
            .collect()
    }
}

impl SectorKernel {
    pub(super) fn output_width(&self) -> usize {
        if self
            .projection
            .as_ref()
            .map_or(matches!(self.backend, Backend::Complex(_)), |p| p.complex)
        {
            2
        } else {
            1
        }
    }
    pub(super) fn native_output_count(&self) -> usize {
        self.exact_zero.len()
            * if matches!(self.backend, Backend::Complex(_)) {
                2
            } else {
                1
            }
    }
    pub(super) fn project_output(
        &mut self,
        output: &mut [f64],
        primary: Option<&[f64]>,
        evaluate: impl FnOnce(
            &mut Self,
            &mut [f64],
            Option<&[f64]>,
        ) -> Result<PrecisionReport, KernelError>,
    ) -> Result<PrecisionReport, KernelError> {
        let mut projection = self.projection.take().expect("projection dispatch");
        let result = if output.len() != projection.output_count {
            Err(KernelError::OutputCount {
                expected: projection.output_count,
                actual: output.len(),
            })
        } else {
            if let Some(primary) = primary {
                if primary.len() != projection.output_count {
                    self.projection = Some(projection);
                    return Err(KernelError::OutputCount {
                        expected: self.output_count(),
                        actual: primary.len(),
                    });
                }
                for (value, index) in projection.primary.iter_mut().zip(&projection.indices) {
                    *value = primary[*index];
                }
            }
            let result = evaluate(
                self,
                &mut projection.values,
                primary.map(|_| projection.primary.as_slice()),
            );
            if result.is_ok() {
                output.fill(0.0);
                for (value, index) in projection.values.iter().zip(&projection.indices) {
                    output[*index] = *value;
                }
            }
            result
        };
        self.projection = Some(projection);
        result
    }
}
