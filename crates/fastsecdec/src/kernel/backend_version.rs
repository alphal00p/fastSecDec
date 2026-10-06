//! Query the linked backend through its public API, never a consumer lockfile.
use std::sync::LazyLock;

use symjit::{Application, CompilerType, Composer, Config, Slot, Translator};

fn version_application() -> Application {
    // SymJIT currently exposes its version on Application, not as a Rust
    // constant. A one-constant bytecode application needs no native JIT or
    // evaluation. Build it only once per process and discard it after querying.
    let config = Config::new(CompilerType::ByteCode, 0)
        .expect("SymJIT must support its public bytecode configuration");
    let mut translator = Translator::new(config);
    let constant = translator
        .append_constant(symjit::Complex::new(0.0, 0.0))
        .expect("SymJIT must accept a finite literal constant");
    translator
        .append_assign(&Slot::Out(0), &Slot::Const(constant))
        .expect("SymJIT must accept a literal output assignment");
    translator
        .compile()
        .expect("SymJIT must construct a literal bytecode application")
}

/// Version code reported by the linked SymJIT `Application::measure("version")`.
///
/// This is the backend's numeric identifier, not a reconstructed semantic
/// version. The first call constructs one literal bytecode application; no
/// machine code or numerical samples are evaluated, and later calls are cached.
pub fn symjit_version_code() -> usize {
    static VERSION: LazyLock<usize> = LazyLock::new(|| version_application().measure("version"));
    *VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reported_version_comes_from_linked_bytecode_backend_without_machine_code() {
        let application = version_application();
        assert!(application.compiled.is_none());
        assert!(application.compiled_simd.is_none());
        assert!(application.compiled_fast.is_none());
        let actual = application.measure("version");
        assert_ne!(actual, 0);
        assert_eq!(symjit_version_code(), actual);
        assert_eq!(symjit_version_code(), actual);
    }
}
