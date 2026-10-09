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

The public/source audit found that current native RealBall::sqrt explicitly does
not certify its rounded endpoint operations. Float exposes directed arithmetic
but no public directed sqrt entry point. The owner-library audit is investigating
a narrow certified RealBall sqrt, with executable evidence and an upstream PR.
FastSecDec must not implement its own square-root or interval-number type.

The proposed owner construction takes any positive finite native approximation
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
Independent numerical checks may have their own preparation and solve cost;
report it separately from production roots and measure it explicitly. Production
Laurent outputs and jets must share one scalar root per distinct coordinate/face
request through native CPE. The unchecked path must perform no ball mapping,
certificate evaluation or hidden validation root solve.

The certified checks enclose the mathematical contour at the supplied binary
input values. They must not be described as certificates for every separately
rounded JIT intermediate. Validation policy/provenance remains outside the
mathematical checkpoint identity. No root, deformation, precision rescue or
certificate computation consumes production RNG state.

The one-shot exact-offset binding path also needs the scoped callback
preparation precision around native `Atom::evaluate_with_prec`. Sector evaluator
mapping already supplies that scope; exact offsets currently do not, which is
safe only while dynamic execution is rejected. Its whole-vector f64/DD/Float
attempts must share the same failure-slot discipline and receive a forced-MP
regression before dynamic exact-offset admission.

## Next focused gates

Test descriptor/payload association corruption; zero-dimensional exact offsets;
fixed full-sector schemas on zero/one faces; native root enclosures against
independent high-precision roots; coefficient-zero and near-unit-S limits;
spectral degeneracy smoothness; tracked imaginary uncertainty; and explicit
nonfinite/underflow failures with checks disabled. Repeat the established
eager/SymJIT batch, fresh-process codec and ordinary/serial scientific controls.
Then compare always/pilot/off using identical production coordinates and verify
unchanged sampling-stream identities and true removal of unchecked overhead.
