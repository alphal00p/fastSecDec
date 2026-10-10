//! Contribution inventory and selected native residency, independent of source extent.
use super::validate::{invalid, ordered};
use super::*;
use std::collections::BTreeSet;

impl StructureCheckedLineage<'_> {
    fn check_manifest(&self, manifest: &Digest) -> Result<()> {
        if manifest != &self.descriptor {
            return Err(invalid("record parent differs from checked manifest"));
        }
        Ok(())
    }
    pub fn validate_record(&self, manifest: &Digest, record: &RecordLineageV1) -> Result<()> {
        self.check_manifest(manifest)?;
        if &record.manifest != manifest {
            return Err(invalid("foreign lineage manifest"));
        }
        if record.contributions.is_empty() {
            return Err(invalid("empty record contribution list"));
        }
        ordered(&record.contributions, "record contributions")?;
        for id in &record.contributions {
            let index = self
                .manifest
                .contributions
                .binary_search_by_key(id, |c| c.id)
                .map_err(|_| invalid("unknown record contribution"))?;
            match (&self.manifest.contributions[index].kind, &record.kind) {
                (ContributionKind::Exact, RecordKind::Exact) => {}
                (
                    ContributionKind::Stochastic { coordinates: a },
                    RecordKind::Stochastic { coordinates: b },
                ) if a == b => {}
                _ => return Err(invalid("record kind differs from contribution")),
            }
        }
        Ok(())
    }

    /// Closed archive coverage. Selective record loading calls validate_record;
    /// it cannot use a partial set to obtain complete-archive acceptance.
    pub fn validate_complete_records(
        &self,
        manifest: &Digest,
        records: &[RecordLineageV1],
    ) -> Result<()> {
        self.check_manifest(manifest)?;
        let mut observed = BTreeSet::new();
        for record in records {
            self.validate_record(manifest, record)?;
            for id in &record.contributions {
                if !observed.insert(*id) {
                    return Err(invalid("duplicate archived contribution"));
                }
            }
        }
        let expected: BTreeSet<_> = self
            .manifest
            .contributions
            .iter()
            .filter(|c| !matches!(c.kind, ContributionKind::CertifiedZero { .. }))
            .map(|c| c.id)
            .collect();
        if observed != expected {
            return Err(invalid("incomplete archived contribution inventory"));
        }
        Ok(())
    }

    /// Return only the scope qualification after checking resident ownership.
    /// `true` is not mathematical verification of the referenced certificates.
    pub fn validate_resident_records(
        &self,
        manifest: &Digest,
        resident: &ResidentLineageV1,
        records: &[RecordLineageV1],
    ) -> Result<bool> {
        self.check_manifest(manifest)?;
        if &resident.manifest != manifest {
            return Err(invalid("foreign resident manifest"));
        }
        match &resident.selection {
            ResidentSelection::Complete => {
                self.validate_complete_records(manifest, records)?;
                Ok(matches!(
                    self.manifest.preparation.extent,
                    SourceExtent::Full { .. }
                ))
            }
            ResidentSelection::Selected {
                stochastic_contributions,
                exact_policy,
            } => {
                use crate::results::ExactContributionPolicy;
                ordered(stochastic_contributions, "selected contributions")?;
                let mut expected = BTreeSet::new();
                for id in stochastic_contributions {
                    let index = self
                        .manifest
                        .contributions
                        .binary_search_by_key(id, |c| c.id)
                        .map_err(|_| invalid("unknown selected contribution"))?;
                    let contribution = &self.manifest.contributions[index];
                    if !matches!(contribution.kind, ContributionKind::Stochastic { .. }) {
                        return Err(invalid("selected stochastic ID is not stochastic"));
                    }
                    expected.insert(*id);
                }
                if *exact_policy == ExactContributionPolicy::IncludeAll {
                    expected.extend(
                        self.manifest
                            .contributions
                            .iter()
                            .filter(|c| matches!(c.kind, ContributionKind::Exact))
                            .map(|c| c.id),
                    );
                }
                let mut observed = BTreeSet::new();
                for record in records {
                    self.validate_record(manifest, record)?;
                    for id in &record.contributions {
                        if !observed.insert(*id) {
                            return Err(invalid("duplicate resident contribution"));
                        }
                    }
                }
                if observed != expected {
                    return Err(invalid(
                        "resident records differ from selected contribution/exact policy",
                    ));
                }
                Ok(false)
            }
        }
    }
}
