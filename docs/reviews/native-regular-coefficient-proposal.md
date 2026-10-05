# Keep regular coefficients compact through native subtraction

This is a source-backed design for a small test-only probe, with its first
symbolic-control outcome recorded below. It is not an approved production
strategy. It follows the original Taylor
whole-graph timeout and the separately bounded existing-IBP preparation failure.
The next question is narrower than another whole-graph attempt: can the existing
native series-first composition operate on named regular coefficients, then
lower only its requested derivative/face bodies into native aliases, while
preserving the complete Laurent vector and native remainder bounds?

## What the two negative experiments establish

Production subtraction repeatedly differentiates and substitutes into each
`Piece.regular`, then groups all resulting physical bodies into one native Atom
(`generation/subtraction.rs`). The failed representative starts with a 1,351-byte
mapped regular factor but produces a 137,835,408-byte Taylor expression before
Laurent expansion. Switching to existing IBP did not finish preparation under
its separate 180-second development/test bound. These observations do not prove
which allocator or individual native operation dominates.

The earlier test-only `subtraction/series_first.rs` already computes native
epsilon series before coordinate differentiation. However, its `Series::map_coeff`
closures still differentiate and copy the complete physical coefficient bodies.
Its negative actual result therefore does not test named coefficient bodies
retained through endpoint assembly. Likewise, the earlier polynomial callback
experiment kept the epsilon-dependent regular density intact through native
series. Neither result establishes the proposed combination.

The active reference direct path separates endpoint assembly requests from
regular coefficient construction. `_g_coefficients_by_symbolic_diff` in
`FastSecDecPathFinder/src/integrand.py:6762` first constructs requested regular
epsilon coefficients, then calls `_local_taylor_coefficient_expr` for requested
coordinate derivatives. `_two_stage_derivative_fused_components` groups those
requests by boundary/zero faces, caches those groups, and substitutes their
bodies into final assembler expressions. It also has custom Python series,
factorial/log coefficient construction and chain-rule composition helpers.
Those helpers are not candidates to port. The useful architectural distinction
is the ordering and reuse of requests, while Symbolica must own the Rust series,
derivatives, substitutions, arithmetic and evaluator.

## Native API and source evidence

* `Series<AtomField>::map_coeff` (`poly/series.rs:717`) retains the native variable,
  expansion point, shift, ramification, field and remainder order, then calls
  native truncation. Its existing `map_coeff` test covers cancellation changing
  the leading exponent. Existing FastSecDec series-first controls additionally
  cover all-zero mapped coefficients and conservative final bounds.
* Native `Atom::derivative` handles ordinary unknown functions and its own
  `der(depths..., function, arguments...)` representation, including repeated
  and composed arguments (`derivative.rs:60–219`). Its native derivative tests
  include mixed/composed requests. The previously executed small formal-function
  controls also verified mixed derivatives and literal face substitutions.
* Native literal `replace_map`/simultaneous replacement can resolve a requested
  partial after calling native derivatives on its original coefficient body.
  No custom chain rule, derivative formula or limit operation is required.
* `AliasedAtom::register_alias` owns exact handle/body definitions. The existing
  production `kernel::program::build` registers one shared native alias map for
  the entire vector, then uses one exact native evaluator for O2, MPFR, workers
  and the validated portable IR. The already executed template-alias controls
  establish this lowering/persistence seam. Do not use the convenience multiple-
  AliasedAtom builder, which would register the same shared definitions repeatedly.

These establish API and source feasibility, plus executable evidence for the
individual seams. The combined named-coefficient composition has not run; the
small probe below is its missing executable reuse check.

## Smallest bounded implementation slice

Keep this entirely under the existing test-only series-first module. Add a
private coefficient representation option to that native composition; leave the
physical-body baseline and production subtraction unchanged.

1. Ask native Series for each admitted mapped regular factor at the current
   requested width. Preserve exact zero and numerical constants. Replace each
   other native coefficient by an ordinary function whose arguments are exactly
   its actual coordinate dependencies. Deduplicate equal native coefficient
   Atoms and retain their original bodies locally. Check that simultaneous
   literal restoration recovers every original native coefficient exactly.
   Reserve names against every input symbol, declared coordinate and regulator.
2. Reuse the same coordinate subtraction/IBP loops and native `map_coeff`,
   addition and multiplication. Native derivatives generate all repeated/mixed
   function requests. Do not multiply out physical coefficient definitions.
   Native endpoint denominators, coordinate powers and common Gamma prefactors
   remain ordinary native series, grouped as before.
3. Resolve each distinct source-function/multiindex/argument tuple with native
   coefficient-body derivatives and simultaneous native coordinate substitution.
   Cache only those native results within this call. Exact native zero/constant
   results may simplify requests; do not infer zeros from a truncated series or
   sampled values. Retain unknown zero tails conservatively. Initial scope keeps
   the existing exact unregulated-endpoint fallback, so opacity cannot bypass or
   create an admission claim for those branches.
4. At the final coefficient boundary, replace remaining requested function/DER
   nodes by fresh plain native alias handles. Store the resolved bodies in one
   shared `AliasedAtom` map; require no formal function or DER node remains in
   roots or definitions. Build with the existing production native program
   helper. No global callbacks or process-lifetime coefficient registry is used.

The formal regular coefficients occur linearly: differentiation, endpoint
substitution, addition, and multiplication by independently known endpoint and
prefactor series. There is no inversion or nonlinear series function of an
opaque regular coefficient. This restriction prevents hidden body identities
from being used to justify an inverse or a new pole order. Hidden cancellations
may retain extra terms and reduce efficiency; they do not authorize dropping an
unknown remainder. Always require the final native absolute bound to exceed the
requested order, use checked native-bound retries, and reject fractional orders.

All bodies inherit the existing admitted input's joint regularity premise at
epsilon zero and the relevant coordinate faces. This is not a facility for
arbitrary moving singularities such as `1/(x+epsilon)`. Face specialization is
performed symbolically on the actual native body before numerical compilation;
there must be no inactive `0*log(0)` branch hidden in an evaluator definition.

## Scientific controls and stop conditions

The first run is small controls only, under a fixed 180-second/five-second-grace,
30-GiB virtual-address bound with one exclusive Symbolica process. Freeze source,
binary, dependencies and inputs; no allocator/feature change. No actual captured
representative or whole graph runs automatically afterward.

Compare every requested Laurent order with the unchanged physical-body native
series-first baseline and ordinary production subtraction on small admitted
densities. Cover Taylor and IBP, Gamma poles, a negative requested maximum,
mixed/composed derivatives and multiple faces, a polynomial remainder that is
exactly zero, an unregulated-axis admission control, and the existing explicit
fractional/error controls. Include a hidden complex coefficient and a compact
high-degree factor. Use exact native coefficient restoration on bounded small
expressions; do not expand a large body merely to force structural identity.

Require nonempty known order sets for nonzero controls, complete order unions,
native remainder coverage, MPFR precision agreement, and the same fresh/decoded
weighted production checks. A fresh-process exact-IR reader must operate without
any symbolic function registry. Report regular-series time, formal assembly time,
unique derivative/face requests, resolved body bytes, output root bytes, total
definitions, native IR bytes and compile/precision costs separately. Native-only
alias bookkeeping is measured too; small success is not a throughput claim.

If the small controls pass, a separately reviewed actual-target proposal can
reuse captured index 80 without first building its huge physical subtraction
expression. Scientific acceptance still needs a source-bound complete-vector
oracle and eventual full-integral controls; the unavailable own-IBP expression
from the timed-out preparation cannot serve as an oracle. No production promotion
or whole-graph scheduling follows from this source proposal alone.

## First small symbolic gate

The initial test-only implementation adds an optional named-coefficient
representation to the existing native series-first composition. It preserves
the exact fallback and native arithmetic, checks every scalar-series multiplier
for independence from formal names, and resolves native derivative/face requests
locally into shared `AliasedAtom` definitions. It does not register global
callbacks or change a production default. Existing complete-vector controls now
also compare named coefficients against the physical-body composition and
ordinary production subtraction. Three additional controls exercise native
composed/mixed derivatives and faces, rejection of a formal scalar multiplier,
and explicit fractional/essential/unregulated errors.

After independent source and concrete-wrapper review and the external rank-two
generation handoff, `output/diagnostics/native-named-coefficients-small-1/` ran
the eight normal controls with the existing two large replay tests still ignored.
All eight passed; the process exited zero in 0.125755668 seconds, with native
peak RSS 15,360 KiB. All 15 immutable pre/post hashes and timer identity checks
passed. The bound was unchanged at 180 seconds plus five seconds of grace and
30 GiB virtual address space, CPU8. The copied test binary and exact source/native
archives retain the development/test profile with native dependencies at opt2;
these tiny durations are correctness diagnostics, not performance evidence.

This closes the first symbolic-control gate only. Production O2/MPFR and weighted
kernel checks, a fresh-process native-IR reader, and any actual captured-target
or full-graph acceptance remain pending. The named bookkeeping currently scans
its reserved namespace from zero for each fresh name; owner-local monotone
counters are a reviewed future bookkeeping improvement before a large attempt,
not a change made to the frozen successful control snapshot.
