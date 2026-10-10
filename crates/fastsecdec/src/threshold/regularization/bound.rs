//! Constant-fiber compiler input. The proof remains owned by the source fiber.
use super::*;

pub struct BoundContinuation<'a> {
    certificate: &'a RegularizedFiber,
    expression: Atom,
    chart_expressions: Vec<Atom>,
    profiles: Vec<Vec<crate::generation::EndpointProfileRow>>,
    functions: FunctionMap,
    bindings: Vec<(Symbol, Rational)>,
}
impl<'a> ContinuedFiber<'a> {
    /// Bind exactly the proved fiber in roots and all native function bodies.
    /// There is no alternate binding or unchecked rebind entry.
    pub fn bind_fiber(
        &self,
        mut observer: impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<BoundContinuation<'a>> {
        let mut chart_expressions = Vec::with_capacity(self.chart_expressions.len());
        for (index, expression) in self.chart_expressions.iter().enumerate() {
            poll(Progress::Continue(index), &mut observer)?;
            let expression = substitute(expression, &self.certificate.parameters);
            no_physical_parameters(&expression, self.certificate)?;
            chart_expressions.push(expression);
        }
        let expression = chart_expressions.iter().cloned().sum();
        let functions = self
            .certificate
            .functions(self.derivative_order, true, &mut observer)?;
        let bindings = self
            .certificate
            .owner
            .request()
            .kinematics()
            .runtime_parameters
            .iter()
            .map(|s| (*s, self.certificate.parameters[s].clone()))
            .collect();
        Ok(BoundContinuation {
            certificate: self.certificate,
            expression,
            chart_expressions,
            profiles: self.profiles.clone(),
            functions,
            bindings,
        })
    }
}
pub(super) fn no_physical_parameters(
    expression: &Atom,
    certificate: &RegularizedFiber,
) -> Result<()> {
    let kinematics = certificate.owner.request().kinematics();
    if kinematics
        .runtime_parameters
        .iter()
        .chain(kinematics.exact_values.keys())
        .any(|s| expression.contains_symbol(*s))
    {
        return Err(unsupported(
            "a physical parameter survived certified fiber binding",
        ));
    }
    Ok(())
}
impl BoundContinuation<'_> {
    pub fn certificate(&self) -> &RegularizedFiber {
        self.certificate
    }
    pub fn expression(&self) -> &Atom {
        &self.expression
    }
    pub fn chart_expressions(&self) -> &[Atom] {
        &self.chart_expressions
    }
    pub fn profiles(&self) -> &[Vec<crate::generation::EndpointProfileRow>] {
        &self.profiles
    }
    pub fn functions(&self) -> &FunctionMap {
        &self.functions
    }
    /// Original runtime parameter order and exact values retained as metadata.
    pub fn bindings(&self) -> &[(Symbol, Rational)] {
        &self.bindings
    }
    pub fn coordinates(&self) -> [Symbol; 1] {
        [self.certificate.unit]
    }
    pub fn regulators(&self) -> [Symbol; 1] {
        [self.certificate.owner.request().input().regulator()]
    }
}
