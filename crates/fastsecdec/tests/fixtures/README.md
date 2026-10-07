# Native example source

`gghh-double-box-source.dot` preserves the originally supplied D05 diagram,
before the double-box exporter's external color/helicity projection. The native
exporter imports it through HEPKit, checks its membership in the generated
two-loop diagrams, and applies the projection once. It is a developer fixture;
ordinary generation reads the complete projected `examples/gghh_double_box/graph.dot`.
It was moved unchanged from the example input directory during input cleanup.
Its SHA-256 is
`1bd26a5a2358a8f14822ee9c1f639e52a41fa36dde90c5e12d71c9b215f87ce7`.

# Historical kernel transport fixtures

`kernel-v2-triangle.json` is the unchanged inner kernel from the validated
pre-v3 triangle artifact in
`output/benchmarks/eight-core-preparation-20261005/triangle.fsd.json`.
Its original scientific content ID is
`60008f1012ce69f28c400df2568b63ab6b75ae0730d785924c74fc35358687af`.
The JSON key ordering reflects the enclosing historical CLI transport.

`kernel-v1-triangle.json` contains exactly those expression, precision and
cancellation fields, without v2 semantic metadata. The pure transport helper
`output/probes/legacy_kernel_fixture.rs` encoded it with the historical v1
field ordering and content-hash prefix. It performs no symbolic computation.
Its content ID is
`2303af9780ddc3528676d24a1bef7d190432dd6023de6e6085320869eea2e5da`.

SHA-256 of the frozen files:

- v1: `87a38c9fa61ac8f2206d423f1189cefe1d079ef010a2ba9f07ffe34bf4f55a5b`
- v2: `49c2133289e1945b9e9ecec89c854e0215bec71adcaa408ff99c6c58e8ec8fc6`

The tests require these historical expression-only formats to be rejected
before any evaluator restoration begins. They are frozen rejection fixtures,
not examples of supported loading or expression recompilation. Supported
artifacts store native evaluator programs instead.
