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
| Native regression suite | **730 distinct enabled tests pass; 29 are ignored.** This combines 245 core scientific/integration tests, the updated library's 252 tests and the final CLI/QMC/sector gate's 233 tests on the final public matrix. The new exact-only pilot and scope regressions pass. Ignored tests are not claimed as acceptance evidence. |
| Strict lint gate | Final public-matrix `cargo clippy --workspace --all-targets --locked -- -D warnings` passes, including the readiness correction and native/CLI artifact foundations. |
| Dynamic algebra | Eight native envelope tests plus shared local-strength Jacobian and higher-jet Taylor/IBP controls pass; the combined contour foundation gate passed 51 tests. Production generation wiring remains pending. |
| Dynamic numerical root | Native callback/implicit-jet and restoration foundations pass. Owner tests cover prepared eager/JIT solves, tracked hypot, certified square roots and portable domains. An independent enclosure probe passed 2,620 cases at 8/24/96/256 bits. Production checker integration remains pending. |
| Recipe-addressable artifacts | The native descriptor gate passes five tests and the artifact gate 37. The real CLI recipe test passes all ordinary/serial integration combinations for undeformed/fixed saved programs, including covariance, selective inspection and checkpoint isolation. Production generation of a complete alternative recipe set remains pending; dynamic admission is explicitly rejected. |
| Portable execution | The full maintained standalone portable suite passes **73 tests, zero failures or ignored**, with ordinary `--locked` Cargo on the final public source matrix. This includes fixed threshold bubbles in both generation modes, all complex Laurent components, certified pilot/production validation, fresh-process restoration, covariance and checkpoint/replay controls. Native portable execution is not an actual browser/WASM run. |
| HEPKit/Python | A frozen full-default private Community wheel passed **3 contour tests and all 205 maintained binding/demo/notebook/wavefunction tests**. That snapshot used earlier fixed-mode prerequisites. The final public-matrix standalone binding check passes, including the new lazy validation-face getter and exact-only readiness correction. Executing these newer Python regressions in a refreshed installed wheel remains a separate gate; actual WASM execution remains pending. |
| Variance and performance | [Matched-work protocol](contour-variance-protocol.md) and fixed/fixed executable control verify matching actual coordinates/weights, separate result identities and complete covariance. No dynamic production variance gain has been measured or claimed. |

The final publicly reproducible source matrix is Symbolica/Numerica
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, SymJIT
`33100ae869057f35d9865c933a48bac6699acdd4`, and Feynkit
`8e3a643f388b45939d6573a648ef3a509086835e`. All three maintained manifests
and lockfiles select public sources; no local file URL or build-directory
override is needed. The Symbolica revision retains existing evaluator
composition as well as the new owner fixes. SymJIT's dependency requirement is
a compatible minimum (`2.27.0`); the temporary public Git patch selects the
reviewable unreleased fixes.

The [upstream PR ledger](contour-upstream-prs.md) records the owner patches,
their focused tests, authorship and actual review-request outcomes. GitHub
accepted some formal `benruijl` requests and rejected others because he is not
a collaborator; the latter have explicit authorized review invitations in
comments. These PRs are not claimed as merged. Current Community's new required
citation URL field also has small owner-specific compatibility PRs; the
host-specific Vakint change preserves its existing RustRed integration.

The installed Python evidence uses the frozen source snapshot
`00b6f4c7eb0e7fb1c3535e0b0cb70f833de245270f5f5b97dc91a753bca12a62`,
not the newer dynamic foundations. No shared notebook installation was replaced.
The [HEPKit audit](contour-hepkit-milestone-audit.md) records remaining wrapper
signature/stub and restored-inspector gaps explicitly.

The required LTD two-loop and three-loop native fixtures now have reproducible
HEPKit graphs, source provenance and four passing importer controls, including
native U/F and normalization checks. All four CLI input inspections pass.
Their contour generation/integration gates remain pending. The six-point
example has a strictly negative interior F and is a branch control, not
evidence of crossing an interior threshold surface. See the
[fixture audit](contour-ltd-fixtures.md).

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

The next private installed-wheel gate deliberately preserves the complete
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
