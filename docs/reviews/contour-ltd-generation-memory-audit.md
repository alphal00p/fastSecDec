# LTD dynamic generation expression-memory audit

2026-10-10. Read-only source review and ignored native probes after `fd8ce34`.
No production source, dependency, contour prescription or memory limit changed.
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
  falsely merge charts. The same native body definitions must participate in
  the candidate graph and final exact proof. No alternative polynomial
  canonicalizer, name-only comparison or omission of this proof is warranted.
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

The runtime owner is measuring a bounded compact-coefficient chart-0 map next.
That probe must establish actual memory benefit before widening implementation
scope. One successful chart would still require complete higher-jet science,
equivalent/non-equivalent chart controls, zero/one faces, actual fresh-process
restoration and an exact-offset cancellation control before accepting the
representation. No full LTD integral or multiloop performance success is claimed.

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
