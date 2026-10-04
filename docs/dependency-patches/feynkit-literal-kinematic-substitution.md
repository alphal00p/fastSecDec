# FeynKit graph propagator kinematics use literal replacement

The native `FeynmanDiagram::propagator_family` bridge substituted the routed
quadratic scalar product with Symbolica's implicit pattern conversion. A valid
caller invariant ending in `_`, such as `native_invariant_`, was interpreted as
an unbound replacement wildcard. Native graph ingestion then panicked before
FastSecDec could bind its parameter value. A dimension with the same naming
convention also appeared in the match expression and needed literal matching.

The patch wraps both exact expressions in `Pattern::Literal`. Graph topology,
routing, masses, and quadratic construction remain owned by FeynKit. No new
algebra or parsing is added. The adjacent method has no other substitution for
this operation.

- Base: FeynKit `8f834d9c62ae06fb327e4ef0b14abffda755b610`.
- Changed source: `crates/feynkit-graph/src/integrals/diagram.rs`.
- Reproducer: FastSecDec's
  `scalar_bindings_and_dimension_names_ending_in_underscore_are_literal` in
  `crates/fastsecdec/tests/native_input.rs`. It exercises native graph ingestion,
  scalar bindings, dimension substitution, and equal scalar/Gaussian prefactors.
- Before the patch, the test panicked in Symbolica with
  `ValueError("Unsubstituted wildcard native_input::native_invariant_")` from
  `FeynmanDiagram::propagator_family`.

The CLI records this patch in its existing FeynKit source-state hash. Artifacts
generated with the previous dependency state require regeneration.

Validation passed: the focused regression, all 14 native input tests, and the
121-test FastSecDec workspace suite. No upstream push has been performed.
