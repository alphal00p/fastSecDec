use super::super::clear_units;
use super::super::differential::relative_differential_ideal;
use super::*;

pub(super) fn ascend(
    level: Arc<RecursiveLevel>,
    child: Arc<RecursiveCenter>,
    b: &mut Budget,
) -> Result<RecursiveCenter> {
    let q = &level.contact;
    if !Arc::ptr_eq(&level.frame, q.source().frame())
        || !Arc::ptr_eq(&level.source, level.normalization.source())
        || level.normalization.normalized().ideal() != q.source().source().as_ref()
        || !Arc::ptr_eq(&child.frame, q.contact())
        || child.source.ideal() != q.differential_coefficient().ideal()
        || child.source.mark() != q.differential_coefficient().mark()
    {
        return Err(Error::Invalid("recursive child/contact owner"));
    }
    let old = level.frame.local();
    let ring = old.ring();
    let candidate = &q.source().candidates()[q.candidate_index()];
    let contact = clear_units(old, &candidate.equation, b)?;
    b.reserve_slots(
        child
            .normals
            .len()
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("recursive lifted normal count"))?,
    )?;
    let mut normals = vec![contact.numerator];
    let mut clearings = Vec::new();
    for f in &child.normals {
        let c = clear_units(q.contact().local(), f, b)?;
        if (ring.len()..c.numerator.nvars())
            .any(|i| c.numerator.degree(i) > 0 || c.denominator.degree(i) > 0)
        {
            return Err(Error::Invalid("recursive lift retained an auxiliary slot"));
        }
        let shrink = |p: &Poly| {
            p.rearrange_with_growth(ring.one().variables())
                .map_err(|_| Error::Invalid("recursive native variable-map drop"))
        };
        let n = shrink(&c.numerator)?;
        let d = shrink(&c.denominator)?;
        let unit = old.ideal().sum(&Ideal::new(ring.clone(), vec![d], b)?, b)?;
        if !unit.contains(&ring.one(), old.unit_relations(), b)? {
            return Err(Error::Invalid("recursive clearing unit fails upstairs"));
        }
        normals.push(n);
        clearings.push(c);
    }
    let ideal = Ideal::new(ring.clone(), normals.clone(), b)?;
    if old
        .ideal()
        .sum(&ideal, b)?
        .contains(&ring.one(), old.unit_relations(), b)?
    {
        return Err(Error::Invalid("recursive lifted center empty"));
    }
    let pulled = q.extension().ideal(&ideal, b)?;
    let down = q.contact().local();
    for (a, c) in [(&pulled, &child.ideal), (&child.ideal, &pulled)] {
        let c = down.ideal().sum(c, b)?;
        for f in a.generators() {
            if !c.contains(f, down.unit_relations(), b)? {
                return Err(Error::Invalid("recursive lifted center restriction"));
            }
        }
    }
    // Admissibility in the actual parent source, using the shared full relative
    // derivative engine. No differentiation in parameter directions.
    let center = old.ideal().sum(&ideal, b)?;
    let mut layer = level.source.ideal().clone();
    for j in 0..level.source.mark() {
        for f in layer.generators() {
            if !center.contains(f, old.unit_relations(), b)? {
                return Err(Error::Invalid("recursive parent source order below mark"));
            }
        }
        if j + 1 < level.source.mark() {
            layer = relative_differential_ideal(&level.frame, &layer, b)?;
        }
    }
    b.reserve_slots(
        child
            .invariant
            .len()
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("recursive invariant count"))?,
    )?;
    let mut invariant = vec![level.ratio.clone()];
    invariant.extend(child.invariant.iter().cloned());
    Ok(RecursiveCenter {
        frame: level.frame.clone(),
        source: level.source.clone(),
        ideal,
        normals,
        invariant,
        lift: Some(LiftReceipt {
            level,
            child,
            clearings,
        }),
    })
}
