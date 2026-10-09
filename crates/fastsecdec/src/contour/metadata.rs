use symbolica::atom::Atom;

/// Native chart data before subtraction, retained for checks and inspection.
#[derive(Clone, Debug)]
pub struct ContourMetadata {
    pub(crate) causal_polynomial: Atom,
    pub(crate) positive_polynomials: Vec<Atom>,
    pub(crate) images: Vec<Atom>,
    pub(crate) ratios: Vec<Atom>,
    pub(crate) jacobian: Atom,
    /// Coordinate restrictions applied to this same full-sector map.
    pub(crate) validation_faces: Vec<Vec<(usize, u8)>>,
}
impl ContourMetadata {
    pub fn version(&self) -> u32 {
        1
    }
    pub fn causal_polynomial(&self) -> &Atom {
        &self.causal_polynomial
    }
    pub fn positive_polynomials(&self) -> &[Atom] {
        &self.positive_polynomials
    }
    pub fn images(&self) -> &[Atom] {
        &self.images
    }
    pub fn ratios(&self) -> &[Atom] {
        &self.ratios
    }
    pub fn jacobian(&self) -> &Atom {
        &self.jacobian
    }
    pub fn validation_faces(&self) -> &[Vec<(usize, u8)>] {
        &self.validation_faces
    }

    pub(crate) fn record_subtraction_faces(
        &mut self,
        metadata: &crate::generation::PreSubtractionMetadata,
        strategy: crate::generation::SubtractionStrategy,
    ) {
        use crate::generation::SubtractionStrategy;
        use std::collections::BTreeSet;
        let mut faces = BTreeSet::from([Vec::new()]);
        for term in metadata.terms() {
            let mut term_faces = vec![Vec::new()];
            for (axis, power) in term
                .powers()
                .iter()
                .enumerate()
                .filter(|(_, power)| power.subtraction_count() > 0)
            {
                let extra = term_faces
                    .iter()
                    .flat_map(|face| {
                        let mut zero = face.clone();
                        zero.push((axis, 0));
                        let mut result = vec![zero];
                        if strategy == SubtractionStrategy::IntegrateByParts
                            && power.subtraction_count() >= 2
                            && !power.slope().is_zero()
                        {
                            let mut one = face.clone();
                            one.push((axis, 1));
                            result.push(one);
                        }
                        result
                    })
                    .collect::<Vec<_>>();
                term_faces.extend(extra);
            }
            faces.extend(term_faces);
        }
        self.validation_faces = faces.into_iter().collect();
    }
}
