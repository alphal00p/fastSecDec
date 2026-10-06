# Fixed constants and decoded evaluator IR: scope audit

Delivery update (2026-10-06): the generic fallback and decoder-validation
patches are removed. FastSecDec now uses upstream numeric-domain APIs and
optional conditioning, with the existing multiprecision rescue as fallback.
This document retains the source audit that motivated the change.

Source-only audit, 2026-10-06. No implementation, build or numerical run was
performed for this review. Public Symbolica community revision
`473b4b8dbc2f9bff8658a047196ba0877238bf9e` changes the alias parser; its evaluator,
numeric-domain and special-function files are unchanged from `6defcca`.

## Actual constant use and limitation

Gamma-prefactor Laurent coefficients contain fixed special-function values,
including `polygamma(1, 3)`. FastSecDec maps the same exact native evaluator into
ordinary arithmetic, `ErrorPropagatingFloat` conditioning, and higher-precision
rescue arithmetic. The conditioning construction is in
[`compilation.rs`](../../crates/fastsecdec/src/kernel/compilation.rs); the complex
equivalent is in [`complex.rs`](../../crates/fastsecdec/src/kernel/complex.rs).
[`mod.rs`](../../crates/fastsecdec/src/kernel/mod.rs) uses the conditioning result
and its estimated error when deciding whether a checked point needs rescue.
This estimates arithmetic error; it is separate from integration uncertainty.

The local fixed-argument patch preserves a registered target-domain callback
when present. Otherwise it evaluates the fixed rational arguments through a
`Complex<Float>` callback and converts the result. Symbolica's
`evaluate/domain.rs:274–313` then assigns nominal binary64 precision
(`15.954589770191003` decimal digits) to the returned error-tracking value.
Subsequent evaluator arithmetic propagates that assigned uncertainty. The
callback's internal cancellation, approximation error and sensitivity to
rounded arguments are **not** propagated through this conversion.

The original polygamma case has a stronger implementation-specific basis:
Symbolica `transcendental.rs:490–531` registers ordinary/multiprecision callbacks,
but no error-tracking callback; `polygamma_checked` at `4501–4613` uses 64 extra
working bits, checks its remainder plus roundoff estimate, and rounds to the
requested precision or declines numerically. The integer argument 3 is exactly
representable. This does not establish the generic fallback's contract for an
arbitrary registered callback. The fallback does not enforce a restriction to
such precision-controlled built-ins.

The square-at-3 regression proves conversion and finite assigned error, not
coverage of callback-internal error. The Gamma worker/rescue regressions prove
their tested values and numeric worker execution, not a universal error bound.
Rescue remaps constants at increasing precision and compares two results in
[`precision.rs`](../../crates/fastsecdec/src/kernel/precision.rs); agreement is
not an independent certificate for an arbitrary callback.

The generic fallback therefore remains a qualified local workaround, not
accepted general error-tracking support. The candidate owner-level correction
is a native target-domain implementation, or an explicit precision-controlled
constant contract, for the required special functions. Until that exists,
returning the existing typed unsupported-domain error is the conservative
generic behavior. No new downstream error estimator, CAS, tolerance change or
behavioral correction is implemented by this audit.

## Why decoded IR was checked

No production builder-generated invalid IR was observed. The tests deliberately
modified serialized native evaluators: for example, a one-input/one-output
program's result index was set to 99. The old decoder accepted these bytes.
Symbolica's public evaluator uses unchecked internal operand accesses
(`evaluate/evaluator.rs:283–325`), while its old serde decoder at `128–155`
returned the decoded fields without checking those invariants.

The validation patch checks representation invariants at the cold-load boundary;
it does not repair generated algebra or prove mathematical equivalence. A
checksum is not structural validation. The
[`kernel_artifacts.rs`](../../crates/fastsecdec/tests/kernel_artifacts.rs)
regression places the deliberately invalid native bytes in an otherwise valid,
re-signed artifact and expects a decode error before compilation. Invalid IR
was never evaluated. See the [IR validation record](../dependency-patches/symbolica-evaluator-ir-validation.md)
for the original fixtures and checks.
