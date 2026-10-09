# SymJIT MRE: complex callback arguments mix SIMD sample lanes

This standalone Rust script requires only the published SymJIT 2.27.0 crate.
It uses no Symbolica, FastSecDec, license, Python or local inputs.
The exact dependency is pinned here solely to reproduce the affected release;
FastSecDec's supported dependency remains a minimum-version requirement.

```sh
rust-script --debug reproduce.rs
```

The script must fail on unpatched SymJIT 2.27.0. It evaluates a two-argument
complex callback `a*a + (2-3i)*b` with distinct real/imaginary data in every row.
One to three rows agree with direct arithmetic on the tested x86_64 machine.
At four rows the implicit SIMD path mixes different samples' real lanes with
imaginary lanes and returns wrong values. For the first row the expected value
is `0.5 - 4.375i`, while the affected implementation returns `-4.25 - 6.625i`.
The same source defect and failure were reproduced in 2.26.4.

[The owner patch](../../docs/dependency-patches/symjit-complex-callback-lanes.patch)
applies to `rust/defuns.rs` in the upstream GitHub repository's `v227` branch.
For the packaged Rust crate the corresponding file is `src/symjit/defuns.rs`.
It gathers matching real/imaginary SIMD lanes into each scalar-complex call,
then scatters results back into the native split layout. It does not disable
SIMD, change the callback's mathematics, or add a FastSecDec fallback.

The regression covers rows 1, 2, 3, 4, 5, 8, 17, 255, 256 and 257, both direct
and ordinary translation, and threading enabled/disabled. The exact dyadic
inputs make strict equality meaningful. All cases pass with the owner patch.
An additional Symbolica contour-callback probe checks the original higher-level
failure; the standalone reproduction deliberately isolates the smaller owner bug.

Verified 2026-10-09 on Linux x86_64. AArch64/RISC-V execution is not claimed.
SymJIT 2.27.0 was the newest non-yanked release in the
[official sparse index](https://index.crates.io/sy/mj/symjit), published
2026-10-08. Its archive SHA-256 is
`f06329022a5123536f7fd50eb01b3e885dfafe3983f90a0d7b17408f4d0469ea`.
