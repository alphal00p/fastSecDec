# Native derivatives with earlier constant-face substitution

The instrumented actual run reaches a complete native composition bound and
then spends substantial time lowering regular-coefficient derivative requests.
This proposal targets that observed phase. The earlier scalar-weight option
remains inactive. The subsequent test-only implementation has passed its small
core gate; `native-interleaved-face-independent.md` records the source and
executed review. Evaluator and actual-representative gates remain separate.

## Exact restricted identity

The existing resolver computes a complete native mixed partial of one regular
coefficient body, then substitutes all request arguments simultaneously. Its
local source is `series_first/named.rs::Coefficients::resolve`. For the actual
Taylor requests, many arguments are fixed faces while the full mixed partial
can become large before their substitution.

The candidate admits a request only when its existing arity/depth checks pass
and **every argument is exactly its own original parameter, native zero or
native one**. Equality is native Atom identity, never a numerical estimate.
At least one constant face is needed to take the new route. Different variables,
swaps, composed arguments such as `x*y`, arbitrary numeric or complex values,
and every unrecognized shape retain the original full-partial and simultaneous
replacement route unchanged. No general substitution simplifier is added.

For distinct independent parameters, evaluation at a constant face of one
parameter commutes with differentiation in another. Evaluation in a parameter
must remain **after all requested derivatives in that same parameter**. Thus a
narrow deterministic schedule is:

1. Start with the original native coefficient body. Apply every admitted
   zero-depth constant face first, in the existing body-parameter order.
2. Visit positive-depth parameters in that same order. Apply the exact requested
   number of existing native derivatives. If its argument is zero or one,
   immediately perform that parameter's native literal replacement. If the
   argument is its own parameter, perform no replacement.
3. Return the native body as the existing request's resolved value.

This preserves the same mixed derivative and final constant faces where the
original admitted regular coefficient has that face value. It introduces no
endpoint limit, regularization, derivative formula, algebraic expansion or
changed domain assumption. Native singular/unsupported behavior remains visible;
zero results are accepted only when the native operation returns exact zero.
The original Taylor admission and mapped-input identity remain prerequisites.
The candidate must not apply to the endpoint monomial factors themselves.

## Ownership and caches

Use the existing native `AtomCore::derivative(parameter)` and
`replace(Pattern::Literal(Atom::var(parameter))).with(face)` operations, or the
equivalent reviewed native literal `replace_map`. No new Symbolica patch,
global callback or derivative-with-aliases implementation is needed. Native
DER decoding, chain rule and request formation remain unchanged. The native
alias builder and exact IR still own later sharing, O2, MPFR and cold reload.

The current `partials[(source, depths)]` cache represents an **unsubstituted**
mixed derivative. Never insert a face-specialized intermediate into that cache:
the same key is reused by composed-argument fallback requests. Preserve its
original use for the fallback and face-free route. The existing full-request
`faces[(source, depths, arguments)]` cache can own the completed specialized
result. A new partial-prefix cache, axis reordering heuristic or general jet
framework is unnecessary for the first proof.

Keep original and candidate strategies explicitly selectable in the test-only
proof so that the old resolver remains an independent executable control. It
must be possible to verify that the restrictive branch executed, and that a
composed or swapped argument selected the old route. Cache statistics may change
but cannot be presented as coefficient identities or accuracy evidence.

## Required small controls

Use native exact comparison of both resolvers on small bodies, with native
expansion only where the comparison stays small. Cover:

- Mixed positive depths, both zero and one faces, unchanged coordinates, and a
  zero-depth face on a later axis that can be applied before any derivatives.
- A derivative above a polynomial degree yielding exact zero, and a nonzero
  mixed derivative, so a vacuous zero result cannot pass the whole proof.
- Negative powers of a positive residual polynomial, including mixed partials
  and zero/one faces. Negative or arbitrary argument values are fallback cases,
  not a reason to expand the fast-path scope.
- Composed arguments, swaps and coordinate-dependent substitutions whose naive
  sequential replacement would change later derivatives; explicitly verify the
  unchanged simultaneous fallback. Reuse a source/depth with different faces
  and a later fallback to detect contamination of the unsubstituted cache.
- Complete Gamma-regulated vectors under both Taylor and IBP, including a
  negative requested maximum and the finite coefficient; original native
  bounds, unsupported cases and conservative conditioning metadata remain.

After symbolic controls pass, retain the existing full-order native program,
exact-point 512/1024-bit, rounded-coordinate weighted/forced-replay, cloned
worker and callback-free cold-IR checks. Only then consider one freshly frozen
bounded actual representative. It still needs independent original-expression
oracles and all available orders; different factored Atom layouts are not a
reason to weaken numerical or source-identity gates. There is no automatic
whole-graph retry or performance claim from this proposal.

## Concrete test-only source review

The subsequent `named/interleaved.rs` and resolver selection in `named.rs`
implement the reviewed restricted schedule. Original resolution remains the
default. The candidate checks every argument, requires a face, processes
zero-depth faces first and differentiates each positive-depth parameter before
its own replacement. The resulting body enters only the full-request face
cache; fallback retains the unchanged unsubstituted partial cache and native
simultaneous map replacement. No production route changes.

The new exact request controls use polynomial, positive-denominator negative
power and logarithmic bodies. They test nonzero mixed derivatives, exact zero,
early zero-depth faces, repeated requests, differing faces followed by original
fallback, swaps, compositions and out-of-scope positive/negative numeric
arguments. Route counters verify actual branch selection. Both resolver modes
run through existing complete-vector and error controls, with an added
coordinate-dependent Gamma case at negative requested maximum.

No actionable source finding blocks the bounded small core gate. Its acceptance
still concerns regular coefficient bodies with the selected face values;
argument shape alone does not prove a general singular-limit identity. The
program/weighted/cold controls and actual representative remain separate
unexecuted gates at this source review point.
