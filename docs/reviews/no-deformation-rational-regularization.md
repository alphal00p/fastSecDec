# Rational one-dimensional endpoint bridge

This milestone adds a restricted endpoint-normalization bridge for verified
one-dimensional rational fibers. It does not implement the general algebraic
resolver, a uniform parameter atlas, auxiliary-regulator removal, or completed
threshold artifact publication.

The native source must be a unit interval or the verified two-coordinate affine
projective gauge. Finite linear GCAD sections are ordered at an admitted exact
rational parameter point and checked to partition the whole interval without
interior gaps or overlaps. Each interval is split into two half charts. Reflection
retains its orientation while integration uses the positive half-width measure.
The initial implementation requires all retained cells to share one admitted
parameter chamber; it does not mistake arbitrary admission failures for cells
to discard.

For each original signed factor, Symbolica computes the exact pulled-back
polynomial valuation and monomial quotient. Native symGCAD Sturm counting and
strictly positive endpoint values establish that the residual is nonzero and
positive on the entire closed half interval. Multiplicities multiply the full
regulator-dependent exponent. The original whole-factor causal phase remains
attached to its density term. Original monomial powers, projective normalization
and the positive chart measure enter exactly once. An exponent that is exactly
zero has no unit obligation; the source layer continues to reject an identically
zero declared singularity.

A private `RegularizedFiber` is constructed only after all chart powers admit a
common nonempty open epsilon convergence strip. This precedes seam omission and
symbolic continuation. Complex numerators are restricted to epsilon-independent
polynomials with native exact complex-rational coefficients at the admitted
fiber. Prefactors initially have the same exact constant coefficient scope;
coordinate independence alone is not a meromorphy certificate. Float, infinity,
finite-field and other generic numeric Atom classes are not silently admitted.
Supporting ordinary finite floating-point source coefficients and native
meromorphic physics prefactors remains subsequent work.

The subsequent [meromorphic-prefactor milestone](no-deformation-meromorphic-prefactors.md)
extends the prefactor scope to exact rational/Gamma families and tests the
actual graph normalization. The epsilon-independent exact-polynomial numerator
and fixed-fiber limitations remain at this checkpoint.

The bridge delegates subtraction to the existing symbolic Taylor/IBP engine.
It preserves full phases, boundary denominators, endpoint profiles and the
complete cell sum. Symbolica differentiates composed native numerator calls;
there is no endpoint AD or second chain-rule implementation. Current native
`FunctionMap` uses `Always` inlining because the independently recorded
`DERIVATIVE`/`Never` owner issue is not in this dependency revision.

`ContinuedFiber` borrows its immutable certificate. Its low-level expressions
and function map do not independently carry a runtime admission guard.
`bind_fiber` constructs an opaque `BoundContinuation`: it binds the proved
physical parameters in both expression roots and every numerator derivative
body, refuses surviving physical parameters, retains the original ordered
bindings and certificate, and exposes no free rebind. The remaining symbolic
inputs are the unit coordinate and epsilon. A future artifact factory must use
this bound view and retain the lineage and certificate obligations.

## Reuse and executable evidence

The native dependency graph is Symbolica/Numerica `c540d3f`, symGCAD `a1132d4`.
Reused APIs are GCAD independent verification and linear cell maps, native
`AtomField` polynomial collection, `degree_bounds`/`quot_rem`, rational Sturm
root counts, native affine endpoint recognition, `CausalCell`, the existing
subtraction engine, `FunctionMap` derivatives, native Laurent series, owner
bincode evaluator restoration, `QmcSession`, and OneLOop's Expression backend.
The new operation is the finite interval ownership and closed-unit certificate;
existing monomial sector maps and origin-only residual checks do not supply it.

The ignored implementation probe passed three grouped controls plus strict
direct Clippy. It covers moving rational fibers, exact coverage/measure,
reflected upper faces, repeated poles, complex named numerators, incompatible
strips and resource/cancellation refusals, incompatible generation options,
exact coefficient admission, zero exponents, native saved evaluator parity, and
parameterized versus constant-fiber programs. A symmetric simple pole has finite
part `i*pi`; the native projective control has `i*pi/2` with normalization once.
Root and the independent ecosystem auditor accepted this restricted source
boundary. The registered feature-enabled native library gate passed **492 tests**
with 20 existing bounded/manual tests ignored, including all four rational
controls. The first strict Clippy run found one range-loop style issue in the
new reference test. After its enumeration-only rewrite, strict library/test
Clippy, workspace formatting and all four rational tests pass. Production code
and numerical tolerances were unchanged; the initial failure remains in
`target/no-deformation-contact-regularization-gates.log`, with the final gate in
`target/no-deformation-contact-regularization-final-gates.log`.

For the equal-mass bubble `s=16`, `m1²=m2²=3`, native GCAD returns sections at
`0,1/4,3/4,1`. The original quadratic F remains **one causal factor**. Six half
charts have a common strip `Re(epsilon)<1`. One saved native Laurent program was
sampled by a caller-owned QMC session with Kuo, Korobov3, 16,384 points and 16
shifts. Its raw orders zero/one map to master pole/finite orders minus-one/zero
under the existing HEPKit rGamma convention. Native OneLOop B0 comparison gave

- pole: `1 + 0i`;
- finite: `0.3520815669978355 + 1.5707963267948966i`;
- maximum reference difference: `5.55e-17`.

The native full four-component covariance is retained. The predeclared check
was six native standard errors plus `2e-10` per component. This single small
allocation, whose standard errors reached roundoff, is not a general convergence
or performance claim. Its complete guarded process took 2.739 seconds and the
sampled process RSS peak was 466.3 MB. Saved evaluator parity does not establish
saved endpoint-certificate persistence or a completed `GeneratedIntegral`.

Ignored provenance: `target/no-deformation-regularization/handoff.json`,
`draft-command.json`, `clippy-command.json`, and
`bubble-qmc-run/{plan,execution,stdout}.json`. The preserved first receipt and
failed zero-base test setup remain separate from the final accepted controls.
