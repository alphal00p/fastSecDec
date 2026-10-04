# Rational-coordinate integration reuse investigation

This is a capability/source audit plus a prepared diagnostic probe, not a production optimization. No dependency or antiderivative implementation has been added.

The community bridge already delegates to `symbolica_integrate::Integrate` (`symbolica-community/src/lib.rs:15,52`). Upstream version 2.0.1 offers best-effort antiderivatives and cooperative cancellation at integration boundaries. It scans Rubi rules and falls back to native rational integration; its cancellation API is not a strict work bound for every internal algebra operation. See the [public API and implementation](https://github.com/symbolica-dev/symbolica-integrate/blob/main/src/lib.rs) and [package manifest](https://github.com/symbolica-dev/symbolica-integrate/blob/main/Cargo.toml). It is not a current FastSecDec dependency, and no local package source was cached during this audit.

The narrower needed capability is already present in our pinned Symbolica checkout, with no extra dependency:

- `src/domains/rational_polynomial.rs:1530`: `RationalPolynomial::integrate(var)` returns separate rational antiderivative pieces and logarithmic pieces. The latter can involve algebraic root sums and should not be silently treated as ordinary logarithms.
- The implementation uses native polynomial division, square-free factorization, partial fractions and rational-function coefficient arithmetic. Other coordinates can remain symbolic parameters.
- Native tests `constant`, `mixed_denominator` and `multiple_variables` exercise such parameter dependence. Symbolica owns this algorithm; FastSecDec must not reimplement it.

The prepared ignored test `double_box_leading_pole_is_an_exact_total_derivative` reads the **actual** preserved double-box coefficient expressions. It verifies a rational primitive by native differentiation and boundary substitution, and independently calls the existing native rational integrator, requiring no logarithmic parts. It checks the integrated leading pole is zero without replacing the numerical output or treating the historical target as an oracle. Execution status is recorded separately with the diagnostic results.

Any later production use should be a small, measured orchestration step on bounded-size rational candidates. A returned primitive would still need exact derivative verification and finite, justified endpoint evaluation. Unsupported logarithms/root sums, expensive candidates and unresolved limits should retain the existing numerical coefficient. This is not a new general integration engine, nor permission to globally expand large factored densities. Existing work limits do not currently provide an interruptible bound inside native rational integration, so admission thresholds need measurement before adoption.
