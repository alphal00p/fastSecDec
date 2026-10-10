//! Native symbolic-to-numeric boundary. One exact program owns every numeric path.
mod callbacks;
mod contour_jacobian;
pub(in crate::kernel) use callbacks::{Callback, callbacks};
#[cfg(test)]
mod captured;
#[cfg(test)]
mod runtime_branch;
use super::{CompilationSettings, KernelError, cancellation::Cancellation};
use std::collections::HashMap;
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{ExportedInstructions, ExpressionEvaluator, Instruction},
    function, symbol,
};

pub(super) type ExactProgram = ExpressionEvaluator<Complex<Rational>>;

pub(super) struct SectorProgram {
    pub symbolic_endpoint_contour_partials: Option<usize>,
    pub parameters: Vec<Symbol>,
    pub runtime_parameters: Vec<Symbol>,
    pub exact: ExactProgram,
    pub cancellation: Cancellation,
    pub exact_zero: Vec<bool>,
    pub real_coefficients: Vec<bool>,
}

pub(super) fn build_sector_with_lowering(
    sector: &crate::generation::GeneratedSector,
    runtime_parameters: &[Symbol],
    settings: CompilationSettings,
    lookup: Option<&crate::contour::functions::dynamic::requests::Lookup>,
) -> Result<SectorProgram, KernelError> {
    let cancellation = Cancellation::new(
        sector.cancellation_degree(),
        Some(sector.cancellation_terms().to_vec()),
        sector.dimension(),
    )?
    .with_endpoint_profiles(sector.endpoint_profiles().to_vec())?;
    if let Some(deferred) = &sector.deferred {
        use crate::generation::numerical_dual::native;
        let exact = if let Some(lookup) = lookup {
            native::build_with_lowering(
                deferred,
                runtime_parameters,
                settings,
                &mut |body, coordinates| {
                    let mut face = Vec::new();
                    for (axis, coordinate) in coordinates.iter().enumerate() {
                        match coordinate {
                            native::Coordinate::Variable(index) if *index == axis => {}
                            native::Coordinate::Zero => face.push((axis, 0)),
                            native::Coordinate::One => face.push((axis, 1)),
                            native::Coordinate::Variable(_) => {
                                return Err(KernelError::Compilation(
                                    "nonidentity dynamic request coordinate projection".into(),
                                ));
                            }
                        }
                    }
                    lookup
                        .lower_on_face(
                            body,
                            crate::contour::functions::dynamic::requested::symbol(),
                            &deferred.parameters,
                            &face,
                        )
                        .map_err(KernelError::Compilation)
                },
            )?
        } else {
            native::build(deferred, runtime_parameters, settings)?
        };
        Ok(SectorProgram {
            symbolic_endpoint_contour_partials: None,
            parameters: sector.parameters().to_vec(),
            runtime_parameters: runtime_parameters.to_vec(),
            exact,
            cancellation,
            exact_zero: sector
                .aliased_coefficients()
                .iter()
                .map(|value| value.get_root().is_zero())
                .collect(),
            // Source factors can need complex intermediates even when all
            // literal coefficients and formal recipe placeholders look real.
            real_coefficients: vec![false; sector.aliased_coefficients().len()],
        })
    } else {
        build_with_lowering(
            sector.parameters().to_vec(),
            runtime_parameters,
            sector.aliased_coefficients(),
            cancellation,
            settings,
            lookup,
            sector.contour_definitions(),
            sector.symbolic_jacobian.as_ref(),
        )
    }
}

/// Coordinate-image maps are flat and shared within one generated vector.
/// Empty maps are allowed for exact zero padding or plain legacy expressions.
#[cfg(test)]
pub(super) fn build(
    parameters: Vec<Symbol>,
    coefficients: &[AliasedAtom],
    cancellation: Cancellation,
) -> Result<SectorProgram, KernelError> {
    build_with_parameters(parameters, &[], coefficients, cancellation)
}

#[cfg(test)]
pub(super) fn build_with_parameters(
    parameters: Vec<Symbol>,
    runtime_parameters: &[Symbol],
    coefficients: &[AliasedAtom],
    cancellation: Cancellation,
) -> Result<SectorProgram, KernelError> {
    build_with_settings(
        parameters,
        runtime_parameters,
        coefficients,
        cancellation,
        CompilationSettings::default(),
    )
}

#[cfg(test)]
pub(super) fn build_with_settings(
    parameters: Vec<Symbol>,
    runtime_parameters: &[Symbol],
    coefficients: &[AliasedAtom],
    cancellation: Cancellation,
    settings: CompilationSettings,
) -> Result<SectorProgram, KernelError> {
    build_with_lowering(
        parameters,
        runtime_parameters,
        coefficients,
        cancellation,
        settings,
        None,
        &crate::contour::ContourDefinitions::default(),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_with_lowering(
    parameters: Vec<Symbol>,
    runtime_parameters: &[Symbol],
    coefficients: &[AliasedAtom],
    cancellation: Cancellation,
    settings: CompilationSettings,
    lookup: Option<&crate::contour::functions::dynamic::requests::Lookup>,
    definitions: &crate::contour::ContourDefinitions,
    symbolic_jacobian: Option<&crate::contour::SymbolicContourJacobian>,
) -> Result<SectorProgram, KernelError> {
    settings.validate()?;
    let mut seen = std::collections::HashSet::new();
    if parameters
        .iter()
        .chain(runtime_parameters)
        .any(|symbol| !seen.insert(*symbol))
    {
        return Err(KernelError::Compilation(
            "duplicate coordinate or runtime parameter".into(),
        ));
    }
    let aliases = coefficients
        .iter()
        .map(AliasedAtom::get_aliases)
        .find(|aliases| !aliases.is_empty());
    if let Some(aliases) = aliases
        && coefficients.iter().any(|coefficient| {
            !coefficient.get_aliases().is_empty() && coefficient.get_aliases() != aliases
        })
    {
        return Err(KernelError::Compilation(
            "coefficient vector has inconsistent native alias definitions".into(),
        ));
    }
    let original_roots = coefficients
        .iter()
        .map(AliasedAtom::get_root)
        .collect::<Vec<_>>();
    let lowered = lookup
        .map(|lookup| {
            original_roots
                .iter()
                .map(|root| {
                    lookup
                        .lower(
                            root,
                            crate::contour::functions::dynamic::requested::symbol(),
                        )
                        .map_err(KernelError::Compilation)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let roots = lowered
        .as_ref()
        .map(|roots| roots.iter().collect::<Vec<_>>())
        .unwrap_or_else(|| original_roots.clone());
    let variables = parameters
        .iter()
        .chain(runtime_parameters)
        .map(|p| Atom::var(*p))
        .collect::<Vec<_>>();
    let mut ordered_aliases = aliases
        .into_iter()
        .flat_map(|aliases| aliases.iter())
        .map(|(handle, body)| {
            let body = match lookup {
                Some(lookup) => lookup
                    .lower(
                        body,
                        crate::contour::functions::dynamic::requested::symbol(),
                    )
                    .map_err(KernelError::Compilation)?,
                None => body.clone(),
            };
            Ok((handle.clone(), body))
        })
        .collect::<Result<Vec<_>, KernelError>>()?;
    ordered_aliases.sort_by(|a, b| a.0.cmp(&b.0));
    let (exact, symbolic_endpoint_contour_partials) = if let Some(plan) = symbolic_jacobian {
        if plan.plan.parameters != parameters {
            return Err(KernelError::Compilation(
                "symbolic contour image coordinate mismatch".into(),
            ));
        }
        let (program, partials) = contour_jacobian::build(
            &roots.iter().map(|root| (*root).clone()).collect::<Vec<_>>(),
            &ordered_aliases,
            &parameters
                .iter()
                .chain(runtime_parameters)
                .copied()
                .collect::<Vec<_>>(),
            definitions,
            plan,
            lookup,
            settings,
        )
        .map_err(KernelError::Compilation)?;
        (program, Some(partials))
    } else {
        let functions = definitions
            .function_map(
                roots
                    .iter()
                    .copied()
                    .chain(ordered_aliases.iter().map(|(_, body)| body)),
            )
            .map_err(KernelError::Compilation)?;
        let program = Atom::evaluator_multiple(&roots, &variables)
            .function_map(functions)
            .optimization_settings(settings.native())
            .add_aliases(ordered_aliases)
            .map_err(|error| KernelError::Compilation(error.to_string()))?
            .build()
            .map_err(|error| KernelError::Compilation(error.to_string()))?;
        (program, None)
    };
    let real_coefficients = coefficients
        .iter()
        .map(|coefficient| is_real_with_parameters(coefficient, &parameters, runtime_parameters))
        .collect();
    Ok(SectorProgram {
        symbolic_endpoint_contour_partials,
        parameters,
        runtime_parameters: runtime_parameters.to_vec(),
        exact,
        cancellation,
        exact_zero: original_roots.iter().map(|root| root.is_zero()).collect(),
        real_coefficients,
    })
}

/// Real literal coefficients alone do not certify real-valued square roots,
/// logarithms or fractional powers. Ask Symbolica under the real input domain.
#[cfg(test)]
pub(super) fn is_real(coefficient: &AliasedAtom) -> bool {
    is_real_with_parameters(coefficient, &[], &[])
}

pub(super) fn is_real_with_parameters(
    coefficient: &AliasedAtom,
    coordinates: &[Symbol],
    runtime: &[Symbol],
) -> bool {
    let mut assumptions = RealInputs::with_coordinates(coordinates, runtime);
    let first_alias = assumptions.replacements.len();
    let mut unresolved = coefficient
        .get_aliases()
        .iter()
        .enumerate()
        .map(|(index, (handle, body))| {
            let index = first_alias + index;
            // Strip any attributes of an opaque handle: its body, including
            // indirect dependencies, determines whether it is real.
            assumptions
                .replacements
                .insert(handle.clone(), proxy(index, false));
            (handle, body, index)
        })
        .collect::<Vec<_>>();
    loop {
        let before = unresolved.len();
        unresolved.retain(|(handle, body, index)| {
            let body = assumptions.apply(body);
            if real_operations(&body) {
                let replacement = if body.is_positive().is_true() {
                    positive_proxy(*index)
                } else {
                    proxy(*index, true)
                };
                assumptions
                    .replacements
                    .insert((*handle).clone(), replacement);
                false
            } else {
                true
            }
        });
        if before == unresolved.len() {
            break;
        }
    }
    // Retain the compact alias graph. Inconclusive bodies stay opaque and
    // select the complex evaluator; no coefficient is expanded to prove it.
    assumptions.is_real(coefficient.get_root())
}

pub(crate) fn is_real_expression(expression: &Atom, inputs: &[Symbol]) -> bool {
    RealInputs::new(inputs.iter()).is_real(expression)
}

#[cfg(test)]
pub(super) fn is_real_coordinate_expression(expression: &Atom, coordinates: &[Symbol]) -> bool {
    RealInputs::with_coordinates(coordinates, &[]).is_real(expression)
}

struct RealInputs {
    replacements: HashMap<Atom, Atom>,
}

impl RealInputs {
    fn new<'a>(inputs: impl Iterator<Item = &'a Symbol>) -> Self {
        Self {
            replacements: inputs
                .enumerate()
                .map(|(index, input)| (Atom::var(*input), proxy(index, true)))
                .collect(),
        }
    }

    fn with_coordinates(coordinates: &[Symbol], runtime: &[Symbol]) -> Self {
        let mut assumptions = Self::new(coordinates.iter().chain(runtime));
        // Numerical coordinates are confined to the closed unit cube. They
        // are positive in its interior; a logarithm at zero remains a native
        // nonfinite endpoint error rather than a new complex branch.
        for (index, coordinate) in coordinates.iter().enumerate() {
            assumptions
                .replacements
                .insert(Atom::var(*coordinate), positive_proxy(index));
        }
        assumptions
    }

    fn is_real(&self, expression: &Atom) -> bool {
        real_operations(&self.apply(expression))
    }

    fn apply(&self, expression: &Atom) -> Atom {
        expression.replace_map(|atom, _, out| {
            if let Some(replacement) = self.replacements.get::<[u8]>(atom.get_data()) {
                out.set_from_view(&replacement.as_view());
            } else if let Some(positive) = fixed_native_constant(atom) {
                let symbol = if positive {
                    symbol!("fastsecdec::kernel_realness::positive_fixed_constant"; Positive)
                } else {
                    symbol!("fastsecdec::kernel_realness::real_fixed_constant"; Real)
                };
                // Retain the exact original identity: unrelated constants must
                // not cancel just because both are known to be real.
                out.set_from_view(&function!(symbol, atom.to_owned()).as_view());
            }
        })
    }
}

fn real_operations(expression: &Atom) -> bool {
    let proxies = [
        symbol!("fastsecdec::kernel_realness::real_input"; Real),
        symbol!("fastsecdec::kernel_realness::positive_coordinate"; Positive),
        symbol!("fastsecdec::kernel_realness::real_fixed_constant"; Real),
        symbol!("fastsecdec::kernel_realness::positive_fixed_constant"; Positive),
    ];
    let mut real = true;
    expression.visitor(&mut |atom| {
        if !real || !atom.is_real().is_true() {
            real = false;
            return false;
        }
        // A real result such as abs(sqrt(-p)) can still require a complex
        // intermediate. Prove each actual operation; only already-proven
        // inputs, alias bodies and immutable constants remain opaque.
        !matches!(atom, AtomView::Fun(function) if proxies.contains(&function.get_symbol()))
    });
    real
}

fn fixed_native_constant(atom: AtomView<'_>) -> Option<bool> {
    match atom {
        AtomView::Var(variable)
            if variable.get_symbol() == symbolica::transcendental::euler_gamma() =>
        {
            // The native owner constructs Constant::Euler with an exact zero
            // imaginary part, but its symbol currently lacks Real. Euler's
            // immutable constant is positive; no callback value is sampled.
            Some(true)
        }
        AtomView::Fun(function)
            if function.get_symbol() == symbolica::transcendental::polygamma()
                && function.get_nargs() == 2 =>
        {
            let mut arguments = function.iter();
            let order = Rational::try_from(arguments.next()?).ok()?;
            let argument = Rational::try_from(arguments.next()?).ok()?;
            // The canonical polygamma is real for an exact nonnegative integer
            // order at a positive real argument. This domain excludes its
            // poles and all unconstrained runtime or complex arguments. Its
            // sign is not fixed (in particular digamma need not be positive).
            (order.is_integer()
                && !order.is_negative()
                && !argument.is_negative()
                && !argument.is_zero())
            .then_some(false)
        }
        _ => None,
    }
}

fn positive_proxy(index: usize) -> Atom {
    function!(
        symbol!("fastsecdec::kernel_realness::positive_coordinate"; Positive),
        Atom::num(index as i64)
    )
}

fn proxy(index: usize, real: bool) -> Atom {
    let symbol = if real {
        symbol!("fastsecdec::kernel_realness::real_input"; Real)
    } else {
        symbol!("fastsecdec::kernel_realness::unknown_alias")
    };
    // Distinct inputs must remain distinct. Replacing x and y by one symbol
    // could cancel x-y and falsely certify sqrt(x-y) as real.
    function!(symbol, Atom::num(index as i64))
}

/// Historical real layouts did not prove branch domains. Read native exported
/// instruction kinds only; do not reconstruct or interpret the saved program.
pub(super) fn legacy_real_branch(program: &ExactProgram) -> bool {
    fn ambiguous(instructions: &ExportedInstructions<Complex<Rational>>) -> bool {
        instructions
            .instructions
            .iter()
            .any(|instruction| match instruction {
                Instruction::Powf(..) => true,
                Instruction::Fun(_, function, _) => {
                    function.0 == Symbol::SQRT || function.0 == Symbol::LOG
                }
                _ => false,
            })
            || instructions
                .sub_evaluators
                .iter()
                .any(|body| ambiguous(&body.instructions))
    }
    ambiguous(&program.export_instructions())
}

pub(super) fn encode(program: &ExactProgram) -> Result<Vec<u8>, KernelError> {
    bincode::serde::encode_to_vec(program, bincode::config::standard())
        .map_err(|error| KernelError::Artifact(format!("native evaluator encoding: {error}")))
}

pub(super) fn decode(bytes: &[u8]) -> Result<ExactProgram, KernelError> {
    // This is Symbolica's native cache codec, not an untrusted IR validator.
    // Only decode programs produced by a trusted, compatible native builder.
    // Native callbacks for external fixed Gamma/polygamma constants must exist
    // before the portable program imports their symbol identities.
    let _ = symbolica::transcendental::gamma();
    crate::contour::functions::register();
    let (program, consumed) =
        bincode::serde::borrow_decode_from_slice(bytes, bincode::config::standard()).map_err(
            |error| KernelError::Artifact(format!("native evaluator decoding: {error}")),
        )?;
    if consumed != bytes.len() {
        return Err(KernelError::Artifact(
            "trailing native evaluator bytes".into(),
        ));
    }
    Ok(program)
}
