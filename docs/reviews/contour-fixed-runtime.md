# Fixed contour runtime and HEPKit binding audit

2026-10-09. This records the fixed-mode foundation, not completion of
[Phase B](../../CONTOUR_DEFORMATION_PLAN.md). Dynamic production deformation
and the complete physical multiloop suite remain pending.

## Native reuse and branch semantics

The lower-lip logarithm is one registered Symbolica function with a native
derivative hook, `d log(z)/dz = 1/z`. Symbolica supplies higher derivatives,
dual domains, expression optimization, native evaluator serialization and
fresh-process callback restoration. Its native logarithm also supplies the
real part and tracked uncertainty: changing the sign of its existing phase
does not create an untracked exact floating constant. Negative-real arguments
with zero imaginary part select the causal lower lip. Wrong-side nonzero
imaginary arguments are not silently reflected.

Fixed strength is a reserved runtime input, bound separately from physical
parameters. Rebinding requires neither symbolic generation nor Horner/CPE
optimization. Native content identity includes the bound strength and
physical point; the optional validation policy remains separate. The
undeformed path contains no contour callback or checker.

Native artifact version 9 stores retained maps, Jacobians, causal and positive
factors, actual subtraction-face requests, and optimized arithmetic check
programs. Previous binary layouts have explicit legacy structs. Selected
indexed loading decodes only its own records and remaps validation chart IDs
alongside inspection chart IDs; it does not reconstruct maps from source
expressions. An exact-only zero record may have no checks only when every
exact expression is identically zero. Compact aggregated exact offsets are
checked through their individual records before metadata is discarded.

## Certified arithmetic and its limits

The checker admits addition, multiplication, assignment, integer powers and
recursively admitted native aliases. It rejects noninteger powers, external
functions, unresolved constant functions and conditional/control-flow
instructions. The whitelist is exhaustive, so a new Symbolica instruction
requires an explicit admission decision. This is numerical-domain admission;
it is not another general native-IR validator.

The tiny owner-library patch adds Symbolica evaluation domains for Numerica
`RealBall` and `ComplexBall`. It does not certify their inherited
transcendental methods or replace unsupported constants with floating-point
centres. Admitted rational arithmetic is evaluated at increasing precision
when an enclosure cannot decide a sign. Prepared ball evaluators are cached
per resident precision rather than remapped at every ordinary sample.

The enclosure certifies the symbolic contour map at the supplied binary
floating inputs. It does **not** certify every rounded intermediate in the
compiled integration kernel. Fixed preflight checks at several homotopy
fractions remain finite diagnostics, not a certificate for unsampled points
or the complete homotopy. These distinctions are documented on the native
API and retained in status descriptions.

Undeformed positive factors are checked before their deformed values. A
stationary negative nonzero causal factor is valid with its lower-lip branch;
an unresolved stationary zero is an error. A vanishing Jacobian alone is not
a crossing criterion. Every subtraction face restricts the same full-sector
map, including the actual zero/one boundary requests. Taylor does not acquire
unused IBP upper faces.

## Caller-owned validation and diagnostics

Both `always` and `pilot` require explicit preflight completion before native
sampling. Only successful full-homotopy checks contribute pilot coverage.
Scoped completion unlocks only the requested chart owners; it never unlocks
an excluded sector. Native callers provide coordinates and steer all loops.
The CLI uses a distinct validation stream without touching production RNG
frontiers.

`off` does not decode or evaluate optional check programs. Unsupported
certified arithmetic remains a usable unchecked artifact, and excluded
unsupported charts do not block a selected valid scope. Mathematical changes
invalidate readiness. A policy-only change preserves the mathematical ID and
existing evidence.

Resident production counters are drained at caller-owned batch boundaries.
They are separate from pilot observations, and draining never resets sampling
state or preflight readiness. Adaptation and production diagnostics remain
separate. Checkpoint provenance retains prior pilot evidence on an unchecked
resume, without granting permission to bypass a newly required pilot.

## Thin HEPKit interface

FastSecDec's Rust binding exposes contour generation on `Integral.generate`,
the retained generation session and existing HEPKit decomposition forwarding.
Native `ContourSettings`, chart descriptions, validation/check reports,
diagnostics and checkpoint provenance have immutable Python views. Physical
runtime parameter listings omit the reserved strength input.

The explicit pilot methods mirror native calls. They contain no hidden
integration loop, sampler, thread pool or Python algebra. A caller must finish
the pilot before creating an integration session. Caller-supplied pilot points
have a named provenance protocol and no invented random seed. Existing
community registration and wildcard reexports expose these bindings without
moving substantive implementation into community.

## Validation evidence

All commands below use the two independently tested local owner fixes:

- Symbolica ball-domain commit
  `3db1607f5acd9669cde747aa048a3ca0c0fcb2e1` in the reference worktree.
- SymJIT 2.27.0 with the owner's complex callback SIMD lane adapter fix.
  The unpatched release produced different values for scalar and batched
  evaluations at identical points; the owner regression and 2,148 owner tests
  pass with the fix.

These local dependencies are temporary validation inputs, not permission to
publish file URLs or build-tree paths in Cargo manifests/lockfiles.

| Gate | Result |
| --- | --- |
| `cargo test -p fastsecdec --lib contour` | 28 passed, including lower-lip/error-tracking, eager/native/batched agreement, exact/scoped pilot readiness, reload/fresh-process restoration, finite homotopy crossing and certified arithmetic admission. |
| Native artifact test filter | 25 passed, including retained metadata, selective loading, corruption and legacy layouts. |
| Physical HEPKit B0, C0 and D0 | Root/generation agent report agreement above threshold in symbolic and numerical-dual generation. |
| Real CLI fixed matrix | Root reports both generation methods, both generation schedules, both integration schedules and all three validation policies, plus policy-only resume and selected-scope failures. |
| CLI unit contour filter | 7 passed, including production/adaptation counter draining. |
| Real CLI benchmark/check-boundaries contour regression | Passed for `always`, `pilot` and `off`, with diagnostic counters separate from production statistics. |
| Native binding `cargo check --all-targets --features python_stubgen` | Passed. |
| Portable host binding `cargo check --all-targets --no-default-features --features portable,python_stubgen` | Passed. |

The new Python scientific tests have been added but have not yet run against
a freshly built community extension. The portable host check does not stand
in for an actual WASM/browser run. Independent runtime/performance and full
Phase B acceptance gates remain required.

## Pre-commit runtime audit and exact-contribution admission

2026-10-09. The final source audit rechecked optional validation, factorwise
causal logarithms, parameter binding, selected scopes and saved-program reloads.
No additional fixed-mode branch or validation-cost blocker was found. Fixed
sampled homotopy checks remain safeguards, not a global certificate; the ball
checks enclose the symbolic map at supplied binary inputs, not every rounded
production-evaluator intermediate.

The audit did find a readiness gap for exact-only Python integrations. Sampling
was already gated on completed `always`/`pilot` preflight, but an exact-only
session has no sampling call. Its observation could therefore be constructed
before that gate. CLI execution already performs explicit exact-record pilots.

The correction adds scope-aware native `KernelSet::validate_integration_readiness`
and the checked `KernelResultManifest::integration_problem_from_kernels` entry
point. Python QMC and Havana creation/restoration share that native entry point.
Exact charts require readiness only when the scope includes exact contributions;
excluded stochastic charts remain excluded. `off` remains an explicit opt-out.
Bare coefficient getters and manifest construction retain their documented
inspection/caller-declaration role, without conferring runtime readiness.

A new native regression covers missing binding, artifact reload, exact-only
`always`/`pilot` rejection and acceptance, exact inclusion/exclusion, mathematical
rebinding and `off`. The selected-sector regression now exercises the checked
problem constructor. New Python tests cover both integration lanes, both enabled
policies, exact-only creation and checkpoint restoration. Rust formatting, diff
checks, Python AST validation and independent source review pass. Root reports the corrected bindings compile successfully, the updated native
library gate passes 252 tests (16 ignored, zero failures), and strict workspace
Clippy passes. The new exact-only and scoped-readiness regressions are included.
Python runtime tests require the refreshed isolated extension and are not
inferred from compilation.
