# Independent CLI native family-preparation review

Source review, 2026-10-05. The CLI opt-in delegates to the existing
`ParametricIntegrand::from_graph_prepared` and native
`FamilyPreparationPolicy`; it introduces no propagator algebra, graph parser,
numerator reduction or alternate parameterization. The original DOT, model,
parameter card, kinematics and all original parameter labels remain the input.
The CLI explicitly defaults to `Original`, independently of the library
policy's different default, and that branch calls the unchanged `from_graph`
entry point. Direct parametric input rejects a requested
projection because it has no native graph family to prepare.

The existing preparation owner admits only one coefficient-one native partial
fraction term with nonnegative powers, fewer active denominators, unchanged
momentum bases and exact native inverse-product identity. Unsupported
decompositions retain the original family with a typed reason. Original-label
validation precedes preparation; native U/F construction and Gaussian
parameterization retain the active original labels, raised-power measure and
Gamma/sign factors. Graph numerator, overall weight and measure multiplier are
applied once. Existing library tests already exercise noncontiguous labels,
loop numerators, exact repeated-line Gaussian integration, fallbacks and graph
weight/raised-power normalization.

The adapter retains the native `FamilyPreparationReport` for an opt-in request,
including a fallback. Its optional provenance field is omitted for the
historical/default route, preserving the previous serialized provenance. The
artifact identity binds a present report, and the existing scientific run-card
fingerprint binds the selected policy. Original source files remain recorded;
normal resume source verification is reused. Inspection already presents this
provenance. Generated coordinate maps describe the actual active projective
domain; the report's active original indices do not claim an invertible map
back to redundant original Schwinger parameters. Numerical scope remains the
complete integral.

The two focused process tests exercise a duplicated massive vacuum line with
graph numerator, projector, overall factor and measure multiplier. The
independent expected full vector is `210*Gamma(eps)` through epsilon one.
Original physical and prepared named routes generate ordinary artifacts, then
fresh CLI processes inspect and integrate them. Assertions cover retained
original source hashes, active original parameter labels, actual zero-dimensional
charts, explicit native state-limit fallback, invalid direct/zero-bound use and
identity rejection after tampering with the report. Separate controls check the
native policy's TOML spelling, the CLI default, exact legacy provenance
serialization and policy-sensitive run-card fingerprints. This is meaningful
adapter coverage alongside the existing native preparation tests.

No source-level correctness or reuse blocker was found in the applied adapter.
The final focused execution is accepted: **38 passed, zero failed, zero
ignored** across the two new process tests, 27 CLI unit tests, four existing
CLI process tests, three dependency-provenance tests and two coefficient tests.
The reviewer independently matched every log summary to the result record,
all eight source files to their archived hashes, and all six selected executable
hashes. Both builds succeeded; owned formatting and CLI all-target Clippy pass.
The final Clippy record takes 0.39 seconds and retains only the existing native
dependency warning.

The first two process tests failed on fixture assumptions: the direct-input
case omitted its required `terms` field, and an expected native canonical
parameter label omitted its empty attribute spelling. The retained log shows
those exact failures. Only the fixture values changed; production delegation,
normalization, numerical tolerances and acceptance scope stayed unchanged.

Evidence is under `output/diagnostics/cli-family-preparation-1/`; the independent
record is `independent-review.json`, SHA-256
`1ac999acdbda9d6675217ae2317a7761f5420126d15568f8bcd6a7fbc3e42ce5`.
This accepts the focused CLI milestone, not a projected fullgraph artifact or
performance claim. The earlier ten-parameter named trial remains a separately
recorded intentional cancellation, with no artifact or integral estimate.
