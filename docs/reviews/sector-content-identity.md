# Additive numerical-sector representation identity

`KernelSet::sector_content_id(index)` implements the separate identity contract
from `geometry-reuse-scheduling-proposal.md`. The four focused tests and
independent implementation review pass. It returns
`fsd-sector-v1:<BLAKE3 digest>` or an error for an unknown numerical sector.
Exact-only contributions have no numerical sector identity.

The typed, domain-separated hash binds the immutable native exact program,
the existing native codec/compiler policy strings, ordered canonical parameter
names, flattened Laurent orders and Real/Imag layout, precision policy,
cancellation degree/tuples, and retained domain/chart semantics. It borrows the
program bytes and streams Serde JSON into the existing BLAKE3 writer; it does
not restore coefficient expressions or allocate a second program-sized JSON
buffer. Metadata uses its existing canonical transport owner and only the
charts associated with this numerical sector, including merged charts.

The selected kernel association becomes local index zero in this private hash
view. Thus moving the kernel within its parent while updating its associations
does not change its identity. Original chart ordinals, permutations and order
are retained; renumbering those charts can cause a safe cache miss. Missing
legacy semantic metadata is an explicit `None`, not an inferred certificate.

Other sector programs, exact global offsets, parent artifact IDs, timing,
sampling settings and mutable evaluation/replay state are excluded. The public
method computes on request; there is no new cache or work in sample evaluation.
Existing artifact formats and saved bytes, index selection, parent identities,
checkpoint/result schemas and replay bindings are unchanged. A legacy artifact
derives this additional identity from its normally validated current native
program while keeping its original artifact intact.

This is representation identity, not a proof of mathematical equivalence.
Native instruction ordering, factoring, symbol names or a codec policy change
can change it. It is also not a sufficient machine-code cache key, which needs
architecture and actual backend build identity. Equal digests do not authorize
transplanting checkpoint statistics or replay state across parent contexts.

Reuse is confined to existing owners: native immutable evaluator bytes,
artifact codec/compiler constants, the metadata transport, Serde and BLAKE3.
No graph, expression canonicalizer, instruction decoder or algebra is added.
Four focused controls cover fresh/loaded complex v3 kernels and evaluations,
legacy byte/parent preservation, a reordered parent with an unrelated exact
offset, and sensitivity to numerical policy/layout/program/retained semantics.
Private field-variation controls explicitly do not claim altered scientific
artifacts are loadable. The focused unit target passed all four tests in 0.07 s
after an 11.38 s build; evidence is `output/sector-content-identity-tests.log`.
The native runtime was serialized and reaped before the next reference owner.
The [independent review](sector-content-identity-independent.md), focused
formatting and library Clippy pass (29.43 s for Clippy, retaining the existing
upstream unused-result warning). The last full workspace gate remains the
earlier 305-test run; combined geometry/identity verification is a later gate.
