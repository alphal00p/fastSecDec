# Evaluator release verification — 2026-10-04

The live [Symbolica registry index](https://index.crates.io/sy/mb/symbolica) and
[SymJIT registry index](https://index.crates.io/sy/mj/symjit) were queried before
the requested function-map/inlining experiments. Their latest non-yanked entries
were Symbolica **3.0.1** and SymJIT **2.26.4**. The Git upstream's latest Symbolica
version tag is also `v3.0.1`, resolving to
`e9a0d35490a834afb2fa40d97c8c0b1e58ef5a5e`. Cached documentation/search results
still showing Symbolica 3.0.0 were not used to select the version.

The existing Symbolica worktree at
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` includes `v3.0.1` and six subsequent
upstream commits, including a directed-self-edge symmetry correction. It retains
the three separately documented local correctness patches. It therefore needs
no release upgrade or downgrade for this experiment.

The root SymJIT pin and lockfile were updated from 2.26.0 to 2.26.4. The backend
update affects only that version and checksum; the upcoming native persistence
probe also adds an explicit development dependency on the already resolved
`bincode` package. The new SymJIT checksum matches the live
registry entry:

```text
37344a51408baa239bbaa771e40cd908686bffed4f4491ec6bc608e8d248c931
```

The native OneLOop reference provider's exact backend pin initially prevented
resolution. Its [independently reviewed compatibility patch](../dependency-patches/oneloop-symjit-2.26.4.md)
changes only the pin and compiled-cache identity. CLI dependency provenance now
records 2.26.4, so old CLI artifacts cannot silently claim the new backend.
Library portable kernels store expressions for fresh compilation, not old JIT
machine code; their patch-level-independent expression schema is unchanged.

Earlier development timings and the 64-shift double-box diagnostic used 2.26.0
and keep that provenance. The new scientific gate and release/alias measurements
must identify 2.26.4 explicitly. This is a dated version check, not a claim that
the registry can never change during later work.

The complete latest-backend workspace gate passed **202 tests**, with eight
explicit long-running scientific/performance probes ignored by the ordinary
gate (`output/symjit-2.26.4-workspace-tests.log`). This includes all native
scalar-master and numerator-reduction comparisons, the new cache round trip and
old-header rejection, compact monomial extraction, and four scientific gap
regressions. Formatting and all-target Clippy pass. The production dependency tree resolves a single
Symbolica 3.0.1 owner and SymJIT 2.26.4, with no Python, pySecDec, native test
reference provider, or CLI presentation dependencies in the library graph.
