# Literal expansion variables in native function Series

Status (2026-10-06): upstream in Symbolica commit `6589d0c`, included in the
selected public `community` revision `473b4b8dbc2f9bff8658a047196ba0877238bf9e`.
The delivery patch and bootstrap application have been removed. The standalone
[author-facing reproduction](../../mre/symbolica-literal-series-variable/README.md)
retains its original patch as historical evidence; it is not applied to builds.

## Historical rationale and validation

The native generic-function Series fallback converted its expansion
`Indeterminate` into a replacement pattern. For a valid symbol ending in `_`,
that conversion creates a wildcard. Substituting the expansion point therefore
replaced the whole function expression instead of only its expansion variable.

A minimal executable against the previous release build demonstrated:

```text
Gamma(eps).series(eps, 0, 0)   = -gamma_E + 1/eps
Gamma(eps_).series(eps_, 0, 0) = 0
```

The second result had no Series terms. This invalidated the first two ignored
formal-function proof attempts: their complete-vector comparisons were empty,
not evidence of evaluator compatibility. Their original outputs remain under
`output/diagnostics/formal-functions/small-{1,2}`. No defect in native
FunctionMap complex-constant storage was established.

The patch changes only the two substitutions in the generic-function fallback
of `src/derivative.rs`. Both the expansion indeterminate and the expansion
point are literal native Atoms. Native differentiation, Series arithmetic,
Gamma regularization and truncation remain unchanged. The adjacent native
regression compares ordinary and underscore-suffixed names for Gamma and an
unknown composed function at expansion points zero and one, and explicitly
checks the nonzero Gamma pole and finite coefficient.

FastSecDec's `tests/gamma_regulator.rs` additionally generates and compiles
`Gamma(eps_) * integral_0^1 x^(eps_-1) dx`. The full Laurent vector through
epsilon order one must equal the independent analytic vector of
`Gamma(eps_)/eps_`, including both poles and both regular coefficients. It may
not silently become a zero integral. This is a dependency correction; no
FastSecDec Series workaround or symbol-name restriction was added.

The base remains Symbolica revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, package version 3.0.1. Before this
fourth local source patch its recorded source state was
`dirty:0d3d7ab734cb4be017018211aa7085f8f2a1d00b95621b507e457a18804a2669`.
The previous release Symbolica rlib had SHA-256
`36b8118e14f4abccdc3ce488c61585cab1f13ad761fba097b98b37380659ffaa`.
The minimal reproducer, old and changed source hashes and isolated patch hash
are retained in `output/symbolica-series-literal-patch-evidence.json`; existing
frozen benchmark identities are not rewritten to claim this patch.

Independent source review accepted the two literal substitutions and regression
scope. Both focused gates passed:

```text
cargo test -p symbolica --lib series_expansion_variables_are_literal --locked -- --test-threads=1
cargo test -p fastsecdec --test gamma_regulator --locked -- --test-threads=1
```

Each command ran one test successfully. Logs are
`output/symbolica-series-literal-native-tests.log` and
`output/gamma-regulator-focused-tests.log`. The rebuilt normal dependency is
`target/debug/deps/libsymbolica-1f655858224cb588.rlib`, dependency optimization
level two, SHA-256
`dda215a39b38358772677beed211e773c9b028f33f0251dc1072135fe37f2e51`.
The focused FastSecDec executable linked that corrected library. Existing frozen
release binaries were not rebuilt or relabelled. The complete workspace gate is
owned by the coordinator. The patch has not been pushed to the dependency
repository.
