//! Constant-fiber compiler input. The proof remains owned by the source fiber.
use super::*;

pub struct BoundContinuation<'a> {
    generation_options: GenerationOptions,
    certificate: &'a RegularizedFiber,
    expression: Atom,
    chart_expressions: Vec<Atom>,
    profiles: Vec<Vec<crate::generation::EndpointProfileRow>>,
    functions: FunctionMap,
    definitions: Vec<BoundDefinition>,
    derivative_order: usize,
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
        let definitions =
            self.certificate
                .definitions(self.derivative_order, true, &mut observer)?;
        let functions = register_definitions(&definitions)?;
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
            generation_options: self.generation_options.clone(),
            certificate: self.certificate,
            expression,
            chart_expressions,
            profiles: self.profiles.clone(),
            functions,
            definitions,
            derivative_order: self.derivative_order,
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
    pub(crate) fn generation_options(&self) -> &GenerationOptions {
        &self.generation_options
    }
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

impl BoundContinuation<'_> {
    pub(crate) fn definitions(&self) -> &[BoundDefinition] {
        &self.definitions
    }

    /// Materialize ONLY this owner's face-restricted numerator calls in a
    /// coordinate-independent Laurent coefficient. This is not a numerical
    /// evaluator, a general FunctionMap expander, or a publication capability.
    /// A constant n(t) is conservatively left stochastic by the caller.
    pub fn materialize_exact(&self, expression: &Atom) -> Result<Atom> {
        use symbolica::atom::AtomView;
        let certificate = self.certificate;
        let request = certificate.owner.request();
        let disallowed = |a: &Atom| {
            a.contains_symbol(certificate.unit)
                || a.contains_symbol(request.input().regulator())
                || request
                    .input()
                    .parameters()
                    .iter()
                    .any(|s| a.contains_symbol(*s))
                || request
                    .domain()
                    .coordinates()
                    .iter()
                    .any(|s| a.contains_symbol(*s))
        };
        no_physical_parameters(expression, certificate)?;
        if disallowed(expression) {
            return Err(unsupported(
                "exact coefficient still depends on a coordinate or regulator",
            ));
        }
        let mut error = None;
        let mut bodies = BTreeMap::<(usize, usize), Atom>::new();
        let result = expression.replace_map_bottom_up(|node, _, out| {
            if error.is_some() {
                return;
            }
            let AtomView::Fun(call) = node else {
                return;
            };
            let parsed = (|| -> Result<Option<(usize, usize, Atom)>> {
                if call.get_symbol() == certificate.numerator {
                    if call.get_nargs() != 2 {
                        return Err(invalid("owned numerator call arity"));
                    }
                    let tag =
                        usize::try_from(call.get(0)).map_err(|_| invalid("owned numerator tag"))?;
                    Ok(Some((tag, 0, call.get(1).to_owned())))
                } else if call.get_symbol() == Symbol::DERIVATIVE
                    && call
                        .iter()
                        .any(|a| a.contains_symbol(certificate.numerator))
                {
                    if call.get_nargs() != 5
                        || call.get(2) != Atom::var(certificate.numerator).as_view()
                        || !call.get(0).is_zero()
                    {
                        return Err(invalid("owned numerator derivative tags/arity"));
                    }
                    let order = usize::try_from(call.get(1))
                        .map_err(|_| invalid("owned numerator derivative order"))?;
                    if order == 0 || order > self.derivative_order {
                        return Err(invalid("owned derivative outside registered orders"));
                    }
                    let tag = usize::try_from(call.get(3))
                        .map_err(|_| invalid("owned numerator derivative term tag"))?;
                    Ok(Some((tag, order, call.get(4).to_owned())))
                } else {
                    Ok(None)
                }
            })();
            let (term, order, argument) = match parsed {
                Ok(Some(p)) => p,
                Ok(None) => return,
                Err(e) => {
                    error = Some(e);
                    return;
                }
            };
            if term >= certificate.numerator_bodies.len() {
                error = Some(invalid("owned numerator term outside source"));
                return;
            }
            let body = bodies.entry((term, order)).or_insert_with(|| {
                // Definitions came from exactly the same bound registration
                // loop; no second symbolic derivative engine is constructed.
                self.definitions()
                    .iter()
                    .find(|d| {
                        d.derivative_order.unwrap_or(0) == order
                            && if order == 0 {
                                d.tags[0] == Atom::num(term)
                            } else {
                                d.tags[3] == Atom::num(term)
                            }
                    })
                    .expect("private complete bound definition inventory")
                    .body
                    .clone()
            });
            **out = body
                .replace(Atom::var(request.domain().coordinates()[0]))
                .with(argument);
        });
        if let Some(error) = error {
            return Err(error);
        }
        no_physical_parameters(&result, certificate)?;
        if disallowed(&result) || result.contains_symbol(certificate.numerator) {
            return Err(unsupported(
                "exact materialization retained a private input or function",
            ));
        }
        let mut free = false;
        result.visitor(&mut |node| {
            if let AtomView::Var(v) = node {
                let s = v.get_symbol();
                free |= s != Symbol::PI
                    && s != Symbol::E
                    && !s
                        .get_evaluation_info()
                        .is_some_and(|e| e.has_constant_evaluator());
            }
            !free
        });
        if free {
            return Err(unsupported(
                "exact materialization retained an unbound symbol",
            ));
        }
        Ok(result)
    }
}
