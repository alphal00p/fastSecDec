# Independent resolver proof review

Reviewed 2026-10-10: [G0 proof addendum](../NO_DEFORMATION_RESOLVER_PROOFS.md).
Reviewed source SHA256:
`8e9b9c07b295f767f24f6bd015ccba63d3b6781ca8b1bfc7ffdf2c2c2f8801ef`.
This is a scientific specification review, not acceptance of implemented
resolution, artifact publication, or a complete threshold integral.

The stated conditional claims pass review. **G0 remains incomplete:** effective
algebraic preparation/rectilinearization, the general certificate producer and
their termination/completeness arguments remain open (A3 and R4).

## Finite ownership is distinct from endpoint normal form

The least-index maximum-absolute-coordinate rule partitions projective
directions: a nonzero homogeneous tuple has a maximum, and strict comparisons
against earlier indices assign every tie once. Squaring makes the predicate
invariant under nonzero real projective rescaling. Propagation through a
supplied finite tree terminates by that tree's depth; it does not choose or
prove termination of a resolution tree.

Retaining the pulled-back domain and blow-up graph is essential for nonlinear
centers. Absolute Jacobians belong to real measure transport; causal phases
remain separate. Unique ownership away from the exceptional locus does not
prove smooth cube faces or finite endpoint derivatives. The document correctly
retains that gap instead of iterating CAD/ownership walls without an invariant.
The finite chart propagation precedent is within an absolutely convergent
rational-integral setting, as the addendum states.
[Viu-Sos, Remark 2.18 and Algorithm 3](https://arxiv.org/pdf/1509.01097).

The proposed stronger preparation target has relevant parameter-preserving
analytic maps and monomial-unit forms, including the Jacobian. It still does
not supply an implemented exact Nash algorithm or uniform boundary jets as a
parameter approaches an exceptional stratum. The distinction between the
rectilinear base and a full cube is justified; boundedness alone cannot license
a hidden discontinuous indicator or a derivative bound.
[Cluckers–Miller, Theorem 1.5 and Proposition 5.3](https://rcluckers.perso.math.cnrs.fr/prints/CM_JFA.pdf).

## Regulator claims have explicit sufficient hypotheses

The complete original-factor family, measure, gauge and regulator-dependent
causal phases must be identity-bound before partitioning. The one-dimensional
check is correct:
`(1-exp(-i*pi*eta))/eta -> i*pi`. Independent transverse thresholds give
`-pi^2`. Direct separation of the positive and negative halves also verifies
the stated higher-order control:

`(1+exp(-i*pi*eta))/(eta-1) + c*(1-exp(-i*pi*eta))/eta -> -2+i*pi*c`.

Freezing the auxiliary phase would lose the finite imaginary terms. These are
analytic controls, not numerical evidence of arbitrary cell cancellation.

The transverse `OriginalFamilyRegular` rule is sound under its supplied
coordinates, same-prescription grouping, unit regularity and proper transport
assumptions. Signed boundary-value powers are entire distributions in their
exponent; one-sided powers have negative-integer poles. The source explicitly
distinguishes these two families.
[Clerc–Kobayashi–Ørsted–Pevzner, Example 2.5/Table 2.5.1](https://arxiv.org/pdf/0906.2874).
The generic-epsilon nonresonance test therefore permits `1/(epsilon+eta)` but
does not make `1/eta` removable. Units, external prefactors and the finite
derivative bounds must cover the regulator neighborhood actually certified;
point samples or an unnamed “non-Landau” flag are insufficient. Tangencies,
opposite prescriptions and unresolved parameter collisions remain outside this
sufficient rule.

## Complete sums and higher/mixed poles

The common convergence region and certified pushforward identity are necessary
before transporting holomorphy to the sum. Lower-dimensional seams cannot be
discarded anew after continuation. Once joint auxiliary regularity is proved,
a generic rational direction and a multiplicity-aware finite pole inventory
legitimize coefficient extraction. This does not authorize direction-based
finite parts for an unproved family: `eta_1/(eta_1+eta_2)` is correctly rejected.

The alternate divisibility certificate checks each bad linear factor to its
full multiplicity, as identities in the other regulator coordinates and one
common transported distribution/face representation. It handles higher and
intersecting poles; a jet solely at the joint origin does not. The
`eta_2^K/eta_1` counterexample exposes precisely that false shortcut. Unrelated
domain integrals cannot be declared equal by ordinary simplification or
floating-point cancellation. No general period-zero procedure is supplied.

## Native implementation boundary

The verified symGCAD probe against Symbolica742 passes signed-factor,
parameter-first, root-selector, exact refinement and tamper controls. It proves
neither closed-face normal form nor arbitrary quantified real geometry.
Native polynomial/series/evaluator primitives may implement certificate
checks, but are not themselves BM, Nash preparation or an auxiliary-regulator
identity engine. Each new operation still requires its focused executable
reuse gate. In particular, a supplied-map checker can be accepted independently
while atlas/certificate production remains unfinished.

No correction to the reviewed conditional claims is required. Acceptance of a
future complete resolver must retain the explicit A3/R4 obligations and cannot
be inferred from A0–A2/R0–R2 examples or passing physical benchmarks.
