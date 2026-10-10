# Independent K1 runtime admission audit

2026-10-10. Read-only review of the caller-owned harness under
`target/contour-ltd-k1-runtime/` against the native source accepted in
`3303005`. The runtime agent owns linking, monitored execution and subsequent
load attribution. This audit ran no generation, sampling or Cargo command.
Raw programs, artifacts and reports remain ignored.

## Scope and input identity

The proposed comparison is the complete finite complex coefficient of
`2L4P.b.K1`, using fixed strengths 0.001 and 0.005, and polynomial dynamic
strength with S=0.8, R=1 and L=0.01 or 0.1. It does not test the sign-aware
construction. The native catalogue must contain exactly 186 six-dimensional
stochastic sectors, global orders `[0,0]`, components `[Real,Imag]`, and local
output maps `[0,1]`. A different grouping or projection requires another
reviewed pairing plan.

The runner pins the historical fixed manifest SHA256, native kernel identity,
data filename and size to `fixed-inventory.json`. It checks equality of fixed
and polynomial scientific provenance: source fingerprints, dimension,
regulator, measure and multiplier, domain, requested order and threshold
policy. The polynomial native archive must carry source identity
`d27e82184cdd7afc90bc12f27b8e6b6bfcd3786007326bb7f8e9c3a5c8b394d5`.
These fences matter because matching geometry and F/U alone would not detect
a changed scalar prefactor. The explicit component check and provenance
fences were added during this review.

`maps()` uses the native selected readers and compares actual source-index
groups, representatives, permutations, geometry, source/target coordinates,
undeformed coordinate images and measure Jacobians, designated causal F,
ordered positive factors and retained face sets. It intentionally does not
require equality of the different deformed maps. At most one fixed and one
polynomial stochastic owner are resident for each comparison.

The input and reference provenance is retained in
[the LTD fixture audit](contour-ltd-fixtures.md). The supplied analytic value
is `(-1.0840618909337886 + 2.8682065140371712 i) × 10^-6`; no reference
uncertainty was supplied. The input already applies the paper-measure
conversion `(4*pi)^(2*eps) * -1/(256*pi^4)`. The harness applies no second
phase, loop measure or normalization. Historical fixed estimates were
unconverged and are not a new numerical reference or timing baseline.

## Actual owners and exact offsets

Every selected exact or stochastic record is bound and piloted as its own
native owner. The 16 source points are the centre, each of the twelve
individual coordinate faces with the remaining coordinates at 0.5, two
interior corner points and one alternating-coordinate point. Native
`validate_contour_point(..., true)`, `finish_contour_pilot()` and full-scope
readiness supply the coverage decision. This finite pilot is not a proof at
every point of the integration cube.

`load_exact()` creates a new owner. The initial draft incorrectly relied on
the earlier per-record pilots; the corrected harness also calls `admit()` on
the actual aggregated exact owner, both in preflight and in the prospective
production path. Readiness is not transferred between owners.

Native `record_reader::load_selected(exact_only=true)` loads only records
whose `sector` is absent. `ResidentAssembly` sums the exact Atoms, performs
the shared native numeric-coefficient normalization and prunes cancelled root
associations. After binding this owner, its complete vector is placed once in
`IntegrationProblem.exact_coefficients`. A stochastic
`WeightedEvaluationContext` retains only the selected sector, not those
offsets. Native `QmcSession::estimate()` adds the exact vector once to the
mean; it does not add deterministic offsets to covariance. The same exact
vector appears once per complete-shift diagnostic vector. No separate
floating-point exact-offset accumulator is introduced.

## Prospective sampling and statistics

Paired production remains disabled in `plan.json`. The prepared code uses
native `QmcSession`, `QmcWorker`, Kuo lattice rules and Korobov3, with two
predeclared seeds and reversed arm order. Different kernel content IDs fence
task submission but do not alter the democratic QMC stream: the native stream
is `0xD000_0000_0000_0000`, and the seed/rule/dimension determine its points.
The harness additionally hashes every actual coordinate and weight with its
sector and range index and requires equality across completed arms.

Native democratic QMC computes covariance from the complete common-shift
full-sector sums. It therefore retains real/imaginary covariance and
cross-sector correlations; it does not incorrectly sum independent-sector
variances. The requested complex target variance is the trace of the native
2x2 covariance. Pilots are separate from production, and production contexts
must show zero optional checked arguments after successful Pilot admission.

No estimate is emitted unless the native session is complete. Failed or
preflight-rejected arms are labelled and receive no full-integral result.
Componentwise agreement within five sampling standard errors plus the
declared absolute floor `1e-14` is only a diagnostic. It is distinct from
the requested relative joint uncertainty and from convergence. The floor is
not a reference uncertainty. This source review authorizes no production
run and establishes no variance advantage.

## Execution boundaries and first result

Only `maps` and conditional `admission` are enabled. Admission requires a
complete native map proof with the same artifact evidence and executable
hash. The Rust admission command can exit successfully after recording a
failed arm; `admission-summary.json` separately requires every arm to have
status `complete`. Process success alone is not scientific admission.

The caller-owned monitor measures only its process group, with a 3 GiB
aggregate RSS ceiling and a 600-second cumulative budget. Initial phase
limits are 120 seconds for maps and 180 seconds for admission. Time expiry
sends SIGINT, followed by SIGKILL after a grace period; RSS expiry kills the
owned group. No unrelated process is terminated. Runtime execution began
only after the generation monitor handoff. The runner now refuses existing
phase commands, run directories or result files before writing new evidence,
closing a retry path which could otherwise relabel an old proof after a
rejected second run.

The first `validate=true` map attempt stopped at its time limit after
120.433811286 seconds, peak aggregate RSS 326762496 bytes. Ten sectors
(0 through 9) completed native comparisons; sector 10 was still loading.
This is an incomplete full-map gate, not a mathematical disagreement.
No complete map proof, admission result or production statistics follow from
it. The source and executable used for that attempt have SHA256:

- Source: `7a43d11eb532e2ca03b330fb4de96e7767c3c6273ff0a0ab7284b5149308e969`.
- Executable: `c44c578acbe91078606d6188379977bb499f5a0a0697618cae5c149f007cdf1d`.

## Restore-path source audit

`binary::load_with_progress` restores the saved native StateMap, helper
programs and optimized sector instructions. It does not invoke symbolic
evaluator construction, Horner/CPE optimization or compact-function lowering.
Native coefficient mapping and backend preparation, including JIT when
selected, still occur; loading must not be described as restoring cached
machine code.

Optional `validate=true` work includes record/envelope/catalogue identity
checks and semantic JSON hashing. `StoredAtom::serialize` canonically prints
all retained metadata, including the large contour Jacobian and compact
bodies. It also rechecks polynomial support, reconstructs coordinate geometry
and verifies compact body-to-name hashes. Mandatory work includes compact-call
structural scans over images, ratios and Jacobian, native program arities,
recipe/helper/certificate association and ordered runtime-symbol checks.
`ContourDefinitions::select` currently constructs argument vectors while
walking calls even when it only needs their referenced identities. Descriptor
validation repeats during restore, generation association and certificate
metadata admission, but none of these paths rebuilds saved checker arithmetic.

The runtime agent's subsequent single-record probe restored polynomial
sector 0 with validation enabled in 7.915375194 seconds and with validation
disabled in 1.368878922 seconds. The native backend restoration intervals
were 0.306151547 and 0.303602967 seconds respectively; the difference preceded
that interval. Both runs emitted the same inspected map metadata and performed
zero sample evaluations. This local pair supports expensive optional
revalidation as the main difference, but does not time canonical printing,
polynomial support and geometry separately, nor establish an idle-host
benchmark. No upstream bug or missing native operation has been reproduced.

Using `validate=false` would preserve every explicit native map comparison
above and all mandatory structural checks. With a separately verified complete
artifact SHA pinned to the generation handoff, it can support a narrower
trusted-artifact comparison under the user's optional-validation contract.
It would not repeat body-identity, geometry or semantic-integrity proofs for
every record and must not be reported as doing so. Root subsequently authorized
a distinct 600-second/3-GiB complete-map attempt using that policy, conditioned
on fixed and polynomial complete-file SHA verification and unchanged native
comparisons. The old partial attempt remains evidence. A first-source pilot
cost probe may follow only after a complete map proof; paired production is
still disabled. The runtime agent owns those changes and execution; their
outcomes are separate from this source review.

The new `maps-trusted/` runner passed independent source review before
execution. `maps()` is unchanged. Before establishing the fixed whole-file
SHA, its small integrity helper opens the native legacy catalogue with
validation enabled, compares it to the catalogue in the original SHA-pinned
manifest, and streams every original record's BLAKE3 receipt. It decodes no
native payload. The polynomial complete-file SHA must equal the generation
handoff. The outer monitor bounds this entire integrity, hashing and native
map operation, not only its final numerical-owner restore. A fresh one-shot
marker prevents retries from overwriting evidence. The additional isolated
`pilot-cost` binary branch is not invoked by the map runner; it requires a
separate complete-map and identity check before any later execution. No
remaining source blocker was found for this authorized map attempt.
At the first trusted-map review, before the later pilot-cost refinement, the
probe source SHA256 was
`43f28ed95a072613a04f214a75418b1a17e87bbc57d841b4bcd054909530fe89`;
its linked binary SHA256 was
`51704fe2d1eb5ececfa5fd7331f6c207db9fb51ab02eb85dcea7dcc7b9cdf7af`.

## Final pilot-cost branch review

Before launching the trusted map run, the runtime agent finalized the
isolated cost branch so both phases use one pinned executable. Its runner
requires a complete map proof with the exact ordered list of sectors 0
through 185 and every native comparison true. It checks the current executable
and source SHA against the map command, rehashes both complete data files,
and records hashes of the map proof and command. Missing or changed evidence
refuses execution. Its own one-shot marker prevents relabelled retries.

For each unchanged arm, the native branch loads, binds and pilots the actual
aggregate exact owner, finishes and checks its readiness, then drops it.
It subsequently loads and pilots sector 0. Successful reports distinguish
record load time, parameter binding time and point certification/finish time.
Failures retain their raw error; structural typed failures stop remaining
arms. Every report states `full_integral_admitted=false`, and the runner
states `production=false`: the other 185 stochastic sectors are not piloted
by this cost probe. No sampling loop or estimator is called by this branch.

This final source review found no remaining ownership or proof-association
blocker. The outer monitor must wrap the runner, including repeated hashing,
under the separately authorized cost budget. Actual map completion and pilot
outcomes remain execution evidence to be reported separately.
Reviewed final probe source SHA256:
`fc22e8f0b30e0f8a7e28d2f4551fefcbcadb1a18b31943b02e468beda741f761`;
pilot-cost runner SHA256:
`ac7578fbdbb5b532dc98a29f7f4588b09bce590d0042436b9c73daaa9ace37f4`.

The subsequent trusted run completed all 186 explicit native comparisons.
Its owned-process monitor closed after 261.226 seconds with peak aggregate
RSS 401584128 bytes. Before native loading, the fixed native catalogue
matched the pinned manifest and all 372 original record digests passed,
covering 244110752 record bytes. The resulting fixed whole-file SHA256 is
`a1a2ab0c3ec274cf3b0382b9bbfbe10e20d58a6f0327d785399f09b90fbf7ae0`;
the polynomial whole-file SHA matched its generation handoff. This completes
the stated trusted map-comparison gate, not numerical pilot or integral
acceptance. The separately bounded pilot-cost measurement follows it.

The pilot-cost monitor subsequently exited successfully after 11.045521
seconds, with peak aggregate RSS 123392000 bytes. All four arms passed the
actual aggregate-exact owner's binding and readiness checks; those owners
had no surviving requests and therefore accepted zero pilot points. Sector
0 passed all 16 prescribed source points in every arm, reaching at most
96 certificate bits. Production checked-argument counts remained zero.

| Arm | Sector load (s) | Sector bind (s) | Sector certification (s) | Checked arguments |
| --- | ---: | ---: | ---: | ---: |
| F001 | 0.106034 | 0.000970 | 0.076528 | 80 |
| F005 | 0.100123 | 0.000604 | 0.071497 | 80 |
| P01 | 1.418088 | 0.005461 | 0.416856 | 16 |
| P1 | 1.392033 | 0.004182 | 0.409520 | 16 |

The fixed homotopy arguments and dynamic bounded-homotopy requests are
different work units. These measurements are debug-host feasibility and
cost attribution, not optimized-host performance comparisons. The raw
summary is retained under `target/contour-ltd-k1-runtime/pilot-cost/`.
Exactly 185 stochastic sectors remain unpiloted by this experiment. No
production, full-integral estimate or convergence claim follows from it.

## Full-pilot runner review and completed admission

The separate `full-pilot/` runner passed read-only review. It invokes the
unchanged, hash-pinned native `admission` branch only after complete map and
four-arm cost prerequisites pass, and rehashes both complete artifacts.
The native loop loads and pilots each of the 372 records separately, drops
each owner, then loads and pilots the actual aggregate-exact owner. Its
summary accepts an arm only after exact record coverage 0 through 371,
stochastic coverage 0 through 185 and all native readiness reports pass.
Global source IDs come from the complete map proof, rather than merging
record-local chart indices. Failed arms retain their native error and last
progress; a process exit code of zero alone is not scientific admission.

The one-shot outer monitor bounds the complete runner to 900 seconds and
3 GiB of owned-process RSS. Concurrent private release compilation and K1*
generation are explicitly recorded, so timing comparisons are excluded.
There is no production sampling. Success certifies the prescribed finite
pilots, not every point of the integration cube.

The full-pilot monitor subsequently closed successfully after 718.760973
seconds with peak aggregate owned-process RSS 1153024000 bytes. Independent
read-only verification of all four raw `admission-*.json` reports confirms
exact record coverage 0 through 371 and stochastic coverage 0 through 185,
with required and validated chart sets equal for every owner. Each arm
accepted 2976 points (16 per source), reaching at most 96 certificate bits.
F001 and F005 each checked 14880 homotopy arguments; P01 and P1 each checked
2976 dynamic requests. Each separately loaded aggregate-exact owner was
ready with zero surviving requests. All production check counters remained
zero, and the monitored process group was independently confirmed empty.

| Arm | All record loads (s) | Binding (s) | Certification (s) |
| --- | ---: | ---: | ---: |
| F001 | 18.020559 | 0.129992 | 12.843739 |
| F005 | 17.929875 | 0.129866 | 12.804176 |
| P01 | 259.158295 | 0.711026 | 63.115507 |
| P1 | 258.801046 | 0.722344 | 63.324953 |

These phase observations include the explicitly recorded concurrent build
and K1* feasibility work. They are not optimized-host timing comparisons.
The complete finite pilot gate now passes all four arms; full-integral
production, accuracy, convergence and variance acceptance remain separate.
Raw report hashes and independently recomputed counts are retained in
`target/contour-ltd-k1-runtime/full-pilot-independent-audit.json`.

## Separate sign-aware source-zero evidence

The generation agent's `target/generation-agent-ltd-next/` probe independently
exercises the actual native massless K1 importer, shared source preparation,
source chart 0 discovery, native sector generation and SymJIT compilation.
The worker unit uses its real chart with the identity permutation and no
invented symmetry multiplicity. It saves a 26545373-byte v12 record and drops
the generated unit before numerical work.

With S=0.8, L=0.01, R=1 and SignAware/Always settings, the original compiled
owner passes its two-point native pilot and readiness check, then evaluates
the complete exact-plus-sector complex vector at the two interior points
`[.2,.3,.4,.5,.6,.7]` and its reversal. A fresh-process continuation restores
the same bytes with validation enabled, checks the unbound identity and
repeats the actual pilot and point evaluations. Both two-component vectors
match exactly in the recorded output. The saved record SHA256 is
`b6448affd73643ba28c10ba7bfdbb21bc6d1a8b2a24361328b3c15ee35d63a3d`.

The initial run's failure is retained: its driver compared the restored
unbound template identity with the original owner's identity after binding.
The correction compares like states and reuses the already-saved record;
it changes no scientific kernel or contour setting. The first process took
63.121176490 seconds with peak RSS 765128704 bytes; the successful fresh
restore continuation took 14.179567324 seconds with peak RSS 388145152 bytes.
These are separate workloads, not a performance comparison.

This source and report review supports one actual sign-aware chart's native
generation, checked point evaluation and saved-owner parity. Two interior
points do not establish all-face coverage, all 186 charts, a full-integral
value, or an independent Jacobian oracle. The full polynomial map/admission
campaign and any later sign-aware integral campaign remain separate gates.

## Private optimized consumer build

A separate native CLI release build completed from exact commit
`33030058d2cd0587dd718ef3e7dc9c56ba6952f8`. The complete Git archive SHA256 is
`219524a8cffb8abab3074950b77c810ee39aa4a292e6cfe0108a94fa6de34d40`;
the original Cargo.lock SHA256 is
`4b110f43930f058044c5d4743e5b886e014f2189ac097ec262b929174836376d`.
Post-build verification matched all 1405 extracted source files, the archive
and lock, and unique pinned owners: Symbolica/Numerica `516beb37`, SymJIT
`d74993ff`, and Feynkit/Linnet/Spenso/Idenso `8e3a643f`.

The CLI-only command was `cargo build --release --locked --offline -j 2
-p fastsecdec-cli --bin fastsecdec`, inside the archived `shell.nix`, with
a unique private `CARGO_TARGET_DIR`. Actual Cargo artifacts show opt-level 3
for the native numerical owners and FastSecDec. The active CLI invocation
confirmed opt-level 3 and thin LTO; no extra target-CPU or numerical settings
were supplied. The compiler was Rust 1.97.1 with LLVM 21.1.8, GCC 15.3.0 and
GNU ld 2.46. Source and command identities, compiler/linker versions and
Cargo output are retained under `target/contour-private-release-3303005/`.

The independent 1800-second/12-GiB monitor completed after 919.142867 seconds,
with peak aggregate owned-process RSS 3531304960 bytes. All owned processes
exited, and the resulting 69557560-byte private executable has SHA256
`6cea9df3afd3d9a95663057a32c5313429d6a009ff95d16eeba1d615be4aef50`.
Its `--help` smoke passed. The user's existing `target/release/fastsecdec`
retained its original hash, inode, size and modification time; neither shared
Cargo output nor an installed environment was changed.

The build overlapped the separately monitored K1* generation and full K1
pilot only after explicit authorization. Its elapsed time is build
feasibility, not a controlled performance comparison. No physical production
was performed by this build gate. Exact matching release rlib fingerprints
and native link directories were supplied to the runtime agent for a
separate optimized consumer; a numerical release-control gate remains
required before any optimized-runtime performance claim.

## Separate massive K1* family publication review

Read-only manifest, catalogue receipt and durable-journal checks confirm the
completed K1* archive contains all four recipes under one physical source
identity, `740924389a4ae0f3debe607a6924204973692fecb2431b64759a068f35906579`.
Each recipe has 30 six-dimensional stochastic records covering sources
0 through 29 exactly once, plus 30 exact records. All 240 published record
spans are contiguous through the catalogue's `records_end`; 352 journal
responses are complete, including all 120 recipe-specific sector jobs.
The selected default remains `undeformed-v1`, with real output, while the
three contour recipes preserve the complete real/imaginary finite vector.
`assume_no_threshold` remains false.

The manifest SHA256 matches
`508005ecd661b7a884a49696d394a917ed206173c201045208606d7ac00b2a20`;
its referenced data file exists at the recorded 1206110163-byte size. This
independent audit checks published metadata and receipt structure, not a
second payload digest pass or evaluator restoration. The generation owner
records the full data hash and compilation evidence separately.

The eight-worker monitor exited successfully after 894.550629 seconds with
peak aggregate RSS 7758467072 bytes, inside its 900-second/8-GiB bounds.
The process group is empty. The concurrent private build and massless K1
pilot are explicitly recorded, so no controlled timing claim is made.
These checks establish complete family publication; neither the stored
undeformed capability nor successful compilation establishes the massive
integral's physical value, runtime causal admission or convergence.
The independent metadata audit is retained at
`target/generation-agent-ltd-massive-family/foundation-publication-audit.json`.

## Massive K1* finite-admission evidence review

The later K1* runtime gate closes the finite-admission gap left by the preceding
publication review. Independent read-only checks of the frozen source,
authorization, executable and seven primary raw-report hashes agree with the
[massive runtime report](contour-ltd-massive-runtime.md). The reviewed summary
has SHA256
`019805037c99a6d9f7845ed57a805e403fba19bb451c4f4c493bf4a25003f13b`;
the native driver's SHA256 is
`543523438f9ee009d8796490f319098798cd271205476f2a21264b0650063f48`.
It uses the verified release core from `3303005`, without production changes.

The completed native integrity report verifies all 240 receipt digests and the
pinned catalogue before payload restoration. The 60 map results cover exactly
sources 0 through 29 for both fixed/polynomial and fixed/sign-aware comparisons.
Each of the six prescriptions contains every record index 0 through 59 and
each stochastic source 0 through 29 exactly once. The recorded settings and
16 actual pilot points match the predeclared plans, including both individual
faces of each of the six axes. All required native chart readiness reports
complete: 480 source-point checks per arm, 2,880 total. Each freshly restored
aggregate exact owner is admitted separately, with zero required roots and
the exact vector `[0,0]`; its readiness is not counted as invented point work.

The conditional source-zero reports contain all 12 arm/seed runs, each with
eight contiguous native packages and 8,192 assigned points. The exact recorded
package ranges, coordinate and weight hashes match across all six arms for
each seed. All 98,304 points complete without a failed report; native metrics
retain the 17–18 DoubleFloat retries per arm and zero arbitrary-precision
calls. Pilot and production remain separate, with zero production causal
checks. Independent arithmetic on the existing counters reproduces the
reported source-zero average costs, including retry work. No estimate,
covariance, reference comparison or full 30-source allocation is emitted.

The raw monitor records exit zero, no limit, and an empty process group after
114.713877740 seconds with peak aggregate RSS 455,798,784 bytes. The planned
concurrency label is historical setup: the earlier massless K1 accuracy
invocation had already rejected its input basename and exited before this
run began. Neither an observed overlap nor an idle-host comparison follows
from that label. The independent audit is retained at
`target/contour-ltd-massive-runtime/foundation-evidence-audit.json`; it performs
no new numerical evaluation and does not repeat the full data-file digest
pass. Full-integral accuracy remains an open scientific gate.

## Accuracy-run wind-down after the priority change

The first standard-CLI accuracy invocation supplied `.fsd.json` instead of the
required `.fsd` basename. It exited before sampling; its failed setup evidence
is retained separately. The corrected one-shot launcher changes that basename
and the fresh output paths only, while hashing the same `.fsd.json` manifest
and retaining the same point, settings, seed and limits.

The corrected run was interrupted on the user's new priority, not at the
requested accuracy. Its outer process group closed after 311.853127828 seconds,
with no hard kill, no remaining processes and a persisted checkpoint/result.
The native result records `NumericalFailure: resident worker exited unexpectedly:
signal: 2 (SIGINT)`. That is a cancellation-classification race, not evidence of
a failed integrand evaluation: the reported numerical failure counter is zero.
The serial coordinator observes worker exit before polling caller cancellation;
its final observer return is also ignored. No source change or further LTD
execution is made by this audit. New ordinary-process timing monitors should
signal the coordinator first and reserve whole-group killing for failed grace.
The saved partial statistics are not promoted to accepted 0.1% accuracy.
