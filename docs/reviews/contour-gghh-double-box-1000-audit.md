# Independent 1000 GeV D05 input and comparison audit

2026-10-10. This audit follows the user's new physical-double-box priority.
It records source findings, the comparison protocol and a bounded unsuccessful
generation attempt; it does not yet report 1000 GeV integration results. The root-owned plan addendum
preserves the exact request and supersedes further variance-only LTD campaigns.
Existing passing scientific controls remain evidence.

## Input and saved-owner boundary

The intended input is the existing D05 s-channel diagram, with incoming `++`
gluons, `sqrt(s)=1000 GeV`, `cos(theta)=4/5`, `mH=125 GeV`, and
`mt=ymt=172.5 GeV`. Its normalization remains the unnormalized `delta_ab`
contraction, `D=4-2*eps`, normalized Minkowski loop measure
`prod_l d^D k_l/(i*pi^(D/2))` and measure multiplier one. No colour/spin average,
diagram sum, extra gamma factor or one-loop amplitude conversion belongs here.
The final total is the sum of all D05 sectors and exact offsets, not the full
gauge-invariant two-loop amplitude.

Source inspection confirms that the maintained D05 card keeps thirteen Gram
and helicity products and six used model inputs as runtime parameters. Only
the incoming self-products are fixed to exact zero before sector discovery.
Changing 300/400 GeV to 1000 GeV therefore changes runtime binding, not the
generation template. The exporter reuses native
`Point::with_sqrt_s`, model bindings and topology admission. The final cards
pass independent source, schema and relative-path review, with the native
conservation/on-shell/wavefunction checks executed by the maintained example.
No accepted 400 GeV D05 contour
artifact is available to reuse; the earlier 400 GeV milestone was input-only.

Once generated, strength/cap and physical-point trials should reuse native
saved programs. They must not rerun sector generation or Horner/CPE merely to
change a binding. A new binding invalidates causal-pilot readiness and exact
offset values, which must be established on the actual rebound owner. Artifact
identity, selected recipe, point-card hash and bound settings must accompany
each result. Existing graph/model/numerator assets remain shared.

The maintained exporter passes four tests, including byte-identical reproduction
of the unchanged 400 GeV fixture and all four new 1000 GeV cards; its focused
strict Clippy gate passes. Independent parsing confirms that both generation
cards are equal and all three relative asset paths resolve to the original
D05 assets. The 1000 GeV point retains 19 finite TOML floats, including
`p0p1=500000`, `p2p2=15625`, and the actual native polarization product
`eps1eps2=-0.9999999999999998`. Exact outgoing spatial components are
`(75 sqrt(15),0,100 sqrt(15))` and their negatives, with all energies 500 GeV.
The report correctly marks the old 400 GeV momentum comparison inapplicable.
Point-card SHA256 is
`4fddfe9a607394452765a4d827673951e803ddd45291f0de2a3dcd342578e69f`;
validation-report SHA256 is
`4e5244e9c1fdc784a74ba7a65b36e20cac46f91d5f23126480d4fdd7d6629d50`.
These are input-admission results, not contour or integration acceptance.

## Four-run comparison protocol

The requested final matrix is fixed and dynamic deformation, each with QMC
and discrete Havana MC. A bounded, separately recorded tuning pilot may choose
the dynamic construction and settings. Freeze the chosen construction, fixed
lambda, S/L/R, sampler settings and seeds before the final runs. Record failed
pilot choices and neutral/worse outcomes; finite pilot checks do not establish
global contour certification. Final production data cannot be reused to tune
the choice whose uncertainty is then reported.

Use the existing ordinary CLI caller-owned pool with 50 evaluator worker
threads and the coordinator. This is one native process, not fifty independent
CLI replicas. Discrete MC rejects serial mode, so an ordinary comparison avoids
presenting unlike worker architectures as equivalent. Bind the numerical worker
pool to the intended CPU allowance if the final protocol requires affinity;
record actual threads/processes and aggregate owned-process RSS. Do not enable
nested worker pools or change shared installations.

Each integration budget uses a conservative 300-second native integration
elapsed-time window, including worker-context preparation, Havana adaptation
and subsequent frozen-grid production. Artifact restoration, primary backend
preparation and causal-pilot setup are timed separately. The first integration
JSON carries the existing elapsed clock, which starts before pool creation;
its receive time minus that elapsed value estimates the external deadline
origin, with status/IPC uncertainty recorded. A first active-worker JSON can
arrive after earlier workers already sample while later contexts are prepared,
so it must not be described as an exact first-sample timestamp. Native
`production_seconds` is an adaptive allocation input, not a wall-clock stop.
A caller-owned deadline/status monitor may request coordinator cancellation;
record the observed bracket, deadline, signal, final time and finishing overshoot.
Count accepted complete native work separately from interrupted or discarded
work. An unfinished replica/batch must not become a zero contribution or an
invented complete integral.

The final result should include the complete Laurent mean and covariance,
highlighting the last requested complex coefficient. Report its real and
imaginary standard errors, real-imaginary covariance, and native joint standard
error `sqrt(trace(C))`. A variance ratio at equal elapsed budget is a comparison
of the achieved estimator uncertainties, not intrinsic pointwise integrand
variance when allocations or proposals differ. With one final run per arm,
“best found” means the best setting observed in the declared tuning campaign,
not a demonstrated global optimum or a high-precision variance-ratio estimate.
There is no independent 1000 GeV D05 reference; fixed/dynamic agreement within
their full uncertainties is useful consistency evidence, not an external
accuracy certification. No large-variance-only result warrants extending this
campaign beyond the user's stopping instruction.

## Existing seed and statistical owners

Native `QmcSession` assigns packages independently of worker completion order.
Democratic QMC intentionally shares shifts across sectors and keeps their
complete-sum covariance; adaptive QMC separates pilot and production streams.
The native discrete session reserves a nonoverlapping xoshiro jump per batch,
retains that frontier across adaptation/production epochs, and admits returns
by original batch identity. Workers must not add their own seed offsets or
restart these streams. Causal-pilot sampling uses its separate named protocol
and does not consume production work.

Independent source review confirms that `double_points=false` uses native
`extend_production_batches` after complete discrete-MC production. This retains
the frozen proposal, previously accepted batch means and the stream frontier;
it does not repeat Havana training. Ordinary QMC similarly appends independent
shifts when its lattice point count is unchanged. This is a suitable existing
mechanism for continuing complete work until the caller's deadline. The normal
CLI causal pilot samples the cube centre and then native uniform points per
chart; its retained request/face obligations are checked, but these coordinates
are not an explicitly enumerated set of boundary points.

Use distinct declared tuning and final seeds. If final fixed/dynamic runs share
a base seed, record that as pairing, not independence; different learned Havana
proposals generally produce different coordinates even from related native
streams. Reuse native full-vector estimators and covariance rather than a new
Python accumulator. Preserve the native distinction between complete production
statistics, live partial observations and the prior complete refinement epoch.

## Measurable optimization leads

Source inspection identifies these bounded candidates, not established speedups:

- Runtime values and contour strengths already support rebinding. Generate
  capabilities once and reuse them across tuning and final sampling.
- Discrete MC launches at most one task per available batch. The example's
  32 batches cannot occupy all 50 workers; use a predeclared sufficient batch
  count and measured batch size, preserving native stream reservations.
- Discrete MC lazily retains visited sector contexts per worker, potentially
  all sectors times 50. QMC retains one active sector context per worker.
  Measure this residency before prewarming or multiplying large evaluators.
- Ordinary adaptive QMC currently passes all-one output weights to native
  allocation, even for an epsilon-zero accuracy target. Discrete MC trains on
  the maximum absolute component across the full vector. If actual D05 pilot
  magnitudes show that poles dominate the requested finite coefficient,
  investigate the existing native allocation/training interfaces. Keep all
  coefficient outputs and covariance; do not discard poles to improve a metric.
- Optional contour observations can remain Disabled during final timing, after
  a separate observed pilot, while ordinary native precision/failure counters
  remain available. Optional expensive artifact semantic revalidation stays
  separate from mandatory structural/backend admission.

Relevant source boundaries are CLI `driver/execution/{qmc,discrete_mc}.rs`,
their worker modules, `driver/refinement.rs`, `contour_pilot.rs`; native
`integration/{qmc,streams}.rs`, `mc_discrete/{session,worker/batch}.rs`, and
`integration/accuracy.rs`. No new sampler, covariance implementation, numerical
worker pool or replacement algebra is proposed by this audit.

### Native JIT callback-clone limitation

A focused native runtime-agent probe subsequently established an owner-level
obstacle to independent worker scratch. Its callback has a custom `Clone` that
creates a new mutex and workspace identity. Eager evaluator clones invoke two
distinct identities, while a JIT evaluator and its clone invoke the same one;
all four numerical values are correct. Independent source review agrees with
the executable evidence in
`target/contour-d05-1000-runtime/clone-callback{.rs,-result.log}`.

Symbolica's derived JIT clone copies the external-function containers, but the
active SymJIT `Applet` retains shared `Config.df` and machine-code ownership.
The generated machine code contains the original callback environment address,
so the newly cloned callback is not used for dispatch. For callbacks with
mutex-protected scratch, this can serialize worker clones even though the
mathematical callback is pure. Actual D05 throughput impact is not yet measured.

No public cheap callback-rebinding operation was found in the pinned owners.
Recompilation can establish a distinct environment, but rebuilding every worker's
large JIT would impose a different setup cost. An efficient remedy belongs in
the native owner and must retain correct callback lifetimes, nested evaluator
dispatch and independent scratch without an unbounded FastSecDec thread cache.
The existing `Defuns` owning raw pointers must not be shallow-cloned as a
workaround. This finding does not establish a wrong numerical result and is
separate from the physical chart's generation-memory problem.

The owner correction is independently reviewed in
[the callback-clone audit](contour-jit-clone-callback-audit.md). Its draft keeps
machine code shared and dispatches through the calling evaluator's borrowed
callback table. A registry identity prevents nested unrelated evaluators from
confusing table indices. A bounded stack-local frame and RAII restore the
thread-local pointer; this is not a retained per-thread callback cache. The
standalone public conversion API keeps its prior scope-free behavior. Native
threaded batches enter the same scope in existing Rayon chunks, preserving
the native packed SIMD layout and scalar tail handling.

An independent include of the actual private scope module passed nested
`A -> B -> A`, ancestor dispatch, same/foreign-registry Rust unwind, stale-scope
rejection and four-thread table-isolation controls. Evidence is ignored under
`target/foundation-jit-scope-audit/`. This validates the scope mechanism only;
actual JIT, named-body, batch and saved-owner controls remain the owner agent's
publication gate. No FastSecDec dependency revision has been changed by this
review.

## First bounded generation and six-dimensional representation review

The first all-four-recipe generation used the verified private release
`3303005` CLI, eight caller-owned workers, NumericalDual/IBP and the unchanged
runtime-parameterized D05 template. Its 900-second / eight-GiB monitor stopped
the owned process group for memory after 70.846 seconds. The sampled aggregate
peak was 9,338,118,144 bytes; this exceeds the trigger because sampling and
process termination are not instantaneous. All owned processes closed, with no
remaining members. This is a resource failure, not a contour-integrand failure.

The journal contains thirty completed shared-source receipts and thirty
completed undeformed units. None of the thirty fixed-map requests has a
completed response receipt. The archive was not published, and those staged
units do not constitute a usable completed contour family. Raw status, receipts
and monitor evidence remain ignored under
`target/contour-gghh-double-box-1000-generation/`.

The existing structured native determinant proof applies in six dimensions as
well as higher dimensions: fixed strength uses `A=I-i lambda Dv`; dynamic
strength uses the bordered matrix retaining the full native gradient of
lambda. Only nonzero principal minors of `A` appear as pivot denominators on
the admitted real cube, including its faces. The full Jacobian may still vanish.
This supports investigating the existing helper at dimension six, but does not
by itself demonstrate lower resource use for physical D05 entries.

The proposed bounded source-zero comparison must use the same prepared source
receipt and native symbols, retain images, endpoint ratios, F/U and face
associations, and compare the determinant against a native high-precision
numerical determinant of the full image derivatives. A subsequent actual
subtraction/compilation/save/restore control must preserve the complete Laurent
vector and native coefficient definitions. Retained Atom, native IR and saved
record sizes matter alongside mapping time: moving expression growth into
lowering or metadata would not resolve the generation bottleneck. No lower-
dimensional threshold change is accepted by this audit before those measurements.

The bounded source-zero determinant comparison subsequently passed, with both
process groups closed and no resource limit reached. Both paths used the same
verified native source receipt, six coordinates, actual 1000 GeV bindings and
fixed strengths `1e-6` and `1e-5`. Each matched the independent 192-bit native
full-Jacobian determinant at two interior and two face points. Maximum normalized
component errors were below `2e-57`.

| Source-zero determinant representation | Retained Atom bytes | Native IR instructions | Process wall seconds | Sampled peak RSS bytes |
| --- | ---: | ---: | ---: | ---: |
| Existing polynomial template | 1,363,406 | 700 | 0.988 | 68,624,384 |
| Structured native `A` | 836,891 | 471 | 0.760 | 50,806,784 |

These are single-process feasibility observations, not controlled throughput
benchmarks. The source contains 236 terms and 10,218,756 bytes of input Atoms,
whereas its F is only 2,056 bytes and all 36 image derivatives total 21,355 bytes.
Thus the determinant-only result does not explain the failed full map campaign.
The current `deform_with` constructs a complete smooth density per term before
combining terms, repeating the determinant and common residual substitutions.
Whole-discovery stage and retained-size measurements are required before
attributing or fixing that amplification. The determinant probe deliberately
does not claim subtraction, full saved-owner or complete-generation acceptance.

The subsequent shared-fixed-J milestone now passes the missing scoped
subtraction/compiled-owner check. Independent review verified every raw report
and saved-file hash in `fixed-parity-result.json`, then recomputed the eight
scalar components across two complete complex Laurent vectors at `lambda=1e-6`.
The maximum absolute discrepancy is `1.4210854715202004e-14`, and the maximum
discrepancy divided by one plus the original component magnitude is
`8.089763278189307e-16`. Both representations retain exactly the same native
causal refusal at `lambda=1e-5`. A separate fresh process verifies restored
owner readiness; the original failed admission attempt remains recorded.
All three process groups closed. The saved shared program is 6.64% larger,
despite smaller intermediates and observed compilation work. The
[generation owner's report](contour-fixed-shared-jacobian.md) gives the full
measurements and limits. This accepts the fixed sharing implementation and
six-dimensional structured extension for the demonstrated scope; complete
physical generation and integration remain pending.

## Native dual-composed Jacobian option

The user's subsequent instruction raises the campaign RSS cap to decimal
100 GB and asks for a selectable dual-evaluated Jacobian investigation. The
earlier eight-GiB failure remains evidence, not the current campaign limit.

Public API and source review identifies an existing native route:

1. Compile the original complex coordinate images to a native exact evaluator.
2. Use `Dualizer<HyperDual<Complex<Rational>>>` to produce their first coordinate
   derivatives as scalar instructions. Runtime parameters have zero derivative
   seeds; each coordinate retains its own unit seed even on a face.
3. Use `EvaluatorComposer` to bind those slots to a reusable native
   `Matrix::det()` determinant program. Physical Atoms are never substituted
   into the determinant expression. Do not use the known defective pinned
   `det_in_place()` path.
4. Compose the resulting scalar into the full smooth density once, before
   applying the existing outer native subtraction jets. Native differentiation
   then obtains the additional image/strength derivative order automatically.
   Multiplying a scalar numerical determinant after subtraction would not be
   equivalent.

For the physical six-dimensional task a native generic polynomial determinant
program is finite and reusable. This recommendation does not extend arbitrary
uncancelled full-Jacobian pivot quotients to higher dimensions. The existing
structured `A`/border proof remains relevant there. Native function bodies must
be inlined before composition, as required by the public composer; its existing
constant/callback ownership and final instruction optimization remain in use.

A bounded executable probe uses fixed strength and the actual two-coefficient
implicit-radius callback. Each image vector is dualized and composed as above,
then included in a smooth density containing a regulator-dependent complex
power. The outer native jets retain all mixed x/y derivatives through total
coordinate order three and regulator order one. At nine interior/face/vertex
points, forty outputs per point agree with the direct native symbolic full-J
reference at 192-bit precision: maximum absolute errors are `2.55e-57` fixed
and `5.10e-57` implicit. The composed program also passes an existing native
`ExpressionEvaluator` encode/decode round trip before the outer jets.

The probe process closed successfully in 4.25 seconds with sampled peak RSS
96,915,456 bytes under a 60-second/one-GiB bound. Source SHA256 is
`59ec70525b0910e9fea57ef84dc553adfed585bd4e3e52c7811cdc5ae67ec8ba`;
raw source, build command, output and resource record are ignored under
`target/foundation-d05-dual-jacobian/`. This is a small native API/derivative
control, not physical generation or throughput evidence. Its codec control is
within one process and does not establish a fresh-process complete artifact.

No new determinant callback, alternate AD engine or owner feature is required
by this demonstrated route. A production option still needs an honest metadata
representation that does not eagerly construct the physical symbolic
determinant merely for inspection, generation/staging identity, genuine physical
subtraction parity, selected save/restore, checked face associations and measured
generation/sampling tradeoffs. Existing factorwise branch programs must enter
the same composed density unchanged. Portable and precision execution should
reuse the saved scalar native IR, rather than rebuilding a Jacobian on load.

A second focused native control establishes the proposed connection to an
existing smooth-body evaluator: the entire unresolved function Atom
`det(x,y)` can be an explicit evaluator input. Compiling `(x+y)*det(x,y)` with
inputs `[x,y,det(x,y)]` and evaluating `[2,3,7]` returns exactly `35`, without a
function body or derivative hook. This permits the composer to supply the
Jacobian output at that input before outer differentiation. The control is
recorded as `function-parameter.rs` and `function-parameter.log` beside the
main ignored probe.

The implementation review requires the body, retained Jacobian call and image
program to use the same face-specific request lowering. Coordinate derivative
seeds remain unit seeds at endpoints; only their value slots become zero or one.
The native source-program cache must distinguish retained Jacobian plans as
well as the expression, ordered inputs, definitions and compilation settings.
Exact contributions must materialize their genuine determinant expression at
the existing exact-output boundary before aggregate cancellation and root
association selection. These are acceptance conditions for the new option,
not claims that its production integration has already passed.

The subsequent source review confirms those composition and cache boundaries.
Inner image jets are seeded in the native order `[value, dx0, ..., dxN]`, their
row-major derivative outputs feed the determinant program, and the resulting
scalar enters the whole smooth body before its outer jets. Staged chart plans
reuse the metadata's native Atom indices. Reading requires one matching plan
for a nonzero-dimensional dual contour, with the original chart index and
native-equal images/J; instantiation also checks ordered coordinate symbols
and the requested mode. A chart requiring symbolic stochastic fallback is
explicitly rejected under this initial dual option. Zero-dimensional and
undeformed charts require no plan.

Public settings use the same native enum in Rust, CLI and the thin Python
bindings. Omitted symbolic compiler policy preserves its historical encoding;
the retained generated choice controls compilation, and requesting dual only
at compilation cannot convert a previously symbolic generation. Native source
and API review passes at this boundary. The mixed-face cache and real generated
higher-jet/checked-owner controls pass 2/2 in 4.23 seconds. The executable was
independently checked against the current compiler working directory, source
hashes and its 428-test inventory after a narrow core-package clean; an earlier
zero-match run used an old executable and supplies no Dual acceptance. The staged filter
passes 4/4, including the actual fresh-worker restoration control, and the public
policy owner tests pass 3/3. These are distinct focused gates; portable and
installed Python execution remain separate. Physical D05 resource
and sampling benefits must be measured independently of the small API proof. In particular, distinct smooth bodies currently rebuild their common
image/determinant program inside separate source-cache entries; sharing that
native work is a candidate optimization, not an established performance result.

## Stochastic kernel-cost harness review

The ignored cost driver reuses native indexed selection, the maintained CLI
Pilot16 implementation, `QmcSession` and weighted complete-vector batches.
All arms use the same native rule, seed and allocation; their actual transformed
coordinate and weight bits are hashed. Task content IDs fence ownership without
changing those QMC streams. Each arm has an independent numerical context, and
sector-wise rotation avoids always measuring the same arm first.

The timed batch wrapper includes native precision rescue and replay. Artifact
loading, binding, causal pilots, context construction and coordinate hashing
are recorded separately. Production causal-check counts must remain zero under
the completed Pilot policy. This measures the full stochastic Laurent vector
with weights applied once; aggregate exact offsets are intentionally outside
this cost table and require separate admission for a full integral run. The
reported maximum is a maximum of sector means, not a maximum callback latency.

Review found and corrected a reporting gap: native process exit zero did not
itself exclude incomplete arms or a limited source prefix. The runner now
requires exact planned arm/sector/row coverage and finite accepted costs, writes
separate scope acceptance, and returns nonzero on incomplete scientific scope.
A source-zero feasibility run cannot be accepted as an all-sector table. Raw
failures and completed partial work remain available without a full-cost claim.

Source review passes; no concrete plan or cost execution is accepted by this
review. A run still needs final release/source/pilot-module and archive hashes,
the frozen physical point, and the actual enclosing resource monitor. The
one-thread cost measurements remain separate from the requested five-minute,
50-worker QMC and discrete-MC comparisons.

The source-zero timing feasibility plan later selects the existing native
`RuleSource::Supplied` with modulus 32, vector `[1,3,5,7,11,13]` and two shifts.
The runner fixes and records that small allocation and still compares actual
coordinate/weight hashes. This avoids pretending that the published Kuo table
supports 32 points; it is a timing-only control, with no QMC quality or integral
claim and no change to the final production rule.

Independent source review confirms one narrow generation-cost candidate in the
initial Dual implementation. `SourcePrograms` caches each distinct regular body;
its Jacobian builder recompiles the common image vector, first native Dualizer
and determinant program for each such body. The existing native composer can
append a borrowed exact program, so a caller-owned prefix cache can reuse that
construction. Its key must retain the complete face-lowered Jacobian plan,
ordered inputs, native definitions and compilation settings. A shared immutable
`OnceLock<Result<Arc<ExactProgram>, ...>>` fits the existing cache lifetime and
error behavior; builds must remain outside index locks. Body composition and the
outer full-density jets remain unchanged. This would avoid repeated generation
work, not eliminate each body's stored instructions or establish a runtime gain.
The actual source-zero measurement must finish before judging the benefit.

The cost harness also admits a raw saved `KernelSet` for source-zero feasibility
only. It checks the saved receipt digest before native decoding, then requires
one retained source-zero chart/sector, the recorded unbound identity and native
equality of source geometry, F/U, positive measure and output layout across
arms. Its real Pilot16 runs on that same owner, including any exact obligations.
The bound exact vector is recorded as provenance without entering the stochastic
timer, and this path always reports `all_sectors=false`.

The first completed physical fixed-Dual source-zero attempt preserves the native
shared all-recipe preparation exactly, then selects one actual source chart.
An earlier singleton-preparation attempt stopped at the strict source-metadata
comparison; its original evidence remains separate. The corrected attempt
completes in 473.363 seconds, with 405.637 seconds in compilation, sampled peak
RSS 3,966,705,664 bytes, and a 1,143,374-byte saved owner. A separate fresh process
restores that owner in a 0.565-second total run with 38,313,984-byte sampled peak.
Both process groups close without a resource limit.

Fresh restoration reproduces the complete admission/refusal records and all
eight scalar values at the two prescribed points. The admitted strength is
`1e-6`; the `1e-5` homotopy refusal remains an identical native causal failure.
Against the earlier shared-Symbolic saved reference, maximum absolute/scaled
component differences are `6.75e-14`/`4.08e-15`. A current-owner Symbolic counterpart
is being measured separately, so these historical comparisons do not establish
a matched generation-speed result. No integral has been sampled and no Dual
performance advantage is accepted from this baseline. The immutable raw review
is retained under `target/foundation-dual-build-provenance/`.

The subsequently completed same-owner Symbolic counterpart also passes fresh
restoration and the cross-policy pointwise comparison: eight scalar components
have the same `6.75e-14` absolute / `4.08e-15` scaled maximum differences, with the
same larger-strength refusal. Its generation run takes 103.498 seconds, including
54.567 seconds of compilation, and peaks at 2,156,773,376 bytes. These are matched
source/settings feasibility observations with concurrent host activity, not an
idle-host benchmark. The initial uncached Dual path is slower and uses more
memory on this control.

The narrow prefix-cache patch has passed independent source review. It retains
one immutable native Jacobian program per complete key within the existing
caller-owned cache, then composes each smooth body and its outer jets normally.
The focused regression now exercises two distinct bodies sharing a prefix,
changed images producing twice the determinant, and recovery after a failed
body and a separately invalid plan. Source ownership, key boundaries and error
isolation are accepted. The revised mixed-face cache and generated checked-owner
controls pass 2/2 in 4.05 seconds, followed by the staged filter 4/4 in 0.33
seconds. Physical performance outcomes for this cache revision remain pending.

The complete fixed singleton artifact now passes an independent publication
review. The immutable manifest and 79,036,643-byte data file match their recorded
SHA256 hashes. The native catalogue contains exactly 30 stochastic and 30 exact
records, with stochastic source indices 0 through 29, six coordinates, and the
complete `[-1 Real, -1 Imag, 0 Real, 0 Imag]` layout. Its 19 physical parameters,
fixed-strength parameter, SymJIT O2 evaluator statistics, threshold policy and
unit measure multiplier agree with the planned input. The owned generation
process group closes successfully after 141.935 seconds, with a sampled aggregate
peak of 16,056,111,104 bytes. Concurrent builds and feasibility work were active;
these are generation observations, not an idle-host benchmark. Native per-record
decoding and actual causal admission remain separate runtime checks.

The first source-zero cost launch stopped before admission because the ignored
driver parsed physical symbol names before the saved native StateMap established
their real attributes. The original refusal is preserved. The reviewed setup
correction restores the saved owner first and then calls the same physical-point
parser; it changes neither point values nor numerical evaluation. Wrapper review
also requires immediate group termination on any RSS breach, including during
wall-clock grace, and permits the predeclared smaller-strength fallback only
for a native causal refusal before sampling starts. A sampled failure cannot be
used to select that fallback.

The later actual SymJIT fixed-artifact sweep retains five predeclared strengths.
Independent review of all 92 raw rows confirms that `1e-8`, `1e-7` and `1e-6`
each complete all 30 sectors, with 16 native pilot points and 64 timing samples
per sector. Actual transformed coordinates and weights match across these arms;
the complete stochastic Laurent vector is evaluated. Their mean costs are
158.0, 144.9 and 143.0 microseconds per accepted point, respectively, including
98, 88 and 81 double-float rescues among 1,920 points each. Maximum sector means
are 267.2, 267.9 and 366.7 microseconds. No arbitrary-precision rescue or optional
production causal check is recorded. These tiny matched allocations size work;
they neither measure integral variance nor establish controlled-host throughput.

The `1e-5` and `1e-4` arms are refused at source-zero centre before sampling, with
native certified positive imaginary parts of F. Their costs remain null. The
111.139-second process group closes without a resource limit at a sampled
264,540,160-byte peak, but the wrapper correctly returns 2 and preserves false
whole-sweep acceptance because two arms failed. Only the three individually
complete arms are admitted by this review. Aggregate exact-owner admission and
full-integral statistics remain outside the cost experiment.

The optimized cache-candidate binary also passes an independent provenance
review: all 1,434 frozen inventory entries match, including 675 current crate,
binding and manifest files, and Cargo reports fresh compilation of all four
runtime workspace packages from the private snapshot at optimization level 3.
The private build closes in 634.812 seconds at a 2,079,318,016-byte sampled peak.
The protected user release binary retains its hash, inode, size and timestamp.
This validates the executable's source identity; physical cached-Dual performance
and the final timed integration results remain separate gates.

Six subsequent standard-CLI fixed-selection runs also pass raw-result review.
Each uses 50 ordinary worker threads, the native Kuo rule with 1,024 points and
eight shifts in each of all 30 sectors, and fresh admission on the actual
aggregate owner. All 245,760 points per run are accepted; the complete four-slot
Laurent vector, covariance and exact offsets agree between native stdout and the
versioned saved result. The two selection seeds are disjoint from final-run
seeds. No failures, cutoff zeros or optional production causal checks occur.
The predeclared mean of finite-order covariance trace times aggregate worker
seconds selects fixed strength `1e-6` among the three surviving candidates.
This selects a prescription from a small noisy allocation, with no pooled
covariance, convergence claim or independent physical reference validation.

## Current implementation milestone and reuse verdict

Independent native and HEPKit boundary review accepts this implementation slice.
The option reuses native generated objects, saved programs, recipe-family
ownership, evaluator composition and higher-order Dualizer operations. Exact
offsets still use native materialization and aggregation; the stochastic path
does not introduce another algebra system, derivative engine or sampling loop.
The new cache is caller-owned and immutable after construction. CLI and Python
settings forward the same native choice and reject unsupported mode combinations;
the default remains backward compatible. Current Python source and compile
checks do not by themselves establish installed-host execution of the new option.

The final current-source workspace run passes **917 tests**, with **33 explicit
ignores**, zero failures and zero filtered tests. Package totals are core
664/23, CLI 175/8, QMC 37/0 and sectors 41/2 (passed/ignored); the core total
already includes its 409/19 library controls. Existing reference tests, native
artifact restoration, public Jacobian policy and real CLI process tests are
included. These totals must not be added to earlier focused subsets.

Strict workspace/all-target Clippy passes with warnings denied after replacing
one checked `Option::unwrap` with an equivalent `if let` binding in numerical-dual
chart admission. The optimized candidate2 snapshot retains the preceding
equivalent spelling; there is no changed branch, numerical instruction, setting
or identity. Workspace formatting and diff checks pass. The final focused rerun
after that lint-only edit passes both Dual Jacobian controls in 4.01 seconds;
these repeat two tests already counted in the workspace total. Physical
cached-Dual generation benefit, complete
dynamic-artifact admission and the four final timed estimates remain pending.
