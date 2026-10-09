# Contour prerequisite pull requests

2026-10-09. The user authorized publishing narrow dependency fixes and requesting
`benruijl` as reviewer. The publishing account was verified as `ValentinHirschi`;
commits use `ValentinHirschi <valentin.hirschi@gmail.com>`.

## SymJIT complex scalar callback lanes

[siravan/symjit#14](https://github.com/siravan/symjit/pull/14) targets the upstream
`v227` branch from the existing `ValentinHirschi/symjit_changes_for_pyamplicol`
fork. Its head is `24017d4f84010716263d3cbc0e7aba2d48b63fe6`. It contains only
the complex gather/scatter correction and regression in `rust/defuns.rs`.
The standalone published-crate reproduction and retained patch remain in this
repository. Full owner tests passed: 2,148 passed, one ignored, on Linux x86_64.

GitHub rejected formal reviewer assignment through both GraphQL and REST for
this account. The requested review invitation to `@benruijl` was posted as
[a PR comment](https://github.com/siravan/symjit/pull/14#issuecomment-6088149269).
It is not recorded as a successful formal reviewer assignment.

The Rust crate's published packaging differs from the upstream Python/C-ABI
repository layout. A standalone Cargo probe fails to import the original Git
crate because its manifest builds only a `cdylib`.

[siravan/symjit#15](https://github.com/siravan/symjit/pull/15) adds an `rlib`
alongside that output and reexports existing native composer API items from
the unchanged C entrypoint. Its head is
`6a49a5b0a95cc12aa3d9ddb2b46c32cca193a54b`, independently based on `v227`.
The external Rust consumer now passes a complex evaluation and version query;
2,147 owner unit tests pass (one ignored), plus the external-consumer test.
An independent runtime-agent review found no blocker. Formal reviewer
assignment was again denied; [the review invitation](https://github.com/siravan/symjit/pull/15#issuecomment-6088355659)
names `@benruijl`. A combined public consumer branch will carry both PRs before
replacing the temporary local override.

## Symbolica and Numerica

- [Symbolica #55](https://github.com/symbolica-dev/symbolica/pull/55) adds native
  real/complex ball evaluator domains. It targets current `main`, head
  `d9dbf4a0f528d5a5a2ac3a9d0cca6b7c2ade22ec`. The full owner library and three
  focused tests pass on that base. Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/55#issuecomment-6088228662)
  names `@benruijl`.
- [Symbolica #56](https://github.com/symbolica-dev/symbolica/pull/56) adds the
  prepared bracketed root solver on a separate current-`main` branch, head
  `bf61144d3dba718ee0a7b9f64e5ffa363459dac4`. The full owner library, eight
  scalar/failure tests and two prepared eager/JIT evaluator tests pass.
  Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/56#issuecomment-6088260343)
  names `@benruijl`.
- [Numerica/Symbolica #57](https://github.com/symbolica-dev/symbolica/pull/57)
  corrects tracked `hypot` in its Numerica owner crate, head
  `1dc4566cc4591f16c56b565bc907ff86e882aba9`. Six focused native and six
  portable-host tests pass, as do 14 owner API and 15 complex regressions.
  Formal reviewer assignment was denied;
  [the review invitation](https://github.com/symbolica-dev/symbolica/pull/57#issuecomment-6088302905)
  names `@benruijl`.

All PRs are authored and published by ValentinHirschi and attached to the task.
The tracked scalar forwarding candidate was not adopted: enabling ordering
also activated complex square-root/division shortcuts that lost uncertainty.
A narrower tracked `hypot` owner fix is published without changing global
branch semantics. No unreviewed correction has been adopted by
FastSecDec or the isolated fixed-mode HEPKit wheel.
