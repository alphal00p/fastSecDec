//! Native callback metadata for preparation and saved dependency ownership.
//! Follow only alias bodies called by the retained instruction stream. This
//! does not interpret, transform, or optimize the owner's arithmetic.
use super::ExactProgram;
use std::{collections::HashSet, sync::Arc};
use symbolica::{
    atom::{Atom, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{ExportedInstructions, Instruction},
};

pub(in crate::kernel) struct Callback {
    pub symbol: Symbol,
    pub tags: Vec<Atom>,
    pub fixed_args: Option<Vec<Complex<Rational>>>,
}

pub(in crate::kernel) fn callbacks(exact: &ExactProgram) -> Result<Vec<Callback>, String> {
    let mut result = Vec::new();
    collect(
        &exact.export_instructions(),
        &mut HashSet::new(),
        &mut result,
    )?;
    Ok(result)
}

fn tags(values: &[String]) -> Result<Vec<Atom>, String> {
    values
        .iter()
        .map(|value| {
            Atom::parse(value, "fastsecdec::artifact", Default::default())
                .map_err(|error| format!("Native evaluator callback tag: {error}"))
        })
        .collect()
}

fn collect(
    exported: &ExportedInstructions<Complex<Rational>>,
    visited: &mut HashSet<usize>,
    result: &mut Vec<Callback>,
) -> Result<(), String> {
    // Mapping resolves these constants when this native program is prepared,
    // even if a later instruction does not read their slots.
    for constant in &exported.constant_functions {
        result.push(Callback {
            symbol: constant.symbol,
            tags: tags(&constant.tags)?,
            fixed_args: Some(constant.fixed_args.clone()),
        });
    }
    for instruction in &exported.instructions {
        let Instruction::Fun(_, call, _) = instruction else {
            continue;
        };
        let (symbol, tag_names, _) = &**call;
        if let Some(body) = exported
            .sub_evaluators
            .iter()
            .find(|body| body.symbol == *symbol && body.tags == *tag_names)
        {
            if visited.insert(Arc::as_ptr(&body.instructions) as usize) {
                collect(&body.instructions, visited, result)?;
            }
        } else if ![
            Symbol::EXP,
            Symbol::LOG,
            Symbol::SIN,
            Symbol::COS,
            Symbol::SQRT,
            Symbol::ABS,
            Symbol::CONJ,
        ]
        .contains(symbol)
        {
            result.push(Callback {
                symbol: *symbol,
                tags: tags(tag_names)?,
                fixed_args: None,
            });
        }
    }
    Ok(())
}
