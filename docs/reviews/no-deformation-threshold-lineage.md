# Threshold lineage and structural admission

The native `kernel::threshold_metadata` modules implement lightweight metadata
and strict structural validation. They do not create GeneratedIntegral/KernelSet,
does not write a new native artifact format, and does not certify mathematical
coverage, endpoint regularity or analytic continuation.

The data model preserves original physical source extent separately from
one-to-many patch/cell/endpoint/contribution lineage and resident contribution
selection. It reuses the actual native SourceSectorSelection validator and
ExactContributionPolicy. Existing GenerationSourceScope and monomial SectorMap
constraints are untouched. Certificate digests are associations, not proof
tokens. Fixed exact rational binding values are native Atom-table references;
their actual Rational value and complete symbol-role association must be checked
by the future native payload owner/factory.

Native expression records are referenced independently. The directory contains
only native record digests, table bounds and IDs, not decoded Atoms, evaluator
owners or global raw proof. This structural fact does not establish a runtime
RSS bound. Caller transport bounds and global-proof/worker peak accounting remain
separate tasks.

The initial schema-kind fence accepts only rational maps and epsilon-strip continuation.
Future algebraic selectors retain exact polynomial/ordinal/domain and branch
references but are explicitly refused by initial executable-capability checks.
Auxiliary integral-cancellation descriptors are likewise inspectable, not
silently accepted as implemented. A direct exact zero-dimensional preparation
has a distinct patch origin and phase/endpoint/partition references; no GCAD
cell or zero-from-empty-result assumption is manufactured.

The pre-registration direct-Rust test links the existing c540 native core only to reuse
the established source-selection and exact-offset policy APIs. There is no Cargo
or dependency rebuild, solver call or numerical run in that probe.
The metadata fixtures use explicit synthetic certificate references and claim
only association/shape validation.

Current focused tests: **12 passed, 0 failed, 0 ignored**, 0.03 seconds, in
`tests.log`. They cover one-to-many descendants; multiple/lower-dimensional
residuals with exact/zero leaves; closed inventory versus partial records;
exact-only/zero-only original extent; ancestry/continuation/native-schema errors;
fixed binding/strategy/extent identity differences; full-source partial,
exact-only, empty and explicit-all resident selections; direct exact
zero-dimensional lineage; unsupported algebraic/auxiliary capability; schema
limits and unknown fields. Native v15 save/restore, numerical parity, indexed
assembly and result projection are explicitly later slice-B/C gates.

The checked structural view is immutable and cannot be deserialized; record,
complete-inventory and resident-scope methods are available only through it.
Its descriptor digest is checked against record parent references, but remains
a structural/transport identity. Future mathematical identity must exclude raw
State-dependent transport digests and grouping, using a separate canonical
semantic projection; adding canonical Atoms to a transport hash is insufficient.

The pinned Serde unknown-field probe initially failed with exit 101 for exactly
ContributionKind::Exact, RecordKind::Exact and ResidentSelection::Complete.
The final code reuses ContourMode's existing strict empty-struct Wire DTO pattern,
preserving valid wire syntax; the tagged-variant regression now passes. No Serde
fork or dependency change was made. Adjacent SourceOrigin::OriginalInput already
rejected unknown fields.

One private strict lint initially reported only a collapsible nested condition;
the equivalent let-chain correction is retained. Final private strict Clippy
passes with warnings denied; final rustfmt passes. The independent initial
source review passed at data-only scope. The runtime reviewer and root
coordinator independently accepted the final checked-view/strict-wire fences,
zero-dimensional admission and partial-residency rules before registration.
All twelve registered lineage Cargo tests pass, as do strict feature-enabled
library/test Clippy and workspace formatting. The separate portable consumer
builds this symGCAD-free metadata and passes its existing 88 scientific,
restoration and integration controls. This checks the host portable backend,
not browser threshold generation. Native threshold execution remains separate.

The concrete integration inventory and parameter/proof decisions remain in the
ignored evidence directory `target/no-deformation-threshold-artifact/`.
No public generation flag is exposed by this work.
