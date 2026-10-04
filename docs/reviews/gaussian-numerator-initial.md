# Independent Gaussian numerator review

Date: 2026-10-04. Reviewer: sector/generation implementation agent, independently of the numerator author. Scope: `parametric/numerator.rs`, the exact-dimension Gamma guard, and their eight scientific tests. No numerator implementation files were changed during review.

**Result:** no scientific correctness defect found in this initial slice. The construction supports polynomial scalar products of loop and external momenta without an inverse external Gram matrix. This does not establish completion of the high-rank fixture performance gates.

## Independent derivation

Let `A` be the sum of physical propagator powers, `L` the loop count, and `beta=A-LD/2`. For a numerator monomial `prod_j N_j^m_j` of total scalar-product degree `m`, introduce auxiliary coefficients `s_j` in the native quadratic form `sum_i x_i D_i + sum_j s_j N_j`. Differentiating its power `-(A-m)` gives `(-1)^m (A-m)_m prod_j N_j^m_j` times the power `-A`. Combining this identity with the scalar Feynman-parameter normalization cancels the rising factorial and leaves exactly

`(-1)^A Gamma(beta-m) / prod_i Gamma(a_i)`

times the mixed source derivative of

`U^(A-(L+1)D/2-m) F^(-beta+m)`.

This confirms both the radial Gamma shift and the unchanged overall `(-1)^A` sign. There is no additional factorial for repeated or mixed source derivatives: the total rising factorial already accounts for their multiplicities.

Writing an intermediate differentiated expression as `U^(a-r) F^(b-r) P_r`, the product rule gives

`P_(r+1) = UF dP_r + ((a-r)F dU + (b-r)U dF) P_r`.

After `m` derivatives and setting all sources to zero, the powers are `U^(A-(L+1)D/2-2m)` and `F^(-beta)`, exactly as implemented. Native Symbolica differentiation and polynomial arithmetic perform these operations; no second CAS is introduced. The additional source rows use the native scalar-product basis and Symanzik construction, so a degenerate external Gram matrix does not require inversion.

## Boundary and normalization checks

- Physical propagator powers and density monomial powers remain in native ascending-edge order. Numerator coefficients undergo the same symbolic tensor-dimension substitution as scalar graph weights.
- The coefficient checks reject hidden nonpolynomial loop dependence instead of treating it as an external coefficient.
- Auxiliary source symbols are checked against all existing input symbols, graph denominators, parameters and regulator.
- A polynomial numerator cannot introduce a denominator scale into a family already certified scaleless by the native scaling test; using that certificate remains valid. `F=0` alone is not substituted for the certificate.
- An exact-dimension source formula can contain `Gamma(nonpositive integer)*0` even when its regulated limit is finite. The explicit error before differentiation avoids dropping such a finite limit; retaining `D=D0+c*eps` permits Symbolica to take the correct meromorphic limit later.
- Graph weights are collected before this stage. The source construction introduces no extra automorphism/symmetry factor.

## Scientific checks

The eight author tests exercise propagator cancellation with raised powers, rank-two/rank-four Lorentz moments, degenerate external Gram data, nonpolynomial rejection, native scalelessness, the exact-dimension removable-pole guard, shifted loop routing, and a mixed two-loop moment. Their expected values follow independently from one-loop radial Gamma integrals, Lorentz isotropy, and factorization of disconnected loop moments. In particular the two-loop mixed moment contracts two rank-two tensors and divides by `D`, a useful test of cross-loop normalization beyond one-loop examples.

An independent run of `cargo test -p fastsecdec --lib parametric::numerator -- --test-threads=1` passed all eight tests. The serial test setting matches the restricted local Symbolica instance; this does not constrain the library's caller-owned numerical worker API.
