# LTD dynamic generation expression-memory audit

2026-10-10. Initial read-only source review and ignored native probes after
`fd8ce34`, followed by the explicitly approved native definition integration.
The contour prescription, dependency identities and memory limit are unchanged.
The runtime owner independently owns the actual chart map/determinant probes;
the foundation reviewer owns the companion native alias-contract audit.

## Observed failure and actual sector degrees

The native 2L4P.b.K1 input has 186 retained shared chart sources. Both the
four-worker attempt and the subsequent single-worker attempt hit the declared
3 GiB aggregate limit before any dynamic discovered-chart receipt. Reducing
worker count therefore does not resolve the first chart's expression growth.
These are generation failures, not failed numerical integrals or zero results.

The actual first chart was loaded through public `IndexedReader` from
`target/contour-ltd-template-k1/integral.fsd.json` and its native data file.
The coordinate dimension is six. A focused public native polynomial conversion
of its small, original retained factors gives:

| Factor | Native Atom bytes | Polynomial terms | Total degree | Degrees by coordinate |
|---|---:|---:|---:|---|
| F | 1449 | 31 | 6 | 1,1,1,1,2,1 |
| U | 220 | 15 | 5 | 1,1,1,1,2,1 |

These are post-sector residual degrees, not the original loop-count degrees.
The full-sector envelope retains causal orders 3,5 and U orders 2,4. Its dense
radius coefficient schema is `[a2,a4,a6,a8]`, with the a6 slot exactly zero;
that slot must not be removed on a face or for a numerical point.

The runtime owner's actual production-algebra probe recorded:

| Stage | Observation |
|---|---|
| Native envelope | 0.034 s incremental |
| Native coefficient collection | 0.155 s incremental; coefficient bytes 4709,379750,3,413440 |
| Registered native strength | 797924 bytes |
| Six images | about 798–799 kB each |
| 36 native Jacobian entries | 15.5–18.6 MB each; all rows complete by 5.77 s elapsed |
| Existing determinant implementation | starts at 5.77 s; 3 GiB limit stops the process at about 8.10 s |

The probe imported an existing dynamic FK05 native record to register the
actual IFT hook, then checked the native identity dλ/dS=λ/S before constructing
the K1 map. The earlier missing-hook probe stopped at its sanity assertion and
is not evidence about map cost. The successful-stage source includes the
unchanged production `contour/determinant.rs`. Its evidence is under
`target/contour-ltd-dynamic-k1/{map-probe.rs,runs/map-native-registered-chart0/}`.
No evaluator construction, Horner pass or CPE had been reached at the failure.

## Source reuse and expression growth

`contour/dynamic/envelope.rs` uses native differentiation of the Hessian,
native directional series with independent direction placeholders, and native
substitution of the physical direction afterwards. It preserves powers and
products. Polynomial degree uses homogeneous scaling; coefficient extraction
collects only the radius variable over `AtomField`. The public owner source
`poly.rs::to_polynomial_in_vars_impl` explicitly preserves other parts in
expression coefficients, and `domains/atom.rs` uses ordinary Atom arithmetic
without a custom normalizer in this call. No FastSecDec `expand`, `together`,
`cancel` or alternative coefficient convolution was found in this envelope
path. The narrow exact-rational U coefficient certificate does convert U into
its coordinate polynomial, appropriately; the actual small-U measurements do
not implicate it in the observed limit.

`generation/mapping/contour/program.rs` puts the complete coefficient Atoms
inside every strength callback before `SmoothContourMap::new` differentiates
the full images. The registered native derivative hook correctly applies IFT,
but its output necessarily repeats the full strength call and its coefficient
arguments. Native Atom differentiation applies the chain rule; the resulting
flat expression is large even though its factored mathematical representation
is compact.

The 4–6D determinant optimization already computes a native determinant in
short entries, avoiding large Bareiss pivot cancellations. It then immediately
substitutes the physical entries into the result. A generic 6D determinant has
720 monomials, so inserting these already large entries repeats them before
any native evaluator graph can share subexpressions. This is the measured
failure boundary. Increasing Horner/CPE settings cannot help code that has not
yet reached evaluator construction. Switching off exact pivot cancellation is
also not a valid remedy: it previously created false poles.

Native coefficient-first/Laurent aliases and numerical-dual source-program
caches exist, but act after this contour-map construction. They do not prevent
the current expansion in residency. Persisting all shared source charts and
running one worker remain useful outer bounds; neither bounds one materialized
Jacobian's size.

## Existing native APIs: capability probes

The isolated probe `target/generation-agent-ltd-memory/alias-probe.rs` links
the coherent Symbolica 516beb3 rlib used by the physical probes. Source/API
inspection and executable controls agree:

1. `AliasedAtom` is an opaque root plus definitions. Its `AtomCore` derivative
   differentiates the root and copies the alias table; it does not differentiate
   hidden definitions. Replacing `(x+1)^3` by a plain opaque alias and then
   differentiating gives zero, while the materialized derivative is nonzero.
   Using that directly for the map would be incorrect.
2. A native `FunctionMap` body by itself does not resolve symbolic
   `der(...,f,args)` nodes. Registering f alone produces an undefined-function
   error for its symbolic derivative.
3. `InliningPolicy::Never` retains a sub-evaluator, but current `Dualizer`
   rejects that body. `EvaluatorComposer::append` likewise explicitly rejects
   non-inlined bodies. No assumption of retained-body higher-jet support is
   justified.
4. **Explicit native derivative-body registration does work.** Registering
   the native `DERIVATIVE` call's static prefix with its formal arguments and
   a body obtained only by `body.derivative(...)` evaluates f, f′ and f″
   correctly. Default `Always` inlining at evaluator build then allows native
   `Dualizer` higher jets. This keeps symbolic function calls short until the
   evaluator boundary without adding differentiation or numerical algorithms.
5. The same route passes nested g=f² and formal-argument shadowing controls,
   then second-order native jets of the complete output vector. The expected
   nested vector is `[729,1458,1215,1458,2430,1620,27,18,3,0,0,0]`.
6. Mixed formal arities produce an actual native tag-count error because
   `FunctionMap` uses one tag count per function symbol, including `DERIVATIVE`.
   A uniform explicit signature per chart avoids this particular conflict.

Thus the tests do not establish an unavoidable missing owner operation. Native
request registration and late inlining are a viable next probe. Never-inline
support or automatic derivative-body lookup would be separate owner features,
not prerequisites already proven necessary.

The repository already uses the same general native request idea in
`generation/coefficient_first/requests.rs`: source-body ownership, native
derivative orders, face substitutions and late alias definitions. Reuse that
contract where possible rather than introducing a second symbolic engine.
The older `subtraction/series_first/named.rs` is test-only evidence, not an
additional production subsystem to copy.

## Required integration boundaries before implementation acceptance

- **Explicit dependencies.** Every compact function's signature must retain
  all relevant coordinates, regulator dependence when present, physical inputs
  and S/L/R. Captured globals are invisible to symbolic differentiation.
  Standardizing the full signature also prevents the native derivative tag
  collision demonstrated above. Preserve the full sector on all faces.
- **Exact symmetry.** `symmetry.rs` currently canonicalizes the literal
  density Atom with Graphica and then proves a native coordinate substitution
  reproduces the representative. Opaque names alone are unsafe: chart-unique
  names lose valid equivalences; reused names with unequal hidden bodies could
  falsely merge charts. The implemented proof instead uses the complete
  branch-aware source density, designated F, ordered U declarations, full
  dimension and recipe before deformation. The existing native graph
  candidate and exact coordinate-substitution proof act on this witness;
  permutation equivariance then establishes equality of the deterministic
  map and its Jacobian. The density reuses the map's `continued_power`, so
  causal and principal fractional powers cannot silently cancel together.
  No alternative polynomial canonicalizer or name-only comparison is used.
- **Faces and higher derivatives.** Native differentiation must precede each
  mathematical face restriction. Existing request caches distinguish
  unbound partials from face-restricted results. Short derivative bodies should
  preserve this ownership instead of setting a coordinate constant before
  differentiating it. Test both Taylor/IBP and actual numerical-dual execution.
- **Saved raw checker sources.** Retain the original full F/U, full-strength
  mathematical root and primitive checker programs. Never substitute rounded
  aliases for the independent proof inputs or recompute structural counts at
  a face. Current root association uses exact native strength Atoms; compact
  callback arguments require a verified native definition mapping before
  associating them with those retained roots.
- **Late diagnostics only.** `kernel/program.rs::build_with_lowering` already
  visits coefficient roots and each alias definition, so the existing late
  lowering boundary can be reused. Mathematical subtraction and exact
  cancellations must finish before request tags appear. The numerical-dual
  boundary must still attach the actual selected face before native jets.
- **Exact contributions.** Direct exact binding evaluates the complete
  mathematical Atom vector with native lazy function-cache tracing. FunctionMap
  is an evaluator-builder facility, not a replacement for that direct path.
  Any needed exact definitions must resolve natively before aggregation and
  preserve root keys, cancellations and lazy IF behavior. Boundedness of that
  resolution is still to be measured; no per-record compiled exact evaluator
  or eager root prebinding is acceptable.
- **Storage and inspection.** `ContourMetadata` currently stores images and
  Jacobian as plain Atoms. Compact names require accompanying native
  definitions and resolvable inspection, not opaque formulas presented as
  complete maps. ProgramData and staged codecs must retain definitions and
  symbols through source release, subprocess restoration and detached jobs.
  Any changed staging shape needs an explicit schema bump. Finished numerical
  records must retain optimized programs; loading must not rebuild the map.

The runtime owner's compact coefficient proof passed for actual chart 0: all
36 compact Jacobian entries materialize to exactly the original native IFT
entries. Compact entries are approximately 6–9 kB, the determinant is 25.6 MB
and completes in 0.124 s. Native Always-inline evaluator construction produces
11763 instructions; the complete bounded lowering probe uses about 602 MB
and 11.49 s. This establishes a measured representation benefit for one chart,
not full-generation acceptance. Complete higher-jet science,
equivalent/non-equivalent chart controls, zero/one faces, actual fresh-process
restoration and exact-offset cancellation controls remain acceptance gates.
No full LTD integral or multiloop performance success is claimed.

The subsequent bounded actual-chart numerical probe passes eight points/caps
using the verified prepared-source-0 record. The compact six images and
determinant are compared with native Dualizer derivatives of the original
expanded six-image vector and the native numerical Matrix determinant at
192-bit precision. Maximum discrepancies are `1.333e-16` for eager,
`4.504e-16` for SymJIT, `5.666e-32` for double-double and `6.238e-58` for
192-bit evaluation, with no callback failures. The probe completes in
2.604 s at 137.3 MiB aggregate peak RSS. Source and raw results are
`target/contour-ltd-dynamic-k1/numeric-compact-probe.rs` and
`target/contour-ltd-dynamic-k1/runs/compact-map-native-numeric-chart0/`.
This directly tests the implemented numerical callback map as well as the
symbolic identity; it still covers one chart rather than the complete integral.

The generation integration retains compact native definitions per chart,
exports their function/formal/body symbols through the existing StateMap
codec, and explicitly bumps generation staging to schema 4. Native source
program cache keys include their definitions before FunctionMap lowering and
Dualizer. Restricted zero or coordinate-independent calls simplify natively
before exact/stochastic classification; only exact outputs are then fully
materialized before aggregate cancellation and lazy direct evaluation.
Finished metadata uses the independently reviewed v12 native wrapper. The
initial combined core all-target check passed in 22.18 s; focused executable
acceptance remains in progress.

The first compiled snapshot passes 11 source/symmetry controls, eight native
dual cache controls, the restricted-call exact-folding control, and 16 streamed
generation controls, including a child-process v12 restore and actual
evaluation after the original owners are dropped. Independent codec gates
pass 48 artifact tests and three metadata tests, including both construction
and generation modes plus selected-record checked evaluation. The program
filter initially passed 13 of 14: the remaining assertion looked for the
smooth-positive symbol directly in the images, whereas the new native
definition body retains it. The assertion now inspects both representations;
its subsequent analytic vector checks are unchanged. These counts precede
the final definition-admission hardening and final rebuild, so are interim
evidence rather than acceptance of the completed source.

The final source also includes a genuine two-chart signature control:
`F = r*x² + t*x*y + t*y²` on the projective simplex. One chart's derivative
depends only on `t`, while the other depends on `r` and `t`. Both modes must
compile these different chart-local signatures, restore them together, and
evaluate complete checked vectors. Uniformity is required inside each native
FunctionMap, not artificially across all metadata in the integral.

The final native library suite passes **401 tests, with 19 explicitly ignored
controls**, in 28.70 s. This includes all eight definition-admission/native
precision controls, the genuine different-signature projective input in both
generation modes, complete cubic/higher-pole Laurent-vector checks, source
symmetry, exact folding, fresh-worker staged restoration, and the full native
artifact/metadata controls. The mixed-signature fixture initially omitted its
required pilot; it now executes the actual pilot for every retained chart and
finishes it before production evaluation. No admission rule was bypassed.
The final raw log is
`target/generation-agent-ltd-memory/compact-final-core-lib-tests.log`.
CLI, lint, installed-consumer and complete K1 generation gates remain separate.

The serial coordinator's source audit finds no new all-chart definition or
Jacobian cache. Source/chart discovery retains indexed record references;
workers compile and write one native unit and return compact receipts.
`ProgramArchiveWriter::append_record` copies those bytes without decoding;
the final metadata-only publication does not restore the chart bodies.
The upcoming process measurement must still include worker scratch, checker
sources, definitions and coordinator RSS. A caller explicitly retaining an
ordinary complete generated object has a different, documented ownership cost.

## First bounded complete-input campaign

The fresh native CLI run uses the unchanged `2l4p_k1/run.toml` input, the
polynomial recipe, symbolic generation, one caller-owned serial worker, and
a predeclared 300-second / 3-GiB aggregate-RSS limit. It prepares all **186
shared sources and persists all 186 dynamic mapped charts**, then records
25 completed exact-symmetry receipts before cooperative cancellation at
**300.261 s**. It has **zero compiled evaluator receipts and no published
artifact**. This is a successful mapping-memory gate, not complete generation
or scientific multiloop acceptance. The earlier representation exhausted more
than 3 GiB before completing its first dynamic chart.

The independent 50-ms process-group monitor observes an aggregate peak of
**246,398,976 bytes (234.984 MiB)**. Its peak worker is the native discovery
job for chart 31; the coordinator's separately sampled maximum is 21.875 MiB.
The bound includes source/checker/definition and map scratch in the worker,
plus the coordinator. RSS sums include shared resident pages per process.
Concurrent workspace and installed-consumer builds were running, so elapsed
time is recorded as feasibility evidence, without an idle-host speed claim.

The private CLI SHA256 is
`0495b4e018a9ffa39aa206ba38a8eaa33a870255dd496ef303ce22133f857176`.
Its frozen workspace source inventory, including all Rust files, workspace
manifests and the root lock, has hash
`7ab154a08196facebccb0de560e33cbab1f944c2dfbbaed711dfb51701bdc8c8`
and was recorded at HEAD `98aa8f9d71f2ea1b62e9f0453b329e0e1557ba63`
plus the pending compact-definition slice. The fresh preparation independently
reports the same canonical physical source identity
`d27e82184cdd7afc90bc12f27b8e6b6bfcd3786007326bb7f8e9c3a5c8b394d5`.

Raw evidence is under `target/generation-agent-ltd-compact-k1/`: `plan.json`,
`native-source-sha256.json`, `preparation-summary.json`,
`first-campaign-summary.json`, `journal-after-first.json`, and
`runs/generation-polynomial-one-worker/{execution.json,rss-samples.json,stderr.log}`.
The fresh schema-4 staging root is `polynomial.fsd.generation/`; the older
failed run remains unchanged. A separately authorized **second 300-second /
3-GiB, one-worker `--resume`** uses the exact same private binary and retained
receipts. Its raw budget and elapsed time are separate, and cumulative time
must include the first 300.261 s. At this documentation cut the resume is in
progress; reaching saved evaluator compilation remains pending.

A separate native parity probe verifies that the failed new run's actual
prepared-source-0 record has exactly the retained fixed chart's F, U and
ordered parameters. Its immutable source identity is
`d27e82184cdd7afc90bc12f27b8e6b6bfcd3786007326bb7f8e9c3a5c8b394d5`;
record BLAKE3 is
`f2b1b9c567b44cbce7a616ef9f05c452ad0adf2e64c137b2d3bd13da320638f3`
(3743 bytes). Public RecordRef verification precedes native schema-3
StateMap decoding. Evidence is
`target/generation-agent-ltd-memory/source-parity.{rs,log}`. This closes the
source-attribution gap between the failed run and the bounded map probes.

## Probe provenance

Both local capability probes are direct `rustc` consumers under `nix-shell`,
without Cargo/dependency mutation or a new host build. The native degree probe
loads the existing saved fixed chart through FastSecDec's public reader.
Sources, executables, build logs and output remain ignored under
`target/generation-agent-ltd-memory/`.

| File | SHA256 |
|---|---|
| alias-probe.rs | `0162ee120a972e487d18d37c7cf6a567fd7f5b777c55a36f1200e566c730bf33` |
| alias-probe | `2026c90fac64cc8f4369f9c8bc74d318ffc28b12c0318d4a9e00696c98a79a3c` |
| degrees-probe.rs | `8077c3be8b19eb964fc66f321f5e1a6ec98cd7ac00a341633736e797cdbcf9fa` |
| degrees-probe | `cb45a7af3e652469bc302e43543cefcf15870da2eeaedbb0f7e12a530441beae` |
