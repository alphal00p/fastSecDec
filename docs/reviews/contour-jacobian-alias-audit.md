# Compact native contour Jacobian representation audit

2026-10-10. Independent read-only audit after `fd8ce34`, with native Symbolica
`516beb37d31af8e3d6ee321a7070f407a0b1b42d`. Runtime and generation agents own the
isolated executable probes; this review changes no production representation,
dependency, solver or differentiation implementation.

## Observed failure boundary

The actual first six-dimensional `2L4P.b.K1` chart has a 797,924-byte dynamic
strength Atom. Its six native coordinate images are approximately 799 KB each.
Native implicit differentiation produces 36 Jacobian entries of
15,534,713–18,573,660 bytes, totaling 577,726,202 native Atom bytes (550.96 MiB).
Derivatives finish at 5.77 seconds. The probe then calls the current registered
determinant-template implementation unchanged and exceeds its three-GiB
aggregate RSS limit; the monitor stops it at 8.10 seconds, observing
3,362,672,640 bytes. Polling explains the overshoot beyond the limit.

This is an expression-construction failure before evaluator optimization or
sampling, not evidence of incorrect values or slow optimizer convergence. The
native envelope and coefficient extraction separately complete with about
48 MB RSS. The four polynomial coefficient Atoms occupy 4,709, 379,750, 3 and
413,440 bytes. Raising the generation-worker count would multiply the observed
pressure and is not a remedy.

The reproducer includes the actual `contour/determinant.rs`; it does not replace
the determinant algorithm. Its source and raw evidence remain ignored under
`target/contour-ltd-dynamic-k1/map-probe.rs` and
`runs/map-native-registered-chart0/`. The associated full dynamic generation
attempt mapped zero of 186 charts within its resource cap. No LTD dynamic
scientific acceptance follows from these probes.

## Native API, source and executable evidence

| Facility | Verified contract | Consequence |
| --- | --- | --- |
| `AliasedAtom` | `atom/alias.rs` documents an opaque root and stores a native alias map. `AtomCore::derivative` in `atom/core.rs:688` differentiates the root and wraps the existing map; there is no alias-aware override. | A hidden body must not be treated as a differentiable variable. The probe returns zero for an opaque alias derivative while the materialized native expression has a nonzero derivative. |
| `FunctionMap::add_function` | Function bodies are native Atoms. Formal arguments shadow global inputs; bodies can additionally see global inputs (`evaluate/function_map.rs:114`). | A short function call can retain visible coordinate dependence, but every mathematical dependency must be passed explicitly if symbolic derivatives are taken. Captured globals are invisible to the call's symbolic chain rule. |
| Native formal derivatives | `derivative.rs` constructs `der(depths..., function, arguments...)` and applies the native chain rule. A function body registration alone does not register its derivative bodies. | An unregistered derivative call fails evaluator construction, as the probe confirms. It must never silently become an independent input or zero. |
| Explicit derivative registrations | `FunctionMap` accepts tagged registrations for `Symbol::DERIVATIVE`; the probe supplies native `body.derivative(x)` definitions. | Existing public APIs suffice for explicit requested derivative bodies. Automatic derivative resolution is not yet a task-blocking owner gap. |
| `InliningPolicy::Always` | Native function bodies lower into the existing evaluator instruction graph at construction. | The registered derivative-body probe evaluates the correct original, first and second derivatives, and subsequent native `Dualizer` produces the correct second-order jets of all three. No FastSecDec AD is needed. |
| `InliningPolicy::Never` | Native retained bodies use a scalar `FunctionBody`, shared by `Arc` with clone-local scratch. They serialize in the native evaluator codec. | Useful ordinary evaluation support exists, but it is not a drop-in higher-jet route. |
| Native `vectorize` and `EvaluatorComposer` | `evaluate/dual.rs:127` explicitly rejects retained sub-evaluators; `evaluate/compose.rs:44` rejects them before mutating the composer. | The executable non-inline control reproduces the rejection. A new recursive AD implementation in FastSecDec is inappropriate. |
| Native function-map codec | `FunctionMap` derives the existing state-aware bincode codec; compiled inline evaluators already use the accepted exact-IR codec. | Reuse those formats and native state registration rather than inventing a second expression or evaluator serializer. |

The small executable probe is
`target/generation-agent-ltd-memory/alias-probe.rs`. At `x=2`, the explicit native
body `(x+1)^3` yields `[27,27,18]` for the function and its first two symbolic
derivatives. Dualizing that complete vector with normalized second-order jets
yields `[27,27,9,27,18,3,18,6,0]`. This establishes the chain-rule capability on a
small control; it does not establish the physical chart's memory bound.

## Recommendation and admission conditions

The actual-chart probe now supports the existing explicit native function-map
route. Represent the envelope coefficients by short native functions of the
**complete ordered chart signature**, retaining the existing visible strength
callback and its native implicit derivative hook. Let Symbolica differentiate
the coefficient calls and determinant, then register only the derivative bodies
actually requested. Construct those bodies with native differentiation and
inline them only when building the native evaluator. This avoids copying the
full coefficient bodies through every Jacobian entry and determinant term while
retaining both symbolic and numerical-dual construction. It remains a measured
candidate, not an accepted production optimization.

The ignored `target/contour-ltd-dynamic-k1/compact-probe.rs` registers four
coefficient functions with nine explicit arguments and their six coordinate
derivatives, using only native `FunctionMap` and `Atom::derivative`. Its 28
bodies total 19,543,857 bytes. The compact strength occupies 151 bytes, images
789–1,368 bytes, and Jacobian entries 5,345–8,901 bytes. The unchanged native
determinant produces a 25,586,888-byte Atom. A separate exact check substitutes
the native bodies into each compact entry and compares all 36 entries with
the original native implicit derivative; all comparisons pass in a 10.03-second
run with 480,006,144-byte peak RSS. Native evaluator lowering separately passes
with 11,763 instructions, 11.49 seconds and 602,570,752-byte peak RSS. These
measurements include artifact restoration and setup. They do not yet establish
the endpoint, higher-jet, full-density or complete-chart generation gates.

The small nested/shadowing control also passes: its four native outputs are
`[729,1458,27,0]`, and their normalized second-order dual vector is
`[729,1458,1215,1458,2430,1620,27,18,3,0,0,0]`. Mixing derivative registrations
with different function arities correctly raises the existing inconsistent tag
count error; a uniform complete signature is required within each map.

The following invariants must be verified before adopting it:

1. **Complete dependencies and argument order.** Include source coordinates,
   physical parameters and the runtime safety/displacement/strength controls in
   the function's explicit argument signature. Do not rely on captured globals.
   Test nested calls and shadowed names against native materialization. Native
   derivative-tag counts are fixed per symbol in one `FunctionMap`, so different
   derivative-function arities cannot be mixed without checking registration.
   A uniform complete chart signature avoids that ambiguity within a chart.
2. **Endpoint and higher-jet mathematics.** Native symbolic differentiation and
   series, Taylor and IBP face restriction, and native `Dualizer` must agree with
   the current materialized controls through every required derivative order.
   Restrict call arguments using the actual inherited source coordinates. Never
   treat an alias as a constant or recompute an endpoint radius in an unrelated
   lower-dimensional chart.
3. **Symmetry and identity.** Recipe-local symmetry currently compares mapped
   mathematical Atoms. Process-local function names or chart-index tags must
   not obstruct equality, introduce false symmetry or alter physical source
   identity. Content-derived names alone are insufficient evidence: coordinate
   permutation and equivalent-body controls must exercise the native comparison
   boundary. The final native compiled-content identity can describe the new
   representation, while physical input and recipe identity retain their roles.
4. **Certified checks and request metadata.** Keep the independently prepared
   exact F/U, regularity, raw bound and coefficient programs unchanged. Compact
   evaluation is not a certificate. Late stochastic request lowering must reach
   the real strength call inside each native body with its actual face bundle.
   All retained matching certificate contexts must still check the candidate.
   Hidden or unknown roots remain structural errors even with validation Off.
5. **Exact offsets and cancellations.** Exact binding evaluates aggregate
   mathematical Atoms through the native shared direct cache and certifies only
   actually executed strength keys. An opaque numeric function callback would
   hide that trace and can inhibit cross-record cancellations. Prefer bounded
   native materialization of surviving endpoint-only exact expressions before
   aggregation and request pruning; measure this rather than assuming it stays
   small. Do not replace exact aggregation with separately evaluated unit
   programs or syntactically pre-evaluate roots in dead branches.
6. **Ownership and restoration.** Generation/staging must retain the native
   function definitions and existing root helpers until compilation finishes.
   Export every referenced native symbol before state-aware decoding. Completed
   inline IR should restore without rebuilding symbolic definitions, derivatives,
   optimizers or JIT-independent mathematics. Test a fresh process after dropping
   original owners, both portable and native backends, and selected-record loads.
7. **Measured boundedness and numerical equivalence.** Compare chart construction,
   peak aggregate RSS, complete coefficient vectors and actual callback counts.
   Verify the existing determinant's removable-pivot and exact-singular controls.
   Small function-map tests do not establish a six-dimensional memory bound or
   the larger required topology's performance.

No owner patch or dependency change is justified yet: the explicit derivative
registration route is supported and its physical construction/lowering probes
pass. Integration through the remaining boundaries above is the next decision
point. The retained-subevaluator vectorization refusal is a real native API
limitation, but it is not established as required to solve this task. If it does
become necessary, prefer native instruction-body inlining before the existing
vectorizer/composer. Blindly recursing the same `Dualizer` into function bodies
would misapply outer-parameter zero-component assumptions to formal/captured
arguments, and retained bodies currently have a single scalar result. Any owner
extension needs nested scopes, lazy branches, globals, zero assumptions, higher
jets, callback identity and native-codec regressions before publication.

## Residency boundary

The current streaming discovery writes each mapped `ChartData` immediately and
returns a receipt, lookup key and compact identifiers. Symmetry comparison
restores one candidate at a time. The family coordinator retains these receipts,
and the archive writer copies bounded record chunks while retaining compact
catalogue entries. New coefficient definitions must follow this same record-local
ownership. A global chart-definition or expanded-Jacobian cache would defeat the
existing bound even if the individual chart now fits memory. The 25.6-MB raw
determinant must not be copied into coordinator status, receipts or JSON previews.

Explicit resident selection has a different existing contract:
`ProgramResidentAssembly` retains all selected numerical objects/metadata and
their selected-only portable archive bytes. Its memory can grow with the selected
recipe. This is not the serial path's per-unit bound. The new representation must
preserve selective inspection without automatically expanding function bodies;
large ordinary resident requests must be reported as such, not presented as
constant-memory generation.

## Independent physical variance cross-check

The separate physical variance evidence does not validate the LTD representation
change. A read-only audit of
`target/generation-agent-physical-variance/probe.rs`, native democratic
`QmcSession::estimate`, and all 48 stored results confirms actual transformed
coordinate/weight hashing at the numerical callback, matching 64 task hashes
within each pair, eight distinct coordinate sets and 3,145,728 total sector
points. The finite variance is the trace of the native complete Laurent-vector
covariance's finite block, with full off-diagonal covariance preserved.

Independent recomputation reproduces every accepted paired ratio and summary:
F8/Psmall median variance ratio 1.084256409 and median time-normalized ratio
0.866681333. All default-cap gains are null, with failed oracle pairs 1 and 3
retained. Polynomial/sign-aware means and whole covariance matrices coincide
at each cap/seed; the review correctly treats these quadratic prescriptions as
one mathematical case, not independent evidence for a gain. Source/report/map
proof SHA-256 values match their ledger. The detailed results and unfavorable
default-cap evidence remain in `contour-physical-variance.md`, owned separately.
