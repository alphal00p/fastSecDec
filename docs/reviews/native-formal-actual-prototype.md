# Captured-sector native polynomial-function prototype

This is an ignored, bounded experiment, not a production strategy change. It
tests the native FunctionMap route on the actual on-shell representative that
exposed the large Laurent expressions. The small compatibility proof is recorded
separately in `native-formal-function-proof.md`; its successful raw binary64
checks are scoped to their stated tolerances and do not establish a general
accuracy guarantee.

There is also an ownership gate before any production adoption: native
Symbolica stores symbol callbacks in process-global state. The proof's strong
`Arc` captures therefore retain polynomial/partial caches for the process
lifetime. This is bounded by the disconnected process, but is unsuitable as an
unbounded per-generation ownership policy in a long-lived HEPKit application.
A future production proposal must demonstrate scoped weak-owned callback state
after all escaping expressions are resolved, or another native-safe bounded
ownership scheme. Unsafe global Symbolica reset is not an option.

The input is mapped representative index 38 (displayed ordinal 39), original
chart index 46, with nine coordinates and highest requested order zero. Its
representative multiplicity is two, but **no multiplicity is applied** in this
pointwise experiment or its independent oracle. The mapped capture and original
production-subtraction capture must have identical fixture bytes, association,
native parameter symbols and regulator. Every imported source is hashed.

The adapter uses Symbolica's structural `is_polynomial` admission without
enumerating monomial support. It wraps only epsilon-independent polynomial
bodies in the already admitted mapped regular expression. Endpoint powers,
prefactor, coordinate measure and original domain certificates stay outside
that substitution. Equal native bodies share a name. Exact literal restoration
must reproduce the mapped expression before any derivative work. Generated
source/partial names are checked against existing symbols.

The standalone program directly includes the production subtraction source,
the existing native relative-depth replay helper, and the production
cancellation-metadata helper. It implements no derivative rule or series
arithmetic. Native derivatives and literal simultaneous coordinate substitution
produce and memoize partial/face polynomial bodies. Only exact native constants
(including zero) are simplified. Because opacity can hide exact zero branches,
this first experiment rejects an unregulated singular axis before wrapping;
that is an experiment scope restriction, not a new public admission policy.

Native relative series must report an absolute remainder beyond order zero.
All expected orders −5 through zero must exist; additional native orders are
retained and compared against zero through the union with the independent
oracle. There is no sampled pole removal. Final retained bodies use fresh user
function slots, avoiding the fixed-builtin DER lowering limitation. Always and
Never use the same explicit direct-translation setting and O2. Native exact
evaluator bincode owns the retained bodies for cold loading, numerical worker
cloning, native error tracking and MPFR; there is no parallel definition
serializer in that numerical path.

The three prescribed points are exact native rationals:

- `[1/2, 1/3, ..., 1/10]`;
- `[1/10^20, 37/100, ..., 37/100]`;
- alternating `1/10^8` and `73/100` in nine coordinates.

The separately authored oracle substitutes these coordinates into the original
captured epsilon-dependent production expression before native series. Its
complete six-order results are bound to the same capture/source hashes. The
prototype compares 512/1024-bit values with `1e-70 * max(1, scale)` agreement and
checks that the retained cancellation bound leaves sufficient effective bits.
Raw eager/O2 values see binary64-rounded coordinates and remain visible even
when they are inaccurate; they are not a substitute for those exact-rational
MPFR checks. Multiplication by `1e40` before conversion tests native weighted
arithmetic only, not production adaptive replay selection.

The frozen first attempt is `output/diagnostics/formal-functions/actual-1`.
It retains source copies, binary/library hashes, compiler output, native
transports, and per-stage process reports. Each prepare, series, builder and
point-evaluation stage has its own 180-second bound, five-second cleanup grace,
and 30 GiB virtual-address limit. The memory limit is not an RSS or process-tree
limit. This is an opt2 standalone proof linked against optimized native/debug
libraries, not a release performance benchmark. Symbolica is revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` plus four documented corrections;
SymJIT is 2.26.4. The successful compiler/linker paths and earlier environmental
link failures are recorded in the attempt directory.

Initial preparation evidence already exposes a cost: the unchanged production
subtraction exactly reproduces the captured expression in 1.332 s and has 241
pieces. Formal polynomial subtraction takes 16.703 s, retains 384 pieces, and
has 232,093,044 Atom bytes before constant-face simplification. The cancellation
tuples are identical. Thus retaining opaque calls without early native zero
pruning can enlarge the expression; no speed or size improvement is claimed.
Preparation completed in 144.909 s with peak waited-child RSS 1,139,600 KiB.
Late native constant-face processing took 126.707 s, reducing the expression to
17,103,500 Atom bytes, still larger than the original 14,505,102 bytes. The two
source bodies total 1,257 bytes; the adapter cached 108 native partials and 768
partial/argument tuples. These timings include the current adapter's many
literal replacement rules, not just polynomial arithmetic.

The unchanged separate native-relative-series stage hit its 180-second limit
and was reaped at 180.090 s. Peak waited-child RSS was 2,420,888 KiB; native
accounting recorded 51.199 user CPU seconds and 127.687 system CPU seconds. The
stage produced no coefficient vector, builder, JIT, or actual-capture oracle
comparison. Its progress marker covers the two native relative-depth calls
together, so this result does not isolate which call dominated. There was no
deadline extension or retry under changed settings. Raw evidence is
`actual-1/process-series/report.json`; the failed attempt remains preserved.

This first actual polynomial-opacity route is a negative result. The next
authorized disconnected control will test public native derivative and
normalization callbacks to preserve zero/constant polynomial pruning earlier,
and actual coordinate dependency arity to avoid false dependence. Symbolica's
callback receives the original derivative-tagged view and the differentiated
argument index before the native chain factor is applied; native polynomial
differentiation and substitution can therefore supply exact zero/constant
decisions without implementing a derivative rule. Callback persistence and
complete Laurent vectors still require focused proof. No production change is
recommended from the current attempt.

## Native callback controls

The follow-up uses public `SymbolBuilder::with_derivative_function` and
`with_normalization_function`. The derivative hook receives the original call,
including native DER depths, and the argument index. It asks native polynomial
differentiation and literal substitution for that next partial. Only exact
numeric zero or constants are returned; other cases leave the output untouched
for native DER fallback. Symbolica still multiplies the argument chain factor
and sums chain terms. The normalization hook handles ordinary polynomial calls;
it does not automatically normalize every DER-tagged face, so later native face
resolution remains part of the design.

The disconnected `output/probes/formal_callbacks.rs` controls passed, with a
coverage correction retained honestly. The first writer/cold-reader pair passed
but counters showed no actual zero-returning callback: its above-degree
derivatives had first collapsed to constants. The second attempt adds
`R=1+x*y^2`, whose second x derivative reaches a DER-tagged callback and returns
zero, with an explicit positive zero-counter assertion. Both attempts remain
under `output/diagnostics/formal-functions/callback-small-{1,2}`.

The final writer completed in 2.538 s and the independent cold reader in
1.552 s. All four analytic/rational × Always/Never
records have nonempty orders `[-3,-2,-1,0]`. Restored coefficients agree exactly
with the concrete native expression; the analytic case also agrees with its
independent Gamma expression. Controls cover minimal dependency arity, repeated
and composed mixed derivatives, native literal-zero faces, ordinary O2/eager
checks, numerical worker clones, and native weighted MPFR. The reader registers
native special functions but no custom polynomial callbacks or body registry;
it loads only native exact evaluator IR and repeats the numerical checks.

The three small source bodies have arities 2, 1, 2 and recorded 27 callback
invocations, 11 cached native partials, 18 cached partial/argument tuples, 14
constant decisions and one zero decision. Total instrumented callback time was
approximately 0.45 ms. This establishes the narrow hook contract; it does not
predict the cost or size of the actual nine-dimensional representative.

## Remaining differences from the reference direct path

The source-only cross-check uses the frozen reference
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/src/integrand.py` and
`subtraction_formula.py`. Its helper name mentions two stages, but
`build_explicit_sector_formula` (`integrand.py:7827–7902`) substitutes source
derivatives and regular-coefficient expressions before building **one** explicit
multi-output evaluator. This observation does not recommend the rejected dual
U/F runtime.

1. The reference expands epsilon in an endpoint assembler containing scalar
   regular-coefficient symbols, keyed by face, coordinate derivative and regular
   epsilon order (`subtraction_formula.py:2267–2359`). The actual callback
   prototype still differentiates the epsilon-dependent regular function through
   sequential subtraction, then asks native series to process the resulting
   entire expression. Its hooks remove exact polynomial zeros/constants, but
   do not remove repeated differentiation of rational powers or repeated
   logarithmic coefficient construction. If callbacks preserve correctness but
   leave large final Atoms, the next meaningful comparison is the native
   regular-coefficient/endpoint-assembler composition boundary, not further
   changes to the small polynomial callback cache.

2. The reference groups only requested regular coefficients by endpoint face,
   forms the ancestor-closed requested derivative set, and processes that face
   before constructing the regular coefficient (`integrand.py:7008–7080`,
   `7110–7231`). On the decomposed-residual path, it differentiates the explicit
   polynomial, substitutes endpoint values immediately, and omits exact zeros.
   The current callback wrapper is chosen once for the interior polynomial.
   Ordinary-call normalization handles constant faces, but a native
   `DER(P,...,0,...)` face is resolved later, and a nonconstant face polynomial
   may lose dependencies while its old function-call argument list retains
   them. This can obstruct production's `contains(coordinate)` exact-remainder
   pruning even with minimal interior arity. The approved prototype records
   piece counts and cancellation tuples to expose this; it does not claim the
   face-dependency issue is solved.

3. Reference regular coefficients are requested by `(derivative multi-index,
   epsilon order)` and constructed from a fixed power prefactor and a combined
   logarithm (`integrand.py:6762–6830`), with a per-face cache. The callback
   prototype memoizes only native polynomial partials and their argument tuples.
   It does not share the larger differentiated regular coefficient across
   endpoint pieces. This is a distinct possible source of expression growth.
   Any Rust follow-up must retain native Series ownership rather than porting
   the reference's Python sparse-series dictionaries or manual convolution.

4. For its alternative original-parameter source path, the reference excludes
   impossible polynomial multiindices using native monomial support and prunes
   inactive map coordinates (`integrand.py:2423–2462`, `7086–7153`). The callback
   adapter uses native structural dependency arity and asks native derivatives
   lazily, so it avoids eager support enumeration. Its partial cache presently
   differentiates each previously unseen multiindex from the original small
   body rather than reusing a cached ancestor. This is a bounded bookkeeping
   opportunity only if measured native-partial work dominates; it is not a
   reason to build a custom derivative engine.

The new adapter's indexed native `replace_map` avoids the first attempt's
many-rule scan over the complete expanded call expression. The reference itself
also uses simultaneous replacement rules (`integrand.py:361–365`), so no blanket
claim about superior reference substitution complexity follows from its source.
Only the actual bounded measurements can attribute the remaining costs.

## Actual callback attempt

The reviewed follow-up is frozen under
`output/diagnostics/formal-functions/actual-callback-1`. A single generation
process retains its live callbacks through production subtraction and native
epsilon series under one total 180-second limit. Diagnostic callback Atom
exports are never used for callback-free reconstruction. Before portable output,
all source and DER calls are resolved to plain slots; assertions forbid callback
symbols in both output coefficients and retained definition bodies. The native
exact IR remains the only numerical body transport.

The preparation portion completed in 6.617 s: subtraction 3.654 s and indexed
constant-face substitution 1.573 s. The two source polynomials have native
coordinate arities eight and nine. Their callbacks recorded 127,311 invocations,
7,963 exact zero decisions, 70 cached native partials, 464 cached partial/argument
tuples, and 0.141 s of instrumented callback time. This is diagnostic attribution
under the recorded host/build conditions, not a production benchmark ratio.

The pre-cleanup expression is 56,027,215 Atom bytes and the remaining expression
16,557,990 bytes. Piece counts remain 384 formal versus 241 production, with
identical conservative cancellation tuples. Thus the improved early-zero and
lookup behavior does not establish compactness of the later native series or
restore every exact remainder-pruning opportunity.

The combined generation hit the unchanged 180-second bound and was reaped at
180.148 s while in native relative series. Peak waited-child RSS was 2,292,320
KiB (53.408 user CPU seconds, 125.577 system CPU seconds). No complete Laurent
vector was produced, so builder, JIT, cold reload and three-point oracle checks
were not run. This is another retained negative actual-workload result, despite
the useful preparation improvement and passing small callback controls.

A further comparison limitation is explicit: this attempt sends the complete
16.56 MB formal subtraction directly to native series. The historical successful
37.33-second relative replay instead operated on production's 5.74 MB template,
after all epsilon-independent coordinate subexpressions had become scalar
placeholders. Polynomial FunctionMap opacity is not that same transformation.
Therefore a slow result here cannot by itself attribute the difference to the
native callback or retained-body mechanism. Reusing the existing late-template
boundary is a separate possible controlled trial, not a change made during this
frozen run.
