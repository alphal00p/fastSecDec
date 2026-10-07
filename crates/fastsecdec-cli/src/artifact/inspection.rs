//! Small producer-recorded display index. Native expressions/evaluators remain
//! exclusively in the binary; this optional observation is not a trust boundary.
use super::Artifact;
use crate::{CliResult, math_display, terminal_policy::ColorPolicy};
use fastsecdec::{
    Atom, AtomCore,
    kernel::{EvaluatorStatistics, KernelSet},
    status::CoefficientComponent,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct KernelSummary {
    pub sectors: usize,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    #[serde(default)]
    pub dimensions: Option<Vec<usize>>,
    #[serde(default)]
    pub evaluator_statistics: Option<Vec<EvaluatorStatistics>>,
    #[serde(default)]
    pub runtime_parameters: Option<Vec<String>>,
}
impl KernelSummary {
    pub fn from_kernels(kernels: &KernelSet) -> Self {
        Self {
            sectors: kernels.sectors().len(),
            orders: kernels.orders().to_vec(),
            components: kernels.components().to_vec(),
            dimensions: Some(kernels.sectors().iter().map(|s| s.dimension()).collect()),
            evaluator_statistics: Some(
                kernels
                    .sectors()
                    .iter()
                    .map(|s| s.statistics().clone())
                    .collect(),
            ),
            runtime_parameters: Some(
                kernels
                    .runtime_parameters()
                    .iter()
                    .map(|s| s.get_name().to_owned())
                    .collect(),
            ),
        }
    }
}
impl Artifact {
    pub fn kernel_summary(&self) -> CliResult<KernelSummary> {
        let summary: KernelSummary = serde_json::from_str(self.kernel.get())?;
        if summary.orders.len() != summary.components.len()
            || summary
                .dimensions
                .as_ref()
                .is_some_and(|v| v.len() != summary.sectors)
            || summary
                .evaluator_statistics
                .as_ref()
                .is_some_and(|v| v.len() != summary.sectors)
        {
            return Err("artifact kernel summary has inconsistent layout counts".into());
        }
        Ok(summary)
    }
    pub fn inspection_index(&self) -> Option<&InspectionIndex> {
        self.inspection.as_ref().filter(|index| index.version == 1)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InspectionIndex {
    pub version: u32,
    pub retained_metadata_available: bool,
    pub total_charts: usize,
    pub total_representatives: usize,
    pub omitted_charts: usize,
    pub charts: Vec<ChartPreview>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartPreview {
    pub source_index: usize,
    pub representative: usize,
    pub kernel_sector: Option<usize>,
    /// Only representative charts carry bounded native display previews.
    pub preview: Option<RepresentativePreview>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RepresentativePreview {
    pub terms: Option<usize>,
    /// First three retained terms, without inferring cancellation or poles.
    pub leading_monomials: Vec<Option<String>>,
    pub omitted_terms: usize,
    /// At most sixteen source-coordinate images; never expanded regular bodies.
    pub coordinate_images: Vec<[Option<String>; 2]>,
    pub omitted_coordinate_images: usize,
    pub positive_measure: Option<String>,
    pub projective_fixed_parameter: Option<usize>,
}
impl InspectionIndex {
    pub fn from_kernels(kernels: &KernelSet) -> Self {
        let metadata = kernels.generation_metadata();
        let mut bytes_left = 1024 * 1024 - 4096;
        let total_charts = metadata.map_or(0, |m| m.charts().len());
        let total_representatives = metadata.map_or(0, |m| {
            m.charts()
                .iter()
                .map(|c| c.representative())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        });
        let charts: Vec<_> = metadata
            .into_iter()
            .flat_map(|metadata| metadata.charts())
            .map(|chart| {
                let map = chart.coordinates();
                ChartPreview {
                    source_index: chart.source_index(),
                    representative: chart.representative(),
                    kernel_sector: chart.kernel_sector(),
                    preview: (chart.source_index() == chart.representative()).then(|| {
                        let context = map
                            .source_parameters()
                            .iter()
                            .chain(map.target_parameters())
                            .map(|p| Atom::var(*p))
                            .collect::<Vec<_>>();
                        let record = chart.pre_subtraction();
                        RepresentativePreview {
                            terms: record.map(|r| r.terms().len()),
                            leading_monomials: record
                                .into_iter()
                                .flat_map(|r| r.terms())
                                .take(3)
                                .map(|term| {
                                    let monomial = map
                                        .target_parameters()
                                        .iter()
                                        .zip(term.powers())
                                        .fold(Atom::num(1), |value, (parameter, power)| {
                                            value * Atom::var(*parameter).pow(power.exponent())
                                        });
                                    // Include the regulator in native collision detection as well.
                                    let mut context = context.clone();
                                    context.push(monomial.clone());
                                    display(&monomial, &context)
                                })
                                .collect(),
                            omitted_terms: record.map_or(0, |r| r.terms().len().saturating_sub(3)),
                            coordinate_images: map
                                .source_parameters()
                                .iter()
                                .zip(map.images())
                                .take(16)
                                .map(|(parameter, image)| {
                                    [
                                        display(&Atom::var(*parameter), &context),
                                        display(image, &context),
                                    ]
                                })
                                .collect(),
                            omitted_coordinate_images: map.images().len().saturating_sub(16),
                            positive_measure: display(map.measure_jacobian(), &context),
                            projective_fixed_parameter: map.projective_fixed_parameter(),
                        }
                    }),
                }
            })
            .take_while(|chart| {
                // Bound the entire human preview, including pretty-print overhead.
                let size = serde_json::to_vec_pretty(chart).map_or(usize::MAX, |v| {
                    v.len()
                        .saturating_add(16 * v.iter().filter(|&&b| b == b'\n').count())
                        .saturating_add(128)
                });
                if size > bytes_left {
                    false
                } else {
                    bytes_left -= size;
                    true
                }
            })
            .collect();
        Self {
            version: 1,
            retained_metadata_available: metadata.is_some(),
            total_charts,
            total_representatives,
            omitted_charts: total_charts - charts.len(),
            charts,
        }
    }
}

pub(super) fn deserialize_index<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<InspectionIndex>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        Some(value) if value.get("version").and_then(serde_json::Value::as_u64) == Some(1) => {
            serde_json::from_value(value)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        _ => Ok(None),
    }
}
fn display(atom: &Atom, context: &[Atom]) -> Option<String> {
    if atom.as_view().get_byte_size() > 4096 {
        return None;
    }
    let value = math_display::in_context(atom, context, ColorPolicy::for_stream(true, false), 4096);
    (value.len() <= 4096).then_some(value)
}
