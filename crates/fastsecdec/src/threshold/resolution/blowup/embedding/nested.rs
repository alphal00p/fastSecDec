//! Nested support construction from actual checked contact/localization owners.
//! This module flattens exact defining equations, not the density or BM history.
use super::*;
use symbolica::poly::PolyVariable;

fn compose(first: &RingExtension, second: &RingExtension, b: &mut Budget) -> Result<RingExtension> {
    if first.target() != second.source() {
        return Err(Error::Invalid("nested extension middle ring"));
    }
    extension_to(first.source(), second.target(), b)
}
pub(super) fn extension_to(
    old: &Arc<Ring>,
    target: &Arc<Ring>,
    b: &mut Budget,
) -> Result<RingExtension> {
    if target.len() < old.len() {
        return Err(Error::Invalid("nested extension dimension"));
    }
    let variables = target.one().variables().clone();
    if variables[..old.len()] != old.one().variables()[..]
        || (0..old.len()).any(|i| old.is_parameter(i) != target.is_parameter(i))
        || (old.len()..target.len()).any(|i| target.is_parameter(i))
    {
        return Err(Error::Invalid("nested extension original coordinate roles"));
    }
    b.reserve_slots(target.len() - old.len())?;
    let fresh = variables[old.len()..]
        .iter()
        .map(|v| match v {
            PolyVariable::Symbol(s) => Ok(*s),
            _ => Err(Error::Invalid("nested extension symbol kind")),
        })
        .collect::<Result<Vec<_>>>()?;
    let extension = RingExtension::with_symbols(old.clone(), &fresh, b)?;
    if extension.target() != target {
        return Err(Error::Invalid("nested extension target identity"));
    }
    Ok(extension)
}

pub(super) fn lift_unit_fraction(
    parent: &SupportEmbedding,
    value: &Poly,
    b: &mut Budget,
) -> Result<UnitClearing> {
    let clearing = clear_units(parent.frame().local(), value, b)?;
    let ring = parent.ambient().local().ring();
    for p in [&clearing.numerator, &clearing.denominator] {
        if (ring.len()..p.nvars()).any(|i| p.degree(i) != 0) {
            return Err(Error::ResourceIncomplete(
                "nested support retains an auxiliary slot",
            ));
        }
    }
    let numerator = super::super::super::elimination::remap(&clearing.numerator, ring, b)?;
    let denominator = super::super::super::elimination::remap(&clearing.denominator, ring, b)?;
    if !super::super::helpers::unit(parent.ambient().local(), &denominator, b)? {
        return Err(Error::ResourceIncomplete(
            "nested support requires ambient denominator open",
        ));
    }
    Ok(UnitClearing {
        original: value.clone(),
        numerator,
        denominator,
    })
}

pub(super) fn full_equations(
    ambient: &EtaleFrame,
    frame: &EtaleFrame,
    extension: &RingExtension,
    equations: &Ideal,
    b: &mut Budget,
) -> Result<()> {
    let source = ambient.local().ideal().sum(equations, b)?;
    let pulled = extension.ideal(&source, b)?;
    let target = frame.local();
    for (a, z) in [(&pulled, target.ideal()), (target.ideal(), &pulled)] {
        for f in a.generators() {
            if !z.contains(f, target.unit_relations(), b)? {
                return Err(Error::Invalid(
                    "nested support full localized equation equality",
                ));
            }
        }
    }
    Ok(())
}

impl SupportEmbedding {
    /// Restrict a support by an existing lower open only when it was already
    /// a unit on the physical ambient chart. Otherwise use a physical cover.
    pub fn restrict_unit_open(
        self: &Arc<Self>,
        localization: Arc<LocalizedCoverHistory>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        Self::localized(self.clone(), localization, b)
    }
    pub(crate) fn nested(
        parent: Arc<Self>,
        contact: Arc<ContactQuotient>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !Arc::ptr_eq(parent.frame(), contact.source().frame()) {
            return Err(Error::Invalid("nested contact actual parent frame"));
        }
        let old_ring = parent.frame().local().ring();
        let equation =
            super::super::super::elimination::remap(&contact.clearing().numerator, old_ring, b)?;
        let clearing = lift_unit_fraction(&parent, &equation, b)?;
        let singleton = Ideal::new(
            parent.ambient().local().ring().clone(),
            vec![clearing.numerator.clone()],
            b,
        )?;
        let equations = Arc::new(parent.equations().sum(&singleton, b)?);
        let extension = compose(parent.extension(), contact.extension(), b)?;
        let frame = contact.contact().clone();
        let codimension = parent
            .codimension()
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("nested support codimension"))?;
        if parent
            .ambient()
            .free_axes()
            .len()
            .checked_sub(frame.free_axes().len())
            != Some(codimension)
        {
            return Err(Error::Invalid("nested support dimension descent"));
        }
        full_equations(parent.ambient(), &frame, &extension, &equations, b)?;
        Ok(Arc::new(Self {
            ambient: parent.ambient().clone(),
            frame,
            extension,
            equations,
            codimension,
            origin: EmbeddingOrigin::Nested {
                parent,
                contact,
                clearing,
            },
        }))
    }
    pub(crate) fn localized(
        parent: Arc<Self>,
        localization: Arc<LocalizedCoverHistory>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        let open = localization.open();
        if !Arc::ptr_eq(parent.frame(), open.source().ledger().frame()) {
            return Err(Error::Invalid("localized support actual parent frame"));
        }
        let clearing = lift_unit_fraction(&parent, open.factor(), b)?;
        if !super::super::helpers::unit(parent.ambient().local(), &clearing.numerator, b)? {
            return Err(Error::ResourceIncomplete(
                "nested support requires actual ambient open",
            ));
        }
        let extension = compose(parent.extension(), open.extension(), b)?;
        let frame = localization.history().ledger().frame().clone();
        if frame.free_axes().len() != parent.frame().free_axes().len() {
            return Err(Error::Invalid("localized support relative dimension"));
        }
        full_equations(parent.ambient(), &frame, &extension, parent.equations(), b)?;
        Ok(Arc::new(Self {
            ambient: parent.ambient().clone(),
            frame,
            extension,
            equations: parent.equations().clone(),
            codimension: parent.codimension(),
            origin: EmbeddingOrigin::Localized {
                parent,
                localization,
                clearing,
            },
        }))
    }
    pub fn nested_contact(&self) -> Option<(&Arc<Self>, &Arc<ContactQuotient>, &UnitClearing)> {
        match &self.origin {
            EmbeddingOrigin::Nested {
                parent,
                contact,
                clearing,
            } => Some((parent, contact, clearing)),
            _ => None,
        }
    }
    pub fn localized_origin(
        &self,
    ) -> Option<(&Arc<Self>, &Arc<LocalizedCoverHistory>, &UnitClearing)> {
        match &self.origin {
            EmbeddingOrigin::Localized {
                parent,
                localization,
                clearing,
            } => Some((parent, localization, clearing)),
            _ => None,
        }
    }
}
