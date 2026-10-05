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

These established API and source feasibility, plus executable evidence for the
individual seams. The combined named-coefficient composition was then checked
by the small symbolic gate recorded below.

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

This closed the first symbolic-control gate only. At that point production
O2/MPFR and weighted kernel checks, a fresh-process native-IR reader, and any
actual captured-target or full-graph acceptance remained pending. The frozen
first snapshot scanned its reserved namespace from zero for each fresh name.
The next test-only layer uses collision-checked, owner-local monotone counters;
it leaves all native mathematical operations unchanged.

## Native program and cold-process control protocol

The next layer reuses the existing production exact-program builder and
`SectorKernel`, with separate Taylor and IBP controls for the admitted density
with prefactor `Gamma(2 epsilon)`, regular body
`(2+3i)(1+x+y)^(-1-epsilon)`, and coordinate powers
`[epsilon-2, 2 epsilon-1]`. It requires the complete order set `[-3,-2,-1,0]`
and an eight-component real/imaginary numerical layout. Bounded small native
coefficient restoration is checked exactly against ordinary subtraction before
building either numerical program. Complex literals must occur in retained
native alias bodies.

Both the ordinary baseline and named program must agree at 512 and 1024 bits
at all three prescribed rational points. Fresh kernels, native fallible worker
clones, same-process decoded kernels, and a separate cold reader check adaptive
and forced weighted evaluation at the identical rounded binary64 points, with
weights one and `1e40`. The rounded-coordinate comparison has its own native
1024-bit reference; it is distinct from the exact-rational checks. The cold
reader consumes native exact IR and expected values without constructing the
formal coefficient registry. This tests native IR transport, not graph-artifact
metadata or a full integration.

The first frozen program attempt is retained at
`output/diagnostics/native-named-program-1/`. Its eight symbolic controls passed
in 0.123431510 seconds, then the writer failed to launch: the timer canonicalized
the Nix `env` symlink to the multicall `coreutils` executable, losing the applet
selection. That process exited one after 0.002723759 seconds; no writer test or
reader ran. All 15 immutable postchecks passed. The scientific binary and
source were unchanged for the corrected launcher in
`output/diagnostics/native-named-program-2/`: per-stage test variables are exported
by the wrapper and `prlimit` remains the first executable. Each sequential stage
retains its own 180-second/five-second-grace, 30-GiB address-space, CPU8 bound.
The timer applet-resolution defect is a separate harness follow-up; the frozen
timer and previous timing evidence were not modified.

The corrected attempt passed all three stages: eight symbolic controls in
0.126907282 seconds, the writer in 15.663043777 seconds (24,628 KiB peak RSS),
and the separate fresh reader in 4.441059926 seconds (21,616 KiB peak RSS).
Every process exited zero and was reaped before its successor. All 17 frozen
source/build checks and all five writer-output postchecks passed. These are
development/test correctness durations, not performance acceptance.

Both strategies retained four nonempty orders `[-3,-2,-1,0]`, reached native
absolute bound one after the recorded width-one/width-four requests, and passed
all exact small-coefficient baseline identities. The ordinary baseline and
named program each passed the 512/1024-bit checks at three exact points.
The writer made 72 complete weighted vector comparisons across fresh, cloned
worker and decoded kernels; the cold reader added 24, totaling 96 vectors and
768 real/imaginary component comparisons. Each kernel/strategy checked and
rescued ten of its twelve calls. The native reports retain 53/256/320-bit choices
for Taylor and 53/256 for IBP; the two unchecked calls were still compared with
the independently computed rounded-coordinate MPFR values.

| Small control | Native source bodies | Derivative / face requests | Aliases | Root / definition bytes | Native IR bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Taylor | 4 | 8 / 19 | 10 | 1,697 / 420 | 2,105 |
| IBP | 4 | 8 / 21 | 10 | 1,597 / 522 | 2,353 |

The coefficient names and body caches are call-local. The cold reader operated
without rebuilding them or registering callbacks. This closes the small native
program, weighted replay, worker clone and fresh-process IR compatibility layer.
It does not establish the cost or correctness of the captured large workload;
that still needs a separately reviewed, bounded complete-vector experiment and
independent original-expression oracles. Production generation is unchanged.
