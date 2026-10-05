# Native alias production gates

The reviewed production slice retains the existing post-subtraction coordinate
images in native `AliasedAtom` coefficients and builds one native exact evaluator
for O2, tracking, and MPFR. It introduces no callback registry or alternate
algebra. Whole-graph acceptance remains open. These timings are bounded
correctness diagnostics, not matched performance measurements.

## Focused executable checks

The first production gate passed 103 tests, with nine explicitly ignored tests,
in `output/native-alias-scientific-tests.log`. It covers generation, native Gamma
and negative Laurent orders, compact factors, complex kernels, weighted replay,
independent symmetry/scientific controls, metadata, and cold-process artifacts.
The subsequent artifact target passed all six tests in
`output/native-alias-artifacts-six-tests.log`, including native structurally
malformed IR rejection and rejection of a fixed complex Gamma constant under a
re-signed real output layout. These six overlap the initial four artifact tests;
they are not six additional distinct tests. The legacy CLI inspection test also
passed in the artifact author's separate gate.

The native decoder owner's four structural-validation tests passed. All six
preserved malformed native inputs were rejected through serde, Decode, and
BorrowDecode against the fixed normal library, without evaluating malformed IR.
The implementation and limits are recorded in
`../dependency-patches/symbolica-evaluator-ir-validation.md`.

The subsequent complete workspace gate passed **294 tests, 18 explicitly
ignored**, in `output/native-alias-workspace-tests.log`. The final affected
two-test alias and six-test artifact targets also passed after test-only lint
corrections. Formatting, all-target Clippy, and the production dependency-tree
audit passed. The machine-readable final build is retained in
`output/native-alias-workspace-artifacts.jsonl`.

## Captured representative: failures and completed gate

Each attempt uses the unchanged original 241-piece subtraction expression for
representative index 38, original chart 46, with nine coordinates and orders
`[-5,-4,-3,-2,-1,0]`. Original multiplicity two is recorded but not applied in
this per-representative comparison. Fresh production Laurent expansion and
native evaluator construction are required. Three independent original-expression
oracles use exact rational points at 512/1024 bits; public weighted kernel checks
use separate MPFR references at the identical rounded binary64 coordinates.

The wrapper freezes the copied test binary, native dependency, source, original
capture/images, preparation record, and all three oracle records. It checks
hashes before and after a single CPU-8 process with a 180-second deadline and
30-GiB virtual-address limit. The dependency is Symbolica 3.0.1 at revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` plus the five reviewed local fixes,
with SymJIT 2.26.4. The Rust test profile uses the existing optimized native
dependency, not an optimized whole-program benchmark binary.

| Attempt | Outcome | Wall time | Peak RSS |
| --- | --- | ---: | ---: |
| `production-alias-representative-1` | Exit 101 at the first exact root comparison; fresh and saved image handles differ | 43.192 s | 533,248 KiB |
| `production-alias-representative-2` | Exit 101 at order −4 root identity after an exact image bijection; order −5 identity passed | 45.060 s | 532,224 KiB |
| `production-alias-representative-3` | Exit 0; exact native series-input identity and all independent numerical/production-kernel checks pass | 78.551 s | 532,220 KiB |

The first two completed native expansion, neither timed out, and both pre/post
manifests passed. Neither reached the numerical oracle or public-kernel checks. Their
stdout, stderr, timer records and copied inputs remain under the respective
`output/diagnostics/` directories.

The second test-only comparison proves a complete 201-body bijection, unique
fresh and saved handles, and exact native image-body equality before a native
simultaneous literal handle replacement. Production coefficients are untouched.
The remaining factored root mismatch is a representation-identity failure; it
does not by itself establish mathematical inequality or scientific agreement.

The third diagnostic exposes the actual pre-series template from the production
cache under `cfg(test)`. It requires exact template identity after the same
proven image bijection, retains each coefficient's structural-identity result,
and exports every fresh native root. It then requires all 18 independent
high-precision oracle comparisons and all fresh/cold weighted production checks.
No large expression is expanded just to force a common factorization. This
revision was independently reviewed before execution.

Attempt three passed the exact 201-image bijection and native pre-series input
identity. All **18 coefficient/point oracle comparisons** passed at 512/1024
bits. At each point, the fresh and decoded kernels were also checked at both
weights 1 and 10^40 against MPFR evaluated at the identical rounded binary64
coordinates: **12 complete vector evaluations, 72 components**, passed. All
requested precision checking and rescue. The accepted second precision was
256 bits at points zero and one, and 512 bits at point two. This tests the
production adaptive precision path, not merely eager/O2 agreement.
The representative decodes its exact IR in the same process. Separate
`artifact_process` tests cover reconstruction and weighted replay in a fresh
process; the representative is not itself a fresh-process load test.

Fresh production Laurent expansion took 45.141 seconds. Program construction,
native exact-IR encoding/decoding and both backend constructions took 6.333
seconds. The encoded native program is **1,497,695 bytes**. These are diagnostic
substage observations from the test binary, not a speed comparison. All 529
pre/post hashes passed. The complete result, exported roots and structural
flags are retained under
`output/diagnostics/production-alias-representative-3/result/`.

After exact handle renaming, the order −5 root is structurally identical and the
other five are not. The successful gate therefore establishes exact native
series-input identity, full-vector agreement at the three independently derived
points, and the tested fresh/cold precision behavior. It does **not** claim
all-six structural coefficient identity or use numerical sampling as a general
symbolic-equivalence proof.

## Acceptance limits

Fresh and cold artifact paths intentionally differ in their proven zero and
real-component facts: cold native IR initially uses conservative false facts.
Weighted zero padding can therefore trigger additional MPFR checks after load.
Worker clones share immutable encoded programs via `Arc<[u8]>`; artifact JSON
envelope size and full-graph memory still need measurement.

The new complete on-shell graph trial remains a separate gate. Its approved
predeclared limits are 1,800 seconds/30 GiB for generation, followed only on
success by bounded inspection and a complete all-kernel 1,024×8 integration
allocation. The original graph/card and all Laurent orders remain unchanged.
A successful representative cannot establish whole-graph coverage, integration
convergence, or sample throughput.

## Original full-graph release trial: bounded failure

`production-alias-fullgraph-1` used the release binary built from committed
`6332676a215f32d1fccf840175a433dc9dbc77d2`, SHA-256
`dbc0dbbb8e5c7ca83e273f8af3590b92fbef2e50d561d4759ae9689b1cd00e7e`.
The original on-shell card, DOT, model and parameter card were copied unchanged.
Committed source and actual patched native dependencies were archived and bound
to an immutable 26-file manifest, so subsequent live-source edits did not alter
the measured binary's provenance. All pre/post checks passed.

Generation hit its predeclared **1,800-second deadline** during displayed
representative **81 of 1,026** (zero-based index 80). SIGINT was sent at
1,800.003 seconds; the original five-second grace ended with SIGKILL at
1,805.003 seconds. The child was reaped with signal 9 and the wrapper returned
124, at **1,805.689 seconds** wall time. Its process group is empty. There is
**no kernel artifact, inspection, integration, checkpoint or saved result**.
No failed or partial vector is promoted into a numerical result.

The original geometry produced 2,112 charts. Eighty Laurent calls completed;
the slowest completed call was ordinal 80, with 504 endpoint terms and
539.1885 seconds in the reported Laurent interval. Ordinal 81 has 872 terms and
started its expansion at elapsed 806.720 seconds. The interval includes template
preparation/cache/native series/wrapping, so it does not isolate native series
arithmetic. The terminal completed-phase totals were geometry 1.977 seconds,
mapping 1.928, symmetry 1.625, subtraction 29.974, and Laurent 771.007; the active
unfinished call is not included in that last cumulative total.

Native `wait4` retained peak RSS **7,085,428 KiB (6.76 GiB)**, user CPU 579.939
seconds, system CPU 1,204.727 seconds, 516,802,223 minor faults, 6,081 major
faults, and 236 voluntary/8,900 involuntary context switches. The unusual system
CPU share is evidence to investigate, not proof of swapping or of a particular
symbolic operation. The imposed 30-GiB limit is virtual address space, not RSS.
Earlier cached templates/roots and pending generated coefficients also contribute
to resident memory. Source attribution and a separately bounded targeted capture
are recorded in `native-series-fullgraph-attribution.md`.

This is a capability failure within the declared bound, not a controlled speed
measurement. Guarded external C++/FORM work overlapped on other CPUs, with no
concurrent Symbolica process. Raw timer/status records, all immutable snapshots,
and the derived completed-call ledger remain under
`output/diagnostics/production-alias-fullgraph-1/`. Whole-original-graph
acceptance therefore remains open despite the earlier representative success.

The subsequent targeted capture and existing-IBP preparation do not close that
gate. Capture81 exported the exact original mapped/Taylor inputs successfully;
`native-ibp-prepare-1` then passed an exact 872-piece Taylor replay but timed out
after 180.483 seconds during existing native IBP subtraction. No IBP expression,
Laurent vector, evaluator or integration was produced. The full retained outcome,
unchanged bounds and development/test profile limitation are recorded in
`native-triplebox-ibp-proposal.md`; no default strategy changed.
