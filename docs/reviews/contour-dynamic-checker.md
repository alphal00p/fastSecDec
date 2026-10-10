# Dynamic admission and certified-check contract

2026-10-09. Next implementation contract following the native v10 storage gate.
Dynamic production remains explicitly unavailable until these boundaries and
their scientific controls pass. This document changes no numerical code.

## Existing essential admission

`SectorKernel::evaluate_scaled_inner` already rejects nonfinite coordinates and
coordinates outside the closed unit cube, independently of contour validation.
The weighted batch entry point validates its inputs before its native batch
call. Reuse these boundaries: the nonnegative weights x(1-x) are a mathematical
condition, not an optional diagnostic. Pilot entry points independently admit
their caller-supplied points. Zero-dimensional exact charts satisfy the cube
condition without inventing a stochastic sample.

Keep full chart dimensions, F/U structural order sets, dense coefficient arity,
regularity and caps unchanged on subtraction faces. Use the recorded actual
zero/one face requests; do not derive a different radius from a restricted F.
Before admitting a dynamic payload, bind every descriptor to its actual retained
chart, dimension, causal/positive factors and checker output schema. Reject
missing, duplicate, out-of-range or inconsistent chart/check associations before
numerical evaluator construction. Native v9 remains a separate explicit layout.

The current native envelope builder proves U positivity through exact rational
nonnegative polynomial coefficients and a strictly positive constant lower
bound. This is independent of the real kinematic point. Its recorded proof must
belong to the actual retained U and must not claim to cover an unproved symbolic
parameter-dependent coefficient. Rebinding validates finite real physics inputs
and the existing physical mass constraints. Any future parameter-dependent U
certificate must be evaluated and certified for that binding, not inferred from
a positive numerical value at one coordinate or repaired with a positive floor.

## Prepared native programs

Original generation prepares and saves these programs, sharing their underlying
Symbolica expressions and optimization where measured beneficial:

- Production: complete smooth density, local strength, Jacobian and required
  subtraction jets, lowered through native derivative hooks and dual machinery.
- Coefficient-only root helper: the existing native H-1/H_u evaluator, with
  dense coefficients as inputs. Restore its saved codec before callback binding.
- Polynomial validation source: direction norm squared, each B_k, each original
  U and even ray coefficient d_k. Store the explicit output layout and local
  chart association with the program.
- Sign-aware validation source: additionally the weighted spectral mean and
  squared gap for each odd order, plus harmful normalized even U coefficients.
  These outputs contain polynomial/rational arithmetic, not the rounded
  production positive-part callback.
- Causal-factor validation: the original F/U evaluated on the same full-sector
  ray with an independent strength input. This permits certified strength
  intervals without evaluating a root callback in the ball domain.

The sign-aware spectral upper bound already uses sqrt(gap_squared + delta^2).
The positive-part smoothing alone would not remove the matrix-degeneracy cusp;
the existing inner delta^2 is essential and its derivatives must remain present.

Only native certifying arithmetic instructions are admitted to the ball-mapped
programs, recursively including alias bodies. Their output schema is recorded at
generation; restoration performs no envelope reconstruction or Horner/CPE.
Prepared precision workspaces belong to the resident kernel and have bounded
lifetimes. Unknown/unsupported certificate capability must remain an explicit
enabled-validation error; it cannot secretly disable the requested policy.

## Independent positive-radius enclosure

Let H(t)=sum_p a_p t^p, with a_p nonnegative, p at least two and a_2 at least one,
and let r be its unique positive root H(r)=1. A positive candidate x from the
native prepared solver provides a certified enclosure using rational arithmetic
alone; the solver's floating bracket is not itself a certificate.

Write h=H(x). If h>1, then r<x. For q=r/x<1,
1=H(r)<=h q^2<=h q, so r>=x/h. If 0<h<1, then r>x and q>1 gives
1=H(r)>=h q^2>=h q, hence r<=x/h. For h=1, x=r. Thus r lies between x and x/h
in all cases. This bound is less tight than x/sqrt(h), but it requires no
uncertified square-root operation and becomes tight near the native root.

With a certified evaluation H(x) in [h_lower,h_upper], where h_lower>0, use

    [min(x, x/h_upper), max(x, x/h_lower)]

with native outward arithmetic. The min/max operations select diagnostic
interval endpoints; they do not change the smooth mathematical radius function.
Escalate native precision if the enclosure is too wide or a required sign is
unresolved. Never clamp a coefficient, alter S or substitute a zero strength.
The radius and coefficient enclosures must come from the independent saved
validation source, not rounded values reported by the production callback.

The isolated `target/contour-radius-enclosure-probe.rs` passes 2,620 controls with
known exact Rational roots. It uses one native prepared Symbolica ball evaluator
for changing positive sextic coefficients, precisions 8/24/96/256, candidates on
both sides of the root and coefficient-zero cases. The oracle checks exact
rational interval endpoints, not an approximate second root solve. No RNG or
FastSecDec replacement polynomial/root routine is used in the probe.

For causality, the remaining safety margin is meaningful: H(S*r)<=S^2<1.
Certify the prescribed strength interval and relevant F/U sign inequalities.
A stationary nonzero F retains its causal lower-lip continuation; F=0 together
with a vanishing weighted gradient remains an unresolved-deformation error.
With optional validation disabled, diagnose this condition through retained
causal factors on the essential nonfinite/failure path, without adding hidden
successful-sample checks. Exact subtraction faces where an apparent zero
cancels from the complete density require a separate mathematical audit.
Neither a zero complex Jacobian alone nor a sampled zero contribution proves a
pole crossing or an exact zero integral.

## Certified square roots and the stable positive part

The public/source/probe audit found that the former native RealBall::sqrt did
not certify its rounded endpoint operations. The narrow owner correction is
now included in the selected public Symbolica/Numerica source through
[PR #58](https://github.com/symbolica-dev/symbolica/pull/58), with native and
portable exact-rational endpoint tests. FastSecDec reuses that certified native
operation and does not implement a square-root or interval-number type.

The tested owner construction takes any positive finite native approximation
g of sqrt(a). The exact root lies between g and a/g; native directed division
therefore supplies enclosing endpoint bounds. Apply this to the original lower
and upper endpoints, retaining exact-zero endpoints and the existing negative
domain error. This uses the owner numeric primitives and has no new root solver.

Production positive-part evaluation uses the native hypot primitive and the
mathematically identical rationalized negative-argument form. Its hooks are
mu_t=mu/hypot(t,delta) and mu_delta=delta/(2*hypot(t,delta)); Symbolica owns higher
derivatives. The accepted narrow tracked-real hypot patch preserves uncertainty
without changing scalar guards. Complex hypot remains a holomorphic expression,
not a Hermitian norm. Extreme complex intermediate overflow must trigger an
explicit native multiprecision rescue initially, with separate measurements;
never replace the function by a nonholomorphic norm or erase imaginary error.

Include t approximately -1e308 and delta=1e-3: the literal denominator
2*(hypot(t,delta)-t) overflows although mu is about 2.5e-315 and representable.
The equivalent scaled ratio `(delta/2)*((delta/2)/(h/2-t/2))` avoids that
denominator overflow. A nonfinite native complex hypot still requires explicit
rescue, and a finite zero must not silently replace the strictly positive mu.
A real-native fast path is valid for real inputs; tracked complex inputs can
use it only if imaginary components are exact zero including their uncertainty
metadata. Otherwise preserve the native complex expression and its error data.

## Failure, policy and statistical boundaries

Each attempted evaluation domain clears its own prior callback failure state.
A later successful callback in the same failed attempt cannot erase an earlier
failure. An f64 failure must not contaminate an independently successful DD/Float
rescue. Solver failure, nonfinite values and nonpositive/underflowed physical
strength remain errors with validation off. Precision escalation and diagnostics
must identify the failed contour request without fabricated samples.

Always, pilot and off retain their established caller-owned preflight semantics.
Independent numerical checks have their own preparation and enclosure cost;
report it separately from production roots and measure it explicitly. They
certify the actual production candidate without solving a second radius. Production
Laurent outputs and jets must share one scalar root per distinct coordinate/face
request through native CPE. The unchecked path must perform no ball mapping,
certificate evaluation or hidden validation root solve.

The certified checks enclose the mathematical contour at the supplied binary
input values. They must not be described as certificates for every separately
rounded JIT intermediate. Validation policy/provenance remains outside the
mathematical checkpoint identity. No root, deformation, precision rescue or
certificate computation consumes production RNG state.

The one-shot exact-offset binding path now supplies scoped callback preparation
precision around native `Atom::evaluate_with_prec`. Its whole-vector f64/DD/Float
attempts isolate failure state and preserve outer scopes. The forced-MP and mixed
callback/native-error regressions pass, as recorded below. This prepares the
lifecycle boundary; dynamic exact-offset admission still requires the saved
checker and all remaining scientific gates.

## Next focused gates

Test descriptor/payload association corruption; zero-dimensional exact offsets;
fixed full-sector schemas on zero/one faces; native root enclosures against
independent high-precision roots; coefficient-zero and near-unit-S limits;
spectral degeneracy smoothness; tracked imaginary uncertainty; and explicit
nonfinite/underflow failures with checks disabled. Repeat the established
eager/SymJIT batch, fresh-process codec and ordinary/serial scientific controls.
Then compare always/pilot/off using identical production coordinates and verify
unchanged sampling-stream identities and true removal of unchecked overhead.

## Executed candidate-association refinement

The following refinement retains the admission and source-contract obligations
above. Its request lookup and saved-check files remain unwired at this milestone.

### Certify the candidate actually used

The checker must certify the physical strength returned by the production
callback. Solving a second radius equation in the checker would both add cost
and validate a different rounded value. For a captured positive strength
`lambda`, certified native ball evaluation must establish `H(lambda/L) < 1`.
This uses the safety-fraction slack; it does not incorrectly require a rounded
root to satisfy `H(u)=1` exactly.

Separately evaluate `H(lambda/(S*L))` and enclose the intended positive root.
For nonnegative coefficients and powers at least two, the true root is between
`x` and `x/H(x)`: if `H(x)>1`, scaling by `1/H(x)` decreases every term at least
quadratically; if `H(x)<1`, that scaling increases every term at least
quadratically. Certified interval endpoints therefore give
`[min(x,x/H_upper), max(x,x/H_lower)]` when `H_lower>0`. Check the width against
the selected numerical accuracy, so a safe but wrongly solved strength is not
accepted. This enclosure is independent of the prepared solver's numerical
bracket. It is not a second root solve.

Capture exact represented centres through native arithmetic. In particular,
double-double conversion must retain both `into_inner().hi()` and `.lo()` as
exact native rationals; rounding the candidate to f64, or assuming that a
106-bit Float conversion preserves an arbitrarily separated low component,
would weaken the certificate. Tracked arithmetic remains heuristic error
tracking; the independent checker certifies the actual supplied centre.

Saved native programs evaluate raw polynomial primitives and assemble the dense
coefficient vector from independent primitive slots. Symbolica owns both H
construction and coefficient collection. Native certified ball arithmetic and
the reviewed native square root enclose the sign-aware positive parts. No
FastSecDec polynomial convolution, alternate AD, root solver or square root is
needed. Optional checks never rebuild an envelope or rerun Horner/CPE on reload.

### Diagnostic identity must not change symbolic mathematics

Adding chart/coordinate tags before exact symmetry comparison changes native
expression identity and can destroy valid sector reuse. Moving them merely
after symmetry is also insufficient: a root independent of x cancels exactly
against its x=0 restriction, but different coordinate tags prevent that native
cancellation and can fabricate a stochastic term or extra dimension.

The symbolic route must therefore retain its original mathematical callback
through symmetry, subtraction, Laurent construction and exact folding. Only
original evaluator lowering adds diagnostic metadata. Build an exact native
lookup from the retained full-strength Atom restricted on the actual required
faces; associate every surviving callback Atom by native Atom equality. Equal
restricted root Atoms share a candidate and a set of associated faces. Missing
associations are explicit generation errors, not omitted check coverage.

The numerical-dual route can specialize the same face-set metadata immediately
before native jet construction. Its mathematical body keeps the full variables
until native input seeds apply the face. Metadata receives zero derivative hooks;
the physical coefficients and their complete native chain rule remain intact.

Request namespaces bind ordered coordinates, exact native-canonical F/U
identities, recipe version and full-sector structural data. They never use
process-local State serialization, optimized native byte digests, or a mutable
record-local chart index. Record-local projections remain separate. Selected
nonzero charts, merged records, exact-only records and representative
multiplicities need explicit association regressions before admission.

### Executed focused probe

`target/contour-request-tag-probe/` imports the actual prepared root workspace,
solver and implicit derivative code, with an isolated third-tag wrapper. It
passes scalar and cubic symbolic/native-dual derivative comparisons against the
independent native quartic closed form, including x=0 and x=1. It demonstrates
the premature-tag cancellation failure above, verifies zero classification when
tags are added after simplification, and counts exactly one scalar root call for
a complete cubic symbolic jet vector with S and L retained as runtime inputs.

This probe validates the native mechanism. It does not yet establish all
production coefficient aliases, merged-record associations, numerical-dual
composition variants, or dynamic sampling performance.

### Optional capture and failure lifetime

Preparation selects separate plain and checked native callback closures. Plain
Off and post-pilot production closures perform no observation or certification.
Later precision/conditioning remaps must retain that selected mode and its
exact owner scope. Checked evaluation may initially use scalar calls behind the
batch API to associate each candidate unambiguously with one accepted output;
report its cost separately from unchecked batched evaluation.

Per-attempt observations are bounded by admitted requests and reset on retries.
Unknown, duplicate or incomplete request coverage must fail explicitly. A failed
attempt cannot contribute observations to a later successful precision attempt.
Nested scopes and unwinding must restore the outer owner and diagnostics.

The exact-offset lifecycle correction is already implemented and tested: native
direct evaluation enters the actual mapping precision, isolates callback
failures for each retry, and preserves an enclosing attempt's failure. The
forced-MP control overflows a2=p² in f64 and double-double at p=1e200, then
recovers lambda approximately 8e-201. A mixed callback failure followed by a
native evaluation error still retries rather than returning prematurely.
`kernel::exact` passes all three controls; the combined contour filter passes
71 controls at this boundary.

### Explicit new storage boundary

Preserve published v10 payloads and descriptor-v1 layouts exactly. New saved
checker programs, factor associations and request projections require a v11
wrapper and descriptor v2. The generation-only DTO containing the full-strength
Atom and primitive-combiner source uses staging schema3. Old dynamic v10
foundation artifacts remain explicitly unavailable for production; fixed and
undeformed legacy payloads retain their supported readers.

## Schema-three source admission and independent candidate probes

The generation-only source now retains the complete mathematical strength
callback, canonical identities for the ordered original F/U factors, an
immutable request namespace, the independent raw polynomial outputs, and a
Symbolica-generated primitive-to-coefficient program source. The namespace
binds coordinates, recipe, structural orders, exact positive lower bounds and
regularity; record-local chart indices are excluded. Admission compares actual
retained F/U identities, helper identity and complete schemas, including a
regression that swaps same-schema factors. The native staging schema explicitly
advances to three. Its fresh-worker test caught and corrected an omitted export
of the newly introduced primitive symbols; no process-local fallback is used.

Focused registered gates passed: generation controls 3/3, streaming 16/16
(including fresh processes for polynomial and sign-aware recipes), and recipe
admission 10/10. These validate source transfer, not production admission.

Separate ignored direct-rustc probes execute the proposed actual-source
candidate and radius modules. Three ball/rational tests distinguish sufficient
causal slack from intended-root accuracy, exercise precision-dependent slack,
and enclose known roots of positive mixtures of powers two, four and eight.
Three candidate tests preserve both double-float limbs exactly, reject missing
or unknown observations, and restore nested contexts after retries/unwinding.
These modules remain unwired while the saved-program and runtime integration
is completed. Their strict duplicate-callback guard is a proposed execution
invariant, not a statement about statistical sampling. Before adopting it,
whole Laurent vectors, actual Dualizer jets, aliases, both evaluators and
precision remaps must demonstrate that native sharing executes each required
root once; otherwise every actual occurrence needs explicit certification.

The planned v11 descriptor separates local chart coverage from self-contained
certificate contexts. Exact records retain only contexts referenced by surviving
exact root requests, with an optional local chart projection. This preserves
the proof when its originating full chart is absent from that record without
retaining every source checker in the serial coordinator. V10 remains a distinct
legacy layout. Neither that wire format nor dynamic production admission is
claimed complete by the source-transfer gates above.

## Actual candidate and compiled checker boundary (2026-10-10)

The late requested callback now passes the complete native Laurent-vector
execution controls for both dynamic constructions and Taylor/IBP subtraction,
including actual numerical-dual jets, eager and SymJIT owners, double-float and
192-bit remapping. The native-dual request includes its actual restricted face:
Taylor has two distinct requests and IBP has three in this control. Symbolic
lowering retains merged face bundles where native mathematical equality shares
the radius. The strict duplicate guard therefore concerns the same actual
request, rather than incorrectly conflating equal radii on distinct IBP faces.

The saved independent arithmetic checker is registered. Eight focused native
checks pass: native outward primitive operations, root-enclosure inequalities,
actual production candidates for both constructions, rejection of a safe but
incorrectly solved strength, binding mismatches, safety fractions immediately
below one, and caps of order 1e±100. The separate recipe admission filter passes
16 tests with one child entry intentionally ignored. These are native component
gates; public dynamic production remains rejected pending the per-attempt
consumer, binding/pilot lifecycle and complete exact-offset integration.

A reduced-dimensional numerical program can omit a full-source coordinate even
while an equal radius request refers to a face on which that coordinate remains
free. The checker represents such missing coordinates by the native closed
interval [0,1]. It must prove the candidate uniformly over the interval, with
recorded fixed-face coordinates overriding the interval exactly. No coordinate
zero, numerical floor, or separate dependency algebra is substituted into the
production map. Insufficient enclosure remains an explicit unresolved check.

### Exact vectors preserve native cancellation and conditional execution

Separate saved per-unit exact-vector evaluators are not acceptable. A native
probe with A=[1/p,1/p+x], B=[-1/p,-1/p+2x], p=0 and x=2 produces nonfinite values
when the separate numerical vectors are summed, whereas native Atom aggregation
first produces [0,6]. A native evaluator composer also retains the singular
intermediate and does not replace algebraic simplification. This counterexample
rules out both the per-unit saved-vector proposal and aggregate-literal-zero
masking as a complete correction.

Exact mathematical Atoms consequently stay untagged through native aggregate
simplification. V11 stores the native root-Atom-to-request-context association
separately. Native equality merges its contexts and removes associations whose
roots actually cancel. A focused probe of Symbolica's public full-function
value map confirms shared root reuse across outputs in seven numerical domains,
including tracked complex uncertainty. Eagerly precomputing every syntactic
root would, however, violate native lazy IF semantics. The narrowly scoped
owner API proposed in Symbolica PR59 shares the existing direct-evaluation cache
across a complete Atom vector and exposes only actually executed custom function
values. Its consuming revision and exact-offset runtime integration are still
pending. Untaken branches and canceled roots must not be converted into missing
candidate errors by the stochastic strict-coverage observer.

The first per-attempt validation test build caught a fixture using original
integrand symbols instead of generated target-coordinate symbols. The fixture
now reads the retained native map and its actual full-source coordinate schema;
its test permutation does not assume source and target axis order agree. The
reduced-coordinate test intentionally prepares a numerical owner with an
irrelevant source axis omitted. It is an owner-projection control, not a claim
that the simple generation fixture currently eliminates that stochastic axis.

### Native attempt owners and public lifecycle wiring (2026-10-10)

The public combined Symbolica/Numerica revision `1e1cb169` now supplies PR59's
direct vector cache. The first consuming runtime gate passed 14 dynamic tests,
including complete checked Laurent vectors through eager and SymJIT numerical
owners, scalar-per-row checked batches, DD/Float remapping and tracked
conditioning. Eight of nine exact-vector tests passed; the remaining test
exposed a sum still containing an undistributed numeric minus sign. Native
`expand_num` normalization at the aggregation boundary is being checked before
accepting that cancellation control. No result from that failed test is counted
as evidence of cancellation support.

The next lifecycle slice prepares all numerical replacements and the complete
accepted exact vector before committing a physical rebind. Policy remapping
uses saved native IR and preserves accumulated production check counts.
Unchecked dynamic callbacks keep essential failure fencing; ordinary/fixed
owners return immediately from the dynamic reset path. Production check counts
aggregate primary, DD, Float and conditioning owners, including failed argument
certificates encountered during rescue. Pilot owners are private and independent
of these counters, retained only for currently resident sectors and released
when their required contexts finish. Pilot `checked_arguments` means successful
certified context/face coverage; it does not count every failed intermediate
precision attempt. The elapsed pilot cost still includes all such work.

Restoration of complete v11 certificate payloads is now admitted; v10 dynamic
payloads still require regeneration. The public dynamic binding guard remains
closed while atomic binding, actual pilot execution, policy transitions and
restored public analytic controls are being tested. This is an implementation
status update, not a claim that dynamic production is accepted.

### Public execution exposes repeated native calls (2026-10-10)

The lifecycle gate now passes 18 tests, including restored eager/SymJIT owners,
actual pilot execution, atomic failed exact-offset rebind, policy transitions,
private pilot-cache lifetime and cumulative numerical check reporting. The
native exact normalization correction passes all 10 exact tests; the same
compiled snapshot passes 17 recipe tests and 45 artifact tests (one subprocess
entry is intentionally ignored in each latter filter).

Public dynamic binding was enabled locally to run the scientific acceptance
controls. The first public run passed the independent analytic recurrence but
both endpoint and threshold controls failed during pilot execution: native direct
translation retained repeated calls for the same radius. An independent native
IR probe found four identical calls for the symbolic threshold bubble with
direct translation enabled, versus one with the tree optimizer. This is valid
numerical IR; unique callback execution cannot be a correctness precondition.
It remains a separate optimization obligation for the one-solve cost target.

The bounded observer now coalesces only an exactly equal rational candidate at
the identical native precision within one admitted request bundle. Each native
call still computes and returns its own value, including tracked uncertainty;
no numerical result is cached or substituted by FastSecDec. The independent
checker certifies that identical actual value against every associated context
and face. A different value or precision fails explicitly, and unknown and
missing request checks remain. The public scientific rerun is pending; dynamic
production acceptance is not claimed by the lifecycle or observation changes.

The next public run passed the threshold-bubble policy/branch control and the
analytic recurrence, then exposed a distinct endpoint issue at x=17/16384 in
the third-order endpoint-pole control. Its distance-selected native Float owner
uses 3,322 bits, while the root callback still allowed only 256 iterations for
a strict relative bracket tolerance near 10^-999. The actual coefficient is
a2=1+4*x^2*(1-x)^2. A focused native Symbolica evaluator/solver probe reproduces
the failure and reaches strict bracket termination in 3,300 iterations when
given sufficient work; this is a FastSecDec configuration mismatch, not evidence
of a solver defect.

The structural one-coefficient helper now uses the native positive solution
1/sqrt(a2), already used as the previous initial guess. Its existing native
complex correction remains responsible for zero-centred imaginary uncertainty.
No higher coefficient is discarded based on its rounded centre. General helpers
retain the same strict convergence rule with a finite max(256,4*bits) work
allowance; this is explicitly not a universal convergence bound, and exhaustion
still fails. Independent native probes cover 53,106,192,3322 and4096 bits with
zero and nonzero quartic coefficients. Consuming tests and the public rerun are
pending at this update.

The consuming root/callback filter now passes 23 tests, including the new
native-domain square-root and 3322/4096-bit refinement controls. Empty and
zero-dimensional generation passes four tests, including refusal to reinterpret
a singular denominator with a zero prefactor as a zero integral. The complete
public analytic target passes all three functions in 1.75 seconds: eight
endpoint cases retain their original 8192-point quadrature and tolerances, and
the threshold bubble exercises restoration, actual pilots, validation policy
changes and the stationary negative-F branch. Paired variance and full milestone
lint/ecosystem gates remain separate requirements.

### Terminal failure context (2026-10-10)

The dynamic lifecycle/checker filter now passes 20 tests, including two new
failure controls. A genuine stationary causal zero reports certified retained
context evidence for F=0 and the weighted-gradient magnitude zero; the same
factor canceled from the complete density remains finite and nonzero. The
diagnostic explicitly does not identify a unique failing density branch or
classify a Landau pinch. A root-free exact failure retains its native nonfinite
reason without inventing a stationary-zero certificate, and failed rebinding
preserves the previous accepted exact vector.

Each detached sector retains only the raw saved certificate bytes and compact
source projection/runtime context for its own requests. Binding prepares that
immutable context before the atomic commit. Decoding, native ball mapping and
stationary testing occur only after terminal numerical failure; successful Off
samples perform none of that work. Missing source axes retain the full [0,1]
enclosure. Independent source review confirms this ownership and failure-only
boundary. The four detailed radius-proof bounds remain available in tests;
production acceptance performs the same inequalities without retaining unused
diagnostic conversions.

The evidence is `target/contour-dynamic-failure-context-tests.log` (20/20,
0.26 seconds). The next owner dependency transition and full native, CLI,
portable and lint gates are separate acceptance steps; this focused result
does not establish WASM execution or overall Phase B completion.

The subsequent portable host gate on public Symbolica/Numerica `516beb37`
passes all 76 maintained tests, including the same three public dynamic analytic
controls (5.83 seconds) with unchanged quadrature and tolerances. It uses the
existing Malachite/Astro/eager consumer and no alternate numerical implementation.
The manifest, lockfile, wrapper and shared scientific source hashes remained
unchanged throughout the gate. Evidence is
`target/contour-dynamic-portable-owner516-tests.log`. Actual Emscripten execution
was checked separately using the existing Rust 1.98.0 / Emscripten 5.0.3 /
Node 24.19.0 installation. The unchanged three-test public wrapper cross-compiled
successfully in 5 minutes 44 seconds and all three tests passed in actual WASM
execution under Node (2.03 seconds). This includes complete endpoint vectors,
saved-program restoration, actual pilot certification and validation-policy
transitions. No alternate test implementation, source adaptation or new toolchain
was introduced. The build and execution logs are
`target/contour-dynamic-wasm-owner516-build.log`, its adjacent Cargo JSON artifact
record, and `target/contour-dynamic-wasm-owner516-tests.log`. These are acceptance
test durations, not a host/WASM performance comparison. Node execution does not
establish browser responsiveness or a Pyodide/Python wheel.
