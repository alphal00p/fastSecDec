# Conditional construction of a finite real integration atlas

The independent research and scientific reviews accept the following
fixed-fiber construction **conditional on its geometric inputs**. It narrows
the effective-atlas gap in [the proof addendum](../NO_DEFORMATION_RESOLVER_PROOFS.md).
It does not complete that gate or implement a general atlas producer.

The input must be the actual compact resolved preimage, given by a finite
polynomial model and a closed semialgebraic predicate, together with checked
regular/SNC charts. A collection of unrelated affine charts is insufficient.
The original-to-resolved map must carry its pushforward degree, orientation,
selected real branches and ramification multiplicities. Every original-domain
inequality needs a signed monomial-times-unit identity, not merely an assertion
that its zero set occurs in the boundary inventory.

For each chart, use divisor equations and complementary coordinate projections
as local coordinates. Native differentiation and directed ball evaluation can
certify a uniform implicit inverse on a closed coordinate box. A rational
preconditioner, strict self-inclusion and a contraction bound give an executable
certificate. All additional defining equations must follow in the same exact
localization. Neither a sample-point determinant nor one selected square
subsystem alone establishes the desired branch.

Choose rational ambient support balls whose closures lie in these inverse
charts. Subdivide a known ambient bounding cube into closed dyadic boxes.
Accept a leaf only if a defining equation excludes zero, a domain inequality
is strictly violated, or one support function is strictly positive throughout
the leaf. Shared faces remain covered. Fair enumeration and refinement terminate
on the supplied compact regular model: every point has a strict support or
exclusion neighborhood, and compactness yields a finite subcover. This is a
conditional existence/termination argument, not a useful worst-case cost bound.

For support polynomials `p_j`, take `rho_j=max(0,p_j)^M` and
`w_j=rho_j/sum(rho)`. The coverage certificate proves a positive denominator on
the resolved domain. Choose integer `M` above the complete mixed-derivative
budget, including Taylor-remainder slack. Include the weights, maps and
absolute Jacobians in the density and every symbolic subtraction derivative.
The zero extension has the required finite smoothness; hard patch indicators
would not. Signed domain identities determine admitted divisor orthants.
Excluded orthants must not evaluate a possibly zero weight denominator.
Finite closed unions require their overlap accounting before integration.

Native controls passed for a whole quintic root neighborhood, a triangular
two-equation implicit system, singular-box refusal, closed coverage of a circle
with deliberately missing-support rejection, projective-coordinate identities,
and a counterexample to using bare blow-up incidence equations. They use native
Symbolica expressions, differentiation, matrices, F4 and Numerica RealBall.
Five test groups and strict Clippy passed. No runtime benchmark or RSS result
is inferred. Ball endpoint/sign methods are required: `RealBall::PartialOrd`
compares centers and cannot certify enclosure inclusion.

Real rank-one symmetric projectors provide bounded, unambiguous projective
coordinates. They do **not** prove that an incidence variety is the actual
blow-up graph. Redundant center generators can introduce spurious exceptional
components; exact graph closure and checked chart transitions remain necessary.
Likewise, a generic parameter chamber approaching a singular fiber does not
admit an automatically uniform finite atlas. Relative strata, the proper global
model, symbolic finite-smooth weights, general BM/AJ and regulator restriction
remain required work.

The uniform inverse step is supported by [Burr–Hauenstein–Lee, Theorem 2.1](https://arxiv.org/pdf/2602.07718).
Finite differentiable semialgebraic localization has precedent in
[Shiota, §2](https://www.numdam.org/item/PSMIR_1986___4_1_0.pdf).
The ambient coverage algorithm and its FastSecDec proof requirements above are
our proposed construction, not an implementation attributed to those papers.
