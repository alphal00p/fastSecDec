# Weighted whole-vector replay

The caller owns `WeightedEvaluationContext` and its serializable `ReplayState`. The context returns an **already weighted** complete Laurent vector. The integration worker must use its explicit `evaluate_weighted` callback lane; the ordinary and `evaluate_with_weight` lanes retain their unweighted-output contracts.

A first nonzero-weight observation is checked at two increasing native MPFR precisions. Later observations trigger the same check if any weighted component grows by the configured factor (16 by default) over an accepted maximum, including a transition from a zero maximum. This growth rule is a diagnostic safeguard, not a mathematical accuracy proof. Constants do not repeatedly trigger it. Existing degree-aware cancellation checks and precision exhaustion remain in force.

The native evaluator multiplies by the finite nonnegative integration weight **before** converting MPFR outputs to `f64`. An amplifying weight triggers this path for raw subnormal or underflowed coefficients even if no growth is observed. Thus `x² + epsilon*x³` at `x=1e-160`, weighted by `1e300`, preserves both `1e-20` and `1e-180`; a comparison of rounded unweighted values would incorrectly accept substantial error and zero. Real and complex vectors use the same rule. Nonfinite weighted JIT values also receive native replay, allowing a raw overflow to recover when the weighted result is representable. Actual unrepresentable outputs fail explicitly after precision exhaustion.

Replay state binds the artifact identity, sector, policy and complete vector layout. Valid states merge componentwise maxima idempotently. A caller should merge only states returned with successfully submitted complete work packages; discard the state of a failed package together with its incomplete numerical contribution. State merging does not itself submit observations to the integration accumulator.

Observation order influences whether the growth heuristic requests additional precision. Restoring the same ordered history preserves its decisions, but changing worker scheduling may change rounding within the precision policy. Bitwise invariance across different schedules is not promised. The statistical work-package identities and complete-shift covariance rules remain independent of this heuristic.

Focused regressions cover first/growth checks, constant reuse, zero maxima and zero weights, invalid weights, explicit weighted overflow failure, full-vector real/complex underflow recovery after prior large maxima, representable weighted recovery of a `10^400` raw coefficient, worker-only numeric evaluation, and checkpoint identity/idempotence.

## Literal zeros after loading native programs

Cold version-three loading previously discarded every exact-zero output fact.
An amplifying weight could consequently send a padded zero Laurent coefficient
through MPFR as if it were an underflowed nonzero value. The loader now reads
Symbolica's public `ExpressionEvaluator::export_instructions()` and accepts only
an output with one direct `Assign(Out, Const)` writer whose exact rational real
and imaginary parts are zero. Slots listed in `constant_functions` are excluded:
their rational placeholders do not establish the registered function's value.
Control flow, non-inlined bodies, arithmetic results, parameters and repeated
output writes remain unproved. This is a native instruction fact, with no
sampled-zero test, expression reconstruction or new simplifier.

The immutable artifact bytes, content identity, schema and strict precision
policy remain unchanged. Fresh and cold real/complex weighted controls retain
the first-sample replay, avoid a second rescue caused solely by zero padding,
and preserve MPFR recovery of genuine tiny nonzero coefficients after prior
large maxima. This change makes no new performance or convergence claim.

The focused native gate passes 24 controls (two artifact unit, fifteen artifact
integration and seven weighted replay tests), formatting and scoped strict
Clippy. Independent source and retained-test review passes. Evidence is retained
under `output/diagnostics/cold-literal-zero-1/`; an initial invocation using a
different Nix-default compiler was stopped before tests, then the accepted
cached Rust toolchain completed the gate. No new browser or physical integration
run is inferred from these controls.
