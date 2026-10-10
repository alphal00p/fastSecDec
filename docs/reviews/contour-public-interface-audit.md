# Public contour interface and ecosystem audit

2026-10-10. Independent source review of the current family/diagnostics increment.
The initial findings below describe the source reviewed before the retained
family-input follow-up. The final section records their resolution and the
exact refreshed native/Pyodide execution gate. No production source was changed
by this review itself.

## Existing paths and native ownership

| Entry point | Available generation | Retained owner and execution |
|---|---|---|
| CLI `generate --contour` | All four native recipes, undeformed archive default | Bounded native family session; ordinary caller dispatch or existing serial journal/process runner |
| CLI explicit `--recipe` | Requested singleton | Existing singleton native path |
| HEPKit diagram/family `sector_decompose(..., contour=True)` | Fixed singleton | Existing `GeneratedIntegral`; compilation remains a separate action |
| `sd.Integral(...).generation_session(contour=True)` | Fixed singleton | Existing caller-stepped singleton generation |
| `sd.Integral(...).generation_family_session(recipes, ...)` | Explicit requested set including both dynamic constructions | Same native `RecipeFamilySession`, with default and optional resident independently declared |

The CLI family route does not duplicate mathematical preparation. It delegates
source sharing, recipe-local symmetry and formulas, bounded jobs, canonical
assembly and archive validation to native owners. The CLI owns its Rayon pool,
serial workers, UI, publication and invocation policy. Requested Run residence
is validated before parametrization and does not rewrite the artifact default.

The Python family wrapper owns only input and storage lifetimes, explicit step
calls, Python interruption translation and frozen snapshot adaptation. It does
not run a Python recipe loop, rebuild graphs, compute determinants or implement
pilot certification. `RecipeArchive.select` reuses an explicitly retained native
resident; other choices use the native selective reader before loading programs.
Loaded kernels retain their programs and lazy metadata independently of archive
file lifetime. Imported raw archives correctly expose no invented default.

The current diagram owner is an actual `GraphIntegral`. The existing family
dispatch invokes native `prepare_family_input` and
`ParametricIntegrand::from_family`, preserving signed powers, explicit weighted
numerators, kinematic assumptions and scalar bindings. No separate physical
graph or CAS representation was found. Validation uses native contour point,
pilot completion and readiness APIs; native QMC/Havana own sampling, statistics,
replay and checkpoint admission. Presentation only reads retained state.

## Actionable interface gaps

1. **Native HEPKit IntegralFamily cannot currently reach dynamic generation
   through Python.** `PyIntegral` in `bindings/python/src/input.rs` retains only
   `GraphIntegral`; its constructor accepts `FeynmanDiagram`. The only public
   IntegralFamily route is the synchronous branch in `decompose.rs`, and its
   boolean generation option selects only undeformed or fixed. Dynamic family
   generation is therefore available for diagrams through `sd.Integral`, but
   not directly for native families with signed powers and weighted numerators.
   This is a Phase B interface acceptance gap, not a numerical-core failure.
   Root approved designing an inert `Integral.from_family(...)` owner that
   reuses the same native family preparation at explicit generation time and
   feeds the existing family session. That follow-up is not implemented by this
   audit. Preserve existing return types and explicitly document the legacy
   fixed-only boolean route rather than silently returning an archive where a
   GeneratedIntegral is promised.
2. **Python sampling diagnostics are not exposed on the session owner yet.**
   The new frozen runtime reports on `PyKernels` correctly describe that
   object's work. QMC and Havana create independent `WeightedEvaluationContext`
   owners, so their sampling callbacks do not accrue on the originating
   KernelSet. No runtime-report getter/aggregate was found on either Python
   sampling session. A thin caller-owned aggregate of the native context
   reports would expose actual sampling root costs; it must retain dropped
   context history and preserve the existing RNG/checkpoint identities. The
   current getter's documented scope is truthful, so this is an observability
   acceptance follow-up rather than corrupted numerical reporting.
3. **Dynamic settings presentation omits its mathematical controls.** The
   current HTML card in `presentation.rs` shows mode, fixed lambda and policy,
   but not dynamic construction, S, L or R despite their native getters.
   Showing them would prevent a notebook from displaying an incomplete
   prescription, especially after the observed physical cap sensitivity.

No additional mathematical admission bypass, duplicated algebra, hidden worker
pool, graph conversion or implicit sampling was found. The input-family gap
does not invalidate the native dynamic science or the existing diagram path;
it must be closed before claiming complete HEPKit family integration.

## Optional conveniences and separate validation

Rich cards for RecipeArchive/RecipeFamilySnapshot, a shorthand requesting all
four recipes from Python, and lightweight per-recipe catalogue summaries before
selection are useful conveniences. Existing explicit recipes, frozen snapshots,
`catalogue_json()` and lazy selected chart/F/U/map/Jacobian/face inspection already
provide the underlying native information, so these are not correctness
blockers. Static recipe labels in CLI metadata are versioned; the Python facade
uses its documented unversioned labels.

Public forwarding signatures and generated HEPKit stubs must be checked against
the actual host revision. Historical cached Feynkit sources omit newer keyword
names even where runtime `**kwargs` forwarding accepts them; an older cache is
not evidence that the root agent's patched current host has the same omission.
The installed-wheel/stub gate remains responsible for that exact source matrix.

The maintained native nonlinear sampling target passed 4/4, and the new portable
family/diagnostics targets passed on both the portable host and actual
Node/Emscripten. Those tests establish native owner and filesystem behavior;
they do not establish a Python wheel, browser responsiveness or physical
multiloop performance. See the separate
[sampling](contour-dynamic-sampling.md),
[portable family](contour-portable-family.md) and
[physical](contour-dynamic-physical-readiness.md) reviews for evidence and limits.

## Follow-up implementation, 2026-10-10

After milestone `a3d97cf`, the retained native family constructor and complete
dynamic settings card were implemented in the isolated binding crate. The
legacy family route delegates to the same native owner and retains its existing
return type and fixed singleton boolean meaning. See
[retained-family review](contour-family-input.md) for the exact interface,
native reuse and tests. Leaf all-target/stubgen check and strict Clippy passed.
Installed candidate-1 source content SHA256
`9950910602b5c5fe040f30aa2a3d388ef8c28f63a4f6b7d11f02d0eec6d77c12`
then passed the full native Python suite **249/249 in 78.45 s** and actual
Pyodide suite **138/138 in 16.32 s**. All eight new family-input cases passed on
both hosts. Findings 1 and 3 are therefore closed for this source matrix,
including signed powers, retained runtime parameters, real dynamic pilots and
integration through the shared native family session.

Finding 2 has an independently reviewed thin implementation: native
`EvaluationDiagnostics` carries optional phase-labelled runtime reports; Python
exposes frozen report copies and drains each context after attempted evaluation.
Havana adaptation history survives its production transition separately from
accepted production statistics. The same native DTO is transported by ordinary
and serial CLI workers. The same candidate host gates also passed the three
new status/diagnostics controls on both hosts. Detailed review and limitations are recorded in
[operational transport review](contour-operational-transport-audit.md).
