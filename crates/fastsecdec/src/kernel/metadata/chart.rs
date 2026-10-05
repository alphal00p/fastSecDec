use super::{KernelError, atom, invalid, symbol_strings, symbols};
use crate::generation::{ChartRecord, DomainAssessment, coordinates_from_parts};
use fastsecdec_sectors::SectorMap;
use serde::{Deserialize, Serialize};
use symbolica::{atom::AtomCore, domains::integer::Integer};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortableChart {
    source_index: usize,
    representative: usize,
    representative_permutation: Vec<usize>,
    kernel_sector: Option<usize>,
    source_parameters: Vec<String>,
    target_parameters: Vec<String>,
    source_domain: super::domain::PortableDomain,
    images: Vec<String>,
    measure_jacobian: String,
    geometry: PortableGeometry,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableGeometry {
    fixed_parameter: Option<usize>,
    exponent_matrix: Vec<Vec<String>>,
    determinant: String,
    jacobian_powers: Vec<String>,
    factor_valuations: Vec<Vec<String>>,
}
fn strings(values: &[Integer]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}
fn integers(values: Vec<String>) -> Result<Vec<Integer>, KernelError> {
    values
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| invalid("invalid exact geometry integer"))
        })
        .collect()
}
impl PortableChart {
    pub(super) fn for_sector(chart: &ChartRecord) -> Self {
        let mut selected = Self::from_native(chart);
        selected.kernel_sector = Some(0);
        selected
    }

    pub(super) fn from_native(chart: &ChartRecord) -> Self {
        let coordinates = chart.coordinates();
        let geometry = chart.geometry();
        Self {
            source_index: chart.source_index(),
            representative: chart.representative(),
            representative_permutation: chart.representative_permutation().to_vec(),
            kernel_sector: chart.kernel_sector(),
            source_parameters: symbol_strings(coordinates.source_parameters()),
            target_parameters: symbol_strings(coordinates.target_parameters()),
            source_domain: super::domain::PortableDomain::from_native(coordinates.source_domain()),
            images: coordinates
                .images()
                .iter()
                .map(AtomCore::to_canonical_string)
                .collect(),
            measure_jacobian: coordinates.measure_jacobian().to_canonical_string(),
            geometry: PortableGeometry {
                fixed_parameter: geometry.fixed_parameter,
                exponent_matrix: geometry
                    .exponent_matrix
                    .iter()
                    .map(|row| strings(row))
                    .collect(),
                determinant: geometry.determinant.to_string(),
                jacobian_powers: strings(&geometry.jacobian_powers),
                factor_valuations: geometry
                    .factor_valuations
                    .iter()
                    .map(|row| strings(row))
                    .collect(),
            },
        }
    }
    pub(super) fn into_native(
        self,
        index: usize,
        assessment: &DomainAssessment,
    ) -> Result<ChartRecord, KernelError> {
        let source = symbols(self.source_parameters)?;
        let target = symbols(self.target_parameters)?;
        let domain = self.source_domain.into_native();
        if self.source_index != index
            || self.representative > index
            || source != assessment.parameters()
            || domain != assessment.domain()
        {
            return Err(invalid("inconsistent source chart metadata"));
        }
        if source.iter().any(|symbol| target.contains(symbol)) {
            return Err(invalid("source and target chart symbols overlap"));
        }
        let dimension = target.len();
        if self.representative_permutation.len() != dimension
            || self
                .representative_permutation
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                != (0..dimension).collect()
        {
            return Err(invalid("invalid symmetry coordinate permutation"));
        }
        let geometry = SectorMap {
            fixed_parameter: self.geometry.fixed_parameter,
            exponent_matrix: self
                .geometry
                .exponent_matrix
                .into_iter()
                .map(integers)
                .collect::<Result<_, _>>()?,
            determinant: self
                .geometry
                .determinant
                .parse()
                .map_err(|_| invalid("invalid determinant"))?,
            jacobian_powers: integers(self.geometry.jacobian_powers)?,
            factor_valuations: self
                .geometry
                .factor_valuations
                .into_iter()
                .map(integers)
                .collect::<Result<_, _>>()?,
        };
        geometry
            .validate(domain)
            .map_err(|error| invalid(&error.to_string()))?;
        if geometry.source_dimension() != source.len() || geometry.dimension() != dimension {
            return Err(invalid("coordinate map dimensions differ"));
        }
        let coordinates = coordinates_from_parts(&source, domain, &geometry, &target);
        let images = self
            .images
            .into_iter()
            .map(atom)
            .collect::<Result<Vec<_>, _>>()?;
        if images != coordinates.images()
            || atom(self.measure_jacobian)? != *coordinates.measure_jacobian()
        {
            return Err(invalid(
                "retained coordinate images or measure differ from exact geometry",
            ));
        }
        Ok(ChartRecord {
            source_index: index,
            representative: self.representative,
            representative_permutation: self.representative_permutation,
            kernel_sector: self.kernel_sector,
            coordinates,
            geometry,
        })
    }
}
