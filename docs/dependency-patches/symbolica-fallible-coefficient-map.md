# Fallible native evaluator coefficient mapping

Status (2026-10-06): the local API patch is removed. FastSecDec uses upstream
`map_coeff_with_prec`, preceded by numerical-domain admission through public
Symbolica metadata and conversion APIs. No native mapping implementation is
copied into FastSecDec.

## Historical proposal and validation

The adjacent patch adds `ExpressionEvaluator::try_map_coeff_with_prec` to the
local Symbolica 3.0.1 worktree. It returns the native mapping result or a `String`
error while preserving the existing `map_coeff` and `map_coeff_with_prec` APIs.
It changes no evaluator arithmetic, instruction format, or backend.

The portable-kernel design needs to construct an interpreted evaluator without
turning an unsupported coefficient domain into a panic. In particular, a fixed
`Gamma(1+i)` coefficient cannot enter a real evaluator, and a callback may lack
an implementation for the selected numerical domain. The current infallible
mapping unwraps constant-conversion errors and defers unsupported nonconstant
callbacks until evaluation.

## Reused owners and narrow gap

The public `jit_compile` constructor already resolves fixed external constants
fallibly and checks mapped callbacks before compiling. Its private
`ExternalFunctionContainer::evaluate_constant` uses the existing registered
implementation or multiprecision constant fallback and target-domain conversion.
`ExternalFunctionContainer::map` preserves the shared native function-body `Arc`,
resolves registered callbacks for the target domain, and starts with empty local
callback stacks. These are the same owners used by the new mapping API.

The public `try_evaluate` validates input/output dimensions; it does not provide
fallible construction or unsupported-callback admission. No existing public
fallible coefficient-map operation was found in the inspected source, examples,
or tests. The new method therefore shares the old mapping implementation and
adds only error propagation and callback admission. Native function bodies remain
valid without a direct callback implementation; their nested external functions
are checked through the existing flattened external-function table.

The new method does not catch panics from a caller's mapping closure or registered
callback. It does not prove that future evaluation points lie in a function's
domain. The previous infallible methods retain their constant-conversion panic
and deferred callback behavior.

## Validation status

Three focused native controls are prepared: real and multiprecision complete
vectors with shared nested function bodies; fixed complex Gamma rejection in a
real domain and successful complex evaluation; and a callback inside a retained
native body that is rejected for an unsupported target while the old API keeps
its previous deferred behavior. Compilation passed in 512.35 seconds with the exact locked native feature set.
All ten focused controls passed: the three new tests plus six existing evaluator
body/cache/stack controls and the fixed-argument error-tracking conversion
regression. The two bounded processes exited 0 without timeout in 0.047 and
0.028 seconds. Their executable and source bindings remained unchanged.
Root's independent source review found no actionable issue.

Evidence is retained locally under
`output/diagnostics/symbolica-fallible-map-1`. The first launcher failed before
Cargo because `/usr/bin/time` is unavailable; a separate compile attempt uses the
existing Python process timer. The patch is generated against the preserved
pre-change `evaluator.rs`, so it excludes the earlier native IR-validation edits
and preserves all five prior local fixes. No reference repository has been
published, and no FastSecDec feature or kernel implementation is changed here.
