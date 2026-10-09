# Native HEPKit contour binding validation

2026-10-09. Independent installed-host validation of the thin FastSecDec
bindings. This file records the actual build boundary; it does not establish
browser/WASM execution or completion of dynamic deformation.

## Isolation and source identities

The shared Community checkout and its running notebook environment are left
untouched. Its JJ metadata was not accessible to this account, so a separate
local Git clone was initialized as an isolated JJ repository at Community
`3aa2608fc1123a6d260e6bb83e76629dc249318b`. The private change is authored as
`ValentinHirschi <valentin.hirschi@gmail.com>`.

Only this ignored validation checkout's dependency configuration and lockfile
are changed. It links the working FastSecDec Rust binding crate and uses the
same tested fixed-contour dependency owners as the CLI:

| Owner | Selected identity |
|---|---|
| Symbolica | Local tested ball-domain commit `3db1607f5acd9669cde747aa048a3ca0c0fcb2e1` |
| Numerica / Graphica | Registry 3.0.1 |
| SymJIT | 2.27.0 with the tested complex callback lane owner fix |
| Feynkit / Linnet / Spenso | Current compatible private-host selection `fd1b43c68af9ca308a9acbdf05f1735dd975b3ce` |
| FastSecDec / native binding | Frozen working-source snapshot above `0e62f05cd2b9a1ed0cf7bd0b3678ac3664540ed7` |

Cargo metadata confirms one package identity for each native owner. The private
host replaces its previous Symbolica branch selection and removes its Numerica
and Graphica Git overrides; no mixed native types or duplicate Python extension
installation is used. These paths and private lock changes are not publication
material.

The wheel points to an immutable source snapshot under ignored
`target/contour-fixed-host-source`, allowing concurrent dynamic implementation to
proceed without changing the fixed-contour validation build. The snapshot covers
the numerical crates, binding crate, root manifests and citation resources;
`SOURCE_SNAPSHOT.json` records content and tracked-diff SHA256 identities. Build
trees, reference repositories, licenses and logs are excluded.

The primary snapshot SHA256 is
`00b6f4c7eb0e7fb1c3535e0b0cb70f833de245270f5f5b97dc91a753bca12a62`.
The separately copied notebook test assets have SHA256
`8762284bb571079a31db03ca81711fa2cc7ec3e60cae56cf2d1112254449b37a`.

The initial private build retained the fixed CLI/reference Feynkit revision
`259df8790f27b8d3ef32778cd7195942691b4ef0`. Community's newer AMFlow and standalone
graph-display registrations require APIs absent at that revision. A temporary
AMFlow exclusion exposed the second mismatch; that omission was abandoned.
The private host now restores all default registration sources and AMFlow, using
the coherent current Feynkit revision above. Source inspection confirms the
previously missing model, diagram, graph-display and connectivity APIs exist.
This newer dependency matrix is separate from the fixed CLI/ggHH reference
milestone. Neither the shared notebook environment nor FastSecDec's dependency
selection changed as a consequence.

## Gate status

The complete default Community extension built successfully into the private
wheel/environment, using two Cargo jobs and its development profile with the
existing Symbolica optimization setting. All original module registrations and
AMFlow are enabled. No individual native extension was copied into a shared
installation. Compilation settings do not establish release performance.
The exact feature selection is Community's default
`module,native,community`, including its ordinary optional modules. The private
`symbolica-3.0.0-cp39-abi3-linux_x86_64.whl` has SHA256
`13da421d3b37854d050d68696fbbba2d2e3c84ace52e9e15604300ceb48a3371`.

The fresh wheel was installed into a separate Python 3.12.14 environment. The
three contour-specific tests passed in 0.46 seconds. The maintained native test
script then passed all **205 tests**, with no skips, failures or errors, in
76.81 seconds. Its scope includes the FastSecDec Python bindings, standalone
notebook/demo controls and Community's native HEPKit wavefunction tests.

The contour controls exercise generated and restored capability, caller-owned
certified pilot points, checked production, identical production values across
`always`/`pilot`/`off`, unchanged mathematical identity on policy changes,
QMC/MC checkpoint restoration, retained pilot provenance without an invented
seed, and the diagram-level `sector_decompose(contour=True)` entrypoint. The
notebook controls include actual relocated standalone notebook generation and
runtime-parameter binding, not merely source inspection.

This installed native gate uses the immutable fixed-source snapshot and owner
identities above. It does not validate subsequent dynamic source changes, the
later public combined owner revision, or execution inside Pyodide/WASM.

Separately, the final native Rust portable consumer passed **73 tests**, with
zero failures or ignored tests, using public Symbolica/Numerica `7ec1be4`,
Feynkit `8e3a643`, registry Graphica 3.0.1 and the portable Malachite/Astro
arithmetic. Its new fixed-contour test exercises the above-threshold complex
bubble, both generation modes, saved native program restoration and certified
pilot/production checks through public APIs. This is portable-host Rust
execution; it does not upgrade the frozen Python wheel's source identity or
establish actual Pyodide execution.

## Independent interface review

The native bridge retains mathematical contour settings separately from optional
checking policy. Caller-supplied pilot points carry no invented RNG seed. A
mutating pilot clones shared evaluator ownership before changing readiness, and
sessions own their production counters. Accepted batch boundaries drain native
check deltas before merging them into the appropriate sampling phase. Checkpoint
pilot evidence stays separate from mathematical compatibility and does not
unlock an independently restored native validation gate.

Community's existing wildcard reexport exposes registered native classes through
`symbolica.community.hepkit.sector_decomposition`; the substantive settings,
generation, checking, execution and checkpoint behavior remains in Rust inside
FastSecDec. The installed native tests above now cover execution of these paths.
