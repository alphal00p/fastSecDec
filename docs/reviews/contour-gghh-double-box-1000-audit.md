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
