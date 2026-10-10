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

## Full-pilot runner review, execution pending

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
There is no production sampling. Full admission remains pending its actual
four-arm results; even success certifies the prescribed finite pilots,
not every point of the integration cube.

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
