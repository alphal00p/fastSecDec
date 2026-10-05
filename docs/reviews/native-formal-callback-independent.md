# Native polynomial callback proof: independent review

This is a source review of a disconnected Rust capability experiment, not a
production change or a performance acceptance. The preceding actual-representative
attempt retained its negative outcome: late constant-face cleanup took roughly
126.7 seconds, left a 17.1 MB expression, and the subsequent native series stage
timed out at 180 seconds without a coefficient vector. Its ordinary production
expression was only 14.5 MB. Smaller source bodies alone did not establish a win.

The existing public `SymbolBuilder::with_derivative_function` and
`with_normalization_function` are the relevant reuse seam. Source inspection of
Symbolica `derivative.rs:60–219` confirms that the derivative callback sees the
original expression: either the ordinary function call or the native `DER` tag
with its existing multiindex. The index identifies an argument of the underlying
function. Symbolica multiplies the returned partial derivative by the coordinate
derivative of that argument and sums the chain-rule terms. An unset result falls
back to the native derivative tag. Therefore an adapter must return only the
partial, not implement or multiply another chain rule.

The small `output/probes/formal_callbacks.rs` follows that contract. It selects
the polynomial body's actual coordinate dependencies as function arguments,
uses native structural polynomial admission, and caches native derivatives and
literal simultaneous face substitutions. Hooks return only exact numeric
zero/constant results; nonconstant bodies leave the setter untouched. Existing
native differentiation, normalization, series and FunctionMap construction own
the algebra. The callback closure is caller-created immutable polynomial data
with synchronized caches, satisfying the native Send/Sync/static contract.

The source controls include an absent coordinate, a derivative above polynomial
degree, repeated native derivative tags, mixed derivatives with composed
arguments, an exact zero-coordinate face, a nonempty analytic Gamma control,
and an admitted positive rational residual. Always and Never each preserve the
complete `[-3,-2,-1,0]` vector and are compared against restored native bodies.
The existing full-vector numeric helper supplies O2, eager, conditioning,
weighted MPFR and worker checks. A separate reader deserializes the native exact
IR with **no callback registration**: numeric persistence must own the resolved
bodies, rather than depend on a process-global symbolic callback registry.

One boundary remains explicit: a normalization hook attached to an ordinary
polynomial function is not automatically a normalizer for every `DER` call on
that function. Late native derivative-face resolution still exists. The source
review does not claim that callbacks eliminate every face cleanup or recover
the difficult representative. Execution evidence and any actual-case adaptation
must be reviewed separately. No dependency patch or custom derivative/series
engine is warranted by the reviewed source.

The second small executable proof is now independently checked. Its frozen
source/binary manifest verifies, both process records report exit zero and
unchanged inputs, and the writer/reader take 2.538/1.552 seconds respectively.
All four analytic/rational × Always/Never records contain exactly
`[-3,-2,-1,0]`, with 32 finite real/imaginary numeric values per record across
the retained point and weighted checks; the vectors are nonzero. The cold reader
has no symbolic callback registration. These observations establish the small
native API/persistence contract, not actual-representative performance.

The first attempt's zero-decision counter was zero because its above-degree
control simplified to a constant before a zero derivative callback was needed.
That incomplete coverage is retained. The second attempt adds `R=1+x*y^2`:
the first x derivative remains the nonconstant native `DER` body `y^2`, and
the second x derivative reaches the hook and returns exact zero. Its separate
cache records three calls, no constant decisions, and one zero decision.
The positive assertion now exercises the intended early-zero path. Evidence is
under `output/diagnostics/formal-functions/callback-small-2/`; no production
algebra or dependency source was changed for this experiment.

The callback lifetime is a further production boundary. Symbolica's symbol
registry retains registered `'static` hooks; strong `Arc` captures of polynomial
bodies/caches can therefore survive for the process lifetime. The bounded proof
does not establish bounded memory across repeated caller jobs. A production
adapter would need bounded caller-owned cache lifetime and proof that unresolved
callback symbols cannot escape into persisted evaluators. Resetting global
Symbolica state is not an acceptable cleanup strategy. This caveat does not
invalidate the isolated writer/reader experiment, but blocks treating it as a
ready production ownership design.

The follow-up actual-case source `formal_actual_callbacks.rs` and its source
adapter pass a separate bounded-probe review. They use the same reviewed hooks,
restrict arguments to actual coordinate dependencies, and keep the same live
registry through production subtraction and native relative series in one
180-second generation process. Captured native powers, input hashes, original
production subtraction identity and cancellation metadata remain bound to the
previous experiment. Only admitted epsilon-independent polynomial bodies are
hidden. Literal indexed native replacement resolves exact constant faces and
then creates ordinary function slots; callback-bearing symbols are forbidden
in exported coefficients. Existing cold native-IR evaluation retains the full
order union and compares all three points with the independent original-body
oracle. This is approval to measure the disconnected proof, not acceptance of
its outcome, production memory ownership or performance. It does not hide whole
logarithmic coefficient bodies or introduce a derivative/series engine.

The actual-case attempt now has a negative, independently checked outcome.
`actual-callback-1/process-generate/report.json` records an owned-process timeout
after 180.147622 s, signal 2, peak 2,292,320 KiB, unchanged inputs/build/binary
and a verified frozen manifest. Its prepared record completes in 6.617374 s:
1.303637 s for the original subtraction identity control, 3.654013 s for formal
subtraction, and 1.572546 s for constant faces. The native hooks made 4,161 and
3,802 exact-zero decisions for the two polynomial sources, so the intended path
was exercised on the actual captured representative.

Nevertheless the remaining expression was 16,557,990 bytes, with 384 formal
pieces versus 241 original pieces and unchanged cancellation tuples. The last
series progress record is incomplete at `native_relative_series`; no coefficient
or evaluator-IR files were produced and no downstream build/evaluation was
attempted. This validates the preparation improvement in the disclosed
diagnostic setup, but does not establish full-vector correctness or a solution
to the difficult native series step. The experiment remains disconnected and
the failure is retained under `output/diagnostics/formal-functions/actual-callback-1`.
