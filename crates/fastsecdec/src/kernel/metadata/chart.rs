use super::{KernelError, StoredAtom, atom, invalid, symbol_strings, symbols};
use crate::generation::{ChartRecord, CoordinateMap, DomainAssessment, coordinates_from_parts};
use fastsecdec_sectors::{ParametricDomain, SectorMap};
use serde::{Deserialize, Serialize};
use symbolica::domains::integer::Integer;
use symbolica::state::StateMap;
mod contour;
mod pre_subtraction;

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
pub(super) struct PortableChart {
    source_index: usize,
    representative: usize,
    representative_permutation: Vec<usize>,
    kernel_sector: Option<usize>,
    source_parameters: Vec<String>,
    target_parameters: Vec<String>,
    source_domain: super::domain::PortableDomain,
    images: Vec<StoredAtom>,
    measure_jacobian: StoredAtom,
    geometry: PortableGeometry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pre_subtraction: Option<pre_subtraction::PortablePreSubtraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    contour: Option<contour::PortableContour>,
}

/// Exact pre-contour wire layout used by binserde versions five through eight.
#[derive(bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
pub(super) struct LegacyChart {
    source_index: usize,
    representative: usize,
    representative_permutation: Vec<usize>,
    kernel_sector: Option<usize>,
    source_parameters: Vec<String>,
    target_parameters: Vec<String>,
    source_domain: super::domain::PortableDomain,
    images: Vec<StoredAtom>,
    measure_jacobian: StoredAtom,
    geometry: PortableGeometry,
    pre_subtraction: Option<pre_subtraction::PortablePreSubtraction>,
}
impl From<LegacyChart> for PortableChart {
    fn from(value: LegacyChart) -> Self {
        Self {
            source_index: value.source_index,
            representative: value.representative,
            representative_permutation: value.representative_permutation,
            kernel_sector: value.kernel_sector,
            source_parameters: value.source_parameters,
            target_parameters: value.target_parameters,
            source_domain: value.source_domain,
            images: value.images,
            measure_jacobian: value.measure_jacobian,
            geometry: value.geometry,
            pre_subtraction: value.pre_subtraction,
            contour: None,
        }
    }
}
#[cfg(test)]
impl From<PortableChart> for LegacyChart {
    fn from(value: PortableChart) -> Self {
        assert!(value.contour.is_none());
        Self {
            source_index: value.source_index,
            representative: value.representative,
            representative_permutation: value.representative_permutation,
            kernel_sector: value.kernel_sector,
            source_parameters: value.source_parameters,
            target_parameters: value.target_parameters,
            source_domain: value.source_domain,
            images: value.images,
            measure_jacobian: value.measure_jacobian,
            geometry: value.geometry,
            pre_subtraction: value.pre_subtraction,
        }
    }
}
#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
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
    pub(super) fn take_contour_definitions(&mut self) -> crate::contour::ContourDefinitions {
        self.contour
            .as_mut()
            .map_or_else(Default::default, contour::PortableContour::take_definitions)
    }

    pub(super) fn attach_contour_definitions(
        &mut self,
        definitions: crate::contour::ContourDefinitions,
    ) -> Result<(), KernelError> {
        self.contour
            .as_mut()
            .ok_or_else(|| invalid("compact definition sidecar has no contour chart"))?
            .attach_definitions(definitions)
    }
    pub(super) fn visit_atoms(&self, visit: &mut impl FnMut(&symbolica::atom::Atom)) {
        for image in &self.images {
            visit(&image.0);
        }
        visit(&self.measure_jacobian.0);
        if let Some(value) = &self.pre_subtraction {
            value.visit_atoms(visit);
        }
        if let Some(value) = &self.contour {
            value.visit_atoms(visit);
        }
    }

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
            images: coordinates.images().iter().map(StoredAtom::from).collect(),
            measure_jacobian: coordinates.measure_jacobian().into(),
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
            pre_subtraction: chart
                .pre_subtraction()
                .map(pre_subtraction::PortablePreSubtraction::from_native),
            contour: chart.contour().map(contour::PortableContour::from_native),
        }
    }
    pub(super) fn into_native(
        self,
        index: usize,
        assessment: &DomainAssessment,
        validate: bool,
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
                != (0..dimension).collect::<std::collections::BTreeSet<_>>()
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
        if geometry.source_dimension() != source.len()
            || geometry.dimension() != dimension
            || geometry
                .exponent_matrix
                .iter()
                .any(|row| row.len() != dimension)
            || geometry
                .factor_valuations
                .iter()
                .any(|row| row.len() != dimension)
        {
            return Err(invalid("coordinate map dimensions differ"));
        }
        match (domain, geometry.fixed_parameter) {
            (ParametricDomain::ProjectiveSimplex, Some(pivot))
                if source.len() == dimension.saturating_add(1) && pivot < source.len() => {}
            (ParametricDomain::UnitCube | ParametricDomain::PositiveOrthant, None)
                if source.len() == dimension => {}
            _ => return Err(invalid("coordinate map domain or fixed parameter differs")),
        }
        if geometry.determinant <= 0 {
            return Err(invalid("nonpositive coordinate measure determinant"));
        }
        let images = self
            .images
            .into_iter()
            .map(atom)
            .collect::<Result<Vec<_>, _>>()?;
        if images.len() != source.len() {
            return Err(invalid("coordinate image count differs"));
        }
        let measure_jacobian = atom(self.measure_jacobian)?;
        let coordinates = if validate {
            geometry
                .validate(domain)
                .map_err(|error| invalid(&error.to_string()))?;
            let coordinates = coordinates_from_parts(&source, domain, &geometry, &target);
            if images != coordinates.images() || measure_jacobian != *coordinates.measure_jacobian()
            {
                return Err(invalid(
                    "retained coordinate images or measure differ from exact geometry",
                ));
            }
            coordinates
        } else {
            CoordinateMap {
                source_parameters: source.clone(),
                target_parameters: target.clone(),
                images,
                measure_jacobian,
                measure_factor: symbolica::atom::Atom::num(geometry.determinant.clone()),
                measure_powers: geometry
                    .jacobian_powers
                    .iter()
                    .cloned()
                    .map(symbolica::atom::Atom::num)
                    .collect(),
                source_domain: domain,
                projective_fixed_parameter: geometry.fixed_parameter,
            }
        };
        let pre_subtraction = self
            .pre_subtraction
            .map(|record| record.into_native(&source, &target))
            .transpose()?;
        let contour = self
            .contour
            .map(|value| value.into_native(dimension, validate))
            .transpose()?;
        Ok(ChartRecord {
            source_index: index,
            representative: self.representative,
            representative_permutation: self.representative_permutation,
            kernel_sector: self.kernel_sector,
            coordinates,
            geometry,
            pre_subtraction,
            contour,
        })
    }
}
