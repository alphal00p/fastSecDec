# Resolver G0 addendum: finite real ownership and certified regulator restriction

Research/specification only, 2026-10-10. This supplements
[the resolver design](NO_DEFORMATION_RESOLVER.md); it does not report implemented
functionality or completed general-resolver acceptance. The native reuse probes
and independent algorithm review are separate prerequisites.

The two gaps have different answers. **Finite, disjoint ownership of a supplied
finite blow-up atlas has a concrete construction and a finite checker.** Turning
those domains into the full closed-face endpoint normal form still requires a
substantial preparation algorithm. **Auxiliary-regulator restriction has useful
checkable sufficient certificates and a finite coefficient-extraction rule.**
Neither meromorphic continuation nor numerical cancellation alone supplies a
general certificate of that restriction. The requested general resolver remains
the deliverable; implementing these sufficient cases is a milestone, not a
redefinition of completion.

## 1. A finite domain-ownership algorithm that can actually be encoded

Input is a *finished, finite* relative BM resolution tree, with algebraic charts,
transition maps, smooth-center certificates and a compact semialgebraic original
domain. The tree must resolve the product of the relevant divisors. Its centers
must keep external parameters fixed. The following algorithm does not choose
new centers and does not restart BM when a domain inequality becomes awkward.

For a blow-up with homogeneous coordinates `[h_0:...:h_{r-1}]`, assign a point
to the smallest index attaining the largest absolute coordinate. Its owner
predicate is

\[
 O_i(h):\quad h_i\ne0,\quad
 h_i^2>h_j^2\ (j<i),\qquad h_i^2\ge h_j^2\ (j>i).
\]

This is projectively invariant and uses exact polynomial inequalities. In chart
`i`, set `u_j=h_j/h_i`, so `|u_j|<=1`. Retain the original parent-domain
predicate pulled back to this chart, its previous ownership predicates, and
the actual blow-up equations. A chart of a blow-up along a nonlinear smooth
center is an algebraic chart; do not assume it is globally an affine space with
freely varying coordinates.

Pseudocode for the ownership part is deliberately simple:

```text
owned := [(root chart, original domain, identity)]
for each level of the supplied finite resolution tree:
    next := []
    for each owned parent record:
        for each supplied child projective chart i:
            D := pullback(parent.domain) AND blowup_graph AND O_i
            retain (child, D, parent.map composed with child.map)
    owned := next
return owned
```

Empty domains may be retained until a certified emptiness operation is
available. A numerical sample cannot delete one. If extra coordinate covers
are needed to describe a smooth center, first disjointize that *finite cover*
by `V_i \ (V_0 union ... union V_{i-1})`, retaining the resulting predicates.

The finite checker verifies: homogeneous coordinates are not all zero; the
owner predicates partition projective space; chart equations and transition
identities hold; each nonexceptional parent point has exactly one owned lift;
and the composite map is the recorded map. These statements give an induction
on the **given finite tree depth**. Termination of domain ownership is therefore
independent of a heuristic complexity measure for its inequalities. Termination
of the BM tree is a separate obligation of the precise marked-ideal algorithm.

[Viu-Sos, *A semi-canonical reduction for periods of Kontsevich–Zagier*](https://arxiv.org/pdf/1509.01097),
Remark 2.18 and Algorithm 3, provides a constructive precedent: projective
cube domains are propagated through a finite resolution chart tree. Its
absolutely convergent rational-integral setting yields compact semialgebraic
domains, not the full cube-and-jets contract required here. The paper also
explicitly distinguishes its reduction from deciding whether an arbitrary
period is zero (the final “Zero-detection problem” discussion).

For integration, retain full-dimensional interiors and a ledger of omitted
lower-dimensional seams. First prove the change-of-variables identity in one
common absolutely convergent regulator region, using `|det Dphi|`. Continue
that *identity*, not the individual assertion that a seam has measure zero at
the target exponents. Distributional terms supported on threshold seams can
reappear in the continued identity. They must not be erased because the seam
had zero Lebesgue measure in the initial convergence region.

### What this does not establish

Ownership inequalities introduced early in the tree can meet later exceptional
divisors tangentially. Even if the original divisor is SNC, the final owned
domain need not be a cube, a union of coordinate orthants, or an SNC domain.
The algorithm above proves finite ownership and change-of-variables coverage;
it does **not** prove finite endpoint jets for a subsequent triangular CAD map.
Adding ownership walls to BM repeatedly until everything looks simple has no
termination proof supplied by this construction.

## 2. The remaining endpoint-atlas operation, stated precisely

The next operation takes a finite owned algebraic domain and a finite inventory
of algebraic factors and returns disjoint parametrizations with rational
monomial valuations and nonvanishing units, plus the required finite mixed jets
on every closed face. Its input includes every denominator, leading
coefficient, root discriminant, relevant boundary function and Jacobian. It may
reuse BM/AJ certificates; it must not forget them after a new parametrization.

There is stronger literature than an arbitrary finite covering theorem:
[Cluckers–Miller, *Lebesgue classes and preparation of real constructible
functions*](https://rcluckers.perso.math.cnrs.fr/prints/CM_JFA.pdf),
Theorem 1.5, gives a finite partition with analytic isomorphisms over the
parameters and monomial-unit forms, including the Jacobian. Proposition 5.3
constructs the rectilinearization by controlled elementary maps after
Proposition 2.7's preparation step. A resulting fiber has the form
`B_x=C_x x (0,1)^(d-l)`, with `C_x` bounded away from coordinate zero for each
fixed parameter. This is not automatically a uniform full closed cube as
parameters approach exceptional strata. Its preparation prerequisite and a
native effective algebraic specialization have not been implemented here.

Use that preparation/rectilinearization construction as the **specific
algorithm-development target**, rather than an unspecified repeat-CAD rule.
Before adopting it as an executable completeness argument, discharge these
three items:

1. Give the semialgebraic/Nash specialization of the preparation step in the
   exact native representation, with its induction invariant. Do not introduce
   general restricted-analytic functions merely because the theorem allows
   them. Identify the coefficient fields and finite algebraic extensions used.
2. Derive and certify the final integration representation. Either construct
   full cube maps with the promised jets, or explicitly extend the integration
   contract to rectilinear domains with a regular compact base. A hidden
   indicator of `C_x` inside a cube is not a smooth endpoint remainder.
3. Prove that any final base-cell parametrization preserves the finite-jet
   contract. Boundedness alone is insufficient: algebraic derivatives can blow
   up at a base-cell boundary. Parameter boundaries require separate relative
   strata rather than assuming a uniform distance from zero.

These are serious additional effective operations, not just data conversion.
The current native inventory does not contain this preparation algorithm.
This addendum does not assert that the general endpoint-atlas proof is closed.

The small first deliverable is nevertheless concrete: a native checker for
supplied maps, checking their graph identities, real branch selectors, domain
ownership, determinant sign, rational valuations, unit nonvanishing and
requested jets. Begin with explicitly supplied nontrivial maps. Separate
producer completeness from checker soundness; passing examples proves neither
a termination invariant nor a general atlas-construction theorem.

## 3. Define the common family before partitioning

Fix the compact projective parametrization and its measure first. Let the
original density on that domain have positive factors `B_a`, causal factors
`F_b-i0`, and a bounded analytic/algebraic numerator after all its poles have
been included in the inventory. Define one common family by exponent shifts
on those **original factors**, not independent factors invented in each cell:

\[
 T(\epsilon,\eta)=N(x,\epsilon)
 \prod_a B_a(x)^{p_a(\epsilon)+(C\eta)_a}
 \prod_b(F_b(x)-i0)^{q_b(\epsilon)+(D\eta)_b}.
\]

The matrices and the gauge are identity-bound inputs. A later gauge change
must preserve this whole family, including the measure. Exponent shifts on
homogeneous factors without compensating projective degree are not generally
Cheng–Wu invariant.

The convergence certificate contains a nonempty open regulator region where
every continued representation is the same ordinary absolutely convergent
integral. On a certified compact normal form this can be checked by rational
strict inequalities for every face valuation, together with unit bounds and
complete numerator/pole accounting. The existence of such a region is checked,
not inferred merely from adding several regulator symbols.

All auxiliary powers also occur in the negative-cell phase:
`exp(-i*pi*(q_b(epsilon)+(D eta)_b))`. Replacing the family by positive absolute
powers while freezing its phase is not the same regularization.

For example, on `[-1,1]`, continuation from `Re(eta)>0` gives

\[
 \int_{-1}^{1}(x-i0)^{-1+\eta}dx
 =\frac{1-e^{-i\pi\eta}}{\eta}\longrightarrow i\pi.
\]

This is a mandatory mutation control: keeping the phase fixed at `eta=0`
would remove the finite imaginary contribution.

## 4. A useful sufficient holomorphy certificate, with an actual proof route

For general analytic factors, resolution proves a *meromorphic* family of
distributions with linear poles. It does not prove that a specified auxiliary
slice is removable. [Dang, *Complex powers of analytic functions and
meromorphic renormalization in QFT*](https://arxiv.org/pdf/1503.00995),
Theorems 1.2–1.3, derives this continuation by signed domains and resolution.
The proof of Theorem 5.2 uses the stronger local fact that a noncritical factor
pulls back the entire distribution `(t+i0)^s`. Proposition 5.7 gives a
holomorphic product criterion under wave-front transversality. Its separate
renormalization construction must not be silently substituted for regulator
removal in this project.

The proposed first `OriginalFamilyRegular` certificate uses a stricter,
readily checkable local model, avoiding a general wave-front algorithm:

* A finite compact cover of the physical domain, or a certified proper
  preliminary UV resolution, has coordinates `(r,y,z)` with `r_i>=0` the
  domain-boundary coordinates.
* Every active threshold is a signed coordinate `y_j` times a positive analytic
  unit. The boundary equations and distinct active threshold coordinates have
  independent differentials. Repeated factors of the same coordinate are
  combined with their complete affine exponents only when their causal
  orientation agrees. Opposite prescriptions on the same coordinate are not
  covered by this rule and can produce a pinch.
* The remaining density and units have sufficiently many uniformly controlled
  derivatives on the localized charts; all rational/algebraic poles are
  represented explicitly. The coordinate maps preserve external parameters.
* One-sided boundary powers have exponents
  `A_i+B_i*epsilon+C_i·eta`. For each possible local pole
  `A_i+1+k+B_i*epsilon+C_i·eta`, restriction to `eta=0` must not be identically
  zero as a function of epsilon. If `B_i=0`, this excludes a constant negative
  integer exponent unless another certified cancellation removes it.

Any regulator-dependent external prefactor belongs to this check too. A pole
in an auxiliary-dependent Gamma prefactor cannot be hidden by proving only the
coordinate factors regular. A prefactor meromorphic solely in epsilon can be
retained for the subsequent physical Laurent expansion.

Independence of differentials is checked on the entire relevant stratum, not
only at its CAD sample point. Native polynomial minors, branch certificates,
and real-domain emptiness/sign certificates are suitable evidence. A positive
minor lower bound may be local to a compact parameter subset; it is not uniform
across an excluded collision stratum.

Here is the proof rule the checker is intended to apply. The distributions
`(y_j-i0)^s` are entire in their exponents. The one-sided distributions
`r_{i,+}^s` are meromorphic with poles at the negative integers. Their tensor
product, multiplication by smooth parameter-holomorphic units, coordinate
transport and compact pushforward give a family holomorphic in `eta` near zero
for generic epsilon under the nonresonance condition. Smooth localization is
used in the proof; an analytic compactly supported partition of unity is not
being claimed. The one-dimensional entire-family statement is recorded in
[Clerc–Kobayashi–Ørsted–Pevzner, *Generalized Bernstein–Reznikov integrals*](https://arxiv.org/pdf/0906.2874),
Example 2.5 and Table 2.5.1.

The local normal forms, regularity witnesses and proper transport are the
certificate payload. Merely setting a flag saying “not a Landau point” is not
a certificate. This rule is useful for nonpinched transverse thresholds and
UV faces admitting the stated model. It is not a proof for every ggHH chart,
every algebraic branch collision, or every meromorphically valid input.
For a genuine tangent/pinched stratum, it must return an unresolved obligation
rather than applying the transverse rule.

## 5. Transport the proof, then extract coefficients finitely

Let the original family pass `OriginalFamilyRegular`. Require a common
convergence witness and all finite-map coverage/branch/density certificates.
Then

\[
 T(\epsilon,\eta)=\sum_j (\phi_j)_*T_j(\epsilon,\eta)
\]

holds first in the convergence region and hence as a meromorphic identity.
Consequently the **complete pushforward sum** is holomorphic on the physical
auxiliary slice at generic epsilon, even if individual cell records have
auxiliary poles. This proof does not require finding a closed form for an
integral or identifying equal periods by numerical sampling.

An executable coefficient rule follows once that joint regularity is proved:

1. Native symbolic endpoint subtraction provides a finite local denominator
   inventory, including multiplicities. Work at generic epsilon. Keep the
   regulator dependence of all phases, units, endpoint derivatives and exact
   terms.
2. Choose a rational direction `v` for `eta=t*v` transverse to every remaining
   linear polar factor containing the physical slice. This is a finite exact
   test; the choice belongs to the receipt.
3. Compute a safe maximum negative `t` order `M` from that inventory. Expand
   each numerator far enough to obtain the finite coefficient after its own
   denominator is applied. A whole-expression bound `M` is valid but need not
   be optimal. Preserve all epsilon dependence until this restriction is done.
4. Emit the sum of the `t^0` contributions and a certificate identifying the
   negative powers whose complete pushforward is zero. Individual coefficients
   expected to cancel remain inspectable proof records. They are not claimed
   pointwise zero in unrelated cell coordinates.
5. Only then take the requested epsilon Laurent expansion. For example,
   `1/(epsilon+eta)` restricts to `1/epsilon`; it must not be rejected merely
   because the joint origin is singular, or expanded in the wrong order.

This directional coefficient is legitimate because **joint regularity has
already been proved**. Without it, a constant term along a chosen line is a
new prescription and can be direction-dependent. The function
`eta_1/(eta_1+eta_2)` is a compulsory counterexample. Checking two numerical
directions is a regression control, not a replacement for the proof.

The formula gives a finite implementation path for the certified family class:
native regulator series, native endpoint jets, finite pole bookkeeping and
proof transport. It does not require a new numerical integration engine.
Current single-epsilon endpoint structures need an explicit extension for this
symbolic computation; auxiliary powers cannot be silently encoded as ordinary
runtime parameters.

## 6. A second, local polar-cancellation certificate and its limit

If original-family regularity is unavailable, a producer may instead supply
exact cancellation identities. After bringing terms to a common *certified
pushforward or face representation*, form a common polar denominator. Over
meromorphic functions of generic epsilon, collect its bad linear factors
`ell(eta)` whose restriction to `eta=0` is identically zero, with multiplicities.
The checker requires divisibility of the combined analytic numerator by every
`ell^m`, or an equivalent finite sequence of exact differential/face identities.

For a given `ell`, choose linear coordinates `(ell,z)` in regulator space.
The conditions are `partial_ell^j N(0,z)=0` for `0<=j<m`, **as identities in
the remaining variables and common distribution/face representation**. This
covers higher-order and intersecting polar divisors. A finite Taylor jet only
at the joint origin is insufficient: a term `eta_2^K/eta_1` can evade any
smaller total-degree jet and still have a nonremovable pole.

Acceptable proof nodes are exact algebraic-branch identities, signed face
incidence cancellation with identical transported jets, or an explicitly
supplied native symbolic primitive with checked Stokes boundary terms. The
checker uses native differentiation and algebraic reductions. It must verify
regularity/convergence for the identity, as well as its algebraic expression.
Ordinary simplification of sums on unrelated domains is not this certificate.

There is a genuine completeness limit here. An arbitrary residual polar
coefficient can be a sum of integrals on different domains. Deciding its exact
vanishing can require equality of periods, not merely equality of polynomial
expressions. No general terminating zero-certificate producer for that problem
has been established by this review. This is not a claim of undecidability;
it is a specific unsupported operation that must not be hidden in `simplify`.

[Oaku, *Algorithms for integrals of holonomic functions over domains defined
by polynomial inequalities*](https://www.lab.twcu.ac.jp/oaku/Oaku2013.pdf),
Algorithms 1 and 4, offers a possible substantial future tool: holonomic
systems for polynomial-power distributions and their integrals. An annihilating
system alone does not fix the integration constants; the paper's Example 7
uses boundary data as well. This does not supply a native general period-zero
oracle, and the required noncommutative algorithms are not present in the
current identified Symbolica API inventory.

Accordingly, an unresolved polar identity must produce an explicit proof
frontier. A finite-part projection or a renormalization map is not an authorized
automatic substitute. The general resolver remains incomplete until its
documented admitted-family contract and certificate production have a reviewed
completeness argument, or the exact remaining class is explicitly retained as
an unfinished research deliverable.

## 7. Native reuse and missing effective operations

Read-only source checks use Symbolica
`74225696cd445247fa81c499c5110decd19257ed` and symGCAD `8030f7ef`.
Existing native primitives include `GroebnerBasis::new`/`reduce`, polynomial
resultants, algebraic extensions, rational-exponent `Series`, native symbolic
derivatives, function maps and evaluator composition. These can check local
identities and implement finite jets. They are not a marked-ideal resolver,
an effective Nash preparation implementation, or a distribution identity
engine merely by being available.

Relevant Symbolica files under the pinned checkout are
`src/poly/groebner.rs`, `src/poly/resultant.rs`, `src/poly/series.rs`,
`src/domains/algebraic.rs` and the native evaluator modules. Two existing
caveats remain: algebraic `norm()` delegates to the shifted norm helper, so it
must not be assumed to be the unshifted divisor norm; the public
`poly/factor.rs::hensel_lift` is a finite-field/integer lifting operation, not
the AJ characteristic-zero multivariate series-lifting algorithm.

The symGCAD public `solver::solve`, `verify::verify` and root-isolation APIs
provide evidence for their supported generic open-cell problem. The adapter
owner independently confirmed that they do not constitute arbitrary quantified
semialgebraic elimination, closure smoothness or Nash all-face certification.
The ownership construction has elementary universal identities that can be
checked directly; additional domain emptiness and finite-jet claims still need
an identified supported native operation or a new audited owner operation.
No unchecked exceptional set may be promoted to a complete-domain proof.

The focused native solve/verify probe belongs to the adapter owner and is
separate from this document. No executable claim for the new resolver,
rectilinearization or regulator certificates is made here.

## 8. Concrete milestone ladder and smallest controls

All entries below are **not implemented** by this addendum.

| Milestone | Deliverable and stopping boundary | Smallest nontrivial control |
|---|---|---|
| A0 | Typed finite blow-up ownership records and independent algebraic predicate checker; supplied tree only | Blow up the origin of `[0,1]^2`: `(x,y)=(u,uv)` and `(uv,u)`, deterministic diagonal ownership, absolute Jacobian `u`, source measure and polynomial moments. Mutate one inequality and one determinant sign. |
| A1 | Nonlinear-center chart graph, branch and relative-parameter certificates | Center `(x,y-p*z^2)=0`; preserve `p,z`, compare exact graph/Jacobian under two projective charts. A mutation mixing `p` into a blown-up coordinate must fail. |
| A2 | Supplied algebraic cell plus endpoint certificate through ramification; exercise the entire closed-face inventory | `0<x<1, 0<y<sqrt(x)`: map `x=u^2,y=u*v`, determinant `2u^2`, branch nonnegative; add density `(sqrt(x)-y)^s`. Check both `u=0` and `v=1`, their intersection, and the finite jets after extracting `u^s(1-v)^s`. |
| A3 | Effective Nash preparation/rectilinearization specialization with its own termination proof | Tangential boundary and root-collision families, not only a monomial quadrant. `y^2=x^3` and a colliding-root parameter family require separate exceptional strata. Failure to prove full-cube jets remains explicit. |
| R0 | Native common-family descriptors and endpoint-pole inventory; generic-epsilon restriction ordering | `1/(epsilon+eta)` versus `1/eta`; positive-content scaling and gauge-change mutations. |
| R1 | Original-family transverse holomorphy certificate and meromorphic pushforward identity | `[-1,1]` signed threshold gives `i*pi` above; the product of two independent signed thresholds gives the complete mixed cancellation and `-pi^2`. Freeze an auxiliary phase to force rejection. |
| R2 | Higher-order native subtraction/phase coefficient extraction under the certificate | Test function `1+c*x` against `(x-i0)^(-2+eta)` on `[-1,1]`: continuation has finite value `-2+i*pi*c`; this tests the first derivative of the seam contribution. A second-order polar mutation must not pass a simple-residue-only check. |
| R3 | Exact local polar-divisibility/face-identity certificates, with no integration oracle | Accept a supplied identity divisible by `(eta_1+eta_2)^2`; reject `eta_1/(eta_1+eta_2)` and `eta_2^K/eta_1` despite vanishing low total-degree jets. |
| R4 | Reviewed completeness argument for certificate production on the declared general admitted family | Must explicitly address nontransverse algebraic strata and integrated polar identities. An unresolved general case keeps this milestone open; successful physical examples do not close it. |
| I0 | Native end-to-end fixture and physical validation after the relevant certificates exist | Scalar threshold controls, massive sunrise algebraic cells, selected and full ggHH complex numerator/Laurent vectors; no new performance or physical acceptance is inferred from A/R unit controls. |

The immediate implementable work is A0–A2 and R0–R2 with supplied proof data,
plus the independent verified-GCAD adapter already assigned elsewhere. A3 and
R4 remain explicit major proof/algorithm milestones. This separates a useful
first implementation from the still-required general completion and prevents
an example-driven shortcut from being mislabeled as the requested resolver.
