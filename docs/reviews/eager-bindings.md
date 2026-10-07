# Eager notebook binding implementation

The native and portable Rust libraries remain Python-free. FastSecDec's isolated
binding crate owns the new PyO3 API; the community host only registers/reexports
its native objects and stubs. The notebook uses one calling thread and explicit
work steps, with no implicit sampling or compilation in constructors or views.

## Native owner reuse

`EvaluatorBackend::Eager` maps the existing exact Symbolica program to its
`ExpressionEvaluator<f64>` or `ExpressionEvaluator<Complex<f64>>` using the
existing function-capability mapping. The native build can now select that owner
without invoking JIT compilation. `Auto` preserves the CLI's SymJIT O2 default;
portable Auto remains eager. The new Auto field is omitted from serialized
compiler settings, preserving the previous canonical policy representation.
Explicit eager policy is retained in the binary cache and on cold loading.

`CompilationSession` reuses `CompilationJob::run` and `KernelSet::finish` at
sector and final-assembly boundaries. Completed evaluator owners stay retained
while paused. The native generation session is independently documented in
[cooperative-generation-review.md](cooperative-generation-review.md). The binding
composes parameterization, that generator and the compiler under one Generate
owner. False callbacks latch for the whole step; Python exceptions and
KeyboardInterrupt propagate only after the completed native unit is retained.
Native errors are terminal. A single algebra/evaluator-build unit remains
indivisible, so this is a work-unit budget rather than a time limit.

HEPKit's `RuntimeModelBindings` continues to own model dependency resolution,
independent real/complex leaf inputs, mass constraints and default metadata.
Explicit kinematic symbols join that schema. Bindings expose defaults without
binding them automatically. `with_parameters` clones native evaluator owners,
applies the complete real input point and validates mass constraints. Rebinding
changes numerical identity and leaves template bytes unchanged. Named model
masses stay symbolic before sector support analysis; zero-width restrictions and
nonzero runtime mass requirements are the existing core contract.

QMC and discrete Havana steps use the core batch worker APIs and
`WeightedEvaluationContext::evaluate_weighted_batch_controlled`. Matrix primaries
remain actual native SymJIT calls when selected; eager/DoubleFloat/arbitrary
precision use the owner's row calls, with no replacement interpreter. The bridge
records native attempted timing deltas once, including speculative rows beyond a
failed/cancelled prefix. Only complete successful statistical packages update
accepted estimates, replay state and checkpoints. The caller can vary numerical
batch size across continuation.

Read-only `IntegrationObservation` and `LiveObservation` wrap core results.
Accepted full-vector covariance is never reconstructed from sector marginals.
QMC previews use native complete-lattice coverage; MC previews pool the native
point accumulators from completed steps in the current phase, or only since a
restored checkpoint. Phase changes clear previews. These views never enter
stopping or checkpoint data.

Explicit selected-coefficient inspection uses
`CompactCoefficient.expression()`, a direct call to native
`AliasedAtom::clone().into_inner()`. It restores only that coefficient, preserves
the retained compact owner and does not introduce an alias walker. Native rich
representations continue to read the compact root and metadata without this
operation; the notebook invokes it only after Inspect.

An independent check of the ggHH input bridge confirmed that native
`overall_factor_expression(evaluate=True)` evaluates graph sign, multiplicity
and symmetry annotations only; model coupling symbols remain analytical. The
actual notebook admission failure was an undeclared reality attribute on its
Gram symbols. They now use Symbolica's public `S(..., is_real=True)` at creation,
preserving the existing core input contract rather than fixing physical values
or weakening mass/schema checks.

## Current validation

- Native eager core controls passed: retained versus synchronous compilation is
  byte-identical; runtime complex Laurent vectors survive cold loading; clone
  and rebind preserve template bytes and give independent physical identities;
  eager batches make zero matrix calls; Auto settings omit the new field.
- Native binding `cargo check` and strict Clippy with `python_stubgen` passed.
- Portable binding `cargo check --no-default-features --features
  portable,python_stubgen` passed. This is a feature build, not a new browser
  runtime claim.
- New installed-Python controls cover callback pause/KeyboardInterrupt retention,
  eager cold-load policy, explicit physical points, QMC/Havana batch-size changes,
  checkpoint continuation and provisional versus accepted observations. The
  final Apple-Clang-linked release community wheel passed all 73 binding tests
  in 1.63 seconds, without failures or skips. This includes all native rich
  object/getter protocols, small nonzero Laurent values and full covariance,
  explicit selected alias restoration, the real Marimo paged-expression widget,
  bounded page caching, resource closure and immutable artifact/checkpoint data.
  The first run found a missing explicit compile-method PyO3 text signature;
  restoring it made progress='auto' discoverable without changing execution.
  A rich-view fixture's invalid zero distance thresholds were corrected to
  legal positive values; native validation remained unchanged.
- Existing numerical-reference Python controls explicitly declare fixed points;
  historical threshold-certificate expectations now assert user responsibility.
  Native binary artifact tests use the real private Payload and encoder for
  semantic mutation controls rather than maintaining a second binary codec.
  The legacy singular-box/rank-two/sunset layout smoke explicitly retains its
  original validated policy; dedicated new eager workflow controls use distance
  routing. A sampled slow run confirmed Symbolica's 1000-digit polygamma constant
  preparation, not a dispatcher deadlock, before this policy was made explicit.
- The migrated public kernel-artifact suite passed all 12 controls, including
  native/legacy cold loads, full vectors, malformed transport, optimizer policy,
  exact endpoint metadata and explicit validated underflow checks. Private
  binary semantic mutation controls and eager-backend units also passed in the
  Clang-linked full core gate; unrelated migrated assertions are tracked there.
- CLI persistence, configuration, replay/checkpoint and worker-slot controls
  passed. Replay fixtures explicitly select the validated policy that their
  assertions exercise; equality excludes elapsed nanoseconds while retaining
  native evaluator point/invocation counts and all scientific diagnostics.

## Local linker control

The host's `cc` was MacPorts GCC 14. A standalone `catch_unwind` program linked
through that driver aborted with native unwind error 5 on Rust 1.99, 1.97.1 and
1.89, including with an independently selected LLVM libunwind. The identical
program linked with Apple Clang or LLVM 18 Clang caught the panic successfully.
This is a local linker-driver issue, not a FastSecDec panic-recovery change.
Relinking the CLI test harness with Apple Clang made all three intentional
worker/coordinator panic-and-drain controls pass. Permanent recovery tests were
retained. The final workspace gate uses the scoped Cargo target-linker override
documented in the development guide; no global compiler configuration changed.

Independent binding/backend review is recorded in
[notebook-workflow-reuse.md](notebook-workflow-reuse.md), including retained
compilation, runtime binding, cold eager policy, exact-only offsets, batch sizes
1/7/256 and real/complex DD106 plus Arb3322 endpoint controls. Notebook UI and
actual installed-host acceptance are recorded separately.

## Cross-review of native retained generation

The binding/backend author independently read the generator's stage transitions
and shared assembly against the synchronous path. Geometry uses the existing
GeometryPlan/PreparedGeometry owners, retains completed chart/cone jobs, and
admits them in native deterministic order. Mapping and symmetry reuse the same
work functions and registry. The ordered representative map, template cache,
exact multiplicities and endpoint profiles remain owned across steps; shared
Assembly performs the same zero-dimensional folding and full Laurent padding.
The callback bridge records a pause while permitting the indivisible unit to
finish, and an error leaves a terminal Failed state without a partial result.
No additional graph/algebra helper, thread or pool was introduced. No additional
source finding was identified; root's independent review and the four native
plus four portable session controls provide the primary acceptance record.

## Installed notebook option controls

The final installed release wheel was exercised through the actual `Study`,
`RunState`, scientific helpers and native Marimo views, without a browser or a
CLI subprocess. Each successful control explicitly generated eager kernels,
inspected the selected coefficient, integrated with QMC followed by Havana,
saved each native checkpoint and resumed the completed session unchanged.
The scalar controls used spacelike invariants −1, mass 1 where applicable,
maximum epsilon order 0 and the frontend minimum of 1024 points and two
replicas, with the default distance policy and batch size 256.

| Selected input | Native sectors | QMC | Havana |
| --- | ---: | --- | --- |
| Default ggHH D035 | 0 | Exact zero, 0 points | Exact zero, 0 points |
| Massive triangle | 2 | Complete, 4096 points | Complete, 2048 production points |
| Rank-two box | 12 | Complete, 24576 points | Complete, 2048 production points |
| Coupled sunset | 6 | Complete, 12288 points | Complete, 2048 production points |
| Scalar massless box | 3 | First package exceeded the 120-second probe budget | Complete, 2048 production points |

The exact-zero D035 control also proves that the native Havana session enters
production and completes without sampling; it does not leave the frontend
waiting for a pilot that cannot produce statistics. Its result contains the
native exact zero mean and uncertainty, and checkpoint/resume preserves that
state. No special frontend zero or extra phase transition was added.

Ignored evidence is retained under
`output/notebook-workflow/option-probes/`: the actual workflow source, separate
case logs, the massless-box Havana log and the sampled native call stack.
The triangle, rank-two box and sunset whole-process controls completed in
0.65, 1.44 and 1.11 seconds respectively; these are acceptance observations,
not comparative benchmarks.

### Cold high-precision constant limitation

The massless-box Generate and Inspect actions completed in about 0.25 seconds.
Its first default Korobov3 QMC package then entered the native 3322-bit mapping
of the finite coefficient's `polygamma(1,2)` constant. A sampled stack places
the work in Symbolica's `ExpressionEvaluator::map_coeff_with_prec`,
`ExternalFunctionContainer::evaluate_constant`, `polygamma_checked` and its
exact Bernoulli-number/GMP arithmetic. The probe terminated its own child at
120 seconds; no numerical error or zero result was reported by the library.
The same integral's default-distance Havana workflow completed normally.

Source inspection of the installed Symbolica owner at revision `58652fa`
confirms that exact normalization covers `polygamma(1,1)` but not this
`polygamma(1,2)` recurrence. The private high-precision numerical fallback
reconstructs Bernoulli numbers during its series loop. There is no public
memoized arbitrary-precision fixed-function mapping interface to reuse here.
FastSecDec does not add a recurrence, replace the constant with an f64 value,
or silently change the requested stability policy.

The existing native `PrecisionCache` retains a successfully mapped evaluator
per precision, with four entries per weighted context; it does not repeat
constant mapping for each point. Havana retains lazy per-sector contexts
across batches. The single-core QMC binding deliberately retains one active
sector context to bound evaluator memory, and the native scheduler rotates
sectors, so a later sector revisit can require another mapping. That tradeoff
does not explain this observed delay: the budget was already exceeded on the
first cold mapping. The initial high-precision special-function preparation
remains an indivisible native operation and an explicit performance limitation.

## Notebook summary formatting

The human integration tables now pair native means and standard errors using
signed normalized scientific notation, Unicode exponents and last-digit
parentheses, for example `+1.234(56) ·10⁻¹`. Missing or invalid uncertainty stays
explicitly unavailable. Counts use the CLI's four-significant-digit base-1000
K/M/B convention, including carry at a unit boundary and exact integer rounding
through `u64::MAX`. Current and retained previous allocations share these
presentation helpers. Raw vector rows, history values, covariance and JSON
reports are unchanged; covariance display formatting operates on the existing
native values.

The installed Python API and pinned native Python binding source expose scalar
`Float` formatting and `NumericalIntegrator`'s internal statistics printing,
but no callable formatter for a supplied mean/error pair. A small standard
`Decimal` adapter therefore places and rounds display digits only, using the
native Python float formatter to round the supplied error to two significant
digits. It never estimates errors, changes precision routing, combines samples
or modifies Python's global decimal context. Focused controls cover ordinary,
zero, missing-error, dominant-error, decade-carry and finite-extreme cases,
compact counts and unchanged native data/export. The complete notebook suite
passed 85 tests after this change; the focused formatting/export subset passed
29 tests after the final restriction to integration-only scientific tables.
