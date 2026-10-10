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

Publication uses `ValentinHirschi`. GitHub rejected formal reviewer assignment;
the authorized [review invitation to benruijl](https://github.com/symbolica-dev/symbolica/pull/66#issuecomment-6101430131)
records the request without claiming formal assignment. The PR is attached.

## Limits and reproduction evidence

Raw command logs, source hashes and process receipts stay untracked under
`target/no-deformation-gates/symgcad-dependencies` and the native probe targets.
Tiny solver timings are not physical integration benchmarks. Compile-time RSS
is not a proof of bounded generation residency. Generic open-cell verification
does not certify closed-face normal forms, auxiliary-regulator removal, or
validity at excluded parameter values.
