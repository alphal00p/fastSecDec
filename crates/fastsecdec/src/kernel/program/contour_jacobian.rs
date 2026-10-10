//! Symbolic endpoint expressions with first-order contour-image inputs.
//!
//! All IBP/Taylor/Laurent derivatives have already been taken by Symbolica.
//! Only exact surviving image-partial subtrees are supplied by native image
//! dual arithmetic. Unmatched higher derivatives remain symbolic arithmetic.
#[cfg(test)]
mod tests;
use super::ExactProgram;
use crate::{
    contour::{ContourDefinitions, SymbolicContourJacobian, functions::dynamic::requests::Lookup},
    kernel::CompilationSettings,
};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    evaluate::{EvaluatorComposer, Instruction, Slot},
    id::{Pattern, Replacement},
    symbol,
};

struct Binding {
    variable: Atom,
    face: usize,
    entry: usize,
}

fn restrict(expression: &Atom, parameters: &[Symbol], face: &[(usize, u8)]) -> Atom {
    expression.replace_multiple(face.iter().map(|(axis, value)| {
        Replacement::new(
            Pattern::Literal(Atom::var(parameters[*axis])),
            Pattern::Literal(Atom::num(*value)),
        )
    }))
}

// Native Composer owns dead-code and callback-constant pruning. We inspect
// only its exported input operands; no parallel optimizer or alias resolver.
fn used_inputs(program: &ExactProgram) -> (BTreeSet<usize>, bool) {
    let exported = program.export_instructions();
    let mut used = BTreeSet::new();
    let mut control_flow = !exported.sub_evaluators.is_empty();
    let mut add = |slot: &Slot| {
        if let Slot::Param(index) = slot {
            used.insert(*index);
        }
    };
    for instruction in &exported.instructions {
        match instruction {
            Instruction::Add(_, args, _) | Instruction::Mul(_, args, _) => {
                args.iter().for_each(&mut add)
            }
            Instruction::Pow(_, base, _, _) | Instruction::Assign(_, base) => add(base),
            Instruction::Powf(_, base, power, _) => {
                add(base);
                add(power);
            }
            Instruction::Fun(_, call, _) => call.2.iter().for_each(&mut add),
            Instruction::IfElse(condition, _) => {
                add(condition);
                control_flow = true;
            }
            Instruction::Join(_, condition, yes, no) => {
                add(condition);
                add(yes);
                add(no);
                control_flow = true;
            }
            Instruction::Goto(_) | Instruction::Label(_) => control_flow = true,
        }
    }
    (used, control_flow)
}

pub(super) fn build(
    roots: &[Atom],
    aliases: &[(Atom, Atom)],
    inputs: &[Symbol],
    definitions: &ContourDefinitions,
    retained: &SymbolicContourJacobian,
    lookup: Option<&Lookup>,
    settings: CompilationSettings,
) -> Result<(ExactProgram, usize), String> {
    let plan = &retained.plan;
    let dimension = plan.parameters.len();
    if !(1..=6).contains(&dimension)
        || plan.images.len() != dimension
        || retained.faces.is_empty()
        || !retained.faces.iter().any(Vec::is_empty)
        || retained.faces.iter().any(|face| {
            face.iter()
                .any(|(axis, value)| *axis >= dimension || *value > 1)
                || face.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        })
    {
        return Err("invalid symbolic contour image plan or endpoint faces".into());
    }
    let variables = inputs.iter().copied().map(Atom::var).collect::<Vec<_>>();
    let lower = |expression: &Atom| match lookup {
        Some(lookup) => lookup.lower(
            expression,
            crate::contour::functions::dynamic::requested::symbol(),
        ),
        None => Ok(expression.clone()),
    };
    let entries = plan
        .images
        .iter()
        .flat_map(|image| {
            plan.parameters
                .iter()
                .map(|coordinate| image.derivative(*coordinate))
        })
        .collect::<Vec<_>>();
    let mut candidates = BTreeMap::new();
    for (face_index, face) in retained.faces.iter().enumerate() {
        for (entry_index, entry) in entries.iter().enumerate() {
            let entry = restrict(entry, &plan.parameters, face);
            let entry = lower(&definitions.simplify(&entry, &plan.parameters)?)?;
            // Native scalar parameters shadow Vars/Funs, not arbitrary Atoms.
            // Constants and existing inputs must never become private bindings.
            if entry.as_num_view().is_some() || variables.contains(&entry) {
                continue;
            }
            candidates.entry(entry).or_insert((face_index, entry_index));
        }
    }
    let mut surviving = BTreeSet::new();
    let mut symbols = inputs.iter().copied().collect::<BTreeSet<_>>();
    for expression in roots.iter().chain(aliases.iter().map(|(_, body)| body)) {
        symbols.extend(expression.get_all_symbols(true));
        expression.visitor(&mut |term| {
            if candidates.contains_key::<[u8]>(term.get_data()) {
                surviving.insert(term.to_owned());
            }
            true
        });
    }
    for (handle, _) in aliases {
        symbols.extend(handle.get_all_symbols(true));
    }
    let mut next = 0usize;
    let bindings = surviving
        .into_iter()
        .map(|entry| {
            let variable = loop {
                let symbol = symbol!(format!("fastsecdec::private_contour_partial_{next}"));
                next += 1;
                if symbols.insert(symbol) {
                    break Atom::var(symbol);
                }
            };
            let (face, index) = candidates[&entry];
            (
                entry,
                Binding {
                    variable,
                    face,
                    entry: index,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let rewrite = |expression: &Atom| {
        expression.replace_map(|term, _, out| {
            if let Some(binding) = bindings.get::<[u8]>(term.get_data()) {
                **out = binding.variable.clone();
            }
        })
    };
    let rewritten = roots.iter().map(rewrite).collect::<Vec<_>>();
    let aliases = aliases
        .iter()
        .map(|(handle, body)| (handle.clone(), rewrite(body)))
        .collect::<Vec<_>>();
    let mut body_inputs = variables.clone();
    body_inputs.extend(bindings.values().map(|binding| binding.variable.clone()));
    let functions =
        definitions.function_map(rewritten.iter().chain(aliases.iter().map(|(_, body)| body)))?;
    let body = Atom::evaluator_multiple(&rewritten, &body_inputs)
        .function_map(functions)
        .optimization_settings(settings.native())
        .add_aliases(aliases)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;
    let mut pruner = EvaluatorComposer::new(body_inputs.len());
    let outputs = pruner
        .append(
            &body,
            &(0..body_inputs.len()).map(Slot::Param).collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;
    let body = pruner
        .finish(&outputs, settings.native())
        .map_err(|e| e.to_string())?;
    let (used, control_flow) = used_inputs(&body);
    let active = bindings
        .values()
        .enumerate()
        .filter(|(index, _)| used.contains(&(inputs.len() + index)))
        .collect::<BTreeMap<_, _>>();
    if !active.is_empty() && control_flow {
        return Err("dual contour image inputs in symbolic endpoint control flow are unsupported; use a symbolic contour Jacobian".into());
    }
    let mut composer = EvaluatorComposer::new(inputs.len());
    let constants = Atom::evaluator_multiple(&[Atom::zero(), Atom::one()], &[] as &[Atom])
        .optimization_settings(settings.native())
        .build()
        .map_err(|e| e.to_string())?;
    let constants = composer
        .append(&constants, &[])
        .map_err(|e| e.to_string())?;
    let mut faces = BTreeMap::new();
    for binding in active.values() {
        if faces.contains_key(&binding.face) {
            continue;
        }
        let face = &retained.faces[binding.face];
        let images = plan
            .images
            .iter()
            .map(|image| match lookup {
                Some(lookup) => lookup.lower_image_on_symbolic_face(
                    image,
                    crate::contour::functions::dynamic::requested::symbol(),
                    &plan.parameters,
                    face,
                ),
                None => Ok(image.clone()),
            })
            .collect::<Result<Vec<_>, String>>()?;
        let images = Atom::evaluator_multiple(&images, &variables)
            .function_map(definitions.function_map(images.iter())?)
            .optimization_settings(settings.native())
            .build()
            .map_err(|e| e.to_string())?;
        // The sole Dualizer receives original contour images and degree-one
        // coordinate seeds. Face coordinates are supplied only afterwards.
        let partials = crate::contour::image_partials(images, &plan.parameters, inputs, settings)?;
        let slots = inputs
            .iter()
            .enumerate()
            .map(|(input, symbol)| {
                plan.parameters
                    .iter()
                    .position(|coordinate| coordinate == symbol)
                    .and_then(|axis| face.iter().find(|(index, _)| *index == axis))
                    .map_or(Slot::Param(input), |(_, value)| constants[*value as usize])
            })
            .collect::<Vec<_>>();
        faces.insert(
            binding.face,
            composer
                .append(&partials, &slots)
                .map_err(|e| e.to_string())?,
        );
    }
    let mut slots = (0..inputs.len()).map(Slot::Param).collect::<Vec<_>>();
    slots.extend(bindings.values().enumerate().map(|(index, binding)| {
        if active.contains_key(&index) {
            faces[&binding.face][binding.entry]
        } else {
            constants[0]
        }
    }));
    let outputs = composer.append(&body, &slots).map_err(|e| e.to_string())?;
    composer
        .finish(&outputs, settings.native())
        .map(|program| (program, active.len()))
        .map_err(|e| e.to_string())
}
