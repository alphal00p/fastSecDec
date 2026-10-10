# Relative quotient charts and constructive contact opens

This extension checks local affine quotient charts and constructs an ordinary
relative maximum-order/contact-open stage. It is actual algebraic construction,
but it is **not a complete BM center producer or general algebraic resolver**.
It does not select real branches, certify real parameter chambers, construct an
integration atlas or establish endpoint regularity and analytic continuation.

The implementation extends the
[supplied polynomial checker](no-deformation-resolution-checker.md) in
[`threshold/resolution`](../../crates/fastsecdec/src/threshold/resolution/mod.rs).
Its remaining general obligations are those of the
[resolver specification](../NO_DEFORMATION_RESOLVER.md) and
[proof addendum](../NO_DEFORMATION_RESOLVER_PROOFS.md).

## Native objects, admission and relative derivatives

`LocalizedAlgebra` retains a native Q-polynomial ideal, active coordinate roles
and explicit principal-open unit relations. Parameter, active and inverse
roles must be distinct; source equations and guard factors cannot depend on
inverse variables. A unit localized ideal is rejected, so identity checks
cannot succeed vacuously. An optional exact rational real-point check has only
pointwise scope. Nonempty complex algebraic data do not imply a real point in
every parameter fiber.

`EtaleCertificate` selects equations and dependent coordinates, recomputes the
relative Jacobian minor using native matrix arithmetic and admits its unit
open. Every unselected equation must be redundant in the selected-equation
ideal on **that same localization**. Extra constraints cannot be discarded.
Parameters remain inert throughout.

Native `Matrix::solve` constructs each free-coordinate relative derivation.
Native rational-expression normalization clears the determinant, and every
inverse guard is differentiated by its exact unit identity. Native ideal
reduction then checks preservation of all original and inverse relations. The
finite generator inventory also checks mixed commutators. Ring maps and
declared coordinate roles are checked before constructing this frame.

The ordinary affine base case has an empty selected equation system, determinant
one and no artificial inverse variable. Any unexplained source constraint is
still refused. Zero relative dimension is explicitly admitted as a terminal
algebraic chart; it does not require an invented integration coordinate.

This reuses Symbolica's Q-polynomials, exact derivatives, rational normalization,
matrices and Groebner reduction. No separate matrix solver, implicit root
evaluator, ideal engine or automatic-differentiation layer was introduced.

## Constructive stage and incomplete outcomes

The ordinary producer generates inclusive native derivative ideals

\[
 I_0=I,\qquad I_{j+1}=I_j+\sum_i D_i I_j.
\]

Each stage retains a native unit/proper check. If the first unit ideal is
`I_d`, the predecessor `I_(d-1)` must have a completed proper-ideal check. This
establishes maximum relative order `d` on the declared algebraic chart and
identifies its algebraic maximum-order locus. It does not identify the maximum
on a selected real fiber.

The producer takes native generators `h` of `I_(d-1)`, constructs their free
relative derivatives, and checks that the corresponding principal opens cover
the locus. The unit-ideal condition for the complement is exact; the caller
does not provide a guessed list of contact opens. The retained candidates carry
their equations, derivative directions and nonvanishing differentials.

This is an **ordinary stage with no boundary ledger inferred**. It does not
replace the published exceptional-divisor, logarithmic derivative, companion
or full invariant/history stages. `OpenCoverCertificate` alone concerns a
supplied algebraic locus; the producer supplies the additional derivative-ideal
identification needed here.

Unit ideals and ideals zero in the localized quotient have separate outcomes.
The latter is not a zero-valued integral and gives no finite maximum order. A
proper nonzero ideal at zero relative dimension is a terminal parameter/base
locus. Derivative-order or work exhaustion preserves the original source,
completed derivative ideals, checked and unchecked unit statuses, and operation
and allocation counters. No numerical zero or guessed finite order replaces
an unresolved locus.

Preflight covers checker-owned sizes and products. Native matrix normalization
and F4 can still require caller-owned hard resource bounds; the counters do not
claim to limit their internal allocations. All work remains caller-driven.

## Executable evidence and independent review

Public API and native-source inspection preceded focused native probes. The
first isolated feasibility probe passed five controls for finite derivative
opens, relative circle/tower lifts, inverse relations, parameter inertia and
commuting mixed derivatives. It includes collision, forged/missing-minor and
empty-localization rejection; those probes are separate from maintained tests.

The implementation draft then passed **15 tests: nine new controls and the six
existing supplied-checker regressions**, zero ignored, in 0.03 seconds native
test time. Strict direct Clippy passed on the same final source, using native
owner `c540d3f68c90fe7bff1e507458e57a20fb95b11c`.

The nine new controls cover:

- exact two-open coverage of `x²+y²-p=0` on `p != 0`, with each missing-open
  mutation and the exceptional `p=0` failure;
- circle and algebraic-tower first/second/mixed relative derivatives, preserved
  unit relations and inert parameter axes;
- actual redundancy versus an additional unaccounted equation, singular
  minors, vacuous localization, incorrect roles and resource limits;
- inclusive derivative towers, generated contact opens and retained proper/
  unit statuses;
- the genuine affine-plane cusp `I=(x²+y³)`, which produces order two and the
  contact candidate `h=2x` with unit differential;
- parameter-only unresolved loci, resource-stop evidence, unit and zero ideals,
  and zero-relative-dimensional terminal outcomes.

The root and runtime reviewers independently accepted the final construction
and its algebraic scope, including the affine and terminal corrections,
same-localization relation equivalence and retained incomplete states. The
runtime reviewer also accepted the earlier native feasibility probe.
Registered source differs from
the reviewed ignored draft only in module/import paths and import formatting.
The registered code passes its nine new controls within the **44/44** combined
threshold Cargo suite. Strict native Cargo Clippy passes (`--lib --tests`,
warnings denied). The log is
`target/no-deformation-native-request-contact-gates.log`. Workspace formatting
passes after removing one leading blank line in an unrelated options test;
that formatting-only correction did not alter any checked expression or test.

Ignored sources, commands, hashes and logs are retained at
`target/no-deformation-etale-probe/` and
`target/no-deformation-etale-draft/`. The final draft handoff SHA256 is
`3853cf95c4040a8c47a8eb2be353806c3988e4525ea763743d6516e971fcad80`.

## Next concrete dependency and unresolved scope

The next local construction must clear admitted units from a produced contact
equation, adjoin a fresh coordinate `z=h`, verify the new relative Jacobian and
construct its native normal derivation. Finite native normal jets restricted
to `z=0` then supply coefficient-ideal descent on an algebraic quotient, without
assuming a polynomial inverse for `h`. Ring extension, exact denominator
recombination and inherited guards must be checked first.

Coefficient descent decreases relative dimension. That fact is not a proof of
termination for the blowup sequence. Boundary-compatible monomial/residual and
companion stages, the complete published invariant/history and gluing of its
maximal loci remain to be implemented and reviewed.

Algebraic monomial-times-unit recombination is also distinct from an endpoint
certificate. The real closure must lie in the admitted unit chart, with correct
branches, signs, absolute measure, all-face jets and the common analytic
regulator family. A principal-open guard that vanishes at a physical boundary
cannot be silently extended to that face. General real atlas construction,
ramification/AJ lifting and auxiliary-regulator removal remain open.
