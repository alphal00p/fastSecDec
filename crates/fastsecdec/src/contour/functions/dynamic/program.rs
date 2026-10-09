//! Saved owner evaluator for the coefficient-only radius equation.
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, Weak},
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{ExpressionEvaluator, Instruction},
    symbol,
};

type Exact = ExpressionEvaluator<Complex<Rational>>;
const TAG_PREFIX: &str = "fastsecdec::contour::dynamic::root_program_";
static ACTIVE: LazyLock<Mutex<HashMap<String, Weak<Program>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Immutable helper ownership is bounded by the resident native callbacks.
/// The routing registry contains weak references and is pruned on admission.
#[derive(Clone)]
pub(crate) struct RootProgram(pub(super) Arc<Program>);

pub(super) struct Program {
    pub coefficient_count: usize,
    pub digest: String,
    pub exact: Exact,
    bytes: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Saved {
    version: u32,
    coefficient_count: usize,
    exact: Exact,
}

impl RootProgram {
    /// Called during original generation, never during numerical evaluation or
    /// restoration. Symbolica owns derivatives and evaluator optimization.
    pub(crate) fn build(coefficient_count: usize) -> Result<Self, String> {
        if coefficient_count == 0 || coefficient_count > (i32::MAX as usize / 2) {
            return Err("invalid dynamic root coefficient count".into());
        }
        let root = symbol!("fastsecdec::contour::dynamic::helper_root");
        let mut inputs = vec![Atom::var(root)];
        let mut level = Atom::num(-1);
        for index in 0..coefficient_count {
            let coefficient = Atom::var(symbol!(&format!(
                "fastsecdec::contour::dynamic::helper_coefficient_{index}"
            )));
            level += &coefficient * Atom::var(root).pow(2 * (index as i64 + 1));
            inputs.push(coefficient);
        }
        let derivative = level.derivative(root);
        let exact = Atom::evaluator_multiple(&[level, derivative], &inputs)
            .build()
            .map_err(|error| error.to_string())?;
        let saved = Saved {
            version: 1,
            coefficient_count,
            exact,
        };
        let bytes = bincode::serde::encode_to_vec(&saved, bincode::config::standard())
            .map_err(|error| error.to_string())?;
        Self::admit(saved, bytes)
    }

    /// Decode the existing native program; no expression or optimizer is run.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let (saved, read) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|error| error.to_string())?;
        if read != bytes.len() {
            return Err("trailing bytes in dynamic root helper program".into());
        }
        Self::admit(saved, bytes.to_vec())
    }

    fn admit(saved: Saved, bytes: Vec<u8>) -> Result<Self, String> {
        if saved.version != 1
            || saved.coefficient_count == 0
            || saved.coefficient_count > (i32::MAX as usize / 2)
            || saved.coefficient_count.checked_add(1) != Some(saved.exact.get_input_len())
            || saved.exact.get_output_len() != 2
        {
            return Err("invalid dynamic root helper schema".into());
        }
        let code = saved.exact.export_instructions();
        if !code.constant_functions.is_empty()
            || !code.sub_evaluators.is_empty()
            || code.constants.iter().any(|constant| constant.im != 0)
            || code.instructions.iter().any(|instruction| {
                !matches!(
                    instruction,
                    Instruction::Add(..)
                        | Instruction::Mul(..)
                        | Instruction::Assign(..)
                        | Instruction::Pow(_, _, 0.., _)
                )
            })
        {
            return Err("dynamic root helper must contain only real polynomial arithmetic".into());
        }
        let digest = blake3::hash(&bytes).to_hex().to_string();
        let mut active = ACTIVE
            .lock()
            .map_err(|_| "dynamic helper registry poisoned")?;
        active.retain(|_, owner| owner.strong_count() != 0);
        if let Some(existing) = active.get(&digest).and_then(Weak::upgrade) {
            return Ok(Self(existing));
        }
        let program = Arc::new(Program {
            coefficient_count: saved.coefficient_count,
            digest: digest.clone(),
            exact: saved.exact,
            bytes,
        });
        active.insert(digest, Arc::downgrade(&program));
        Ok(Self(program))
    }

    pub(crate) fn coefficient_count(&self) -> usize {
        self.0.coefficient_count
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0.bytes
    }
    pub(crate) fn digest(&self) -> &str {
        &self.0.digest
    }
    pub(crate) fn tag(&self) -> Symbol {
        symbol!(&format!("{TAG_PREFIX}{}", self.0.digest))
    }
}

pub(super) fn resolve(tags: &[AtomView<'_>]) -> Result<Arc<Program>, String> {
    if tags.len() != 2 {
        return Err("dynamic root callback requires arity and program identity tags".into());
    }
    let Some(tag) = tags[1].as_var_view() else {
        return Err("dynamic root helper identity must be a native symbol".into());
    };
    let symbol = tag.get_symbol();
    let name = symbol.get_name();
    let Some(digest) = name.strip_prefix(TAG_PREFIX) else {
        return Err("invalid dynamic root helper identity namespace".into());
    };
    let active = ACTIVE
        .lock()
        .map_err(|_| "dynamic helper registry poisoned")?;
    let owner = active
        .get(digest)
        .and_then(Weak::upgrade)
        .ok_or("dynamic root helper was not restored before its evaluator")?;
    if tags[0] != Atom::num(owner.coefficient_count).as_view() {
        return Err("dynamic root helper arity differs from its callback".into());
    }
    Ok(owner)
}
