# Additive sector content identity: independent review

Source and focused execution review, 2026-10-05. The implementation in
`kernel/artifact/sector_identity.rs` adds a derived accessor without changing
artifact versions, parent content IDs, original sector indices, checkpoint or
replay bindings. No source defect was found.

The domain-separated streaming digest binds exact existing native-program
bytes and codec/compiler policy, ordered input names/output layout, precision
and cancellation policy, and the existing portable metadata owner's selected
chart/domain records. It never restores aliased coefficient expressions or
evaluates instructions. Only the selected parent kernel ordinal is localized
to zero; original chart IDs/order are retained. This permits safe cache misses
under chart renumbering and makes no symbolic-equivalence claim. Missing legacy
metadata remains distinct. Architecture and actual backend build identity still
belong to any future machine-code cache, not this representation digest alone.

The four focused tests passed (`output/sector-content-identity-tests.log`):
native complex/Gamma full-layout reload and evaluation; unchanged legacy
bytes/parent identity and metadata distinction; isolation from unrelated
kernels/exact offsets and changed parent ordinal; and sensitivity to bound
program, input/output layout, precision, cancellation and retained semantics.
Private field mutations in sensitivity controls are explicitly not represented
as loadable altered scientific artifacts. Inspection also confirms mutable
evaluation state does not enter the identity.

This review does not claim a measured hash cost, persistent machine-code cache,
new sector-scoped sampling identity, or invariance under algebraic rewriting.
