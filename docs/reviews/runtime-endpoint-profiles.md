# Runtime endpoint provenance review

Independent source review by the dashboard/artifact agent, 2026-10-07.

Accepted the generation-side provenance changes in `generation/types.rs`,
`conditioning.rs`, `subtraction.rs`, `coefficients.rs`, `coefficient_first.rs`
and `mod.rs`. The review covered information flow and ecosystem reuse, rather
than independently reimplementing endpoint algebra.

The physical subtraction path extracts the exact constant endpoint exponent
through the existing `endpoints::endpoint_power` owner before any integration
by parts. It stores its negation as the canonical native Rational string.
Every derived boundary/remainder piece shares those original powers, including
pieces whose active exponent is subsequently incremented by integration by
parts. The retained cancellation order remains the existing per-piece order;
it is not used to reconstruct the original power. The new profiles are sorted
and deduplicated deterministically, with inactive axes represented by empty
source lists. The existing expression and cancellation-row assembly is
otherwise unchanged.

The coefficient-series bound keeps every original power/order alternative per
axis, instead of retaining only the source of the largest integer order. This
is necessary because a power-specific runtime threshold can change which
alternative has the largest normalized loss. It retains the existing
conservative mapped-endpoint-bound semantics; it does not assert that every
cross-axis combination occurs in one physical remainder. The exact physical
fallback carries the retained physical profiles. Native Symbolica affine
recognition, differentiation and exact Rational arithmetic remain the owners
of endpoint admission. No alternate algebra, polynomial representation or
threshold certification was introduced.

The public generated-sector accessor exposes retained provenance. Its canonical
Rational text is state-independent metadata, and the existing cancellation
schema validates it before runtime use. Runtime kernels with old profiles
must continue to distinguish absent original powers from known powers; they
cannot infer a per-power override from an old Taylor order.

The root agent's focused native probe was reviewed as supporting author
validation: all four Physical/NativeNamed × Taylor/IBP routes preserved exact
original powers 2 and 5/2, produced projected cancellation rows equal to the
existing rows, survived format-7 cold serialization, and produced equal
power-override evaluation before and after loading while retaining immutable
template bytes. The source/log are retained only under ignored
`output/runtime-dashboard`. This review makes no new claim about end-to-end
performance or convergence and does not migrate the deferred permanent gates.
