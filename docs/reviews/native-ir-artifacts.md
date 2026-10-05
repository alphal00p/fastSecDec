# Native evaluator artifact transport

Version three stores Symbolica's exact `ExpressionEvaluator<Complex<Rational>>`
for each sector, together with ordered input symbols, Laurent orders, numerical
real/imaginary layout, cancellation metadata, precision policy and the existing
portable chart/domain record. Generation builds this native program from the
retained `AliasedAtom` vector directly. Saving it neither expands coefficient
images nor requires a host JIT. Ordinary compilation, conditioning and MPFR
rescue derive from that same exact program.

The codec identifies Symbolica 3.0.1 upstream revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, evaluator schema v1 and the
serde/bincode-2 standard configuration. It separately binds SymJIT 2.26.4,
O2, direct translation and zero Horner iterations. Native serde is not assumed
stable across arbitrary builds with the same package version. The fifth local
dependency patch adds structural validation without changing this wire format;
the outer CLI dependency provenance still identifies the patched source build.

Public loading requires v3 semantic metadata and validates the envelope,
content identity, codec, policy, contiguous orders, component layout, native
input/output counts and existing chart associations. Native built-in Gamma
callbacks are initialized before program decoding. Program-byte exhaustion
is required. Symbolica's decoder owns instruction/stack structural validation;
the checksum and public dimensions are explicitly insufficient substitutes.
See [the native validation patch](../dependency-patches/symbolica-evaluator-ir-validation.md).
There is no FastSecDec instruction interpreter or parallel IR validator.

The exact bytes are captured before JIT compilation or mutable worker evaluation
and shared immutably across worker clones. `KernelSet::to_bytes` returns the
retained original transport. The loader never trusts serialized exact-zero or
component-realness flags: cold programs initially use conservative facts.
The explicit component layout remains authoritative, and exact imaginary
rational constants cannot be silently projected into a real backend even when
their f64 conversion would underflow. The backend owner additionally reviews
fixed external constants, which require native conversion rather than just a
literal-root scan.

Versions one and two retain the historical expression payload ordering and
content-hash prefixes. The current native compiler can load those expressions,
but the resulting kernel keeps the original legacy ID and bytes. This preserves
existing checkpoint identity. Frozen pre-v3 triangle expressions test both
metadata-bearing v2 and metadata-unavailable v1; their source, hashes and v1
transport derivation are recorded in
[the fixture note](../../crates/fastsecdec/tests/fixtures/README.md).

The coordinated initial scientific gate passes 103 tests, including the cold
Gamma/log worker replay, hidden complex aliases and negative maximum order,
weighted precision controls and semantic-metadata tampering. Its log is
`output/native-alias-scientific-tests.log`. The first follow-up artifact target passes
all five tests (`output/native-ir-artifact-followup-tests.log`), including the
33-byte codec-valid native fixture with an out-of-range result slot: a genuine
one-input/one-output v3 payload, with its checksum recomputed, returns the native
decode error before backend compilation. The other controls retain old v1/v2
bytes/IDs and reject revision/compiler/layout mismatches, truncation/trailing
bytes and an exact imaginary coefficient below binary64 range in a real layout.

The separate native structural-validation gate passes all four tests and rejects
all six frozen original malformed layouts through all three native codecs.
Malformed programs are never evaluated. CLI legacy inspection also passes its
focused test (`output/native-ir-legacy-inspect-tests.log`). The final artifact
target passes six tests, including supported fixed complex `Gamma(1+i)*x`
loaded through its correct complex path and then rejected fallibly under a
re-signed real layout (`output/native-alias-artifacts-six-tests.log`). Final
combined milestone gates remain pending; no loading/performance acceptance
follows from source or format tests alone.
