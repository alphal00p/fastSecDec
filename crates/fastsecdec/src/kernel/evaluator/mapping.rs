//! Admission for upstream coefficient mapping, which assumes that each external
//! callback and constant supports the requested numeric domain. This inspects
//! native callback metadata only; it does not validate or rewrite native IR.
use super::super::program::ExactProgram;
use std::{collections::HashSet, sync::Arc};
use symbolica::{
    atom::{Atom, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{EvaluationDomain, ExportedInstructions, ExpressionEvaluator, Instruction},
};

struct Requirement {
    symbol: Symbol,
    tags: Vec<Atom>,
    // None is a point-dependent callback; Some([]) is a registered constant.
    fixed_args: Option<Vec<Complex<Rational>>>,
}

pub(in crate::kernel) struct MappingRequirements(Vec<Requirement>);

impl MappingRequirements {
    /// Called while constructing the kernel, before caller-owned workers start.
    /// In particular, parsing canonical native tags never happens per sample.
    pub(in crate::kernel) fn new(exact: &ExactProgram) -> Result<Arc<Self>, String> {
        let mut requirements = Self(Vec::new());
        requirements.collect(&exact.export_instructions(), &mut HashSet::new())?;
        Ok(Arc::new(requirements))
    }

    fn collect(
        &mut self,
        exported: &ExportedInstructions<Complex<Rational>>,
        visited: &mut HashSet<usize>,
    ) -> Result<(), String> {
        for constant in &exported.constant_functions {
            self.0.push(Requirement {
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
            if [
                Symbol::EXP,
                Symbol::LOG,
                Symbol::SIN,
                Symbol::COS,
                Symbol::SQRT,
                Symbol::ABS,
                Symbol::CONJ,
            ]
            .contains(symbol)
                || exported
                    .sub_evaluators
                    .iter()
                    .any(|body| body.symbol == *symbol && body.tags == *tag_names)
            {
                continue;
            }
            self.0.push(Requirement {
                symbol: *symbol,
                tags: tags(tag_names)?,
                fixed_args: None,
            });
        }
        for body in &exported.sub_evaluators {
            if visited.insert(Arc::as_ptr(&body.instructions) as usize) {
                self.collect(&body.instructions, visited)?;
            }
        }
        Ok(())
    }

    pub(in crate::kernel) fn map<T: EvaluationDomain>(
        &self,
        exact: &ExactProgram,
        coefficient: impl Fn(&Complex<Rational>) -> T,
        bits: u32,
    ) -> Result<ExpressionEvaluator<T>, String> {
        for requirement in &self.0 {
            let info = requirement.symbol.get_evaluation_info().ok_or_else(|| {
                format!(
                    "External function '{}' has no evaluation info",
                    requirement.symbol
                )
            })?;
            let tags = requirement
                .tags
                .iter()
                .map(Atom::as_view)
                .collect::<Vec<_>>();
            if let Some(args) = &requirement.fixed_args {
                if args.is_empty() {
                    // Reuse the owner's fallible constant/domain conversion.
                    // Symbolica caches untagged 53-bit constants internally.
                    T::try_from_complex_float(info.evaluate_constant(&tags, bits)?)?;
                    continue;
                }
                for value in args {
                    T::try_from_complex_float(Complex::new(
                        value.re.to_multi_prec_float(bits),
                        value.im.to_multi_prec_float(bits),
                    ))?;
                }
            }
            if T::resolve_function(&tags, info).is_none() {
                return Err(format!(
                    "External function '{}' has no implementation for the requested numeric domain",
                    requirement.symbol
                ));
            }
        }
        // No generic constant fallback: fixed functions must have their own
        // implementation in T. User callbacks retain Symbolica's own contract.
        Ok(exact.clone().map_coeff_with_prec(&coefficient, bits))
    }
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
