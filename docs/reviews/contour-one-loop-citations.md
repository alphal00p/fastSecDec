# One-loop reduction citation compatibility

2026-10-09. The current upstream Symbolica community `Citation` API adds a
required `url: String`. An independent source inventory found that the existing
one-loop-reduce Python module still initializes the earlier record shape.
Latest upstream `lcnbr/one-loop-reduce:main` was verified at
`b53a70776a43bd14c6562c52a03bc4909568e473` before creating an isolated checkout.
The shared OneLOop/reduction repositories and running notebook environment are
untouched.

The change adds exactly two initializer fields: the existing package repository
URL, and `https://doi.org/{doi}` for its six already-listed papers. Native graph
types, reduction, numerical references, citation identities, BibTeX and usage
tracking are unchanged. The runtime agent independently reviewed both URL
constructions and found no blocker.

A separate complete native `one-loop-reduce-python` consumer passed `cargo
check` in 1 minute 22 seconds. It uses public Symbolica/Numerica `7ec1be4`, public
SymJIT `33100ae`, registry Graphica 3.0.1, and the exact Feynkit citation owner
source `8e3a643` through private path overrides. This is a compile/API gate, not
an additional numerical test or a full refreshed Community-wheel gate.

The two-line change is published as draft
[one-loop-reduce PR 2](https://github.com/lcnbr/one-loop-reduce/pull/2), head
`fae8a926f25a432262b48151802a3ab7febba99b`, authored and published by
ValentinHirschi. It remains draft while the consuming community API/dependency
updates are coordinated; the owner's standalone registry manifest and lockfile
are unchanged. The patch is retained at
[`one-loop-reduce-citation-urls.patch`](../dependency-patches/one-loop-reduce-citation-urls.patch).
GitHub denied formal reviewer assignment; an explicit
[`@benruijl` invitation](https://github.com/lcnbr/one-loop-reduce/pull/2#issuecomment-6088870552)
is posted on the PR.
