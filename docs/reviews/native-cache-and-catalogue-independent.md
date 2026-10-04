# Native support reuse and published lattice review

This review is independent of the mapping-cache and Numerica catalogue authors.
It examines native ownership, scientific invariants, serialization and future
library/API boundaries. The initial review is source/design evidence; execution
results are recorded separately when the authors' focused gates complete.

## Generation-local source support cache

Reviewed `generation/support.rs`, `generation/mod.rs`, `generation/mapping.rs`,
the existing `PolynomialFactor::support` adapter, and
`tests/support_reuse.rs`, against the measured
[mapping attribution](rank-five-mapping-attribution.md).

The cache owns one fixed ordered native `Vec<Symbol>` per `generate` invocation.
Keys are native immutable `Atom` values compared by Symbolica, and values are
the existing exact `fastsecdec_sectors::PolynomialSupport`, whose exponents use
the shared native integer type. There is no new polynomial representation,
canonicalizer, evaluator or arithmetic. Its only computation on a miss is the
same `factor.support(parameters)` call used before this change.

Both original singular supports needed by geometry and subsequent mapping
requests use this owner. The mapped coordinate-face fast path still runs first
for regular factors, so compact large powers are not enumerated just to fill a
cache. The key correctly excludes factor role and exponent: these do not change
the original polynomial's support. Mapping still applies each occurrence's own
exponent, role, coordinate map, prefactor and residual/domain checks. Mapped
expressions and certificates are not cached. Failed collections are not inserted.
Dropping the generation owner, including on cancellation/error, drops the cache;
there is no cross-input or cross-parameter-order state.

No scientific or ecosystem-reuse issue was found in that implementation. The new
independent integral test uses the same polynomial in two different roles and
with different occurrence powers. Existing signed orthant, hidden-cancellation,
large factored-power, literal-symbol and complete-vector tests remain necessary:
the cache does not replace their admission or proof obligations. Test-only
profiling/bypass state does not enter the public API or production configuration.

The change preserves native objects between modules and adds no serialization
boundary. Future Python bindings can continue to call the same generation API.
Future maps may reuse an unchanged source support, but any new representation
must first satisfy the existing polynomial adapter; this cache does not give
opaque evaluator aliases polynomial semantics or provide phase-two algorithms.

## Published lattice catalogue design

Reviewed [the native proposal](qmc-catalogue-proposal.md), the existing Numerica
`numerical_integration/qmc/rule.rs` implementation, FastSecDec
`integration/config.rs`, CLI checkpoint/refinement code, and the separate
[constant and physical controls](six-line-qmc-convergence.md).

The proposed ownership is appropriate: Numerica stores attributed published
numerical data, exposes the native `PublishedLattice` identity and capability
bounds, constructs `Rank1Rule`, and checks serialized provenance against the
entire selected generator. FastSecDec wraps that native enum as an explicit
`RuleSource::Published` choice. Neither FastSecDec nor its CLI should copy a
table, reproduce modular arithmetic, search generating vectors, or estimate
errors through a second implementation. Existing QMC work packages, caller-owned
workers, shift accumulators and complete-vector covariance stay in place.

Kuo33002 remains the old default and retains the exact meaning of serialized
`Kuo`/`Kuo33002`. The supplied-vector API remains available. Each alternative
must reject unsupported dimension/count requests without substituting a different
catalogue. An all-exact integral still validates the selected count. Complete
power-of-two point sets admit either linear or radical-inverse ordering as a
permutation; the proposal does not claim support for arbitrary all-N sequences.
Native published bounds are capability limits, not promises of good uncertainty.

The new enum and bounds are suitable for later HEPKit bindings: callers choose
the catalogue explicitly, receive typed errors and metadata, and retain control
of work/parallelization/stopping. A catalogue choice changes numerical sampling
identity and checkpoint compatibility, while leaving the symbolic kernel identity
unchanged. CLI refinement must read the selected native bounds, grow point counts
within them, then grow independent shifts at the cap. Missing historical CLI
fields must retain Kuo33002 meaning; a new field must not silently invalidate or
reinterpret a resumed session. Explicit alternatives require distinct settings
identity and cannot reuse another catalogue's partial production samples.

The documented Apache-2.0 LDData distribution pin, author attribution, complete
component comparisons and source/payload hashes are the proper data-import
boundary. No integration code is copied and no runtime download is needed.
Implementation review must confirm those files and hashes after import; the
proposal alone is not executable provenance validation.

No design blocker was found. The physical five-axis result and nine-dimensional
controls argue for explicit choices and broader evidence before a default
change, exactly as proposed. Focused tests should cover bounds, full-vector
provenance tampering, worker partition/checkpoint identity, old Kuo behavior and
the independent periodized constant control. Those are pending at this initial
review; no catalogue implementation or changed default is claimed here.

## Imported implementation and physical evidence

The subsequent implementation review inspected Numerica's `qmc/catalogue.rs`,
`rule.rs`, `mod.rs`, the complete imported source/payload files and
`tests/qmc_catalogues.rs`, together with FastSecDec's native settings adapter.
There is no implementation blocker. `Rank1Rule::published` owns the bounds and
generator construction; serialized published provenance is checked against the
complete selected vector. Old `Kuo`/`Kuo33002` identities and defaults remain
unchanged. Point counts are validated even for all-exact inputs, and unsupported
dimensions produce errors instead of fallback rules.

All six new source/payload SHA-256 values were independently recomputed and
matched the native data README: HKKN source/payload `c423a2d9…`/`e961023a…`,
Kuo38005 `d6558c9d…`/`101369d6…`, and Kuo39101 `2a17256b…`/`682ebbde…`.
The full values, original author attribution and LDData revision
`5c55b76bf6b6aba3415a0dece9cc1269ff1be883` remain in that README; the Apache-2.0
license and notice are retained. This is a numerical-data import, with no copied
integration algorithm or runtime download. Root's separate provenance audit
confirmed that dependency source-state hashes cover binary diffs and every
nonignored untracked file, including these embedded payloads, before and after
the native commit; recursive source watching covers rebuild invalidation.

The [48-run physical campaign](qmc-catalogue-cross-case.md) was independently
checked against its Rust probe and JSONL inventory. Four fixed artifacts cover
dimensions 4–7, four catalogues and three seeds, with 8192 points and 16 shifts
per run. Each case retains one unchanged kernel content ID, every complete shift
vector, and zero evaluation failures. The probe sums sectors within each shift
using the existing session API, pairs catalogues by `(seed, shift)`, and delegates
joint covariance to native `QmcEstimate::from_shift_means`. Shift generation is
independent of the selected generator, so the paired covariance has the stated
meaning. The exploratory runs used native supplied vectors before the catalogue
API existed; their recorded vectors match the imported published data.

HKKN improved the reported standard error over historical Kuo33002 in all four
cases, while other catalogues won individual cases. This supports exposing
choices; it does not establish a universal winner, convergence rate, or matched
performance claim. No default change is part of this implementation.

The catalogue author reports 29 native QMC tests (26 existing plus three catalogue
tests), nine existing MC tests and 26 FastSecDec runtime tests passing. These
exercise data/bounds, provenance tampering, native restore, adaptive allocation,
constant-integrand quality controls and existing Monte Carlo behavior. This
review's own CLI integration and checkpoint-boundary gate is recorded below once
executed; those results are distinct from the author's native test evidence.

## CLI boundary validation

The subsequent focused CLI gate passed 23 tests: 15 unit tests, two catalogue
process tests, four existing CLI process tests and two reference process tests
(`output/cli-catalogue-tests.log`). A separate exact shipped massive-box D0 test
passed, as recorded in the native one-loop review. The CLI source was also
independently read by the catalogue author after implementation; no additional
blocker was found.

The process tests integrate one unchanged artifact with all four explicit choices,
check native rule identities and effective allocations in the final report, and
resume with a changed worker count without repeating production. They exercise
the TOML selector, HKKN's native minimum, historical Kuo's different minimum and
HKKN's full-dimension rejection. Library tests provide the multidimensional
catalogue-quality controls; a one-dimensional CLI plumbing test is not evidence
of comparative lattice quality.

Root's checkpoint review found a latent boundary gap: a separately valid native
session could previously disagree with the outer CLI settings. The driver now
compares the restored native design and method with the expected selected
settings at the saved refinement round before constructing workers. Dedicated
tests reject valid-but-mismatched catalogue, method and round sessions without
modifying their checkpoint files. Legacy missing fields retain their historical
meaning; native `QmcDesign` is exposed directly, including adaptive allocations.
Sampling overrides leave the symbolic kernel unchanged. The CLI envelope's
existing source-card fingerprint still includes integration steering, so this
does not claim that editing a saved artifact's source card leaves its identity
unchanged.

The cache author subsequently completed the untimed same-input bypass/enabled
comparison with identical full Laurent-vector fingerprint
`59b852056e2efe644dd381f1969a3ed6b9cee8710ca73bb293b5a5625c17b6f7`.
This supplements the 61-test scientific gate and the separately recorded three
paired release timing runs; it is exact representation-equivalence evidence,
not an additional performance measurement.
