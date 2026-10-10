# Native GCAD evidence staging

This boundary saves global preparation evidence for caller-owned workers. It does
not add a threshold integration recipe, endpoint certificate, solver checkpoint
resume, trusted proof deserialization, or independently loadable cell programs.

## Reuse and authority

The implementation reuses the existing generation-record envelope (schema 4),
Symbolica State/Atom/Symbol transport, content-addressed `RecordRef` validation,
and atomic temporary-file/write/sync/rename publication. The native
`ParametricIntegrand` Factor/Term DTO was extracted from ordinary source staging
and reused. Ordinary Source field order, domain tags, Atom order and symbol
layout are preserved; a frozen pre-extraction DTO is used only as a test oracle.
There is no expression-string parser or second algebra/atomic-record codec.

A request record keeps the original native input, exact kinematics, typed domain
and gauge origin, ordered coordinate and parameter symbols, and the complete
native GCAD Problem/options/limits. Reconstructing it calls the same typed native
constructors and never solves. A canonical association witness, using the
existing native canonical Atom/Symbol serializers, checks prepared terms,
projective images and measure, factor origins, signed factors and parameter
roles. It hashes the factorized density without expanding it or retaining a
second prepared Atom table. The physical source identity is independent of
process-local transport IDs; the RecordRef digest identifies the saved native
execution context and is not a mathematical-equivalence claim.

The raw result is a separate record bound to that exact request record. Native
`SolveResult` retains raw cells, root selectors, exceptional polynomials,
projection tables, lifting/alternate evidence, status and diagnostics. Incomplete
and failed-status results remain readable evidence. Same-geometry results cannot
be reassociated with a different numerator, full exponent/domain association or
solver settings merely because their polynomial split matches.

Verification is an explicit later operation. It loads untrusted native evidence,
runs the current independent verifier, and records an accepted, incomplete or
rejected observation. A previously accepted observation is historical only and
never yields `VerifiedDecomposition`. No restore path calls solve. Raw evidence
is already published when verification fails. Solver errors that return no
native result remain ordinary errors; no successful or zero result is invented.

## Ownership and resource scope

`StagedRequest` retains one supplied `Arc<GcadRequest>`; serialization borrows the
native raw result rather than cloning the full proof. A caller can reuse that
context for successive result attempts. Request and proof payloads are separate,
and the coordinator may retain only their small receipts.

Load operations check the caller's explicit maximum total referenced record
bytes, length/digest/path validity, and association before acceptance. The byte
limit is not a peak-RSS guarantee: the existing codec materializes buffers, and
native reconstruction/verification can allocate substantially. Hard worker
memory and wall limits remain caller-owned; no library pool is introduced.

Independent native verification still loads the complete global SolveResult.
Sharing it through Arc avoids copies but does not establish cell-bounded
residency. Later serial workers require independently loadable cell records,
emitted after global verification and bound to the request/proof identity and
lineage. Global CAD/verification memory must be reported separately. Arbitrary
serialized verification observations cannot authorize such records.

## Validation

An isolated source snapshot using the exact existing c540 Symbolica/Numerica and
a113 symGCAD graph passed **6/6** staging controls (final run 0.11 seconds) and **17/17**
ordinary streaming controls (0.87 seconds). The latter includes the exact legacy
source metadata/native-record byte oracle and existing fresh-worker contour
transport. The staging controls cover moving-root parameter families, complex
numerators and epsilon/auxiliary exponents, projective and explicit domains,
owner sharing, incomplete/rejected evidence, foreign density/settings,
checksums/kinds/truncation/path/byte limits, and a real child process that changes
symbol-registration order before restoration and reverifies without solving.

Independent foundation source review passed. The first private compile failed
only because its direct-rustc launcher omitted Cargo's crate-name environment
variable; that setup failure is retained. The corrected compilation had no
source errors. Strict private Clippy passed with warnings denied after two routine lint fixes
(a redundant closure and a test-only unnecessary borrow); the final six staging
controls and legacy-byte control were rerun afterward. The registered Cargo
gate subsequently passed all six staging controls and all seventeen ordinary
streaming controls, followed by strict feature-enabled library/test Clippy and
workspace formatting. Root review also checked the unchanged ordinary DTO
layout and the request/proof/observation trust boundaries. These small tests make no performance, general
endpoint-resolution, selective-cell-memory or complete-integral claim.

Raw source hashes, compiler/dependency provenance and logs remain ignored under
`target/no-deformation-gcad-staging/`. A subsequent wire-admission regression
uses an empty-struct `UnitCube {}` variant to reject unknown fields with the
adopted Serde revision. Its valid serialized bytes are unchanged. This reuses
the existing strict contour DTO pattern and requires no dependency patch.

The preparation and geometry boundaries are described in
[the GCAD audit](no-deformation-gcad-audit.md),
[projective preparation](no-deformation-projective.md), and
[cell maps](no-deformation-cell-maps.md).
