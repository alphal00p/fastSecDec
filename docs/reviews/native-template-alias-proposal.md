# Native Laurent templates and evaluator aliases: bounded proposal

This is a source-reviewed proposal, not an executed result or a production
strategy change. The actual callback attempt remains a negative result in
`native-formal-actual-prototype.md`: preparation completed, but the unchanged
180-second generation bound expired before a complete Laurent vector existed.
The next experiment should first isolate the existing production representation
from callback-induced extra subtraction pieces.

## First compare the existing production capture

The original index-38 capture has 241 subtraction pieces; the callback variant
has 384. The first route therefore uses the original production expression,
its exact existing late template and all 201 image definitions from
`output/diagnostics/laurent-capture/index38`. It neither regenerates callback
subtraction nor silently discards its extra pieces. The fixture, original
expression, parameter/regulator names, representative index 38, source chart 46,
requested maximum 0 and absence of representative multiplicity remain bound to
the three independent point-first oracles.

The recorded native relative expansion already produced orders −5 through 0
in 37.332 seconds with final native absolute bound 1. The six template
coefficients occupy 50,571,155 Atom bytes, versus 350,726,808 bytes after image
restoration. The finite coefficient alone occupies 36,898,483 versus
256,030,180 bytes. All 201 image bodies together occupy 21,513 recorded Atom
bytes. These historical values identify a representation opportunity; they are
not fresh timing or alias-builder evidence. The original capture reports
5,736,028 template bytes and the later import/replay reports 5,717,264, so native
Atom byte counts must always retain their process/source context.

`crates/fastsecdec/src/generation/laurent.rs:37` performs the existing selection
after endpoint subtraction: replace a non-numeric coordinate-dependent subtree
only when it contains no regulator. The exact selection, collision guard,
native replacement traversal and native Atom equality must be reused. This
step is distinct from hiding only polynomial factors before differentiation.
It is legitimate for a late image to contain logarithms or rational functions:
no later coordinate differentiation or endpoint substitution is proposed.
Existing mapped domain and face certificates remain authoritative.

## Native ownership and the remaining executable checks

Paths in this section are relative to the pinned Symbolica worktree,
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica`.

- `src/atom/alias.rs:158` exposes `AliasedAtom::register_alias`;
  `evaluator_multiple` at line 178 passes roots and definitions to the native
  builder. Native `get_byte_size` includes the root, handles and bodies.
  `into_inner` explicitly restores the Atom and is excluded from the large
  evaluator path. Small controls may use it to prove exact restoration.
  The multi-output convenience method registers each output's aliases in turn,
  while `FunctionMap` rejects duplicate keys even with identical bodies. Thus
  six separate objects each containing the same 201 images are not the proposed
  path. Use the shared native builder and register the image map exactly once;
  preserve duplicate-registration errors in a focused small control.
- `src/evaluate/function_map.rs:156` and line 382 expose native `add_aliases`.
  Aliases always inline in the caller's scope. This means inlining into native
  evaluator instructions, not a requirement to materialize the giant restored
  coefficient Atom first. Register the one shared image map once for the whole
  coefficient vector; no FastSecDec alias graph or symbolic serializer is
  needed.
- `src/evaluate/tree.rs:944–993` lowers alias bodies using the caller's native
  bindings and subexpression cache. The common multi-output lowering context
  starts at line 657. Native instruction deduplication owns any sharing; the
  proposal must measure resulting IR rather than assume every alias occurrence
  yields one instruction or claim aliases remain separate runtime functions.
- A separate optional policy uses native ordinary zero-argument functions
  registered under the placeholder symbols, with `InliningPolicy::Never`.
  The Var branch at `tree.rs:1055–1083` supports this call shape. Ordinary
  definitions see declared global evaluator inputs, while aliases inherit
  caller-local bindings; these are different scope contracts. A focused
  global-input/shadowing control must pass before applying this policy to the
  capture. Keep direct translation enabled for both policies because Never
  forces that path (`function_map.rs:463`).
- Upstream controls at `src/evaluate.rs:758`, 889 and 932 cover alias resolution
  inside retained functions and recursive-definition rejection. Alias native
  ownership and body-inclusive sizes are tested in `src/atom/alias.rs:640`.
  They do not replace the proposed complete-vector executable proof.
- Native `ExpressionEvaluator<Complex<Rational>>` serde stores instructions,
  coefficients, external constants and retained function bodies
  (`src/evaluate/evaluator.rs:19`, 109–157). Native
  `map_coeff_with_prec` at line 653 remaps that same IR for MPFR. Its bincode
  transport, not an independent definition transport, is the cold numeric
  artifact. No callbacks or polynomial registry are installed in the reader.

Aliases inherit scope, so source/input/placeholder symbol collisions are checked
before builder registration. Each original image must be regulator-free and
must not contain another generated placeholder; native builder errors remain
errors. Any optional nested aliases in a small control use native dependency
resolution and recursion checks. The actual template route needs no new
dependency graph, topological sorter or normalization hook.

## Staged diagnostic

1. **Small source controls.** Compare native late-template series with direct
   native series for complete vectors containing Gamma/prefactor poles,
   negative requested maxima, exact cancellation and zero coefficients. Include
   literal underscore-suffixed symbols, a complex image body, an admitted
   logarithmic/rational late image, global coordinate bindings and retained
   zero-argument definitions. Require nonempty expected order sets where
   appropriate, preserve fractional/essential-series errors, and prove a fresh
   native-IR reader, numeric clone and MPFR remapping work. This is the missing
   third reuse check for this particular composition.
2. **Original template baseline from saved coefficients.** First reuse all six
   frozen template coefficient Atoms from the already validated native
   absolute/relative replay, together with all original image pairs. Bind their
   exact file hashes and the earlier exact-vector equality evidence. Verify
   exact native restoration of the compact epsilon-dependent template against
   the original 14.5 MB expression, without expanding it. This isolates the
   builder boundary without repeating the 37-second source series. It is
   labelled warm input reuse and excluded from fresh generation timing.
3. **Native aliases and numeric transport.** Feed all complete template
   coefficients and image definitions to one native builder, first with
   `add_aliases`. A zero-argument Never comparison follows only if its small
   scope control passes. Keep direct translation, optimization settings and O2
   fixed and recorded. Export native exact IR. Report import, preparation,
   series, builder, JIT, serialization/load and evaluation separately, plus
   root bytes, image-handle/body bytes, IR bytes, output/order counts, operation
   counts if publicly available and peak process RSS.
4. **Independent numerical acceptance.** Reuse the existing source-hash-bound
   three rational points and all-order union comparison in
   `output/probes/formal_actual/numeric.rs`: 512/1024-bit agreement and oracle
   agreement at every order −5…0, no representative multiplicity. Retain raw
   eager/O2 values and error estimates at the rounded binary64 inputs; do not
   certify those by comparison at different exact rational inputs. Preserve
   cancellation tuples and the existing lost-bit lower bound. The native
   scale-before-conversion weight check remains arithmetic evidence, not a
   claim that this experiment exercises the production adaptive replay policy.
5. **Fresh generation, only after lowering and numerical checks work.**
   Reproduce the captured template and images with the unchanged selection,
   run the already controlled native relative-depth helper, require its final
   absolute bound, and compare every native template coefficient with the saved
   relative replay. Report this stage independently from saved-input lowering.
6. **Callback-template comparison, only afterward.** Apply the identical late
   template selection to the live callback subtraction, under one combined
   preparation/series bound. Before evaluator construction, resolve callback
   polynomial and DER calls inside every image definition to plain native
   bodies/slots, and inspect both roots and definitions for escaping callbacks.
   Reuse native alias registration; preserve all pieces and additional orders.
   Compare against the same three independent oracles. This route answers
   whether callbacks help once the previously omitted transformation is held
   fixed. It is not needed to establish the simpler original-template route.

Each generation, builder and cold numeric stage has an independently declared
180-second/30-GiB bound, with no dynamic extension after failure. A timed-out
generation does not proceed to builder or numeric stages. The frozen runner,
source/dependency hashes, exact input hashes, nonempty order assertions,
unchanged-input checks and all failed stages are retained. Symbolica execution
waits for explicit runtime handoff and independent source review.

This proposal does not solve native series growth: the original finite template
coefficient is itself 36.9 MB. It tests whether preserving already-owned image
definitions avoids the subsequent 256 MB Atom restoration and makes a complete
native evaluator feasible. If successful, full on-shell generation and
long-lived HEPKit ownership still require separate evidence. The original
template route creates no callback hooks; the callback route retains the
documented process-global strong-Arc lifetime limitation and cannot be promoted
to production on the strength of an isolated-process success.

## Implementation and first control result

After independent proposal review, the ignored proof was implemented in
`output/probes/template_alias.rs` and `template_alias/controls.rs`. It shares the
previous native numeric/cold-reader checks and registers the complete image map
once. The first frozen run, `output/diagnostics/formal-functions/template-alias-1`,
exported all eight case/policy IR files before a test-only exact-zero control
failed the shared strict absolute-bound assertion. It did not finish the scope
control or produce a completed nine-record report, so it is not counted as a
passing proof.

A separate native bounds probe records exact zero with absolute/relative cutoff
0 and no terms; `sqrt(epsilon)` has absolute cutoff 2, relative cutoff 3/2 and
its expected exponent 1/2. Source inspection explains the former:
`poly/series.rs:500` sets `order = 0` when absolute truncation leaves no stored
coefficients. The revised control knows its input is structurally `Atom::Zero`
and checks native zero/empty terms directly. It does not infer an all-order zero
from an arbitrary empty truncated series. All nonempty complete-vector bounds
remain unchanged, and the fractional input still reaches typed rejection under
a strictly covering native bound. The first failure and bounds output are
retained. The separately frozen second attempt passed its small writer in
1.226 seconds and its fresh-process native-IR reader in 0.870 seconds. All nine
records are present: four explicit complete Laurent vectors under aliases and
Never, plus the native global/local-scope control. The complex-image and exact
IR constant checks pass. Independent HEPKit review verified these outcomes.

## Saved original coefficient lowering: complete numerical proof

`output/diagnostics/formal-functions/template-alias-2` retains the frozen source,
executable, included dependency hashes, all imported image/coefficient inputs,
reviewed cancellation metadata and three oracle reports. Its original aliases
builder process completed in 5.906 seconds, with peak waited-child RSS
293,276 KiB. Import, provenance checks, exact template restoration and six
relative-versus-absolute native coefficient equalities took 5.757 seconds. The
native direct builder itself took 0.123 seconds; exact IR serialization took
0.004 seconds and produced 1,523,445 bytes. The 201 image bodies still total
21,513 Atom bytes. Native operation counts are 41,878 additions, 126,368
multiplications, 64 inversions and 30 function calls. This is cached coefficient
lowering; it excludes fresh epsilon series and original graph generation.

All three separate cold numeric processes passed every order −5 through 0:
18 independent oracle comparisons, each using exact rational coordinates,
512/1024-bit native MPFR agreement and the unchanged `1e-70 * max(1, scale)`
comparison tolerance. Process times were 4.421, 4.432 and 4.453 seconds. Native
IR load, O2 compilation/evaluation, cloned numeric worker, native error tracking
and scale-before-conversion weighting are retained in each report. The reader
does not install alias definitions or callbacks. Original capture/image/source
hashes and the oracle association are checked; multiplicity two is not applied.

Raw eager and O2 outputs agree, but native tracking is **unstable at the recorded
production tolerance for all three points**, and every point requests a
cancellation check. For example, the raw finite coefficient at the interior
point differs from the MPFR value by about `1.54e-6`; the simultaneous-boundary
point has catastrophic binary64 cancellation. These values remain in the raw
reports. The evidence establishes the shared exact IR and native MPFR path,
not binary64 accuracy or a relaxed rescue policy. Independent HEPKit outcome
review confirms all nine small records, all 18 actual coefficient comparisons
and the frozen hashes.

The next separately frozen stage,
`output/diagnostics/formal-functions/template-alias-fresh-1`, rebuilds the exact
late template from the captured original post-subtraction expression, invokes
native relative series under the unchanged 180-second/30-GiB bound and requires
exact equality of all six new coefficient Atoms to the validated saved inputs.
Its source has independent approval. The bounded process completed successfully
in 42.539 seconds with peak waited-child RSS 547,900 KiB. Provenance/import
checks took 0.785 seconds, late-template construction 0.408 seconds and native
relative series 38.389 seconds. The first relative-depth-one result had leading
exponent −5 and absolute bound −4; the native depth-six request returned final
absolute bound 1. All six new coefficient Atoms exactly equal the previously
validated saved inputs; the equality check took 2.398 seconds. The same 201
image definitions and original template identity also passed exact checks.

This closes the fresh Laurent-substage link to all three independent numerical
oracles without constructing the giant restored coefficients. It does not
rerun graph generation, mapping or subtraction, and is not a controlled
performance comparison or whole-integral convergence result. No production
path, persistence schema or precision policy has changed. An independently
reviewed modular production integration proposal is the next step; the callback
variant and optional Never comparison are deferred.
