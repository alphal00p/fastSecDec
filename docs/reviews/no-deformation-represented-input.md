# Faithful represented coefficients and threshold source provenance

This milestone adds an explicit native threshold-input route for finite
Symbolica Float coefficients. It does not change ordinary scalar bindings,
add a CLI option, infer an uncertainty model, or supply an endpoint certificate.

The converter reuses native `Coefficient`, `Float::try_to_rational`, Atom tree
replacement and `ParametricIntegrand` constructors. Every real and imaginary
component becomes its actually represented rational value, preserving native
precision and binary-exponent provenance. A decimal-looking binary coefficient
is not rounded to a simpler rational. Function heads and wrappers remain intact;
later geometry and analytic admission decide whether they are supported.
Explicit uncertainty-bound semantics, nonfinite values and unsupported coefficient
fields are refused. Factor roles, causal semantics, regulator and domain survive.

An immutable owner retains both original and exact native inputs. GCAD geometry
and the affine projective preparation use the exact view. The original remains
the source object and signed-factor provenance. The request's one source-identity
accessor binds both views, the versioned interpretation and a canonical ordering
of all conversion occurrences. It is shared by staging, the kernel factory and
threshold metadata. Exact-only inputs keep their historical identities. A Float
input and an explicitly rational input can produce equal values without becoming
the same declared source.

Staging uses the existing native Atom/State codec. Restoration reconstructs the
exact view from the original under an independent caller resource cap, compares
every conversion and the complete transformed input, and rebuilds the request
and projective preparation. It retains the existing separate native GCAD
verification boundary. Serialized observations never construct a verified owner.
Conversion limits are operational association data, not mathematical identity.

The public API, exact c540 source and executable utility probe establish native
conversion for ordinary, multiprecision, subnormal and complex values. The owner
integration adds six maintained groups covering distinct source identities,
unchanged exact routing, original/prepared causal association, altered records,
caller caps, nonfinite values, fresh-process cube/projective restoration and
compiled/restored native vector parity. Independent factory and root reviews
accept these source, reconstruction and ecosystem boundaries.

Two additional maintained utility groups cover 192-bit complex coefficients,
subnormal values, idempotence, allocation caps, huge exponents, cancellation and
explicit refusal of uncertainty-bound interpretation. The registered owner
passes 107 threshold tests and nine kernel-factory tests; the two utility groups
pass separately. Strict native library/test Clippy and the native build without
the threshold feature also pass.

Input bytes, traversal nodes/depth, coefficient precision, rational-size bounds
and literal counts are preflighted. Native normalization can allocate transiently
before the output-size postcheck; this is not a hard aggregate-RSS bound. The
caller still owns process limits. Arbitrary floating-point domain/chamber
constraints are outside this route, and parameter-chamber compatibility is not
proved by converting coefficient literals.
