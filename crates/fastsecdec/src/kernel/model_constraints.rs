//! Validity of a generic named-mass expansion at a supplied physical point.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::float::{Complex, Float, SingleFloat},
};

use super::{KernelError, KernelSet};

#[cfg(test)]
mod tests;

/// A generic propagator mass must remain finite, real and nonzero at runtime.
/// A massless specialization has different endpoint structure and must be
/// generated explicitly; the model's literal ZERO needs no such constraint.
#[derive(Clone, Debug)]
pub struct RuntimeMassConstraint {
    pub name: String,
    pub expression: Atom,
}

impl KernelSet {
    pub fn runtime_mass_constraints(&self) -> &[RuntimeMassConstraint] {
        &self.runtime_mass_constraints
    }

    pub fn with_runtime_mass_constraints(
        mut self,
        constraints: Vec<RuntimeMassConstraint>,
    ) -> Result<Self, KernelError> {
        if self.template_content_id.is_some() {
            return Err(KernelError::Parameters(
                "attach mass constraints before binding a physical point".into(),
            ));
        }
        self.runtime_mass_constraints = constraints;
        self.validate_runtime_mass_constraints()?;
        self.initialize_artifact()?;
        Ok(self)
    }

    pub(super) fn validate_runtime_mass_constraints(&self) -> Result<(), KernelError> {
        let mut names = BTreeSet::new();
        if self.runtime_parameters.is_empty() && !self.runtime_mass_constraints.is_empty() {
            return Err(KernelError::Artifact(
                "runtime mass constraints require runtime parameters".into(),
            ));
        }
        for constraint in &self.runtime_mass_constraints {
            if constraint.name.is_empty() || !names.insert(&constraint.name) {
                return Err(KernelError::Artifact(
                    "invalid or duplicate runtime mass name".into(),
                ));
            }
            // Reuse native indeterminate discovery without constructing an
            // evaluator. Do not descend into functions: their leading arguments
            // may be symbolic callback tags, not runtime variables. Native
            // evaluation validates those bodies/arity at the supplied point,
            // before a session can be created from the bound kernel set.
            for value in constraint.expression.get_all_indeterminates(false) {
                if let AtomView::Var(variable) = value {
                    let symbol = variable.get_symbol();
                    if !self.runtime_parameters.contains(&symbol)
                        && !symbol
                            .get_evaluation_info()
                            .is_some_and(|info| info.has_constant_evaluator())
                    {
                        return Err(KernelError::Artifact(format!(
                            "runtime mass {} contains undeclared parameter {symbol}",
                            constraint.name
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_runtime_masses_at(
        &self,
        values: &BTreeMap<Symbol, f64>,
    ) -> Result<(), KernelError> {
        if self.runtime_mass_constraints.is_empty() {
            return Ok(());
        }
        let point = values
            .iter()
            .map(|(symbol, value)| {
                (
                    Atom::var(*symbol),
                    Complex::new(Float::with_val(128, *value), Float::with_val(128, 0)),
                )
            })
            .collect::<HashMap<_, _>>();
        for constraint in &self.runtime_mass_constraints {
            let mass = constraint
                .expression
                .evaluate_with_prec(&point, 128)
                .map_err(|error| {
                    KernelError::Parameters(format!(
                        "mass {} could not be evaluated: {error}",
                        constraint.name
                    ))
                })?;
            if !mass.re.is_finite() || !mass.im.is_finite() || !mass.im.is_zero() {
                return Err(KernelError::Parameters(format!(
                    "mass {} must be finite and real",
                    constraint.name
                )));
            }
            if mass.re.is_zero() {
                return Err(KernelError::Parameters(format!(
                    "mass {} is zero; generate an explicit massless specialization because its endpoint structure can change",
                    constraint.name
                )));
            }
        }
        Ok(())
    }
}
