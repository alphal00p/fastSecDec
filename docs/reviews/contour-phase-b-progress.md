# Phase B acceptance ledger

Updated 2026-10-09. The authoritative requirements are in
[CONTOUR_DEFORMATION_PLAN.md](../../CONTOUR_DEFORMATION_PLAN.md). This phase is
active; passing the fixed-mode controls does not complete dynamic deformation
or the required physical multiloop tests.

| Area | Evidence and remaining work |
| --- | --- |
| Fixed map and subtraction | Symbolica-derived map, Jacobian, endpoint ratios and factorwise causal logarithms; symbolic and numerical-dual Taylor/IBP controls pass. |
| Fixed runtime | Runtime binding, eager/SymJIT and precision rescue, scoped preflight, `always`/`pilot`/`off`, selective reload and checkpoint provenance pass native tests. |
| Caller-owned execution | Real CLI tests cover ordinary/serial generation and integration, all validation policies, cancellation during preflight, exact offsets and policy-only resume. |
| Independent physical references | Above-threshold B0, C0 and D0 agree with native HEPKit/OneLOop in both generation modes. Full fixed-mode 400 GeV ggHH agrees with HEPKit and MadLoop at 0.39 standard errors, with relative uncertainty `2.96355e-4`, pole cancellations and Ward checks. Physical double box and required LTD cases remain pending. |
| Fixed milestone native regression suite | At `f2c2d930`, **730 distinct enabled tests passed; 29 were ignored**: 245 core scientific/integration tests, 252 library tests and 233 CLI/QMC/sector tests. Results for the subsequent dynamic generation increment are recorded separately below. Ignored tests are not acceptance evidence. |
| Strict lint gate | Final public-matrix `cargo clippy --workspace --all-targets --locked -- -D warnings` passes, including the readiness correction and native/CLI artifact foundations. |
| Dynamic algebra | Polynomial and sign-aware recipe generation apply the full local-strength map before subtraction. Analytic complex Laurent controls now execute both symbolic and actual native numerical-dual evaluators, including repeated endpoint poles and Taylor/IBP. Certified dynamic runtime admission remains pending. |
| Dynamic numerical root | Native callback/implicit-jet and restoration foundations pass. Owner tests cover prepared eager/JIT solves, tracked hypot, certified square roots and portable domains. An independent enclosure probe passed 2,620 cases at 8/24/96/256 bits. Production checker integration remains pending. |
| Recipe-addressable artifacts | Ordinary and serial generation now publish the same native v2 archive. Internal family orchestration shares source preparation and retains recipe-local symmetry/formulas, with actual-process recovery tests. Public family selection and dynamic admission remain pending; ordinary/serial integration, covariance, selective inspection and checkpoint isolation pass for undeformed/fixed programs. |
| Portable execution | The full maintained standalone portable suite passes **73 tests, zero failures or ignored**, with ordinary `--locked` Cargo on the final public source matrix. This includes fixed threshold bubbles in both generation modes, all complex Laurent components, certified pilot/production validation, fresh-process restoration, covariance and checkpoint/replay controls. Native portable execution is not an actual browser/WASM run. |
| HEPKit/Python | The refreshed full-default private Community wheel built from fixed milestone `f2c2d930` passes **8 contour tests, 210 maintained binding/demo/notebook/wavefunction tests, and 6 owner compatibility controls**. The independent standalone binding check also passes for the subsequent generation selector increment. The wheel does not contain that dynamic increment; latest Community `9a65` and actual WASM execution remain separate gates. |
| Variance and performance | [Matched-work protocol](contour-variance-protocol.md) and fixed/fixed executable control verify matching actual coordinates/weights, separate result identities and complete covariance. No dynamic production variance gain has been measured or claimed. |

The sign-aware generation milestone's publicly reproducible source matrix is Symbolica/Numerica
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, SymJIT
`33100ae869057f35d9865c933a48bac6699acdd4`, and Feynkit
`8e3a643f388b45939d6573a648ef3a509086835e`. All three maintained manifests
and lockfiles select public sources; no local file URL or build-directory
override is needed. The Symbolica revision retains existing evaluator
composition as well as the new owner fixes. SymJIT's dependency requirement is
a compatible minimum (`2.27.0`); the temporary public Git patch selects the
reviewable unreleased fixes.

A subsequent focused dependency increment updates SymJIT alone to public
`d74993ffd76a6fc322a7bcf3963fa786783a38a8`. It fixes a reproduced 256-byte native
callback-name limit through [upstream PR #16](https://github.com/siravan/symjit/pull/16).
The owner suite passes 2,151 tests (one ignored), and FastSecDec's rebuilt
contour filter passes 77 tests. The separate higher-jet observer control
identified incorrectly grouped subtraction-face requests; its correction and
complete dynamic runtime admission remain under validation. A settings regression also yielded
[Serde PR #3109](https://github.com/serde-rs/serde/pull/3109), without adding a
Serde fork to FastSecDec. The upstream ledger records exact identities and
review-request outcomes for both fixes.

The [upstream PR ledger](contour-upstream-prs.md) records the owner patches,
their focused tests, authorship and actual review-request outcomes. GitHub
accepted some formal `benruijl` requests and rejected others because he is not
a collaborator; the latter have explicit authorized review invitations in
comments. These PRs are not claimed as merged. Current Community's new required
citation URL field also has small owner-specific compatibility PRs; the
host-specific Vakint change preserves its existing RustRed integration.

The refreshed installed Python evidence uses the immutable `f2c2d930` archive,
SHA256 `355538baccb107fc81f247132f2e4001cd7cbefe63ee9dab74cad9389f7299fc`.
Its wheel SHA256 is
`06c602e256e8d6eb9c690756ba749c812c24692c6f3325ff0402b3a5e7776202`.
The two exact-only QMC test fixtures explicitly select the existing
`hkkn_alpha3` lattice for their 32-point requests; this test-only overlay does
not modify the wheel's compiled source. No shared notebook installation was
replaced. The [Python review](contour-python.md) records the full owner matrix,
test and generated-stub evidence, and remaining current-host/browser gates.

The required LTD two-loop and three-loop native fixtures now have reproducible
HEPKit graphs, source provenance and four passing importer controls, including
native U/F and normalization checks. All four CLI input inspections pass.
Their contour generation/integration gates remain pending. The six-point
example has a strictly negative interior F and is a branch control, not
evidence of crossing an interior threshold surface. See the
[fixture audit](contour-ltd-fixtures.md).

A subsequent bounded fixed-mode `2L4P.b.K1` probe completes generation of all
186 charts using four workers, including intentional cancellation and resume,
with peak aggregate RSS below 269 MB. Two integration budgets complete without
failed evaluations but remain far too uncertain for the analytic accuracy
gate. A larger fixed strength is rejected by the residual-U pilot guard.
See the [measured probe](contour-ltd-fixed-probe.md); no dynamic variance or
release speedup is claimed.

The initial parallel test invocation exposed a native model-symbol registration
collision in the existing auxiliary-momenta test binary. The documented
single-test-thread workspace command passed that test and the complete suite.
An artifact immutability test also still expected the previous v8 writer
header; it now expects v9, while explicit legacy reader fixtures remain intact.

The final public-matrix workspace run completed the core suites, then stopped
on a CLI test's old diagnostic string: the new program-archive reader correctly
rejected an expression-only payload but reported its own unsupported-header
message. The assertion was updated without weakening rejection or metadata-only
inspection. The complete CLI/QMC/sector gate then passed. Separately, the final
library gate includes the independently reviewed exact-only readiness fix;
the original 251-test library result is replaced by that updated 252-test
result in the count above, not counted twice. A Clippy range-pattern warning
was corrected and the complete strict lint gate rerun successfully.

The final native acceptance commands were `cargo test --workspace --locked --
--test-threads=1` for the completed core integration suites, then
`cargo test -p fastsecdec --lib --locked -- --test-threads=1` and
`cargo test -p fastsecdec-cli -p fastsecdec-qmc -p fastsecdec-sectors --locked --
--test-threads=1` for the corrected targets. The first command's overall exit
was a failure for the superseded CLI assertion; only its completed successful
core suites supply acceptance evidence.

The refreshed private installed-wheel gate deliberately preserves the complete
Community `3aa2608` module set used by the earlier comparison, with corrected
public owner dependencies. Current Community `9a65` has added IBP dispatch,
WASM RustRed registration and positive-epsilon master dependencies; validating
that newer complete host with its corresponding owner pins is an additional
required ecosystem gate before Phase B completion. Neither snapshot is a claim
that the shared notebook environment has been upgraded.

Variance comparisons will preserve complete complex Laurent covariance and
distinguish QMC replica variance from pointwise integrand variance. They must
use matching actual coordinate sequences, independent repeated runs, and
frozen pilot-selected settings. Report variance gain alongside sampling cost,
variance times runtime, and time to one-per-mil accuracy; larger deformation
alone is not evidence of better convergence.

## Dynamic generation and helper ownership milestone

Explicit native recipe selection and polynomial dynamic maps now run before
subtraction in both generation modes. The analytic Laurent controls pass for
Taylor and IBP, and streamed staging restores native helper owners before its
Atom context. Public physical source identities use Symbolica canonical
expressions instead of process-local serialization state. Recipe-aware receipts
reject foreign source identities, repeated permutations and incorrect chart
coverage before accepting completed workers. See the
[generation recovery review](contour-generation-recovery.md).

The updated core library passes **271 tests** (16 ignored), including the final
legacy-loader scope regression. The complete CLI package passes **156 tests**
(7 ignored), including real process recovery and all fixed-contour execution
combinations. The full portable consumer passes **73 tests**, with no failures
or ignored tests. The final standalone binding check, strict workspace/all-target
Clippy, formatting and whitespace checks pass. The portable execution gate
predates the final three-line legacy JSON loader scope guard; the native core
gate exercises that actual nested-load path, and the standalone binding check
also includes it. These results do not claim portable dynamic production or a
new installed dynamic Python wheel.

An independent fresh-process probe identified and verified correction of an
identity issue:
equivalent native root helpers can have different optimized instruction layouts
when helper symbols are registered in a different order. All 125 process cases
passed native exact polynomial and derivative comparisons, rational evaluations
and restoration checks. Versioned semantic callback tags now remain stable,
while saved-byte checksums still identify their exact native representations.
Descriptor-scoped callback construction prevents another loaded integral from
supplying its helper. Independent tests cover simultaneous callers, nested
scopes and unwinding, legacy helpers, and detached multiprecision remapping.
The thread-local scope guard cannot move to another thread. See the
[helper identity audit](contour-helper-identity.md).

Dynamic production admission remains explicitly closed. Saved certified
checkers, sign-aware production envelopes, shared recipe preparation, public
runtime integration and physical multiloop validation remain required.

A bounded exploratory fixed-mode `2L4P.b.K1` run using a frozen debug binary
reached the ten-minute cap while mapping its first four of 186 source charts.
Its observed aggregate parent/worker RSS peaked at 2,130,710,528 bytes; it
cancelled cleanly and published no artifact. This is neither a completed
scientific gate nor a release performance comparison. Independent native
determinant probes point to a faster existing Symbolica alias route, which must
preserve subtraction derivatives before it can replace the current path.


## Shared preparation and bounded Jacobian templates

Shared native recipe preparation now computes geometry once and persists each
complete residual source before applying recipe-specific deformation. Four
native controls pass, including symbolic/numerical-dual Taylor/IBP Laurent
parity, zero-exponent and cancelled causal declarations, opaque undeformed
dual mapping, and foreign receipt rejection. CLI worker requests support these
caller-owned units; the new recovery test exercises durable unacknowledged
receipts, changed worker counts, cross-recipe formula rejection and a truncated
source record. Public recipe-family orchestration remains a subsequent slice.

The registered contour filter passes 71 tests, including the bounded native
Jacobian-template optimization and stable smooth positive-part callbacks.
The three exact-offset controls pass, including callback failure followed by
native multiprecision recovery and preservation of a surrounding caller's
failure state. Final broad gates pass: **292 core tests** (16 ignored), **157 CLI tests**
(7 ignored across 24 executables), and **73 portable tests** (none ignored).
Strict workspace/all-target Clippy, formatting and whitespace checks also pass. The unchanged fixed production interfaces remain available; dynamic
production admission stays closed.

Independent review caught an important diagnostic-design hazard: attaching a
face tag before symbolic subtraction can prevent an exact cancellation even if
its derivative hook is zero. The next runtime slice must retain untagged
mathematics through subtraction, Laurent construction and assembly, and only
associate diagnostic requests at evaluator lowering. Matching restricted root
expressions and face sets must use native exact operations. This correction is
being implemented before any dynamic production admission.

The frozen fixed-milestone Community wheel also completes its generated-stub
gate: six files parse under Python 3.9 grammar, eight public signatures match
the installed extension, and all eight contour/provenance classes are present.
A separate current-Community overlay preserves its newer OneLoop, RustFlow,
RustRed and reduction features; that host has not yet been built. Neither the
shared notebook installation nor the user's release binary was replaced.

## Sign-aware generation and universal recipe publication

Both approved dynamic envelopes now enter the native generation pipeline. The
nonlinear cubic causal/positive-factor control verifies its reference using
Symbolica's exact partial fractions and differentiation of a causal primitive.
All fixed/polynomial/sign-aware combinations pass in symbolic and numerical-dual
generation with Taylor and IBP. The strengthened dual tests execute native
`Dualizer` programs without materializing coefficient expressions. A repeated
endpoint pole additionally exercises higher local-strength jets and the required
subtraction faces. These are complete complex Laurent-vector checks, not yet
production validation or variance measurements.

Internal CLI family orchestration shares geometry and residual extraction while
retaining recipe-local symmetry, formula and worker identities. Journal version
three checks the canonical family/default before completed-resume admission.
Ordinary and serial generation now publish the same native v2 archive; ordinary
publication retains its existing compiled owner. Optional per-recipe previews
and observations remain outside scientific identity. Public family selection is
still a subsequent interface step; the public contour flag retains its tested
fixed-only behavior.

The final registered source passes **297 core library tests** (16 ignored),
**159 CLI tests** (8 ignored across 24 executables), and **73 portable tests**
(none ignored). The five focused actual-dual/analytic controls are included in
the core total, not counted twice. Independent generation, runtime and CLI
reviews cover source/helper identities, fresh-process native symbol export,
resident publication, recovery and ecosystem ownership. Strict workspace
all-target Clippy and formatting pass. The lint-only correction removes a
redundant default update from the native publication test without changing its
field values. Certified runtime
program wiring, dynamic public interfaces, repeated variance comparisons and
the required physical multiloop accuracy gates remain open.
