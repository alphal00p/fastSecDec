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
