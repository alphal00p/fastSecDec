# RustFlow citation compatibility with the updated host

2026-10-09. The updated public Symbolica Python `Citation` has a required `url`
field. RustFlow's existing helper omitted it. This is an owner API compatibility
change, not a FastSecDec numerical change.

Latest upstream RustFlow main was checked before editing:
`b3a4843e8327835d1ec3ada5a6f32f1841bab2c2`. It still had the omission previously
observed at the private host's `9599e35` pin. Separate isolated checkouts were
used for the untouched baseline and correction; the shared host and any live
RustFlow checkout were not changed.

The correction adds one explicit URL argument to the existing citation helper,
its native field assignment, and the four arXiv URLs already present in the
records' BibTeX metadata. The existing process-isolated citation regression
also retains and checks the URL. No manifest or lockfile change is included.

The public correction is [RustFlow PR #3](https://github.com/alphal00p/RustFlow/pull/3),
commit `f0885454de06b26bd909c975ad1ae8c439754dc7`, on
`codex/symbolica-citation-urls`. It is a draft, authored and published by
`ValentinHirschi <valentin.hirschi@gmail.com>`, with a formal `benruijl` review
request. The coordinating agent verified the published metadata and attached
the PR to the task.

## Validation and its limits

An external consumer selected RustFlow's `python` feature, including native
and automatic functionality, with this coherent owner graph:

| Owner | Revision |
| --- | --- |
| Symbolica, Numerica, Graphica | `7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca` |
| SymJIT | `33100ae869057f35d9865c933a48bac6699acdd4` |
| Feynkit, Linnet, Spenso and companion packages | `8e3a643f388b45939d6573a648ef3a509086835e` |
| RustRed | `91a877e1` from its public main selection |

The Feynkit correction is separately reviewed in
[gammaloop PR #128](https://github.com/alphal00p/gammaloop/pull/128). Native
package identities were unified in the consumer; the helper does not introduce
a second Symbolica Python type or extension.

The unmodified RustFlow baseline reproducibly failed its library check with
`E0063: missing field url in initializer of Citation` at
`src/python/citations.rs:31`. With the correction, the library and existing
`python_portable_higgs_seeds.rs` test target passed `cargo check`, including its
updated citation regression. The final clean incremental check completed in
7.54 seconds. The test target was **compiled, not executed**; no installed
Python-wheel or runtime-regression result is inferred from it.

The ignored consumer is `target/rustflow-citation-check`, with evidence in
`target/contour-rustflow-citation-baseline.log` and
`target/contour-rustflow-citation-fixed.log`. Its only build-target reuse was
sequential, after the Feynkit owner gate released the binding target and before
the one-loop-reduction gate took it. No concurrent shared Cargo target was used.

RustFlow's unchanged standalone lock still selects the earlier Symbolica
`ed2374f` API without this field. Consequently its old standalone `--locked`
Python build is **not claimed to pass** with the correction. The draft PR is
for the updated embedding host and should be merged with a coordinated owner
dependency refresh. This narrow compatibility gate is separate from the
FastSecDec public-source workspace tests and the earlier private 3+205
installed-host tests.
