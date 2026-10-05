# Captured-sector native polynomial-function prototype

This is an ignored, bounded experiment, not a production strategy change. It
tests the native FunctionMap route on the actual on-shell representative that
exposed the large Laurent expressions. The small compatibility proof is recorded
separately in `native-formal-function-proof.md`; its successful raw binary64
checks are scoped to their stated tolerances and do not establish a general
accuracy guarantee.

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
