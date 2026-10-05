# Native color closure of the ggHH input

The optimized ordinary CLI completed all 30 mapped sectors and coefficient
expansions on the original generated ggHH input, exposing orders `[-1, 0]`.
Kernel construction then rejected `cas(2,coad(8))`. Native HEPKit algebra
deliberately retains symbolic color invariants by default; this was a numerical
input preparation gap, not a missing color identity or evaluator algorithm.

The existing `idenso::color::ColorSimplifySettings` provides
`with_cof_dimension_invariants`. The public `ColorSimplifier` also provides
`to_cof_dimension_invariants` for already reduced scalar color expressions.
Their implementation in Idenso's `color/casimir.rs` owns the supported SU(N)
formulas and recognizes concrete fundamental/adjoint dimensions. Existing
Idenso color tests verify the fundamental normalization `T_F = 1/2`, quadratic
fundamental Casimir and adjoint result for dimension eight. GammaLoop uses the
same setting in `integrands/process/evaluators.rs` when reducing tensors, and
the scalar conversion when its algebra is already reduced. HEPKit exposes the
same option in `TensorExpression.simplify_algebra`.

The example exporter now applies native color-only simplification to the raw
numerator multiplied by the single unnormalized external color delta. Gamma and
epsilon identities stay disabled and `AlgebraContraction::None` authorizes only
the color identity prerequisites. It retains the D-dimensional Dirac/Lorentz
algebra for the ordinary FastSecDec input path. Native `with_numerator` stores
the resulting numerator consistently in the graph's fragments. The remaining
projector contains only the two Lorentz wavefunctions; the color delta is not
multiplied again. Couplings, numerator prefactor, diagram weight and graph
topology retain their original native owners and values.

The generator verifies that both native color policies complete, and compares
the explicit result exactly with the symbolic result passed through the native
scalar invariant conversion. It rejects any remaining color invariant or
fundamental/adjoint interface. No hand-written color replacement, numerical
Casimir callback, arithmetic expansion or dependency patch was introduced.
An independent read-only API/source review confirmed the native normalization,
the contraction policy and this placement of the external delta.

The actual generated fixture probe passed in 0.5230 seconds with peak RSS
49,344 KiB. The raw diagram JSON/DOT, four raw numerator/projector/weight files,
model, parameter card and run card all remain byte-identical. Diagram selection,
the physical point and the external Gram data are unchanged. The native DOT
round-trip and exact two-policy comparison pass. The generated color-projected
numerator is 7,117 canonical bytes. Evidence is
`output/diagnostics/gghh-color-closure-1`; its compiled and executed sources and
hashes are bound there. A subsequent indentation-only formatter correction is
recorded separately with the executed source preserved. Example formatting and
Clippy pass. Attempt 8's original input is archived beside its process evidence.

Only the prepared `graph.dot`, its explicit color-numerator record, and the
additive color-provenance fields change. The original generator archive is
retained. Ordinary compiled generation and bounded integration subsequently
completed; the [native feasibility review](gghh-native-feasibility.md) records
the coarse Laurent estimate, full covariance and measured costs. Browser
feasibility is not inferred from the color probe or native run.
