# Ordinary HEPKit builds: dependency preparation audit

Ordinary native and Wasm community builds should resolve their Rust dependencies
through Cargo, without a manual checkout/patch step. The existing preparation
script is a temporary development workaround. Passing tests on patched wheels
are not evidence that unpatched public sources already build.

## Updated public owners, 2026-10-06

Symbolica `community` revision `473b4b8dbc2f9bff8658a047196ba0877238bf9e`
includes three of our corrections: canonical complex-product printing,
attributed-symbol alias preservation, and literal series-variable substitution.
The latter was already upstream in `6589d0c`. The three local patch files and
their bootstrap applications are removed. Upstream's alias implementation also
preserves registered callbacks and user data; the comparison checks behavior,
not just whether the old test-containing diff reverses cleanly.

FeynKit's literal kinematic substitution is published at
`259df8790f27b8d3ef32778cd7195942691b4ef0`. OneLOop's
[PR #1](https://github.com/alphal00p/oneloopmaster/pull/1) is merged into `main`
at `27c3723434b7d99cf70ce612b0b8041d3f5c0e78`, with the same source tree as the
tested PR head. Both superseded patch files are removed. OneLOop declares
`symjit = "2.26.0"`; FastSecDec declares `"2.26.4"`. These are compatible
minimum versions, with actual resolution in lockfiles. Stored OneLOop data is
Symbolica evaluator IR, recompiled on load.

QMC now lives in `crates/fastsecdec-qmc` under the user's revised ownership
instruction. Ordinary registry Numerica supplies RNG, numerical backends and
Havana MC. The bootstrap no longer fetches or patches the Numerica fork.

## Four remaining Symbolica changes

| Change | Reason and limitation |
|---|---|
| Fixed-argument constant-domain fallback | Fixed polygamma values from Gamma expansions need the conditioning evaluator's numerical domain; the generic fallback does not propagate callback-internal uncertainty and remains an unresolved upstream design proposal |
| Evaluator IR decoding validation | Reject deliberately malformed saved programs before evaluation; no faulty generated IR was observed |
| `try_map_coeff_with_prec` | Return unsupported coefficient/callback-domain errors during native/portable evaluator construction |
| `to_polynomial_in_vars_with_field` | Use the existing polynomial converter with a configurable native coefficient field, preserving compact expressions |

All four residual patches apply to the updated public revision. The two named
APIs remain absent from its unpatched source. Removing the bootstrap before
resolving these requirements would break current builds. The constant fallback
is explicitly qualified in the [numerical audit](fixed-constant-error-tracking.md);
a successful conversion test does not establish a sound error bound for an
arbitrary registered callback.

The updated development setup selects four public source owners and applies
only these four Symbolica changes. Historical reproduction notes remain under
[dependency-patches](../dependency-patches/); removed patches are not reapplied.
The user's existing working trees in `/common/dev/` are left untouched.

## Validation and remaining delivery work

The QMC move passes 37 native and 37 portable-feature host tests plus 43 focused
core QMC/MC controls. Version reporting, artifact compatibility and CLI provenance
pass 46 focused tests and scoped strict Clippy. Owner validation passes three
FeynKit regressions, two OneLOop fixture/cache controls and the updated core
one-loop cache regression. Those gates precede the latest Symbolica source
update. The updated public base plus four residual patches subsequently passes
all 23 selected controls: two fresh-process artifact tests, four complex-kernel
tests, the Gamma-regulator regression and sixteen kernel-artifact tests. There
are no failures or skips; the locked graph has one owner per shared crate and
the root lock is unchanged. Compilation and tests take about 497 seconds. The
source comparison and execution receipt are retained under
`output/diagnostics/symbolica-public-update-1/`.

Native, Wasm, stub-generation and core-only metadata graphs have been checked
for the locally prepared removal of the experimental feature. That promotion
and its development lock are not published as a working ordinary public build.
No fresh Python wheel, Wasm runtime or scientific performance claim follows
from these dependency checks.

Once the remaining owner requirements are resolved, select the public Symbolica
source through the consuming workspace's normal Cargo patch table, resolve
lockfiles without path overrides, remove the preparation scripts and their
workflow invocations, and validate ordinary builds. Library crates should retain
registry-based Symbolica requirements so the final consumer controls the single
shared Rust type owner.
