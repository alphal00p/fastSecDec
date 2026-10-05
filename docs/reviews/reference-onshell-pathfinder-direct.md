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
