# Contour prerequisite pull requests

2026-10-09. The user authorized publishing narrow dependency fixes and requesting
`benruijl` as reviewer. The publishing account was verified as `ValentinHirschi`;
commits use `ValentinHirschi <valentin.hirschi@gmail.com>`.

## SymJIT complex scalar callback lanes

[siravan/symjit#14](https://github.com/siravan/symjit/pull/14) targets the upstream
`v227` branch from the existing `ValentinHirschi/symjit_changes_for_pyamplicol`
fork. Its head is `24017d4f84010716263d3cbc0e7aba2d48b63fe6`. It contains only
the complex gather/scatter correction and regression in `rust/defuns.rs`.
The standalone published-crate reproduction and retained patch remain in this
repository. Full owner tests passed: 2,148 passed, one ignored, on Linux x86_64.

GitHub rejected formal reviewer assignment through both GraphQL and REST for
this account. The requested review invitation to `@benruijl` was posted as
[a PR comment](https://github.com/siravan/symjit/pull/14#issuecomment-6088149269).
It is not recorded as a successful formal reviewer assignment.

The Rust crate's published packaging differs from the upstream Python/C-ABI
repository layout. A standalone Cargo probe fails to import the original Git
crate because its manifest builds only a `cdylib`.

[siravan/symjit#15](https://github.com/siravan/symjit/pull/15) adds an `rlib`
alongside that output and reexports existing native composer API items from
the unchanged C entrypoint. Its head is
`6a49a5b0a95cc12aa3d9ddb2b46c32cca193a54b`, independently based on `v227`.
The external Rust consumer now passes a complex evaluation and version query;
2,147 owner unit tests pass (one ignored), plus the external-consumer test.
An independent runtime-agent review found no blocker. Formal reviewer
assignment was again denied; [the review invitation](https://github.com/siravan/symjit/pull/15#issuecomment-6088355659)
names `@benruijl`. The combined public consumer branch
[`fastsecdec-contour-fixes`](https://github.com/ValentinHirschi/symjit_changes_for_pyamplicol/tree/fastsecdec-contour-fixes)
is published at `33100ae869057f35d9865c933a48bac6699acdd4`.
Its exact head passed 2,148 owner unit tests (one ignored) and the external Rust
consumer test. It contains both narrow PRs and replaces the temporary local
override in consuming workspaces.

## SymJIT native label lengths

[siravan/symjit#16](https://github.com/siravan/symjit/pull/16) fixes the native
MIR and saved-function-table serializers, which previously asserted when a
valid UTF-8 label or callback name required 256 bytes or more. A one-callback
native reproduction isolates the issue without Symbolica or FastSecDec.
The focused branch is based on upstream `v227`
`41ee3e28dd4c4cd8032171400c0604b67fd691b7`, with head
`d309b193fb610e615b84691f94a6ee0c783eac2c`; it does not include the earlier
unmerged callback/API changes.

All existing names through 255 bytes retain their exact encoding. Longer names
use a disjoint UTF-8 escape and checked length, with bounded slice reads and
incremental stream reads. New long-name files require the updated reader.
Three focused regressions cover ASCII/multibyte boundaries, malformed/truncated
records, and actual O2 callback compilation, evaluation and save/restore. The
complete focused-branch owner suite passes **2,151 tests**, with one ignored.
Root and runtime-agent independent source reviews found no blocker.

The PR and its commit are authored/published by `ValentinHirschi`, using
`valentin.hirschi@gmail.com`, and the PR is attached to the task. GitHub rejected
formal reviewer assignment through `RequestReviewsByLogin`; the authorized
[`@benruijl` review invitation](https://github.com/siravan/symjit/pull/16#issuecomment-6091096705)
is recorded separately. This is not a successful formal assignment or a merged
fix.

The compatible combined consumer branch
[`codex/contour-long-label-consumer`](https://github.com/ValentinHirschi/symjit_changes_for_pyamplicol/tree/codex/contour-long-label-consumer)
is published at `d74993ffd76a6fc322a7bcf3963fa786783a38a8`, applying only this
six-file fix to the preceding `33100ae` consumer. Its full owner suite also
passes 2,151 tests with one ignored, and the standalone native 256-byte callback
reproduction now passes. All three maintained manifests/lockfiles select this
public revision, with exactly one changed revision line per file and no unrelated
dependency migration. Their locked metadata admission passes.

The consuming native contour filter passes **77 tests** after rebuilding the
actual public dependency graph. This is a focused dependency acceptance gate,
not a claim that the entire pending dynamic-runtime increment passes: the
separate composed higher-jet test identified incorrectly grouped face-request
identities. That FastSecDec correction subsequently passed complete-vector
controls for both constructions and subtraction methods in eager, SymJIT,
double-float and 192-bit execution. No additional owner CSE patch was needed.

The published dependency-only commit `1137e73113d9cb7fc956f09f1ed20747e7082488`
was also tested in a clean detached worktree, independently of the development
changes: `cargo test --locked -j 2 -p fastsecdec --lib -- --test-threads=1`
passes **297 tests**, with 16 explicit ignored controls, in 25.29 seconds of
test execution. This is the exact published-commit gate; the 77-test filter
above describes the subsequent combined development source and is not added
to this total.

## Serde strict unit-variant settings

The native settings regression reproduces the existing
[Serde issue #2294](https://github.com/serde-rs/serde/issues/2294): an internally
tagged unit variant accepts additional fields despite `deny_unknown_fields`.
The public [container attribute contract](https://serde.rs/container-attrs.html#deny_unknown_fields)
specifies rejection. Owner source inspection identifies the permissive unit
visitor; a new native token regression fails on the unmodified upstream base.

[Serde PR #3109](https://github.com/serde-rs/serde/pull/3109) forwards the enum
policy to that existing private visitor. Its head is
`71de7c9d6b0c3266562eae47e563765608cf151a`, based on upstream `master`
`6693a89cca77e0151437da1c7f890090b9ebf04c`. The patch preserves default permissive
behavior, serialized representation, valid empty maps/sequences, skipped newtype
defaults and `serde(other)`. The foundation agent independently reviewed it.

The stable owner suite passes 406 tests across 21 executables; its compiler
diagnostic UI test is explicitly excluded. All 25 internally tagged enum tests
were rerun after the final unknown-tag fallback control. An alloc-only/no-std
owner check, changed-file formatting and whitespace checks pass. No nightly or
complete minimum-supported-Rust matrix was run locally.

The PR/commit are owned by `ValentinHirschi` with the requested email and attached
to this task. Formal reviewer assignment was denied; the authorized
[`@benruijl` review invitation](https://github.com/serde-rs/serde/pull/3109#issuecomment-6091204967)
records the request without claiming formal assignment. FastSecDec retains the
released Serde dependency and uses a narrow strict settings representation;
this PR introduces no additional consuming dependency fork.

## Symbolica and Numerica

- [Symbolica #55](https://github.com/symbolica-dev/symbolica/pull/55) adds native
  real/complex ball evaluator domains. It targets current `main`, head
  `d9dbf4a0f528d5a5a2ac3a9d0cca6b7c2ade22ec`. The full owner library and three
  focused tests pass on that base. Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/55#issuecomment-6088228662)
  names `@benruijl`.
- [Symbolica #56](https://github.com/symbolica-dev/symbolica/pull/56) adds the
  prepared bracketed root solver on a separate current-`main` branch, head
  `bf61144d3dba718ee0a7b9f64e5ffa363459dac4`. The full owner library, eight
  scalar/failure tests and two prepared eager/JIT evaluator tests pass.
  Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/56#issuecomment-6088260343)
  names `@benruijl`.
- [Numerica/Symbolica #57](https://github.com/symbolica-dev/symbolica/pull/57)
  corrects tracked `hypot` in its Numerica owner crate, head
  `1dc4566cc4591f16c56b565bc907ff86e882aba9`. Six focused native and six
  portable-host tests pass, as do 14 owner API and 15 complex regressions.
  Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/57#issuecomment-6088302905)
  names `@benruijl`.
- [Numerica/Symbolica #58](https://github.com/symbolica-dev/symbolica/pull/58)
  provides certified finite nonnegative real-ball square roots, independently
  based on current `main`, head `59c5eaa04870c94c5c6aa79f52e9960169a01d58`.
  Six exact-rational enclosure tests and ten existing interval tests pass with
  each native and portable backend. An independent source/proof review passed.
  Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/58#issuecomment-6088599198)
  names `@benruijl`.
- [Symbolica #59](https://github.com/symbolica-dev/symbolica/pull/59) exposes
  direct expression-vector evaluation with the owner's existing function cache.
  Head `7553de426d63628f679ed6e7b4a95acb834c8d76` targets `community` base
  `f4e787074d45f1dddd0b9646a5ebc85690deb2d1`. The four-file change preserves
  native lazy branches, explicit function-map precedence, cancellation and
  numeric tracking; it adds no alternate evaluator or algebra. Eight new tests
  and 15 existing evaluation tests pass, with one existing stress test ignored,
  in the direct owner build using cached native dependencies. This is not a
  claim that the full owner Cargo suite ran. Independent source/API review
  passed; the [exact-vector audit](contour-exact-map-audit.md) records the
  public-API, source and focused-probe evidence for the missing operation.
  GitHub rejected formal `benruijl` assignment with `RequestReviewsByLogin`;
  the [explicit review invitation](https://github.com/symbolica-dev/symbolica/pull/59#issuecomment-6091614060)
  requests his review. The PR is attached to the task.

All PRs are authored and published by ValentinHirschi and attached to the task.
The tracked scalar forwarding candidate was not adopted: enabling ordering
also activated complex square-root/division shortcuts that lost uncertainty.
A narrower tracked `hypot` owner fix is published without changing global
branch semantics. No unreviewed correction has been adopted by
FastSecDec or the isolated fixed-mode HEPKit wheel.

The combined public Symbolica/Numerica consumer branch
[`codex/contour-owner-integration`](https://github.com/ValentinHirschi/symbolica/tree/codex/contour-owner-integration)
is published at `7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, based on upstream
`community` revision `f4e7870`. It includes the locally validated changes from
the still-unmerged PRs 55–58 and the exact
existing commits of [PR 54](https://github.com/symbolica-dev/symbolica/pull/54),
which supplies native evaluator composition and output pruning already required
by FastSecDec.

The current consumer extends that exact revision with only PR 59's four-file
API change, at `1e1cb169bec35ed3b8536050f789321f063a2047` on
[`codex/contour-direct-vector-consumer`](https://github.com/ValentinHirschi/symbolica/tree/codex/contour-direct-vector-consumer).
The combined owner build repeats the eight new and fifteen existing passing
evaluation tests, with one existing stress test ignored. All three FastSecDec
manifests/lockfiles select this public revision without a dependency migration.
Their `cargo metadata --locked` checks pass with one Symbolica/Numerica identity
per consumer; the portable graph correctly excludes SymJIT. FastSecDec's next
runtime build remains a separate gate from these owner and resolution checks.

The initial combined revision `eccd039` passed its leaf owner controls but
failed the actual FastSecDec consumer build: upstream `community` does not yet
contain PR 54, unlike FastSecDec's previous `1deccb8` fork. The correction
cherry-picks those original commits without reimplementing or dropping newer
upstream changes, and appends the independently reviewed real-square-root fix.
No history was rewritten.

The corrected exact head passed full native Numerica/Graphica/Symbolica
rebuilds, three composition tests, three ball-domain tests, eight bracketed-root
tests, two eager/JIT prepared-evaluator tests, six tracked-hypot tests and six
certified-square-root tests. The last two six-test suites also pass on the
portable backend. FastSecDec's consuming native/portable checks remain a
separate gate. Individual PRs 55–58 remain independent against upstream `main`.

## FeynKit citation URL compatibility

[GammaLoop/FeynKit #128](https://github.com/alphal00p/gammaloop/pull/128)
targets `feynkit` from the same repository's `symbolica-citation-urls` branch,
head `8e3a643f388b45939d6573a648ef3a509086835e`. It adds exactly 16 required
`Citation.url` fields to the existing FeynKit, Spynso3 and Vakint records.
The links use the records' existing repository, arXiv or DOI identifiers;
no bibliography, usage tracking or numerical implementation changes.

Source inspection confirmed the required `String` field in Symbolica's
`api/python/citation.rs`. A coherent native external consumer compiled all
three owner crates, including Vakint's `symbolica_community_module` feature,
against the public Symbolica/Numerica head `7ec1be45` and SymJIT `33100ae`.
Formatting and diff checks passed, and an independent source review approved
the links. The commit and PR are authored by ValentinHirschi; GitHub confirms
the formal reviewer request to `benruijl`. The PR remains draft because only
focused compatibility checks have run, rather than the monorepo's complete
CI and cache-upload final-review workflow.

The isolated Community host's Vakint revision `6203c6c` has an additional
RustRed citation and substantial native RustRed functionality absent from
current `feynkit`. Its compatibility update must preserve that branch: apply
the five common URL additions plus the existing RustRed repository URL to its
sixth record. Replacing that owner wholesale with current `feynkit` is not a
valid citation-only update.

## One-loop reduction citation URL compatibility

[one-loop-reduce #2](https://github.com/lcnbr/one-loop-reduce/pull/2) is a draft
against current upstream `main`, head
`fae8a926f25a432262b48151802a3ab7febba99b`. It adds only the two required URL
initializers to the existing Python citation records. The complete native
Python consumer compiles with the coherent public contour owners and FeynKit
citation compatibility source, and an independent review approved both links.
The standalone registry manifest/lock and numerical reducer remain unchanged.
The commit and publishing account are ValentinHirschi. Formal reviewer
assignment was denied; [the explicit review invitation](https://github.com/lcnbr/one-loop-reduce/pull/2#issuecomment-6088870552)
names `@benruijl`. The [focused audit](contour-one-loop-citations.md) records the
source identities and exact compilation scope. Core numerical references keep
their existing `b53a707` source until a separate consumer update is required.

The host-specific correction is published separately as
[Vakint/RustRed fork #1](https://github.com/ValentinHirschi/gammaloop/pull/1),
head `854e849570cf44e115063e97d612090a0ff48121`, targeting that fork's existing
`codex/vakint-shared-rustred` branch at `6203c6c`. Only six URL fields in
`crates/vakint/src/citations.rs` changed. Its coherent native consumer compiles
with `symbolica_community_module`, public Symbolica/Numerica `7ec1be45`,
FeynKit `8e3a643f` and the host's retained RustRed `7c1ed037`. Compilation
reports 11 unused-code/import warnings in unchanged Vakint implementation;
none were suppressed. Formatting, diff and independent source reviews pass.
These gates do not claim numerical-integral execution or complete monorepo CI.

The commit and draft PR are authored by ValentinHirschi. GitHub's REST endpoint
rejects a formal reviewer request because `benruijl` is not a collaborator of
this fork; the CLI's create/edit commands did not preserve the request either.
The [explicit review invitation](https://github.com/ValentinHirschi/gammaloop/pull/1#issuecomment-6088897751)
records the requested review without claiming a successful formal assignment.

## OneLoopMaster citation compatibility

The current Symbolica community API requires `Citation.url`. The isolated
OneLoopMaster patch adds three fields using each record's existing BibTeX URL,
without changing the scalar masters or their already-merged numerical fixes.
It is based on public `main` `27c3723434b7d99cf70ce612b0b8041d3f5c0e78`.

- PR: [OneLoopMaster #3](https://github.com/alphal00p/oneloopmaster/pull/3).
- Head: `62ae35d6b7ab4caa94ecf7d0caed95cc190d878d`, branch `citation-urls`.
- Author: `ValentinHirschi`; formal reviewer request: `benruijl` (verified).
- Native external-consumer `cargo check` of `oneloop-python` with
  `community,prebuilt` passed against public Symbolica/Numerica `7ec1be45` and
  SymJIT `33100ae` in 44.44 seconds. This is a compilation/API gate, not a
  numerical integral rerun.
- Changed-file formatting, `git diff --check` and independent source review
  passed. Recursive formatting reported an existing attribute layout in the
  untouched `python/src/inspection.rs`; the patch does not modify it.

The corrected fixed host retains the existing main-based master APIs. The
newest Community host separately needs the positive-epsilon branch
`6c9874dc5670ed655b9cce84702ec27d36098a9f`; its Python citation file is identical,
so the same three-field fix can be applied there for that later host gate.
No unrequested numerical upgrade or second PR is included in this slice.

## FeynKit contour interface metadata

[FeynKit PR 129](https://github.com/alphal00p/gammaloop/pull/129), head
`917b20751ea5b05c38b5b027e54f236e93d44699`, is a draft stacked on PR 128. It
aligns diagram/family method signatures and stub templates with the actual
FastSecDec contour/runtime-parameter interface and corrects the regular-backend
error text. Native forwarding is unchanged. The actual linked owner test and
parsed stub-schema comparison pass; independent reviews agree. The commit and
publishing account are ValentinHirschi, and GitHub confirms a formal review
request to `benruijl`. See [the interface audit](contour-python-interfaces.md).

## Community citation metadata

[Community PR 25](https://github.com/symbolica-dev/symbolica-community/pull/25),
head `48d1d745f42233f4bd04696ea127c7d0f2e7aa58`, adds exactly six URL fields
across three existing citation owners against current main `9a65fbb7`. It is
authored and published by ValentinHirschi, remains draft for the coordinated
owner API update, and has a confirmed formal `benruijl` reviewer request. The
exact six edited constructor literals compile against public Symbolica
`7ec1be45`; independent source review passes. No dependency rollout or live
notebook deployment is included. A fork upload was rejected because historical
workflow files were absent from that fork's base; the unchanged six-field
commit was successfully published to an authorized upstream feature branch,
without workflow edits, force-pushing or switching accounts.

## Hyperbolica community adapter

[Hyperbolica PR #1](https://github.com/benruijl/hyperbolica/pull/1), head
`ac84d6ed484b09176802f86824ccb8771a871f5b`, adds three citation URL fields to
the current `codex/hepkit-integration` adapter at `312920855`. It preserves
that adapter and its recent owner fixes; current upstream `main` has a
different standalone Python interface. The unchanged adapter fails with three
missing-field errors under the current Symbolica API, and the corrected
coherent consumer compiles successfully. The draft PR is authored and
published by ValentinHirschi. GitHub denied formal reviewer assignment; the
PR explicitly invites `@benruijl` and records that limitation. See the
[focused source and compilation audit](contour-hyperbolica-citations.md).

## Direct-translation external-expression cache

[Symbolica PR 60](https://github.com/symbolica-dev/symbolica/pull/60), head
`85a993fd9b8d9e25099261dff476c64afac15f7c`, adds a missing insertion into the
existing scope-local expression cache. Registered external calls otherwise
repeat when direct translation bypasses later optimization; numerical values
were already correct. Six focused real/complex, alias, tag and lazy-branch
controls pass, along with 15 existing evaluator tests and one existing ignored
stress test. Independent reviews pass. The commit and publishing account are
ValentinHirschi. Formal reviewer assignment was denied by GitHub permissions;
an explicit PR comment invites `@benruijl`. The combined consumer child
`516beb37d31af8e3d6ee321a7070f407a0b1b42d` is published on
`codex/contour-external-cache-consumer`; all six new controls, 15 existing
evaluator controls and eight direct-vector controls pass on that exact head,
with one existing stress test ignored. The three maintained consumer manifests
now select that revision. The consuming core/QMC/sectors gate passes 692 tests
with 24 explicit ignores, the CLI passes 168 with eight ignores, the portable
host passes 76, and actual Emscripten/Wasm passes the three public dynamic
controls. Strict workspace all-target Clippy passes. Installed current-host
Python and physical multiloop acceptance remain separate gates.
See [the owner reuse review](contour-direct-external-cache.md).

## In-place matrix determinant row-swap sign

[Symbolica PR 61](https://github.com/symbolica-dev/symbolica/pull/61), head
`74d712d0e50692012c9c45778a06efd72f7c33ef`, fixes Numerica's
`Matrix::det_in_place` against upstream community
`f4e787074d45f1dddd0b9646a5ebc85690deb2d1`. The two-row permutation matrix
`[[0,1],[1,0]]` returned `+1` instead of `-1`: its existing row reducer discarded
swap parity. A private parity-returning adapter now shares that same reduction
and applies the sign once for a full-rank determinant. Public reduction, rank
and solve interfaces remain unchanged.

The freshly compiled baseline fails two odd-swap controls. The patch passes all
five new odd/even/late/singular/rectangular regressions, 176 existing Numerica
unit tests and 14 existing API tests: **195 distinct passes, zero ignores**.
These are direct-rustc owner tests with a coherent cached native GMP/MPFR graph,
not a claim of complete Symbolica workspace CI. Changed-file formatting,
whitespace checks and independent root source review pass.

The commit and publication account are `ValentinHirschi`, with author email
`valentin.hirschi@gmail.com`. The PR is attached to this task; the root's
successful attachment supersedes the earlier unconfirmed tool attempts.
GitHub denied a formal `RequestReviewsByLogin` assignment. The explicit
[`benruijl` review invitation](https://github.com/symbolica-dev/symbolica/pull/61#issuecomment-6094531274)
records that limitation; no formal assignment or merge is claimed.
FastSecDec's contour path uses the already-correct `Matrix::det()`, so this
separate owner defect does not require a consumer revision change. See the
[higher-dimensional determinant audit](contour-higher-dimension-determinant-audit.md).
