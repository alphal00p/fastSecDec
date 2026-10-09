# Fixed-contour HEPKit integration audit

2026-10-09. Independent review of the current FastSecDec native and Python
interfaces. Source review was read-only; after review the coordinator assigned
one narrow lazy inspection getter and its binding regression. No dependency
checkout or running notebook environment was changed, and no Cargo build was
started for this review.

The fixed-mode interface has no newly identified scientific, native-owner or
caller-scheduling blocker. The installed-host evidence is substantial but
applies to a specific immutable source/dependency snapshot. It does not establish
deployment in the shared Community installation, validation against the newly
selected public owner revisions, or contour execution in WASM.

## Native input and implementation ownership

`bindings/python/src/decompose.rs` consumes the existing Feynkit Python owners
`PyFeynmanDiagram`, `PyIntegralFamily` and `PyKinematics`, plus native Symbolica
expressions. The diagram path retains its native numerator, projector and
weights, rejects a competing numerator, and applies positive power overrides by
native edge ID. The family path uses the native family and kinematics, with
explicit signed denominator powers and an already weighted scalar numerator.
Its existing native preparation handles scalar binding and auxiliary vectors.

Both paths delegate parametrization, sector generation, differentiation,
subtraction and evaluator construction to native Rust owners. The bindings add
no graph representation, momentum router, polynomial package, differentiation
engine or numerical estimator. `input.rs` retains native `GraphIntegral` and
`RuntimeModelBindings`; diagram-expression replacement clones and updates the
existing diagram. The numerical core and default CLI remain free of PyO3.

The canonical namespace remains
`symbolica.community.hepkit.sector_decomposition`. Community's `src/hepkit.rs`
registers the FastSecDec module and citations; its Python package is a guarded
wildcard reexport. The substantive implementation remains in FastSecDec's
isolated Rust binding crate. Native Feynkit diagram/family methods forward
keyword arguments to the same registered function. The installed control
exercises `diagram.sector_decompose(contour=True)` directly.

The existing independent numerical references also remain native owner APIs:
the physical B0/C0/D0 controls use HEPKit/OneLOop, and the full physical ggHH
comparison uses the retained HEPKit reduction workflow and MadLoop. This review
does not introduce another master-integral implementation.

## Caller-owned checking and execution

`ContourSettings` is a frozen wrapper over the native validated settings. Fixed
strength is separate from physical parameters, and the public physical parameter
inventory excludes the reserved lambda input. `Kernels.with_parameters` clones
the native evaluator owner and invokes native binding; it does not reconstruct
an integrand or rerun symbolic optimization.

The caller chooses validation coordinates through `validate_contour_point`,
then finishes the required scope through `finish_contour_pilot`. These methods
delegate to native certified arithmetic and retained subtraction-face checking.
No binding-owned sampler or validation RNG is introduced. Homotopy checks are
explicitly required for pilot readiness; finite fixed-strength checks are
documented as safeguards, not a global causal certificate. A scoped finish
cannot unlock unvalidated sectors.

Before mutating pilot readiness, the binding clones a shared `KernelSet` owner.
An already-created session therefore keeps its own readiness and prescription.
Production sessions remain caller-stepped: QMC packages and Havana batches are
issued, evaluated and submitted through existing native work objects. Only
complete successful packages enter statistics. Signal and observer interruption
remain explicit; failed work is retried through the native session rather than
silently accepted. This is the existing ordinary Python execution lane, not a
new process pool or a claim that Python already exposes serial residency.

Native weighted evaluation drains contour diagnostic deltas at batch boundaries
and merges them into the appropriate production/adaptation phase. Optional
checks are absent under `off`; under `pilot`, their production counters remain
zero. These behaviors are exercised by the installed contour tests.

## Status, presentation and retained inspection

Generation and integration observers receive owned native snapshot clones.
`GenerationSnapshot`, `IntegrationSnapshot`, complete-vector estimates,
diagnostics, contour reports and provenance views are frozen Python classes.
Their getters do not run sampling, generation or contraction. The progress
adapter reuses Feynkit's `MarimoProgress`; it coalesces painting only, preserving
the full native callback stream and cancellation semantics.

Generated inspection views retain an immutable `Arc<GeneratedIntegral>` and a
native chart index. `ContourRecipe` exposes stored residual F, positive factors,
complex coordinate images, analytic endpoint ratios and the symbolic contour
Jacobian through native Symbolica expressions. Existing chart mappings,
pre-subtraction powers, coefficient recipes and evaluator operation statistics
remain available. Inspecting them does not invoke a second CAS. The explicitly
requested coefficient materialization operation remains distinct from passive
presentation.

General presentation regressions check that HTML rendering leaves native
artifact bytes, identities and QMC checkpoints unchanged. Those tests cover the
established immutable-view pattern; the new contour recipe HTML itself is
reviewed by source, rather than claimed as a separate installed-browser test.

One narrow inspection improvement follows from this review:

`ContourRecipe.validation_faces` now copies the already retained native list of
coordinate restrictions; an empty restriction denotes the interior. It does
not derive faces or regenerate any expression. A focused binding regression
checks the finite massive triangle's interior-only scope, independent returned
lists, and read-only ownership. Rust formatting and Python syntax checks pass;
execution of this added regression awaits an updated host wheel. The previously
installed 3-test result below does not cover this later addition.

Contour production counters could also be included in the concise diagnostics
HTML summary. They are already available as frozen properties and dedicated
report views; this is a presentation follow-up, not a missing native diagnostic.

Rich chart views currently belong to the retained generated object. A reloaded
Python `Kernels` owner exposes capability, check charts and evaluator statistics,
but does not yet offer the same complete lazy generation-metadata explorer.
The native artifact retains that information; a future Python artifact viewer
should reuse it instead of reconstructing generation.

## Mathematical identity and checkpoint evidence

Fixed lambda enters the native bound content identity. Validation policy is
observational and does not. `with_contour_validation` delegates the policy-only
change to the native owner without rebinding the mathematical integrand.
Native QMC/Havana checkpoint restoration receives the expected native integration
problem and preserves native replay validation.

Checkpoint pilot provenance is separate from statistical compatibility. The
binding records the caller-supplied protocol and honest `seed=None`, rather than
inventing a sampling seed. Previous pilot records can be displayed after resume
but do not unlock a newly restored owner's validation gate. The contour tests
exercise identical production values across all three policies, policy-only
QMC and Havana resume, and preserved evidence with no fabricated seed.

## Installed evidence and source boundary

The [installed-host review](contour-python.md) gives the complete dependency and
build identities. This audit checked the actual logs:

- `target/contour-community-contour-tests.log`: **3 passed in 0.46 s**.
- `target/contour-community-native-tests.log`: **205 passed in 76.81 s**, with
  no skips, errors or failures.

The installed private wheel's SHA256 was independently recalculated as
`13da421d3b37854d050d68696fbbba2d2e3c84ace52e9e15604300ceb48a3371`.
Its immutable FastSecDec snapshot records source SHA256
`00b6f4c7eb0e7fb1c3535e0b0cb70f833de245270f5f5b97dc91a753bca12a62`.
Before adding the inspection refinement above, all twelve binding test source
files matched the working tree. All 36 binding Rust files also matched, apart
from `citations.rs`, whose current source adds the URL fields required by the
newer public Symbolica citation API. Generated Python bytecode was excluded from
this comparison. The later getter/test changes are recorded separately rather
than attributed to the previously installed wheel.

This wheel used Symbolica's local ball-domain owner commit `3db1607`, registry
Numerica/Graphica 3.0.1, the tested SymJIT 2.27.0 lane correction and the coherent
Feynkit owner `fd1b43c`. The current core has subsequently gained dynamic
foundations/artifact work and selected public owner revisions. The 3+205 result
must therefore not be relabeled as an installed-host gate for that later source
and dependency matrix. The separate public-source workspace rebuild is not an
installed Python wheel test.

## Fixed delivery follow-ups versus later phase gates

Before claiming the current fixed feature published and usable from the shared
HEPKit installation, update the host's FastSecDec selection, regenerate its
stubs, and repeat the installed-host gate against the final public owners.
The shared Community checkout inspected here still selects the earlier
FastSecDec `7f75c158...` revision. The successful wheel is private validation,
not an update of that running notebook environment.

The native Feynkit forwarding hooks accept `**kwargs`, so current execution
supports `contour`, `mode` and `subtraction`. Their handwritten text signatures
and owner stub declarations still omit those options. This is a discoverability
and typing gap, not a runtime rejection. Correct the owner signatures when
updating the host; regenerating only FastSecDec's stubs cannot fix them.

Dynamic settings, recipe selection in Python, root/displacement statistics and
dynamic checkpoint compatibility await the corresponding native production
implementation. Fixed/dynamic variance claims await matched-work measurements.
Actual Pyodide/browser contour execution remains a distinct required gate;
earlier undeformed portable tests and a portable Rust build do not establish it.
These pending items prevent declaring Phase B complete, but do not invalidate
the reviewed fixed native binding milestone.
