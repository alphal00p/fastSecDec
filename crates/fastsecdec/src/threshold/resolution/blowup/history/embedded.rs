use super::*;
impl ResolutionHistory {
    pub(crate) fn advanced_embedded_presentation(
        &self,
        owner: &super::super::presentation_transition::EmbeddedTransition,
        open: &super::super::presentation_transition::EmbeddedTransitionOpen,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, owner.source().history().as_ref())
            || !self.same_root(owner.source().original_cycle().initial_history())
            || !owner.opens().iter().any(|o| std::ptr::eq(o.as_ref(), open))
            || !Arc::ptr_eq(open.coefficient.source(), owner.source().coefficient())
            || !Arc::ptr_eq(open.incidence_sum.source(), owner.source().incidence_sum())
            || !Arc::ptr_eq(open.center.support(), owner.support())
            || !Arc::ptr_eq(open.coefficient.open(), open.center.open())
            || !Arc::ptr_eq(open.incidence_sum.open(), open.center.open())
            || open.center.source().ideal() != &owner.lower_center
            || open.center.source().mark() != 1
            || open.center.exceptional_power()
                != usize::from(owner.chart().geometry().exceptional().is_some())
            || open.born.is_some() != owner.chart().geometry().exceptional().is_some()
            || open.divisors.len() != self.ledger.divisors().len()
            || !Arc::ptr_eq(open.ledger.frame(), open.center.open().frame())
        {
            return Err(Error::Invalid(
                "repeated embedded history actual source/transition owner",
            ));
        }
        let local = open.ledger.frame().local();
        if !local
            .ideal()
            .sum(open.center.target().ideal(), b)?
            .contains(&local.ring().one(), local.unit_relations(), b)?
        {
            return Err(Error::Invalid("repeated lower center principal pullback"));
        }
        for (old, receipt) in self.ledger.divisors().iter().zip(&open.divisors) {
            let actual = open.ledger.divisors().iter().find(|d| d.id == old.id);
            if receipt.id != old.id
                || receipt.absent != actual.is_none()
                || actual.is_some_and(|d| d.equation != receipt.strict_equation)
            {
                return Err(Error::Invalid("repeated lower strict divisor inventory"));
            }
        }
        let provenance = HistoryCenter::RelativeIdeal {
            source_ring: self.ledger.frame().local().ring().clone(),
            ideal: owner.lower_center.clone(),
            normals: owner.lower_normals.clone(),
        };
        let mut births = self.births.clone();
        let mut contexts = self.birth_contexts.clone();
        let (stage, next_id) = if let Some((id, stage)) = open.born {
            if (id, stage) != self.next_transition()? || births.insert(id, stage).is_some() {
                return Err(Error::Invalid("repeated lower birth frontier"));
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
                    "repeated lower identity exhaustion",
                ))?),
            )
        } else {
            (self.stage, self.next_id)
        };
        if open
            .ledger
            .divisors()
            .iter()
            .any(|d| !births.contains_key(&d.id))
        {
            return Err(Error::Invalid("repeated lower unissued divisor"));
        }
        b.reserve_slots(
            self.path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("repeated lower ancestry count"))?,
        )?;
        let mut path = self.path.clone();
        let geometry = owner.chart().geometry();
        let support = open.center.open();
        path.push(HistoryStep::EmbeddedBlowup {
            center: provenance,
            ambient_path: owner.chart().history().chart_path().to_vec(),
            source_open: geometry.open().source_open().clone(),
            pivot_normal: geometry.pivot_normal(),
            support_equations: support.chosen_equations().to_vec(),
            support_columns: support.columns().to_vec(),
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger: open.ledger.clone(),
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id,
            path,
            birth_contexts: contexts,
        }))
    }
}
