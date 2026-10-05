# Historical kernel transport fixtures

`kernel-v2-triangle.json` is the unchanged inner kernel from the validated
pre-v3 triangle artifact in
`output/benchmarks/eight-core-preparation-20261005/triangle.fsd.json`.
Its original scientific content ID is
`60008f1012ce69f28c400df2568b63ab6b75ae0730d785924c74fc35358687af`.
The JSON key ordering reflects the enclosing CLI transport; the v2 loader
reconstructs the historical typed payload before checking its identity.

`kernel-v1-triangle.json` contains exactly those expression, precision and
cancellation fields, without v2 semantic metadata. The pure transport helper
`output/probes/legacy_kernel_fixture.rs` encoded it with the historical v1
field ordering and content-hash prefix. It performs no symbolic computation.
Its content ID is
`2303af9780ddc3528676d24a1bef7d190432dd6023de6e6085320869eea2e5da`.

SHA-256 of the frozen files:

- v1: `87a38c9fa61ac8f2206d423f1189cefe1d079ef010a2ba9f07ffe34bf4f55a5b`
- v2: `49c2133289e1945b9e9ecec89c854e0215bec71adcaa408ff99c6c58e8ec8fc6`

The tests load these old expressions through the current native compiler,
check their leading triangle density, and require original bytes and IDs to
survive evaluation and re-saving. They do not regenerate a legacy fixture
using the current native-IR serializer.

`malformed-exact-program.bin` is a 33-byte native serde/bincode exact evaluator
from the independently preserved four-patch Symbolica build. The public native
builder created the single-output function `x`; its public serde value was
changed only at the result index (99, outside its stack) and reencoded by the
old native codec. Input/output counts remain 1/1. It was never evaluated.
The source is `output/probes/native_ir_exact_fixture.rs`; source/library hashes,
original JSON and the successful old decode report are retained under
`output/diagnostics/native-ir-decode/`. SHA-256:
`20b3746faff76e5a045f6d4db176509a4ad52fb6a4d9778dff38e73884175a62`.
This frozen input verifies that native structural rejection reaches the public
kernel loader even when the enclosing checksum and public dimensions match.
