# Regular algebraic sections: exact secants as endpoint units

Status: independently reviewed construction and native API probe. This adds
no production owner and does not complete the general resolver or an integration
artifact. It extends
the typed regular-section work with a degree-independent normalization seam.

## Preconditions and authority

Let `P(x,y)` be the full native polynomial defining an admitted simple section
`r(x)`, with its actual verified GCAD selector and original factor association.
Let `A(x) <= r(x) <= B(x)` bound its neighboring intervals. Require a *separate*
uniform derivative certificate `sigma*P_y >= c > 0` on a closed cylinder
containing every segment used below. A root-isolation bracket alone does not
cover an adjacent cell extending outside that bracket. Neither a proof sample
nor interior positivity establishes this condition.

Here `sigma` is fixed in `{+1,-1}`. The source root equation, the full interval
domain, the derivative receipt and the endpoint range receipts stay bound to
the same immutable owner. Parameter conditions stay in that domain. The proof
is sufficient rather than complete: cells between several roots of the same
polynomial cannot pass a uniform monotonicity check covering both roots.
Those cases retain their other resolution/cover obligations.

Define the exact polynomial secant

    Q(x,y,r) = (P(x,y)-P(x,r))/(y-r).

Use native polynomial replacement and exact `quot_rem`; retain the zero
remainder and recombination. At `y=r`, its polynomial value is `P_y(x,r)`.
Never evaluate it numerically as a quotient of two almost equal values.
The fundamental theorem of calculus gives

    sigma*Q(x,y,r) = integral_0^1 sigma*P_y(x,r+s*(y-r)) ds >= c.

Thus this same closed unit includes the diagonal and faces. No radical or
quadratic formula is involved. Its derivatives come from Symbolica's native
chain rule, including the root's implicit derivatives and every dependency.

## Exact width and pulled-factor identities

Write `Q_A=Q(x,A,r)`, `Q_B=Q(x,B,r)` and impose the retained equation `P(x,r)=0`.
Native exact identities give

    r-A = -P(x,A)/Q_A,
    B-r =  P(x,B)/Q_B.

For the increasing orientation, on the lower map `phi=A+(r-A)*t`,

    -P(x,phi) = (-P(x,A))*(1-t)*Q(x,phi,r)/Q_A,
    J_y        = (-P(x,A))/Q_A.

On the upper map `phi=r+(B-r)*t`,

    P(x,phi) = P(x,B)*t*Q(x,phi,r)/Q_B,
    J_y      = P(x,B)/Q_B.

Consequently the lower contribution from `J_y*|P(phi)|^q` has endpoint factors
`(-P(A))^(q+1)*(1-t)^q` and smooth factor `Q(phi,r)^q/Q_A^(q+1)`.
The upper contribution has factors `P(B)^(q+1)*t^q` and the analogous smooth
factor. For decreasing orientation apply these positive-unit statements to
`sigma*P`; retain the original causal sign and full regulator-dependent phase
separately. Do not combine positive-unit logarithms with the causal phase.

The Jacobian power is included exactly once. The numerator remains the native
composition `N(x,phi)`; no complete substitution/expansion is needed. Other
denominator factors retain their independent unit/valuation obligations.

Width zeros have become zeros of the *lower-dimensional* endpoint expression
`P(x,A)` or `P(x,B)`, rather than being dropped or declared units. If `A` or
`B` belongs to an earlier algebraic tower, that exact tower and its denominators
remain present. Rational endpoints permit ordinary native polynomial geometry;
collisions or nonmonotone sections still need the general resolver.

## Proposed later owner boundary

A checked `SecantUnit` would consume a regular section and a derivative margin
over the actual cell-range cylinder, derive its polynomial internally, and
retain the original factor identity and orientation. A width normalization
would additionally consume the actual endpoint owner and certify the identities
above. Neither object could manufacture a complete closed-face atlas.

The existing dimension-generic symbolic endpoint engine can only consume these
maps after all width/prefactor valuations, face intersections and the common
analytic convergence family have their own certificates. A regular implicit
derivative must not be used at a collision where its minor vanishes. A general
parameter-stratum, global root transport or auxiliary-regulator cancellation
claim does not follow from this lemma.

## Native reuse and probe

Public/source audit: Symbolica `MultivariatePolynomial::{replace_with_poly,
quot_rem,derivative}` in `src/poly/polynomial.rs` (revision `1ac765f`), existing
division tests, and FastSecDec's checked factor/transform owners already supply
the exact algebra. The probe uses those operations directly through symGCAD's
existing native polynomial owner; no new algebra helper is implemented.

The probe checks generic secants, the diagonal derivative, both width identities
and both exact pulled-factor identities at degrees 5, 7, 9, 11 and 17, including
shifted coefficients, negative orientation and moving rational endpoints.
Those are polynomial-identity tests, not claims that all their sample domains
pass a branch or closed-cylinder certificate. Actual GCAD ownership, numerical
root callbacks and proof provenance are covered by the separate regular-section
slice and must be connected before production admission.

The direct native probe passes in 0.19 seconds against Symbolica `1ac765f`
and symGCAD `a1132d4`. Root authored the construction; the independent runtime
agent checked both orientations, width factors, Jacobian powers, closed-margin
hypotheses and native API reuse. The numerical root/selector evidence remains
separate from these polynomial-identity checks.
