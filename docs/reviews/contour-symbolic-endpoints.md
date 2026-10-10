# Symbolic endpoint reduction and contour-only duals

2026-10-10. This independent audit applies the user's clarification in
[the contour plan](../../CONTOUR_DEFORMATION_PLAN.md#symbolic-endpoint-reduction-and-contour-only-duals).
The required comparison uses native symbolic endpoint reduction in both arms:
symbolic deformation Jacobian versus a Jacobian evaluated using native duals.
It does not authorize numerical-dual endpoint reduction.

## Confirmed implementation mismatch

The previously measured `ContourJacobian::Dual` path requires
`GenerationMode::NumericalDual`. It composes a first-order image Dualizer and
determinant into the smooth density, then applies the NumericalDual subtraction
pipeline's outer jets to that entire density. Those controls established the
mathematical correctness of that construction, but not compliance with the
clarified endpoint algorithm. Its generation and sampling measurements remain
historical evidence. In particular, their cost cannot reject the requested
first-order, contour-only dual approach.

The current symbolic path already applies native `Atom::derivative` to the
complete mapped regular density. The IBP boundary term evaluates that density
at one; each bulk step differentiates it before proceeding. Taylor subtraction
then differentiates before restricting its coordinate to zero. Thus the
Jacobian, local strength, image substitutions and factorwise analytic branches
all participate. There is no justification here for reordering IBP before
deformation or treating the Jacobian as constant.

## Mathematically admissible lowering under investigation

Write the image matrix entries as `m_ij(x) = partial_j z_i(x)` and retain the
native compact determinant call `D(m_11(x), ..., m_nn(x))` inside the full
deformed density. Symbolica's symbolic chain rule differentiates this call
during endpoint reduction. It produces determinant partial functions and the
required symbolic higher derivatives of the images, including derivatives of
the position-dependent strength. Existing native function-map registration
resolves determinant partial bodies with `Atom::derivative`.

At the final evaluator boundary, the candidate lowering replaces surviving
first-image-partial subtrees with private scalar symbols using native exact
Atom matching, then supplies those symbols from a **first-order image-only** Dualizer.
The already-symbolically-reduced density is compiled normally; no outer
Dualizer computes endpoint derivatives. Native evaluator composition and the
saved exact instruction codec are intended to remain the execution boundary.
This is a representation proposal, not an accepted implementation yet.

Faces require derivatives of the original image evaluated on the face.
Restricting an image first and differentiating the restricted expression loses
normal derivatives. Coordinate seeds must therefore remain one on a face;
non-coordinate runtime parameters have zero coordinate seeds. Repeated equal
matrix entries must share one native input, and literal entries must not be
reinterpreted as free parameters. Exact offsets must still resolve their full
semantic expressions after restriction and before exact aggregation.

The higher derivatives of the local implicit strength needed by those symbolic
formulas remain legitimate contour derivatives. Existing native derivative hooks
may supply them. The prohibited endpoint mechanism is applying a higher-order
Dualizer to the full smooth density to construct its subtraction coefficients;
the new image evaluator uses degree one only. This distinction neither freezes
local strength nor drops derivatives demanded by symbolic IBP.

Exact-only boundary terms retain native symbolic materialization, cancellation
and direct exact binding in both constructions. They do not acquire a stochastic
Dual prefix merely because the Jacobian setting is Dual. This deliberate exact
path must be visible in documentation and checked with both a surviving
nonzero exact offset and genuinely cancelling exact contributions.

## Native evidence and current limits

Source inspection covers `generation/subtraction.rs`,
`contour/definitions.rs`, `kernel/program.rs`, and the existing
`generation/numerical_dual/native/cache/jacobian.rs` and outer-jet pipeline.
These establish the current coupling and identify reusable native symbolic
derivative/function-map and evaluator-composition operations. No alternate CAS,
AD rule implementation, or dependency defect is proposed.

A focused ignored probe is retained under
`target/foundation-symbolic-endpoints/`. Its first control confirmed an important
native API constraint: `evaluate/tree.rs::linearize_impl` checks the parameter
map only for variables and function calls. Supplying the sum `x+y` as an extra
input does not override that sum. With `x=2`, `y=3` and a purported sum input of
7, the native program returned 125 for `(x+y)^3`, including the same behavior
inside a registered function's arguments. The baseline source, failed assertion
and exact owner/build identity are preserved. This is not treated as a broken
documented owner contract or a reason to add a separate evaluator.

The corrected probe passed using native exact-subtree replacement with private scalar
symbols, followed by native evaluator composition. It checks actual replacement
counts and that the composed program retains only its original public inputs;
this prevents an apparent Dual option that silently recompiles the entire
Jacobian symbolically. There is exactly one Dualizer construction, with the
degree-one shape `(0,0), (1,0), (0,1)`, applied only to the two images. All
derivatives of the density use native symbolic `Atom::derivative` before its
ordinary evaluator is composed with those image outputs.

Three fixtures each check the density, its x derivative, mixed xy derivative
and mixed xxy derivative over five interior/face choices and three points:

| Fixture | Compared complex components | Actual subtree replacements | Maximum private inputs | Maximum absolute error at 192 bits |
|---|---:|---:|---:|---:|
| Rational position-dependent strength | 60 | 1,514 | 4 | 6.373e-58 |
| Repeated matrix entries and a singular determinant face | 60 | 330 | 3 | 0 |
| Literal zero/one entries | 60 | 15 | 1 | 0 |

The reference differentiates the explicitly substituted native determinant and
complete density symbolically. Every composed program is encoded and decoded
with the native instruction codec before comparison and retains exactly the
original ordered three public inputs. Face inputs are set after original-image
differentiation, with coordinate seeds still one. This tests normal as well as
tangential face derivatives without dividing by the determinant.

Execution took 0.115552 s with 6,680,576 bytes peak child RSS and no physical
sampling. The pinned owner is `74225696cd445247fa81c499c5110decd19257ed`.
Source SHA256 is
`e169444207883b188995a9c96d84aa6aea92c4bd6c82495642bdbf7f4e22e12b`;
the build, executable, output and resource identities are in `build-identity.json`
and `corrected.json`. The initial unsupported-input assertion is preserved
separately in `raw-input-baseline.log` and `probe-raw-input.rs`.

This is an executable native capability proof, not a full endpoint-generation
or physical test. The varying strength is an explicit rational function, not
the production implicit-root callback. The codec check runs in the same
process; it is not fresh-process symbol-state restoration. Production acceptance
still requires both Jacobian choices through actual Symbolic IBP/subtraction,
complete Laurent vectors, surviving and cancelled exact offsets, actual
implicit-strength derivatives, retained face/checker associations, and fresh
saved-artifact execution. No new physical generation or timing comparison is
accepted by this probe alone.

## Production review requirements

Review of the initial compiler draft identified two additional execution
boundaries. Candidate matches in an unused alias, or a nested match removed by
a larger replacement, must not cause a contour-image prefix to be constructed.
Only actually used private inputs may request faces or radius callbacks. Native
`EvaluatorComposer::finish` already prunes unused straight-line instructions
and callback constants; the public `ExportedInstructions` operands can then
identify used `Slot::Param` inputs. A separate alias dependency engine or CAS
optimizer is unnecessary. The native API has no direct used-input accessor.

Native composition explicitly retains programs with conditional control flow
intact. Hoisting a contour prefix out of a lazy branch could execute a radius
callback that the original program never calls. The opt-in Dual-J compiler must
therefore preserve that execution or reject unsupported control flow explicitly.
For this delivery an explicit rejection is acceptable; silently compiling the
whole Jacobian symbolically while reporting a Dual measurement is not. Actual
physical Dual generation must confirm that its compiled body satisfies the
supported boundary.

Ordinary symbolic lowering merges diagnostic contexts when native algebra
makes restricted radius expressions equal. A face-specific image evaluator
must preserve that same union while differentiating the unrestricted callback
arguments, then supply face coordinates afterwards. Taking only one face's
context after deduplicating equal matrix entries would lose existing proof
obligations. Tests must cover this union, actual degree-one construction,
cancelled/unused callbacks, private-input collision and removal, aliases, and
changed runtime parameters. The executable gates below cover this boundary.

The corrected source draft now implements this boundary in
`kernel/program/contour_jacobian.rs`: it compiles the rewritten body by itself,
uses native Composer pruning, scans every exported instruction's input operands,
and constructs image prefixes only for faces supplying surviving private inputs.
Unused private inputs receive a literal zero after their absence has been proven
by native liveness. A live image binding combined with conditional flow or a
sub-evaluator yields an explicit unsupported-compilation error. The default
Symbolic Jacobian path remains available.

Independent source review found no remaining liveness or ownership blocker.
`contour/jacobian/program.rs` constructs only value and degree-one coordinate
components, seeds runtime parameters with zero derivatives, and extracts the
row-major first image partials. Face coordinates are composed after this
operation. Native chart restoration checks the single record-local plan index
zero, images and Jacobian; streamed execution separately checks its original
job/source identity. `Assembly` checks a unique local plan, coordinate order and Dual
policy, without confusing an original source index with the record-local
representative index. Compilation also checks the retained coordinate order. Private names avoid the
symbols present in roots, aliases and inputs, and the final evaluator has only
the original input count. Exact-only coefficients retain the existing native
materialization route. The observed-binding count makes actual contour-only
construction testable instead of inferring it from the setting alone.

The draft tests use genuine Symbolic Taylor and IBP reduction of a third-order
endpoint pole, all three contour recipes, native implicit-radius derivatives,
both eager and SymJIT evaluation, and saved-program restoration. A separate
unknown-image/unused-alias control requires no contour prefix at all. Public
closed-form first- and second-order endpoint tests separately check surviving
exact offsets and complete residual vectors. Executable controls also cover
conditional rejection and private-name collision, as recorded below.

The new multi-chart staged regression exposed a pre-existing optional-Dual
transport error: a saved program's local index zero was compared with its
original source index. The correction preserves the local index check and all
native image/Jacobian comparisons; numerical-dual instantiation compares the
plan with its unique definition owner, whose index is remapped in the same
operation. The original streamed job/source identity checks remain unchanged.
Independent source review accepts this correction; the genuine source-greater-
than-zero fresh-worker regression is the executable gate.

The public closed-form test initially assumed a finer exact/stochastic partition
than the native API provides. `Assembly` folds a complete coordinate-independent
Laurent vector; constant summands of a stochastic vector remain in that vector.
For the endpoint fixture its exact owner is therefore zero, while its stochastic
kernel includes the pole and endpoint constants as well as the residual. The
unchanged closed form must be compared with that complete vector. A separate
zero-gradient constant source checks a surviving exact offset. This test-contract
correction changes neither the integral nor its tolerance and does not establish
a numerical implementation defect.

The final focused native gates pass: five contour-only compiler controls, four
staged controls including both endpoint modes and source indices greater than
zero in fresh workers, three public policy/closed-form controls, exact
cancellation, root-context union, and two historical numerical-dual controls.
These are **16 distinct tests**. The public closed forms retain their original
`2e-12` tolerance; the standalone constant source restores and binds an exact
`3/2` value. Compiler statistics now expose an optional count of surviving
first-image-partial inputs: `Some(n)` at construction, including `Some(0)` after
native pruning, and `None` after restoration or for an inapplicable route. The
count is not stored in the executable codec or used during evaluation. Tests
check positive construction counts and unavailable restored counts.

Logs are retained under `target/contour-symbolic-endpoint-`: `core-tests.log`,
`staged-tests-final.log`, `public-tests-final.log`, and `regression-tests.log`.
The independent review also confirms that candidate6's frozen production source
differs from the current source only by line wrapping in the record-local index
predicate; exact token-level inspection found no semantic change. The final
workspace and lint gates run on the current formatted source. Focused counts are
not added to their overlapping total.

The complete current workspace passes **925 tests**, with **33 explicitly
ignored** tests, across 91 result groups. This comprises core 671/23 ignored,
CLI 176/8, QMC 37/0 and sectors 41/2. The command is
`cargo test --workspace --locked -j8 -- --test-threads=1`; it closed normally in
605.929 seconds with unchanged production/test source hashes. Locked metadata
and actual Rustc invocation evidence identify owner `74225696` and the current
worktree, rather than a cached frozen-source path. Logs, source inventory and
package counts are retained under `target/contour-symbolic-endpoint-final-`.

One additive public test subsequently closes the portable implicit-root gap.
It uses genuine Symbolic IBP of `x^(-3-eps)` with cubic
`F=(1/4-x)(1+x^2)` and numerator `1+i`, both dynamic constructions and both
Jacobian choices. The undeformed source's native second Taylor coefficient
independently gives residue `-60(1+i)`. Complete vectors agree after whole-owner
same-process restoration at three points, with actual positive Dual first-image
slot counts, no additional runtime parameters, native pilot admission and
readiness. The public target passes four tests on native and portable hosts;
three overlap the workspace gate, so the distinct native total is **926 passed,
33 ignored**. Standalone portable strict Clippy also passes in 51.00 seconds.
`target/contour-symbolic-endpoint-dynamic-public-result.json` pins the new test
and native/portable logs. This is not an actual WASM or installed-wheel test claim.

The thin Python binding all-target check with `python_stubgen`, its strict
Clippy gate and formatting pass. The final workspace strict lint initially
reported only an unnecessary clone in a test slice; its correction changes no
production source. Strict workspace/all-target Clippy then passes with warnings
denied in 16.316 seconds. The current source differs from the full-suite snapshot
only in that test borrow and the additive public test; the production inventory
is unchanged. The final focused borrow control passes, as do workspace
formatting and diff checks; every owned QA process has closed. Evidence is
recorded in `target/contour-symbolic-endpoint-final-completion.json`.
