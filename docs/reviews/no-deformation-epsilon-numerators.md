# Epsilon-dependent numerators and positive scales

This milestone extends the certified rational-fiber continuation path. It does
not complete general algebraic endpoint resolution, parameter-uniform
rebinding, or the public threshold generation workflow.

## Implementation and native reuse

Symbolica collects the numerator in epsilon only, using an exact `AtomField`
with statistical zero testing disabled. The coordinate-dependent coefficient
bodies remain factored and are registered as tagged native functions. Their
epsilon powers stay outside these functions so native Laurent construction
can see them. The existing symbolic derivative and subtraction machinery
handles every coefficient, and the outputs are combined into one Laurent
vector. There is no separate numerator evaluator or endpoint AD implementation.

Each coefficient is checked for exact complex polynomial closure on the
admitted physical fiber. The common convergence strip remains the sufficient
one obtained from singular factors; accidental numerator cancellations at a
particular epsilon do not enlarge it.

Prefactors additionally admit `c^(a+b*epsilon)` for exact rational `c>0` and
exact rational `a,b`. The real-log branch is entire and nonzero in epsilon.
The native power is retained for Symbolica's series expansion. This admission
adds no excluded regularity-witness points and does not alter Gamma pole
distances. In particular, `sqrt(2)*Gamma(epsilon)` is accepted through this
explicit positive-scale certificate, while unrestricted algebraic coefficients
remain outside the rational-prefactor checker.

The API, source and executable probe audit covers
`to_polynomial_in_vars_with_field`, native `FunctionMap` differentiation,
`Atom::series`, and the existing affine endpoint-exponent recognizer. The
adopted Symbolica revision is `c540d3f68c90fe7bff1e507458e57a20fb95b11c`;
this slice needs no owner-library change.

## Scientific evidence and independent review

Four primitive API groups and seven integrated bridge groups pass. Controls
include factored coordinate bodies, unsupported nonpolynomial epsilon
dependence, two physical parameter fibers, complex numerators, repeated poles,
ordinary endpoints, both symbolic IBP and Taylor, and complete restored native
coefficient-vector parity against an independent sum of source terms with
epsilon powers in their prefactors.

An actual HEPKit graph-derived bubble retains its Gamma/rGamma normalization.
Changing `mu^2` from one to four leaves its pole coefficient unchanged and
shifts the finite coefficient by `log(4)`, checked against the native OneLOop
reference without additional sampling. The existing graph/QMC reference test
remains registered. No new convergence or variance claim follows from these
deterministic controls.

The frozen direct bridge ran in 1.551 seconds with sampled aggregate RSS
488,480,768 bytes; strict direct Clippy passed. The ignored handoff is
`target/no-deformation-epsilon-numerators/handoff.json`, SHA-256
`aa9b83b2ba202fb0c55e2402b1e078a4a5ed36a59d1b3f7ae464f3e4f07c7106`.
The independent resolution agent reviewed the coefficient exposure, source
bindings, branch choice, common strip and native reuse and found no issue in
the declared scope. All 12 registered rational-continuation tests pass,
including both existing native OneLOop/QMC controls and the new numerator and
scale regressions.
Strict registered library/test Clippy and workspace formatting checks also
pass with the native threshold feature enabled.

## Limits and next interfaces

Float conversion was probed separately using native `Float::try_to_rational`;
it is not enabled by this milestone. Existing numeric TOML bindings already
preserve their binary values. Imported native floating coefficients need a
separate represented-value conversion record, precision/size limits and an
explicit treatment of uncertainty-bearing inputs.

Positive-scale admission is for the checked exact fiber, not a proof of
positivity throughout a parameter chamber. General auxiliary-regulator
restriction, algebraic towers, closed-face resolution and efficient native
threshold artifacts remain separate acceptance gates.
