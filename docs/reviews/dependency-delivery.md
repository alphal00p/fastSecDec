# Reproducible dependency delivery

This document retains the original bootstrap/delivery evidence. Substantive
Python bindings now belong to FastSecDec; community links/registers them. The
[current public metadata/MC delivery](hepkit-metadata-mc-portable.md) records
`a3d09e`/`c9bacce` source ownership and actual native/Wasm runtime acceptance.

This change replaces reference-worktree paths in the published FastSecDec
manifests with public sources and a required, explicit dependency bootstrap.
It changes source delivery and CLI provenance, not numerical algorithms or
backend selection. The numerical library remains Rust; the Python bridge remains
in symbolica-community.

## Source ownership and bootstrap

`scripts/bootstrap-dependencies.sh` fetches exact public revisions for FeynKit,
Symbolica, Numerica, OneLOop and one-loop-reduce. It checks the nine recorded
patch hashes before applying the seven Symbolica patches, one FeynKit patch and
one OneLOop compatibility patch. It refuses an existing destination, including
symlinks, and leaves failed preparations intact. It never overwrites the original
reference worktrees or edits their sources.

The fresh network bootstrap under `output/bridge-bootstrap-1` was independently
compared with the previously accepted owners: all 3,776 tracked file contents
and source symlink identities agree. The corresponding public commits are in
`docs/DEVELOPMENT.md`; the bootstrap script contains the exact patch hashes.
Evidence is retained in
`output/diagnostics/bridge-delivery-proposal/{bootstrap-network-1-result,coordinator-bootstrap-check}.json`.
The final installed script preserves that fetching and patching logic; its
subsequent changes concern consumer-specific config emission and removal of the
draft label. The installed script was then run end-to-end into the separate fresh
`output/bridge-bootstrap-live-1` directory. All 3,777 tracked entries, including
ten symlinks (one points to a directory), match the accepted bootstrap owners.
Its three emitted configs exactly match the validated configs after replacing
only the destination prefix; root and portable `--locked` metadata also match.

Three generated configs select a single source owner for each ecosystem crate:

- `overlay-root.toml`: native CLI, library workspace and native reference tests.
- `overlay-portable.toml`: the standalone portable validation consumer.
- `overlay.toml`: community development, including a local FastSecDec override.

Root and portable configs contain only used package patches. The community
publication gate must remove its local FastSecDec override and resolve the
published Git revision. Absolute paths appear only in generated, untracked
configs. The root workspace excludes `output` so dependencies bootstrapped below
that directory keep their original workspace inheritance. The full consumer
continues to reuse HEPKit graph/model/kinematics/tensor owners and native scalar
master/reduction providers. No alternate algebra, graph or numeric implementation
was introduced.

The generated configs also set the three source roots used by the CLI's existing
provenance hashing. A missing root or unreadable Git revision fails explicitly;
there is no fallback `unavailable` identity. Tracked changes and nonignored
untracked source remain covered by the existing source-state hash.

## Validation and limitations

The earlier isolated metadata gates passed for root, portable, community native
and community Wasm consumers. The live root and portable metadata gates now also
pass with the fresh owners. Their lockfiles remain byte-for-byte unchanged,
contain no unused patch records, and select exactly `native` or `portable` for
FastSecDec. The root workspace retains its three members. Live evidence is under
`output/diagnostics/dependency-delivery-live-1`.

The first live metadata attempt exposed Cargo's automatic workspace membership
for dependencies below `output`; adding that directory to `workspace.exclude`
resolved it without dependency edits. An initial Clippy launcher put `--config`
before the external subcommand, so Clippy's nested Cargo lost the overlay and
failed dependency resolution before compilation. The corrected form is
`cargo clippy --config FILE ...`; setup documentation uses it. Both failures are
preserved and are not scientific executions.

Fresh-owner native all-target Clippy passes (59.30 s), all three existing CLI
provenance controls pass (0.09 s after a 6 min 40 s build), and portable-host
all-target checking passes (1 min 02 s). Formatting and Bash syntax pass. Actual
execution of the newly compiled build script rejects both missing source-root
environment variables and unreadable Git roots with exit 101 and explicit
messages. Embedded revisions agree with the fresh configured owners. The
installed bootstrap rejects an existing destination with exit 2. Both lockfiles
remain unchanged, and all five owners' source diffs still match their bootstrap
records. These checks used Rust/Cargo 1.98.1 and GCC 15 inside `nix-shell`, on
CPUs 20–21 with two build jobs. The previous installed native
wheel, Pyodide tests, browser lifecycle and ggHH native execution remain immutable
runtime evidence for their recorded builds; they are not relabelled as runs from
these new manifest paths. Source equivalence plus fresh-owner delivery checks do
not claim another numerical campaign or Wasm wheel rebuild. Community published
Git resolution and its PR were a separate final delivery gate. That gate now
passes: [draft HEPKit PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18)
uses the published FastSecDec revision, with 39 fresh installed-native tests and
zero skips. Its isolated Cargo-home preparation also passes the actual Pyodide
PEP 517 metadata flow. See the [bridge review](hepkit-fastsecdec-bridge.md) for
the remaining upstream dependency prerequisite and the separate earlier Wasm
runtime evidence. Windows path handling is unverified; delivery checks run on
Linux.
