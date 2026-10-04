# Independent review of native factored monomial stripping

Reviewed the change in `generation/mapping.rs::factored_residual`, its three
mapper regressions, the native capability probe, and the existing Symbolica
implementations/contracts of `AtomCore::collect_factors` and
`AtomCore::is_polynomial`. No actionable correctness or reuse finding was found.

The fast path changes the representation of a residual after its valuation is
known. Exact support minima, signed-coordinate arithmetic and checked exponent
conversion precede the new call and are unchanged. Native collection followed by
multiplication by the exact inverse monomial preserves the expression. The new
acceptance test is deliberately stricter than `is_polynomial(true, false)` alone:
the native API may regard a reciprocal, function or compound power as an
indeterminate. The adapter accepts only actual target coordinates or expressions
independent of every target coordinate. Thus a hidden coordinate denominator,
`log(x)`, fractional coordinate power or unevaluated compound reciprocal cannot
masquerade as a polynomial residual merely by being renamed by the native
predicate.

When that sufficient condition is inconclusive, the previous signed native
polynomial collector and `mul_exp` path remains unchanged, including the explicit
negative-exponent check. Exact original/minimum/difference bounds are checked
before either path. The singular residual domain/boundary certificate still runs
after either result, so collecting factors does not weaken positivity or endpoint
admission. Coordinate-independent regulator/special-function factors remain
legitimate coefficients; the adapter does not try to implement new algebra for
them.

The author's three focused tests cover compact degree-128 monomial extraction,
a signed orthant infinity chart including its measure, and a cancellation that
needs the sparse fallback. The author reported these tests and 30 existing
generation/scientific/strategy tests passing, with logs linked in
`native-monomial-stripping.md`. The independent reviewer additionally executed
the four new graph/Gamma/order/numerator regressions in
`tests/regression_gaps.rs` against the changed mapper: all passed. These checks
provide correctness evidence; they do not establish a rank-five speedup or the
project's complete performance acceptance gate.

The thread-local switch is compiled only under `cfg(test)` and forces the exact
previous fallback for controlled profiling. Production has no environment flag
or alternate numerical policy. Any final timing report must identify the
optimized dependency build, same fixture/input, and switch setting; the paired
rank-five release measurement remains the author's separate task.
