# Ordinary upstream dependency delivery

FastSecDec now selects public Symbolica `community` revision
`58652fabc2f736302a570deaaf8d517679f7fe6e` through ordinary Cargo source
selection. This revision supplies the configurable coefficient-field API.
There are no local Symbolica source patches or dependency-preparation scripts.
The historical patch proposals remain documented under
[dependency-patches](../dependency-patches/) and in Git history.

## How the remaining requirements were resolved

| Previous local change | Current implementation |
|---|---|
| Configurable fixed-variable coefficient field | Upstream `to_polynomial_in_vars_with_field`; existing compact homogeneity, monomial and domain checks retain their deterministic zero policy |
| Fallible coefficient mapping | Preflight native callback metadata and target-domain conversion through public Symbolica APIs, then call upstream `map_coeff_with_prec` |
| Generic fixed-constant fallback | Removed; ordinary and multiprecision Gamma/polygamma implementations already exist upstream. Unsupported error-tracking conditioning skips that shortcut and uses existing precision rescue for flagged points |
| Native evaluator decoding validation | Removed as an optional owner hardening change; saved evaluator IR is a trusted application cache, with the existing outer envelope and compatibility checks |

The admission adapter does not implement arithmetic, special functions, a
coefficient mapper or an IR validator. It collects callback requirements at
kernel construction and shares them with worker-local precision caches. Native
callbacks retain their own numerical contract. It does not assign a fictitious
error estimate to an arbitrary callback result. The earlier
[constant-error audit](fixed-constant-error-tracking.md) records the motivation.

Native artifacts preserve their original bytes and IDs. Their loader is not a
validator for arbitrary rewritten instruction streams. No malformed IR was
observed from generation; the removed decoder regression deliberately changed
an instruction index and depended on the discarded upstream patch.

## Public dependency ownership

Root, standalone Python and portable consumers each select one public Symbolica
owner. FeynKit is locked at tested public revision
`259df8790f27b8d3ef32778cd7195942691b4ef0`, OneLOop `main` at
`27c3723434b7d99cf70ce612b0b8041d3f5c0e78`, and ordinary Numerica/Graphica at
registry version 3.0.1. SymJIT remains a compatible minimum with exact resolution
in the lockfile. QMC belongs to FastSecDec.

Resolving without path overrides exposed two Git source identities for the
same OneLOop commit: one revision request and one branch request. The core now
uses the same `main` source as the reducer, with its revision fixed by the lock.
No unrelated package version upgrade is needed. The root's only local packages
are FastSecDec's own workspace crates; the standalone consumers likewise use
only their own FastSecDec source paths.

The standard Cargo `[patch.crates-io]` table selects the public Symbolica Git
package for ecosystem consumers. It is source selection, not a modification of
upstream source. No generated Cargo home, manual checkout or source-root
environment variable is needed for normal builds. Local developer overrides
remain possible but are excluded from ordinary-delivery verification.

## Validation and HEPKit delivery

Ordinary locked metadata passes for the root and both native/portable variants
of the standalone Python and portable validation consumers. The maintained
owner checker confirms a single shared ecosystem identity; the portable target
tree has no active GMP dependency. Existing package versions remain unchanged.
Native workspace tests pass: 475 passed, zero failed, and 25 existing ignored
diagnostic tests. All five callback-admission controls pass, including fixed
special-function rescue in real and complex arithmetic. The nine maintained
portable Rust targets pass all 58 controls on the host. Strict all-target
workspace Clippy passes, as does the standalone Python binding's all-target
check with `python_stubgen`. Root and both standalone-consumer formatting checks
pass. These checks use ordinary public dependencies without Cargo path overlays.

The corresponding [HEPKit PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18)
update includes FastSecDec in the ordinary `community` feature and removes the
experimental opt-in and preparation workflow. Its final dependency pin follows
publication of this FastSecDec milestone. Generated stubs and numerical
implementation ownership remain unchanged: community links/registers;
FastSecDec owns the binding crate.

Receipts are retained under
`output/diagnostics/symbolica-ordinary-cargo-1/`. Initial compile errors in the new
adapter/tests and bounded linking timeouts remain recorded separately from the
successful corrected run. The native test run was followed only by routine
rustfmt corrections and removal of an unused malformed-IR fixture.

Earlier 118-test native and 89-test actual Wasm wheel results predate this
migration. They are historical validation, not a claim of freshly rebuilt
wheels or a new physical ggHH convergence run.
