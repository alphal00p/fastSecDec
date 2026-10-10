# Late numerical-dual contour request lowering

The native numerical-dual builder now has a narrow `build_with_lowering` hook.
It receives the complete smooth body and full zero/one/free coordinate map
after mathematical subtraction has selected the body, immediately before
Symbolica's native Dualizer seeds and differentiates it. Existing `build`
retains the unchanged path. Opaque undeformed source programs bypass the hook.
The hook changes callback diagnostic tags only: it does not alter mathematical
arguments, introduce a smaller-dimensional radius, or put tags into geometry,
exact symmetry, subtraction cancellation or Laurent algebra.

## Reuse and request identity

All jets, evaluator composition, optimization, precision mapping and native
compilation use the existing Symbolica APIs. The complete-vector controls use
the same native deferred builder as production compilation; they do not read
materialized symbolic coefficients in place of actual Dualizer execution.

An initial complete-vector control exposed a diagnostic association error.
Two different IBP endpoints had equal restricted radius expressions, so the
symbolic lookup correctly grouped them but the dual lowering incorrectly gave
both independent face requests that same group. Exported native instructions
showed three calls with distinct physical face work. This was not evidence of
a missing native common-subexpression optimization.

The corrected `Lookup::lower_on_face` retains only the actual supplied face,
including every matching mathematical namespace, while keeping physical
callback arguments unchanged. Symbolic lowering still retains the union for
a genuinely shared surviving symbolic root. Strict repeated execution of the
same request remains an error. These identities describe deterministic root
work; they are unrelated to integration sampling stream identities.

## Scientific gates, 2026-10-10

The native `generation::program` filter passed **10/10** controls, including:

- Taylor and IBP with an `x^(-3-eps)` endpoint and second/higher required jets;
- polynomial and sign-aware envelopes with complete pole/finite complex vectors;
- actual eager and SymJIT O2 callback execution, DoubleFloat and 192-bit Float
  precision remaps, and unchanged values versus untagged native programs;
- two distinct Taylor requests and three distinct IBP face requests, with
  exact observed request coverage and strict duplicate detection;
- cubic causal/positive-factor analytic controls and empty fixed-recipe
  direct/cooperative generation, compilation and restoration.

The separate actual-face association regression passed **1/1**. Independent
review confirmed that filtering metadata leaves the full physical arguments
available to the native implicit derivatives. Root and runtime owners reviewed
the diagnosis; no speculative CSE patch or observer relaxation was accepted.

Long native callback labels also exposed an independent owner serialization
limit. [SymJIT PR 16](https://github.com/siravan/symjit/pull/16) fixes that codec
while preserving all valid legacy short encodings. The accepted public
consumer includes that fix; the full JIT controls above keep the original
long labels and higher derivatives.

These are generation/callback execution gates. They do not yet admit dynamic
production integration: saved certified checker association and runtime
production validation remain separately gated work.
