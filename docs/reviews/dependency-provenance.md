# Local dependency source identity

The coordinator reproduced a provenance defect in a temporary Git repository:
after a tracked module added `pub mod added`, changing the untracked
`src/added.rs` from one implementation to another left `git diff --binary HEAD`
identical. The build script previously hashed only that diff and could therefore
record the same dependency identity for different locally compiled sources.

The build helper now extends the tracked patch hash with sorted, length-delimited
paths and contents of nonignored untracked files. Symlink records include the
target path and resolved file contents. The existing identity is unchanged for
tracked-only modifications, preserving current artifact compatibility. Git or
file-read failures are errors, rather than evidence of a clean checkout.

Cargo watches those files and the native source trees, including Symbolica's
`lib` directory containing Graphica, plus manifests, the lockfile, build scripts
and ignore rules. A tracked module edit that introduces a new source file thus
triggers a fresh scan. Ignored build outputs remain excluded. This is source
provenance, not a claim to fingerprint the complete toolchain or arbitrary
external inputs consumed by a dependency's build script.

Three focused tests passed: untracked module creation/edit/removal, stable
identity across staging and ignored build output changes, and rejection of
missing Git provenance. The original reproduction is retained in ignored
`output/probes/dependency-provenance.hnQtN9`; the test log is
`output/dependency-provenance-tests.log`.

The independent reviewer checked the helper, build-script watch paths, tests,
and actual dependency layout. Sorted raw UTF-8 path bytes, explicit record/type
markers and length-delimited fields avoid path/content concatenation ambiguity.
The tracked diff includes staged and unstaged changes, and external diff/textconv
drivers are disabled. The reviewer confirmed Graphica is tracked under the
Symbolica checkout's `lib/graphica`, which the new directory watch includes.

One practical watch defect was identified and corrected: emitting a Cargo watch
for an absent `packed-refs` or loose branch reference forces perpetual rebuilds.
An isolated two-build probe reproduced Cargo's `the file ... is missing` dirty
reason (`output/probes/dependency-watch/second.log`). The final script watches
index/packed-refs only when present and uses the nearest existing ancestor of a
missing loose branch reference, preserving detection of subsequent ref creation.
The reviewer inspected the correction; no unresolved finding remains in this
scope. Git failures and unsupported/unreadable filesystem entries remain explicit
errors instead of weakening identity. Linux execution was verified; this source
review does not substitute for the project's pending macOS runtime gate.
