# Independent Series-first prototype review

This review covers the test-only prototype in
`generation/subtraction/series_first.rs`, its controls/replay, and the additional
mapped-input capture hook. Production subtraction and Laurent generation have
not switched to this route. At this review point its shared typecheck passed;
small executable controls and the bounded real-representative replay are pending.

The numerical/algebra ownership is appropriate. Native `Series<AtomField>` owns
all epsilon coefficients, addition, multiplication and truncation metadata;
native Atom operations own coordinate differentiation and endpoint replacement.
Inspection of Symbolica's `map_coeff` and `truncate` confirms that mapping all
known coefficients to zero retains the same absolute remainder bound, expressed
as an empty series shifted to that bound. Native addition and multiplication
continue to propagate that uncertainty. The prototype does not infer an exact
zero from an empty coefficient array or create a separate convolution engine.

Each input is initially expanded at a positive native relative width. The
complete composed series must have an absolute remainder order strictly greater
than the requested maximum. An insufficient result increases the input width by
its observed deficit and recomputes; the new bound is checked again. Iteration
and width caps return explicit resource errors. This is conservative bound
orchestration, not an assumed upper bound on endpoint poles. Final extraction
rejects fractional epsilon exponents and coefficient atoms retaining epsilon.

The Taylor derivative/factorial recurrence and optional IBP denominator/power
updates agree with production. Every boundary denominator, retained coordinate
weight and common prefactor is multiplied as a native series. Prefactor grouping
preserves cancellations before their final multiplication. Keeping truncated
zero boundary/remainder pieces is necessary because subsequent meromorphic
factors can expose coefficients outside the earlier known range. Exact
unregulated endpoint admission is a separate problem: the prototype delegates
the whole density to production subtraction whenever a singular coordinate
power has zero regulator slope. This preserves production's exact branch
pruning and boundary-vanishing proofs; a finite truncated zero never substitutes
for an all-order identity. The fallback route is explicitly recorded.

Coefficientwise face substitution requires regularity jointly in epsilon and
the coordinate face. This review checked the existing admission boundary rather
than introducing a new validator: `ParametricIntegrand::new` forbids
regulator-dependent singularity polynomials and restricts polynomial numerator
factors to nonnegative integer powers. Generation also checks resolved singular
residuals at faces. Consequently an arbitrary example such as `1/(x+epsilon)`
cannot enter this captured admitted workload. The prototype's private mapped
term interface should retain that inherited premise; the commutation statement
is not a general assertion about arbitrary atoms with moving singularities.

The mapped capture runs only under `cfg(test)` and retains native power,
prefactor, regular-factor and parameter atoms before subtraction. Representative,
source-chart and multiplicity associations are explicit. Its active guard
continues to cancel generation before any usable result, including a missing
target; dropping the guard restores ordinary generation. The replay initializes
native Gamma callbacks before importing atoms and compares all captured Laurent
orders. It attempts exact Atom equality first, and separately retains values at
three identical points with 512/1024-bit native evaluation and a precision-
stability check. Large representation differences are not forced through a
blanket expansion merely to claim equality.

The five drafted control groups exercise Taylor/IBP complete vectors, endpoint
poles, Gamma/prefactor cancellations, negative requested order, fractional
coordinate powers, complex weights, unknown-zero bounds, typed invalid
expansions, an unexpanded degree-10000 factor, and the polynomial-pruning-before-
unregulated-axis edge. The latter specifically protects the reason for the
whole-density fallback. No scientific or reuse blocker was found in this source
review. Runtime results, output-size evidence and any proposal to promote a
strategy into production require their own gate.

## Executed controls and bounded target replay

All **five** Series-first control groups passed in 0.05 seconds. The existing
capture/cancellation and native-depth controls passed **three of three**, with
the explicitly invoked large capture still excluded from the ordinary test run.
The executed logs are `output/series-first-controls.log` and
`output/series-first-capture-controls.log`; the latter records 0.20 seconds for
that test process. These gates include the degree-10000 compact expression and
the unregulated-axis fallback edge.

The mapped-only actual capture then retained representative index 38/displayed
ordinal 39, source chart 46, multiplicity two, nine coordinates and one mapped
regular factor of only **1,305 bytes**. The bounded test-only replay completed
and exported orders -5 through zero, taking **82.874 seconds** for composition.
Its native attempts progressed from width one/absolute remainder -4 to width
six/remainder one, keeping 384 pieces. The resulting finite coefficient is
**191,666,453 bytes**, versus 256,030,180 bytes from the previous production-order
capture. Native composition reduced this representation size but did not remove
the large repeated bodies.

The measured process peak was **5,629,400 KiB** (about 5.37 GiB), and process wall
time was 84.046 seconds. The earlier native-relative production-order series and
restoration together took about 41.6 seconds, with a 1,281,632 KiB process peak.
Thus this attempt is a negative performance result, not a candidate production
speedup. Its same-bounds/full-order output is retained for the pending exact or
matched MPFR comparison; successful export alone does not certify equality on
this large representative. No production route was changed.

Evidence is `output/diagnostics/series-first/index38/{mapped,series-first}.json`
and `output/diagnostics/series-first/replay-index38.process.json`. The result
motivates inspecting native function definitions and formal coefficient reuse
before constructing large expanded Atom bodies, as a separate measured
experiment rather than a new derivative or series implementation.
