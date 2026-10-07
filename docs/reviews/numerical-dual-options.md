# Numerical-dual generation options

## CLI and Python boundary

The run card reuses native `GenerationMode` and `SubtractionStrategy` directly:
`generation.mode = "symbolic" | "numerical_dual"` and
`generation.subtraction = "taylor" | "integrate_by_parts"`. Historical cards keep
`symbolic` and `taylor`. The CLI forwards both fields to `GenerationOptions`
before entering its existing caller-owned dispatch. It adds no worker pool,
subtraction implementation, expression evaluator or numerical adapter.

Python `Integral.generate`, `Integral.generation_session` and the free
`sector_decompose` function accept the same keywords. Synchronous and retained
entrypoints share one options constructor; the native enums own deserialization.
Unknown selections fail before generation observers and parameterization.
Retained sessions and generated owners expose the requested selections as
read-only properties and in their existing native rich displays. Construction
and display still perform no generation or compilation. Supported stub metadata
includes the new keywords and properties.

Both coefficient-expansion selections remain admissible. Numerical-dual formal
recipes use the existing coefficient-series composer, including its configured
request/series limits. A saved *requested* full-expression method does not claim
that the symbolic full-expression route ran. Generation mode, subtraction,
evaluator backend and integration periodization remain separate choices.

## Human metadata and compatibility

`GenerationRecord` adds optional `mode` and `subtraction` fields. Older producers
remain readable; absence is shown as “Not recorded”, not inferred from today's
defaults. Both final generation and inspection reuse the same formatting helper.
Actual source-chart execution modes are retained separately from the requested
mode. Cold sector inspection joins them through native source IDs, avoiding
position assumptions after exact-sector folding. Folded zero-dimensional charts
retain their actual symbolic admission route even without a numerical kernel;
full inspection includes these charts in its mode counts. Deferred charts label regular
storage as source bytes; symbolic charts label it as mapped bytes. Python
inspection likewise exposes the actual sector mode and regular-expression basis.
JSON reports retain the typed saved record. This observation record stays
outside the artifact's mathematical identity, as existing generation workers,
timings and requested coefficient settings already do. The native compiled
payload remains authoritative for evaluation and kernel identity; changing human
metadata cannot switch its execution lane. No binary serialization schema was
changed by this slice.

## Independent architecture checks

The mathematical review found that a leading boundary jet cannot replace the
full deflated residual at interior coordinates. For the symmetric overlap
integral with `F=x+y+xy`, the sector map gives `F=s(1+t+st)`. Retaining only the
leading coefficient `1+t` keeps the residue but changes the finite coefficient
from `1-log(3)` to `1-log(2)`. Correct native jet deflation must retain the needed
shifted coefficients and factorial normalization, and mixed subtractions need
boundary-face functions as well as the all-zero corner.

Korobov-3 is safest after subtraction. If applied first, its fourth-order
endpoint map changes each exponent from `a+b*eps` to `4*a+3+4*b*eps` and changes
regular boundary coefficients. The logarithmic case produces `35^eps/eps` in
its boundary term; the transformed remainder cancels the extra `log(35)` finite
part. Native `fastsecdec-qmc::Korobov3` formulas were inspected, and independent
bounded arithmetic checks reproduced both finite-part identities. This is a
correctness review, not a performance claim.

Explicit Python coefficient inspection now delegates deferred materialization
to `GeneratedSector::coefficients()[index]`; ordinary symbolic sectors retain
their selected-coefficient `AliasedAtom::into_inner` route. Passive views never
call the materializing accessor. Native local symbolic fallback for exact
unregulated endpoint admission or signed monomial maps is reported, not hidden.
The numerical lane retains source charts without symbolic density symmetry.

The synchronous compile and retained `CompilationSession` paths must consume
the same native numerical-dual representation. Formal recipe placeholders must
not become unresolved physical parameters. Explicit coefficient inspection must
use the native deferred owner to materialize the requested expression rather
than implement another symbolic expansion in the binding. These points belong to the core lane's
acceptance, not to option parsing alone.

## Validation

On this host `nix-shell` is unavailable; checks used Rust 1.99 and the existing
Apple Clang linker override.

- Focused CLI native enum/default/strict-name test: passed.
- Focused human artifact historical-omission, pair round-trip, requested-setting
  display and identity-exclusion test: passed for the mode/subtraction fields.
  The later extension covering per-source-chart modes also passes the integrated rerun.
- Isolated Python binding `cargo check`: passed again against the complete new
  core lane and isolated Symbolica prerequisite; its updated lock is retained.
- Isolated Python binding locked all-target strict Clippy with `python_stubgen`:
  passed against the final prerequisite and current core (5.64 seconds).
- Isolated Python binding locked `python_stubgen` feature check: passed again
  against the complete new core lane and prerequisite.
- Thirteen Python controls were added for inert options, invalid selections,
  defaults, rich metadata, and synchronous versus retained native results.
  Execution awaits a rebuilt host wheel containing the new core lane.

The complete portable numerical-dual target passes all 12 scientific controls.
The folded-exact-chart metadata regression also passes the integrated CLI gate:
the independent review found that filtering only numerical kernel associations
omitted those charts. The final native workspace gate passes 515 tests and the
full portable host gate passes 71 tests.

No triple-box generation, parameterization or integration was run by this slice.
The unrelated original untracked triple-box example remains untouched.

## Independent source review

The native API/reuse agent independently accepted the option boundary: native
enum reuse; unchanged symbolic/Taylor defaults; validation before generation
callbacks and parameterization; retained requests; explicit historical absence;
requested mode separated from actual per-chart mode; and cold association by
stable source IDs. No silent execution-mode inference was found. This source
acceptance is separate from the pending rebuilt-wheel runtime checks. The native
and portable core runtime gates have passed, as recorded above.


## Corrected direct artifact construction path

Coordinating review found that `GeneratedIntegral::to_kernel_bytes_with_settings`
still compiled formal placeholders through the old alias builder and selected
real/complex layout from recipe roots alone. It is a fourth construction path,
separate from synchronous compilation, retained compilation and dispatched work.
The path now calls the same `program::build_sector` and `sector_is_real` owners
as the other three routes. Endpoint/cancellation profiles and exact IR codec
remain unchanged; this requires no wire-version change.

The new `direct_generated_bytes_lower_deferred_real_and_complex_vectors_without_jit`
control passes natively and in the portable consumer. Four cases cover both
Taylor/IBP and real/complex input, placing the imaginary constant inside the
original source polynomial rather than the formal recipe. Each first builds
bytes directly without a KernelSet or JIT, then decodes with eager policy,
compares its bytes and component layout to ordinary eager compilation, and
checks the complete Laurent vector against an analytic polynomial integral.
Native and portable runtimes were each 0.03 seconds. Evidence:
`output/numerical-dual-review/direct-artifact-native.log` and
`output/numerical-dual-review/direct-artifact-portable.log`.

## Independent review of the final ggHH benchmark evidence

The reviewer inspected the saved configuration, four artifact pairs, benchmark
implementation and `output/numerical-dual-study/bench/report.json` without
changing numerical or benchmark code. All four run cards are identical after
removing only their requested mode and subtraction choices. Their metadata
records eight generation workers, Horner 10, CPE 1000, native optimizer cores 1,
and 30 retained six-dimensional charts/kernels. Every recorded source-chart
mode matches its request. The evaluator has 27 inputs: six integration
coordinates and 21 explicitly bound runtime inputs (15 Gram quantities and six
model leaves). The complete output layout is real/imaginary coefficients at
orders −1 and 0. All evaluators use native SymJIT O2.

The comparison validates equal native source IDs, geometry, coordinate images
and Jacobians. It maps each chart point into its representative coordinates,
divides a merged representative value by its source-chart multiplicity, then
sums the complete Laurent vector with global exact offsets included once.
This matches the native assembly's multiplication by that same multiplicity.
The result is a comparison of chart-normalized full-vector sums, not a reported
maximum error for every individual chart. The benchmark's separate symmetric
fixture covers the case where the symbolic lane merges two charts while the
numerical lane retains both.

Both subtraction pairs pass 19 chart-point assignments: 16 seeded interior
assignments and three uniform coordinates 0.01, 0.2 and 0.8. Recomputing the
reported differences from the saved vectors gives a maximum absolute difference
of `1.4551915228366852e-11` and a maximum normalized relative difference of
`3.126308197951741e-15`. The relative denominator is
`max(abs(reference), abs(candidate), 1e-10)`. All four Laurent components enter
the comparison; it does not project onto a finite coefficient or discard
cancellations. This finite set of pointwise controls is not an independent
physical integration benchmark, and the saved report correctly has no paired
QMC estimate for these pointwise-equivalent pairs.

Independent byte comparisons establish that Taylor and IBP produce identical
`.dat` files within each mode for this specific diagram: symbolic files are
2,477,048 bytes (SHA-256
`bf067a344201507cc459efe2bea7a90cfed89105d7c38154f44f7aa29d5ad613`),
and numerical-dual files are 17,438,972 bytes (SHA-256
`162684dd70bd43cc83c08f31b4d03b8d34477d2638031f52fb9907763677bd3f`).
The human metadata still preserves the distinct requested strategies. This
fixture therefore does not establish a general performance advantage of one
subtraction strategy; the separate analytic controls exercise their behavior.

The timing boundary in `kernel/evaluator/batch.rs` surrounds only the native
matrix evaluator call. Point generation, runtime input preparation, complex
conversion, output conversion, weighting, stability routing, evaluator-context
construction and loading are outside that measured interval. Four warmup
batches precede 64 measured batches of 256 fixed interior points per sector.
The benchmark verifies primary f64 classification and rejects escalation or
replay; its modified stability thresholds are explicitly benchmark-only.
Recomputing the denominator confirms 491,520 evaluated points and 1,920 matrix
invocations per case, with zero DoubleFloat, arbitrary-precision or conditioning
calls. The reported means are aggregate native nanoseconds divided by actual
rows, so they are amortized matrix costs per point, not scalar-call latency or
end-to-end integration throughput.

Observed batch costs are 1.954/1.950 microseconds per point for symbolic
Taylor/IBP and 23.124/23.284 microseconds for numerical-dual Taylor/IBP. The dual
lane is approximately 11.84/11.94 times slower in these warm matrix measurements.
It also stores larger exact evaluator programs: 11.48 MB versus 1.59 MB across
the 30 sectors. End-to-end generation, including evaluator construction and
compilation, took 115.25/118.58 seconds symbolically and 23.34/32.28 seconds in
the dual lane for these runs. These observations show a generation/runtime
tradeoff. Identical kernels within each mode and varying phase times require
keeping cache and scheduling effects in mind; the measurements are not a
cache-controlled timing distribution or a universal speedup claim. The original
symbolic default remains justified by the measured evaluation cost.

The final isolated portable consumer gate passes 71 tests, with zero failures
or ignored tests, including all 12 numerical-dual controls and the direct
artifact regression. Final locked binding Clippy with all targets,
`python_stubgen` and warnings denied passes (11.65 seconds). Evidence is in
`output/numerical-dual-review/portable-all.log` and
`output/numerical-dual-review/bindings-clippy-final.log`. Executing the new Python
option controls against an installed rebuilt wheel remains pending; the current
CLI generation milestone does not claim that runtime validation.

## Independent scalar-report and final write-up audit

The reviewer also checked `output/numerical-dual-study/bench/scalar-report.json`
and the final [benchmark write-up](numerical-dual-benchmark.md). Each of the four
cases records 4,096 direct `evaluate_weighted` calls per sector, 122,880 overall,
with both matrix invocation and matrix point counts exactly zero. The native
f64 nanoseconds divided by those calls reproduce 3,138.3188 ns symbolic versus
33,425.6200 ns dual for Taylor and 3,210.3759 ns versus 32,514.2622 ns for IBP.
No measured call used DoubleFloat, arbitrary precision or conditioning. The
scalar report repeats the successful complete Laurent-vector comparison.

These are genuine scalar evaluator timing spans; the helper does not substitute
a one-row matrix call. Each sector reuses one seeded interior point for its
4,096 measurements, just as the matrix protocol repeats its 256-point block.
This is a warm microbenchmark and does not claim that those call counts are
independent random samples or an integration-throughput measurement. The
reported 10.65/10.13-fold scalar slowdowns and the separate batch ratios follow
from the saved counters. The write-up preserves that scope and distinguishes
generation savings from evaluation cost. Publication and public-source pinning
were pending during this audit; the same reviewed revision has since been
published through Symbolica PR #54 and selected in the consumer manifests.

Cold native inspection independently confirms the explanation for identical
Taylor/IBP payloads on this diagram: every retained pre-subtraction monomial
power is one of `0`, `1+eps`, `eps` or `-eps`. Their constant powers at epsilon
zero are zero or one, so no higher endpoint power requires IBP lowering. This
check does not generalize that strategy equivalence to other diagrams; the
separate analytic tests cover nontrivial higher-power subtraction.

## Post-rebase branch safety and public-source validation

The milestone was rebased onto the incoming runtime complex-intermediate fix
at `2092eaa`. Its version-8 artifact contract and legacy real-program branch
guards remain intact. An actual deferred sector now conservatively selects
complex evaluation and does not assert that its output coefficients are real:
the compact recipe alone cannot prove that its original source factors stay
real for every runtime point. Wholly symbolic fallback and exact-only inputs
continue to use the upstream native proof. Existing comparisons now normalize
the physical Laurent layout to real/imaginary pairs, checking the added zero
components rather than relying on an optional real-only representation.

The new independent regression keeps `sqrt(p)` inside the original polynomial
factor `(1+x)^2*sqrt(p)`, with `p` declared real and supplied as a runtime input.
The regulated integral of `x^(-3+eps)` times that factor is analytically
`sqrt(p) * (1/eps - 5/2 - 9*eps/4 + O(eps^2))`. Binding `p=4` therefore gives
the complete vector `[2,0,-5,0,-4.5,0]`; binding `p=-4` gives
`[0,2,0,-5,0,-4.5]`. The fixture explicitly requires nonempty, genuinely
numerical-dual sectors, so a symbolic fallback cannot accidentally satisfy it.

This control passes for Taylor and endpoint integration by parts, eager and
automatic native backends, synchronous/retained/reversed-dispatch compilation,
and both directly compiled and cold-decoded templates. Rebinding preserves the
unbound artifact bytes. Scalar evaluation and seven-row importance-weighted
batches agree with the analytic vector and each other. When the actual backend
is SymJIT, counters verify one native matrix invocation for seven rows; portable
eager evaluation records no matrix invocation. Both Distance and explicitly
Validated stability policies retain legitimate zero real or imaginary
components, without relaxing numerical failure checks. The existing direct
generated-to-bytes control separately covers the fourth construction path for
real and complex deferred source coefficients under both subtraction methods.

After the rebase, the shared native numerical-dual target passes all 13
controls with no failures or ignored tests. The full isolated portable suite
passes 72 tests across 12 targets, also with no failures or ignored tests.
Locked binding Clippy passes with all targets, `python_stubgen`, and warnings
denied (43.39 seconds). These checks use the public Symbolica fork revision
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`, registry Numerica/Graphica, and Apple
Clang, with `FASTSECDEC_SYMBOLICA_SOURCE_ROOT` unset. Logs are
`output/numerical-dual-review/postmerge-native-numerical-dual.log`,
`output/numerical-dual-review/postmerge-portable-tests.log`, and
`output/numerical-dual-review/postmerge-bindings-clippy.log`.

The full native workspace gate is coordinated separately; this review does not
infer its result from the focused target. Executing the new Python option
controls and the numerical-dual notebook/browser workflow against a rebuilt
installed wheel remains pending. No Python wheel or browser runtime validation
is claimed by the binding compilation gate.
