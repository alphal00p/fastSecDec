# HEPKit binding ownership and native API audit

Date: 2026-10-05. Scope: the existing community notebook bridge and the latest
user correction. This review changes ownership; it does not accept new numerical
performance or browser responsiveness. The numerical core and default CLI remain
Python-free. The previous requirement to implement every PyO3 binding inside
community is superseded by the user's explicit correction in `FIRST_PHASE_PLAN.md`.

The decision below is implemented locally. The [relocation audit](hepkit-relocation-audit.md)
accepts the native release wheel, 61 installed controls, generated stubs and
actual native triangle/gg→HH notebook lifecycles. Publication of the new
dependency pin and current portable execution remain separate delivery gates.

## Decision

Use an isolated `bindings/python` package named `fastsecdec-python`, with its own
`[workspace]`, resolver 3 and lockfile, explicitly excluded from the root
workspace. It is an `rlib` which exports module registration; community remains
the extension-module distributor. There is no second independent Python package
or extension-module initializer to maintain.

The current root and portable-consumer lockfiles have no PyO3 or FeynKit-Py
packages. Adding a root workspace member would make existing
`cargo ... --workspace --all-targets` gates select that member and resolve its
Python dependencies even though `default-members` selects the CLI. Disabling a
default feature does not remove a selected workspace package. The isolated
layout preserves the existing root gate and lock boundaries; the repository
already uses this arrangement for `tests/portable-kernel`. Cargo discovers a
requested Git package anywhere in the fetched repository, so the nested binding
can still be consumed by package name and one exact public revision. Validate
that final published consumer graph before release. See the official
[workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [Git dependency documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories).

The dependency graph is acyclic:

```text
community extension -> fastsecdec-python -> fastsecdec -> native ecosystem
                      fastsecdec-python -> feynkit-py / Symbolica Python exports
```

The leaf depends on core through `../../crates/fastsecdec`, with default features
disabled. Within a Git dependency Cargo retains the same repository source for
these internal packages. It must never depend on community itself. Use the same
FeynKit-Py package/source, Symbolica 3.0.1 `python_export`, PyO3 0.28 and optional
stub generator 0.17 as the existing ecosystem bridges. Do not introduce another
Symbolica owner or copy of its global symbol table. The existing
`one-loop-reduce-python` crate demonstrates the appropriate PyO3 ownership:
`abi3` and `extension-module` belong to the consuming host, not the leaf.

| Leaf feature | Forwarding and purpose |
|---|---|
| `native` | Core native evaluators, FeynKit-Py native, Symbolica GMP/MPFR and native code generation |
| `portable` | Core portable evaluators, FeynKit-Py wasm, Symbolica wasm; no native backend |
| `python_stubgen` | Optional stub inventory plus existing FeynKit-Py/Symbolica stub features |
| default | Native for direct leaf development; community disables leaf defaults and selects its backend explicitly |

Community's experimental feature selects this optional leaf. Its native/wasm
features forward weakly to the leaf's native/portable features. Keep the feature
opt-in until the upstream owner patches are integrated. Consumer-root patches
remain authoritative; downstream `[patch]` sections do not configure the host.
The bootstrap needs a binding consumer scope with the existing Python owners
and a local leaf override for development, while published-Git validation removes
all FastSecDec local overrides. Native and portable metadata must each resolve
one core, Symbolica, Numerica, FeynKit-Py, graph, kinematics and PyO3 owner.

## What moves and what remains

Move the existing `input`, `generation`, `kernels`, `session`, `status`, `error`
and registration code out of community. The long status module can be split by
native status family without changing its public Python classes. Preserve the
public `symbolica.community.hepkit.fastsecdec` class/module paths, exception types,
typed arguments, native artifact bytes and checkpoint envelope.

Community keeps the optional dependency/features, the registration call, the
thin Python facade and generated/reexported stubs. FastSecDec-specific exception
stub additions should be supplied by the binding crate rather than custom API
implementation in community's stub tool. Generic HEPKit, wavefunction and model
bindings remain with their existing owners.

Move the showcase, FastSecDec input/view/ggHH helpers, dedicated fixtures, export
helper, dedicated tests and build/use instructions into FastSecDec's examples or
binding directories. Community can retain a link and a small opt-in integration
smoke test. Move the substantive FastSecDec dependency validation and test/build
guidance too; retain only the minimum host workflow necessary to build/register
the optional module. Do not copy the same example implementation into both repos.
Generic `test_hep_wavefunctions.py` is not a FastSecDec-owned test.

## Native boundary and reuse findings

The existing bridge already accepts FeynKit-Py's `PyFeynmanDiagram` and
`PyKinematics`, and Symbolica's `PythonExpression`. It calls native graph builders,
`ParametricIntegrand::from_graph`, generation, kernel compilation, QMC session and
weighted worker APIs. Its snapshots wrap native structs. No alternate graph,
sector algorithm, CAS, QMC estimator or uncertainty combination was found.

The bridge nevertheless owns substantial execution glue: the serial package
driver, replay/context lifecycle, diagnostics, observer/error conversion and an
outer checkpoint envelope. Those belong in FastSecDec's binding crate. Keep the
library caller-driven: the caller requests a bounded `step`, supplies observers
and chooses cancellation. Do not introduce an autonomous integration loop or
worker pool during this move. Promoting reusable Python-free orchestration into
core would be a separate native API change, justified by actual reuse rather
than by relocation alone.

The ggHH scalar binding helper composes existing Model/card and Symbolica APIs;
its analytic internal parameters, card overrides and exact external data were
independently checked against the native CLI. It is a fixed-example policy,
not a general evaluator. Moving it does not justify duplicating model algebra or
silently replacing symbolic internal definitions with rounded model values.

Core now has additive `Periodization::Korobov2`, while the older community pin
only matches `None`/`Korobov3`. Moving to the current core requires the corresponding
string parser/getter arm. Keep `korobov3` as default and preserve the previous enum
tags, checkpoint identity checks and all weighting/replay rules.

## Required sector exploration and status work

The present Python `GeneratedIntegral` exposes only orders, sector count, exact
coefficients and snapshot/compile. The notebook then discards the generated
object. Core already retains the needed introspection; no new algebra is needed:

| Python view to expose | Existing native owner |
|---|---|
| Domain and factor certificates | `GeneratedIntegral::metadata`, `GenerationMetadata::domain_assessment`, `DomainAssessment`, `FactorAssessment` |
| All-chart overview and equivalence | `GenerationMetadata::charts`, `ChartRecord` source index, representative, permutation and optional kernel sector |
| Coordinate pullback detail | `CoordinateMap` source/target parameters, images, positive measure Jacobian, source domain and fixed projective parameter |
| Sector statistics | `GeneratedSector` dimension, parameters, conditioning basis, cancellation degree/terms and native `SectorMap` |
| Compact Laurent expressions | `GeneratedSector::aliased_coefficients`, Symbolica `AliasedAtom::get_root`, `get_aliases`, `get_byte_size` |
| Reloaded kernel metadata | `KernelSet::generation_metadata`; optional for legacy artifacts |

Retain one shared generated owner and return indexed immutable views. A sector
list should not clone all expressions. Return native Symbolica expressions for
requested roots and definitions, preserving alias sharing. Do not call
`GeneratedSector::coefficients()` for the overview: that explicitly materializes
and caches complete expressions. The searched Symbolica Python module has no
existing AliasedAtom wrapper; use a small read-only view over its public Rust
getters, not another alias engine. Stable presentation ordering may sort a copied
list of definition keys without rewriting expressions.

Distinguish source charts from compiled sectors. A chart with `kernel_sector=None`
may be exact, cancelled or truncated at the requested order; metadata does not
distinguish these cases or retain a separate exact contribution per chart. Do not
invent those classifications. Coordinate maps describe the density pullback
before endpoint subtraction, not every final boundary term. Reloaded kernels
retain metadata but do not reconstruct a fresh generated symbolic coefficient
object; show that limitation explicitly.

Generation and integration snapshots already contain useful native stages,
counts, timings, full signed Laurent vectors, real/imaginary components,
covariance, coverage and failure states. Presentation should consume those
structured values, not parse `detail` text. Current Python execution is synchronous
and cancellation occurs at emitted native events or package/sample checks; do
not promise fine-grained interruption inside a long native phase. The binding
currently exposes democratic serial QMC only. Core adaptive/Havana and external
worker APIs, saved-result/reference-comparison APIs and supplied rules remain
unexposed; relocation is not evidence that those Python features exist.

## Migration gates

1. Preserve existing source/evidence before moving code. Keep class paths,
   exceptions and numerical behavior stable; update only the required K2 enum
   coverage. Publish one binding source, not two competing copies.
2. Check root and portable metadata remain Python-free. Check isolated binding
   and community native/portable graphs have unique owners and mutually exclusive
   backends. Validate the actual published Git package after the root milestone.
3. Build the relocated binding and host, regenerate stubs, and run the existing
   meaningful native bridge/input/ggHH controls against that new binary. Old
   wheel tests remain old-layout evidence. Check artifacts/checkpoints, typed
   HEPKit arguments, signed complex vectors/covariance, observer exceptions,
   cancellation and accepted-prefix resume. Add only necessary relocation/API
   controls, including the K2 setting roundtrip.
4. Add lazy native sector views separately and test ownership lifetimes,
   chart-to-kernel mappings, alias roots/definitions, legacy metadata absence and
   exploration without evaluation. Keep algebraic tests small and exact.
5. Replace the combined notebook action: **Generate** retains generated objects
   and compiled kernels at zero samples without creating a QMC session;
   **Integrate** alone creates/advances the session. Require explicit numerator
   preparation where contraction is expensive. Input edits invalidate dependent
   state, and stale results must not acquire the new input identity.
6. Exercise the real notebook controls and visible event updates, including
   sector exploration before integration, cancellation/resume and the fixed
   native ggHH example. A rendering replay or unpaced API driver cannot establish
   smooth live UI operation or browser performance.

The architecture decision is accepted by the coordinator. At the time of this
review the relocation and new interaction gates are pending implementation;
previous performance data and completed source audits remain separately valid.
