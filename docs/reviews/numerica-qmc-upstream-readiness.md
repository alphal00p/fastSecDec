# Numerica QMC upstream-readiness review

The reviewed head is `e4638da22a17cfa931fa14c6829d3350b7a8de2b` on
`codex/havana-qmc`. A fresh fetch from
`https://github.com/symbolica-dev/numerica` confirmed the target `main` at
`a8a8fcb` (Numerica 3.0.1): the feature branch is seven commits ahead and none
behind, with a clean checkout. The coordinator independently reviewed the
branch scope and approved publication after these gates. User authorization
now includes the feature-branch push, a PR against upstream main, and requesting
`benruijl` as reviewer (the user's corrected spelling); publication remains coordinator-owned.

## Scope and API

The entire diff adds the separate `numerical_integration::qmc` lane, documentation,
the serial/threaded example, tests and attributed numerical tables. Outside that
lane, production code changes only add its module export. The sole manifest
addition is test-only `serde_json`. Existing Havana grids, Monte Carlo arithmetic
and RNG sequences are unchanged.

Public rules, shift plans, work packages, partial returns and accumulators remain
caller-driven. Coordinates are generated on workers into caller buffers; no
integrand callback, worker pool, adaptive integration loop, stopping policy or
CBC search is introduced. Covariance comes from complete independent shift
means, and correlated integrands must be summed within a shift before variance
estimation. Package identity binds rule/randomization/shape, with overlap and
gap checks. The caller remains responsible for its integrand identity.

The review revisited the modular arithmetic, small periodic residues, RNG prefix
contract, canonical package reduction, offset-centered covariance, serde
constructor validation, failed partials, exact endpoint versus lost-range
periodization behavior, and count-only coverage inspection. No essential source
fix was needed in this review. Numerical limitations remain explicit: binary64
coordinates/observations, conservative range rejection, and integrand-dependent
lattice/periodization quality. No universal performance claim is made.

## Data and portability

All four payload hashes and the three preserved source-text hashes match the
documented immutable imports. Original source revisions, author references,
format conversions and Apache-2.0 data notices are retained under the QMC data
directory. The integration implementation remains MIT licensed; no Python or
C++ integration code was imported. `cargo package --list` includes the vector
payloads, source text and attribution/license files.

The declared Rust minimum remains 1.89. The new fixed-array chunk API is stable
since 1.88, within that baseline. Tests here used Rust 1.98.1 on Linux; an exact
1.89 toolchain run and a macOS run were not performed in this review. Both native
GMP/MPFR and alternative Malachite/Astro backends were exercised.

## Validation

| Command | Library | Integration | Documentation | Result |
| --- | ---: | ---: | ---: | --- |
| `cargo test --locked` | 174 | 41 | 22 | Pass |
| `cargo test --locked --features serde` | 174 | 44 | 22 | Pass |
| `cargo test --locked --no-default-features --features integer-malachite,float-astro,serde` | 151 | 44 | 22 | Pass |

The serde configurations include all 30 QMC tests, and all full library gates
include the existing nine Monte Carlo tests. The serial/caller-threaded example
also ran: 32 shifts and 131,072 evaluations produced matching estimates of 1/4
and 3/4. `cargo fmt --check` and `git diff --check` passed.

Scoped Clippy for the library, three QMC test targets and example completed with
no QMC diagnostics. Whole-crate all-target Clippy is not clean: 29 existing
deny-by-default `approx_constant` errors occur in unchanged
`src/domains/float/complex_tests.rs` and `src/domains/integer.rs`, alongside
existing warnings. Those unrelated source files were not changed. The PR body
states this limitation rather than claiming a clean all-target lint gate.

Local evidence is retained as `output/numerica-upstream-{default,serde,alternative}-tests.log`,
`numerica-upstream-example.log`, `numerica-upstream-fmt.log`,
`numerica-upstream-package-files.log`, `numerica-upstream-clippy.log` and
`numerica-upstream-qmc-clippy.log`. Exact PR title/body drafts are in the
coordinator's temporary files.

## Publication

[Numerica PR #8](https://github.com/symbolica-dev/numerica/pull/8) is open against
upstream `main`, with head `e4638da22a17cfa931fa14c6829d3350b7a8de2b` on
`ValentinHirschi:codex/havana-qmc`. The authenticated account has read access to
upstream, so the branch was published through its fork. Upstream main was not
modified or merged. The PR is attached to the implementation task.

GitHub denied the formal reviewer assignment, including the retry with the
user's exact corrected username `benruijl`, because the account
lacks permission for `RequestReviewsByLogin`. The PR description instead
explicitly tags `@benruijl` for review; that mention was verified after updating
the PR. At publication, the `license/cla` check was pending. No contributor
agreement was signed on the user's behalf. The PR description records the
complete successful test gates and the preexisting all-target lint failures.
