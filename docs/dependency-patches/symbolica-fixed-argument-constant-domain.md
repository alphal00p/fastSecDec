# Fixed-argument external constants in alternate numeric domains

Status (2026-10-06): the local fallback patch is removed. FastSecDec uses
upstream ordinary/multiprecision implementations and makes the error-tracking
conditioning shortcut optional. Unsupported conditioning selects the existing
precision rescue. The generic callback uncertainty limitation below is not
carried into upstream dependency delivery.

## Historical proposal and validation

Status (2026-10-06): qualified local workaround, **not accepted general
error-tracking support for arbitrary callbacks**. The fallback assigns the
target domain's nominal floating-point uncertainty after callback evaluation;
it does not propagate error accumulated inside that callback. Public Symbolica
community `473b4b8` has not incorporated this change. Behavior is unchanged by
this source-only audit; see the [scope and correction proposal](../reviews/fixed-constant-error-tracking.md).

The adjacent patch changes only Symbolica `src/evaluate/external.rs`. While
building a native `ExpressionEvaluator<ErrorPropagatingFloat<f64>>` for a
Laurent coefficient containing `polygamma(1, 3)`, the fixed-argument constant
path required an ErrorPropagatingFloat-specific function implementation and
panicked through the evaluator's constant conversion. The same external
function already supplies a `Complex<Float>` implementation, and the requested
numeric domain already supplies `try_from_complex_float`.

The workaround preserves a directly registered domain implementation when available.
Otherwise it evaluates the fixed rational arguments using the existing
`Complex<Float>` implementation and converts the resulting constant through
the existing domain conversion. This resembles the existing zero-argument
constant path, but an arbitrary function callback does not thereby acquire a
verified precision contract. It adds no special-function implementation or algebra.

Validation: `cargo test -p symbolica --lib
fixed_argument_constant_converts_from_multiprecision_to_error_tracking --
--test-threads=1` passed (one focused regression). The regression registers a
function exclusively for `Complex<Float>`, fixes its rational argument, maps a
native evaluator to ErrorPropagatingFloat, and checks the numerical result and
finite assigned error estimate. That square-at-3 fixture does not test
callback-internal cancellation or establish that the assigned error covers it.
A FastSecDec Gamma-series regression additionally covers the production
polygamma trigger, whose native implementation has its own precision checks.

The production Gamma regression now also clones the native real evaluator, and
the public complex-kernel suite checks Gamma/logarithm precision rescue on a
worker. These pass using the same numeric constant conversion, without Atom
evaluation or Workspace operations on the worker.

No upstream push of this patch has been performed. A generic promotion should
await an agreed error/precision contract or a native target-domain
implementation for the required special functions. The reference worktree
remains separate from FastSecDec's main repository history.
