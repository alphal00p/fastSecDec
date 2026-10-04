# Independent sector symmetry review

Date: 2026-10-04. Reviewed the native Graphica adapter in
`generation/symmetry.rs`, generation registry and multiplicity application, and
the independently authored analytic box and literal-symbol regressions.
No scientific correctness defect was found in this slice.

## Identity and measure preservation

Graphica receives the existing factored Atom DAG. Additive and multiplicative
children are unordered, power and ordinary function arguments keep their
positions, and all integration parameters have vertices, including unused
dimensions. Regulator-dependent and other parameter-independent subexpressions
remain exact Atom constants. No polynomial expansion is needed for this step.

The native canonical vertex map has the input-to-canonical orientation expected
by the adapter. A canonical graph match is only a candidate: the adapter checks
that the recovered coordinate map is a bijection and verifies exact equality of
the complete density using native simultaneous literal substitution. This
preserves coefficients, numerator factors, exponents, scalar weights, and the
absolute sector Jacobian. Structural mismatches can miss a symmetry, but cannot
authorize an unequal density to merge.

The allowed map is a permutation of the same-dimensional unit cube, with unit
absolute Jacobian. Generation keeps the first representative and multiplies
every Laurent coefficient by the count exactly once, before classifying the
whole sector as constant or stochastic. Consequently the runtime covariance
uses the actual weighted vector. Individual boundary terms are not split into
artificially independent integrations.

## Independent integral checks

`tests/independent_symmetry.rs` adds three tests with complete QMC production
coverage and the whole Laurent vector:

- For Gamma(eps) times the square integral of `(x+y)^eps`, two charts merge to
  one. The residue is 1 and the finite coefficient is
  `2 log(2) - 3/2 - EulerGamma`, from the elementary meromorphic integral.
- With numerator `x^2+2*y^2`, the two chart residues are `5/12` and `7/12`.
  They must remain separate even though the singular support is identical;
  the total residue is 1.
- Separate `a*x+b*y` terms check opposite, unequal, and equal weights. Opposite
  weights cancel the residue and finite term without merging unequal densities;
  unequal weights retain the correct residue; equal weights do merge.

All three passed in the full workspace gate: **133 passed, 0 failed, 4 ignored**.
The author's seven structural symmetry tests also passed. The new tests check
integral identities independently of the graph-canonicalization implementation.

## Native scientific examples and observations

The native massless box test uses a Gram matrix with zero external masses,
momentum-conserving row sums, and `s=t=-1`. Its measure convention predicts
Laurent coefficients `[4, -4*EulerGamma, 2*EulerGamma^2-4*pi^2/3]`, and the
native graph generation and integration pass that check. The literal-symbol
test uses integration and regulator names ending in underscores; the identity
integral of `x^(eps-1)/(1+x)` gives `[1, -log(2)]`. It tests the literal-substitution
boundary against accidental pattern wildcard behavior. Both tests passed in
the same workspace gate.

The CLI timing metadata was also checked read-only. Generation timings live
outside the scientific identity hash, while artifact load time is a fresh
nonserialized observation. Changing timing metadata therefore does not change
the integrand or invalidate a scientifically compatible checkpoint; an existing
process test exercises this behavior. These observations do not constitute a
new broad performance endorsement or an audit of future non-permutation maps.
