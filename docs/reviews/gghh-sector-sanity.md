# gg→HH sector geometry cross-check

The independent pySecDec denominator decomposition agrees with all **30 native
FastSecDec charts**, up to permutation of target coordinates. This is a basic
geometry check for the exact diagram and physical point already used by the
bounded reference attempt. It does not establish numerator reduction, Laurent
coefficients or an independent amplitude value.

The retained full pySecDec attempt stopped before producing a sector package.
For this check, its existing rank-six input is passed to pySecDec 1.6.6's native
`LoopIntegralFromPropagators` constructor. Only the denominator `U` and `F`
properties are then evaluated; the expensive numerator parametrization and
`loop_package` are not called. Native pySecDec `Sector`, `primary_decomposition`
and `iterative_decomposition` perform the decomposition. Original coordinate
monomials are carried as `other` polynomials to recover each transformation.
No FastSecDec polynomial or chart is supplied to the independent decomposition.

| Check | FastSecDec | Independent pySecDec |
|---|---:|---:|
| Primary domains | 7 | 7 |
| Charts by fixed propagator 0…6 | 4, 4, 4, 4, 4, 4, 6 | 4, 4, 4, 4, 4, 4, 6 |
| Total charts / integration dimension | 30 / 6 | 30 / 6 |
| Absolute map determinants | 1 | 1 |
| Exact chart-map multiset, allowing target permutations | Same 30 maps | Same 30 maps |
| Exact transformed volume of each primary unit cube | 1 | 1 |
| Linear and quadratic coordinate moments on each primary cube | Pass | Pass |

Independent `U` has 15 monomials of degree two; `F` has 57 monomials of degree
three. All their coefficients are positive at the transported physical point.
For every native chart, transforming these independent supports reproduces its
stored factor valuations and leaves each residual with a nonzero constant
term. The exact determinant and monomial Jacobian agree with the stored native
geometry and with pySecDec. The volume and moment checks integrate only the
coordinate maps against simple polynomial probes, not the Feynman integrand.

The input uses the already audited reference transport: seven propagators,
six top masses and one gluon, the same rank-six numerator, masses, helicities,
colour projection and kinematics. Its existing small numerical transport
rounding remains; this check does not upgrade it to exact native scalar
arithmetic. In particular, equality of denominator charts alone does not prove
the numerator or the epsilon expansion correct.

The diagnostic completed in **1.09 seconds after imports**, inside a 60-second,
3-GB address-space bound on one CPU. No FORM process, evaluator compilation or
integration was invoked, and neither infeasible full reference was retried.
Evidence is retained under `output/diagnostics/gghh-sector-sanity-1/`, including
the complete matrices, exact moments, source hashes and process receipt. The
external diagnostic script is `output/probes/gghh_sector_sanity.py`.

The native artifact SHA-256 is
`7daec10b85fdd596f33b8cdebf67abd623c36e06fbcbac8bcc49f611f6db989a`;
the independent input SHA-256 is
`998e08c5bc0c794e93f2c96ad7e4d73d6b3e560d711e6ca0ca9d3bd706a32101`.

## Native numerator-inclusive endpoint powers

A separate native diagnostic maps the complete original numerator, its factors
and the Jacobian, then records `MappedTerm.powers` immediately before symmetry,
endpoint subtraction or Laurent expansion. All 30 six-dimensional charts finish;
the resulting 54 nonzero combined mapped terms contain 324 coordinate powers:
270 are zero, 24 are `eps`, 24 are `1 + eps`, and six are `-eps`.
Thus the largest integer `a` in `1/x^(a + n eps)` is **0**. No mapped coordinate
has a negative integer exponent at epsilon zero. The factors `x^eps` and
`x^-eps` correspond to `n = -1` and `n = 1`, respectively.

This numerator-inclusive statement is distinct from the independent denominator
geometry comparison above. The native numerator parameterization contains radial
Gamma prefactors `Gamma(beta - order)`, with `beta = 3 + 2 eps` here; for example,
order three gives `Gamma(2 eps)`. Such prefactors can supply the observed overall
pole despite nonnegative integer endpoint powers. This explanation follows the
native parameterization source, rather than a new prefactor decomposition.

The diagnostic used an ignored source copy, retained all physical inputs and
ran no subtraction, Laurent expansion, JIT compilation or integration. Its
bounded build and mapping process both completed and were reaped; 277 frozen
source/input/library hashes remained unchanged. Powers and the checked report
are retained in `output/diagnostics/gghh-endpoint-powers-1/{powers.jsonl,assessment.json}`.
The coordinator independently checked the capture boundary and complete power
histogram. No production code changed for this diagnostic.
