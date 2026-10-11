# Constructive local algebraic root recursion

The native `threshold::algebraic_branch::aj` module constructs selected real
root germs after the checked discriminant/SNC admission. It computes shifts,
Cartier valuations, rational ramification powers, signed source choices and
fiber clusters, then descends by factor degree. This implements a local part of
the [Abhyankar–Jung construction](https://math.univ-cotedazur.fr/u/parus/publis/A-J.pdf),
not a complete real integration atlas.

Every terminal retains the full original defining polynomial, its
monicization denominator, the composed root expression and a checked exact
recombination. No quadratic formula or radical expression is required. All real
clusters remain present until a separate branch selection can justify choosing
one. Finite endpoint series are not used as bulk polynomial approximations.

## Native reuse and admission

Public API, source and focused executable probes confirm reuse of Symbolica's
`shift_var`, `quot_rem`, `resultant`, `Root`, `AlgebraicContext`, native polynomial
reduction and monomial reordering. Existing FastSecDec checked étale, SNC,
component, Cartier and localization owners supply geometry and quotient checks.
The private germ-local history cannot authorize resetting global BM history.
Native algebraic element rendering preserves one common primitive element;
it does not create independent symbolic copies of the same extension.

Multiplicity greater than one uses the complete implicit factor-coefficient
owner. A simple root instead retains its full polynomial and native monic
division complement with one dependent root coefficient. The variants are
distinct. With guarded coefficients, the cleared derivative/resultant identity
is checked in the **original extended base algebra**, before adjoining the root
equation; a later quotient cannot hide a wrong source identity.

The étale term preflight now uses native derivative row term counts to bound
determinants and all Cramer numerators, while retaining the previous valid dense
bound. The native determinant and solve operations are unchanged. These are
expression-size checks, not guarantees on native intermediate allocations.

## Membership cost

Membership first tries native division by the admitted original generators.
A zero remainder is a constructive membership witness even without a Gröbner
basis. A nonzero remainder is inconclusive and falls through to native F4 in
GrevLex order. The existing verifier checks Buchberger closure and preservation
of the original equations. Explicit elimination continues to use Lex order.
Both remainder paths retain polynomial-budget checks; input validation occurs
before the shortcut.

For `(z²-x)(z-1)`, the earlier repeated-cluster control stopped at its 60-second
limit with roughly 1.1 GB sampled RSS. With the same 60-second/2-GB limits, the
membership change completed the genuine degree `3 → 2 → 1` recursion in 7.441
seconds with 13,934,592 bytes sampled peak RSS. This single diagnostic comparison
does not establish general scaling. Earlier quartic and larger cubic failures
remain recorded; their completion is not inferred from the simpler control.

## Evidence and limits

The frozen isolated suite passes 103 controls in 10.21 seconds, including eight
new production AJ groups, three membership groups and three supplementary
native API probes. Measured outer time is 10.353 seconds and sampled peak RSS
19,136,512 bytes. Strict isolated Clippy passes. Controls include:

- Automatic fifth ramification, shifted/non-binomial quintics, regular quintic
  roots and nested exact constants with actual coefficient-field degree ten.
- All real signed square-root branches, coprime-to-simple repeated recursion,
  guarded factor rows and original-base simple-root identities.
- A nonzero direct remainder that nevertheless belongs to the ideal,
  Lex/GrevLex agreement, zero ideals, malformed variable maps and exhausted
  input/output/operation/slot limits.

After registration, the joined native threshold suite passes 173 tests with one
existing ignored test (17.21s). Strict native library/test Clippy passes (21.07s),
as does workspace formatting. The registered source retains the newer BM cycle
work; the earlier private resolver snapshot was not copied over it.

Root and resolver-agent reviews accept the local mathematical, native-reuse
and memory boundaries. Some composed-frame tests explicitly increase
bookkeeping limits, including the conservative term bound; production defaults
remain unchanged. Native CAS calls still need caller-owned hard process limits.

Nontrivial ramification currently requires coordinate SNC divisors; other
presentations report `CoordinatePreparationRequired`. Local target coordinates
are signed: odd powers cover both signs, while even powers retain both source
signs. A nonnegative integration cube requires separate orthant and multiplicity
accounting. No global CAD selector, runtime bracket, parameter box, closed-face
endpoint certificate, auxiliary-regulator cancellation or complete resolver is
conferred by this owner. Native `solve::nsolve_bracketed` is already available
for the later runtime map; certified moving brackets remain separate work.
