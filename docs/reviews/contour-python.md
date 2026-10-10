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

## Refreshed public-owner milestone gate

The corrected fixed milestone `f2c2d930c4660b5fe200bf392af3e863040a37ef` was
committed and pushed before the new immutable source snapshot was created.
Its Git archive contains 1,251 files and has SHA256
`355538baccb107fc81f247132f2e4001cd7cbefe63ee9dab74cad9389f7299fc`.
The private staged Community `3aa2608` host points exclusively to this snapshot;
concurrent dynamic development cannot change its compiled FastSecDec sources.
All original default `module,native,community` modules are retained.

| Owner | New private gate identity |
|---|---|
| Symbolica / Numerica | Public `7ec1be45` |
| Graphica | Registry 3.0.1 |
| SymJIT | Public `33100ae8` |
| FeynKit companions | Public `917b2075`: citation compatibility plus reviewed signature metadata |
| Hyperbolica | Public `ac84d6ed`: current Community adapter `31292085` plus three URL fields |
| OneLoopMaster | Public `62ae35d6`: main `27c3723` plus three URL fields |
| One-loop reduction | Public `fae8a926` |
| RustFlow | Public `f0885454` |
| Vakint | Preserved RustRed-enabled owner `854e8495` |
| RustRed companions | Public `91a877e1` |

Cargo metadata confirms one identity for each numerical and graph owner. The
new RustFlow API requires the native RustRed feature absent at former `7c1`;
the necessary private RustRed update is explicit. Unchanged Vakint `854e8495`
compiles successfully against that newer owner in a coherent external consumer
(1 minute 39 seconds). There is no additional Vakint implementation patch.
The staged host also receives five required citation URL initializers. These
metadata-only fields are the subset of Community PR 25 present at `3aa2608`.

The complete native extension built successfully in 31 minutes 10 seconds,
using the private development profile with Symbolica optimized. Maturin produced
an ABI3 Python 3.9+ Linux wheel, SHA256
`06c602e256e8d6eb9c690756ba749c812c24692c6f3325ff0402b3a5e7776202`.
The wheel was installed into a new private Python 3.12.14 environment; the earlier
accepted wheel and live notebook remain untouched. This build does not establish
release sampling performance or a portable PyPI wheel.

All **8 contour tests passed** in 0.29 seconds. The initial run exposed a test-only
configuration error: the two new exact-only QMC cases requested 32 points with
the default Kuo33002 rule, whose minimum is 1,024. Selecting the already used
`hkkn_alpha3` rule fixes the fixture without changing production code. The
corrected test-only overlay is byte-identical to the live test fix; the wheel's
compiled FastSecDec source remains exactly `f2c2d930`.

The maintained native runner then passed **210/210 tests**, with zero skips,
failures or errors, in 73.69 seconds. It covers the complete binding tests,
notebook/demo controls and HEPKit wavefunctions. The exact-only controls now
exercise required preflight at QMC/MC session creation and checkpoint restoration
under both `always` and `pilot`, plus explicit unchecked access. The separate
bounded owner smoke passed **6/6 tests** in 0.29 seconds: native one-loop RustRed
generation, lazy artifacts and terminal normalization, and the preserved
48-digit Vakint numerical wrapper. No expensive four-loop reference was run.

The isolated stub-generator build passed in 29 minutes 19 seconds with
`--no-default-features --features python_stubgen`. The matching explicit
RustFlow build-feature metadata, private Python shared-library path and private
`CARGO_MANIFEST_DIR` were supplied; no production source workaround was needed.
Its six generated HEPKit-related stub files all pass Python 3.9 syntax parsing.
Eight public signatures match the installed native wheel exactly, including
both diagram/family `sector_decompose` methods, the direct entrypoint, contour
settings and caller-owned pilot methods. All eight native contour settings,
reporting and provenance classes appear in the generated sector-decomposition
stub and the installed module.

These generated stubs remain private. They are not copied wholesale into newer
Community main, which has additional APIs requiring its own regeneration gate.
The accepted installed tests and stub checks are separate from both the earlier
205-test wheel and subsequent dynamic changes.

Current Community main `9a65fbb7` introduces additional default IBP dispatch,
LiteRed2 metadata and positive-epsilon OneLoopMaster requirements. Validating
that latest host with its preserved owner branches is a separate required gate
before completing Phase B; the staged fixed gate does not claim to cover it.

## Current Community overlay prepared, not yet built

A separate JJ workspace now stages current Community `9a65fbb7` plus the
six-field citation compatibility commit `48d1d745`. The remote main revision
was rechecked before preparation. It retains the complete module set, default
IBP capacity dispatch and the optional runtime-arity selection feature. The
reproduction script and owner/diff hashes are recorded locally in
`target/prepare-contour-current-host.py` and
`target/contour-current-host-plan.json`.

This next gate preserves the newer host's actual owner behavior: OneLoop
`6c9874dc` positive-epsilon masters, RustFlow `995e531c` capacity/remote-data
support, all RustRed companion crates at `9cf14d3a`, and one-loop-reduce
`87f9758a` higher-point scalar fallback. Existing citation-only patches are
applied to isolated copies of those newer owner bases rather than replacing
them with the earlier wheel's owner revisions. The OneLoop and RustFlow
citation source files are unchanged between those bases; the reducer's two
native fallback changes are retained as well. No new owner implementation or
additional PR is introduced by these private overlays.

Public contour prerequisites remain Symbolica/Numerica `7ec1be45`, SymJIT
`33100ae`, Hyperbolica `ac84d6ed`, independent Vakint `854e8495`, and FeynKit's
reviewed interface/citation commits. Published Community dependencies must
first consolidate FeynKit fixes onto its upstream `feynkit` branch. The private
manifest still references the immutable `f2c2d930` FastSecDec snapshot as a
placeholder; a later accepted snapshot must be selected before the next build.
Lock resolution, unique-owner verification, full current-host wheel/tests and
regenerated current stubs are pending. No build has been started for this new
workspace, and the live Community environment is unchanged.

The next private metadata audit now resolves that full-default `9a65fbb7`
workspace with Symbolica/Numerica `516beb37`, SymJIT `d74993f` and the reviewed
FeynKit `917b2075` companion set. It preserves every RustRed companion at
`9cf14d3a`, current OneLoop/RustFlow/reduction owner bases and both IBP feature
definitions. Locked metadata confirms exactly one Symbolica, Numerica, SymJIT,
Linnet, Spenso and FeynKit Python owner. The old host lock needed a targeted
SymJIT update from 2.26.4 and explicit FeynKit advancement from the earlier
`fd1b43c6` wheel's selection; no version requirement was made exact.

This remains a dependency-admission gate. The isolated manifest currently
points at the frozen development bindings and must select an immutable accepted
snapshot before a validation wheel is installed. No wheel has been built from
this current host. The new local provenance is
`target/contour-current-host-516-readiness.json`; the older preparation script
and plan above describe the preceding overlay and would need updating before
reuse. No current host capability was removed to make resolution succeed.

The subsequent full-default current-host `cargo check --lib --locked` passes
in 4m57s on that exact owner graph, including the current native IBP/RustRed,
positive-epsilon OneLoop, reduction and FastSecDec bindings. It uses a separate
target directory and the existing Python 3.11 interpreter; no shared extension
was replaced. A fresh private Python 3.12 environment is prepared for the
forthcoming installed-wheel gate. This consuming compilation result does not
replace wheel execution, stub generation or actual browser tests.

### Installed current-host gate at `184803d`

The complete current-host overlay now builds and installs privately against an
immutable Git archive of FastSecDec `184803dbb1fe6e1c4966cad9bd6db90ecb596c96`.
The archive SHA256 is
`023e81e956490cb02fe781d08f8b23a80a348783da863b593e65e394a3680771`.
The wheel SHA256 is
`7cc5f50427d6b673e1a477ba558d0771ae443a890e847ca5e902d046f0ba6dc2`.
It uses the full default Community features, CPython 3.12, Symbolica/Numerica
`516beb37` and the owner overlay above. Compilation took 10m36s in the
development profile. It is a Linux validation wheel, not a release-performance
or distributable PyPI build. Shared installations remain untouched.

The maintained binding/demo suite initially passed 221 tests with two failures,
both in the same archive-lifetime assertion for the two generation modes. The
test incorrectly required a selected archive and retained resident archive to
have identical physical record order. Independent native-owner review confirms
that selection stream-copies the original records in canonical directory order;
resident retention may preserve append order, including exact contributions.
The logical identity and native programs agree; no codec change is required.

The corrected test captures each owner's immutable bytes, deletes the source
file/archive, restores the retained bytes and checks identical native QMC
estimates against the resident owner. Both corrected cases pass on the same
installed wheel. Thus all **223 distinct binding/demo cases pass across those
two runs**, with no skipped cases. Community's **14/14 wavefunction controls**
also pass. This includes the new dynamic settings and actual family generation,
selection, restoration, pilot, integration and checkpoint tests in both
generation modes. It does not include the later optional-runtime-observation
Python test, which needs the next wheel.

The immutable source initially sat under the outer workspace's `target`
directory, where Cargo reported a duplicate path-package collision before
compilation. Moving the unchanged archive into the excluded private worktree
area resolves metadata with one owner per crate; no production manifest or
dependency source was modified for this build-environment correction.
Ignored `target/contour-current-host-build-plan.json` records manifest, lock,
source and wheel identities. Matching build, original test, corrected-lifetime
and wavefunction logs use the `contour-current-host-184803d` prefix. Current
stub generation and browser/Pyodide execution were separate pending gates at
the installed-test boundary.

The full current-host stub generator now builds with its explicit
`python_stubgen` feature graph in 11m56s. RustFlow's workspace fingerprint uses
that same feature selection and `NO_DEFAULT_FEATURES=1`. The generator runs
with Cargo's manifest-directory environment, as it would under `cargo run`.
Six generated module files parse successfully; **14 public signatures match
the installed runtime**, including dynamic settings and the recipe-family
session/archive methods. All eleven inspected contour/family classes are
present. This validates the immutable `184803d` interface, not the subsequent
runtime-observation additions. No stub-generation owner fix was required.

### Family and optional-observation wheel preparation at `a3d97cf`

The published family/observation milestone is frozen separately as
`a3d97cf981726944fc967799c5345fe88ebe0b72`, Git archive SHA256
`758cf165bed172d79281318f8ce0c1371ff819b394f6f898accdd72af3b8f2a0`.
A new native wheel and the complete Community `wasm` wheel now build
from that same immutable source in isolated targets. Both retain the current
owner overlay; concurrent family-input and status-transport changes cannot
enter them. The WASM build uses Rust 1.98, Python 3.14.7, Pyodide 314.0.7,
pyodide-build 0.39 and Emscripten 5.0.3 in development mode. Compilation took
2m29s for the native wheel and 5m23s for WASM. Their SHA256 identities are:

```text
native: f8e6793f9df2a5b7a8dfa75e2afc7120184c167ff4d5eafdf36fc8c4b0495de6
WASM:   d741a02e8da827e5d6597b470a95f291a8c76a7963938c5c60b4d4a23d9f0903
```

The native wheel installs into a new isolated CPython 3.12 environment and
passes **238/238 maintained binding/demo/wavefunction tests**, with no skips,
in 75.56 seconds. This includes the optional observation owner's identity,
native pilot, detached snapshots and unchanged sampled estimates.

The complete WASM wheel installs in actual Pyodide under Node and passes
**127/127 selected maintained Python tests**, with no skips, in 14.25 seconds.
The runtime reports Python 3.14.2 and `sys.platform == "emscripten"`; its ABI3
wheel was built with the compatible 3.14.7 build interpreter. The maintained
runner copies the selected scientific tests and native fixture assets into
Pyodide's filesystem. It now includes fixed/dynamic contour and recipe-family
controls plus the shared `_fixtures.py`, and accepts an explicit immutable
test-source root. Tests include actual disk-backed family generation,
restoration, both dynamic recipes, native pilots, complete-vector integration,
policy-only checkpoint resume and optional owner observations. Fresh-process
citation controls also pass. This is actual Python/WASM execution, not a
browser UI or release-performance measurement.

Ignored build/result records use `contour-community-a3d97cf` and
`contour-current-host-a3d97cf` prefixes; the source/wheel identities are also
in `target/contour-family-observations-build-plan.json`. The generated
observation stubs also pass six-module parsing, sixteen runtime-signature
matches and fourteen class-presence checks. No shared notebook installation
was changed.

### Retained family inputs and session observations

The next implementation snapshot, based on `a3d97cf`, has archive SHA256
`9950910602b5c5fe040f30aa2a3d388ef8c28f63a4f6b7d11f02d0eec6d77c12`.
It includes `Integral.from_family`, the shared synchronous preparation path,
native operational diagnostics in sampling snapshots and retention of their
phase labels across Havana adaptation. It contains no new algebra, estimator,
physical graph type or sampling stream generator.

Fresh full-default native and complete `wasm` wheels pass **249/249 native
tests** in 78.45 seconds and **138/138 selected actual-Pyodide tests** in 16.32
seconds, with no skips. The eight new family controls cover signed/zero powers,
weighted numerators, graph equivalence, inert construction, runtime rebinding,
exact normalization, both dynamic recipes and full complex reference checks.
The three new observation controls exercise QMC/discrete-Havana snapshots,
mode-only checkpoint continuation with identical means/covariances and
retained adaptation history outside production sampling statistics.

```text
native: 17c0711d0aea55a236eabe3e11944623955af163a595ad05af65f64eaf47814c
WASM:   32351e08fb66bdf12cfdca0fdc99e2596d47e0d4d3504df0ab989409f3007cf1
```

The installed tests use the immutable `contour-interface-candidate1` source;
its manifest and per-file hashes are retained with the ignored build records.
Independent source review and leaf all-target/stub-feature compilation and
strict Clippy pass. The regenerated stubs pass six-module Python syntax
parsing, seventeen installed-runtime signature comparisons and fifteen
class-presence checks, including `Integral.from_family` and the phase-labelled
`ContourRuntimeDiagnostics`. The native status and complete CLI regression
gate passes 175 distinct tests, with eight CLI tests explicitly ignored.
These CLI results are separate from the installed Python counts.

## Dynamic settings mirror after the recipe-family milestone

The next binding increment mirrors native `ContourMode::Dynamical` and
`DynamicConstruction`, with a `ContourSettings.dynamical` convenience constructor
and read-only safety-fraction, cap and construction properties. Native Rust
continues to own range checking, recipe selection and the reserved-parameter
classifier. Python rejects incompatible keyword combinations; JSON decoding
uses the same native schema. Constructing settings performs no generation,
root solve, validation pilot or integration.

The standalone native binding check passes on this increment (14.44 seconds).
Independent CLI-owner source review found no algebra/execution duplication or
API blocker. Python regression sources parse under Python 3.9 grammar; their
new runtime cases still require a refreshed installed host. The existing wheel
described above predates this increment. The later registered callback/checker
and final schema changes require another consuming build; this intermediate
compile check is not dynamic production, installed-wheel or WASM acceptance.

The subsequent validation-chart mirror adds read-only `kernel_sectors` and
`includes_exact` properties alongside the legacy single-sector display field.
Native Rust owns association discovery and scope decisions. The bridge returns
a copied list and exposes the native flag; it does not infer exact contributions
from a missing sector ID. The finite-triangle pilot regression checks this
metadata and copied-list ownership. Rust formatting and Python 3.9 syntax
checks pass; compilation/execution of these newest properties awaits the
completed native lifecycle and refreshed host.

The subsequent complete leaf binding, including dynamic settings, chart
associations and native recipe-family ownership, passes `cargo check --locked
--features python_stubgen` on public Symbolica/Numerica `516beb37` and SymJIT
`d74993f` (2026-10-10, 1m40s). It uses the existing Python 3.11 interpreter and
the separate binding target. The explicit `PYO3_PYTHON` path is required in this
shell; the first invocation without it stopped during interpreter discovery,
before checking the bridge. The same leaf then passes all-target binding-only
strict Clippy with `--no-deps -- -D warnings` in 5.47 seconds. No shared Python
installation was changed.

The family tests now also specify actual dynamic selection/restoration,
native pilots, analytic-triangle integration, policy-only checkpoint resume and
changed-strength rejection in both generation modes. Python 3.9 syntax checks
pass. These are pending installed-host tests, not relabeled results from the
older fixed wheel.
