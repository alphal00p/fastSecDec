use super::super::{
    Budget, ContactProduction, ContactQuotient, Error, OldBoundaryCoefficient,
    OldBoundaryProduction, construct_contact_quotient, produce_old_boundary_coefficient,
};
use super::CompanionContactOpen;
use std::sync::Arc;
use symbolica::atom::Symbol;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct CompanionCoefficientChart {
    source: Arc<CompanionContactOpen>,
    contact: Arc<ContactQuotient>,
    coefficient: Arc<OldBoundaryCoefficient>,
}
impl CompanionCoefficientChart {
    pub fn source(&self) -> &Arc<CompanionContactOpen> {
        &self.source
    }
    pub fn contact(&self) -> &Arc<ContactQuotient> {
        &self.contact
    }
    pub fn coefficient(&self) -> &Arc<OldBoundaryCoefficient> {
        &self.coefficient
    }
}
#[derive(Clone, Debug)]
pub enum CompanionCoefficientProduction {
    Coefficient(Box<CompanionCoefficientChart>),
    ContactPending {
        source: Arc<CompanionContactOpen>,
        evidence: ContactProduction,
    },
    BoundaryPending {
        source: Arc<CompanionContactOpen>,
        contact: Arc<ContactQuotient>,
        evidence: OldBoundaryProduction,
    },
}
/// Local coefficient recursion arithmetic with the actual restricted history.
/// No global BM-center lifting/gluing or general termination is inferred.
pub fn construct_companion_coefficient(
    source: Arc<CompanionContactOpen>,
    candidate: usize,
    fresh: [Symbol; 2],
    budget: &mut Budget,
) -> Result<CompanionCoefficientProduction> {
    let contact =
        match construct_contact_quotient(source.contact().clone(), candidate, fresh, budget)? {
            ContactProduction::Constructed(contact) => Arc::new(*contact),
            evidence => {
                return Ok(CompanionCoefficientProduction::ContactPending { source, evidence });
            }
        };
    // The existing owner gate rejects equality-only substitutes: this pointer
    // must be exactly the actual upper-order-open frame used by contact.
    let history = source.restriction().history().clone();
    if !Arc::ptr_eq(contact.source().frame(), history.ledger().frame()) {
        return Err(Error::Invalid("companion coefficient history owner"));
    }
    match produce_old_boundary_coefficient(contact.clone(), history, budget)? {
        OldBoundaryProduction::Coefficient(coefficient) => Ok(
            CompanionCoefficientProduction::Coefficient(Box::new(CompanionCoefficientChart {
                source,
                contact,
                coefficient: Arc::new(*coefficient),
            })),
        ),
        evidence => Ok(CompanionCoefficientProduction::BoundaryPending {
            source,
            contact,
            evidence,
        }),
    }
}
