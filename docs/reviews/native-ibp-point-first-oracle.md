# Independent point-first oracle for the existing IBP expression

The ignored Rust source `output/probes/point_first_ibp.rs` is a source-only
adaptation of the previously reviewed exact-rational point-first oracle. It is
formatted, independently read by the production author and HEPKit reviewer,
and frozen at SHA256
`48c4afca6a41936b319aed870818bd6643185426a93b6df3f02e4dd6d251df9c`.
No compilation or execution of this adaptation is claimed yet.

The input is the separately prepared **own IBP expression**, after the unchanged
native subtraction entry processes the actual mapped inputs for representative
80, source chart119, multiplicity4, nine coordinates and maximum order0.
Preparation must first reconstruct the original Taylor expression exactly and
retain its872-piece identity check. The oracle checks that completed preparation,
source hashes, imported coordinate/regulator identity and the exported IBP
expression's exact digest. It never treats Taylor and IBP as pointwise-equal
densities. Multiplicity remains unapplied on both sides of the eventual program
comparison.

The three prescribed points remain exact rationals:

- `[1/2,1/3,...,1/10]`;
- `[1/10^20,37/100,...,37/100]`;
- nine coordinates alternating `1/10^8` and `73/100`, beginning with the small
  coordinate.

Literal native coordinate substitution happens in the complete original IBP
Atom before any Laurent operation. Every coordinate must disappear. The native
Gamma initializer runs before import. There is no copied derivative or series
arithmetic, interpolation, quadrature, alias expansion, fitted coefficient or
f64 point conversion.

Use native relative width1 initially, inspect its actual native remainder and
relative bound, then request only the missing width using checked integer
conversion. At most four native calls are allowed within the proposed shared
180-second per-point external deadline and30-GiB virtual-address cap. The memory
cap is RLIMIT_AS, not a resident-memory claim. These proposed runtime bounds
still need the coordinator's specific handoff and actual frozen build evidence.
Failures remain retained and do not authorize a changed point or retry.

The native series remainder must cover the finite coefficient. Infer the lowest
nonzero coefficient from its native terms, reject fractional powers and vacuous
requested vectors, and export the complete inferred range through0. Missing
terms within that proven range are native exact zeros, not missing-data guesses.
There is no assumed minimum of−5 or fixed six-coefficient count. The downstream
comparison must use the complete union of actual production and oracle orders,
including any coefficient absent from either representation.

Every exported coefficient is evaluated through native Float at512 and1024 bits
with the same existing `1e-70 * max(1,abs(value))` agreement check, without f64
conversion. Original coefficient Atoms, exact-zero status, both values, differences,
remainder bounds, width attempts, hashes and incomplete progress survive in
separate output records. Inputs are rehashed before writing a completed result.
This establishes a distinct coefficient/program oracle if executed; it is not
an independent proof that the IBP rewrite preserves the full integral or a
performance measurement.

The first bounded preparation subsequently reached its unchanged180-second
deadline inside native IBP subtraction. Its exact872-piece Taylor reconstruction
and captured-expression identity passed in32.382 seconds, but no completed
`prepared.json` or `ibp-expression.atom` exists. The child was reaped and all52
frozen hashes passed. Evidence is
`output/diagnostics/native-ibp-prepare-1`. Consequently this oracle remains
uncompiled and unexecuted; it cannot substitute the Taylor expression or infer
IBP coefficients from incomplete preparation. A further preparation attempt
requires its own reviewed proposal, not an automatic retry.
