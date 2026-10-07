# Numerical-dual IBP formula reuse review (2026-10-08)

The existing dynamic formula cache supports integration by parts as well as
Taylor subtraction. This review changes neither the default strategy nor the
cache architecture. No three-loop generation was run.

## Exact ownership and reuse

`generation/numerical_dual/formula.rs` keys recipes by the ordered endpoint
powers (exact constant and epsilon slope), complete prefactors, dimension,
requested Laurent order, subtraction strategy, coefficient-expansion method
and resource limits. Native Atom and Rational equality own these comparisons.
The shared generation context owns regulator, ordered coordinates and reserved
symbols. Taylor and IBP requests cannot collide.

The discovery pass gathers all signatures before preparation. Stable ordered
keys identify unique formula jobs, which execute through the existing
caller-owned dispatch. Only complete admitted results proceed to sector
instantiation. Matching sectors share the same `Arc<Recipe>` while retaining
their distinct maps and regular sources. Cooperative sessions retain completed
recipes across pauses; the cache is neither process-global nor persisted for
reuse by another generation.

Recipe construction calls the existing coefficient-first endpoint/Laurent
composer with opaque regular functions. IBP boundary values at one and the
remaining subtraction at zero become native derivative requests; there is no
second hand-written subtraction algorithm. Source evaluators and their native
Symbolica `HyperDual`/`Dualizer` programs are cached separately by exact source,
ordered inputs, compilation settings, requested minimal ancestor-closed shape
and structural-zero mask. Identical keys share standard-library `OnceLock`
owners; unrelated builds do not hold the index lock while compiling. Distinct
sector maps and numerical evaluations are not shared.

## Current checks and measured scope

The focused native rerun passed 24 tests: 20 numerical-dual unit tests, two IBP
scientific integration tests, the retained/reordered dispatch identity test and
the formula-admission test. Tests include mixed endpoint powers
`x^(-3-2*eps) y^(-2+eps)`, complete Laurent vectors, exact reconstructed recipes,
IBP boundaries at one, strategy/key separation, runtime source ownership,
concurrent program reuse and malformed completion rejection. A two-chart
control observes one prepared formula, two uses and one shared use; eight
concurrent identical jet requests share one program owner and one cache entry.
No defect was found. Logs remain ignored under
`output/ibp-formula-cache-review/`.

Existing on-shell double-box Taylor metadata records four prepared formulas
for 30 eligible chart uses, with 26 shared uses and 0.196 seconds of formula
preparation. The earlier generic-kinematics IBP benchmark records the same
counts and 0.178 seconds. That historical measurement does not establish the
cost of IBP for the current on-shell graph or either three-loop input. See the
[formula benchmark](numerical-dual-formula-benchmark.md) for its scope and the
[native cache review](numerical-dual-native-cache.md) for the underlying owners.
