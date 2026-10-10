# Causal phases of verified threshold cells

The `CausalCell` view borrows its verified geometry owner. It extracts one
phase **per density term**, and keeps the original factor association and sign.
Shared geometric split polynomials do not merge distinct term exponents.
Polynomial numerator factors, including complex numerators, remain unchanged.

For each negative causal factor with complete exponent `q`, the magnitude is
the sign times the prepared polynomial, and the phase is `exp(-i*pi*q)`.
Its positivity statement is restricted to the open cell and the owner's exact
kinematic bindings/chamber. It is not a closed-face certificate.

The implementation recognizes `q = q0 + sum(qj*rj)` with the existing native
endpoint admission routine, including physical epsilon and every explicitly
declared auxiliary regulator. It represents the phase as

```
(-1)^(-q0) * exp(-i*pi*(q-q0))
```

Here the native power uses the principal upper-lip logarithm of `-1`, so its
negative exponent supplies the required lower causal phase. This factoring
lets native exact integer/half-integer powers simplify before cell poles are
combined. It does not freeze any regulator dependence. Generic negative
singular factors require explicit branch semantics; the code does not invent
an `i0` prescription for them. Regulator names cannot also denote coordinates
or kinematic inputs.

## Reuse evidence and validation

Public APIs: `AtomCore::pow`, `exp`, `series`, native evaluator construction,
and the existing affine endpoint recognizer. Source inspection covered
Symbolica's power normalization, rational coefficient powers, and series
constant/exponential handling at consumer revision `c540d3f`. No phase algebra,
series engine, numerical complex type or analytic endpoint integrator was
reimplemented.

A focused executable probe passed four controls. These cover exact half powers,
third-root phases against native complex exponentials, full auxiliary/epsilon
phase slopes, and cancellation of the two cell poles in
`integral[-1,1] (x-i0)^(-1+eta) dx`, with finite value `i*pi`.
The initial plain-exponential probe retained `exp(i*pi)` symbolically rather
than simplifying it to `-1`; the factored form uses the owner's exact power
normalization and preserves that failing observation as evidence.

Five integrated geometry/phase controls pass in the native Cargo-built core.
They test separate phases for terms with opposite original factor
signs, unchanged complex numerator data, the scaled finite-imaginary control,
rational third phases, and rejection of ambiguous branches/regulator roles.
An independent runtime review checked the mathematical factorization and
requested the regulator-role guard, which is implemented.

The fifth control admits exactly bound kinematic symbols in the phase exponent,
retaining all regulator dependence and the original unbound source identity.
Geometry and phase now share the native simultaneous specialization helper;
see the [request review](no-deformation-request-options.md). Runtime-dependent
exponents still require a separately admitted fiber and are not inferred from
an open-cell proof sample.

## Scope

This view is an interior branch identity. It does not construct cell maps,
resolve algebraic faces, establish a common convergence chamber, or certify
auxiliary-regulator cancellation for general integrals. The analytic controls
have explicit affine maps and known scalar integrals; they are not evidence
that the general endpoint resolver or threshold integration pipeline is done.
No sampling or production RNG stream is used. Raw probe evidence remains
untracked in `target/no-deformation-phase/`.
