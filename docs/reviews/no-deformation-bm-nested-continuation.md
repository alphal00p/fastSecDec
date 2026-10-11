# Original coefficient hierarchies through a new companion cycle

This milestone advances local marked-ideal recursion through a genuine second
physical blow-up. It does not implement the complete BM invariant/driver,
global centre gluing, relative parameter resolution, a real integration atlas,
or an endpoint certificate for a physical integral.

## Construction and retained coverage

The control starts from `z^2+x^3+y^3` with mark one. Its original coefficient
construction supplies the lower marked problem `(x^3+y^3, 2)`, whose companion
has residual maximum three. The six first physical charts retain the original
coefficient hierarchy, current marked ideal and companion, original ancestor
C/J, actual support maps, and exceptional-divisor history.

The complete component inventory proves the supported residual maximum drops
from three to one. A new companion cycle consumes that receipt. It retains
the balancing exceptional monomial because the new maximum is below mark two;
it does not mint a new initial history or identify a transformed ideal with
the original coefficient construction.

The resulting second physical blow-up issues nine charts. Five can currently
carry all required owners; four need further physical localization. The
caller-owned `ContinuedSupportedFrontier` retains every issued chart, accepted
results and pending reasons. It cannot report a complete inventory while any
chart is pending. Repeating an accepted entry performs no algebra. Retrying an
incomplete entry recomputes its inner work; this is in-memory progress, not
durable recovery or measured bounded residency.

Strict-support saturation precedes nonempty-frame admission. An exactly empty
auxiliary support retains its entire ambient chart and does not establish a
zero density. A candidate minor frame must generate the full localized support
ideal; excluded candidates retain missing-relation evidence, and acceptance
still requires the exact open cover. No support-only unit becomes an ambient
coordinate inverse.

Shared lower-boundary transport preserves historical births even when the
corresponding exceptional equation is a unit on a lower support and is absent
from its active SNC ledger. The inherited regression now checks this unit
identity explicitly while retaining independent birth, root and foreign-owner
checks. Historical authority is never reconstructed from the active ledger.

## Disconnected terminal contacts

An independently compiled public consumer selects another valid contact order.
Its terminal frame has no free or parameter coordinates but is disconnected:
native localized identities give `t^6=120^6`, with coefficient ideal
`(t^3+120^3)`. That ideal is zero on one component and a unit on the other.
It is neither the global zero ideal nor an invalid parameter specialization.

`RecursiveComponentCover` uses the existing native component splitter and
retains both opens, the original marked source, normalization receipt and
complete recursive level stack. All driver layers preserve this as explicitly
incomplete. No zero-ideal centre is issued on the unlocalized frame. A separate
`t^2-1` regression checks the two localized zero/unit results and stable retries.
Continuation through these physical opens is the next required operation.

## Reuse, review and validation

The implementation reuses native Symbolica ideal arithmetic, saturation,
derivatives, component splitting, checked frames, exact open covers, existing
controlled transforms and history issuance. New code orchestrates their
source-bound owners; it supplies no parallel CAS or blow-up algebra engine.
The generation agent independently reviewed source/history authority, complete
coverage, empty-support treatment, component receipts and ecosystem reuse.
The coordinator independently reviewed these boundaries and imported the exact
frozen source after checking every source/preimage and evidence digest.

The private native suite passes all 109 resolver tests in 152.57 seconds;
strict library/test Clippy and formatting pass. The separately built public
consumer reports the expected retained, incomplete component receipt in
1.07 seconds. The joined Cargo resolver suite passes all 109 tests in
170.92 seconds. Strict workspace Clippy with all targets, formatting and diff
checks also pass.

The frozen handoff SHA-256 is
`2604128fd996919d844ee6b37923e858d509285e1c48edc850e18738de1c899c`.
The initial joined compile caught a required test fixture omitted from the
registration list. The additive correction, SHA-256
`aeed0a4432327e123b54e5bcdaa37d8919f105b32b0c5aa23d2bfe642a7d95be`,
adds that exact fixture already used by the private suite; no test was removed.
Raw probes and logs remain untracked under
`target/no-deformation-bm-nested-carry/`. This milestone neither closes the
four pending charts nor establishes a full resolver or numerical performance.
