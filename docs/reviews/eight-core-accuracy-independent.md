# Independent review of the eight-core accuracy baseline

This review covers the protocol in `eight-core-accuracy-protocol.md` and the
ignored reader/runner source. The four smoke outcomes are reviewed below; the
paired campaign is not accepted because both reference worker runs aborted.

The protocol uses the user's corrected observable: the largest signed requested
epsilon order, zero for the proposed triangle and box cases. The finite-part
relative error is an observed native sampling error, not a certified true-error
bound. Full Laurent vectors, covariance where exported, precision diagnostics
and complete allocation coverage remain necessary even though the timing target
uses one coefficient.

The prescribed completed-allocation ladder avoids comparing incompatible
stopping rules: native FastSecDec's tolerance checks every component, while the
reference CLI uses an aggregate measure. Independent final rows at fixed N and
16 shifts are inspected without pooling different lattice sizes, interpolating
a crossing or treating a partial snapshot as complete. The first successful
row gives a grid-observed bound; if the first 1024-point row succeeds, an earlier
crossing remains unmeasured. All earlier row costs remain in cumulative timing.

The explicit common Kuo33002/linear/full-support/Korobov3 choice, disabled
reference point cap and unchanged precision safeguards are appropriate. Equal
integer seeds do not imply equal random shifts across implementations. The
reference's validated complex O2 path and the documented backend-version
difference must remain visible in any comparison. No reference covariance or
aggregate work should be fabricated where its public output lacks them.

Eight physical cores must be verified from the actual allowed cpuset and
package/core identities. The proposed singleton native-library thread settings
avoid nested parallelism while each CLI retains eight workers. No overlapping
project timing or build is allowed in measured rows; host exclusivity is not
claimed. Generation, load/setup, numerical loop and full process costs are kept
separate. The maximum sector mean is correctly distinguished from an observed
individual-sample maximum; the latter belongs to a separate instrumented
diagnostic.

No protocol blocker was found. Before execution, the runner review must verify
the exact commands and source/build bindings, actual complete N/shift coverage,
finite nonzero target selection, preservation of lower orders and failures,
alternating fixed seed campaigns, and the cumulative cap. The reviewer asked
the author to make the 600-second cap's metric explicit and bound each next
watchdog by its remaining budget. Timeout grace/shutdown costs must remain
reported rather than appearing as a successful crossing. The labelled smoke
pair is excluded from timing summaries and must pass these checks before the
measured campaign begins.

The first reader/runner source review checked
`output/probes/eight_core_result.rs` and `output/probes/eight_core.sh`.
Three reader preflight gaps were reported and corrected: the selected CPUs must
be present in the allowed affinity mask; native rule/periodization/method are
checked from the typed effective design rather than only caller metadata; and
reference replica counts must equal R for each declared active coefficient,
with zero counts permitted only for inactive orders. This agrees with the
reference integrator's `active_laurent_orders` and per-order count fields.

The shell clips each row's deadline to the remaining 600 seconds of completed
process wall, preserves failed process reports, and names its sum of completed
processes accordingly. It alternates whole program ladders across fixed seeds
without rerunning a failed row. Watchdog shutdown remains recorded. A remaining
source-freeze request was to hash the imported reference Python source tree,
not merely the top-level CLI and lock file, and verify it before/after execution.
The author added a tracked/untracked Python-source hash manifest and explicit
preparation/campaign verification. Failed/incomplete row process costs are now
also summed separately from the completed-loop totals.

The coordinator also correctly distinguished residual consistency from
comparison eligibility. The historical triangle/box target transport remains
`Unverified`; finite pulls below five can be a frozen-target residual check but
must not silently upgrade that status or certify the uncertainty model. The
previous independently executed analytic/native-master checks are separate
scientific evidence. The reader now retains both labels explicitly and calls
the criterion `full_vector_residual_check_pass`. It also labels the elapsed
boundaries: the reference loop may include pool startup and lazy evaluator load,
while native eager artifact loading and pool preparation precede its driver
timer. Total process time remains necessary to assess those different paths.
No static source blocker remains for the labelled smoke stage. No smoke or
seven-pair timing result has yet been accepted by this review.

The legacy reference JSON includes non-finite tokens in some per-sector
diagnostics even when its complete physical vector and standard errors are
finite. The reviewed `reference_json_transport.sh` uses jq's existing parser
and `walk` to tag only numeric NaN or positive/negative infinity, retaining the
original bytes, SHA-256 and length. It neither replaces these values by zero nor
changes strings or null. The Rust reader still requires ordinary finite numbers
for every physical mean/error and every residual pull. This transport runs
outside the timed process. Its parser capability controls and actual jq identity
must accompany preparation evidence; tagged diagnostics are unavailable values,
not newly valid numerical estimates.

The retained transport control under
`output/diagnostics/eight-core-reader-validation/transport/` distinguishes NaN,
unsigned/explicit-positive/negative infinity, the corresponding quoted strings,
null, positive and signed zero, a finite decimal and an escaped quoted string.
Inspection of its strict output confirms the intended tagging without changes
to the finite/string/null values. The final reader also accepts the four prior
one-worker 8192×16 triangle/box records, including the reference records that
previously failed strict JSON parsing. This is a parser/extraction control only:
those older records are not fresh eight-core smoke or performance evidence.

## Fresh preparation and smoke outcome

The reviewer inspected the completed records in
`output/benchmarks/eight-core-preparation-20261005` and
`output/benchmarks/eight-core-smoke-20261005`. All four generation processes
exited zero. The full artifact/bundle and frozen-executable/source hash manifests
were independently rechecked successfully after smoke. The frozen reader is
`dd7a42acaa2e1a92d40b63b8058e14402a7477f8e4b3edcddc76491d31c63bb4`;
the runner is `b4bc65b56300154841b647a6cdeb99b4fa4bb10bfca6a6aaa024d3227ac599a0`.
The jq 1.8.2 executable is also hash-bound. CPU0–7 are allowed, distinct physical
cores on socket/node zero, with SMT siblings 256–263 excluded. Every numerical
command requests eight workers and inherits `taskset -c 0-7` with singleton
nested-library thread settings.

Both native rows completed the prescribed 1024×16 allocation at seed 20261210.
Triangle covers two complete sectors/32,768 evaluations; box covers three
complete sectors/49,152 evaluations. Each sector contributes all 16 common
replicas, and the full orders `[-2,-1,0]` plus covariance survive in the native
saved results. The maximum signed order is zero. Triangle's finite part is
`0.6558780721990614 ± 2.8856239473106127e-9`; box's is
`-12.49312230872735 ± 8.116940021334103e-6`. Both satisfy the observed 0.1%
relative-error criterion and the frozen-target residual check. Comparison
eligibility still explicitly reports `UnverifiedReference` and `MissingEstimate`
(the historical targets include imaginary keys); no validation is promoted.

The native production precision policy remains boundary threshold 1e-3,
128–4096 bits, relative tolerance 1e-12 and absolute tolerance 1e-300, with the
16×/128-bit weighted replay policy. Triangle/box retain 3,207/4,516 rescues,
respectively, at a maximum 320 bits and zero evaluation failures. Their recorded
driver intervals are 0.041874569/0.049499684 seconds, eager artifact loads
0.005482508/0.006639842 seconds, and complete processes
0.053248097/0.061733734 seconds. These single smoke observations are excluded
from timing summaries and do not estimate a median or speedup.

Both reference eight-worker processes abort with signal 6 before returning a
physical result. Their stdout repeatedly reports that another unlicensed
Symbolica instance is already running. The processes are not watchdog timeouts;
their retained wall times are 1.119443074 and 1.114424386 seconds. Input,
executable and build hashes remain unchanged. The author reports all child
groups reaped. The reviewer accepts these as retained environment/capability
failures, not numerical estimates or performance measurements. The prepared
bundle's complex O2 setting is confirmed, but successful serial generation does
not establish that its lazy multi-process evaluator loading can run here.

**The smoke does not qualify the seven-pair campaign.** No eight-core reference
speedup is available, and a reduced-worker run would be a different campaign.
Read-only investigation of the native reference pool/loading contract can
identify the constraint; this review does not authorize altering licensing or
discarding these failed attempts. The native elapsed interval excludes eager
load/pool setup, whereas the reference elapsed interval, if a future valid run
returns one, includes pool startup and possible lazy evaluator loading. Only
complete process times share the same outer boundary.

The coordinator separately approved a native-only continuation. The reviewer
checked `eight_core_native_only.sh` against the frozen paired runner: the changes
are limited to the admitted mode, explicit `accepted_native_only=true` /
`paired_accepted=false` acceptance fields and `programs=(native)`. The original
runner remains unchanged; seed block 20261211–20261217, allocation ladder,
coverage reader, precision, affinity and bounds are retained. The two successful
native smoke rows support that separately labelled measurement. Its results
must not be presented as paired-reference timing evidence.

The native-only continuation subsequently completed all fourteen prescribed
rows in `output/benchmarks/eight-core-native-only-20261005`. Independent JSON
inspection checked the exact seven seeds per case, no smoke-seed reuse, full
orders/covariance, common-shift coverage, effective Kuo33002/Korobov3 design,
eight workers, unchanged weighted replay policy, zero failures and all process
provenance guards. Frozen source/executable and summary-input hash manifests
rechecked successfully. All rows satisfy the finite-part target at their first
1024×16 complete allocation. Every historical real-coefficient residual is
within 2.81 combined errors; comparison eligibility remains unverified.

| Native case | Median driver interval (s) | Median eager load (s) | Median complete process (s) |
| --- | ---: | ---: | ---: |
| Triangle | 0.032968704 | 0.004713895 | 0.043067946 |
| Box | 0.041598931 | 0.006712451 | 0.054015710 |

These independently recomputed presentation medians agree with the author
summary. All row vectors and covariance remain separate; no across-seed
estimate or new uncertainty calculation is introduced. The accepted package
intervals give median pooled worker costs of 3.8842/3.8415 microseconds per
point for triangle/box, and median slowest-sector averages of 7.5997/5.8863
microseconds. These include point generation, transformation, evaluation,
precision rescue and accumulation. They are worker-time averages, **not measured
individual-sample maxima**. Precision reaches 256 or 320 bits with rescues active.
The retained `independent-review.json` and `independent-pulls.json` record this
data-only audit. No paired reference timing, optimal crossing time or hard-case
performance acceptance follows from these native-only observations.
