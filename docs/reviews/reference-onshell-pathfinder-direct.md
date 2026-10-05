# Original on-shell direct Pathfinder reference

This separate correctness attempt reuses the existing Pathfinder `generate` and
strict `integrate` commands, without pySecDec package generation or FORM. It is
the next provider after the preserved 600-second native pure-Taylor/FORM failure
in [the external-reference record](reference-onshell-full-vector-proposal.md).
The original ten unit-power massless lines and on-shell `s12=s23=-1` card are
unchanged; no projection, numerator change or new algebra is introduced here.

The ignored `output/probes/onshell_pathfinder_reference/run.py` uses the existing
process-group/RSS watchdog and performs only orchestration and JSON admission.
`admission.py` checks the returned native bundle/result. The commands reuse the
[reviewed original generation route](reference-onshell-generation-source.md).
Both phases pass the original run card, full support, no sector selection,
complex SymJIT O2 and highest order zero. The native projector/IBP and derivative
presets remain as shipped. Generation saves a complete strict evaluator bundle;
integration loads that same bundle with ordinary native global-prefactor handling.
No target is supplied or derived from the new Rust candidate.

The independent bounded chain is generation **600 seconds**, numerical
integration **1,200 seconds**, and an enclosing **1,900-second** deadline, each
with the existing 30 GiB aggregate process-tree RSS cap and five-second
interrupt/kill grace periods. One worker runs on CPU 0. Numerical allocation is
one complete democratic correlated QMC iteration, full support, **1,024 points
and eight shifts**, QMCPy linear lattice, Korobov3, seed 20261205 and batches of 1,024 points.
The point cap is disabled; early/partial/interrupted outputs are retained but
not admitted as a complete reference. There is no automatic retry or extension.
Both phases use Symbolica, so the scientific slot stays reserved until the
entire chain is reaped. Native fullgraph capability has scheduling priority.

The existing working reference venv is deliberately used. It has Python
Symbolica 2.1.0 / native Symbolica 2.1.0 with embedded SymJIT 2.18.6; this is a
versioned correctness provider, not the new isolated 3.0.1 binding environment
or a matched performance run. Preparation records actual distribution versions,
the loaded module files, checkout revision/status, source archive, commands and
immutable input hashes. Both existing formula-cache read roots are frozen; new
writes and mirrored hits go to a fresh attempt cache. Available formulas are
allowed inputs, without claiming a cold or fully warm cache.

Bundle admission requires all returned native sector IDs, complete native
Laurent labels through 0, O2 complex/projector settings, original input paths,
three loops, ten unit powers, D=4−2epsilon, native Gamma(4+3epsilon), and its
complete stored prefactor coverage. Strict integration must preserve the bundle
bytes and return every sector and every signed order with finite coefficients
and errors. Actual minimum order/sector count come from the bundle rather than
from earlier failed packages. No leading row is trimmed.

Retain the ordinary `result.json` intact: its per-sector `raw_sector` values,
physical `raw`/`display` arrays, actual sampling/precision diagnostics and native
normalization metadata are evidence. `formatting.apply_global_convention`
convolves the means once and propagates errors with an L1 sum of absolute
prefactor weights. These physical errors are not supplied joint covariance or
newly derived independent standard errors. No custom convolution, covariance or
uncertainty estimator is added. A completed finite full vector is a provider
output; scientific agreement/calibration must still be assessed honestly.

Source/compilation preparation is separate from scientific execution. The root
coordinator schedules the exact fresh campaign after independent source/freeze
review and native runtime release. Syntax checking uses the existing Python
without importing Symbolica. No scientific attempt has run under this protocol.

Independent source and concrete preflight review accepted the fresh campaign
`output/diagnostics/onshell-pathfinder-direct-attempt-1/`: all 572 frozen hashes
and 571 source bindings pass. Both read-cache inventories (2 primary and 498
legacy files) are intact and the new write cache is empty. The reviewed
admission derives prefactor coverage from the returned native lower order.
The independent preflight SHA256 is
`be9121b4559d2d7d5e4f70cd9a9c72ca44db6cd76adb822cd9d60154e033c817`.
The archived run command is the existing venv Python followed by
`<attempt>/sources/run.py --run <absolute-attempt>`; runtime remains queued
behind native fullgraph and the separate small binding smoke.

## Attempt 1 outcome

The original working-environment generation exited 1 after 19.186 seconds;
the enclosing process finished in 20.310 seconds. This was neither a deadline
nor a memory-cap failure. Native endpoint-projector preparation found a missing
cache signature and rejected it with its existing message requesting
`--allow-fallback-for-missing-caches`. The declared read caches were available,
but did not cover every required formula. No complete bundle, integration,
coefficient vector or result was produced. The numerical phase was not started.

All 572 frozen checks passed afterward and both read-cache inventories were
unchanged. Mirrored cache hits remain in the attempt-owned write directory.
Generation process group 100054 and outer group 99945 were reaped and absent;
the scientific slot was released. The failure and all partial files remain in
the fresh attempt directory. No retry was launched automatically.

Source inspection identifies one existing option for a subsequent decision:
`FSD.py` forwards `--allow-fallback-for-missing-caches` to the topology; the
failing `subtraction_formula.py:560–616` branch then uses the existing native
endpoint-projector builder and writes its formula to the configured attempt
cache. It changes missing-cache admission, not physical inputs, normalization
or the integration estimator. No Python API port or custom algebra is needed.
A fresh invocation would retain the declared bounds and all previous evidence;
it is separate from the failed attempt and awaits the coordinator's decision.

## Missing-cache fallback outcome

The separately authorized
`output/diagnostics/onshell-pathfinder-direct-fallback-attempt-1/` added only the
existing `--allow-fallback-for-missing-caches` option. Original inputs, working
environment, full-sector QMC allocation and 600/1,200-second bounds were retained.
The native endpoint-projector builder created cache entries without FORM.
Independent preflight accepted all 572 immutable checks
(`ee444f814230d665cb88526ccc3b8c33f9581e63e1ff2c9029a9c488f533059b`).

Generation reached its 600-second deadline. The watchdog returned 124 after
610.444 seconds, including interrupt/termination grace. Its immediate group
absence assertion raced teardown, so the outer wrapper exited 1 after 611.768
seconds. The missing normal generation-reap record and original traceback remain
intact. A later explicit check in `terminal-slot-release.json` confirms both
process groups were absent; no process was left running. No complete bundle,
numerical stage or reference vector was produced.

The immutable checks and both read-cache inventories still pass. The retained
cache contains 137 completed JSON records totaling 13,226,131 bytes: 41 are
byte-identical mirrors of existing read-cache entries and 96 have new signatures.
An additional 96 zero-byte lock files are not formulas. Independent review
checked JSON syntax, native signature-derived filenames and complete stored
Laurent-vector lengths. Ordinary native symbolic-loader admission remains the
owner of any later reuse; these files are not a completed evaluator bundle.
Maximum sampled generation-tree RSS was 6.194 GiB. Quiet logs do not identify
the final sector or active native call, and the cache count is not a completion
percentage.

The independent terminal review is `independent-terminal-review.json`, SHA256
`efceb0e09a46a7ab0c8038e4c16e3f2c39f790180772e0ead93317289d1891db`.
This is an accepted failure record, not a scientific result. The useful cache
progress and remaining memory headroom motivate preparing a fresh, separately
recorded 1,800-second generation attempt with the same native algorithm and
inputs. Only completed JSON records would seed its isolated write cache; locks,
partial bundles and numerical results would not be copied. That preparation
introduces no deadline extension to the retained failed attempt and no runtime
is launched before the persistence correctness gate releases the slot.

## Seeded-cache continuation preparation

The independent reviewer accepted the narrow source delta in
`output/probes/onshell_pathfinder_cached_reference/`. Pure-data preparation
created `output/diagnostics/onshell-pathfinder-direct-cached-attempt-1/` with
separate immutable seed copies and a fresh mutable write cache containing the
same 137 completed JSON records. The previous terminal review, complete parent
cache inventory and each copied record are bound. No lock file or partial
bundle is reused. Existing read roots remain frozen, and the ordinary native
loader must admit every formula it uses.

Generation has a fresh 1,800-second allowance; numerical integration remains
1,200 seconds and the enclosing deadline is 3,100 seconds. The 30 GiB process-tree
RSS limit, one worker, original science arguments and complete 1,024×8 allocation
are unchanged. This records generation with available formula inputs, not a cold
construction benchmark. A five-second bounded group-absence poll observes
teardown after the watchdog has already enforced its deadline; it neither
restarts work nor extends the scientific bound and still fails if the group
persists. The attempt is prepared only. Execution waits for the combined
persistence gate and explicit scientific-slot handoff.

Concrete preflight independently verified all 851 frozen checks and the complete
137-record seed inventory. Its record is `independent-preflight.json`, SHA256
`f280c4cf555d974a734d340dcb852c5e2268d7372046bdd28ea90100c5c1a773`.

## Intentional pause and resumed cached attempt

Cached attempt 1 was stopped at the user's requested pause. Its wrapper exited
130 after 640.404 seconds; all owned processes and groups were reaped. This was
an intentional cancellation, not a deadline failure. The 137 JSON cache records
remained unchanged, and no bundle or numerical result existed. The original
`intentional-pause.json` and partial logs remain intact.

After the user resumed, cached attempt 2 was prepared in a fresh directory with
identical source, science arguments, 137 seed JSONs and 1,800/1,200/3,100-second
limits. Its independent preflight accepted all 851 frozen checks
(`fc2ed6340eedbcc4dfbb12a24f684fee2b5ef95709e2f9c8297ec06e1c446983`).
It started only after the complete native fullgraph-4 chain was reaped.

Generation reached its unchanged 1,800-second deadline. The watchdog interrupted
its process tree at 1,800.3 seconds; the tree exited after SIGINT. The generation
record reports 1,802.576 seconds and exit 124, and the outer process reports
1,803.632 seconds and exit 124. All five recorded PIDs and all three process
groups were absent afterward. The numerical phase was not started. There is no
bundle, coefficient vector or result, and no retry followed automatically.

All frozen hashes and both read-cache inventories remain unchanged. The mutable
cache exactly equals its initial 137 JSON records; no new lock files or other
cache outputs were created. Maximum sampled generation-tree RSS was 3.069 GiB,
well below the limit. `terminal-slot-release.json`, normal generation/outer
process and reap records, postflight checks and full logs are retained under
`output/diagnostics/onshell-pathfinder-direct-cached-attempt-2/`.

Bounded passive observations did not identify an exact active stage. No cache
file or lock descriptor was held at the snapshot, and no `.locks` directory had
been created. The existing cache lock helper leaves its lock files behind, so
there is no evidence that this attempt entered the cold endpoint/regular formula
cache-build lock. In contrast, the native warm endpoint-cache loader parses each
stored expression and builds evaluators before returning; unchanged JSON files
do not imply that this work is cheap or complete. Topology/sector construction
also precedes runtime formula preparation. Two anonymous executable-memory
snapshots roughly 698 seconds apart both contained two regions totaling 73,728
bytes; that supplies no observed growth evidence and cannot establish an active
JIT call. The precise expression, signature or sector remains unknown.

Independent terminal review rechecked all 851 frozen files, both 500-file read
roots, the unchanged 137 seed/write-cache records and process absence. Its
`independent-terminal-review.json` has SHA256
`3bbd211ff2631f7a186bd531ce53c89abb65983441c5fa357c66dae5dcd53a59`.
Terminal stderr contains only the existing CLI's fixed interruption message;
its `main` catches `KeyboardInterrupt`, so no Python traceback was retained.
The watchdog records establish the deadline cause despite that generic wording.

The subsequent narrow diagnostic was prepared at
`output/probes/onshell_pathfinder_stage/`: a fresh 180-second generation process
with the same math/cache inputs, stdlib periodic Python stack dumps and the
existing `--log-file` stage output. It changes no reference source or algorithm,
and contains no integration follow-on.

The authorized `onshell-pathfinder-stage-attempt-1` process exited 124 in
181.809 seconds at its fixed deadline. Its process group and wrapper were reaped;
all 993 frozen checks and both read-cache inventories passed. The stage log shows
endpoint preparation completed in 6.737 seconds: 223 requested signatures,
137 cache hits and no newly generated formulas. It then entered the explicit
build for 968 sector evaluators. The 30-second stack samples move through native
regular-coefficient differentiation, simultaneous substitution and expression
support inspection inside `_two_stage_derivative_fused_components`. Thus this
attempt establishes expensive explicit expression construction, not a cache-load
stall or a single expression that is proven stuck. No sector completion count or
exact active expression is established. The final stage log says “finished”
because `prepare_explicit_sector_formulas` calls `finish_stage` in `finally`;
that line does not certify successful generation. There is no bundle or vector.

Root explicitly allowed this diagnostic on CPU 8 to overlap the native
continuation on CPUs 0–7. The recorded overlap through the diagnostic wrapper's
final trace was 143.567 seconds. These are diagnostic/capability timings, not
isolated performance measurements. `terminal-slot-release.json` retains the
session, process group, timestamps and absence check.

## Existing alternative reference layouts

Source inspection finds no existing explicit-only flag that avoids the observed
coefficient construction. `two-stage-explicit` also calls
`_two_stage_derivative_fused_components`; it only avoids the later substitution
of U/F derivative bodies into its assembler. The existing native function-map
aliases in `_build_qmc_optimized_evaluators` wrap Korobov functions after explicit
coefficient expressions already exist, so enabling them does not remove this
generation stage. Changing the direct-projector cache threshold can select a
shipped Taylor projector for large IBP signatures, but still uses the same
explicit coefficient-construction owner and has no demonstrated benefit here.

The smallest existing route that does avoid that owner is replacing `--explicit`
with `--projector-generation`, while preserving the original card's
`symbolic-derivatives`, IBP policy, full sector/order coverage, O2 and precision
settings. `FSD.py` then prepares endpoint projectors, regular-Taylor formulas and
mapped chain rules instead of explicit per-sector expressions. Symbolic mode
uses shared U/F derivative evaluators; the existing formula guards and runtime
fallbacks remain the reference's responsibility. Its strict bundle serializer
already transports those artifacts, and `SectorProcessor` assembles the full
Laurent vector through its existing endpoint-projector route. Existing triangle
and box source tests compare explicit and projector complete pointwise vectors.
No new evaluator, algebra or reference source patch is proposed.

This is a feasible separately labelled whole-integral correctness reference,
with uncertain generation and sampling cost. It is not evidence that the
projector layout is the efficient explicit performance baseline, nor a reason
to replace FastSecDec's production direct O2 evaluator. Any concrete run remains
under coordinator scheduling.

## Bounded projector correctness attempt

Root authorized the existing route in a fresh
`output/diagnostics/onshell-pathfinder-projector-attempt-1/` directory. The only
scientific steering change was `--projector-generation` in place of `--explicit`;
the original card, symbolic derivatives, IBP, full 968-sector scope through order
zero, complex O2, precision policy, seed and 1,024×8 allocation stayed fixed.
The existing watchdog used 300 seconds for generation, 1,200 for conditional
integration, 1,600 for the enclosing process and 30 GiB of process-tree RSS.
Both phases were assigned CPU 8 with the existing stage log and stdlib traceback
wrapper. Root allowed overlap with native capability work on CPUs 0–7; this
attempt is not an isolated performance observation.

Generation timed out before producing a bundle. The watchdog sent SIGINT at
300.4 seconds, then SIGTERM and SIGKILL at its existing five-second grace
intervals. The generation record reports exit 124 in 310.706 seconds; the outer
process reports exit 124 in 312.223 seconds. Both recorded process groups were
absent after reaping. All 853 immutable checks and both read-cache inventories
passed. Integration was never launched, and there is no coefficient vector or
result.

Endpoint preparation completed in 6.833 seconds with the same 137 cache hits.
The next stage requested 226 regular-Taylor signatures; repeated stack samples
locate work in the existing `evaluator.dualize` call at
`subtraction_formula.py:750`. Thus the alternative bypassed explicit expression
fusion but encountered regular-formula preparation cost. The mutable cache
contains 174 JSON records, including all unchanged 137 seeds and 37 additional
records, plus 37 serialized evaluator sidecars and 38 lock files. Their presence
is not a complete-bundle certificate.
Maximum sampled generation-tree RSS was 6.147 GiB. Full logs, tracebacks,
inventories and `terminal-slot-release.json` are retained; no retry followed.

The completed new regular-formula prefix spans 267.372 seconds from its first
lock creation to its last completed JSON. Individual lock-to-JSON intervals
range from 0.002 to 174.167 seconds, with a 0.095-second median. These are observed
cache-write intervals, not isolated evaluator benchmarks or a forecast for all
226 requests. The last completed signature has five Taylor axes with maximum
orders `[2, 2, 1, 0, 0]`, seven outputs, 56 input slots and 126 native dual
components. The subsequent unmatched cache lock has digest `96f93624…`; its
signature and dual shape were not exported and cannot be reconstructed from the
hash alone. `regular-cache-progress.json` retains the complete data inventory.

Existing supported flags can avoid additional proactive universal-formula work:
`--regular-taylor-signature-limit 0` keeps already cached regular formulas but
does not build cold signatures, and `--chain-rule-formula-signature-limit 0`
skips proactive mapped chain-rule construction. The ordinary strict-bundle
runtime then uses its existing symbolic-derivative Taylor assembly for missing
auxiliary formulas (`integrand.py:3488` and `11711`); no coefficient, sector or
precision policy is removed. The reference's existing tests cover the guards,
and its documentation describes this fallback for hard triple-box signatures.
Any subsequent reuse must carry the completed native evaluator sidecars with
their JSON metadata, leaving lock files behind. This is a source-supported
correctness alternative with uncertain sampling cost, not a performance claim.

The supplied reference checkout contains only triangle, box and double-box
target JSON files, including its ignored example/output locations. No stored
complete original on-shell triple-box result was found. Its historical
`docs/FastSecDec.tex` records a six-order −6 through −1 stress result interrupted
at 30.19 GiB and explicitly labels it as unvalidated; it lacks the finite row.
Its leading-pole generation examples likewise do not supply the required full
vector. Those historical records cannot close the current full-integral
comparison.

## Completed guarded projector bundle

The separately authorized `onshell-pathfinder-projector-cached-attempt-1`
preserved the original physics and added the two existing zero signature limits.
It reused 174 completed JSONs with their 37 native evaluator sidecars, without
locks. Generation succeeded in 39.637 seconds. The native stage report records
137 endpoint cache hits, 37 regular-formula cache hits, 189 regular signatures
left to runtime, four chain-rule requests skipped by their guard, and no new
formula construction. The saved bundle covers all 968 sectors and all seven
orders −6 through zero. It contains 12,616 native evaluator files; the complete
12,621-file bundle occupies 311,773,686 bytes.

The enclosing harness then failed before integration because its normalization
check expected `gamma_argument=(4,3)` inside the parametric representation.
Native `pysecdec_bridge.py:1276` deliberately records `(0,0)` there for an
external-prefactor DOT integral; lines 1327–1342 separately construct and store
the Gamma series. The actual bundle records `gamma(3*eps + 4)`, U exponent
`(2,4)`, F exponent `(-4,-3)`, the original ten unit powers and seven regular
prefactor coefficients starting at 6. The failed check and its exit 1 are
preserved; generation itself succeeded and was reaped with exit 0.

The corrected data admission requires that exact native external convention,
the fixed U/F exponents and complete finite real prefactor coverage. Existing
`formatting.apply_global_convention` convolves the stored Gamma once into the
complete raw vector; the adapter adds no normalization arithmetic. Independent
source review accepted this correction. A fresh numeric-only continuation,
`onshell-pathfinder-projector-numeric-attempt-1`, binds all 12,621 original bundle
files and 13,632 total immutable inputs, then invokes ordinary strict integration
with the unchanged 1,024×8 allocation, seed, precision and full-scope arguments.
It was bounded at 1,200 seconds and 30 GiB on CPU 8, with no generation replay.

Strict integration exited 1 in 4.547 seconds: `PSD0` required an unprepared
numerator-epsilon dual shape. All 13,632 hashes, the complete original bundle and
read caches remained unchanged. This is an existing prepared-artifact coverage
limitation in the guarded projector fallback, not a normalization mismatch or a
numerical comparison result. The missing shape and full error remain in the
numeric attempt's stderr; no result was produced.

Root then authorized one ordinary reference `run` command with the same guards,
cached formulas, original full scope, precision and 1,024×8 allocation. The fresh
`onshell-pathfinder-projector-run-attempt-1` output avoids the CLI's existing
strict-bundle reuse branch. Its normal in-memory runtime may create the required
native numerator and chain-rule evaluator artifacts through existing APIs.
No strict-loader flag is changed internally, and no reference source is patched.
The single 1,500-second/30-GiB bound includes setup and integration on CPU 8;
overlap with native capability work remains explicitly non-benchmark evidence.
All 1,000 immutable checks and the exact command delta were verified before
launch. The attempt reached numerical integration but did not complete its
correlated iteration within the bound.

The existing quiet/JSON output does not report partial first-iteration work.
`integrator.make_progress_bar` returns `None` for either output flag, and its
progress updater consequently emits nothing. The native QMC loop updates that
display after each complete sector/support group, but calls the result-writing
callback only after every group has contributed to the full correlated
iteration. No result file therefore does not identify the current sector or
prove that no batches finished. The retained traceback establishes numerical
projector evaluation with the normal precision policy; later native chain-rule
cache writes establish additional lazy evaluator work, not a completed-sample
count.

If a further diagnostic is needed, the existing `update_progress_bar_timed`
arguments already contain completed raw work and its target. An isolated entry
wrapper can retain those values while calling the original function unchanged.
Recording entry/return of the existing `_evaluate_qmc_batch` would additionally
identify the current sector and batch size, including a batch interrupted before
the next group update. Neither observation needs evaluator, estimator or
precision changes. The frozen process was not modified.

The watchdog returned 124 after 1,502.472 seconds and the process group was
reaped. All 1,000 immutable hashes and both read-cache inventories passed. The
native interrupt handler wrote a result explicitly marked `interrupted=true`:
14 complete sector groups (IDs 0–13), 114,688 completed raw rows, and zero rows
in the reported correlated aggregate, against a 7,929,856-row target. This
partial result is not a full-integral estimate. Sampled watchdog RSS peaked at
0.650 GiB. Ten additional native chain-rule formula records completed; their
evaluator and expression sidecars remain bound by the final cache manifest.
`terminal-slot-release.json` has SHA-256
`64c9b3bb27dc939292691dd0111a837d8e7e2634600e5ee1542fea66c0a37b59`.

The next authorized coarse attempt, `onshell-pathfinder-projector-coarse-attempt-1`,
changes only the reference allocation to 16 points × 8 shifts, retaining all
968 sectors, seven orders, the original input and normal precision policy. It
uses the same ordinary non-strict `run`, a fresh output, 330 completed cache
assets without locks, and the reviewed observation wrapper. Its 1,500-second /
30-GiB CPU-8 bound includes setup and observer overhead. This is a coarse
independent full-vector cross-check; neither uninformative error bars nor an
incomplete aggregate can establish agreement. It is not a one-per-mille
certificate or a matched performance baseline. Native capability keeps its
original allocation. No complete result is yet claimed for this attempt.

The coarse attempt also reached its deadline before the later user stop request:
exit 124 after 1,504.552 seconds, with all owned processes reaped. All 1,240
immutable hashes and both read-cache inventories remained unchanged. Its native
progress records contain 63 completed groups (IDs 0–62), 8,064 raw rows and no
complete correlated aggregate. Group 63 remained active; retained traces show
successive lazy native chain-rule constructions, rather than one confirmed
unchanging expression. The interrupted result is not an independent full-vector
reference. Sampled maximum RSS was 3.137 GiB. Completed formula/evaluator assets
remain preserved; no result or uncertainty was promoted. The terminal record
is `onshell-pathfinder-projector-coarse-attempt-1/terminal-slot-release.json`,
SHA-256 `9979c7703ae4182798d648af411a491603331800fd805ec2ec0735a1b549c5f7`.

The user subsequently accepted completed FastSecDec generation of this on-shell
integral as sufficient for phase A and directed attention to faster cases.
Further on-shell integration, reference construction and performance attempts
are therefore stopped. Full-integral numerical agreement remains unverified
and deferred; the preserved partial records do not establish it.
