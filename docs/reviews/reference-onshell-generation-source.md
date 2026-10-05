# Original on-shell reference generation route

This is a read-only command/cache audit. No reference generation is launched by
this note. The frozen Pathfinder checkout is
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder`, revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`.

From that checkout, the generation-only command is:

```sh
.venv/bin/python FSD.py generate \
  --run examples/runs/dot_triple_box.yaml \
  --output NEW_ABSOLUTE_BUNDLE \
  --explicit --jit-compile --jit-optimization-level 2 --complex-evaluator \
  --workers 1 --max-eps-order 0 \
  --sampling-mode qmc --qmc-support-mode full --no-qmc-optimized-evaluators \
  --quiet-summary --no-progress --json
```

The original ten-propagator graph and on-shell kinematics are unchanged:
`s12=s23=-1`, with all external squared momenta zero. The card SHA256 is
`0cbcd34139add9806bf7aa8dc53e3484439feb034f59ab1e273570ee289e1133`,
graph `493d8a579248106ed8cb9c2404d1033121f68ab079373552d1234b0a06b078eb`,
and kinematics `0cf6a4570b9084cf3743753a64b5e56d49f1cbce3c2383b6e486c4474bbf2faa`.
The shipped projector/IBP policy, direct-projector threshold zero, derivative
settings and formula signature/output-length limits remain unchanged. The
command explicitly uses the corrected complex O2 evaluator route; the earlier
real O2 smoke failure is not an accepted baseline.

`FSD.py:297–313` validates generation without requiring a target file.
`FSD.py:3003–3009` activates generation metadata and streaming evaluator storage;
`FSD.py:3081–3114` saves the complete prepared bundle and returns before numerical
integration. The requested highest signed order is zero. Actual minimum order,
physical prefactor convention and complete bundle outputs must be read from the
returned manifest; neither order count nor numerical precision is inferred here.
The sampling flags fix prepared output layout, not a QMC integration run.

A proposed bounded readiness attempt uses the existing external process-group
watchdog, one allowed CPU, 600 seconds and 30 GiB simultaneous process-tree RSS,
with its existing five-second interrupt and kill grace periods. The coordinator
must approve its concrete fresh output and command before launch. This is a
readiness diagnostic, not matched generation timing. No concurrent Symbolica
instance or extra worker is authorized. Preserve the complete command, source
and environment hashes, raw stdout/stderr, partial bundle and terminal process
outcome. Do not silently restart or extend a failed attempt.

## Available formula cache

The actual read-only inventory is
`output/reference-onshell-cache-inventory.sha256`, SHA256
`f719d8de1a24d73dda4429ffa4f0dfcb770c37cc1da2118182426d934960da84`.
The primary `cache/subtraction_formulae` contains two generated JSON files and
4,976 logical bytes. The tracked `assets/subtraction_formulae` tree contains
498 files and 50,840,247 logical bytes: 488 endpoint-projector JSON records, six
regular-Taylor JSON records, two READMEs and two placeholders. The tracked tree
has no working-tree changes. No separate distributed `FSD_cache` archive was
found in this checkout. The curated records contain native formula expressions;
their presence is not proof that every required heavy signature is cached or
that compatible compiled evaluator sidecars exist.

`src/cache_utils.py:40–60` makes `FSD_SUBTRACTION_FORMULA_CACHE_DIR` the write
location while still reading the default primary and legacy asset roots.
`subtraction_formula.py:1003–1014` checks curated and ordinary entries in those
read roots. Therefore a fresh override alone does not define a cold cache.
Before an attempt, freeze/hash both existing read roots, preserve them unchanged,
and direct any new writes/mirrored hits to a fresh attempt-owned cache. Record
that the existing formula inputs are available and retain the generated cache
delta afterward. A subsequent cold-cache experiment would require a separately
defined environment and is not implied by this command.
