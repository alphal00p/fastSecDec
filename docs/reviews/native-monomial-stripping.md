# Native factored monomial stripping

The rank-five diagnostic showed that flattening a signed sparse polynomial after monomial extraction can grow a small factored input into a large mapped Atom. The new mapping fast path changes only that representation choice. Exact transformed supports, coordinate minima, checked signed-exponent bounds, and singular residual validation remain in place.

The required algebra was checked at three levels:

1. Public APIs: `AtomCore::collect_factors` extracts common factors without expansion; `AtomCore::is_polynomial(true, false)` recognizes a factored polynomial and reports its indeterminates. Signed `to_polynomial_in_vars::<i32>`, `mul_exp`, and `flatten(false)` remain the native fallback.
2. Native source: Symbolica `src/collect.rs:1266` recursively intersects factors of sums, combines products and handles integer powers. It retains a combined base for noninteger powers. `src/id.rs:1500` and the public contract at `src/atom/core.rs:1836` show that polynomial recognition may select compound indeterminates. Therefore a reported indeterminate is accepted only if it is a literal integration coordinate or independent of every integration coordinate. A reciprocal or `log(x)` cannot pass merely by being treated as a new variable.
3. Executed probe: `generation::profiling::native_common_factor_collection_can_strip_chart_monomials` passed for nonnegative charts, signed orthant charts, multiple coordinates and a compact degree-10000 power. It also demonstrated a hidden polynomial cancellation that collection does not expose, requiring the existing signed sparse fallback. Evidence is `output/probes/native-monomial-strip.log`.

After the exact minima have been computed, mapping calls native collection and multiplies by the known inverse monomial. A polynomial residual is retained in that factored form. Failed recognition falls back to the prior native signed collector, including its explicit negative-residual-exponent check. No custom factorization, division, valuation or symbolic simplification algorithm was added.

Three production-mapper regressions passed: a degree-128 chart power stays compact with the correct density exponent; an infinity chart preserves the Jacobian and `(1+t)^-2` residual; and hidden cancellation yields the exact sparse result. Thirty focused generation, independent scientific and subtraction-strategy tests also passed. Logs are `output/mapped-factor-tests.log` and `output/mapped-factor-scientific-tests.log`. Full rank-five release and independent reviewer results remain separate gates; these checks alone do not establish a speedup or all-example parity.

The profiling harness has a thread-local `cfg(test)` switch to force the unchanged sparse fallback. This permits paired measurements with identical optimized dependencies and fixture setup. Production builds contain no switch or environment-variable behavior. Probe output names include build and stripping modes, preserving the original development evidence.

## Valuation API follow-up

Native `MultivariatePolynomial::degree_bounds(axis)` (`src/poly/polynomial.rs:2224`) returns exact minimum and maximum exponents, but it operates on an already collected sparse polynomial. Native `Series::get_trailing_exponent()` (`src/poly/series.rs:564`) reports the lowest stored exponent of a series. No public exact Atom-level valuation operation was found in the checked core and collection APIs. Series depth and cancellation semantics need a separate executable probe before it can serve as a compact valuation oracle. The current change deliberately retains exact support minima rather than introducing such an oracle or a custom engine.
