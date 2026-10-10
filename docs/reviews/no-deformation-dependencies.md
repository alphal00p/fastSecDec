# Native threshold dependency boundary

2026-10-10. This accepts dependency packaging and native API prerequisites,
not complete threshold generation or general algebraic resolution.

## Direct symGCAD dependency

[symGCAD PR #1](https://github.com/alphal00p/symGCAD/pull/1), revision
`a1132d4f4545c7784b3ec61239a05485e544b403`, replaces its private Symbolica path
with a registry minimum and public standalone pins. Solver Rust sources are
unchanged. The consuming workspace owns the Symbolica identity; symGCAD's
standalone patches do not propagate into FastSecDec. No dependency preparation
script or private checkout is required for the library build.

The standalone owner gate passed 225 tests, with one existing ignore. An actual
downstream consumer passed seven groups covering shared native types, signed
factor orientation, parameter-first geometry, algebraic root selectors and
refinement, certificate mutation, incomplete solves and cache reuse. The owner
format check reports three unchanged baseline files; no unrelated formatting
was included in this packaging patch.

FastSecDec's optional native `threshold-decomposition` feature resolves exactly
one Symbolica/Numerica identity, `74225696`, and registry Graphica 3.0.1.
The integrated adapter passes eight tests and strict core-library Clippy with
the feature enabled. Workspace formatting passes. Existing contour/artifact
and streaming controls pass 29 and 16 tests respectively.
Metadata for the standalone portable and Python consumers excludes symGCAD;
their lockfiles are unchanged. These metadata checks are dependency evidence,
not fresh browser or Python execution.

The PR and commit belong to `ValentinHirschi`; GitHub confirms the formal
`benruijl` reviewer request. The PR is attached to this task. The independent
[geometry adapter review](no-deformation-gcad-audit.md) covers the subsequent
FastSecDec integration.

## Native fractional-series prerequisite

[Symbolica PR #66](https://github.com/symbolica-dev/symbolica/pull/66), revision
`da8e234bc33c20c372859e7bb19adf7577123be7`, fixes native Puiseux slot scaling.
Fractional requested precision exposed wrong ramification, monomial shifts and
rational-power precision. The failing FastSecDec probe was retained; no
replacement series arithmetic was introduced.

The exact upstream `community` baseline fails eight of ten focused public
controls. The patch passes all ten and 26 existing series/derivative controls,
with passing focused strict lint, formatting and independent source review.
These owner checks are separate from adopting a combined FastSecDec consumer
revision and replaying its maintained algebra probes.

The public combined child `c540d3f68c90fe7bff1e507458e57a20fb95b11c` adds only
this two-file fix to `74225696`. It passes the ten owner controls and all four
unchanged maintained FastSecDec algebra probes when linked against that exact
owner. These include fractional series, polynomial blow-up identities,
resultants versus shifted norms, and symbolic moving-root derivatives with
native evaluator composition. The moving-root test agrees with its explicit
control to a maximum scaled difference of `2.1e-15`; it does not implement a
generic implicit-root solver or general collision detector.

All three consuming manifests and lockfiles now select that combined revision.
Only the Symbolica/Numerica source identities changed in each lockfile. Their
Cargo-built portable consumer passes all 88 tests; the Python binding and stub
build passes with an explicit Nix Python interpreter. These are separate from
the direct native-owner probe evidence above. The combined native core gate is
recorded with the projective, phase and regulator milestone in
[the reuse audit](../REUSE_AUDIT.md): 477 tests pass with 20 existing ignores.

Publication uses `ValentinHirschi`. GitHub rejected formal reviewer assignment;
the authorized [review invitation to benruijl](https://github.com/symbolica-dev/symbolica/pull/66#issuecomment-6101430131)
records the request without claiming formal assignment. The PR is attached.

## Zero-generator ideal admission

The native resolution probes also expose a constructor robustness issue:
`GroebnerBasis::new` attempts monic normalization of a zero generator and divides
by zero. FastSecDec's supplied-ideal admission removes exact native zeros, so
this does not require another consumer dependency change. A separate narrow
owner patch retains the unified variable map, drops zeros and uses the existing
empty-ideal return. Four public controls fail before the patch and pass after
it; strict focused lint, formatting and independent review pass.
[Symbolica PR #67](https://github.com/symbolica-dev/symbolica/pull/67), commit
`a22686a`, is published as ValentinHirschi and attached to this task. GitHub
declined formal reviewer assignment; the requested review is recorded by
tagging `benruijl` in the PR. This optional owner robustness fix is not part of
FastSecDec's pinned consumer revision.

## Limits and reproduction evidence

Raw command logs, source hashes and process receipts stay untracked under
`target/no-deformation-gates/symgcad-dependencies` and the native probe targets.
Tiny solver timings are not physical integration benchmarks. Compile-time RSS
is not a proof of bounded generation residency. Generic open-cell verification
does not certify closed-face normal forms, auxiliary-regulator removal, or
validity at excluded parameter values.
