# Symbolica MRE: an underscore-suffixed Series variable becomes a wildcard

This folder is standalone. `reproduce.rs` depends only on the **unpatched
published Symbolica 3.0.1** through its embedded Cargo manifest; it needs no
FastSecDec checkout, graph data, Python, or local dependency paths.

## Reproduce

Use Rust **1.96 or newer**, the native build tools required by Symbolica's
GMP/MPFR backends, and [rust-script](https://rust-script.org/):

```sh
cargo install rust-script --version 0.36.0 --locked
rust-script --debug reproduce.rs
```

The first invocation builds its pinned dependency. The script intentionally
exits nonzero when the bug is present. Do not add a local Cargo patch while
claiming to reproduce the published failure. `rust-script --package reproduce.rs`
prints the generated Cargo-package path for inspection without running it.

Expected Gamma expansions through order zero are:

```text
Gamma(eps)  = 1/eps  - EulerGamma
Gamma(eps_) = 1/eps_ - EulerGamma
```

Both must contain **two nonzero terms**. On the verified unpatched published
release, `Gamma(eps_)` instead becomes zero with no terms. The script prints
both cases, checks the ordinary-name control against the analytic answer, and
asserts nonempty/equal results. It also compares the first-order native series
of the undefined composed function `f(1+eps,sin(eps))` after a literal variable
rename. It cannot pass by comparing two empty vectors.

## Cause and minimal fix

Generic function-series fallback in `src/derivative.rs` uses:

```rust
a.replace(x.clone()).with(expansion_point.to_owned())
d.replace(x.clone()).with(expansion_point.to_owned())
```

Here `x` is an `Indeterminate`. Its conversion to `Pattern` interprets a symbol
ending in `_` as a wildcard. The replacement therefore matches the whole
function expression. Both substitutions should instead use literal native
Atoms for the variable and expansion point:

```rust
.replace(Pattern::Literal(Atom::from(x.clone())))
.with(Pattern::Literal(expansion_point.to_owned()))
```

`fix.patch` is the reviewed isolated patch, including an upstream-style native
regression for Gamma and an unknown composed function at expansion points zero
and one. It changes no derivative, Gamma-regularization or Series arithmetic.

## Versions and validation

- Published dependency: `symbolica = "=3.0.1"`; crate SHA-256
  `0965398bbe5063c00b3a0d078d7d3dc57b61819b97e960b73e9a7e9602662795`.
  Its packaged `.cargo_vcs_info.json` identifies upstream commit
  `e9a0d35490a834afb2fa40d97c8c0b1e58ef5a5e`.
  Its source contains the same two faulty substitutions at lines 514 and 534.
- Original executable reproduction: upstream revision
  `98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, version 3.0.1 plus six upstream
  commits, with unrelated evaluator/parser/printer fixes. Ordinary Gamma gave
  two terms; underscore-suffixed Gamma gave no terms.
- Fixed checkout: the same revision plus `fix.patch`. The upstream regression
  and an independent complete-vector `Gamma(eps_)/eps_` regression both pass.
- **This script was executed against the actual published 3.0.1 crate:**
  `rust-script 0.36.0 --package` generated its standalone Cargo package, which
  was built without workspace overrides. Cargo.lock identifies the registry
  crate and the checksum above. Execution returned **101**, with the outputs
  below and the explicit two-term assertion failure.
- **The unchanged script was then executed against the corrected checkout:**
  it returned **0**, preserving both Gamma terms and the composed-function
  series. This second run linked the corrected native library explicitly; it
  is not represented as a patched published-release build. The fixed library
  SHA-256 was
  `dda215a39b38358772677beed211e773c9b028f33f0251dc1072135fe37f2e51`.

The failing release printed:

```text
Gamma(eps):  -γ+1/eps (2 terms)
Gamma(eps_): 0 (0 terms)
Expected Gamma(eps_): -γ+1/eps_
f(1+eps,sin(eps)):   eps*(der(0,1,f,1,0)+der(1,0,f,1,0))+f(1,0)
f(1+eps_,sin(eps_)): 0
```

After correction, both underscore-suffixed results equal the literal-renamed
ordinary-name results and the script prints `PASS`. Validation used Rust
1.98.1 on Linux; the declared minimum is 1.96. Source, patch and library hashes
and the two outcome records are included in `validation.json`; concise output
and exit statuses are in `observed-output.txt`. `git apply --check --verbose`
also verified `fix.patch` against a fresh unpacking of the published 3.0.1
crate outside any parent Git checkout. No generated
Cargo project, binary or dependency checkout is required in this folder.

To test an upstream checkout without changing this script's algorithm, change
only its embedded dependency to an absolute `path` pointing at that checkout,
with the same feature selection. Apply the patch from that checkout's root:

```sh
git apply --check /path/to/this-folder/fix.patch
git apply /path/to/this-folder/fix.patch
```

Then rerun with `rust-script --debug --force reproduce.rs`. Keep those runs
distinct from the published-release result. The default
script never selects a patched local checkout implicitly.

Symbolica may display its normal licence banner. This small serial reproducer
does not require parallel Symbolica execution, and no licence key or licence
setting is included. It does not contact an upstream maintainer automatically.
