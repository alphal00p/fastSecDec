use super::super::{StoredAtom, atom, invalid, symbol_strings, symbols};
use crate::{
    contour::{ContourDefinitions, ContourMetadata},
    kernel::KernelError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore},
    state::StateMap,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortableContour {
    version: u32,
    causal_polynomial: StoredAtom,
    positive_polynomials: Vec<StoredAtom>,
    images: Vec<StoredAtom>,
    ratios: Vec<StoredAtom>,
    jacobian: StoredAtom,
    validation_faces: Vec<Vec<(usize, u8)>>,
    #[serde(default, skip_serializing_if = "PortableDefinitions::is_empty")]
    function_definitions: PortableDefinitions,
}

/// Canonical readable metadata; binary records use the owner's StateMap codec
/// in the v12 sidecar instead. This delegates Atom text to the existing adapter.
#[derive(Default)]
struct PortableDefinitions(ContourDefinitions);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    function: String,
    parameters: Vec<String>,
    body: StoredAtom,
}
impl PortableDefinitions {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl Serialize for PortableDefinitions {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut sequence = serializer.serialize_seq(Some(self.0.entries().len()))?;
        for entry in self.0.entries() {
            sequence.serialize_element(&Definition {
                function: Atom::var(entry.function()).to_canonical_string(),
                parameters: symbol_strings(entry.parameters()),
                body: entry.body().into(),
            })?;
        }
        sequence.end()
    }
}
impl<'de> Deserialize<'de> for PortableDefinitions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<Definition>::deserialize(deserializer)?;
        let parts = values
            .into_iter()
            .map(|value| {
                let function = symbols(vec![value.function])?.remove(0);
                Ok((function, symbols(value.parameters)?, atom(value.body)?))
            })
            .collect::<Result<Vec<_>, KernelError>>()
            .map_err(serde::de::Error::custom)?;
        ContourDefinitions::from_parts(parts)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

// Preserve the exact v9–v11 contour layout. V12 extracts definitions into its
// explicitly versioned sidecar before encoding this nested legacy record.
impl bincode::Encode for PortableContour {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        if !self.function_definitions.is_empty() {
            return Err(bincode::error::EncodeError::Other(
                "compact contour definitions require the v12 sidecar",
            ));
        }
        self.version.encode(encoder)?;
        self.causal_polynomial.encode(encoder)?;
        self.positive_polynomials.encode(encoder)?;
        self.images.encode(encoder)?;
        self.ratios.encode(encoder)?;
        self.jacobian.encode(encoder)?;
        self.validation_faces.encode(encoder)
    }
}
impl bincode::Decode<StateMap> for PortableContour {
    fn decode<D: bincode::de::Decoder<Context = StateMap>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            version: bincode::Decode::decode(decoder)?,
            causal_polynomial: bincode::Decode::decode(decoder)?,
            positive_polynomials: bincode::Decode::decode(decoder)?,
            images: bincode::Decode::decode(decoder)?,
            ratios: bincode::Decode::decode(decoder)?,
            jacobian: bincode::Decode::decode(decoder)?,
            validation_faces: bincode::Decode::decode(decoder)?,
            function_definitions: PortableDefinitions::default(),
        })
    }
}
bincode::impl_borrow_decode_with_context!(PortableContour, StateMap);

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
            function_definitions: PortableDefinitions(value.function_definitions().clone()),
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
        for entry in self.function_definitions.0.entries() {
            visit(&Atom::var(entry.function()));
            for parameter in entry.parameters() {
                visit(&Atom::var(*parameter));
            }
            visit(entry.body());
        }
    }
    pub(super) fn take_definitions(&mut self) -> ContourDefinitions {
        std::mem::take(&mut self.function_definitions).0
    }
    pub(super) fn attach_definitions(
        &mut self,
        definitions: ContourDefinitions,
    ) -> Result<(), KernelError> {
        if self.version != 2 || definitions.is_empty() || !self.function_definitions.is_empty() {
            return Err(invalid(
                "invalid or duplicate compact contour definition sidecar",
            ));
        }
        definitions.validate(false).map_err(|e| invalid(&e))?;
        self.function_definitions = PortableDefinitions(definitions);
        Ok(())
    }
    pub(super) fn into_native(
        self,
        dimension: usize,
        validate: bool,
    ) -> Result<ContourMetadata, KernelError> {
        let expected_version = if self.function_definitions.is_empty() {
            1
        } else {
            2
        };
        if self.version != expected_version
            || self.images.len() != dimension
            || self.ratios.len() != dimension
        {
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
        self.function_definitions
            .0
            .validate(validate)
            .map_err(|e| invalid(&e))?;
        self.function_definitions
            .0
            .select(
                self.images
                    .iter()
                    .chain(&self.ratios)
                    .map(|value| &value.0)
                    .chain([&self.jacobian.0]),
            )
            .map_err(|e| invalid(&e))?;
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
            definitions: Arc::new(self.function_definitions.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_nested_layout_has_no_definition_field() {
        #[derive(bincode::Encode)]
        struct HistoricalContour<'a> {
            version: u32,
            causal_polynomial: &'a Atom,
            positive_polynomials: &'a [Atom],
            images: &'a [Atom],
            ratios: &'a [Atom],
            jacobian: &'a Atom,
            validation_faces: &'a [Vec<(usize, u8)>],
        }
        // A captured byte-layout comparison is independent of the current manual
        // metadata adapter; existing v9/v10/v11 public load tests exercise readers.
        let x = symbolica::symbol!("legacy_contour_wire::x");
        let map = crate::contour::FixedContourMap::new(&[x], Atom::num(2) + Atom::var(x)).unwrap();
        let contour = map.metadata();
        let old = bincode::encode_to_vec(
            HistoricalContour {
                version: 1,
                causal_polynomial: contour.causal_polynomial(),
                positive_polynomials: contour.positive_polynomials(),
                images: contour.images(),
                ratios: contour.ratios(),
                jacobian: contour.jacobian(),
                validation_faces: contour.validation_faces(),
            },
            bincode::config::standard(),
        )
        .unwrap();
        let portable = PortableContour::from_native(contour);
        assert!(
            serde_json::to_value(&portable)
                .unwrap()
                .get("function_definitions")
                .is_none()
        );
        assert_eq!(
            bincode::encode_to_vec(portable, bincode::config::standard()).unwrap(),
            old
        );
    }
}
