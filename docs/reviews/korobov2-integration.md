# Optional Korobov-2 periodization

The additive numerical transform is published in
[Numerica PR #8](https://github.com/symbolica-dev/numerica/pull/8) at
`bb996e415bee9ae2c143f97408675d3051c3c2aa`. Both author and committer are
`ValentinHirschi <valentin.hirschi@gmail.com>`, and publication used the verified
`ValentinHirschi` account. The existing review request/mention is retained.

## Native owner and reuse review

Public exports, implementations/tests and an external Rust caller established
that Numerica exposed Korobov-3 but no reusable symmetric Korobov-2 transform.
The nearby private geometry smoothstep is Korobov-1. No generic beta-function
implementation, symbolic algebra or integrator was added to fill this gap.

Numerica owns `Korobov2::{map, jacobian, transform_in_place}`. The map is
`t^3 (10 - 15t + 6t^2)`, with Jacobian `30t^2(1-t)^2`; lower-half evaluation and
reflection avoid cancellation near one. Both transforms share the existing
checked product-Jacobian loop. Korobov-3 arithmetic, endpoint treatment,
attenuation guards, Monte Carlo behavior and defaults are preserved.

The independent source/reuse reviewer accepted the additive API and shared
guard. The guard protects representability of the Jacobian product; it does
not certify arbitrary endpoint-sensitive integrands. A mapped interior binary64
coordinate can round to an endpoint. Standalone map/Jacobian functions retain
the existing unchecked-input semantics, while `transform_in_place` validates
coordinates.

Exact Rust 1.89 locked suites pass: 243 default tests, 246 with serde, and 223
with Malachite/Astro plus serde; none failed or were ignored. Six new controls
cover exact Rational values, reflection/derivatives, normalization/moments,
weight range, endpoint behavior and caller-owned parallel vector/covariance
evaluation. Existing Korobov-3 and Monte Carlo controls remain in the suites.
Formatting and scoped Clippy pass; unrelated upstream warnings and the earlier
all-target lint limitation remain documented. Evidence is retained under
`output/diagnostics/numerica-korobov2-1`.

## FastSecDec boundary

FastSecDec dispatches its optional `Periodization::Korobov2` directly to the
Numerica owner. The CLI spelling is `periodization = "korobov2"`; omitted
settings still select Korobov-3. The new enum variant follows the existing
variants, preserving their ordering. Native design/status output, task identity
and checkpoint validation carry the selected transform. No transform arithmetic,
estimator or worker pool is implemented in FastSecDec or Python.

The dependency bootstrap selects the published Numerica commit. Validation uses
a fresh task-owned Numerica checkout and derived native/portable overlays;
previously measured source owners remain unchanged. The HEPKit demo retains its
separately pinned dependency identity until an explicit bridge update.

The accepted FastSecDec gate passes 37 focused tests: 16 integration-session,
11 saved-result, seven weighted-replay and three CLI-configuration tests.
Both callback weighting lanes and checkpoint continuation exercise both
transforms. Mismatched task/return/checkpoint transforms are rejected before
accepting numerical work. Workspace all-target Clippy with warnings denied,
formatting and a portable-host test-target compile check pass. The first
portable command requested a nonexistent library target; the corrected target
selection passes, with the failed command retained. No additional portable or
Wasm runtime qualification is claimed.

The independent final review verifies all eight changed-file hashes, five
unchanged manifest/lockfile hashes, raw test/lint logs and native/portable owner
and backend metadata. Evidence is retained under
`output/diagnostics/native-korobov2-wiring-1`, including
`independent-review.json`. The current HEPKit bridge remains pinned to the
preceding FastSecDec revision. Its periodization parser/getter must admit the
new variant when that pin is advanced; it currently accepts only none/Korobov-3.

This change makes a controlled transform comparison possible; it does not
establish improved physical convergence, performance parity or a reason to
change the default.
