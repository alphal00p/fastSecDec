# Local monomial blowup construction

The `threshold::resolution::blowup` modules construct adapted algebraic opens
and all standard pivot charts from a verified monomial center. These overlapping
opens are not a disjoint real integration atlas. General BM invariant selection,
center gluing, algebraic branch normalization and endpoint certificates remain
required work.

The construction reuses native Symbolica polynomial substitution, localized
ideal membership, matrix determinants and the existing relative étale-frame
checker. The source incidence/minor opens pass a separate native unit-ideal
cover check. The graph equations clear only certified units. Every target frame
checks pulled relations and guards; each controlled generator and old divisor
recombines exactly. A shared private matrix helper serves both the existing
boundary-rank checker and new charts, avoiding duplicated arithmetic and bounds.

The standard blowup determinant is `e^(c-1)` in adapted coordinates. A separately
computed determinant in the original relative coordinate frame verifies the
inverse adaptation factor as well. Neither determinant includes an earlier
projective map. Real branch selection, absolute orientation and complete measure
assembly are later integration responsibilities.

Immutable histories retain a fixed old-boundary snapshot and issue new IDs only
through checked transitions. A history root, parent chart path and center qualify
each born identity; bare integer equality across sibling paths does not assert
global divisor gluing. Effective Cartier centers preserve the coordinate map
while still dividing the marked ideal and recording the transition. Empty
exceptional divisors remain explicit receipts and are excluded from the active
divisor ledger. Resource-incomplete results retain completed charts and an
unfinished inner boundary check rather than claiming a completed cover.

The mathematical inputs are [BM 2008, §§2 and 5, Step II.A](https://ems.press/content/serial-article-files/41043)
and [Stacks Project, blowups](https://stacks.math.columbia.edu/tag/01OF): standard
charts, flat base change, the complement of the center and the Cartier case.
They justify this local construction; they do not make it a complete resolver.

Ten pre-registration native controls pass, including nonlinear and algebraic
relative frames, all pivots, multiple rank opens, guarded parameters, consecutive
births, empty exceptional loci, wrong owners, matrix shape/ring checks and
resource stops. Runtime and root reviews accepted the scoped construction.
The root's requested full open-cover and original-coordinate Jacobian checks
were added before acceptance. The 38 integrated resolver controls pass,
including all ten new blowup controls and existing contact, geometry and
supplied-certificate checks. Strict native library/test Clippy and workspace
formatting pass after removing one import made redundant by registration.
No mathematical source changed after that test run.

Raw source hashes, owner identity, commands, intermediate failures and results
remain ignored in `target/no-deformation-blowup-producer/`. Native Gröbner
internal allocations still require caller process limits; these small controls
do not establish aggregate-RSS scaling or a production integration result.
