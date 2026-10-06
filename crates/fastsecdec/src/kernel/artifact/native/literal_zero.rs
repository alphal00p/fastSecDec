//! Recover literal output facts from Symbolica's decoded native IR from a trusted producer.
use crate::kernel::program::ExactProgram;
use symbolica::evaluate::{Instruction, Slot};

pub(super) fn outputs(program: &ExactProgram) -> Vec<bool> {
    let exported = program.export_instructions();
    let mut zeros = vec![false; exported.output_count];
    // Calls to non-inlined bodies may have their own output layout. This narrow
    // proof never follows bodies or assumes a call writes only its first slot.
    if !exported.sub_evaluators.is_empty() {
        return zeros;
    }
    let mut written = vec![false; exported.output_count];
    for instruction in &exported.instructions {
        let target = match instruction {
            Instruction::Add(target, ..)
            | Instruction::Mul(target, ..)
            | Instruction::Pow(target, ..)
            | Instruction::Powf(target, ..)
            | Instruction::Fun(target, ..)
            | Instruction::Assign(target, ..) => target,
            // Do not perform control-flow analysis or follow computed values.
            Instruction::IfElse(..)
            | Instruction::Goto(..)
            | Instruction::Label(..)
            | Instruction::Join(..) => return vec![false; exported.output_count],
        };
        let Slot::Out(output) = target else {
            continue;
        };
        zeros[*output] = !written[*output]
            && matches!(instruction, Instruction::Assign(_, Slot::Const(index))
                if exported.constants[*index].re.is_zero()
                    && exported.constants[*index].im.is_zero()
                    // Registered constants/functions have rational placeholders;
                    // a zero placeholder is not a literal numerical zero.
                    && !exported.constant_functions.iter().any(|f| f.index == *index));
        written[*output] = true;
    }
    zeros
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{
        atom::{Atom, AtomCore},
        parse,
    };

    #[test]
    fn native_literal_outputs_exclude_deferred_constants_and_small_rationals() {
        let _ = symbolica::transcendental::gamma();
        let roots = [
            Atom::Zero,
            parse!("gamma(1/3)"),
            parse!("literal_zero::x"),
            parse!("1/10^400"),
            parse!("𝑖/10^400"),
            Atom::Zero,
        ];
        let program = Atom::evaluator_multiple(&roots, &[parse!("literal_zero::x")])
            .direct_translation(true)
            .horner_iterations(0)
            .build()
            .unwrap();
        let exported = program.export_instructions();
        assert!(!exported.constant_functions.is_empty());
        assert!(exported.constant_functions.iter().any(|f| {
            exported.constants[f.index].re.is_zero() && exported.constants[f.index].im.is_zero()
        }));
        assert_eq!(outputs(&program), [true, false, false, false, false, true]);
    }
}
