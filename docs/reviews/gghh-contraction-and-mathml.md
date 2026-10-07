# gg → HH contraction and expression display (2026-10-07)

The native notebook reported a polynomial-admission failure for graph
`6586fc41a2a00087ef7be79f59b61224`, displayed as D00 in the community notebook's
filtered catalogue. The corresponding graph in the broader FastSecDec catalogue
has the same native ID. Tests select native IDs, not display counters.

Both routes share the native Standard Model, diagram, projector, graph factors,
Gram inputs and parametrization. The first changed boundary is tensor
contraction. `contract="minimal"` can leave closed tensor networks inside
products of sums. `is_scalar` only excludes free indices; it does not certify an
index-free polynomial. The existing `contract="dots"` primitive resolves those
networks. The native polynomial guard remains unchanged. No alternate tensor
algebra, graph representation, global numerator expansion or error-to-zero
fallback was introduced.

The matched graph and a second triple-gluon graph have nonzero projected
numerators and now reach native parametrization and nonempty sector geometry.
The seven native ggHH tests also cover a complete one-loop eager compilation,
runtime Gram bindings and retained expression viewers. They do not claim full
two-loop integration or convergence for the reported graph.

`dot_i_j` remains a stable runtime symbol. A diagram-specific legend uses
HEPKit's external-edge, leg and momentum-basis owners to name its momentum or
polarization pair. Failed generation visibly retains its error and labels
partial counts as incomplete. The standalone and modular copies share focused
source-parity controls; isolated single-file startup remains covered.

The formula overlap is independent of numerator admission. The shared Spenso
pager used CSS flex layout on nested MathML sums inside products and fractions,
and allowed outer terms to shrink. The owner fix wraps only an outer sum and
keeps each term's native width. Nested expressions retain MathML layout and
native subexpression navigation. The two new regressions fail on the old
renderer; all seven pager controls pass on the corrected owner source.
A private Chromium comparison of captured live MathML preserves token order
and removes the nested flex rows and overlapping fences.

An independent ecosystem review confirmed that the legend follows native routing,
the contraction preserves projectors and graph factors, and passive presentation
retains its existing action boundaries. It also caught a shared failure-state
ambiguity: later integration errors must not label a completed generation as
failed. The presentation regression covers that distinction separately.

The renderer fix is published on GammaLoop's `feynkit` branch at
`69a6b97e6cd81ecba4f2000d68a4ca97245dab6c`. Its Python source is embedded in the
community extension, so deployment requires a rebuilt wheel and a fresh Python
kernel. No shared notebook server or installed environment was modified during
these checks. Packaged-wheel validation is tracked separately by the community
delivery, rather than inferred from a source-level renderer override.

## Progress callback follow-up

The live notebook had received the contraction fix, but its first progress
callback exposed a presentation regression: reading `generation_session.complete`
inside the callback attempted to borrow the native session while `step` already
held it mutably. This produced `Already mutably borrowed` and paused generation
before sectors were reported. Source-only presentation tests and native tests
without the display callback had missed this combination.

Generation presentation now uses the Python-owned `RunState.kernels` field,
which is updated after the native step returns. It performs no native session
inspection from a progress callback. This preserves the distinction between an
incomplete compilation and an integration failure after successful generation.
The regression runs the real native generation loop with the notebook's display
observer attached, in addition to the presentation controls.
