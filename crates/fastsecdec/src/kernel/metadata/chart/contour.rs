use super::super::{StoredAtom, atom, invalid};
use crate::{contour::ContourMetadata, kernel::KernelError};
use serde::{Deserialize, Serialize};
use symbolica::{atom::Atom, state::StateMap};

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
pub(super) struct PortableContour {
    version: u32,
    causal_polynomial: StoredAtom,
    positive_polynomials: Vec<StoredAtom>,
    images: Vec<StoredAtom>,
    ratios: Vec<StoredAtom>,
    jacobian: StoredAtom,
    validation_faces: Vec<Vec<(usize, u8)>>,
}
impl PortableContour {
    pub(super) fn from_native(value: &ContourMetadata) -> Self {
        Self {
            version: value.version(),
            causal_polynomial: value.causal_polynomial().into(),
            positive_polynomials: value
                .positive_polynomials()
                .iter()
                .map(Into::into)
                .collect(),
            images: value.images().iter().map(Into::into).collect(),
            ratios: value.ratios().iter().map(Into::into).collect(),
            jacobian: value.jacobian().into(),
            validation_faces: value.validation_faces().to_vec(),
        }
    }
    pub(super) fn visit_atoms(&self, visit: &mut impl FnMut(&Atom)) {
        visit(&self.causal_polynomial.0);
        visit(&self.jacobian.0);
        for value in self
            .positive_polynomials
            .iter()
            .chain(&self.images)
            .chain(&self.ratios)
        {
            visit(&value.0);
        }
    }
    pub(super) fn into_native(self, dimension: usize) -> Result<ContourMetadata, KernelError> {
        if self.version != 1 || self.images.len() != dimension || self.ratios.len() != dimension {
            return Err(invalid(
                "unsupported contour recipe or inconsistent chart dimension",
            ));
        }
        if !self.validation_faces.iter().any(Vec::is_empty)
            || self.validation_faces.iter().any(|face| {
                let unique = face
                    .iter()
                    .map(|entry| entry.0)
                    .collect::<std::collections::BTreeSet<_>>();
                unique.len() != face.len()
                    || face
                        .iter()
                        .any(|(axis, value)| *axis >= dimension || *value > 1)
            })
        {
            return Err(invalid("invalid contour subtraction-face coordinates"));
        }
        Ok(ContourMetadata {
            causal_polynomial: atom(self.causal_polynomial)?,
            positive_polynomials: self
                .positive_polynomials
                .into_iter()
                .map(atom)
                .collect::<Result<_, _>>()?,
            images: self
                .images
                .into_iter()
                .map(atom)
                .collect::<Result<_, _>>()?,
            ratios: self
                .ratios
                .into_iter()
                .map(atom)
                .collect::<Result<_, _>>()?,
            jacobian: atom(self.jacobian)?,
            validation_faces: self.validation_faces,
        })
    }
}
