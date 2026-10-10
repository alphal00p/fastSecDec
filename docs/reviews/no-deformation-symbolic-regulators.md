# Symbolic endpoint subtraction with an explicit regulator family

This slice extends the existing symbolic endpoint admission and subtraction
operations to an ordered family `(epsilon, eta_1, ...)`. It is an internal
building block for the threshold pipeline. It does not remove auxiliary
regulators, prove a common convergence chamber, or establish cancellation
between independently continued cells.

## Native reuse and implementation

The implementation is in
[`generation/subtraction/endpoints.rs`](../../crates/fastsecdec/src/generation/subtraction/endpoints.rs)
and [`generation/subtraction.rs`](../../crates/fastsecdec/src/generation/subtraction.rs).
The existing single-epsilon entry remains a wrapper around the same operation
with a one-element regulator list. The explicit-family entry is internal; it
does not add an integration loop, worker pool, sampler, or public runtime option.

`endpoint_power_with_regulators` recognizes

\[
q=c+\sum_j a_j r_j,\qquad c,a_j\in\mathbb Q.
\]

It uses native Symbolica replacement to obtain the constant, native derivatives
to obtain each ordered slope, native rational extraction, and an exact expanded
reconstruction check. Nonlinear dependence, undeclared symbolic coefficients,
and empty or duplicate regulator inventories are rejected. The subtraction entry
also rejects coordinate/regulator overlap. No second affine parser, rational
type, or polynomial engine was added.
The same recognizer is available within the crate to the
[causal-phase view](no-deformation-phases.md).

The subtraction degree still comes from the native rational floor and the
existing checked degree limit. Any nonzero slope in the declared family permits
the existing symbolic continuation step. All-zero slopes retain the existing
boundary test: a vanishing boundary coefficient is pruned, while a nonzero
unregulated divergent coefficient remains an error. Exponent admission still
precedes its degree-limit check.

Taylor subtraction and integration by parts reuse the existing coordinate
derivatives and term construction. Each denominator retains the complete native
Atom `q+j+1`; no auxiliary slope is evaluated at zero. Prefactors, including
complete regulator-dependent causal phases, also remain native expressions.
Mapped-power dimensions are checked before subtraction. The implementation does
not expand a full Laurent family or choose an order of auxiliary removal.

## Executable evidence

The final coherent native gate uses Symbolica/Numerica
`c540d3f68c90fe7bff1e507458e57a20fb95b11c`, symGCAD
`a1132d4f4545c7784b3ec61239a05485e544b403`, and the unchanged SymJIT
`d74993ffd76a6fc322a7bcf3963fa786783a38a8`.

The endpoint module has ten controls, including five new explicit-family tests:

- exact fractional constant and ordered rational slopes, with nonlinear,
  undeclared, empty and duplicate-family rejection;
- an auxiliary-only slope regulating a fractional endpoint with no epsilon
  slope, under both Taylor and integration-by-parts strategies;
- actual power-pole IBP and Taylor equality with complete mixed-regulator
  denominators and an unchanged causal-phase prefactor;
- two-coordinate mixed regulator hyperplanes and both causal lips retained
  symbolically, without taking an auxiliary limit;
- all-zero-slope refusal and exact zero-boundary pruning, including the
  coordinate-role guard.

The separate four phase tests check term-local phases, rational-third branches
with a native complex evaluator, role/branch rejection, and the finite
`+i*pi/2` term of the exact two-cell scalar control after its auxiliary pole
cancels. That analytic control verifies this phase layer; it does not claim that
the general cell-normalization or integration pipeline is implemented.

Commands executed through `nix-shell` with the normal native license setup:

```sh
cargo test -p fastsecdec --features threshold-decomposition \
  --lib --example threshold_native_probes \
  --test generation --test coefficient_expansion -j2 -- --test-threads=1
cargo clippy -p fastsecdec --features threshold-decomposition \
  --lib --tests --example threshold_native_probes -j2 -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Results: **449 library tests passed, 20 ignored; 8 coefficient-expansion tests,
16 generation tests and 4 maintained native probes passed**. These are 477
distinct passing tests. Strict Clippy, formatting and diff checks passed.
The final library run took 35.18 seconds; strict Clippy took 67 seconds.
The ordinary generation tests cover the existing single-epsilon pole vectors,
gamma-function references, both endpoint strategies, restored artifacts and
precision rescue.

Ignored evidence is retained in
`target/no-deformation-combined-native-tests-r4.log`,
`target/no-deformation-combined-clippy.log`, and
`target/no-deformation-combined-native-test-inputs-r4.json`. All 364 pinned
source/manifest inputs were unchanged at gate closure. Earlier compiler
inference failures and two projective test normalization failures are preserved
in the preceding logs; their repairs used explicit native Atom types and native
expansion, without changing mathematical tolerances or production arithmetic.

## Independent review and remaining boundary

The independent foundation/reuse reviewer accepted the draft and final tracked
endpoint implementation. The final delta comprised only helper visibility for
phase reuse and test type annotations. The reviewer confirmed exact affine
recognition, complete denominators/phases, pruning/error boundaries and reuse of
the existing symbolic subtraction engine.

A nonzero regulator slope is not a holomorphy certificate. The next pipeline
must still establish a compatible auxiliary family, a common continuation
domain or an appropriate certified continuation argument, and the required
vanishing of complete mixed and higher polar principal parts before restricting
auxiliary regulators. It must also supply the real-cell pullbacks, closed-face
normal forms, measure and source-coverage proofs. This endpoint slice neither
certifies those obligations nor treats an unresolved pole as zero.
