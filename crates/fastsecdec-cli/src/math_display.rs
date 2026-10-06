//! The native Symbolica printer owns every mathematical presentation choice.
use std::{borrow::Cow, collections::HashMap};

use symbolica::{
    atom::{Atom, AtomCore},
    printer::{ColorMode, PrintOptions},
};

use crate::terminal_policy::ColorPolicy;

pub(crate) fn atom(expression: &Atom, colors: ColorPolicy, width: usize) -> String {
    in_context(expression, std::slice::from_ref(expression), colors, width)
}

/// Local chart labels may omit namespaces only when Symbolica's own symbol
/// inventory proves that every short name identifies exactly one symbol.
pub(crate) fn in_context(
    expression: &Atom,
    context: &[Atom],
    colors: ColorPolicy,
    width: usize,
) -> String {
    let mut names = HashMap::new();
    let unique = context
        .iter()
        .flat_map(|atom| atom.get_all_symbols(true))
        .all(|symbol| {
            names
                .insert(symbol.get_stripped_name().to_owned(), symbol)
                .is_none_or(|old| old == symbol)
        });
    expression
        .printer(PrintOptions {
            color_mode: if colors.enabled() {
                ColorMode::Always
            } else {
                ColorMode::Never
            },
            max_line_length: Some(width.max(1)),
            multiplication_operator: '·',
            num_exp_as_superscript: true,
            hide_all_namespaces: unique,
            hide_namespace: Some(Cow::Borrowed("fastsecdec")),
            include_attributes: false,
            ..PrintOptions::new()
        })
        .to_string()
}
