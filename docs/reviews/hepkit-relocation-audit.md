# HEPKit binding relocation audit

Source review, 2026-10-05. The reviewed design satisfies the latest ownership
requirement: FastSecDec owns its optional Rust/PyO3 library, notebook, helpers,
fixtures and scientific controls; community hosts and registers the public
module. The follow-up below also accepts the relocated **native release wheel
and 61 installed controls**, followed by the actual native triangle notebook
lifecycle and the sole native ggHH notebook lifecycle. Generated stubs pass,
and the presentation follow-up passes using retained ggHH data only. Public Git
consumption, hosted native CI and the current Wasm wheel's portable controls
subsequently pass, as recorded below. Updated browser interaction remains open.

The reviewer read `AGENTS.md`, the governing ownership/notebook clarification in
`FIRST_PHASE_PLAN.md`, the current FastSecDec files, and both the working-tree
and cumulative community diff. No build, import of scientific modules,
generation, integration or performance run was launched for this audit. File
hashes and saved Cargo metadata were inspected with Python's standard library.

## Evidence identity

FastSecDec HEAD is `10362c13ca61b9c4f6a2af8809ee411c078e5257`; community HEAD is
`a698db6089f811cf48e56aad3ea00b0f01b8a916`, with uncommitted relocation changes.
The cumulative community comparison uses local `origin/main`
`69509f1b8a386d22f2333a2182d4b1505cd804e3`.

Saved evidence is under ignored
`output/diagnostics/hepkit-binding-relocation-1/`. Its
`build-source-manifest.json` has SHA-256
`f85a21e7a09b0edfbda32239c40204f15641562ee47be7b8b5ee684669cecfc0`;
all 223 recorded source/input hashes matched at review. The following hashes
also bind the migration scripts reviewed outside that Rust build freeze:

| Source | SHA-256 |
|---|---|
| `bindings/python/Cargo.toml` | `920445d65c163f5fd186841f77bb5b3833c60c424f640f87694e91401b52e3c8` |
| `scripts/bootstrap-dependencies.sh` | `267624a05422686e4f67a017b275eba550d09260bbd07c7f15e44186a4e12391` |
| `bindings/python/scripts/prepare-community.sh` | `07bf7eff033e70aaf86dedefd486c022a3b076d2cd718c16d11653d1b79ffd4c` |
| `bindings/python/scripts/test-native.sh` | `3b8447272395aadda25be44578c6d4a67c3ed120fa0ca4a107a650f21bc4ee53` |
| `bindings/python/scripts/test-pyodide.mjs` | `c62912b62d04f4240ae69a6509e37846fbec425744eefd608e18359bce82b982` |
| `bindings/python/scripts/check_dependencies.py` | `9e709742c135ee8a1a7bd460bcc7b39192ba09717b3e911136ef066ca4d7fcaa` |
| `examples/hepkit/export.py` | `7738156ce5b13efb424c7806f463e0bb2a0aaabea2ba9d96c3e429a8e830258d` |
| community `scripts/prepare_fastsecdec_dependencies.sh` | `95ef4f312800759a58539169307bf124568e66e3ed01defccc530fcb94016071` |
| community `.github/workflows/fastsecdec.yml` | `5931a737a469c9ad8efbc90d9115cf4aebdc110425274db96ae47b55af1823a4` |
| community `src/bin/stub_gen/compat.rs` | `ca94942d12f9bb843c7d887f27502b2109eb646e2896464cecbfd800436de588` |

The notebook author is still refining presentation and bounded sector detail.
This report does not freeze or certify that evolving interface.

## Accepted ownership and dependency boundary

- `bindings/python` has its own workspace and lockfile and is explicitly excluded
  from the numerical workspace. It is an `rlib` linked into the community wheel,
  not a second extension module with a duplicate Symbolica runtime. The host
  retains its ABI/extension-module choices.
- Core and sectors dependencies disable defaults. The leaf's native/portable
  features forward to the existing native/portable owners; inspection's direct
  sectors dependency also disables defaults. Existing core guards reject
  contradictory backend selection. No Python feature was added to the core or
  CLI, and their manifests and locks remain independent.
- Saved root and portable-kernel metadata contain no `pyo3*`, `feynkit-py` or
  `fastsecdec-python` package. Native leaf metadata has GMP/MPFR only; portable
  leaf metadata has Malachite/Astro only. Shared graph, tensor, kinematics,
  Symbolica, Numerica and PyO3 packages each resolve to a single owner. Root and
  portable-kernel lock hashes, and the prior command-scoped Cargo config, still
  match `preserved-inputs.json`.
- Community metadata selects exactly one FastSecDec backend, but its
  target-conditional native integrations also appear in Cargo's aggregate
  numeric feature set for Wasm metadata. The checker describes this limitation
  instead of presenting that union as a compiled target result. Actual portable
  compilation and Pyodide execution remain separate gates.

Relevant saved metadata hashes are:

| Metadata file | SHA-256 |
|---|---|
| `metadata-root.json` | `c8887513ec6a452d218d0509fd17b8968480a788aa6441e62cf1054b0f060e7c` |
| `metadata-portable-kernel.json` | `9b34b8ac58f1fc52663f02dd8c53a428b9b437bb1e618f3a73378727c852e5cb` |
| `metadata-python-native.json` | `1478c48e0eca68c78263e77e4c5830282a23b57b53c554598012a3183858b8bc` |
| `metadata-python-portable.json` | `2501607194cf6fd2d468ef5d359cbfd9fe2c87e5f2b78e9bcc2d20a563730398` |
| `metadata-community-wasm.json` | `e16311b9113fe6a7a40c8eb23deb481304240697f6275580d7fa6b650695dae5` |
| `metadata-maturin-cwd.json` | `cfabbd446511c01fcb85ae5b2e2fc9fd2ae78695bbe653c7fb4d956bd999298e` |

## Community remains a host

The cumulative working-tree diff removes `src/fastsecdec`, the old notebook,
input/display/export helpers, FastSecDec fixtures and dedicated scientific tests.
Their maintained destinations are `bindings/python` and `examples/hepkit`.
The earlier untracked ggHH helper/assets/tests have also moved; the current
community tree contains no second copy of that implementation.

Remaining FastSecDec-specific community changes are the optional Git dependency
and feature forwarding, one Rust registration call, public Python reexports and
stubs, brief documentation links, exact-pin checkout/bootstrap forwarding, and
CI invocation of the maintained tests. The dependency identity checker and
native/Pyodide test orchestration now live in FastSecDec. The community workflow
keeps a host import smoke test. Generic shared wavefunction controls correctly
remain in community.

The cumulative PR also contains `src/bin/stub_gen/compat.rs`. Its 54 lines
convert generic tuple-unpacking annotations for the host's supported Python 3.9
syntax; the common package writer applies this to tensor, Vakint and other
stubs. It contains no FastSecDec name, schema or scientific behavior and belongs
in the host. FastSecDec-specific exception declarations instead come from the
leaf's `stub_source`. Existing generic Pyodide smoke adjustments select the
tensor interpreted backend and install NumPy; they do not implement FastSecDec.

## Native objects, caller execution and inspection

The input wrapper borrows the existing `PyFeynmanDiagram`/`PyKinematics` owners,
preserves the native selected-subgraph guard, and invokes `GraphIntegral` with
Symbolica expressions. It introduces no DOT parser, graph type, contraction
algorithm or scalar-master implementation. Existing native master/reduction
dependencies and shared external-state primitives remain unchanged.

Generation and compilation invoke existing native APIs and forward typed events.
The session requests bounded native packages on the caller's thread, preserving
native covariance, replay policy, accepted coverage and error boundaries. It
does not own a background thread pool or automatic integration loop. Its existing
Rust checkpoint envelope retains native session/replay data; the relocation
does not introduce a Python codec or estimator.

The generated-object inspection API shares `Arc<GeneratedIntegral>` ownership
with index views. Map/chart associations, exact integers, Symbolica expressions,
domain proofs and compact alias roots/definitions come from existing getters.
It never calls the materializing `coefficients()` path. Stored alias counts are
per coefficient. A chart's missing kernel association remains `None`, without
inventing a per-chart zero or exact offset. Generated complex coefficients and
compiled real/imaginary rows are distinguished. This reviewer implemented that
inspection slice; the relocation author separately reviewed and accepted its
source and six focused controls. All six subsequently passed against the installed
relocated release wheel, as recorded below.

## Bootstrap, exports and remaining acceptance

The thin community setup reads the exact `fastsecdec-python` manifest pin,
fetches that checkout and delegates to its maintained setup script. The latter
creates the shared-owner overlay and removes FastSecDec path overrides for
public-source validation. Local development is explicit. The new standalone
Python overlay has the required shared owners; only the root/community scopes
include their existing scalar-master reference patches.

Relative paths in the moved test runners, helper imports and fixture loaders
resolve to the new FastSecDec tree. The browser exporter uses `--no-execute`,
bundles the four portable inputs and presentation package, and excludes native
ggHH assets. The notebook verifies the wheel/archive hashes and file list before
mounting them. Opening/importing this presentation does not invoke the input
builders. Native ggHH remains an explicit fixed-point choice, with browser cost
unvalidated. Source inspection supports these boundaries; a newly exported
browser lifecycle is still required.

Two concrete delivery issues were found:

1. The community manifest still declares `be9c3d29`, which predates the new leaf.
   This is documented as a draft pin and local builds use an explicit overlay.
   Publish the validated leaf, update the exact pin/lock and validate the public
   checkout path before presenting the migration as installable.
2. Python caches and the isolated leaf's default `target` directory were not
   originally ignored. The coordinator added `__pycache__/`, `*.py[cod]`,
   `.pytest_cache/` and `/bindings/python/target/` during this review. Verify staged
   paths before committing. No generated cache is an implementation artifact.

The leaf manifest also contained a newly guessed SPDX license field; the
architecture author identified and removed it after the bounded build reaped.
Exact reconstruction of the old manifest confirms this was its only change.
The numerical workspace has no corresponding license declaration, so the move
does not invent one. The earlier hash table binds the original review snapshot.

The relocated normal-release wheel and installed controls now pass. Before closing
the milestone: regenerate/check the public stubs; validate the new public-pin
bootstrap; and separately validate the portable target and actual notebook
actions/streaming. Earlier pre-relocation
native tests, browser lifecycles and ggHH CLI timings remain evidence for their
recorded sources, not certification of this move. Performance work stays parked.

### Bounded build follow-up

The original 1,200-second compilation window ended without a wheel. The corrected
command returned timeout status 124; postflight found unchanged frozen sources,
locks/configuration and no surviving owned processes. After the license metadata
cleanup, an independent review accepted one proposed additional 1,200-second
cached compilation on eight reserved CPUs, with a 24 GB sampled aggregate RSS
stop, 30 GiB per-process address-space backstop, atomic attempt marker and
automatic preflight/postflight checks. The normal release profile and source
owners remain unchanged. This is explicitly additional compilation after a
retained timeout, not a reset of the original bound. Evidence and its
228-entry source manifest are in
`output/diagnostics/hepkit-binding-relocation-1/continuation-1/independent-review.json`.

### Installed native acceptance

The accepted continuation completed with exit zero: Cargo reports **17m 49s**
for release compilation; the complete supervised command took **18m 00.13s**.
The sampled process-tree RSS maximum was **12.263 GiB**. Postflight and the
independent review found no surviving owned session members and no changes to
228 frozen inputs or the three preserved locks/configuration files. This result
includes the explicit local leaf overlay and does not validate the public draft
Git pin.

The installed wheel has SHA-256
`d500e865d0ebcd63fd987781f72ded47b80a194e16a40e7a54bf9d96c945d69b`;
the installed `symbolica/core.abi3.so` has SHA-256
`0ad29b9a715c39c5c54244ee03987f9553164477415d0f7deb7463bb0a882338`
and matches the wheel's corresponding member byte for byte. The independent
review rehashed the retained evidence, all frozen build inputs, and all twelve
test/helper inputs. JUnit records **61 passed**, zero failures, errors or skips,
in **1.249 seconds** (pytest displays 1.25 seconds): 52 API/input/wavefunction/
inspection/Korobov controls and nine presentation state/report controls.

All six inspection controls actually ran against this installed wheel. They
check native chart/representative/kernel associations, independently retained
exact coordinate images and positive measure, genuine compact aliases and
immutability, owner lifetime after parent Python variables are dropped,
generated complex orders versus compiled real/imaginary rows, and truncated
charts retaining `None` without invented per-chart zero values. The independent
review used retained logs/XML and source hashes; it launched no duplicate
generation or tests.

Evidence is `controls-result.json` (SHA-256
`d0b7ac490d09de7342688135f8bdbd79482a70a6b1b2614392c910c5c5045088`)
and `independent-native-controls-review.json` in the relocation diagnostics
directory. These controls establish native API and presentation-state behavior;
they do not substitute for real notebook button/streaming interaction or the
pending stub, public-consumer and portable/Wasm gates.

### Actual native triangle notebook acceptance

The independently reviewed `hepkit-notebook-relocation-1/triangle-ui-3` gate
exercised the actual browser controls against the installed release wheel.
Generate emitted two distinct live Geometry progress states and stopped with
no session or snapshot. The native graph, all-sector summary, compact selected
coefficient and source chart rendered; inspection left the generated diagnostic
report byte-identical. Draft edits after generation preserved the bound input.
Explicit Integrate accepted 1,024 points before Cancel; three refresh periods
left the entire paused report unchanged. Resume preserved the exact accepted
checkpoint prefix and completed 16,384 points across two sectors and eight
shifts, with no evaluation failures or browser errors.

The full two-component Laurent vector and four covariance entries are retained.
The covariance is symmetric and its diagonal agrees with squared standard
errors. The finite mean is `-0.46312964429055936`, differing from the existing
equal-mass spacelike analytic control by `3.1361705454813205e-9`. This analytic
check concerns the finite term; the epsilon-one coefficient is preserved with
native joint covariance, without claiming an additional independent reference.
The supervised lifecycle took 15.079 seconds including browser startup and
deliberate pause; this is a UI acceptance measurement, not a performance benchmark.

Two earlier browser harness failures are retained: an incorrect numeric-input
role before Generate, then a hidden alias-control label before Integrate. Their
corrections changed only diagnostic selectors. The pure-data validator was also
corrected against native `ReplayState`: an untouched sector starts unverified
with zero maxima, accepted work verifies it, and merge preserves verification
by logical OR. Validation now requires that legitimate monotonic transition,
final verification, exact accepted prefixes and monotonic maxima. No application,
precision policy or numerical result was changed to obtain acceptance.

Screenshots show readable live progress, native graph and map records, sector
tables and the completed vector with its uncertainty history. No blocking visual
issue was found. All owned browser/server processes were reaped and the frozen
application/extension inputs remained unchanged. Later output-only full-page
inspection capture is archived separately and does not retroactively change
this run. Evidence and independent source/data/rendering review are retained in
`triangle-ui-3/independent-review.json`.

### Actual native ggHH notebook acceptance

The sole `hepkit-notebook-relocation-1/gghh-ui-1` browser lifecycle passed in
**236.872 seconds** including startup, inspection, deliberate cancellation and
pause, within its original 300-second server bound. The installed native owner
generated and compiled **30 six-dimensional sectors** through ordinary
positive-coefficient domain admission, without a caller assertion. Generate
stopped at zero sampling; selected compact coefficient/source-chart inspection
left the generated report identical. Integrate, Cancel and Resume retained the
exact **4,096-point** accepted prefix and unchanged paused report, then completed
**245,760 points** with zero failures. All 36 frozen inputs matched; owned
process identities were absent after cleanup.

The complete complex vector uses rows `[-1 real, -1 imag, 0 real, 0 imag]` and
retains all 16 covariance entries. Its four means agree exactly with the retained
CLI result; maximum differences are `3.469446951953614e-18` for standard errors
and `1.3877787807814457e-17` for covariance. This is implementation consistency,
not a new independent amplitude reference. Precision policy and replay state
remain native and unchanged; the report retains 32,550 rescues, a maximum of
256 bits, and zero failures. Finite relative standard error is **1.027254%**, so
the UI correctly reports allocation completion while marking the 0.1% target
unmet.

The browser recorded 102 changing live DOM count observations before ready;
screenshots show actual Geometry progress beyond initial parametrization.
The diagnostic driver's whole-DOM phase regex also sees historical labels, so
those observations must not be called 102 distinct active phases. Generate to
ready took 122.489 seconds in the browser. Integration to completion including
the deliberate pause took 96.409 seconds; reported active wall time was 89.326
seconds. These caller-paced, single-worker UI measurements are not comparable
to the earlier eight-worker CLI benchmark.

The fixed native-only point, native graph, sector overview, selected compact
expressions/chart and full result table render correctly. Independent visual
review found one presentation follow-up: the default history chooses the zero
finite real component although this projected diagram has a nonzero imaginary
component. The full table and target are correct. An informative history choice
and compact disclosure of zero histories should be checked using retained data,
without repeating ggHH science. The accepted run is frozen independently of any
such display-only change. Its source/data/rendering review is retained in
`gghh-ui-1/independent-review.json`.

### Retained-data presentation follow-up

The history display correction is independently accepted. Informative histories
appear first; histories whose displayed means **and** standard errors are all
zero remain available in a compact disclosure. The disclosure explicitly states
that recorded zeros do not establish symbolic exactness. A zero mean with
nonzero uncertainty stays prominent. The Laurent vector, covariance and all
**354 recorded history points** are unchanged. The view now also reads the
existing native aggregate `snapshot.worker_seconds` getter directly and exposes
optional native `stop_detail` in the status table.

Seven focused presentation controls pass. A separate browser replay imports only
the presentation and saved report, with no Symbolica core import or scientific
actions. The informative imaginary plot is visible first; opening the disclosure
shows the retained real-component history. Independent screenshot review and all
eight frozen replay hashes pass; the replay took 5.848 seconds and its owned
processes were reaped. These are display checks, not another numerical run.

The actual ggHH run retains its archived original presentation source. Its only
later application differences are this view change and a cosmetic asset-path
comment whose Python AST is identical. The saved numerical report is unchanged.
Evidence is `hepkit-history-presentation-1/independent-review.json`; no repeated
ggHH generation or integration was used to close this finding.

### Generated stub acceptance

The actual stub utility generated the public FastSecDec package and passed its
installed-export inventory gate. Independent read-only review verified all
**24 public names**, every prior public member, all eight inspection classes,
and Python 3.9 AST compatibility. The generated stub has SHA-256
`28d7324ebad9582039d76bd96dc51c8362af3b6beb1fcb2add1f125c9ae69be3`.
Its inventory was checked against the exact accepted native extension; the
reviewer independently checked the retained evidence, hashes and AST without
repeating the import or compilation.

One dev-profile utility compilation completed in **6m 53s**. Its initial launch
failed with status 127 because the shared Python library was absent from the
loader environment. A bounded invocation of the same frozen executable with
the verified Python library directory failed with status 101 because direct
execution lacked Cargo's `CARGO_MANIFEST_DIR`. Restoring that environment value
produced the stub successfully in **0.67 seconds**. Both failure prefixes remain
retained; neither required a rebuild, dependency patch or numerical execution.
The generated stub is the sole intentional source output. All 254, 259 and 262
frozen input sets from the three attempts match, all owned sessions are absent,
and the native wheel remains unchanged. Evidence is
`hepkit-binding-relocation-1/independent-stub-result-review.json`.

### Published source and portable-runtime acceptance, 2026-10-06

FastSecDec `539019a72622d0997e7ee2da8c21101234df228a` and community
`d82eff433f187f818e22d811ef7e1654744447a3` publish the reviewed ownership split.
The thin setup fetches that exact FastSecDec revision, applies the reviewed
shared-owner patches and resolves core, sectors and binding to the same Git
source. Fresh source equivalence and unique-owner metadata checks pass.
[Hosted CI](https://github.com/symbolica-dev/symbolica-community/actions/runs/37391742450)
then actually builds and installs the public-source native development wheel;
61 controls pass in 1.79 seconds, followed by the host import check. This is
separate from the earlier local optimized release wheel and native UI timings.

The public-source portable build also succeeds. A retained initial host-linker
failure was corrected by selecting an absolute native-target linker before
Pyodide wrapped generic `cc`; an actual native ELF/Wasm target probe verified
the boundary. No Rust source or Wasm-target linker was changed. The corrected
build took 2,436.859 seconds with 17,575,956,480 bytes peak sampled owned RSS,
within its original deadline. Frozen sources remained unchanged and all owned
processes were reaped. Wheel SHA-256 is
`fcb82c9ed385ad5a598e2722322f1d21a86ad02386c8697c81f23c5da3196006`;
embedded Wasm core SHA-256 is
`54fdf1169fdd8f36c5968be269daa4ad773971a3e908d27a919e8e8a51eb68df`.

Generic Pyodide smoke initially rejected 22 inventory-constructor globals from
the newly linked binding crate. The independently reviewed host test adds only
that anchored namespace, retains the exact function allowlist and rejects 110
negative controls. The unchanged wheel then passes generic smoke. The focused
suite passes **50 tests in 3.31 seconds**, with no failures or skips: 18 binding
API, six inspection, 12 input and 14 shared wavefunction cases. An obsolete
planned count of 46 initially rejected the report; independent enumeration of
the frozen test sources reconciled the same successful run without a rerun or
weaker test criterion. All failure prefixes and source/process checks remain.

Evidence is `hepkit-public-leaf-1/hosted-ci-success-1`,
`hepkit-relocated-wasm-1/host-link-correction-1` and
`hepkit-relocated-portable-runtime-2`. The optional browser gg→HH demo is a
subsequent Python/assets change; its showcase identity must be recorded apart
from the compiled wheel revision. Neither these runtime controls nor a static
export establish the updated notebook's browser lifecycle or gg→HH completion.
