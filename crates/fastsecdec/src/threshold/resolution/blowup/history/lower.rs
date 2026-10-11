use super::*;
impl ResolutionHistory {
    /// One birth/path implementation for original and repeated supported levels.
    pub(in super::super) fn advanced_lower_boundary(
        &self,
        owner: &super::super::lower_boundary::LowerBoundaryTransform,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, owner.prior.as_ref())
            || owner.divisors.len() != self.ledger.divisors().len()
            || !Arc::ptr_eq(owner.ledger.frame(), owner.center.open().frame())
            || owner.born.is_some() != owner.chart.geometry().exceptional().is_some()
        {
            return Err(Error::Invalid("shared lower history source owner"));
        }
        let local = owner.ledger.frame().local();
        if !local
            .ideal()
            .sum(owner.center.target().ideal(), b)?
            .contains(&local.ring().one(), local.unit_relations(), b)?
        {
            return Err(Error::Invalid("shared lower center principal pullback"));
        }
        for (old, receipt) in self.ledger.divisors().iter().zip(&owner.divisors) {
            let actual = owner.ledger.divisors().iter().find(|d| d.id == old.id);
            if receipt.id != old.id
                || receipt.absent != actual.is_none()
                || actual.is_some_and(|d| d.equation != receipt.strict_equation)
            {
                return Err(Error::Invalid("shared lower strict divisor inventory"));
            }
        }
        let provenance = HistoryCenter::RelativeIdeal {
            source_ring: self.ledger.frame().local().ring().clone(),
            ideal: owner.center.source().ideal().clone(),
            normals: owner.normals.clone(),
        };
        let mut births = self.births.clone();
        let mut contexts = self.birth_contexts.clone();
        let (stage, next_id) = if let Some((id, stage)) = owner.born {
            if (id, stage) != self.next_transition()? || births.insert(id, stage).is_some() {
                return Err(Error::Invalid("shared lower birth frontier"));
            }
            contexts.insert(
                id,
                BirthContext {
                    parent_chart_path: self.path.clone(),
                    center: provenance.clone(),
                },
            );
            (
                stage,
                BoundaryId(id.0.checked_add(1).ok_or(Error::ResourceIncomplete(
                    "shared lower identity exhaustion",
                ))?),
            )
        } else {
            (self.stage, self.next_id)
        };
        if owner
            .ledger
            .divisors()
            .iter()
            .any(|d| !births.contains_key(&d.id))
        {
            return Err(Error::Invalid("shared lower unissued divisor"));
        }
        b.reserve_slots(
            self.path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("shared lower ancestry count"))?,
        )?;
        let mut path = self.path.clone();
        let geometry = owner.chart.geometry();
        let support = owner.center.open();
        path.push(HistoryStep::EmbeddedBlowup {
            center: provenance,
            ambient_path: owner.chart.history().chart_path().to_vec(),
            source_open: geometry.open().source_open().clone(),
            pivot_normal: geometry.pivot_normal(),
            support_equations: support.chosen_equations().to_vec(),
            support_columns: support.columns().to_vec(),
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger: owner.ledger.clone(),
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id,
            path,
            birth_contexts: contexts,
        }))
    }
}
