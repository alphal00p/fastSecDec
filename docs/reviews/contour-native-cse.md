# Native common-subexpression elimination in contour generation

2026-10-10. The 1000 GeV D05 campaign exposed a generation bottleneck in the
existing Symbolica evaluator optimizer. This review records the owner-library
investigation; an improvement in the small reproduction is not itself a
measurement of physical generation speed.

## Reuse and diagnosis

The public API, owner source/tests and executable reproduction were checked
before changing an algebra operation. FastSecDec continues to use Symbolica's
`EvaluatorComposer`, `Dualizer` and native evaluator representation. No alternate
common-subexpression elimination or evaluator codec is introduced.

A five-second, 49 Hz user-cycle profile of a real polynomial-dynamic D05 worker
collected 243 samples. Approximately 61.4% were attributed directly to native
instruction-list construction under `EvaluatorComposer::finish`; allocation and
hash-table operations account for much of the remaining work. This is a short
diagnostic sample, not a whole-run attribution. The callback root solver and
SymJIT translation were not the active bottleneck in that sample.

The composer always performs common-subexpression elimination before bounded
common-pair optimization. Setting the latter's round limit to zero does not
remove the former's repeated passes. Abusing the optimizer's abort polling to
skip those passes would change its intended API contract and was rejected.

The owner pass formed its lookup keys from old operand positions and renamed
operands only after the lookup. Consequently, a chain could discover duplicates
of its first layer in one pass, its second layer in the next, and so on. A public
Composer reproduction with four identical sine chains and common-pair rounds
set to zero took 1.29, 4.76, 18.77 and 74.24 ms at depths 128, 256, 512 and 1024.
Its optimized outputs were numerically correct; the issue was avoidable work.

## Narrow owner correction

The owner correction applies the existing operand remapping before
constructing the existing borrowed lookup key. Each disjoint instruction keeps
stable backing storage throughout the pass. Retained instructions are cloned
once; ordered callback arguments, canonical Add/Mul ordering and branch-ancestry
rules are preserved.

An extracted method-body control reduces four depth-1024 chains in one pass
instead of 1024 passes, retaining the same 1024 instructions. That shim uses
the standard hash map in place of the owner's `ahash` map; it demonstrates
the deterministic pass count, not a native-owner timing comparison.

Independent generation-agent review found no semantic blocker. Three new
deterministic owner controls cover dependent duplicate chains, all supported
lookup-key kinds and argument ordering, and parent/sibling/post-join branch
reuse. They check evaluation equivalence as well as the single-pass property.
The actual upstream owner's focused evaluation unit gate passes 47 tests, and
its public evaluation gate passes 15 tests with one existing stress control
ignored. [Symbolica PR #63](https://github.com/symbolica-dev/symbolica/pull/63)
contains only this correction and its three controls, at commit
`d953467dc089d620c4d4602146725ad8ec5a845b` atop upstream `community` `f4e7870`.

The clean combined consumer `74225696cd445247fa81c499c5110decd19257ed` adds the
same patch to the preceding `650d942` consumer. Its 36 public evaluation,
direct-cache, direct-vector and callback tests pass (one existing stress
control ignored). The public Composer depth-1024 probe takes 0.844 ms with
the corrected owner, retaining the same outputs and operation count. Different
library builds were used for this diagnostic comparison. The complete consumer
workspace passes 917 tests (33 existing diagnostics ignored), the portable
consumer passes 82 tests, and the thin Python binding passes all-target and
stub-generation checks. Strict workspace Clippy and formatting also pass.
Physical generation measurements are recorded in
[the D05 generation audit](contour-gghh-1000-generation.md).

Both commits and publication use `ValentinHirschi
<valentin.hirschi@gmail.com>`. GitHub denied formal `benruijl` reviewer assignment;
the authorized [review invitation](https://github.com/symbolica-dev/symbolica/pull/63#issuecomment-6095912476)
is recorded as a comment, not a formal assignment or merge. The PR is attached
to the task. All three FastSecDec manifests and lockfiles select the new public
owner, and locked metadata confirms unique Symbolica/Numerica identities with
unchanged SymJIT and other dependencies.

## Artifact compatibility

The CLI admits artifacts only when the complete dependency identity matches.
Generation staging additionally checks its build and settings identity. The
campaign therefore preserves the old artifacts and incomplete journals and
generates fresh artifacts with a coherent new consumer build. It does not
rewrite journal identities or bypass compatibility checks to reuse old work.

## Serial residency check on the physical workload

The dynamic run uses `generate --serial --workers 4`. The preparation barrier
spools mapped records before formula deduplication and compilation; it does not
retain every mapped expression in the coordinator. At the recorded live check,
29 completed mapped records occupied 38,896,576,348 bytes on disk while the
coordinator used approximately 10.5 MB RSS. After all 30 were mapped, the
coordinator used approximately 11.2 MB and the four new compilation processes
used approximately 3.0–3.3 GB each.

Independent source review confirms that the coordinator holds compact
`PreparedChartSource`, `DiscoveredSector`, formula and compiled-record receipts.
Each scientific child loads one sector, compiles and persists it, and is reaped
before its slot is reused. Native source/template caches have that sector's
process lifetime. The generation runner does not queue completed evaluators.

Single-sector peaks still include native Atom-table copies, payload/envelope
encoding buffers and temporary overlap between a generated unit and its
compiled kernel. These are per-active-worker costs, not a cache growing with
the number of completed sectors. Streaming those codec buffers may be a later
local optimization; no whole-family retention bug was found in this audit.
