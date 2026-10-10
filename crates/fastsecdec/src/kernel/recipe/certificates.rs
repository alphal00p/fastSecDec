//! Saved certificate ownership follows selected executable requests, independently
//! of mutable chart projections. No program is decoded or rebuilt here.
use super::*;
use std::sync::Arc;
use symbolica::atom::Atom;
#[cfg(test)]
mod tests;

impl NativeProgramDescriptor {
    /// Saved face ownership is a structural requirement even when optional
    /// numerical certification is disabled.
    pub(crate) fn validate_request_coverage<'a>(
        &self,
        bundles: impl IntoIterator<Item = &'a crate::contour::functions::dynamic::requests::Bundle>,
    ) -> Result<(), KernelError> {
        let certificates = self
            .certificates()
            .ok_or_else(|| invalid("dynamic requests lack saved full-sector certificates"))?;
        for request in bundles.into_iter().flat_map(|bundle| &bundle.0) {
            if !certificates.iter().any(|certificate| {
                certificate.namespace == request.namespace
                    && certificate.faces.contains(&request.face)
            }) {
                return Err(invalid(
                    "surviving dynamic request lacks its saved full-sector face context",
                ));
            }
        }
        Ok(())
    }

    /// The saved arithmetic consumes the receiving owner's ordered physical
    /// inputs. Equal arity and matching deformation controls alone are not
    /// sufficient: two physical parameters must never be silently exchanged.
    pub(crate) fn validate_certificate_runtime(
        &self,
        runtime: &[symbolica::atom::Symbol],
    ) -> Result<(), KernelError> {
        for certificate in self.certificates().into_iter().flatten() {
            let saved = certificate
                .runtime_parameters
                .iter()
                .map(|name| {
                    symbolica::atom::Symbol::parse(name, "fastsecdec::artifact").map_err(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if saved != runtime {
                return Err(invalid(
                    "saved certificate runtime inputs differ from the receiving numerical owner",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn certificates(&self) -> Option<&[DynamicCheckProgram]> {
        self.certificates.as_deref()
    }

    pub(crate) fn with_certificates(
        mut self,
        certificates: Vec<DynamicCheckProgram>,
    ) -> Result<Self, KernelError> {
        self.certificates = Some(certificates.into());
        self.validate()?;
        Ok(self)
    }

    pub(super) fn validate_certificates(&self) -> Result<(), KernelError> {
        let Some(certificates) = self.certificates() else {
            return Ok(());
        };
        let mut projections = BTreeSet::new();
        let mut namespaces = BTreeMap::<&str, &DynamicCheckProgram>::new();
        for certificate in certificates {
            certificate.validate()?;
            if certificate.recipe != self.recipe {
                return Err(invalid(
                    "certificate belongs to a different mathematical recipe",
                ));
            }
            let helper = self
                .helpers
                .iter()
                .find(|helper| helper.digest() == certificate.structure.helper_digest)
                .ok_or_else(|| {
                    invalid("certificate references a missing full-sector root helper")
                })?;
            certificate.structure.validate(helper)?;
            if let Some(index) = certificate.chart_index {
                if !projections.insert(index) {
                    return Err(invalid("duplicate certificate chart projection"));
                }
                let mut expected = self
                    .charts
                    .iter()
                    .find(|chart| chart.chart_index == index)
                    .ok_or_else(|| invalid("certificate has an unknown local chart projection"))?
                    .clone();
                expected.chart_index = 0;
                if expected != certificate.structure {
                    return Err(invalid(
                        "certificate proof differs from its local chart descriptor",
                    ));
                }
            }
            if let Some(previous) = namespaces.insert(&certificate.namespace, certificate)
                && !previous.same_mathematical_context(certificate)
            {
                return Err(invalid(
                    "shared certificate namespace has conflicting mathematical context",
                ));
            }
        }
        if projections
            .iter()
            .copied()
            .ne(self.charts.iter().map(|chart| chart.chart_index))
        {
            return Err(invalid(
                "saved certificates do not cover every retained dynamic chart",
            ));
        }
        Ok(())
    }

    pub(super) fn select_certificates(
        &self,
        sources: &[usize],
        exact: &[Atom],
        exact_requests: &[crate::contour::functions::dynamic::requests::ExactRequest],
    ) -> Result<Option<Arc<[DynamicCheckProgram]>>, KernelError> {
        let Some(certificates) = self.certificates() else {
            return Ok(None);
        };
        let mut requests = crate::contour::functions::dynamic::requests::referenced_bundles(exact)
            .map_err(invalid)?;
        requests.extend(
            crate::contour::functions::dynamic::requests::merge_exact_requests(
                exact,
                exact_requests.iter().cloned(),
            )
            .map_err(invalid)?
            .into_iter()
            .map(|request| request.bundle),
        );
        self.validate_request_coverage(&requests)?;
        let requests = requests
            .into_iter()
            .flat_map(|bundle| bundle.0.into_iter().map(|request| request.namespace))
            .collect::<BTreeSet<_>>();
        let mut covered = BTreeSet::new();
        let mut selected = Vec::new();
        for certificate in certificates {
            let projection = certificate
                .chart_index
                .and_then(|index| sources.iter().position(|source| *source == index));
            let exact_request = requests.contains(&certificate.namespace);
            if projection.is_some() || exact_request {
                let mut certificate = certificate.clone();
                certificate.chart_index = projection;
                if exact_request {
                    covered.insert(certificate.namespace.clone());
                }
                selected.push(certificate);
            }
        }
        if covered != requests {
            return Err(invalid(
                "surviving exact request lacks its saved certificate context",
            ));
        }
        Ok(Some(selected.into()))
    }

    pub(super) fn remap_certificates(&mut self, sources: &[usize]) -> Result<(), KernelError> {
        if let Some(certificates) = &mut self.certificates {
            for certificate in Arc::make_mut(certificates) {
                if let Some(index) = certificate.chart_index {
                    certificate.chart_index = Some(
                        *sources
                            .get(index)
                            .ok_or_else(|| invalid("invalid local certificate chart index"))?,
                    );
                }
            }
        }
        Ok(())
    }

    pub(super) fn merge_certificates(
        &mut self,
        other: Option<Arc<[DynamicCheckProgram]>>,
    ) -> Result<(), KernelError> {
        match (&self.certificates, other) {
            (None, None) => {}
            (Some(current), Some(other)) => {
                let mut merged = Vec::with_capacity(current.len() + other.len());
                merged.extend(current.iter().cloned());
                merged.extend(other.iter().cloned());
                self.certificates = Some(merged.into());
            }
            _ => {
                return Err(invalid(
                    "cannot merge certified and legacy native recipe descriptors",
                ));
            }
        }
        Ok(())
    }
}
