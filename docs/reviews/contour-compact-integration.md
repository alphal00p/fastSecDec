# Compact contour coefficients: integration review

This change retains repeated dynamic-envelope coefficients as explicit native
Symbolica functions through map construction and subtraction. Their bodies,
ordered formal parameters and source-chart ownership remain available for
inspection and native artifact restoration. The radius callback itself remains
visible, preserving the existing implicit derivatives and certified face
requests. This is a representation change, not a new contour prescription.

## Native ownership and algebra

The [Jacobian audit](contour-jacobian-alias-audit.md) records the public API,
source and executable probes supporting the implementation. Symbolica owns
derivatives, simultaneous substitution, function-map lowering, dual jets,
optimization and instruction serialization. No replacement algebra,
automatic-differentiation or numeric callback system is introduced.

Each generated sector owns the definitions for its representative source chart.
Symmetry multiplicities reuse that representative; other source-chart metadata
does not enter its evaluator function map. Numerical-dual caches compare native
bodies and ordered signatures as well as expressions, inputs and compilation
settings. Exact contributions materialize restricted calls before aggregation.
Hash-derived names locate definitions; native body and formal-parameter equality
remain mandatory when merging owners.

Serial generation retains disk record references and compact completion receipts
in the coordinator. A worker persists its indexed native unit before returning
its receipt; publication copies that record without decoding its evaluator or
metadata. Full definition bodies and Jacobians therefore remain worker-local.
This is a source-level ownership finding; aggregate RSS must still be measured
on the actual multiloop generation path.

Dynamic symmetry uses the undeformed source density with its explicit causal
factor, ordered positive factors and recipe. It reuses native graph candidates
and exact coordinate-permutation verification. The source density uses the
same branch-aware powers as the contour map: cancelling plain fractional powers
cannot erase a distinction between causal and principal logarithms. Constant
or cancelled densities retain their causal declarations separately.

An independent final source review verifies the corresponding covariance:
the gradient, coordinate weights, Euclidean displacement norm and directional
envelope transform under the proven coordinate permutation; the Jacobian is
conjugated and retains its determinant. Full source signatures remain local to
each separately lowered program. This review found no unresolved duplication
of Symbolica algebra or unsafe cross-chart alias admission.

## Artifacts and inspection

The [codec review](contour-compact-codec.md) describes the v12 native definition
sidecar and unchanged historical nested layouts. Definition bodies and symbols
use Symbolica's existing StateMap codec. Saved optimized programs remain the
runtime input; loading does not reconstruct or optimize sector expressions.
Generation staging advances independently to schema 4 and rejects old partial
records explicitly.

HEPKit exposes the same native metadata through retained `ContourRecipe` views,
including `Kernels.contour_recipes` after selection or restoration. Compiled views
share the existing `Rc<KernelSet>` and are Python-unsendable; generated views
retain their existing native owner. `function_definition_count` is a cheap
summary. `function_definitions` returns formal calls and native bodies only on
demand. Rendering uses the existing bounded Symbolica formatter and never
materializes definitions, differentiates, compiles or samples. Community needs
no substantive implementation for these additions.

## Acceptance boundary

An independent raw-evaluator precision probe isolated an endpoint refusal in
both compact and materialized maps. At `x=1`, the optimized double-double
program passes `a2 = 1 - 3.0814879110195774e-33` to the radius callback; its
mandatory `a2 >= 1` admission correctly rejects that rounded value. The same
coefficient evaluated alone gives exactly one, illustrating the effect of
different arithmetic ordering. Interior results agree within `3.1e-33`, and
192-bit evaluations agree within `4e-59` at all five tested points, including
the endpoint. This is not evidence of a compact-lowering or upstream arithmetic
bug. Retain the explicit refusal and precision recovery; do not clamp the
coefficient, fabricate a finite result or relax the finite parity tolerance.

Independent source reviews cover runtime/generation ownership, branch-aware
symmetry and artifact compatibility. The final native library gate passes
401 tests (19 ignored), including genuine mixed-signature chart compilation
and restoration in both generation modes with actual pilots. The portable
suite passes 79 tests across sixteen executables, with no ignored tests.
These totals include their focused subsets; do not count those subsets again.

The immutable installed-consumer source archive has SHA256
`8429ae41d2a5bee14b02d621d6f3eaa4cbdeb36c2513f2c49bdd42effb27c1f1`,
based on `98aa8f9` plus this increment. Its actual Pyodide wheel has SHA256
`5927fdccce3e35526cc13a6f676b26d8d8ee6164e661ba42cb491f055a5004ac`
and passes 138 maintained tests in 16.63 seconds, with no skips. This exercises
the new compiled inspection views and restored metadata; it is not browser UI
evidence. The same snapshot's full-default native Community wheel has SHA256
`bb9a79d7ce0472633afc5acdd273e28304429cd84b213cc4477c0ead7478776b`
and passes all 249 native binding/demo/wavefunction tests in 79.63 seconds,
with no skips. Its initial 247-pass/two-failure run omitted the wheel's declared
`notebook-display` extras; the nested widget exception identified missing
`typst`. Installing the same `anywidget 0.11.0` and `typst 0.15.0` extras used
in the preceding accepted environment fixes the two notebook-rendering tests.
The wheel and native extension hashes remain unchanged. No production fallback
or reduced assertion was introduced.

Fresh generated stubs pass six-module parsing, seventeen runtime-signature
comparisons, sixteen native class checks and four new inspection-property
checks. Standalone binding Clippy passes with native and stub-generation
features, all targets and warnings denied. The production numerical and
binding code matches the immutable tested snapshot byte for byte. Two later
test-only lint corrections borrow existing Atoms instead of cloning them;
they change neither the tested expressions nor their assertions. Shared
Community and notebook installations remain untouched. The complete workspace suite passes 904 tests with 33 explicitly ignored
controls across ninety result groups: core 653/23 ignored, CLI 173/8,
QMC 37/0 and sectors 41/2. The 401-test library count is included in the core
subtotal. Strict workspace all-target Clippy passes with warnings denied
(21.24 s); workspace and binding formatting checks pass. After the two test-only
borrow corrections, all eight definition tests pass again in 0.26 seconds.

An independent probe of the actual six-dimensional K1 chart compares its six
coordinate images and determinant against a native 192-bit Dualizer/Matrix
reference. Eight point/settings combinations, including a coordinate face,
pass on eager, SymJIT, double-double and 192-bit backends. Maximum scaled
differences are respectively `1.34e-16`, `4.51e-16`, `5.67e-32` and `6.24e-58`.
All 32 backend vectors are finite and callbacks report no failures. The run
takes 2.60 seconds and peaks at 137.3 MiB. Its numeric workload differs from
the earlier 575 MiB lowering probe; no direct memory ratio is inferred.

The bounded production K1 campaign uses the same canonical physical source
as the earlier failure, symbolic generation, one worker and a separate process
monitor sampling aggregate parent/child RSS every 50 ms. At its declared
300-second limit it has completed all 186 source records, all 186 dynamic
chart mappings and 25 symmetry receipts. Peak sampled aggregate RSS is
246,398,976 bytes (234.984 MiB), with a coordinator peak of 21.875 MiB.
The previous representation exceeded 3 GiB before its first mapping completed.
This establishes a large improvement in actual chart preparation; it does not
bound uncompleted evaluator compilation or claim a complete artifact. Host
builds/tests overlapped, so the 300.261-second campaign is feasibility evidence,
not idle-host performance. A separately budgeted resume retains the receipts.
The [memory audit](contour-ltd-generation-memory-audit.md) records its provenance.

The required multiloop accuracy and fixed-versus-dynamic variance gates remain
open; neither the standalone probes nor partial generation complete them.
