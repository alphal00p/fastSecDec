# Minimal portable FastSecDec backend proposal

This is a source-only proposal. No manifest, library code or dependency source
was changed for it. The current native default remains GMP/MPFR with direct
SymJIT O2. A portable build should reuse the same FastSecDec public generation,
kernel, weighted-replay and integration APIs with Symbolica's interpreter and
Numerica's Malachite/Astro types.

## Existing evidence and limits

The [isolated evaluator probe](../../output/probes/portable_evaluator_feasibility/README.md)
already executes the existing exact evaluator in a real Emscripten/Node build.
Its [Wasm result](../../output/diagnostics/portable-evaluator-feasibility-1/wasm-result.json)
records six triangle coefficient-vector evaluations, aliases, real and complex
interpretation, 128/256-bit arithmetic and 4,096 caller-driven QMC points with
covariance. The selected dependency tree contains Malachite/Astro and no
SymJIT/GMP/MPFR. This is not yet a build of the FastSecDec crate, a Python wheel
import, a complete integral or a browser-responsiveness measurement.

## Feature ownership

Use mutually exclusive `native` and `portable` features, with `native` as the
default. Reject both-selected and neither-selected configurations clearly.
Keep the CLI native; a portable consumer builds only the library with default
features disabled. Do not use an all-features build to combine incompatible
numeric backends.

| Owner | Smallest change |
| --- | --- |
| Workspace dependencies | Keep common Symbolica/Numerica `serde` and `bincode` features; move unconditional numeric and JIT selections to crate features. Make inherited FastSecDec/sector defaults explicitly controllable. Retain the exact SymJIT version pin. |
| `fastsecdec` | Default `native` enables the current Symbolica/Numerica GMP/MPFR features, Symbolica native code generation, optional direct SymJIT dependency and sector native feature. `portable` enables Symbolica `wasm`, Numerica Malachite/Astro and sector portable support. |
| HEPKit dependencies | Portable forwards existing `feynkit-graph/wasm`, `feynkit-kinematics/wasm`, `linnet/wasm`, `spenso/wasm` and `idenso/wasm`. The model crate has no separate Wasm feature and already uses backend-neutral Symbolica dependencies. Preserve current native selections; do not enable extra HEPKit native features merely to rename the configuration. |
| `fastsecdec-sectors` | Forward the same selected Numerica backend. Give standalone sector builds their current native default, while FastSecDec disables that dependency's default features and selects it explicitly. Geometry algorithms stay unchanged. |
| CLI and later Python bridge | CLI explicitly selects native. The bridge chooses the library's compile-time backend and reports it honestly; it does not add an evaluator or an integration driver to the library. |

The inspected HEPKit workspace dependencies already disable Symbolica/Numerica
and graph/tensor default features. Their existing Wasm feature graph is the
reuse path; no HEPKit backend implementation is needed. Check the resolved
actual-library feature tree after wiring rather than assuming that the isolated
probe's tree establishes transitive feature closure for this larger crate.

## One private evaluator boundary

Add a small private `kernel/evaluator.rs` adapter, with real and complex
constructors plus the existing evaluate/clone operations. Native variants wrap
the present `JITCompiledEvaluator<f64>` and
`JITCompiledEvaluator<Complex<f64>>`; portable variants wrap the corresponding
`ExpressionEvaluator` types mapped from the unchanged exact program. Backend
selection is compile-time, with no public per-call switch or duplicated
precision algorithm.

The direct edits are the evaluator field types in `kernel/mod.rs` and
`kernel/complex.rs`, plus their constructors in `kernel/compilation.rs` and
`kernel/complex.rs`. Preserve exact-program construction, direct translation,
zero Horner iterations, shared aliases, complete Laurent/component layouts and
worker-local cloning. Native constructors keep exactly their current O2 call.

Conditioning, cancellation checks, precision caches, weighted rescue and replay
remain in their existing modules. `Float` already selects the backend through
Numerica. `map_coeff_with_prec` continues to receive the chosen binary precision,
including all input/output constants; the two-increasing-precision agreement
policy is unchanged. Backend-neutral wording should replace MPFR-specific
documentation where it describes the shared policy. No changes are needed to
caller-owned `QmcSession`, `QmcWorker`, complete-shift covariance or checkpoint
scheduling. Synchronous browser yielding/cancellation remains the caller's
responsibility; this feature does not promise browser responsiveness.

### Preserve fallible native admission

A direct substitution of `map_coeff` for `jit_compile` is **not sufficient**.
The current Symbolica exact-program `jit_compile` fallibly resolves fixed
constants and rejects unsupported external callbacks before returning a
kernel. Public `map_coeff_with_prec` currently unwraps constant-conversion
errors; an unavailable variable callback can instead panic at evaluation.
`try_evaluate` checks dimensions but is not an equivalent callback-admission
operation. The external-function list and constant owner are private.

The existing regression
[`complex_fixed_gamma_cannot_be_loaded_with_a_real_output_layout`](../../crates/fastsecdec/tests/kernel_artifacts.rs)
is a concrete acceptance boundary: the complex constant must evaluate in its
supported complex layout and produce a typed load error in a forged real
layout, without a panic. It must continue to pass under the portable path.

The smallest missing native operation is an additive, fallible coefficient-map
entry in Symbolica, reusing its existing mapping and `evaluate_constant` owner
and checking mapped external implementations as its JIT admission already
does. Keep existing infallible methods compatible. A focused native regression
for this operation should precede its use; do not copy callback evaluation or
constant resolution into FastSecDec, use panic catching as ordinary admission,
or expose private IR to a new validator. This is a narrowly identified API gap,
not a request for a new evaluator.

## Artifacts and compatibility

Keep native version-three bytes, codec string, compiler policy and content-ID
domain unchanged. Preserve native version-one/two loading and IDs. Portable
artifacts may reuse the same envelope fields and native Bincode serializer,
but need a distinct portable codec identifier, interpreter policy and hash
domain. Include the selected exact-integer backend and high-precision backend
in that identity. Validate these before exact IR decoding.

Do not load a native O2 artifact into the portable interpreter while retaining
its original execution identity. The numerical `Integer::Large` representation
delegates serialization to different backend types; the small triangle's
successful decode does not prove general GMP/Malachite IR interchange. The
minimal first implementation rejects cross-backend artifacts and native legacy
artifacts in the portable build with a typed policy error. It generates and
round-trips portable programs within the portable build. A general conversion
format is outside this slice.

Keep source/domain/chart metadata, precision policy, ordered output layout and
content-hash validation in the current envelope owner. No alternate serializer,
instruction format, JSON evaluator or persistent runtime stack is needed.

## Short implementation and validation sequence

1. Add the native fallible-map API and its focused constant/callback rejection
   regression, preserving the existing mapping behavior.
2. Wire feature selection and the private hot-evaluator adapter; make artifact
   codec/policy selection explicit without changing native identities.
3. Run existing meaningful real/complex Gamma, alias, weighted-rescue, clone,
   malformed-artifact and complete-QMC tests in the native build and the
   portable host build. Use native v1/v2/v3 tests for unchanged native behavior
   and portable round trips plus cross-policy rejection for the new path.
4. Build the actual FastSecDec library for the already-proven Emscripten target,
   through a tiny consumer using its ordinary public API. Generate a small
   admitted input, compile/evaluate complete vectors, round-trip its portable
   artifact and finish a small caller-driven QMC allocation. Reuse the existing
   pinned toolchain/probe orchestration; no new benchmark framework is needed.

Only after those boundaries pass should the community Python bridge enable
the feature and test wheel import, event delivery and browser interaction.
Neither the current standalone probe nor this proposal claims those later
outcomes.
