# Independent Stage A orchestration review

This is a source review of the ignored `output/probes/massive_holdout.rs` and
`native_qmc_diagnostic.rs`, against the committed
[campaign design](convergence-campaign-design.md). The reviewer did not author
the runner. No Stage A numerical row has been executed as part of this review.

The draft enumerates all 72 prescribed combinations: six fixed massive inputs,
two published rules, two point counts and three holdout seeds. It rotates rule
order by seed and count order by case. Every row uses 16 shifts, Korobov3,
1,024-point packages, one caller-owned worker and one complete democratic
allocation. The total work is recomputed from the frozen partition and checked
against the reviewed allocation; a partition change stops preparation for review.

The numerical helper uses the native `KernelSet`, `QmcSession`, workers,
`WeightedEvaluationContext`, observations, checkpoints and complete-shift APIs.
No independent estimator or covariance calculation has been introduced. Replay
maxima are accepted after successful native submission, while failed package
prefixes remain excluded. The authoritative total retains cross-sector common-
shift covariance and the exact offset. Concrete native rules and the complete
effective design are retained; actual shifts remain in native checkpoints.

The summary aligns complete vectors by seed and shift, concatenates the two rule
outputs and invokes native `QmcEstimate::from_shift_means` only when all 48 rows
are present. Each point count gets its own summary. Incomplete or failed rows
are explicitly listed and cannot produce a selected-survivor joint estimate.
The session's centered total remains authoritative; the absolute shift-vector
summary is labelled diagnostic. Frozen references retain their nonzero standard
errors and original provenance, and comparisons use the native adapter.

The parent process preserves all row outcomes, including hard timeouts and rows
not started before the stage deadline, then continues independent rows. A hard
process kill can only retain previously written evidence; it must not be
described as a complete or resumable numerical observation. Soft numerical
timeouts return native accepted coverage and a checkpoint.

## Findings before launch

The first draft read the historical CLI envelope as arbitrary JSON and passed
only its kernel field to `KernelSet::from_bytes`. The latter validates native
kernel content, but does not validate the outer CLI identity, source fingerprints
or recorded dependency states. The existing six artifacts have older dependency
provenance. Preparation needs explicit validated envelope/input evidence, plus a
decision between regeneration under the current CLI and a clearly labelled
historical-generation/current-recompilation experiment. Historical dependency
metadata must not be relabelled as the current execution environment.

The coordinator selected regeneration of all six artifacts through the current
production CLI. Preparation will bind existing CLI inspection/preflight evidence
to the exact frozen outer bytes and preserve current outer/inner identities,
dependency states and input fingerprints. The ignored numerical runner may then
load the same validated inner kernel without duplicating the envelope validator.
Old exploratory artifacts and the external references remain unchanged.

The current linked build must also be identified independently of the old
artifact: an executable hash, source revision and `Cargo.lock` alone do not
describe local patches in path dependencies. Preserve the actual linked build
and dependency evidence along with each frozen campaign.

A derived reference-comparison error originally occurred before writing the
accepted report/checkpoint. The author has separated that error from the native
observation so comparison failure cannot erase accepted numerical evidence.
The revised direct row command rejects existing report/checkpoint evidence and
creates an exclusive attempt marker before evaluation, preventing a failed row
from being overwritten under the same identity. Incomplete or failed numerical
rows now return a nonzero process status after writing their retained report.
These failure-retention findings are closed by source review; they are
orchestration boundaries, not changes to native statistics.

The revised preparation now invokes bounded current CLI `generate` and
`inspect` commands for each case, preserves their arguments/results/process
records, verifies unchanged outer bytes during inspection, and binds the
inspection's identity/provenance to those bytes. Generation provides the source
fingerprint evidence; inspection is not claimed to reread source files. The
preparation record keeps old and new inner IDs and stops for review if they
differ. The plan also retains supplied linked-build/dependency evidence and its
hash, separately from the CLI executable hash and historical artifacts. These
changes close the preparation design findings by source inspection; the actual
evidence file still needs review when the binaries/artifacts are built.

The source is suitable for the bounded preparation step. Executed preparation,
row evidence and any resulting scientific conclusions remain pending; this
review makes no convergence or default-lattice recommendation.

## Executed preparation audit

The six-artifact preparation was subsequently executed and independently checked
before any campaign row was started. All 12 bounded current-CLI generation and
inspection processes exited successfully. Their complete output records agree
with the separately saved preparation records and the exact artifact bytes.
The inspector's content ID and provenance match each frozen outer artifact;
source provenance comes from the freshly executed generation, not a claim that
inspection rereads the input files.

The frozen CLI SHA-256 is
`b9bcec8797df82f1a124dc607c2f6a33eb064205257bd80b36ebb07321860851`.
The independently hashed optimized caller is
`e304fb622eff36d1d4c78ba503765fb1693e7ca89721493d662289984840248e`.
Its recorded direct FastSecDec rlib has SHA-256
`c29ca3bfa77102d858fa63ef5f5ef2ae2a9bd4ccbb72d7601ca07a150ca66901`,
identical to the actual current CLI build's linked core. Direct native rlib and
helper-source hashes were checked, as were the complete inherited dependency
records, current CLI binary, preparation's copied build evidence, artifact
hashes and frozen references. The plan's current orchestration repository
revision is distinct from the inherited binary's archived source snapshot;
the linked-build evidence preserves both rather than relabelling a build.

| Input | Kernels | Full dimension |
| --- | ---: | ---: |
| Massive kite, two loops | 4 | 4 |
| Massive self-energy, three loops | 60 | 6 |
| Massive three-point, two loops | 6 | 4 |
| Massive three-point, two loops, six lines | 6 | 5 |
| Massive three-point, three loops | 60 | 6 |
| Massive three-point, three loops, eight lines | 117 | 7 |

Every current inner kernel ID equals its historical counterpart. All full
native dimensions satisfy the explicit HKKN bound; no dimension truncation or
rule fallback is needed. The new outer identities retain current dependency
states. The copied reference bytes exactly equal the six committed reference
fixtures, including their positive `StandardError` values and qualified
implementation/input-validation labels. Raw model/DOT source fingerprints were
recomputed; native canonical run-card fingerprints are retained from the fresh
production generator. This audit does not implement a second TOML identity or
portable-artifact validator.

The frozen plan contains exactly the Cartesian product of six cases, two rules,
two point counts and three prescribed seeds, with unique IDs and contiguous
row indices. Every row retains 16 shifts, one worker and 1,024-point packages.
Its count sum is exactly **223,838,208 kernel-point evaluations in 72 rows**.
No row output existed at review time. The bounded Stage A launch is accepted;
row outcomes, convergence conclusions and any default-lattice decision remain
pending and require separate evidence.

Concrete evidence is retained in
`output/diagnostics/massive-holdout-stage-a/plan.json`, the six preparation files,
`independent-preparation-review.json`, and
`output/diagnostics/massive-holdout-independent-hashes/invocation.json`.
The latter independently recomputes the frozen BLAKE3 identities with a native
hashing utility; no additional Symbolica process or replacement estimator was
used for this audit.

## Independent completed-row audit

All **72** prescribed processes subsequently completed without timeout, missing
report, skipped row or nonzero exit. The independent audit matched every row's
specification to the frozen plan, then inspected its complete native checkpoint
and observation. Every sector has sixteen complete replicas; all **218,592**
canonical packages have contiguous nonoverlapping 1,024-point coverage, matching
processed counts and no failed partial. Total accepted work is exactly
**223,838,208 evaluations**. Snapshot and contribution totals agree, and all
marginal coverage, exact offsets, layouts, allocations and accepted replay
identities remain bound to the correct kernel and policy.

The native checkpoints also establish the correlation claim directly: for each
case/count/seed, the actual stored shift vectors are identical across the two
rules, and across all same-dimensional sectors. This was checked from the stored
bit representations, not inferred merely from equal seed numbers. Each of the
twelve joint summaries contains the exact concatenated row vectors for all 48
`(seed, shift)` keys, in the declared Kuo/HKKN component order. There are no
excluded inputs and no pooling across lattice counts. Source inspection confirms
that the runner passes those vectors directly to native
`QmcEstimate::from_shift_means`; the audit did not implement another covariance
estimator. Reported covariance symmetry, diagonal/error consistency, finiteness
and nonnegative two-component principal determinant were also checked.

Every per-row comparison retains the original numerical reference, its positive
standard error, its provenance and its explicit normalization/kinematics/
independence context. All comparisons are available and eligible; none crosses
the frozen five-standard-error investigation threshold. The largest recorded
absolute native pull is **3.8666270369**, at row 9. This is the recorded native
comparison, not an independently invented aggregate significance statistic.

The aggregate uncertainty table in `massive-holdout-stage-a.md` matches the
retained native summaries: HKKN has lower 48-shift standard error in all six
8,192-point cases, but higher error in four of six 1,024-point cases. Both rules
still have appreciable uncertainty in the seven-dimensional eight-line case.
No default recommendation or convergence-rate proof follows from this finite
block. The original external reference uncertainties remain part of the
comparison; neither their exact values nor their missing off-diagonal covariance
has been manufactured.

The diagnostic counts independently sum to **7,056** weighted checks, conditioning
checks, rescues and additional replays, maximum 256 bits, with zero failures.
Process wall times sum to 58.709499 seconds and have a 3.466979-second maximum,
consistent with the author's diagnostic timing record. These runs retain the
50 ms polling wrapper and are not matched end-to-end performance acceptance.
No statistical, coverage or provenance discrepancy was found.

The compact audit result is
`output/diagnostics/massive-holdout-stage-a/independent-completion-review.json`.
It records the checked coverage, all twelve uncertainty ratios, maximum pull,
precision totals and matched actual shift-vector evidence. This completion audit
used retained data and native source inspection; no symbolic evaluation or new
integration run was launched.
