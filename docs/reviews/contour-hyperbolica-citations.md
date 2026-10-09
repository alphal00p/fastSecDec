# Hyperbolica community citation compatibility

2026-10-09. The public Symbolica Python `Citation` API at
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca` requires a `url: String` field.
Hyperbolica's community adapter omitted it. The owner correction is confined
to the three existing citation records; it does not change integration or
registration behavior.

## Upstream and adapter identity

Fresh upstream inspection found main at
`90b78212e1b0e1958a604082c6ad032aa856a39f`, with a standalone Python adapter.
The existing Community host had pinned
`6acf21500c4807ac1fbd67e1429efcad90791de1`, whose `CommunityModule` adapter
is required by that host. Substituting upstream main would not preserve the
registration interface.

The upstream `codex/hepkit-integration` branch has advanced to
`31292085504b794dc444a006eba1d3013ed30944`. It retains `CommunityModule` and
includes two subsequent owner commits: an infinity/MZV output correction and
improved upstream citations. The coordinating agent approved using that current
adapter branch as the correction's base and PR target, preserving both commits.
The refreshed host therefore needs the corrected child of `3129208`, rather
than silently treating it as the older `6acf215` snapshot.

An isolated checkout and separate unchanged baseline worktree were used.
Shared Hyperbolica and Community checkouts remain untouched. The patch adds
the repository URL and the two arXiv URLs already present in the existing
records' identities and bibliography. No manifest or lockfile change is part
of the owner patch.

## Focused validation

The public API and source were inspected directly, including
`symbolica/src/api/python/citation.rs` and Hyperbolica's
`src/python/mod.rs`. The external consumer under the ignored
`target/hyperbolica-citation-check` selects the existing community registration
type with Hyperbolica's `python` and `native` features, Symbolica/Numerica
`7ec1be4` and SymJIT `33100ae869057f35d9865c933a48bac6699acdd4`.
Owner-local dependency patches do not override this consumer's coherent host
selection.

The untouched adapter reproduced exactly three `E0063` missing-`url` errors
at `src/python/mod.rs:69`, `:76` and `:94`. The corrected adapter and consumer
passed `cargo check --lib` in 4.40 seconds. Logs are retained only in the ignored
`target/contour-hyperbolica-citation-{baseline,fixed}.log` files. The shared
binding target was used after the Feynkit signature gate released it, then
released to the independent OneLoopMaster owner gate. No installed-wheel,
WASM or complete Hyperbolica runtime claim follows from this narrow check.

## Publication

The three-line change is [Hyperbolica PR #1](https://github.com/benruijl/hyperbolica/pull/1),
draft commit `ac84d6ed484b09176802f86824ccb8771a871f5b`, targeting the existing
`codex/hepkit-integration` branch. It was authored and published by
`ValentinHirschi <valentin.hirschi@gmail.com>` from the isolated fork branch
`codex/symbolica-community-citation-url`.

GitHub rejected the formal `benruijl` reviewer assignment: the publishing
account lacks permission to request reviews on this upstream repository.
The PR description explicitly asks `@benruijl` to review and records that
permission limitation. This is not reported as a successful formal reviewer
assignment. The owner-local vendor/lock configuration is unchanged; this
compatibility update is for the coherent refreshed host dependency set.
