# Original-expression native dual oracle results

Updated 2026-10-05. All three prescribed exact points have independently accepted
complete numerical oracles for the captured original on-shell representative.
The subsequent [cold candidate gate](native-dual-original-oracle-independent.md#complete-cold-candidate-comparisons)
also passes for this one captured Taylor representative, without multiplicity:
21 full-order comparisons and 24 weighted vectors comprising 168 components.
Production adoption, whole-graph integration and matched performance remain
open. The original generation route remains the
production default. See the [adapter protocol](native-dual-original-adapter.md)
for the exact admission and record contract.

## Original source and mathematical coverage

All actual attempts read only the original capture in
`output/diagnostics/production-alias-capture-81/capture` and the completed Taylor
identity in `output/diagnostics/native-ibp-prepare-1/prepared/progress.json`.
The proof has 872 Taylor pieces and `Taylor_native_expression_identity: true`;
its enclosing record is incomplete because a later IBP stage failed. No IBP
expression, candidate coefficients, candidate leading order or interpolation
enters this oracle.

The input is representative 80, source chart 119 of the massless on-shell
triple box, with nine coordinates, 2,112 geometry charts and 1,026
representatives. Multiplicity four is recorded and is **not applied**. Identity
anchors are:

| Identity | BLAKE3 |
| --- | --- |
| Canonical density | `2950af91e8efe30160db08945a72c5131332bba4569269107e8313f1facd6ed7` |
| Original expression | `bd70f67410f7a36bf517da47eb5c7e9ee4b0927ca85f7d2b1c193c4c25fa40d6` |

The source construction gives a conservative pole bound of six from regulated
singular axes 1, 2, 3, 5, 7 and 8; the common `Gamma(4+3*eps)` is regular at zero.
At each exact point the adapter uses native factor collection and `eps^6`
scaling, admits every intermediate inverse/power/log/Gamma domain, and verifies
native monomial reversal. Native `Dualizer`/`HyperDual` components with shape
`[[0],[1],[2],[3],[4],[5],[6]]` are already Taylor-normalized and supply the
complete signed layout `[-6,-5,-4,-3,-2,-1,0]`. There is no manual derivative,
factorial, convolution or Laurent-algebra implementation.

This certificate is source pole bound plus exact analyticity admission plus
native Taylor shape. It is not a native `Series::absolute_order` certificate;
the distinct `native-original-taylor-dual-oracle` format never emits that field.
Independent 512/1024-bit evaluator mappings check all real/imaginary components
with the existing scaled `1e-70` rule. Approximate agreement does not certify
symbolic equality or exact vanishing.

## Preserved failures and precision correction

The unchanged native-Series point-zero attempts remain separately retained at
`original-taylor-point-0-attempt-{1,2}` under `output/diagnostics/`. They timed
out at 180.167230052 and 600.315441 seconds, without complete coefficients.

`native-original-taylor-dual-build-1` failed at compile time with `E0514`: the
default Nix Rust 1.97.1 could not consume the frozen Rust 1.98.1 native rlibs.
Subsequent builds explicitly select the existing Rust 1.98.1 compiler inside
the Nix native-tool environment. The failed build retains its earlier shared
review's count typo (19 instead of 21); later builds bind the corrected review,
whose scientific acceptance fields were unchanged.

Build two's `original-taylor-dual-point-0-attempt-1` completed exact native
operations and all seven precision/realness checks, then failed during export
because the adapter incorrectly asserted actual Float precision `[512,512]`.
Native arithmetic returned `[505,511]`. The process exited 1, was reaped after
28.704886240 seconds, did not time out, and retained incomplete progress and
unchanged frozen inputs. It is not an accepted complete oracle.

Numerica deliberately tracks result precision dynamically. Cancellation can
lower it; native square root can raise it. The correction records requested
evaluator bits separately from actual positive component bits, without resetting,
padding or bounding the latter by the former. Coefficient transport now uses
native `Float::as_raw().to_string_radix(10, None)`; native `Display` can omit
digits and remains diagnostic only. The reader parses at each actual component
precision and checks exact equality against the native exported value and
precision. Evaluator mathematics, source bound, tolerances and shared core did
not change.

The exact corrected `io.rs` passed `native-float-export-control-1`: six cases
cover cancellation, square-root precision growth and numeric zero at both
requested precisions, with exact decimal and native Atom round trips. The
independent review accepted all 19 frozen postchecks; the process exited 0 in
0.009625888 seconds. Cancellation produced actual precision 483/995 bits, and
square root 513/1025 bits. The separate additive reader controls and their
metadata rejection checks are recorded in
[native-dual-reader-admission.md](native-dual-reader-admission.md).

Every coefficient export is labeled `native-mpfr-approximation` with
`symbolic_zero_certified: false`. Nonzero Float exports preserve actual values
and both component precisions. Native `Atom::num` canonicalizes numeric zero to
`Atom::Zero`; that storage has null stored precision and supplies no exact-zero
certificate for the original expression.

## Corrected frozen build

All corrected actual attempts use the same
`output/diagnostics/native-original-taylor-dual-build-3`. Its 32 frozen entries
and all 474 available-link-input postchecks pass; the latter inventory is a
superset and does not claim that every listed rlib was linked. Source archives,
compiler identity, native feature identity, accepted controls and executable
are frozen. The selected Symbolica 3.0.1 with five local patches and SymJIT
2.26.4 reuse the same native identity as the controls.

| Item | SHA-256 |
| --- | --- |
| Main source | `00ae9f58e827ad2d88c34b6a6e5c0edaeaba9a597d5233d7936341b8080eaf04` |
| Shared native core | `4be1d53b69ae54c28034bcc4a84680c9056ab63f3092a082985a77e4b6969c5f` |
| Accepted transport source | `3251cab5482b8e962a38fefd0ee11b4bbdbd2278cd19ccab4da6626f9f6ffb52` |
| Symbolica rlib | `01601b6df1747703693fdbf78818f9bf2bb74f4e4b1a888a5971b678cc60b61a` |
| Executable | `557079a5f7fcab50005736d1cbecbbcf0dd6daf205419cc64732d4c9b4421e89` |
| Build frozen manifest | `91a767e3ab8668eb3b291ffbb018b1c2d87b4ce2bb67518109e732f28907169f` |

Each actual point has a fresh directory, separate 78-entry immutable manifest,
independent preflight, and coordinator-owned runtime handoff. Bounds are
180 seconds plus five seconds termination grace, 30 GiB address space and CPU8.
The caller's process-group timer reaps the process; the address-space cap is
not an RSS guarantee. No point or candidate reader starts automatically after
another point.

## Actual point outcomes

Paths in this section are relative to `output/diagnostics/`.

| Point | Exact coordinates | Attempt | Accepted outcome |
| --- | --- | --- | --- |
| 0 | `[1/2,1/3,1/4,1/5,1/6,1/7,1/8,1/9,1/10]` | `original-taylor-dual-point-0-attempt-2` | Complete oracle; exit 0, 28.316419658 s, 589,832 KiB peak RSS |
| 1 | `[1/100000000000000000000,37/100,37/100,37/100,37/100,37/100,37/100,37/100,37/100]` | `original-taylor-dual-point-1-attempt-1` | Complete oracle; exit 0, 32.789803612 s, 610,396 KiB peak RSS |
| 2 | `[1/100000000,73/100,1/100000000,73/100,1/100000000,73/100,1/100000000,73/100,1/100000000]` | `original-taylor-dual-point-2-attempt-1` | Complete oracle; exit 0, 31.645152999 s, 598,164 KiB peak RSS |

Point zero's independent review verifies all seven signed orders, all precision
and realness checks, 14 approximate native coefficient exports, all original
source/proof and 17 artifact digests, and all 32 build plus 78 attempt frozen
checks. No timeout occurred. The 137,835,408-byte original Atom became
22,686,159 bytes after exact coordinate binding, 20,397,948 bytes after native
factor collection and 20,397,941 bytes after regularization.

Point zero's actual `[real, imaginary]` component precision is preserved:

| Orders | Requested 512 | Requested 1024 |
| --- | --- | --- |
| -6, -5, -4, -3 | `[505,511]` | `[1017,1023]` |
| -2 | `[501,511]` | `[1013,1023]` |
| -1 | `[504,511]` | `[1016,1023]` |
| 0 | `[502,511]` | `[1014,1023]` |

The numerical values and native exports are retained in `point/result.json`
and its bound artifact files. No rounding in this document serves as reader
input. Point-zero result BLAKE3 is
`d448310303c4a15a5d8915e583e2237ba18b21f3fe9977a7b5ba365802ab0b36`;
its independent-review SHA-256 is
`858a405d80028fb1fc9408fd9cd2917113dc4164614f88ed5e588318ff3b47f3`.

Point one's independent review accepts the same full seven-order layout,
14 native exports, all original/proof and 17 artifact digests, and 32 plus 78
frozen checks. Its actual real precision reaches 455/967 bits at order -5;
all original two-precision and realness criteria still pass without changing
the tolerance. The result BLAKE3 is
`1c948f789b73161f8e0ccd00b52a4eda296ee65201cfc617612a933ae38e3988`
and independent-review SHA-256 is
`e387547646b42a3cbd5fc3e4ed0c39663ac362c26be7779c6852b008564d167b`.

Point two independently passes the same seven-order, 14-export, 17-artifact,
source/proof and 32-plus-78 frozen checks, without timeout. Its minimum actual
real precision is 479/991 bits. Result BLAKE3 is
`47afbfe4b1f7e4ce2062c4271d096d8bc020c028dbaf1f6c64c495a262513548`
and independent-review SHA-256 is
`aad6d83b509e190c933914274e84a44f21335fe7c9b590cf2a89961023dbb531`.
All three complete numerical vectors are accepted as original-expression
oracles; none supplies a symbolic-zero certificate. Their comparison with the
candidate is established separately by the linked cold-reader audit.

For reproducible later handoffs, point one's frozen-manifest SHA-256 is
`f32a790f59be4ca5ac0fcd9f83cd1e327e3dd34b3fc0783de65d96018398e632`
and point two's is
`8037b86a4de7638709cc5d9c8d17ee991b1c4d22d00df3b8b624a650a1f466ba`.
Their preparation checks and independent preflights pass without any source,
compiler, dependency or mathematical change.

Point zero's exclusive phase timings distinguish the work: exact coordinate
binding 9.788394487 s, native import/source admission 6.114403082 s, factor
collection 2.023910663 s, exact operation admission 0.326369203 s, monomial
reversal 3.377994700 s, evaluator construction 0.039912266 s, vectorization
0.793709229 s, 512-bit evaluation 0.701985511 s and 1024-bit evaluation
2.419632280 s. Input/build hashing, exports and postflight are separate fields;
the process wall time also includes startup and teardown. These bounded
single-run development observations do not establish matched performance.

The three complete source-bound original oracles satisfy the independent-input
prerequisite of the accepted cold candidate gate. Its separate audit covers
full-order comparisons, cold native reload, weighted evaluation, worker clone
and forced high-precision replay under the reviewed reader contract. This
acceptance is confined to the captured representative; public production
integration, full-graph completion and wider integral/performance gates remain
open.
