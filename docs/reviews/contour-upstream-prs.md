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
