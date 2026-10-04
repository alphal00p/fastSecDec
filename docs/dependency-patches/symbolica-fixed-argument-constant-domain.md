# Fixed-argument external constants in alternate numeric domains

The adjacent patch changes only Symbolica `src/evaluate/external.rs`. While
building a native `ExpressionEvaluator<ErrorPropagatingFloat<f64>>` for a
Laurent coefficient containing `polygamma(1, 3)`, the fixed-argument constant
path required an ErrorPropagatingFloat-specific function implementation and
panicked through the evaluator's constant conversion. The same external
function already supplies a `Complex<Float>` implementation, and the requested
numeric domain already supplies `try_from_complex_float`.

The fix preserves a directly registered domain implementation when available.
Otherwise it evaluates the fixed rational arguments using the existing
`Complex<Float>` implementation and converts the resulting constant through
the existing domain conversion. This matches the existing zero-argument
constant path; it adds no special-function implementation or algebra.

Validation: `cargo test -p symbolica --lib
fixed_argument_constant_converts_from_multiprecision_to_error_tracking --
--test-threads=1` passed (one focused regression). The regression registers a
function exclusively for `Complex<Float>`, fixes its rational argument, maps a
native evaluator to ErrorPropagatingFloat, and checks the numerical result and
finite error estimate. A FastSecDec Gamma-series regression additionally covers
the production polygamma trigger.

The production Gamma regression now also clones the native real evaluator, and
the public complex-kernel suite checks Gamma/logarithm precision rescue on a
worker. These pass using the same numeric constant conversion, without Atom
evaluation or Workspace operations on the worker.

No upstream push has been performed. The patch is intentionally local and
upstream-ready; the reference worktree remains separate from FastSecDec's main
repository history.
